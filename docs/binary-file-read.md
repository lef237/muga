# Binary File Read

Status: read-only binary file reads are implemented through
`std::fs::read_bytes(path)` and `std::fs::read_bytes_path(path::Path)`, returning
opaque `std::bytes::Bytes`.

This slice made `Bytes` useful outside package resources without adding binary
writes, mutable buffers, codecs, streams, or file-handle modes. The follow-up
[bytes-sha256-hash.md](bytes-sha256-hash.md) adds one digest helper without
opening broad cryptographic APIs, and the later
[binary-file-write.md](binary-file-write.md) slice adds full-file binary writes
without adding buffers, codecs, or streams.

## Goals

Short-Term Goal: let CLI tools and small apps inspect local binary files with
the same recoverable error model as `fs::read_text`.

Medium-Term Goal: keep string paths and `path::Path` wrappers aligned across
text and binary full-file reads.

Long-Term Goal: leave binary writes to a separate narrow slice, and leave
chunked IO, encodings, broad hashing, and network payloads to later slices that
have concrete callers.

Final Goal: make Muga practical for file-oriented tools while preserving the
small, explicit standard-library surface.

## Implemented Contract

`std::fs` exports:

```txt
pub fn read_bytes(path: String): Result[bytes::Bytes, io::IOError]
pub fn read_bytes_path(file_path: path::Path): Result[bytes::Bytes, io::IOError]
```

`std::bytes` exports the first indexing helper:

```txt
pub fn at(bytes: Bytes, index: Int): Option[Int]
```

`bytes::at` uses zero-based indexing. It returns `Option::Some(value)` for byte
values in the inclusive range `0..255`, and `Option::None` for negative or
out-of-range indexes.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| `fs::read_bytes(path)` and `fs::read_bytes_path(path::Path)` | Mirrors the existing text full-file read API and gives immediate CLI utility. | Loads the whole file into memory. | Select |
| `bytes::at(bytes, index): Option[Int]` | Lets programs inspect headers and small payloads without a `Byte` type. | Uses `Int` for byte values until a dedicated scalar proves necessary. | Select |
| Binary `File` handles | Enables large-file streaming. | Requires mode, cursor, buffering, and cleanup policy beyond this slice. | Defer |
| `hash::sha256_hex(bytes)` | Useful for local verification tools and matches Muga archive hash internals. | Digest-specific rather than a general bytes formatting API. | Done in the follow-up slice |
| `bytes::to_hex` or base64 helpers | Useful for debugging and text transport. | Commits to encoding APIs before concrete package/service callers need them. | Defer |
| Byte literals or builders | Enables in-language byte construction. | Requires syntax or mutable buffer design. | Defer |

## Non-Goals

This slice does not add:

- binary writes or append APIs in this read-only slice;
- binary `File` handles, streams, or async IO;
- byte literals, mutable buffers, or builders;
- hex/base64/UTF-8 codecs;
- broad hashing, checksums, or compression.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `standard_fs_read_bytes_reads_file_and_indexes_bytes_for_source_and_built_runs`

Release-readiness coverage keeps this document, `std::bytes`, `std::fs`,
builtin typing, runtime behavior, specs, and public docs aligned.
