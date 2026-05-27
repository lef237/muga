Status: JSON/config validation attributes implemented

# JSON/Config Validation Attributes

Muga's JSON/config decoder now handles concrete typed shapes, explicit wire
names, strict unknown-field rejection, input aliases, and field-level scalar
validation. The implemented validation metadata slice fails decoding early for
well-typed but invalid data while keeping the same contract available to future
TOML, CLI schemas, generated docs, and generated schemas.

## Goals

Short-Term Goal: let record fields declare simple scalar invariants that become
path-aware `json::decode[T]`, `json::decode_or[T]`, and
`config::load_json_or[T]` errors.

Medium-Term Goal: persist validation metadata through package interfaces and
artifacts so dependency consumers, source execution, and `run --built` decode
the same contract without provider source.

Long-Term Goal: make Muga's data models practical for production config and API
boundaries by keeping names, compatibility aliases, strictness, and value
constraints together in auditable source.

## Syntax Decision

Use a general field-level `@validate(...)` attribute, separate from `@json(...)`:

```muga
@json(deny_unknown_fields)
record Settings {
  @json(rename: "server_host", alias: "host")
  @validate(non_empty, max_len: 255)
  host: String

  @validate(min: 1, max: 65535)
  port: Int
}
```

The first slice supports record fields only. Variant-payload validation,
cross-field validation, and custom validators stay deferred.

The parser should extend attribute argument values beyond strings so validation
can use natural integer literals:

- flag argument: `non_empty`;
- string argument: existing `rename: "..."`, future pattern-like validators;
- integer argument: `min: 1`, `max: 65535`, `min_len: 1`, `max_len: 255`.

Existing `@json(rename: "...")` and `@json(alias: "...")` string behavior stays
unchanged.

## Syntax Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| Field-level `@validate(...)` | Format-neutral, local to the public field contract, reusable by JSON, future TOML, CLI schemas, generated docs, and generated schemas. Keeps `@json(...)` focused on wire naming and decoding policy. | Requires adding a second allowed field attribute and typed attribute argument values. | Select |
| Put validators inside `@json(...)` | Minimal parser target-surface expansion and keeps all current decoder metadata under one namespace. | Validation is not JSON-specific; TOML and CLI schemas would inherit a misleading name or require duplicate metadata. `@json(...)` would become a mixed wire/semantic bag. | Reject |
| Dedicated attributes such as `@min(1)` or `@non_empty` | Very readable for one validator. | Attribute namespace grows quickly and makes shared parsing, persistence, and docs harder. | Reject |
| Refined wrapper types such as `NonEmptyString` | Strong type-level modeling and reusable after validation. | Requires new stdlib types, constructors, ergonomics, and generic derivation before solving the immediate decode-boundary gap. | Defer |
| Manual validation after decode | Already possible and maximally flexible. | Scatters contract logic away from fields, cannot be preserved in `.mgi` / `.mgb`, and cannot drive future TOML/CLI/schema tooling. | Reject |

## First Validator Set

Keep the first implementation scalar and type-specific:

- `String`: `non_empty`, `min_len: Int`, `max_len: Int`;
- `Int`: `min: Int`, `max: Int`;
- `Option[String]` and `Option[Int]`: apply the same validators only when the
  value is `Option::Some`; `Option::None` remains valid;
- `Bool`: no validators in the first slice.

Defer regex/pattern validators, list/map length validators, enum variant
validators, required/default annotations, deprecation warnings, custom
validators, and cross-field validation.

Type checking should reject:

- validators unsupported by the field's effective target type;
- negative `min_len` or `max_len`;
- `min > max` and `min_len > max_len`;
- duplicate validators with conflicting values;
- `@validate(...)` on records, enum variants, functions, locals, or parameters
  in the first slice.

## Runtime Semantics

Validation runs after structural decoding of a field succeeds. This keeps shape
errors and validation errors distinct:

- wrong JSON shape: existing messages such as `expected JSON Int at path .port`;
- invalid decoded value: new messages such as
  `validation failed at path .port: expected Int >= 1`.

The first slice should stop at the first validation failure, matching the
current decoder's first-error behavior. Error accumulation can be designed later
for richer tooling.

The implementation adds a public `json::ErrorKind::Validation` variant and
matching runtime `JsonErrorKind::Validation`. `config::load_json_or[T]`
continues mapping validation failures to `config::ErrorKind::Decode` while
preserving the JSON validation message and offset `-1`.

## Metadata Pipeline

The implementation adds validation metadata beside field JSON naming:

- AST: extend attribute values to string/int/flag forms;
- parser: accept `@validate(...)` only on record fields in the first slice;
- typing: attach validated field metadata after checking type compatibility;
- typed HIR/package signatures: preserve validation rules on record fields;
- `JsonDecodeFieldSchema`: add `validation: Vec<JsonDecodeValidationRule>`;
- MIR and bytecode: preserve validation rules through schema lowering;
- runtime: validate decoded field values before constructing the record value;
- package interfaces: persist public validation rules and keep v7 interfaces
  readable with empty validation rules;
- `.mgb` artifacts: persist validation rules in decoder schema text.

## Package Interface Format

Validation metadata was introduced with `muga-package-interface-v8`. The current
persisted interface header is `muga-package-interface-v11` after CLI enum
subcommand metadata, and v7/v8/v9 interfaces remain readable with empty
validation or CLI metadata lists.

Use an extended field line only when aliases or validation rules are present:

`field <name> <span> <type> <rename_or_-> <alias_count> <alias>...
<validation_count> <validation>...`

Validation tokens should be stable text such as:

- `non_empty`;
- `min=<int>`;
- `max=<int>`;
- `min_len=<int>`;
- `max_len=<int>`.

## Artifact Tokens

Keep existing decoder artifact tokens readable:

- `R`, `RA`, `RF`, and `RG` remain valid record tokens;
- records without validation should keep emitting the narrowest existing token.

The implementation adds a validation-capable record token:

`RV <type_symbol> <flags> <field_count> <field_name> <wire_name>
<alias_count> <alias>... <validation_count> <validation>... <field_schema>...`

`RV` uses the same record flag bits as `RF` and `RG`; bit `1` remains
`deny_unknown_fields`. Unknown record flag bits and malformed validation payloads
remain malformed artifacts.

Enum validation is deferred, so `E`, `EA`, and `EG` remain unchanged.

## Tests

The implementation is covered by these anchors:

- parser accepts field-level `@validate(non_empty, min_len: 1, max_len: 255)`;
- parser rejects `@validate` on records, enum variants, functions, locals, and
  parameters;
- type checking rejects validators for unsupported target types and impossible
  bounds such as `min > max`;
- formatter preserves validation attributes;
- `json::decode[T]`, `json::decode_or[T]`, and `config::load_json_or[T]` return
  path-aware validation errors;
- `Option[T]` validators apply only to present values;
- package interfaces preserve validation metadata without provider source;
- artifact-backed execution and `run --built` preserve validation behavior;
- malformed validation artifact payloads are rejected.

## Deferred Work

- regex/pattern validators and a regex dependency/versioning policy;
- list/map length validators;
- enum variant payload validators;
- required/default annotations beyond current required decode behavior;
- validation error accumulation;
- custom validator functions;
- cross-field or whole-record validation;
- validation deprecation/warning metadata;
- TOML/YAML/JSON5 parsing;
- full CLI parser schemas/help generation;
- schema/client generation;
- generic record or enum decoding;
- host-effect APIs such as `Bytes`, process, network, and streams.

## Implementation Plan

1. Done: implement structural and enum JSON/config decoder targets.
2. Done: implement `@json(rename: "...")`, `@json(deny_unknown_fields)`, and
   `@json(alias: "...")` metadata.
3. Done: audit post-alias adoption and select validation attribute design.
4. Done: design the validation attribute syntax, first validator set, runtime
   error policy, package-interface format, artifact token, and test anchors.
5. Done: implement the smallest validation attribute slice across parser,
   formatter, typing, typed HIR, package signatures, package interfaces,
   decoder schemas, artifacts, runtime, source/artifact/`run --built` tests,
   docs, and release readiness.
6. Done: audit the post-validation JSON/config adoption gap and select schema
   export design in
   [json-config-schema-export.md](json-config-schema-export.md).
7. Done: design JSON/config schema export in
   [json-config-schema-export.md](json-config-schema-export.md).
8. Done: implement the smallest schema export slice in
   [json-config-schema-export.md](json-config-schema-export.md).
9. Done: audit the post-schema-export JSON/config adoption gap in
   [json-typed-encoding.md](json-typed-encoding.md).
10. Done: design typed JSON encoding in
    [json-typed-encoding.md](json-typed-encoding.md).
11. Done: implement the smallest typed JSON encoding slice in
    [json-typed-encoding.md](json-typed-encoding.md).
12. Done: audit typed JSON encoding adoption in
    [cli-parser-schema.md](cli-parser-schema.md)
    and select full CLI parser schema design next.
13. Done: design full CLI parser schemas in
    [cli-parser-schema.md](cli-parser-schema.md).
14. Next: implement the smallest CLI parser schema overlay before TOML, full
    client generation, generic encoding/decoding, broader validators, config
    discovery automation, or host effects.
