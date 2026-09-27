# Runtime benchmark suite

For the separate mini-git AI authoring benchmark, see
[ai_authoring/README.md](ai_authoring/README.md). Run that token-intensive
benchmark only for releases that materially affect AI authoring.

This is the Phase 0 measurement baseline for the reference VM, Go, and Rust.
It checks the result of every invocation before recording a measurement. The
three implementations use the same input, operation counts, and final checksum.
The Go and Rust versions use their normal standard library data structures and
JSON parsers; this suite compares language implementations, not identical
machine instructions.

## Run

Requirements: Python 3.9+, Go, and the repository's Rust toolchain. The Rust
baseline has its own committed `Cargo.lock` and uses `serde_json` for the JSON
case. The runner builds the Muga VM harness and both native baselines in release
mode before measuring. Build time is excluded.

The first run may spend a while compiling the Rust dependency and Go binary;
later runs reuse Rust build caches. Progress appears immediately on stderr:
each build step reports when it starts and finishes, with a status update every
five seconds while it is running. Each workload then shows its current case,
implementation, warm-up count, and measured iteration count. The JSON output
file is written after all cases pass.

```sh
python3 benchmarks/run.py --warmup 3 --iterations 20 --output /tmp/muga-runtime.json
```

For a quick correctness check of all three implementations:

```sh
python3 benchmarks/run.py --warmup 0 --iterations 1 --output /tmp/muga-runtime-smoke.json
```

If you use uv, the equivalent commands are:

```sh
uv run --python 3.13 benchmarks/run.py --warmup 0 --iterations 1 --output /tmp/muga-runtime-smoke.json
uv run --python 3.13 benchmarks/run.py --warmup 3 --iterations 20 --output /tmp/muga-runtime.json
```

The runner needs only Python's standard library, so uv is optional; there is
no `uv sync` step or Python package lockfile for this suite. `uv run` selects
the Python interpreter directly, which also avoids shell `python3` shim issues.
If Python 3.13 is not installed, uv may download it on first use. Keep the
same Python and native toolchain versions when comparing benchmark runs; the
JSON result records their versions.

Use `--case cpu_loop` (repeatable) to select cases. `--go-binary PATH` uses a
prebuilt Go executable when the local environment cannot build Go from the
runner. Do not use the one-iteration smoke result to compare performance.

The suite covers integer loops, recursive Fibonacci, record updates, enum
matching, strings, lists, maps, JSON processing, recursive directory traversal,
and text processing. Input files and directory trees are generated from fixed
data in a temporary directory. CPU cases do bounded, state-dependent work so
optimizing compilers cannot simply replace the loops with constants.

## What the JSON means

Each case and language has raw samples plus median and nearest-rank p95
nanoseconds, throughput per second, median allocation count and allocated
bytes, and the process peak resident memory in bytes. The runner also records
toolchain versions, host details, commit, warm-up count, and iteration count.
Keep raw results when comparing changes; run on the same quiet machine with
the same toolchains and settings.

Timing starts immediately before one workload invocation and stops immediately
after it. The Muga source is compiled once when its persistent VM process
starts. All implementations stay alive through warm-up and measured iterations,
so startup and pipe communication are outside the timed interval. Allocation
counts cover the same interval, using a counting allocator for Rust and the VM
host, and Go runtime allocation counters for Go. Counts include each runtime's
work to execute the workload. Resident memory is the process high-water mark
over initialization, warm-up, and measurement; for Muga it therefore includes
compilation. On platforms without `wait4`, peak RSS is `null`.

The suite is a baseline for future native backend experiments. It does not set
wall-clock pass/fail thresholds or support public performance claims from one
run. The older `scripts/benchmark-health-check.sh` remains a local smoke tool
for compiler stages and artifact reuse; its one-shot timings are not part of
this runtime comparison.

The first committed machine-readable baseline is
[`results/2026-09-27-macos-arm64.json`](results/2026-09-27-macos-arm64.json).
It was captured on clean commit `17148a28bc3a316034d9bdbea7437a872c153efc`
with 3 warm-up and 20 measured runs per case and implementation. Keep the raw
samples and host metadata when reviewing changes; this one host run is a
starting point, not a universal speed claim.
