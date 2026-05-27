# Binary File Write

Status: `std::fs::write_bytes(path, data)` and
`std::fs::write_bytes_path(file_path, data)` are implemented as full-file binary
writes over opaque `std::bytes::Bytes`.

This slice closes the local/resource bytes round trip for file-oriented tools:
programs can read bytes, inspect or hash them, and write the same opaque payload
to a chosen file path. It deliberately avoids byte construction, mutation,
encoding codecs, append modes, binary file handles, streams, and async IO.

## Goals

Short-Term Goal: let CLI tools and generated workflows materialize binary data
that was already read from a local file or manifest-declared package resource.

Medium-Term Goal: keep text and binary full-file IO symmetrical across string
paths and `path::Path` values while preserving recoverable `io::IOError`
failures.

Long-Term Goal: leave mutable buffers, binary handle modes, chunked streaming,
codecs, compression, and broader resource pipelines to later slices with
concrete callers.

Final Goal: make ordinary local file workflows practical while keeping Muga's
standard library explicit, typed, and small.

## Implemented Contract

`std::fs` exports:

```txt
pub fn write_bytes(path: String, data: bytes::Bytes): Result[Unit, io::IOError]
pub fn write_bytes_path(file_path: path::Path, data: bytes::Bytes): Result[Unit, io::IOError]
```

Both helpers overwrite an existing destination when the host filesystem permits
it, return `Result::Ok(Unit)` on success, and return `io::IOError` with
`operation = "write_bytes"` on recoverable host failures such as a missing
parent directory or permission denial.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| `fs::write_bytes(path, data)` plus `fs::write_bytes_path(file_path, data)` | Mirrors existing text writes and binary reads; enables local file/resource byte materialization without new data types. | Full-file write only; callers must already have `Bytes`. | Select |
| Binary `File` handles | Supports large payloads and append/stream workflows. | Requires mode, cursor, buffering, cleanup, partial-write, and binary/text capability policy. | Defer |
| `bytes::from_list` or byte literals first | Lets programs construct new binary payloads in Muga. | Adds byte scalar/bounds policy and possible mutable-buffer pressure before round-trip IO needs it. | Defer |
| Encoding helpers such as hex/base64 | Useful for transport and debugging. | Orthogonal to writing opaque bytes and should be designed with broader codec rules. | Defer |
| Recursive directory materialization first | Useful for resource export tools. | Much larger filesystem mutation and partial-failure policy than one full-file write. | Defer |

## Non-Goals

This slice does not add:

- byte literals, mutable buffers, builders, or `Bytes` mutation;
- binary append APIs or binary `File` handles;
- streams, async IO, partial-write status, or backpressure;
- hex/base64/UTF-8 codecs, compression, or generic encoders;
- directory copy, recursive export, or sandbox containment policy.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `standard_fs_write_bytes_writes_binary_file_for_source_and_built_runs`
- `standard_fs_write_bytes_missing_parent_returns_io_error`
- `package_std_fs_write_bytes_sample_runs`

Release-readiness coverage keeps this document, `std::fs`, builtin typing,
runtime behavior, samples, stdlib review, specs, and public docs aligned.
