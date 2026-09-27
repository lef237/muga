#!/usr/bin/env python3
"""Measure fresh-process Muga edit-check latency; Python 3.9+, no packages."""

import argparse
import datetime as dt
import hashlib
import json
import math
import platform
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
COMMANDS = ("check", "run", "test")
MEDIUM_TEST = """

@test
fn defaults_have_one_server(): Result[Unit, String] {
  test::assert_eq_int(1, default_servers().len())
}
"""


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tool_version(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def build():
    print("Building release Muga (build time excluded)...", file=sys.stderr, flush=True)
    subprocess.run(["cargo", "build", "--release", "--locked"], cwd=ROOT, check=True)
    return ROOT / "target" / "release" / "muga"


def fixture(destination, size):
    if size == "small":
        shutil.copytree(HERE / "fixtures" / "small", destination)
    else:
        shutil.copytree(ROOT / "samples" / "projects" / "config_app", destination,
                        ignore=shutil.ignore_patterns(".muga"))
        source = destination / "src" / "main" / "main.muga"
        source.write_text("import std::test\n" +
                          source.read_text(encoding="utf-8") + MEDIUM_TEST,
                          encoding="utf-8")
    return destination / "src" / "main" / "main.muga"


def invoke(binary, command, source):
    args = [str(binary), command, str(source)]
    start = time.perf_counter_ns()
    process = subprocess.run(args, cwd=ROOT, capture_output=True, timeout=60)
    elapsed = time.perf_counter_ns() - start
    if process.returncode:
        raise RuntimeError("%s failed (exit %d): %s" %
                           (" ".join(args), process.returncode,
                            process.stderr.decode("utf-8", errors="replace")))
    if command == "check" and process.stdout != b"ok\n":
        raise RuntimeError("unexpected check output: %r" % process.stdout)
    if command == "test" and b"1 passed" not in process.stdout:
        raise RuntimeError("test did not report one passing test: %r" % process.stdout)
    return elapsed, process.stdout, process.stderr


def measure(binary, command, sources, iterations, label):
    # Each sample starts a new Muga process. Initial samples use independent
    # project copies; repeated samples reuse the final copy and its path.
    initial = []
    expected_stdout = expected_stderr = None
    for index, source in enumerate(sources):
        elapsed, stdout, stderr = invoke(binary, command, source)
        if index == 0:
            expected_stdout, expected_stderr = stdout, stderr
        elif (stdout, stderr) != (expected_stdout, expected_stderr):
            raise RuntimeError("%s initial output changed on iteration %d" %
                               (label, index + 1))
        initial.append(elapsed)
        if (index + 1) % max(1, iterations // 4) == 0 or index + 1 == iterations:
            print("%s: initial %d/%d" % (label, index + 1, iterations),
                  file=sys.stderr, flush=True)
    repeated_source = sources[-1]
    repeated = []
    for index in range(iterations):
        elapsed, stdout, stderr = invoke(binary, command, repeated_source)
        if (stdout, stderr) != (expected_stdout, expected_stderr):
            raise RuntimeError("%s output changed on iteration %d" % (label, index + 1))
        repeated.append(elapsed)
        if (index + 1) % max(1, iterations // 4) == 0 or index + 1 == iterations:
            print("%s: repeated %d/%d" % (label, index + 1, iterations),
                  file=sys.stderr, flush=True)
    ordered_initial = sorted(initial)
    ordered_repeated = sorted(repeated)
    return {
        "initial_ns": initial,
        "initial_median_ns": statistics.median(ordered_initial),
        "initial_p95_ns": ordered_initial[math.ceil(0.95 * len(ordered_initial)) - 1],
        "repeated_ns": repeated,
        "repeated_median_ns": statistics.median(ordered_repeated),
        "repeated_p95_ns": ordered_repeated[math.ceil(0.95 * len(ordered_repeated)) - 1],
        "stdout_sha256": hashlib.sha256(expected_stdout).hexdigest(),
        "stderr_sha256": hashlib.sha256(expected_stderr).hexdigest(),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--iterations", type=int, default=20,
                        help="initial and repeated invocations per command and project (default: 20 each)")
    parser.add_argument("--output", type=Path, help="write JSON to this path")
    parser.add_argument("--skip-build", action="store_true",
                        help="use the existing target/release/muga")
    args = parser.parse_args()
    if args.iterations < 1:
        parser.error("iterations must be positive")
    binary = ROOT / "target" / "release" / "muga" if args.skip_build else build()
    if not binary.is_file():
        parser.error("release binary not found: %s" % binary)
    print("Measuring 2 projects x 3 commands x (%d initial + %d repeated)" %
          (args.iterations, args.iterations), file=sys.stderr, flush=True)
    started = time.monotonic()
    with tempfile.TemporaryDirectory(prefix="muga-edit-check-") as directory:
        results = {}
        fixture_hashes = {}
        for size in ("small", "medium"):
            results[size] = {}
            for command in COMMANDS:
                sources = [fixture(Path(directory) / size / command / str(index), size)
                           for index in range(args.iterations)]
                fixture_root = sources[0].parents[2]
                fixture_hashes[size] = {
                    str(path.relative_to(fixture_root)): sha256(path)
                    for path in sorted(fixture_root.rglob("*")) if path.is_file()
                }
                label = "%s / %s" % (size, command)
                results[size][command] = measure(binary, command, sources,
                                                 args.iterations, label)
                print("%s: initial median %.2f ms; repeated median %.2f ms" %
                      (label, results[size][command]["initial_median_ns"] / 1e6,
                       results[size][command]["repeated_median_ns"] / 1e6),
                      file=sys.stderr, flush=True)
    report = {
        "schema_version": 1,
        "created_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "commit": tool_version("git", "rev-parse", "HEAD"),
        "git_dirty": bool(tool_version("git", "status", "--porcelain")),
        "host": {"platform": platform.platform(), "machine": platform.machine(),
                 "python": platform.python_version()},
        "toolchain": tool_version("rustc", "--version"),
        "binary_sha256": sha256(binary),
        "fixture_sha256": fixture_hashes,
        "config": {"iterations": args.iterations,
                   "timing": "fresh Muga process for every sample; subprocess wall time includes CLI startup, source loading, compilation, execution, and output capture",
                   "initial": "one invocation on each of N independent project copies; OS cache not cleared",
                   "repeated": "subsequent invocations on the same source path, each in a fresh process"},
        "results": results,
    }
    rendered = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
        print("Result written to %s" % args.output, file=sys.stderr, flush=True)
    else:
        sys.stdout.write(rendered)
    print("Finished in %.1f s" % (time.monotonic() - started),
          file=sys.stderr, flush=True)


if __name__ == "__main__":
    main()
