# Bytes SHA-256 Hash

Status: `std::hash::sha256_hex(bytes)` is implemented for opaque
`std::bytes::Bytes`.

This slice gives file/resource tools one deterministic digest helper that
matches Muga's existing package and app archive integrity model, without adding
signing, HMAC, KDFs, checksum families, or streaming hash state.

## Goals

Short-Term Goal: let Muga programs compute a stable SHA-256 hex digest for bytes
read from files or package resources.

Medium-Term Goal: make local distribution and CI scripts able to compare byte
content with the same lowercase hex convention used by `.mgp` and `.mga`
metadata.

Long-Term Goal: keep broader cryptographic APIs behind a dedicated security
policy instead of growing ad hoc helpers.

Final Goal: make Muga practical for small verification tools while preserving
the narrow standard-library contract.

## Implemented Contract

`std::hash` exports:

```txt
pub fn sha256_hex(bytes: bytes::Bytes): String
```

The return value is exactly 64 lowercase hexadecimal characters. It does not
include the `sha256:` prefix used by archive metadata; callers can add that
prefix explicitly when needed.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| `hash::sha256_hex(bytes): String` | Directly composes with `fs::read_bytes` and package resource bytes, and matches existing archive hash internals. | Full-file hashing only; no streaming state. | Select |
| `hash::sha256(bytes): String` returning `sha256:<hex>` | Matches manifest/archive fields exactly. | Hides formatting policy in the hash function name and is less reusable. | Defer |
| `bytes::to_hex(bytes)` | Useful for debugging arbitrary bytes. | Not a digest and can produce very large strings. | Defer |
| Streaming hash handles | Needed for huge files. | Requires resource lifetime, update/finalize states, and error policy. | Defer |
| HMAC/KDF/signing APIs | Useful for security protocols. | Requires a broader cryptographic API and misuse policy. | Reject for this slice |

## Non-Goals

This slice does not add:

- hash handles or streaming updates;
- HMAC, KDFs, signatures, or key management;
- non-SHA-256 algorithms;
- checksum families;
- archive verification policy changes.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `standard_hash_sha256_hex_hashes_read_bytes_for_source_and_built_runs`

Release-readiness coverage keeps this document, `std::hash`, builtin typing,
runtime behavior, specs, and public docs aligned.
