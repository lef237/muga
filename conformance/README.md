# Muga Conformance Fixtures

This directory contains executable fixtures for the current language contract.
The fixtures change together with the living specifications and compiler. A
future stable compatibility suite may be snapshotted when Muga reaches
`1.0.0`; the `current/` tree itself continues to describe current behavior.
The suite is intentionally separate from `samples/`: samples teach the
language. Diagnostic sections in
[errors.md](../errors.md) link to rejecting fixtures here as their referenced
fixtures.

The initial skeleton is tied to:

- `LANGUAGE.md`
- `spec/001-core-language.md`
- `spec/002-name-resolution.md`
- `spec/003-typing.md`
- `spec/004-functions.md`
- `spec/005-records.md`
- `spec/006-packages.md`

Layout:

- `current/valid/`: runnable script programs. Each fixture declares
  `// expect-main: <value>`.
- `current/rejecting/`: invalid script programs. Each fixture declares
  `// expect-error: <code>`.
- `current/package-artifacts/`: package-mode fixtures that must work through
  source-compatible execution and explicit artifact-backed execution.

Keep this suite small and spec-shaped. Broad regression coverage can remain in
`tests/examples.rs`; conformance fixtures should be representative and stable.
