# Filesystem Directory Size Metadata

Status: `std::fs::DirectorySizeMetadata` and
`std::fs::directory_size_metadata_path(root_path)` are implemented as a
read-only recursive directory aggregate over `path::Path` values.

This slice builds on deterministic recursive traversal without adding
destructive filesystem behavior to the aggregate API, globbing, public
symlink classification, or sandbox policy.

## Goals

Short-term: let package/report/cache tools ask for total regular-file bytes and
entry counts for a directory tree through one typed `Result` value.

Medium-term: use the same aggregate in source-free bundle validation,
resource-audit reports, installer preflights, and generated project helpers
without requiring every caller to reimplement recursion.

Long-term: keep this read-only aggregate as a small foundation before adding
directory copy, recursive removal, tree synchronization, filesystem watches,
or policy-rich sandbox APIs.

Final goal: move Muga closer to practical adoption by supporting real
file-processing tools while keeping filesystem behavior explicit, deterministic,
and reviewable.

## Public Shape

```muga
pub record DirectorySizeMetadata {
  size: Int
  file_count: Int
  directory_count: Int
  other_count: Int
}

pub fn directory_size_metadata_path(root_path: path::Path): Result[DirectorySizeMetadata, io::IOError]
```

`size` is the total byte length of regular descendant files. `file_count`,
`directory_count`, and `other_count` count descendants of `root_path`; the root
directory itself is not counted.

The public `std::fs` wrapper calls a compiler-provided runtime helper:

```muga
pub fn directory_size_metadata_path(root_path: path::Path): Result[DirectorySizeMetadata, io::IOError] {
  __muga_std_fs_directory_size_metadata(path::as_string(root_path))
}
```

The runtime traversal sorts each directory's direct children before counting
them. It recurses only into entries whose directory-entry metadata reports a
directory, sums only entries whose directory-entry metadata reports a regular
file, and counts symlinks or other entry kinds in `other_count`.

Errors return the first `io::IOError` from reading the root, reading a child
directory, converting an entry path to UTF-8 text, inspecting an entry type,
reading file metadata, or converting aggregate counts and byte lengths to
`Int`. Runtime errors use `operation = "directory_size_metadata"` and the path
that caused the failure.

## Candidates Compared

| Candidate | Benefit | Cost | Decision |
|---|---|---|---|
| Add `DirectorySizeMetadata` with byte and count fields | Gives practical tree-size reporting while preserving explicit file/directory/other counts. | Needs a runtime helper so traversal, sorting, symlink treatment, and overflow handling stay coherent. | Select |
| Return only `Result[Int, io::IOError]` | Minimal public shape. | Hides file/directory/other counts that reports and preflights need, and makes symlink treatment harder to audit. | Reject |
| Compose `read_dir_recursive_path` with `path_size_metadata_path` in Muga source | Avoids a new builtin. | Would mix traversal `DirEntry` policy with later path metadata reads, including symlink-following differences and more race windows. | Reject |
| Add destructive recursive operations first | Useful for cleanup and installers. | Deletion/copy/move needs overwrite, partial-failure, ownership, and sandbox policy before it is safe. | Covered separately |

## Deferred Policy

- public symlink classification remains deferred; this helper counts symlinks
  as `other_count` and does not recurse through symlinked directories.
- partial-result reporting remains deferred; traversal stops at the first
  recoverable `io::IOError`.
- recursive removal is covered by `remove_dir_all_path`; no-overwrite
  directory copy is covered by `copy_dir_all_path`; copy-then-remove directory
  move is covered by `move_dir_all_path`; globbing, permissions, owner
  metadata, accessed/created timestamps, and sandbox containment remain separate
  slices.

## Validation

- `package_std_fs_directory_size_metadata_sample_runs`
- `standard_fs_directory_size_metadata_path_returns_public_record`
- `standard_fs_directory_size_metadata_path_missing_dir_returns_io_error`
- `standard_fs_directory_size_metadata_path_file_returns_io_error`
- `standard_fs_directory_size_metadata_path_counts_symlinks_as_other`
- `standard_fs_directory_size_metadata_path_type_mismatch_reports_expected_path`
- `standard_fs_directory_size_metadata_artifact_run_uses_emitted_std_implementations`
- `fs_directory_size_metadata_is_documented_and_covered`
