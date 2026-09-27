# Edit-check loop latency

This benchmark measures the time from starting the `muga` CLI to its exit for
`check`, `run`, and `test`. Each invocation uses a fresh process. Unlike the VM
runtime benchmark, these times include process startup, source loading,
compilation, execution where applicable, and captured output. Release build
time is excluded.

Two projects are measured:

- **Small:** the package in `fixtures/small`, with one function and one test.
- **Medium:** a temporary copy of `samples/projects/config_app` (about 170
  source lines), with one passing test added. The generated `.muga` cache is
  excluded from the copy. `run` reads the checked-in configuration fixture.

Each **initial** sample is the first invocation of a command on a fresh
temporary project copy. **Repeated** samples use the same path as the last
initial sample. The OS file cache is not cleared, so initial is not a fully
cold operating-system measurement. Every sample starts a new CLI process;
this is not an in-process compiler cache benchmark.

Run from the repository root with Rust/Cargo and Python 3.9+ (or uv):

```sh
uv run --python 3.13 benchmarks/edit_check/run.py --iterations 20 --output /tmp/muga-edit-check.json
```

For a quick correctness smoke run, use `--iterations 2`. The runner builds the
release binary before timing. To reuse an already built release binary, add
`--skip-build`; confirm that it matches the source revision yourself. Progress
and medians appear on stderr, and the JSON is written after all six conditions
succeed. Each sample must exit successfully, and output must remain identical
within a condition. `test` must report one passing test.

Results include individual initial and repeated samples, medians, nearest-rank
p95, toolchain and host metadata, the binary hash, and fixture hashes. Compare
measurements from the same machine under similar load. The initial values
describe first use of a fresh project path with unspecified OS cache state.
This benchmark does not set timing pass/fail thresholds.

The first baseline is in
[`results/2026-09-27-macos-arm64.json`](results/2026-09-27-macos-arm64.json).
