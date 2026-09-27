#!/usr/bin/env python3
"""Run the Phase 0 VM/Go/Rust workload suite. Python 3.9+; no pip packages."""

import argparse
import datetime as dt
import hashlib
import json
import math
import os
import platform
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CASES = (
    "cpu_loop", "cpu_recursion", "cpu_records", "cpu_enums",
    "string", "list", "map", "json", "directory", "text",
)
def loop_checksum():
    value = 0
    for i in range(200000):
        value = (value * 33 + i * 17 + 3) % 1000003
    return value


def records_checksum():
    x, y = 1, 2
    for i in range(10000):
        x = (x * 33 + i) % 1000003
        y = (y * 17 + x) % 1000003
    return x + y


def enums_checksum():
    state, total = 1, 0
    for i in range(20000):
        state = (state * 33 + i) % 1000003
        total += state if i % 2 == 0 else -state
    return total


EXPECTED = {
    "cpu_loop": loop_checksum(),
    "cpu_recursion": 17711,
    "cpu_records": records_checksum(),
    "cpu_enums": enums_checksum(),
    "string": 5401,
    "list": 319600,
    "map": 44850,
    "json": 4950,
    "directory": 20,
    "text": 100,
}


def command(label, *args, cwd=ROOT):
    print("%s: starting" % label, file=sys.stderr, flush=True)
    started = time.monotonic()
    process = subprocess.Popen(args, cwd=cwd)
    while True:
        try:
            returncode = process.wait(timeout=5)
            break
        except subprocess.TimeoutExpired:
            print("%s: still running (%.0f s)" %
                  (label, time.monotonic() - started), file=sys.stderr, flush=True)
    if returncode:
        print("%s: failed after %.1f s" %
              (label, time.monotonic() - started), file=sys.stderr, flush=True)
        raise subprocess.CalledProcessError(returncode, args)
    print("%s: done (%.1f s)" %
          (label, time.monotonic() - started), file=sys.stderr, flush=True)


def version(*args):
    return subprocess.check_output(args, text=True, cwd=ROOT).strip()


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build(build_dir, go_binary_override=None):
    command("[build 1/3] Muga VM harness", "cargo", "build", "--release", "--locked", "--manifest-path", "benchmarks/vm/Cargo.toml")
    command("[build 2/3] Rust baseline", "cargo", "build", "--release", "--locked", "--manifest-path", "benchmarks/rust/Cargo.toml")
    go_binary = go_binary_override or build_dir / "benchmark-go"
    if not go_binary_override:
        command("[build 3/3] Go baseline", "go", "build", "-o", str(go_binary), "benchmarks/go/main.go")
    else:
        print("[build 3/3] Go baseline: using %s" % go_binary,
              file=sys.stderr, flush=True)
    suffix = ".exe" if os.name == "nt" else ""
    return {
        "muga_vm": ROOT / "benchmarks" / "vm" / "target" / "release" / ("muga-benchmark-vm" + suffix),
        "go": go_binary,
        "rust": ROOT / "benchmarks" / "rust" / "target" / "release" / ("muga-benchmark-rust" + suffix),
    }


def fixtures(root):
    json_path = root / "records.json"
    json_path.write_text(json.dumps([{"score": i, "name": "item-%d" % i} for i in range(100)], separators=(",", ":")), encoding="utf-8")
    text_path = root / "events.txt"
    text_path.write_text("".join("error item %d\n" % i if i % 5 == 0 else "info item %d\n" % i for i in range(500)), encoding="utf-8")
    tree = root / "tree"
    for directory in range(4):
        folder = tree / ("part-%d" % directory)
        folder.mkdir(parents=True)
        for file_number in range(4):
            (folder / ("item-%d.txt" % file_number)).write_text("payload\n", encoding="utf-8")
    return {"json": json_path, "directory": tree, "text": text_path}


def source(case):
    if case in ("json", "directory", "text"):
        return ROOT / "benchmarks" / "muga" / "bench" / case / "main.muga"
    return ROOT / "benchmarks" / "muga" / (case + ".muga")


def sample(process):
    process.stdin.write("run\n")
    process.stdin.flush()
    line = process.stdout.readline()
    if not line:
        raise RuntimeError("benchmark process ended early: " + process.stderr.read())
    fields = line.strip().split("\t")
    if len(fields) != 4:
        raise RuntimeError("invalid benchmark protocol response: " + repr(line))
    checksum, elapsed_ns, allocations, allocated_bytes = map(int, fields)
    if elapsed_ns < 0 or allocations < 0 or allocated_bytes < 0:
        raise RuntimeError("negative benchmark measurement: " + repr(line))
    return {"checksum": checksum, "elapsed_ns": elapsed_ns,
            "allocations": allocations, "allocated_bytes": allocated_bytes}


def peak_rss(process):
    """Reap this one process with wait4, so peak RSS belongs to this case."""
    if hasattr(os, "wait4"):
        _, status, usage = os.wait4(process.pid, 0)
        process.returncode = os.waitstatus_to_exitcode(status)
        # Darwin reports bytes; Linux reports KiB.
        multiplier = 1 if sys.platform == "darwin" else 1024
        return int(usage.ru_maxrss * multiplier)
    process.wait()
    return None


def measure(binary, language, case, data, warmup, iterations, on_progress):
    args = [str(binary)]
    if language == "muga_vm":
        args.append(str(source(case)))
    else:
        args.append(case)
    if data:
        args.append(str(data))
    process = subprocess.Popen(args, cwd=ROOT, stdin=subprocess.PIPE,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               text=True, bufsize=1)
    samples = []
    try:
        on_progress(0)
        for index in range(warmup + iterations):
            measurement = sample(process)
            if measurement["checksum"] != EXPECTED[case]:
                raise RuntimeError("%s/%s returned %s, expected %s" %
                                   (language, case, measurement["checksum"], EXPECTED[case]))
            if index >= warmup:
                samples.append(measurement)
            on_progress(index + 1)
        process.stdin.close()
        memory = peak_rss(process)
        stderr = process.stderr.read()
        if process.returncode != 0:
            raise RuntimeError("%s/%s exited %s: %s" % (language, case, process.returncode, stderr))
    except BaseException:
        if process.returncode is None:
            process.kill()
            process.wait()
        raise
    finally:
        if not process.stdin.closed:
            process.stdin.close()
        process.stdout.close()
        process.stderr.close()
    elapsed = sorted(item["elapsed_ns"] for item in samples)
    median = statistics.median(elapsed)
    return {
        "checksum": EXPECTED[case],
        "samples": samples,
        "median_ns": median,
        "p95_ns": elapsed[math.ceil(0.95 * len(elapsed)) - 1],
        "throughput_per_second": 1_000_000_000 / median if median else None,
        "median_allocations": statistics.median(item["allocations"] for item in samples),
        "median_allocated_bytes": statistics.median(item["allocated_bytes"] for item in samples),
        "process_peak_rss_bytes": memory,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--warmup", type=int, default=2)
    parser.add_argument("--iterations", type=int, default=10)
    parser.add_argument("--case", choices=CASES, action="append", dest="cases")
    parser.add_argument("--output", type=Path, help="save the JSON result to this file")
    parser.add_argument("--go-binary", type=Path, help="use an already built Go baseline binary")
    args = parser.parse_args()
    if args.warmup < 0 or args.iterations < 1:
        parser.error("warmup must be nonnegative and iterations must be positive")
    selected_cases = args.cases or CASES
    case_word = "case" if len(selected_cases) == 1 else "cases"
    run_started = time.monotonic()
    print("Preparing %d %s across 3 implementations (%d warm-up + %d measured runs each)" %
          (len(selected_cases), case_word, args.warmup, args.iterations),
          file=sys.stderr, flush=True)
    with tempfile.TemporaryDirectory(prefix="muga-benchmark-") as temporary:
        temp = Path(temporary)
        binaries = build(temp, args.go_binary.resolve() if args.go_binary else None)
        data = fixtures(temp)
        binary_hashes = {language: sha256(path) for language, path in binaries.items()}
        input_hashes = {case: sha256(path) for case, path in data.items() if path.is_file()}
        input_hashes["directory"] = hashlib.sha256("".join(
            "%s:%s\n" % (path.relative_to(data["directory"]), sha256(path))
            for path in sorted(data["directory"].rglob("*")) if path.is_file()
        ).encode()).hexdigest()
        results = {}
        total_pairs = len(selected_cases) * len(binaries)
        for case_index, case in enumerate(selected_cases):
            results[case] = {}
            for language_index, (language, binary) in enumerate(binaries.items()):
                prefix = "[%d/%d] %s / %s:" % (
                    case_index * len(binaries) + language_index + 1,
                    total_pairs, case, language)
                pair_started = time.monotonic()

                def show_progress(completed):
                    if completed <= args.warmup and args.warmup:
                        phase = "warm-up %d/%d" % (completed, args.warmup)
                    else:
                        phase = "measured %d/%d" % (completed - args.warmup, args.iterations)
                    message = "%s %s (%.1f s)" % (
                        prefix, phase, time.monotonic() - pair_started)
                    if sys.stderr.isatty():
                        print("\r" + message + "\033[K",
                              end="\n" if completed == args.warmup + args.iterations else "",
                              file=sys.stderr, flush=True)
                    elif (completed == 0 or completed == args.warmup + args.iterations or
                          (completed >= args.warmup and
                           (completed - args.warmup) % max(1, args.iterations // 4) == 0)):
                        print(message, file=sys.stderr, flush=True)

                result = measure(binary, language, case, data.get(case),
                                 args.warmup, args.iterations, show_progress)
                results[case][language] = result
                print("%s median %.3f ms" % (prefix, result["median_ns"] / 1_000_000),
                      file=sys.stderr, flush=True)
    report = {
        "schema_version": 1,
        "created_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "commit": version("git", "rev-parse", "HEAD"),
        "git_dirty": bool(version("git", "status", "--porcelain")),
        "host": {"platform": platform.platform(), "machine": platform.machine(), "python": platform.python_version()},
        "toolchains": {"rustc": version("rustc", "--version"), "go": version("go", "version")},
        "binary_sha256": binary_hashes,
        "fixture_sha256": input_hashes,
        "config": {"warmup": args.warmup, "iterations": args.iterations,
                   "timing": "workload only, compiled/loaded before first sample",
                   "rss": "process high-water mark including initialization and warmup"},
        "results": results,
    }
    rendered = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(rendered, encoding="utf-8")
        print("Result written to %s" % args.output, file=sys.stderr, flush=True)
    else:
        sys.stdout.write(rendered)
    print("Finished %d %s in %.1f s" %
          (len(selected_cases), case_word, time.monotonic() - run_started),
          file=sys.stderr, flush=True)


if __name__ == "__main__":
    main()
