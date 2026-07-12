# Muga Conformance Fixtures

This directory contains executable fixtures for the evolving v1 language
contract. Until Muga reaches `1.0.0`, these fixtures pin the current candidate
behavior but may change together with the specifications. After v1, they become
the compatibility suite that future compiler versions and alternate
implementations must preserve. The suite is intentionally separate from
`samples/`: samples teach the language. Diagnostic sections in
[errors.md](../errors.md) link to rejecting fixtures here as their referenced
fixtures.

The initial skeleton is tied to:

- `spec-v1.md`
- `spec/001-core-language.md`
- `spec/002-name-resolution.md`
- `spec/003-typing.md`
- `spec/004-functions.md`
- `spec/005-records.md`
- `spec/006-packages.md`

Layout:

- `v1/valid/`: runnable script programs. Each fixture declares
  `// expect-main: <value>`.
- `v1/rejecting/`: invalid script programs. Each fixture declares
  `// expect-error: <code>`.
- `v1/package-artifacts/`: package-mode fixtures that must work through
  source-compatible execution and explicit artifact-backed execution.

Keep this suite small and spec-shaped. Broad regression coverage can remain in
`tests/examples.rs`; conformance fixtures should be representative and stable.
