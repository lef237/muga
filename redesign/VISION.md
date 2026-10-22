# Muga Redesign Vision

Status: settled direction (2026-10-22). Each point refers to a decision in
[DECISIONS.md](./DECISIONS.md).

## In One Sentence

Muga is a small language that AI agents write, people review, and the machine
keeps safe.

The name means "selflessness" (無我). The redesign takes the name literally:
code should not carry the habits of whoever wrote it, person or AI, and it
should be safe to run without trusting its author.

## Why Redesign

By October 2026 the legacy implementation had grown to about 70,000 lines of
Rust in `src/`, plus about 48,000 lines of tests. It was pursuing three large
goals at once:

- top-tier runtime performance through a native backend
- be easy for AI agents to write
- ship complete tooling: packages, several artifact formats, lockfiles, API
  diffing, editor queries, and JSON output

Each decision was reasonable on its own. Together they made the project hard
to finish. The core value, speed plus AI authoring, was still unproven while
the surface kept growing.

Safety became the core value for a simple reason. When a language runtime has
a memory-safety bug, a program running on it can be taken over remotely, no
matter how carefully the program itself was written. Two classic causes are:

- an abnormal exit, such as running out of memory, that skips cleanup
- state that outlives the memory it points to

A runtime implemented in safe Rust, with errors as values and no references,
avoids both classes of bug by construction. The redesign puts that property at
the center of the language instead of leaving it as a side effect.

## The Three Promises

1. **One way to write it.** Each operation has one spelling in the grammar and
   in the standard library. Other spellings are rejected with a diagnostic
   that shows the Muga spelling. The whole language fits in a short guide that
   an AI agent can hold in context.
2. **The signature tells you what it does.** Parameters, return type, failure,
   and authority (files, commands, environment) all appear in a function's
   signature. A reviewer can tell what a function is able to do without
   reading its body.
3. **Only the authority you grant.** A program can perform I/O only through
   capability values that `main` receives. `main` receives only what the person
   running it grants on the command line. The language is memory safe and has
   no undefined behavior, no null, and no exceptions. Even a wrong or
   compromised program is limited to the authority it was given.

## Priorities

When goals conflict, safety wins (D-01). Speed is a constraint, not a goal:
the v1 bar is "fast enough for a scripting language".

## Scope Of v1

v1 is for command-line and automation tools (D-02). These tools read and write
files, process JSON, run external commands, and use standard input and output
and environment variables.

Out of scope for v1:

- a native backend, or any goal of competing on raw performance (D-01, D-23)
- networking; use `--exec curl` when HTTP is needed (D-05)
- concurrency
- user-defined generic types, constraints, and traits (D-06)
- external packages, a registry, manifests, lockfiles, and artifact formats
  (D-14)
- JSON output, a language server, documentation generation, and build
  artifacts (D-15)

## Definition Of Done For v1

v1 is done when an AI agent, given only the one-page guide, solves a suite of
about ten representative CLI tasks, each run with the minimum necessary grants
(D-24). Features that no task needs are not part of v1.

Implementation guardrails:

- safe Rust only (`unsafe_code = "forbid"`) and no external crates
- a tree-walking interpreter over a typed intermediate representation (D-23)
- a target size of roughly 10,000 to 20,000 lines of Rust, excluding tests;
  going past that is a signal to cut scope, not to keep going
