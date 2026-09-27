# Mini-git AI authoring benchmark

This runner adapts [mame/ai-coding-lang-bench](https://github.com/mame/ai-coding-lang-bench)
to Muga and Go. It uses the upstream `SPEC-v1.txt`, `SPEC-v2.txt`,
`test-v1.sh`, and `test-v2.sh` from commit
`7f6ffdc8c61a3ec7b7f2a64301a6f47d1eaceeb5`. It checks that Git HEAD
matches that commit and records SHA-256 hashes of all four files. The specs
and tests live in the upstream checkout and are copied into isolated trial
directories at run time; they are not maintained here.

## Prepare

On macOS or Linux, install Rust/Cargo, Go, Bash, Python 3.9+, and the Claude
Code CLI. Authenticate the CLI before starting (`claude auth login`); the
runner checks this before launching an agent. The runner uses only Python's
standard library, so uv is optional. Checkout the exact upstream revision:

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

Use a **new** output directory for each run. A full run with three trials per
language (six v1 and six v2 agent invocations) is:

```sh
uv run --python 3.13 benchmarks/ai_authoring/run.py \
  --upstream-dir /tmp/muga-ai-coding-lang-bench \
  --output-dir /tmp/muga-ai-no-reference-01 --trials 3 --model sonnet
```

`python3` can replace `uv run --python 3.13`. Each agent invocation has a
default USD 2 API budget and 20 minute timeout; set `--max-budget-usd` and
`--agent-timeout` explicitly for a different experiment. The runner uses
Claude's automatic permission mode and safe mode. Run it in an isolated
environment if agent authored shell commands require stronger containment.
At the defaults, the three trial, two language example allows up to USD 24
of CLI reported API cost across its 12 agent invocations.

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

Each stage stores the agent prompt, wall time, CLI result, cost and token usage
when reported, original test pass/fail counts, and raw agent/test stdout and
stderr logs. `result.json` is updated after every stage so interrupted runs
retain completed measurements. Agent stdout is JSONL and contains the tool
transcript. Workspaces and logs remain in the chosen output directory for
diagnosis. A run with an agent error or budget/timeout exhaustion is recorded
as such; a test result from that run is not evidence of a completed attempt.
The report also records the upstream input hashes, runner hash, Muga and
adapter binary hashes, requested model, and model actually reported by the CLI.
If a timeout stops the CLI before its final result event, its cost is unknown
in the report even though the transcript is preserved.
Provider API errors (including a session limit) stop the run immediately and
are left unscored. Retry with a new output directory after service access
returns; those errors must not be counted as language failures.
Raw CLI transcripts can contain account and session metadata. The local
`benchmarks/ai_authoring/runs/` directory is Git ignored for retaining them;
review them before sharing.
The runner checks whether the agent changed the supplied spec, test, or Muga
launcher, and restores the pinned test before official grading. If the test
script exits before its summary, the report counts observed passes against
the known 11 (v1) or 30 (v2) tests and marks the script incomplete.

For a balanced comparison, use the same agent CLI and model, budget, trial
count, host, upstream commit, and time period for Muga and Go. Report per-stage
pass rates together with median agent wall time and reported cost. A single
trial is a pipeline check, not an estimate of success probability. The
upstream project used 20 runs per language; use at least that many independent
trials for a comparable distribution. Costs and wall times may differ from
published upstream numbers because the agent version, model, prompts, and
provided scaffolds differ.

The default condition supplies **no additional Muga language reference**
beyond the generated scaffold. The agent still has its normal CLI and file
access, so it may discover installed documentation or source on the host; the
transcript records that exploration. Once the compact reference planned in
Phase 2 exists,
rerun the Muga arm with `--language muga --guide PATH` and a new output
directory. The guide is copied as `REFERENCE.md`; its hash and condition are
recorded. Keep the Go arm and other settings matched. Do not compare the two
Muga conditions until that reference is fixed and versioned.
