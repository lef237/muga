# Mini-git AI authoring benchmark

This runner adapts [mame/ai-coding-lang-bench](https://github.com/mame/ai-coding-lang-bench)
to Muga and Go. It uses the upstream `SPEC-v1.txt`, `SPEC-v2.txt`,
`test-v1.sh`, and `test-v2.sh` from commit
`7f6ffdc8c61a3ec7b7f2a64301a6f47d1eaceeb5`. It checks that Git HEAD
matches that commit and records SHA-256 hashes of all four files. The specs
and tests live in the upstream checkout and are copied into isolated trial
directories at run time; they are not maintained here.

Run this token-intensive benchmark **only once while preparing a new release**.
Do not run it after individual changes, in CI, or as part of the routine
release-quality gate. The runner requires `--for-release vX.Y.Z` before it will
start an agent. `--prepare-only` remains available for checking setup without
using AI tokens.

## Prepare

On macOS or Linux, install Rust/Cargo, Go, Bash, Python 3.9+, and the Codex
CLI. Sign in to Codex before starting (`codex login`). The runner uses only
Python's standard library, so uv is optional. Checkout the exact upstream
revision:

```sh
git clone https://github.com/mame/ai-coding-lang-bench.git /tmp/muga-ai-coding-lang-bench
git -C /tmp/muga-ai-coding-lang-bench checkout 7f6ffdc8c61a3ec7b7f2a64301a6f47d1eaceeb5
```

Run from this repository. Start with a preparation check; it builds Muga and
the fixed adapter, creates one Muga and one Go workspace, and writes prompts
without invoking the agent:

```sh
uv run --python 3.13 benchmarks/ai_authoring/run.py \
  --upstream-dir /tmp/muga-ai-coding-lang-bench \
  --output-dir /tmp/muga-ai-prepared --prepare-only
```

Use a **new** output directory for each release. A release run with three trials
per language (six v1 and six v2 agent invocations) is:

```sh
uv run --python 3.13 benchmarks/ai_authoring/run.py \
  --upstream-dir /tmp/muga-ai-coding-lang-bench \
  --output-dir /tmp/muga-ai-luna-high-v0.6.1 --trials 3 \
  --for-release v0.6.1
```

Replace `v0.6.1` with the version being prepared. `python3` can replace
`uv run --python 3.13`. The default agent is Codex `gpt-6-luna` with `high`
reasoning. Each invocation has a 15 minute timeout; change it with
`--agent-timeout`. The runner starts a fresh ephemeral Codex
session in each trial workspace, uses workspace-write sandboxing, and ignores
user configuration. There is no CLI dollar budget or known per-run billing
figure; monitor account usage separately. The run has 12 agent invocations.
Both prompts ask the agent to inspect only its trial workspace. The Muga prompt
names the exact benchmark-built compiler path, avoiding any other installed
`muga` version. Check the raw transcript if strict input isolation matters:
the Codex sandbox can still read files outside the workspace.

## What is measured

The runner starts each Muga v1 trial with `muga new --template app`. The
scaffold time is recorded separately. Go v1 starts with a minimal `go.mod`
and Makefile. The v2 workspace is a copy of that trial's v1 workspace, as in
the upstream benchmark. Both languages get the same upstream specifications
and tests. The language specific prompt only explains the supplied scaffold.
The prebuilt Muga binary and adapter are copied into the output directory's
`tools/` folder before trials start, so their build time is excluded and trial
workspaces do not point to the Muga source checkout.

Muga's public `run_path_with_args` API returns a `main` value rather than a
process status. A fixed adapter maps `main(): Int` values in `0..255` to exit
codes and relays program output. Its launcher is supplied before the agent
runs, and the result records whether it stayed unchanged. The agent writes
the mini-git behavior in Muga source. Go's Makefile builds the executable
with the same test entry point, `./minigit`.

Each stage stores the agent prompt, wall time, CLI result, token usage
when reported, original test pass/fail counts, and raw agent/test stdout and
stderr logs. `result.json` is updated after every stage so interrupted runs
retain completed measurements. Agent stdout is JSONL and contains the tool
transcript. Workspaces and logs remain in the chosen output directory for
diagnosis. A run with an agent error or timeout is recorded
as such; a test result from that run is not evidence of a completed attempt.
The report also records the upstream input hashes, runner hash, Muga and
adapter binary hashes, requested model, and reasoning effort. Codex JSONL does
not confirm the actual model or provide a per-run dollar cost. If a timeout
stops the CLI before its final result event, its token usage is unknown.
Provider or CLI errors stop the run immediately and
are left unscored. Retry with a new output directory after service access
returns; those errors must not be counted as language failures.
Raw CLI transcripts can contain account and session metadata. The local
`benchmarks/ai_authoring/runs/` directory is Git ignored for retaining them;
review them before sharing.
The runner checks whether the agent changed the supplied spec, test, or Muga
launcher, and restores the pinned test before official grading. If the test
script exits before its summary, the report counts observed passes against
the known 11 (v1) or 30 (v2) tests and marks the script incomplete.

For a balanced comparison, use the same agent CLI, model, reasoning effort, trial
count, host, upstream commit, and time period for Muga and Go. Report per-stage
pass rates together with median agent wall time, tests passed, and reported token
usage. Cached input tokens are recorded separately. A single
trial is a pipeline check, not an estimate of success probability. The
upstream project used 20 runs per language, so the three-trial release check
does not estimate a comparable success-rate distribution. These Codex Luna
results cannot be directly compared with results from a different agent or
model. Wall times also depend on the prompts and provided scaffolds.

The default condition supplies **no additional Muga language reference**
beyond the generated scaffold. The agent may use documentation surfaced by
the pinned `muga` executable. The transcript records that exploration. Once
the compact reference planned in Phase 2 exists, evaluate it at the next
release with `--language muga --guide PATH` and a new output directory. The
guide is copied as `REFERENCE.md`; its hash and condition are
recorded. Keep the Go arm and other settings matched. Do not compare the two
Muga conditions until that reference is fixed and versioned.

## Recorded Luna high baseline

The [2026-09-27 result](results/2026-09-27-luna-high.json) uses Codex CLI
0.155.1, `gpt-6-luna`, `high` reasoning, a 15 minute limit per stage, and
three fresh trials per language on macOS arm64. The repository was clean at
commit `3102762d4d27f712c5255d05e2ecd00c4b2ede5c`. All supplied inputs and
the Muga launcher remained unchanged.

| Language | Stage | Complete passes | Median agent time | Median input tokens (cached) | Median output tokens |
| --- | --- | ---: | ---: | ---: | ---: |
| Muga | v1 | 3/3 (11/11 tests) | 583.3 s | 2,095,229 (2,000,896) | 23,173 |
| Go | v1 | 3/3 (11/11 tests) | 111.5 s | 204,189 (186,880) | 4,252 |
| Muga | v2 | 3/3 (30/30 tests) | 385.8 s | 1,443,460 (1,369,344) | 15,851 |
| Go | v2 | 3/3 (30/30 tests) | 105.8 s | 173,438 (153,600) | 4,026 |

Muga succeeded in all six stages, but its median agent time was 5.23 times
Go's for v1 and 3.65 times Go's for v2. The large cached share means logical
input tokens are not equivalent to newly processed tokens or a billed cost.
Codex CLI did not provide a per-stage dollar amount. Three trials are enough
for a first baseline, not a precise success-rate estimate. Keep the model,
reasoning effort, prompt, and timeout fixed when comparing future changes.

The complete workspaces and JSONL transcripts are retained locally in the
Git-ignored `benchmarks/ai_authoring/runs/luna-high-2026-09-27/` directory,
excluding rebuildable binaries and Go build caches. Review these logs before
sharing them because they may include session metadata.
