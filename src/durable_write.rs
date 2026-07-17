//! Crash-safe replacement of compiler-owned files.
//!
//! Compiler-owned persistent state — `muga.lock`, `.mgi`, `.mgb`, `.mgc`,
//! `.mgp`, `.mga`, bundle metadata, launchers, and installation ownership
//! metadata — must never be replaced in place, because a write interrupted
//! halfway leaves a truncated file where the last valid output used to be.
//! See spec/006-packages.md section 17.11 for the contract this implements.
//!
//! The guarantee is per file: at any point, the destination holds either its
//! previous complete contents or the complete new contents, and a command
//! that reports success has made the new contents durable. Callers that write
//! several files still own their own commit boundary; one atomic file
//! replacement is not a transaction over an artifact set.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

/// Distinguishes temporary names created concurrently inside one process.
static TEMPORARY_SERIAL: AtomicU64 = AtomicU64::new(0);

/// Bounds the search for an unused temporary name so a hostile or broken
/// directory cannot spin here forever.
const TEMPORARY_NAME_ATTEMPTS: u32 = 32;

/// Replaces `path` with `contents`, atomically and durably.
///
/// The caller is expected to have serialized and validated `contents`
/// already: this function touches the destination only after the new bytes
/// are safely on disk. On failure the destination keeps its previous
/// contents and no temporary file is left behind.
pub fn replace_file(path: &Path, contents: impl AsRef<[u8]>) -> io::Result<()> {
    let parent = destination_parent(path);
    let (temporary_path, file) = create_temporary_sibling(path, &parent)?;
    match write_temporary_then_replace(file, &temporary_path, path, &parent, contents.as_ref()) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(&temporary_path);
            Err(error)
        }
    }
}

fn write_temporary_then_replace(
    mut file: File,
    temporary_path: &Path,
    path: &Path,
    parent: &Path,
    contents: &[u8],
) -> io::Result<()> {
    file.write_all(contents)?;
    file.sync_all()?;
    drop(file);
    fs::rename(temporary_path, path)?;
    sync_directory(parent)
}

fn create_temporary_sibling(path: &Path, parent: &Path) -> io::Result<(PathBuf, File)> {
    let Some(file_name) = path.file_name() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("`{}` has no file name to replace", path.display()),
        ));
    };
    for _ in 0..TEMPORARY_NAME_ATTEMPTS {
        let mut temporary_name = OsString::from(".");
        temporary_name.push(file_name);
        temporary_name.push(format!(
            ".tmp-{}-{}",
            process::id(),
            TEMPORARY_SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        let temporary_path = parent.join(temporary_name);
        // `create_new` refuses to open an existing entry, so a leftover
        // temporary file or a symlink planted at this name is an error
        // rather than something written through.
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
        {
            Ok(file) => return Ok((temporary_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!(
            "could not create a unique temporary file next to `{}`",
            path.display()
        ),
    ))
}

fn destination_parent(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

/// Makes the rename itself durable. Without this a crash can lose the
/// directory entry even though the file contents were flushed.
#[cfg(unix)]
fn sync_directory(parent: &Path) -> io::Result<()> {
    File::open(parent)?.sync_all()
}

/// Windows offers no portable directory flush through `std::fs`, and
/// `MoveFileEx`-backed renames are ordered against the flushed file
/// contents, so the file flush above carries the guarantee.
#[cfg(not(unix))]
fn sync_directory(_parent: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::replace_file;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn temp_root(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("muga-durable-write-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("temp root should be created");
        root
    }

    fn directory_entry_names(root: &Path) -> Vec<String> {
        let mut names = fs::read_dir(root)
            .expect("temp root should be readable")
            .map(|entry| {
                entry
                    .expect("temp root entry should be readable")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    #[test]
    fn replace_file_creates_a_missing_destination() {
        let root = temp_root("create");
        let path = root.join("artifact.mgi");

        replace_file(&path, "created").expect("missing destination should be created");

        assert_eq!(fs::read_to_string(&path).unwrap(), "created");
        assert_eq!(directory_entry_names(&root), vec!["artifact.mgi"]);
    }

    #[test]
    fn replace_file_overwrites_an_existing_destination_without_leaving_temporaries() {
        let root = temp_root("overwrite");
        let path = root.join("muga.lock");
        fs::write(&path, "previous").expect("existing destination should be written");

        replace_file(&path, "next").expect("existing destination should be replaced");

        assert_eq!(fs::read_to_string(&path).unwrap(), "next");
        assert_eq!(directory_entry_names(&root), vec!["muga.lock"]);
    }

    #[test]
    fn replace_file_writes_arbitrary_bytes() {
        let root = temp_root("bytes");
        let path = root.join("package.mgp");
        let bytes: Vec<u8> = (0..=255u8).collect();

        replace_file(&path, &bytes).expect("archive bytes should be written");

        assert_eq!(fs::read(&path).unwrap(), bytes);
    }

    #[test]
    fn replace_file_keeps_the_previous_contents_when_the_temporary_cannot_be_created() {
        let root = temp_root("failure");
        let path = root.join("muga.lock");
        fs::write(&path, "previous").expect("existing destination should be written");
        let unwritable = root.join("missing-directory").join("muga.lock");

        replace_file(&unwritable, "next")
            .expect_err("a destination inside a missing directory should fail");

        assert_eq!(fs::read_to_string(&path).unwrap(), "previous");
        assert_eq!(directory_entry_names(&root), vec!["muga.lock"]);
    }

    #[test]
    fn replace_file_removes_its_temporary_file_when_the_replacement_fails() {
        let root = temp_root("replacement-failure");
        let path = root.join("muga.lock");
        // A directory at the destination lets the temporary file be created
        // and written, then fails the replacement itself.
        fs::create_dir_all(&path).expect("destination directory should be created");

        replace_file(&path, "next").expect_err("a directory destination should not be replaced");

        assert!(path.is_dir());
        assert_eq!(directory_entry_names(&root), vec!["muga.lock"]);
    }

    #[test]
    fn replace_file_rejects_a_destination_without_a_file_name() {
        let root = temp_root("no-file-name");

        let error = replace_file(Path::new(".."), "next")
            .expect_err("a destination without a file name should fail");

        assert!(error.to_string().contains("no file name to replace"));
        assert!(directory_entry_names(&root).is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn replace_file_replaces_a_symlinked_destination_without_writing_through_it() {
        let root = temp_root("symlink");
        let target = root.join("target");
        let path = root.join("muga.lock");
        fs::write(&target, "target contents").expect("symlink target should be written");
        std::os::unix::fs::symlink(&target, &path).expect("symlink should be created");

        replace_file(&path, "next").expect("symlinked destination should be replaced");

        assert_eq!(fs::read_to_string(&path).unwrap(), "next");
        assert_eq!(fs::read_to_string(&target).unwrap(), "target contents");
        assert!(
            !fs::symlink_metadata(&path)
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }
}
