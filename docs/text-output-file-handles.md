# Text Output File Handle Design

Status: implemented. The implementation follows this selected design after the
scalar `eprint` / `eprintln` slice.

## Purpose

The read-only `std::fs::File` runtime handle proves that Muga can expose a
public opaque resource handle, load its `.mgi` metadata from artifacts, reject
direct use-after-consume through `T026`, and reject stale or closed aliases at
runtime with `R022`.

The next practical gap is incremental text output. Muga already has
`fs::write_text_path` for one-shot full-file writes, but real CLI and report
programs need append and explicit flush without introducing `Bytes`, streaming,
standard-stream handles, process pipes, or async IO.

## Selected Public API

Add these functions to `std::fs`:

```muga
pub fn create_text(file_path: path::Path): Result[File, io::IOError]
pub fn append_text(file_path: path::Path): Result[File, io::IOError]
pub fn write_text_to(file: File, text: String): Result[Unit, io::IOError]
pub fn flush(file: File): Result[Unit, io::IOError]
```

Keep the existing read/close API unchanged:

```muga
pub fn open_text(file_path: path::Path): Result[File, io::IOError]
pub fn read_text_from(file: File): Result[String, io::IOError]
pub fn close(file: File): Result[Unit, io::IOError]
```

Rules:

- `open_text` opens a read-only text handle.
- `create_text` creates or truncates a text file and opens a write-only handle.
- `append_text` creates the file if missing and opens an append-only handle.
- `write_text_to` writes the supplied UTF-8 `String` exactly as provided.
- `flush` borrows the handle and keeps it usable after success or recoverable
  failure.
- `close` remains the only consuming operation for `File`.
- only `close(file)` has `paramMode=consume` in `.mgi`; write and flush borrow.

## Candidate Comparison

| Candidate | Benefit | Cost | Decision |
|---|---|---|---|
| One public `File` with runtime access mode | Small API, preserves existing `close`, avoids extra opaque types in docs/editor output | Wrong-mode operations are runtime recoverable errors | Select |
| Separate `TextReader` and `TextWriter` opaque types | Statically prevents wrong-mode read/write calls | Requires new close functions, more `.mgi` metadata, harder migration from existing `File` | Defer |
| `open_text(path, mode)` with an enum or options record | Fewer function names and easier future mode growth | Requires designing public mode values now and makes simple use noisier | Defer |
| Expand one-shot `write_text_path` only | No handle complexity | Does not support append, flush, or long-running report generation | Reject |
| stdout/stderr handles | Useful for process and service adapters | Ambient, often uncloseable, task-sharing and redirection semantics unclear | Defer |

The chosen design intentionally accepts runtime wrong-mode errors because the
handle is valid; the requested operation is unsupported for its open mode.
That makes wrong-mode behavior a recoverable `io::IOError`, not `R022`.

## Runtime Representation

Extend the existing VM-local `std::fs::File` slot:

```text
Open {
  path: String,
  file: host file,
  mode: Read | Write | Append,
  generation: u64,
}
Closed { generation: u64 }
```

The public runtime handle value remains `{ family, slot, generation }` with
`family = "std::fs::File"`. Live host file descriptors never appear in `.mgi`,
`.mgc`, or `.mgb`.

Mode behavior:

- `read_text_from` requires `Read`, seeks to byte offset 0, and reads the full
  file as UTF-8 text, preserving the current behavior.
- `write_text_to` requires `Write` or `Append`.
- `flush` requires `Write` or `Append`.
- wrong-mode operations return `Result::Err(io::IOError)` with the attempted
  operation name and handle path.
- stale slot, wrong family, double close, and use after successful close remain
  hard `R022` diagnostics.

## Close And Flush

For read-only handles, `close` closes the slot and returns `Result::Ok(())`.

For write or append handles, `close` attempts `flush` first, then closes the
runtime slot regardless of the flush result. If the flush fails, `close`
returns `Result::Err(io::IOError)` and the handle is still consumed/closed.

This keeps the consuming metadata honest: after `close(file)`, source should not
use `file` again, even if close reports a recoverable host failure. Callers that
need retryable failure handling should call `flush(file)` before `close(file)`.

## Typechecking And Interface Contract

- `create_text` and `append_text` take `path::Path` and return
  `Result[File, io::IOError]`.
- `write_text_to` takes `File` and `String`, returning
  `Result[Unit, io::IOError]`.
- `flush` takes `File` and returns `Result[Unit, io::IOError]`.
- `close` remains `consume`; all other `File` parameters remain `borrow`.
- `.mgi` interface hash should include the new public functions through the
  normal function signature path. Opaque handle facts do not need a format bump.
- Artifact-backed `run` must allocate fresh handle tables per VM run, exactly
  as the read-only file slice does.

## Tests

The implementation slice covers:

- source execution for `create_text` + `write_text_to` + `flush` + `close`;
- source execution for `append_text` preserving existing content;
- `write_text_to` on a read-only handle returning recoverable `io::IOError`;
- `read_text_from` on a write-only handle returning recoverable `io::IOError`;
- `close` consuming a write handle with `T026` on direct same-scope reuse;
- artifact-backed execution writing text through emitted `std::fs` artifacts;
- `.mgi` metadata showing the new functions while only `close` is consume;
- release-readiness evidence that `Bytes`, binary handles, standard-stream
  handles, process APIs, async IO, and streams remain deferred.

## Deferred

Do not add these in the text output file handle slice:

- broader binary `Bytes`, encoding conversion, or binary write/stream APIs;
- generic stream traits or iterator protocols;
- `stdin`, `stdout`, or `stderr` handles;
- read/write cursor controls or seek APIs;
- options records, permissions, exclusive create, rename, or recursive removal;
- cancellation-aware async IO or process pipe integration.
