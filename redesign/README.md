# Muga Redesign

Status: design phase, started 2026-10-22.

Muga is being redesigned from scratch. This directory is the single source of
truth for the new language and for handing work over between machines, people,
and coding agents.

The code currently on `main` is the legacy implementation (the `v0.6.x` line).
It stays in place until the replacement step in [PLAN.md](./PLAN.md). The
legacy [ROADMAP.md](../ROADMAP.md) is suspended.

## Start Here

Read these documents in order:

1. [VISION.md](./VISION.md): why the redesign exists, the three promises, what
   v1 leaves out, and when v1 is done.
2. [DECISIONS.md](./DECISIONS.md): every settled design decision, with the
   alternatives considered and the reasons.
3. [SPEC.md](./SPEC.md): the language and tool specification draft assembled
   from those decisions, including the open questions.
4. [PLAN.md](./PLAN.md): the work order and the resume cursor that names the
   next task.

## Rules For Continuing The Work

- Start each session by reading the resume cursor in PLAN.md. End each session
  by updating it, so the next person or agent can continue without context
  from earlier conversations.
- Treat DECISIONS.md as settled. Do not reopen a decision without explicit
  maintainer approval. Record an approved change as a new entry that
  supersedes the old one instead of rewriting the old entry.
- When the specification needs a choice that DECISIONS.md does not cover, add
  it to the open questions in SPEC.md and ask the maintainer:
  - Ask one question at a time.
  - Offer two to four concrete options with short examples.
  - Recommend one option and give the reason.
  - For keyword and syntax choices, show how people coming from other
    languages could misread each option. Prefer options where a misreading
    fails as a compile error with a suggested fix over options where it
    silently changes meaning (see D-17).
- Do not continue work from the legacy ROADMAP.md, LANGUAGE.md, or `spec/`
  directory. They describe the legacy implementation.
- Keep these documents in English, like the rest of the repository.
- Do not name other languages as rivals or as cautionary examples, for
  example "faster than X" or "X had this vulnerability". State goals and
  lessons in general terms. Neutral design comparisons, such as "as in early
  Go" or the keyword table in D-17, are fine.
- Commit messages use short imperative English subject lines, matching the
  existing history. Do not add `Co-Authored-By` trailers.
- Pull request descriptions must not include generated-by or tool attribution
  footers.
- `AGENTS.md` is intentionally gitignored for local agent instructions. Local
  agent configuration should point to this directory rather than duplicate
  these rules.
