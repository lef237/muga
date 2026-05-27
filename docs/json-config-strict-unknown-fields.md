Status: JSON/config strict unknown-field policy implemented

# JSON/Config Strict Unknown-Field Policy

Muga's JSON/config decoders now support practical concrete model shapes and
explicit wire names through `@json(rename: "...")`. The remaining trust gap is
unknown object keys: records currently ignore extra JSON fields, which is useful
for loose compatibility but unsafe for application config and closed API
contracts.

This implemented slice adds an opt-in record-level strictness policy. It keeps existing
programs permissive, but lets a record declaration say that decoded JSON/config
objects must not contain keys outside the record's accepted wire-key set.

The alias metadata design in
[json-config-alias-metadata.md](json-config-alias-metadata.md) extends strict
accepted-key sets with input aliases while keeping primary wire names
canonical.

## Goals

Short-Term Goal: let config and API boundary records reject misspelled or stale
JSON object keys with path-aware errors.

Medium-Term Goal: make one accepted-key-set rule reusable by future aliases,
TOML, CLI schemas, schema/client generation, and diagnostics.

Long-Term Goal: make public Muga record types trustworthy external contracts
without making the language surprising for compatibility-oriented decoders.

## Syntax Decision

Use a record-level `@json` flag:

```muga
@json(deny_unknown_fields)
record Settings {
  @json(rename: "server_host")
  serverHost: String
  port: Int
}
```

This requires extending the static attribute argument grammar from only named
string arguments to both flags and named string arguments:

- `@json(deny_unknown_fields)` is a flag argument on record declarations;
- `@json(rename: "server_host")` remains a named string argument on fields and
  enum variants;
- unannotated records keep the current permissive behavior.

The flag is intentionally record-level, not call-site-level. Source execution,
package consumers, and artifact-backed execution should all observe the same
public record contract.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| Record-level `@json(deny_unknown_fields)` | Local, explicit, compatible with existing `@json(rename: "...")`, reusable by `json::decode_or[T]`, strict `json::decode[T]`, `config::load_json_or[T]`, TOML, CLI schemas, and schema generation. | Requires record declaration attributes, flag-argument parsing, schema/interface/artifact metadata, and runtime unknown-key checks. | Select |
| Make all record decoders strict by default | Catches mistakes without annotations. | Breaks existing permissive payloads and makes `json::decode_or[T]` overlays less migration-friendly. | Reject |
| Add separate strict decode functions or call options | Avoids syntax changes. | Splits the public type contract from the call site; package interfaces and artifacts could decode the same record differently depending on caller behavior. | Reject |
| Field-level unknown-key markers | Gives fine-grained control. | Unknown keys are a property of the object, not a field. Field-level syntax would be ambiguous and difficult to explain. | Reject |
| Defer to future validation attributes | Could unify all validation work. | Too broad for the immediate trust gap; unknown-key rejection is structural and should be settled before range/regex/custom validation. | Defer |

## Attribute Validation

Parser and AST changes:

- add `attributes: Vec<Attribute>` to `RecordDecl`;
- let `AttributeArgument` carry either a flag or a named string value;
- parse top-level attributes before record declarations in source and package
  mode, preserving the existing `@test` function-only behavior;
- accept `@json(deny_unknown_fields)` only on record declarations;
- accept `@json(rename: "...")` only on record fields and enum variants;
- reject duplicate `@json` attributes on the same target with `P014`;
- reject multiple or mixed arguments in the first slice, such as
  `@json(deny_unknown_fields, rename: "...")`;
- keep `@json(rename: "...")` string validation unchanged: non-empty and no tab
  or newline characters.

The formatter should place record-level attributes immediately above the record
declaration, matching function `@test` and field/variant `@json(rename)` style.

## Accepted-Key Semantics

For a strict record schema, the accepted key set is the effective JSON wire name
for each field:

- a field with `@json(rename: "server_host")` accepts `server_host`;
- an unrenamed field accepts its Muga field name;
- duplicate effective wire names remain a type-checker error;
- future aliases should extend this accepted set without changing the strict
  unknown-field algorithm.

Strictness composes by schema:

- a strict outer record checks only the keys in that outer JSON object;
- nested records check according to their own record declaration metadata;
- typed `Map[String, T]` and raw `Map[String, json::Value]` values remain open
  maps and are not affected by record strictness;
- enum representation stays unchanged, but record payloads inside enum variants
  apply their own strictness.

## Runtime Behavior

Apply the policy consistently to:

- `json::decode_or[T](value, fallback)`;
- strict `json::decode[T](value)`;
- `config::load_json_or[T](path, fallback)`;
- source execution, artifact-backed execution, and `run --built`.

The object shape check still happens first. Once the input is known to be an
object, a strict record checks every incoming key against the accepted set before
returning a decoded value. `json::decode_or[T]` keeps fallback behavior for
missing fields, but it should still reject unknown keys when the record opts in.

Diagnostics should use the external key path:

- unexpected top-level key: `unexpected JSON field `server_portt` at path
  .server_portt`;
- unexpected nested key: `unexpected JSON field `scalee` at path
  .next_action.scalee`.

The error remains a recoverable `json::Error` or `config::Error` decode failure,
not a hard runtime diagnostic, because malformed input is user data.

## Metadata Pipeline

The implementation should add a single boolean strictness bit:

- AST: `RecordDecl.attributes`;
- typing: `RecordDef.json_deny_unknown_fields: bool`;
- typed HIR/package signatures: record-level `json_deny_unknown_fields`;
- `JsonDecodeSchema::Record`: add `deny_unknown_fields: bool`;
- MIR and bytecode: preserve the schema bit through existing schema lowering;
- package interfaces: persist record strictness in public `.mgi` metadata;
- `.mgb` implementation artifacts: persist strictness in decoder schema text;
- runtime: reject unknown object keys only when the schema bit is set.

## Package Interface Format

Bump the persisted interface header and keep legacy interfaces readable:

- legacy v5 record lines remain valid and imply `json_deny_unknown_fields =
  false`;
- current v6 record lines append a record JSON flags token after `field_count`;
- bit `1` means `deny_unknown_fields`;
- reject unknown flag bits in new interfaces rather than silently accepting
  unsupported schema semantics.

Field and variant lines keep the optional rename column introduced by
`@json(rename: "...")`.

## Artifact Tokens

Keep existing permissive record tokens readable:

- `R ...` remains permissive with no explicit wire names;
- `RA ...` remains permissive with explicit wire names;
- add `RF <type_symbol> <flags> <field_count> <field_name> <wire_name>
  <field_schema>...` for records that need record-level JSON flags.

`RF` always writes each field's effective wire name, even when it equals the
source field name. Bit `1` of `flags` is `deny_unknown_fields`; unknown flag bits
are malformed implementation artifacts. Non-strict records should keep emitting
`R` or `RA` to minimize artifact churn.

Enums do not need a strictness token in this slice.

## Tests

Design the implementation around these anchors:

- parser accepts `@json(deny_unknown_fields)` on records;
- parser rejects `@json(deny_unknown_fields)` on fields, variants, functions,
  and enums;
- formatter preserves record-level `@json(deny_unknown_fields)`;
- strict `json::decode[T]` rejects an unknown renamed top-level key with a
  path-aware `json::Error`;
- `json::decode_or[T]` rejects unknown keys even when fallback covers missing
  fields;
- unannotated records continue to ignore unknown keys;
- nested strict records report nested paths;
- `config::load_json_or[T]` maps strict unknown-field failures to
  `config::ErrorKind::Decode`;
- package interfaces preserve record strictness without dependency source;
- `.mgb` artifacts and `run --built` preserve strictness through `RF` schemas;
- malformed artifacts reject unknown `RF` flag bits.

## Deferred Work

- automatic case conversion or whole-record rename rules;
- validation attributes such as ranges, regexes, length checks, required-field
  annotations, or custom validators;
- TOML/YAML/JSON5 parsing;
- full CLI parser schemas/help generation;
- schema/client generation;
- generic record or generic enum decoding;
- host-effect APIs such as `Bytes`, process, network, and streams.

## Implementation Plan

1. Done: implement structural and enum JSON/config decoder targets.
2. Done: implement `@json(rename: "...")` on record fields and enum variants.
3. Done: audit post-rename adoption and select strict unknown-field policy.
4. Done: design record-level `@json(deny_unknown_fields)` syntax, accepted-key
   semantics, metadata flow, package-interface persistence, artifact tokens,
   runtime behavior, and tests.
5. Done: implement the strict unknown-field policy across parser/AST,
   formatter, typing, typed HIR, package signatures, package interfaces,
   decoder schemas, artifacts, runtime, source/artifact/`run --built` tests,
   docs, and release readiness.
6. Done: audit post-strict unknown-field adoption and select alias metadata
   design as the highest-payoff compatibility slice.
7. Done: design `@json(alias: "...")` syntax, accepted-key interaction,
   duplicate/conflict policy, package-interface and artifact payloads, runtime
   diagnostics, tests, and docs.
8. Done: implement alias metadata across parser/formatter, typing, interfaces,
   artifacts, runtime, tests, docs, and release readiness.
9. Next: audit whether validation attributes, TOML, full CLI schemas,
   schema/client generation, generic decoding, or host APIs offer the highest
   practical payoff.
