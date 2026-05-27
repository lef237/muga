Status: JSON/config field and variant rename implemented

# JSON/Config Schema Polish

Muga now decodes practical concrete JSON/config models: scalars, options,
recursive lists, typed string maps, records, and enums. This schema-polish slice
makes those models fit external wire shapes without manual `json::Value`
traversal.

The implemented first slice is explicit field and variant wire-name metadata:
`@json(rename: "...")` on record fields and enum variants. Record-level
`@json(deny_unknown_fields)` is implemented as the next schema-polish slice, and
field/variant `@json(alias: "...")` metadata is implemented for legacy input
names. Validation attributes, automatic case conversion, TOML, full CLI schemas,
schema/client generation, and generic decoding remain separate follow-up
decisions.

[json-config-strict-unknown-fields.md](json-config-strict-unknown-fields.md):
record-level `@json(deny_unknown_fields)`, accepted wire-key semantics,
path-aware unknown-key errors, `.mgi` record flags, and `RF` decoder artifact
tokens now work, with alias metadata layered on top before validation
attributes, TOML, full CLI schemas, schema generation, generic decoding, or host
effects.
The alias metadata design in
[json-config-alias-metadata.md](json-config-alias-metadata.md) selects a single
field/variant `@json(...)` attribute with repeated `alias: "..."` arguments and
defines accepted-name conflicts and is now implemented across parser,
formatter, typing, interfaces, artifacts, runtime, tests, and docs.

## Goals

Short-Term Goal: decode common JSON/config names such as `server_host`,
`max-retries`, or lowercase enum tags into idiomatic Muga field and variant
names.

Medium-Term Goal: establish one compiler-owned schema metadata path that future
TOML, CLI schemas, generated schema metadata, and config tooling can reuse.

Long-Term Goal: make Muga's public record and enum types usable as practical
external contracts while keeping the language predictable and easy to audit.

## Syntax Decision

Use a single compiler-recognized `@json` attribute with named options:

```muga
record Settings {
  @json(rename: "server_host")
  serverHost: String
}

enum Mode {
  @json(rename: "auto")
  Auto
  @json(rename: "manual")
  Manual
}
```

This extends the existing static-attribute model used by `@test`; it does not
introduce macros, code rewriting, hidden runtime reflection, or user-defined
attributes.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| `@json(rename: "...")` on fields and variants | Explicit, local, stable across JSON/config surfaces, and compatible with future `alias`, `deny_unknown_fields`, TOML, CLI schemas, and schema generation. Keeps default behavior unchanged when absent. | Requires attribute parsing beyond top-level functions plus schema/interface/artifact metadata. | Implemented |
| Separate attributes such as `@json_name("...")` | Slightly simpler parser shape for one option. | Does not scale cleanly to aliases, strict unknown policy, or future format-specific options; encourages attribute proliferation. | Defer |
| Automatic case conversion such as snake_case by default | Low annotation overhead. | Hidden behavior, ambiguous acronyms, and breaking changes for existing payloads that intentionally use Muga identifiers. | Reject |
| External schema declarations detached from types | Could model many formats and transformations. | Too heavy for the current language; separates the contract from the public record/enum definitions users already read. | Defer |
| Runtime helper transforms before decode | Avoids compiler changes. | Reintroduces manual JSON traversal and cannot help `config::load_json_or[T]` or future artifact-backed schema consumers cleanly. | Reject |

## Implemented Surface

The first implementation supports only explicit `rename` metadata for record
fields and enum variants:

- parse `@json(rename: "wire-name")` directly before record fields and enum
  variants;
- keep `@test` function-only and reject unknown attributes or invalid `@json`
  targets with `P014`;
- reject duplicate effective wire names within a record or enum during type
  checking;
- preserve default behavior when no `@json(rename: "...")` attribute exists;
- decode object fields by wire name but construct record values with the Muga
  field name;
- decode enum tags by wire name but construct enum values with the Muga variant
  name;
- keep diagnostics path-aware by reporting the external wire name that appeared
  in or was expected from JSON/config, such as `.server_host` or `.mode.auto`;
- persist schema metadata through package interfaces and implementation
  artifacts so artifact-backed consumers decode imported public types the same
  way as source consumers.

## Metadata Pipeline

The implementation adds schema metadata without changing runtime value shapes:

- AST: field and variant `attributes` are carried on `RecordFieldDecl` and
  `EnumVariantDecl`;
- parser: attribute parsing supports named string options for
  compiler-owned `@json` only on supported targets;
- typing: computes effective wire names for decoder schemas and rejects duplicate
  wire names inside one record or enum;
- typed HIR/MIR/bytecode: carry effective wire-name symbols in
  `JsonDecodeFieldSchema` and `JsonDecodeVariantSchema`;
- package interfaces: persist public field/variant wire-name metadata so
  downstream packages and artifact-backed checks use the same decoder schema;
- `.mgb` artifacts: keep existing `R` and `E` schema tokens backward-compatible
  and add annotated schema tokens for renamed fields/variants rather than
  changing the old token shape;
- runtime: looks up JSON object keys and enum tags using the wire name, but
  construct Muga records/enums with source-level names.

## Artifact Tokens

Keep old unrenamed schemas readable:

- `R ...` and `E ...` remain the compact unrenamed forms;
- add annotated forms for schemas with at least one rename:
  - `RA <type_symbol> <field_count> <field_name> <wire_name> <field_schema>...`
  - `EA <type_symbol> <variant_count> <variant_name> <wire_name> <payload_flag> [payload_schema]...`
- both `field_name` / `variant_name` and `wire_name` should be interned symbols
  in the implementation artifact symbol table.

If `wire_name` equals the Muga name for every field or variant, the compiler may
keep emitting `R` or `E` to minimize artifact churn.

## Diagnostics

- Unknown attribute syntax or misplaced `@json` should use `P014`, matching the
  current static-attribute diagnostic family.
- Duplicate record field wire names and duplicate enum variant wire names should
  be type-checker diagnostics because they require resolved declaration context.
- Shape and missing-field errors should use the JSON/config wire path. For
  example, a missing renamed `serverHost` field with wire name `server_host`
  should report `.server_host`.
- Unknown enum tags should report the unknown wire tag at the containing path.

## Deferred Work

- validation attributes such as ranges, regexes, required markers, or custom
  validators;
- automatic case conversion for whole records/enums;
- TOML/YAML/JSON5 parsing;
- full CLI parser schemas/help generation;
- schema/client generation;
- generic record or generic enum decoding;
- host-effect APIs such as `Bytes`, process, network, and streams.

## Implementation Plan

1. Done: implement structural and enum JSON/config decoder targets.
2. Done: audit post-enum adoption and select schema polish design.
3. Done: choose explicit field/variant `@json(rename: "...")` as the first
   schema-polish implementation slice.
4. Done: implement parser/AST/typechecker metadata for field and variant
   renames, with duplicate-wire-name diagnostics.
5. Done: carry wire names through decoder schemas, package interfaces,
   artifacts, runtime decode, source/artifact/`run --built` tests, docs, and
   release readiness.
6. Done: audit post-rename adoption and select strict unknown-field policy
   design as the next practical JSON/config/API boundary.
7. Done: design record-level strict unknown-field policy before aliases, TOML,
   full CLI schemas, schema generation, generic decoding, or host effects.
8. Done: implement record-level `@json(deny_unknown_fields)` across parser/AST,
   formatter, typing, package interfaces, decoder schemas, artifacts, runtime,
   source/artifact tests, docs, and release readiness.
9. Done: audit post-strict unknown-field adoption and select alias metadata
   design as the next JSON/config compatibility slice.
10. Done: design `@json(alias: "...")` metadata before validation attributes,
   TOML, full CLI schemas, schema generation, generic decoding, or host effects.
11. Done: implement alias metadata across parser/formatter, typing, package
   interfaces, decoder schemas, artifacts, runtime, tests, docs, and release
   readiness.
12. Next: audit whether validation attributes, TOML, full CLI schemas,
   schema/client generation, generic decoding, or host APIs offer the highest
   practical payoff.
