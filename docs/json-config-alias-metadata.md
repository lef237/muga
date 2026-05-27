Status: JSON/config alias metadata implemented

# JSON/Config Alias Metadata

Muga's JSON/config decoder now supports explicit primary wire names through
`@json(rename: "...")`, opt-in closed record contracts through
`@json(deny_unknown_fields)`, and input-only field/variant aliases through
`@json(alias: "...")`. This closes the immediate compatibility gap for accepting
legacy external names during schema evolution without changing the canonical
Muga field or enum variant name.

This implemented slice adds input-only alias metadata for record fields and enum
variants.
Aliases extend the accepted-name set used by JSON/config decoding, including
strict unknown-field checks, while keeping the primary wire name canonical for
future encoders and schema generation.

## Goals

Short-Term Goal: let JSON/config records and enum tags accept older wire names
after a rename without manual `json::Value` migration adapters.

Medium-Term Goal: give strict records one accepted-key policy: primary wire name
plus aliases, with deterministic duplicate and conflict diagnostics.

Long-Term Goal: make Muga practical for long-lived config files and API
payloads where contracts evolve but typed source models should stay canonical
and auditable.

## Syntax Decision

Use one `@json(...)` attribute per record field or enum variant, with a compact
argument list:

```muga
record Settings {
  @json(rename: "server_host", alias: "host", alias: "serverHost")
  host: String
}

enum Mode {
  @json(rename: "auto", alias: "automatic")
  Auto
}
```

Rules:

- field and enum-variant `@json` may contain at most one `rename` argument;
- field and enum-variant `@json` may contain zero or more `alias` arguments;
- alias values use the same string rules as `rename`: non-empty and no tab or
  newline characters;
- duplicate `@json` attributes on one field or variant remain rejected;
- record declarations continue to support only `@json(deny_unknown_fields)`.

## Syntax Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| Single `@json(rename: "...", alias: "...", alias: "...")` attribute | Keeps all JSON wire metadata for a field or variant in one local attribute, reuses the current argument parser, allows multiple aliases without a new list literal grammar, and preserves the existing duplicate-attribute rule. | Long alias lists can make one line wide; formatter wrapping can be a later polish item. | Select |
| Repeated `@json(alias: "...")` attributes | Reads naturally one alias per line and avoids duplicate argument names. | Requires relaxing the current duplicate `@json` validation only for aliases and makes mixed `rename`/alias ordering less obviously one contract. | Reject |
| `@json(aliases: ["a", "b"])` list argument | Compact for many aliases. | Attribute arguments do not have list values; adding list grammar just for aliases is more surface than the feature needs. | Reject |
| Separate `@json_alias("...")` attribute | Simple to parse independently. | Fragments the JSON metadata namespace and scales poorly with future options. | Reject |
| Automatic case conversion | Covers common `snake_case` or `camelCase` migrations without per-field metadata. | Too broad and implicit for public contracts; explicit aliases are easier to audit and package through interfaces. | Defer |

## Accepted-Name Semantics

For each field or enum variant:

- the primary wire name is `@json(rename: "...")` when present, otherwise the
  source field or variant name;
- aliases are accepted input names only;
- the accepted-name set is primary wire name plus aliases;
- decoded records use source field names and decoded enums use source variant
  names regardless of which accepted name matched.

Type checking should reject ambiguous accepted-name sets:

- one field or variant cannot repeat an alias;
- an alias cannot equal that target's primary wire name;
- two fields in the same record cannot share any primary or alias name;
- two variants in the same enum cannot share any primary or alias name.

These are source-level and loaded-interface compatibility errors. They should
point at the conflicting `@json` argument and include a related note for the
previous accepted name when source spans are available.

## Decode Conflict Policy

Aliases are accepted inputs, but they should not introduce hidden precedence.

For records, if a single JSON object contains more than one accepted key for the
same field, decoding returns a recoverable `json::Error` or `config::Error`.
This applies to `json::decode_or[T]`, `json::decode[T]`, and
`config::load_json_or[T]`, even when the record is not strict. A payload that
contains both `server_host` and `host` for one field is ambiguous user data.

For strict records, unknown-field rejection uses the full accepted-name set.
Known aliases are not unknown fields; unrelated keys still fail with the
existing path-aware unknown-field error.

For enum targets, string tags and one-payload object keys match either the
primary wire name or any alias. The duplicate accepted-name type-checker rule
prevents ambiguous variant resolution.

## Metadata Pipeline

The implementation adds alias lists beside existing rename metadata:

- AST: no new grammar node is needed; aliases are `@json` arguments;
- typing: record fields and enum variants carry `json_aliases: Vec<Symbol>`;
- typed HIR/package signatures: preserve alias symbols with fields and
  variants;
- `JsonDecodeFieldSchema` and `JsonDecodeVariantSchema`: add
  `aliases: Vec<Symbol>`;
- MIR and bytecode: preserve alias lists through existing schema lowering;
- runtime: match incoming object keys and enum tags against primary wire name
  plus aliases and detect per-field alias conflicts;
- package interfaces: persist public aliases and keep legacy interfaces
  readable;
- `.mgb` artifacts: persist alias metadata in decoder schema text.

## Package Interface Format

Alias metadata was introduced with `muga-package-interface-v7`. The current
persisted interface header is `muga-package-interface-v11` after validation and
CLI enum subcommand metadata, and v6/v7/v8/v9 interfaces remain readable with
empty alias, validation, or CLI metadata lists as needed.

To minimize churn, only fields or variants with aliases need the extended line
shape:

- legacy field line without rename: `field <name> <span> <type>`;
- legacy field line with rename: `field <name> <span> <type> <rename>`;
- alias field line:
  `field <name> <span> <type> <rename_or_-> <alias_count> <alias>...`;
- legacy variant line without rename: `variant <name> <span> <payload>`;
- legacy variant line with rename: `variant <name> <span> <payload> <rename>`;
- alias variant line:
  `variant <name> <span> <payload> <rename_or_-> <alias_count> <alias>...`.

`-` means no explicit primary rename. Alias counts must match the remaining
columns. Loaded v6 and older interfaces imply no aliases.

## Artifact Tokens

Keep existing decoder artifact tokens readable:

- `R` remains a permissive record token with no explicit wire names or aliases;
- `RA` remains a permissive record token with explicit primary wire names;
- `RF` remains a record token with flags and explicit primary wire names;
- `E` remains an enum token with no explicit wire names or aliases;
- `EA` remains an enum token with explicit primary wire names.

Add alias-capable general tokens:

- `RG <type_symbol> <flags> <field_count> <field_name> <wire_name>
  <alias_count> <alias>... <field_schema>...`;
- `EG <type_symbol> <variant_count> <variant_name> <wire_name> <alias_count>
  <alias>... <has_payload> <payload_schema>? ...`.

`RG` uses the same record flag bits as `RF`; bit `1` remains
`deny_unknown_fields`. Unknown record flag bits remain malformed artifacts.
Records without aliases should keep emitting `R`, `RA`, or `RF` to avoid
artifact churn. Enums without aliases should keep emitting `E` or `EA`.

## Tests

The implementation is covered by these anchors:

- parser accepts one field/variant `@json` attribute with `rename` and repeated
  `alias` arguments;
- parser rejects `alias` on record declarations and `deny_unknown_fields` on
  fields/variants;
- type checking rejects duplicate aliases, alias-primary collisions, and
  cross-field/cross-variant accepted-name collisions;
- formatter preserves alias arguments;
- `json::decode[T]` and `json::decode_or[T]` accept record field aliases;
- strict records treat aliases as known keys and still reject unrelated keys;
- decoding rejects an object that contains both a primary and alias for one
  field;
- enum string tags and one-payload object keys accept aliases;
- package interfaces preserve aliases without provider source;
- artifact-backed execution and `run --built` preserve alias semantics;
- malformed alias artifact payloads are rejected.

## Deferred Work

- validation attributes such as ranges, regexes, required markers, or custom
  validators;
- deprecation metadata or warnings for legacy aliases;
- automatic case conversion for whole records/enums;
- JSON encoding from typed records/enums;
- TOML/YAML/JSON5 parsing;
- full CLI parser schemas/help generation;
- schema/client generation;
- generic user record or enum decoding;
- host-effect APIs such as `Bytes`, process, network, streams, or broader
  resource handles.

## Implementation Plan

1. Done: implement field/variant `@json(rename: "...")` metadata.
2. Done: implement record-level `@json(deny_unknown_fields)`.
3. Done: audit post-strict adoption and select alias metadata design.
4. Done: design alias syntax, accepted-name semantics, conflict policy,
   metadata flow, package-interface persistence, artifact tokens, runtime
   behavior, and tests.
5. Done: implement the smallest alias metadata slice across parser/formatter,
   typing, typed HIR, package signatures, package interfaces, decoder schemas,
   artifacts, runtime, source/artifact/`run --built` tests, docs, and release
   readiness.
6. Done: re-audit post-alias adoption and select validation attribute design in
   [json-config-validation-attributes.md](json-config-validation-attributes.md).
7. Done: design the smallest validation attribute slice in
   [json-config-validation-attributes.md](json-config-validation-attributes.md).
8. Done: implement validation attributes in
   [json-config-validation-attributes.md](json-config-validation-attributes.md).
9. Done: audit the post-validation JSON/config adoption gap and select schema
   export design in
   [json-config-schema-export.md](json-config-schema-export.md).
10. Done: design JSON/config schema export in
    [json-config-schema-export.md](json-config-schema-export.md).
11. Done: implement the smallest schema export slice in
    [json-config-schema-export.md](json-config-schema-export.md).
12. Done: audit the post-schema-export JSON/config adoption gap in
    [json-typed-encoding.md](json-typed-encoding.md).
13. Done: design typed JSON encoding in
    [json-typed-encoding.md](json-typed-encoding.md).
14. Done: implement the smallest typed JSON encoding slice in
    [json-typed-encoding.md](json-typed-encoding.md).
15. Done: audit typed JSON encoding adoption in
    [cli-parser-schema.md](cli-parser-schema.md)
    and select full CLI parser schema design next.
16. Done: design full CLI parser schemas in
    [cli-parser-schema.md](cli-parser-schema.md).
17. Next: implement the smallest CLI parser schema overlay before TOML, full
    client generation, generic encoding/decoding, broader validators, config
    discovery automation, or host effects.
