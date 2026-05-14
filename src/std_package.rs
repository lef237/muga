pub const IO_PACKAGE: &str = "std::io";
pub const FS_PACKAGE: &str = "std::fs";
pub const IO_ERROR_MANGLED_NAME: &str = "__muga_pkg__std__io__IOError";
pub const IO_ERROR_VISIBLE_NAME_IN_FS: &str = "io::IOError";
pub const FS_READ_TEXT_BUILTIN: &str = "__muga_std_fs_read_text";
pub const FS_WRITE_TEXT_BUILTIN: &str = "__muga_std_fs_write_text";

#[derive(Clone, Copy, Debug)]
pub struct VirtualPackageFile {
    pub module_path: &'static str,
    pub source: &'static str,
}

pub fn virtual_package_files(package_path: &str) -> Option<&'static [VirtualPackageFile]> {
    match package_path {
        IO_PACKAGE => Some(IO_FILES),
        FS_PACKAGE => Some(FS_FILES),
        _ => None,
    }
}

pub fn allows_internal_builtins(package_path: &str) -> bool {
    package_path == FS_PACKAGE
}

const IO_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "io.muga",
    source: r#"
package std::io

pub record IOError {
  operation: String
  path: String
  kind: String
  message: String
  raw_code: Option[Int]
}
"#,
}];

const FS_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "fs.muga",
    source: r#"
package std::fs

import std::io

pub fn read_text(path: String): Result[String, io::IOError] {
  __muga_std_fs_read_text(path)
}

pub fn write_text(path: String, text: String): Result[Unit, io::IOError] {
  __muga_std_fs_write_text(path, text)
}
"#,
}];
