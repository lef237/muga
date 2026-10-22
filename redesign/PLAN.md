# Muga Redesign Plan

This file is the working checklist for the redesign. It replaces the legacy
[ROADMAP.md](../ROADMAP.md) for all new work.

## Resume Cursor

Update this section at the end of every working session.

- [ ] **NOW:** Phase B. Write the v1 task suite under `redesign/tasks/`,
  starting from the candidate list below.
- [ ] **NEXT:** Phase C. Resolve the open questions in SPEC.md that the tasks
  need, one at a time with the maintainer.
- Last updated: 2026-10-23. Phase A (design) is complete: VISION.md,
  DECISIONS.md (D-01 to D-25), and the SPEC.md draft.

## Phase A: Design

- [x] Write the vision and the three promises (VISION.md).
- [x] Settle the core decisions one question at a time (DECISIONS.md).
- [x] Assemble the specification draft and the list of open questions
  (SPEC.md).

## Phase B: Task Suite

The task suite is the definition of done for v1 (D-24), so it comes before the
standard library and the implementation.

- [ ] Write about ten tasks as `redesign/tasks/NN-short-name.md`. Each task
  file contains:
  - a problem statement written for an AI agent
  - input fixtures and the expected output (or a checking rule)
  - the minimum grants, such as `--read data/`
  - the language features and standard library operations it exercises
- [ ] Sketch a Muga solution for each task by hand, against SPEC.md. List
  every standard library operation the sketches use. That list settles O-1.

Candidate tasks (adjust freely):

1. Count lines and words in every file under a directory (`--read`).
2. Convert a CSV file to JSON (`--read`, `--write`).
3. Validate a JSON configuration file and report every problem with context
   (`--read`).
4. List the N largest files under a directory (`--read`).
5. Summarize `git log` by author (`--exec git`).
6. Rename files that match a pattern, with a dry-run mode (`--read`,
   `--write`).
7. Search files for a pattern and print matches with line numbers
   (`--read`, standard input).
8. Collect `TODO` comments into a Markdown report (`--read`, `--write`).
9. Read settings from environment variables with defaults (`--env`).
10. Fetch JSON from an HTTP API with `curl` and print selected fields
    (`--exec curl`).

## Phase C: Open Questions And The Guide

- [ ] Resolve the open questions in SPEC.md that the task suite needs. Record
  each answer in DECISIONS.md, and update SPEC.md to match.
- [ ] Mark the remaining open questions as "after v1" when no task needs them.
- [ ] Write the one-page guide for AI agents (`redesign/GUIDE.md`). It covers
  the whole syntax, the standard library operations from Phase B, and the
  canonical patterns, and is short enough to fit in an agent's context.

## Phase D: Replace The Legacy Implementation

Follow D-22.

- [ ] Create the `legacy` branch from the latest `main` and push it. The
  `v0.6.0` tag already marks the last legacy release.
- [ ] Remove the legacy implementation, tests, samples, benchmarks,
  conformance fixtures, and documents from `main`. Keep `LICENSE.txt`, and
  adapt `RELEASING.md` and the CI workflows.
- [ ] Create the new crate skeleton: no dependencies, `warnings = "deny"`,
  `unsafe_code = "forbid"`, and Clippy lints denied. CI runs `cargo fmt
  --check`, Clippy, and the tests.
- [ ] Decide where the redesign documents live after the switch, and update
  the root README.md.

## Phase E: Implementation

Build in this order. Each task in the suite becomes an end-to-end test as soon
as the features it uses exist.

1. Lexer
2. Parser, including diagnostics for habits from other languages (SPEC.md
   section 12)
3. Name resolution and type checking
4. Typed intermediate representation
5. Tree-walking evaluator
6. Capability runtime and the grant flags of `muga run`
7. The standard library operations the task suite needs
8. `muga test`
9. `muga fmt`

Keep the size guardrail in VISION.md in view. When the implementation heads
past about 20,000 lines, cut scope.

## Phase F: Evaluation And Release

- [ ] Give AI agents only GUIDE.md and run them on the task suite. Record the
  results in `redesign/evaluations/`.
- [ ] Fix the guide, the diagnostics, or the language where agents fail.
  Changing the language needs a new decision entry.
- [ ] Release when the definition of done in VISION.md is met. The version
  number is the maintainer's decision (O-19).
