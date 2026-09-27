#!/usr/bin/env python3
"""Run the pinned mini-git AI authoring benchmark for Muga and Go (Python 3.9+)."""

import argparse
import datetime as dt
import hashlib
import json
import os
import platform
import re
import signal
import shlex
import shutil
import statistics
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
UPSTREAM_COMMIT = "7f6ffdc8c61a3ec7b7f2a64301a6f47d1eaceeb5"
UPSTREAM_FILES = ("SPEC-v1.txt", "test-v1.sh", "SPEC-v2.txt", "test-v2.sh")
PASS_RE = re.compile(r"^PASSED:\s*(\d+)\s*$", re.MULTILINE)
FAIL_RE = re.compile(r"^FAILED:\s*(\d+)\s*$", re.MULTILINE)
TOTAL_RE = re.compile(r"^TOTAL:\s*(\d+)\s*$", re.MULTILINE)


def log(message):
    print(message, file=sys.stderr, flush=True)


def checked(*command, cwd=ROOT):
    log("setup: " + shlex.join(str(part) for part in command))
    subprocess.run(command, cwd=cwd, check=True)


def version(*command):
    try:
        return subprocess.check_output(command, text=True, stderr=subprocess.STDOUT).strip()
    except (OSError, subprocess.CalledProcessError):
        return None


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def copy_upstream(upstream, stage, workspace):
    for name in UPSTREAM_FILES[:2] if stage == "v1" else UPSTREAM_FILES[2:]:
        shutil.copyfile(upstream / name, workspace / name)


def run_logged(command, cwd, prefix, timeout, environment=None):
    started = time.monotonic()
    stdout_path = Path(str(prefix) + ".stdout.log")
    stderr_path = Path(str(prefix) + ".stderr.log")
    with stdout_path.open("w", encoding="utf-8") as stdout_file, \
            stderr_path.open("w", encoding="utf-8") as stderr_file:
        process = subprocess.Popen(command, cwd=cwd, env=environment, stdin=subprocess.DEVNULL,
                                   stdout=stdout_file, stderr=stderr_file,
                                   start_new_session=True)
        timed_out = False
        last_update = started
        try:
            while True:
                remaining = timeout - (time.monotonic() - started)
                if remaining <= 0:
                    timed_out = True
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
                    break
                try:
                    process.wait(timeout=min(15, remaining))
                    break
                except subprocess.TimeoutExpired:
                    if time.monotonic() - last_update >= 30:
                        log("%s: still running (%.0fs)" %
                            (prefix.name, time.monotonic() - started))
                        last_update = time.monotonic()
        except BaseException:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
            raise
        exit_code = None if timed_out else process.returncode
    elapsed = time.monotonic() - started
    stdout = stdout_path.read_text(encoding="utf-8", errors="replace")
    return {"exit_code": exit_code, "timed_out": timed_out,
            "elapsed_seconds": round(elapsed, 3),
            "stdout_log": str(stdout_path), "stderr_log": str(stderr_path)}, stdout


def write_launcher(workspace, adapter_binary):
    entry = workspace / "src/main/main.muga"
    launcher = workspace / "minigit"
    launcher.write_text("#!/bin/sh\nexec " + shlex.quote(str(adapter_binary)) +
                        " " + shlex.quote(str(entry)) + " \"$@\"\n", encoding="utf-8")
    launcher.chmod(0o755)
    return sha256(launcher)


def setup_muga(workspace, muga_binary, adapter_binary):
    checked(muga_binary, "new", "--template", "app", workspace)
    return write_launcher(workspace, adapter_binary)


def setup_go(workspace):
    workspace.mkdir()
    (workspace / "go.mod").write_text("module minigit\n\ngo 1.22\n", encoding="utf-8")
    (workspace / "Makefile").write_text("all:\n\tgo build -o minigit .\n", encoding="utf-8")
    return None


def prompt(language, stage, guide, muga_binary):
    if stage == "v1":
        intro = ("Implement minigit as described in SPEC-v1.txt. "
                 "The executable must be named minigit and runnable as ./minigit. ")
    else:
        intro = ("Read SPEC-v2.txt and extend the existing minigit implementation. ")
    if language == "muga":
        details = ("Use Muga for the implementation in src/main/main.muga. "
                   "The existing minigit launcher runs that source and maps main's Int "
                   "return value (0 for success, nonzero for errors) to the process exit code. "
                   "Keep the launcher unchanged. Use only " + str(muga_binary) +
                   " for Muga commands; other installed muga versions may differ. ")
    else:
        details = ("Use Go. The provided Makefile builds ./minigit; keep it. ")
    reference = "Read REFERENCE.md first. " if guide else ""
    return (reference + intro + details +
            "Inspect only this workspace; do not search "
            "home directories, other repositories, or previous run logs. "
            "Verify by running bash test-%s.sh." % stage)


def codex_data(output):
    events = []
    for line in output.splitlines():
        try:
            events.append(json.loads(line))
        except ValueError:
            continue
    completed = next((item for item in reversed(events)
                      if isinstance(item, dict) and item.get("type") == "turn.completed"), None)
    failed = next((item for item in reversed(events)
                   if isinstance(item, dict) and item.get("type") == "turn.failed"), None)
    started = next((item for item in events
                    if isinstance(item, dict) and item.get("type") == "thread.started"), None)
    return {"thread_id": started.get("thread_id") if started else None,
            "usage": completed.get("usage") if completed else None,
            "completed": completed is not None,
            "error": failed.get("error") if failed else None}


def grade(workspace, stage, prefix, timeout):
    result, output = run_logged(["bash", "test-%s.sh" % stage], workspace, prefix, timeout)
    counts = [pattern.search(output) for pattern in (PASS_RE, FAIL_RE, TOTAL_RE)]
    script = (workspace / ("test-%s.sh" % stage)).read_text(encoding="utf-8")
    expected = len(set(re.findall(r'^\s*pass "([^"]+)', script, re.MULTILINE)))
    complete = all(counts)
    passed = int(counts[0].group(1)) if complete else len(re.findall(r"^PASS: ", output, re.MULTILINE))
    failed = int(counts[1].group(1)) if complete else expected - passed
    total = int(counts[2].group(1)) if complete else expected
    result.update({"passed": passed, "failed": failed, "total": total,
                   "test_script_completed": bool(complete),
                   "expected_total": expected})
    result["all_passed"] = (result["exit_code"] == 0 and complete and
                            failed == 0 and total == expected)
    return result


def summary(report):
    result = {}
    for language in report["config"]["languages"]:
        result[language] = {}
        for stage in ("v1", "v2"):
            items = [run["stages"][stage] for run in report["runs"]
                     if run["language"] == language and "tests" in run["stages"].get(stage, {})]
            times = [item["agent"]["elapsed_seconds"] for item in items]
            input_tokens = [item["agent"]["codex"]["usage"]["input_tokens"] for item in items
                            if item["agent"]["codex"]["usage"] and
                            item["agent"]["codex"]["usage"].get("input_tokens") is not None]
            output_tokens = [item["agent"]["codex"]["usage"]["output_tokens"] for item in items
                             if item["agent"]["codex"]["usage"] and
                             item["agent"]["codex"]["usage"].get("output_tokens") is not None]
            cached_tokens = [item["agent"]["codex"]["usage"]["cached_input_tokens"] for item in items
                             if item["agent"]["codex"]["usage"] and
                             item["agent"]["codex"]["usage"].get("cached_input_tokens") is not None]
            successes = sum(bool(item["agent"]["completed_attempt"] and
                                 item["tests"]["all_passed"] and
                                 all(item["inputs_unchanged_after_agent"].values()) and
                                 item.get("launcher_unchanged", True)) for item in items)
            result[language][stage] = {
                "graded_attempts": len(items), "successful_attempts": successes,
                "pass_rate": successes / len(items) if items else None,
                "median_agent_wall_seconds": statistics.median(times) if times else None,
                "median_tests_passed": statistics.median(item["tests"]["passed"] for item in items)
                                       if items else None,
                "median_input_tokens": statistics.median(input_tokens) if input_tokens else None,
                "median_cached_input_tokens": statistics.median(cached_tokens) if cached_tokens else None,
                "median_output_tokens": statistics.median(output_tokens) if output_tokens else None,
            }
    return result


def persist(report, path):
    report["summary"] = summary(report)
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    temporary.replace(path)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--upstream-dir", required=True, type=Path,
                        help="checkout of mame/ai-coding-lang-bench at the pinned commit")
    parser.add_argument("--output-dir", required=True, type=Path,
                        help="new directory for workspaces, transcripts, and result.json")
    parser.add_argument("--language", action="append", choices=("muga", "go"),
                        help="repeat to select languages (default: both)")
    parser.add_argument("--trials", type=int, default=1)
    parser.add_argument("--for-release", metavar="vX.Y.Z",
                        help="release version being evaluated; required to run an agent")
    parser.add_argument("--model", default="gpt-6-luna", help="Codex model ID (default: gpt-6-luna)")
    parser.add_argument("--reasoning-effort", default="high",
                        choices=("none", "low", "medium", "high", "xhigh", "max"))
    parser.add_argument("--agent-timeout", type=int, default=900)
    parser.add_argument("--test-timeout", type=int, default=180)
    parser.add_argument("--guide", type=Path,
                        help="optional compact Muga reference, copied as REFERENCE.md")
    parser.add_argument("--prepare-only", action="store_true",
                        help="create workspaces and prompts without invoking Codex or tests")
    args = parser.parse_args()
    if args.trials < 1 or args.agent_timeout < 1 or args.test_timeout < 1:
        parser.error("trials and timeouts must be positive")
    if not args.prepare_only and not args.for_release:
        parser.error("AI authoring runs are release-only; pass --for-release vX.Y.Z")
    if args.for_release and not re.fullmatch(r"v\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?", args.for_release):
        parser.error("--for-release must be a version such as v0.6.1 or v1.0.0-rc.1")
    languages = tuple(dict.fromkeys(args.language or ("muga", "go")))
    upstream = args.upstream_dir.resolve()
    output = args.output_dir.resolve()
    if version("git", "-C", str(upstream), "rev-parse", "HEAD") != UPSTREAM_COMMIT:
        parser.error("upstream checkout must be at " + UPSTREAM_COMMIT)
    if any(not (upstream / name).is_file() for name in UPSTREAM_FILES):
        parser.error("upstream checkout is missing a spec or test file")
    if version("git", "-C", str(upstream), "status", "--porcelain", "--",
               *UPSTREAM_FILES):
        parser.error("upstream specs and tests must match the pinned commit")
    if output.exists():
        parser.error("output directory already exists: " + str(output))
    guide = args.guide.resolve() if args.guide else None
    if guide and (not guide.is_file() or "muga" not in languages):
        parser.error("--guide must be a file and requires --language muga")
    if not args.prepare_only and not shutil.which("codex"):
        parser.error("codex CLI is required; use --prepare-only to inspect workspaces")

    built_muga = ROOT / "target/debug/muga"
    built_adapter = ROOT / "benchmarks/ai_authoring/adapter/target/release/muga-minigit-adapter"
    tooling_started = time.monotonic()
    if "muga" in languages:
        checked("cargo", "build", "--locked", "--bin", "muga")
        checked("cargo", "build", "--release", "--locked", "--manifest-path",
                ROOT / "benchmarks/ai_authoring/adapter/Cargo.toml")
    output.mkdir(parents=True, exist_ok=False)
    tools_dir = output / "tools"
    tools_dir.mkdir()
    muga_binary = tools_dir / "muga"
    adapter_binary = tools_dir / "muga-minigit-adapter"
    if "muga" in languages:
        shutil.copy2(built_muga, muga_binary)
        shutil.copy2(built_adapter, adapter_binary)
    tooling_setup_seconds = round(time.monotonic() - tooling_started, 3)
    report = {
        "schema_version": 1,
        "created_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "upstream": {"repository": "https://github.com/mame/ai-coding-lang-bench",
                     "commit": UPSTREAM_COMMIT,
                     "files_sha256": {name: sha256(upstream / name) for name in UPSTREAM_FILES}},
        "muga_commit": version("git", "-C", str(ROOT), "rev-parse", "HEAD"),
        "muga_dirty_at_start": bool(version("git", "-C", str(ROOT), "status", "--porcelain")),
        "runner_sha256": sha256(Path(__file__)),
        "host": {"platform": platform.platform(), "machine": platform.machine()},
        "toolchains": {"codex": version("codex", "--version"),
                       "muga": version(str(muga_binary), "--version") if "muga" in languages else None,
                       "go": version("go", "version") if "go" in languages else None},
        "tool_sha256": {"muga": sha256(muga_binary), "adapter": sha256(adapter_binary)}
                       if "muga" in languages else {},
        "tooling_setup_seconds": tooling_setup_seconds,
        "config": {"languages": languages, "trials": args.trials, "model": args.model,
                   "for_release": args.for_release,
                   "reasoning_effort": args.reasoning_effort,
                   "agent_timeout_seconds": args.agent_timeout,
                   "test_timeout_seconds": args.test_timeout,
                   "condition_by_language": {
                       language: ("with_muga_reference" if guide and language == "muga"
                                  else "no_reference") for language in languages},
                   "guide_sha256": sha256(guide) if guide else None,
                   "prepare_only": args.prepare_only},
        "runs": [],
    }
    result_file = output / "result.json"
    persist(report, result_file)
    environment = os.environ.copy()
    environment["PATH"] = str(muga_binary.parent) + os.pathsep + environment.get("PATH", "")

    for trial in range(1, args.trials + 1):
        for language in languages:
            name = "%s-%02d" % (language, trial)
            v1 = output / (name + "-v1")
            v2 = output / (name + "-v2")
            log("[%s] preparing v1" % name)
            started = time.monotonic()
            launcher_hash = (setup_muga(v1, muga_binary, adapter_binary)
                             if language == "muga" else setup_go(v1))
            copy_upstream(upstream, "v1", v1)
            if guide and language == "muga":
                shutil.copyfile(guide, v1 / "REFERENCE.md")
            setup_seconds = round(time.monotonic() - started, 3)
            record = {"language": language, "trial": trial, "stages": {}}
            report["runs"].append(record)
            persist(report, result_file)
            for stage, workspace in (("v1", v1), ("v2", v2)):
                stage_launcher_hash = launcher_hash
                if stage == "v2":
                    log("[%s] preparing v2 from v1" % name)
                    started = time.monotonic()
                    shutil.copytree(v1, v2, ignore=shutil.ignore_patterns("testrepo", ".minigit"))
                    copy_upstream(upstream, "v2", v2)
                    if language == "muga":
                        stage_launcher_hash = write_launcher(v2, adapter_binary)
                    setup_seconds = round(time.monotonic() - started, 3)
                current_prompt = prompt(language, stage, guide is not None and language == "muga",
                                        muga_binary)
                (workspace / "PROMPT.txt").write_text(current_prompt + "\n", encoding="utf-8")
                item = {"workspace": str(workspace), "setup_seconds": setup_seconds,
                        "prompt": current_prompt}
                record["stages"][stage] = item
                persist(report, result_file)
                if args.prepare_only:
                    continue
                command = ["codex", "exec", "--json", "--ephemeral", "--ignore-user-config",
                           "--skip-git-repo-check", "--sandbox", "workspace-write",
                           "-C", str(workspace), "--model", args.model,
                           "-c", 'model_reasoning_effort="%s"' % args.reasoning_effort,
                           current_prompt]
                log("[%s/%s] agent running (timeout %ds)" % (name, stage, args.agent_timeout))
                agent, response = run_logged(command, workspace, output / (name + "-" + stage + "-agent"),
                                             args.agent_timeout, environment)
                agent["codex"] = codex_data(response)
                agent["completed_attempt"] = (agent["exit_code"] == 0 and
                                              not agent["timed_out"] and
                                              agent["codex"]["completed"])
                item["agent"] = agent
                persist(report, result_file)
                if agent["codex"]["error"] or (agent["exit_code"] != 0 and not agent["timed_out"]):
                    log("[%s/%s] Codex failed; stopping without scoring this stage" % (name, stage))
                    raise SystemExit(2)
                log("[%s/%s] agent finished in %.1fs (exit %s); running original tests" %
                    (name, stage, agent["elapsed_seconds"], agent["exit_code"]))
                stage_files = UPSTREAM_FILES[:2] if stage == "v1" else UPSTREAM_FILES[2:]
                item["inputs_unchanged_after_agent"] = {
                    filename: (workspace / filename).is_file() and
                    sha256(workspace / filename) == report["upstream"]["files_sha256"][filename]
                    for filename in stage_files
                }
                copy_upstream(upstream, stage, workspace)
                item["tests"] = grade(workspace, stage, output / (name + "-" + stage + "-tests"),
                                      args.test_timeout)
                if stage_launcher_hash:
                    item["launcher_unchanged"] = ((workspace / "minigit").is_file() and
                                                  sha256(workspace / "minigit") == stage_launcher_hash)
                persist(report, result_file)
                tests = item["tests"]
                log("[%s/%s] %s/%s passed" % (name, stage, tests["passed"], tests["total"]))
    log("Results: " + str(result_file))


if __name__ == "__main__":
    main()
