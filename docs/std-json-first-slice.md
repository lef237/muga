# Std Json First Slice

Status: design and implementation contract for the first `std::json` slice.
This document does not implement the package by itself; the implementation
follows this boundary.

The purpose of the first `std::json` slice is to parse and encode JSON through
explicit Muga data types and `Result`, without adding new syntax, hidden
exceptions, reflection, schema generation, HTTP support, `Float`, `Decimal`,
`Bytes`, streaming IO, or resource handles.

## Scope

The first slice should add one compiler-provided package:

```muga
import std::json
```

Public API:

```muga
pub enum Value {
  Null
  Bool(Bool)
  Number(Number)
  String(String)
  Array(List[Value])
  Object(Map[String, Value])
}

pub enum Number {
  Int(Int)
  Raw(String)
}

pub enum ErrorKind {
  UnexpectedEnd
  UnexpectedToken
  InvalidEscape
  InvalidNumber
  NumberOutOfRange
  DuplicateKey
  TrailingCharacters
  NestingLimitExceeded
}

pub record Error {
  kind: ErrorKind,
  message: String,
  offset: Int
}

pub enum PathSegment {
  Field(String)
  Index(Int)
}

pub fn parse(text: String): Result[Value, Error]
pub fn encode(value: Value): Result[String, Error]
pub fn number_as_int(number: Number): Result[Int, Error]
pub fn int(value: Int): Value
pub fn as_bool(value: Value): Result[Bool, Error]
pub fn as_string(value: Value): Result[String, Error]
pub fn as_number(value: Value): Result[Number, Error]
pub fn as_int(value: Value): Result[Int, Error]
pub fn as_array(value: Value): Result[List[Value], Error]
pub fn as_object(value: Value): Result[Map[String, Value], Error]
pub fn at(value: Value, path: List[PathSegment]): Result[Option[Value], Error]
pub fn at_required(value: Value, path: List[PathSegment]): Result[Value, Error]
pub fn at_string(value: Value, path: List[PathSegment]): Result[Option[String], Error]
pub fn at_string_or(value: Value, path: List[PathSegment], default_value: String): Result[String, Error]
pub fn at_string_required(value: Value, path: List[PathSegment]): Result[String, Error]
pub fn at_int(value: Value, path: List[PathSegment]): Result[Option[Int], Error]
pub fn at_int_or(value: Value, path: List[PathSegment], default_value: Int): Result[Int, Error]
pub fn at_int_required(value: Value, path: List[PathSegment]): Result[Int, Error]
pub fn at_bool(value: Value, path: List[PathSegment]): Result[Option[Bool], Error]
pub fn at_bool_or(value: Value, path: List[PathSegment], default_value: Bool): Result[Bool, Error]
pub fn at_bool_required(value: Value, path: List[PathSegment]): Result[Bool, Error]
pub fn at_array(value: Value, path: List[PathSegment]): Result[Option[List[Value]], Error]
pub fn at_array_or(value: Value, path: List[PathSegment], default_value: List[Value]): Result[List[Value], Error]
pub fn at_array_required(value: Value, path: List[PathSegment]): Result[List[Value], Error]
pub fn at_object(value: Value, path: List[PathSegment]): Result[Option[Map[String, Value]], Error]
pub fn at_object_or(value: Value, path: List[PathSegment], default_value: Map[String, Value]): Result[Map[String, Value], Error]
pub fn at_object_required(value: Value, path: List[PathSegment]): Result[Map[String, Value], Error]
pub fn at_string_array(value: Value, path: List[PathSegment]): Result[Option[List[String]], Error]
pub fn at_string_array_or(value: Value, path: List[PathSegment], default_value: List[String]): Result[List[String], Error]
pub fn at_string_array_required(value: Value, path: List[PathSegment]): Result[List[String], Error]
pub fn at_int_array(value: Value, path: List[PathSegment]): Result[Option[List[Int]], Error]
pub fn at_int_array_or(value: Value, path: List[PathSegment], default_value: List[Int]): Result[List[Int], Error]
pub fn at_int_array_required(value: Value, path: List[PathSegment]): Result[List[Int], Error]
pub fn at_bool_array(value: Value, path: List[PathSegment]): Result[Option[List[Bool]], Error]
pub fn at_bool_array_or(value: Value, path: List[PathSegment], default_value: List[Bool]): Result[List[Bool], Error]
pub fn at_bool_array_required(value: Value, path: List[PathSegment]): Result[List[Bool], Error]
pub fn array_strings(values: List[Value]): Result[List[String], Error]
pub fn array_ints(values: List[Value]): Result[List[Int], Error]
pub fn array_bools(values: List[Value]): Result[List[Bool], Error]
pub fn object_get(value: Value, key: String): Result[Option[Value], Error]
pub fn object_array(value: Value, key: String): Result[Option[List[Value]], Error]
pub fn object_array_or(value: Value, key: String, default_value: List[Value]): Result[List[Value], Error]
pub fn object_array_required(value: Value, key: String): Result[List[Value], Error]
pub fn object_string_array(value: Value, key: String): Result[Option[List[String]], Error]
pub fn object_string_array_or(value: Value, key: String, default_value: List[String]): Result[List[String], Error]
pub fn object_string_array_required(value: Value, key: String): Result[List[String], Error]
pub fn object_int_array(value: Value, key: String): Result[Option[List[Int]], Error]
pub fn object_int_array_or(value: Value, key: String, default_value: List[Int]): Result[List[Int], Error]
pub fn object_int_array_required(value: Value, key: String): Result[List[Int], Error]
pub fn object_bool_array(value: Value, key: String): Result[Option[List[Bool]], Error]
pub fn object_bool_array_or(value: Value, key: String, default_value: List[Bool]): Result[List[Bool], Error]
pub fn object_bool_array_required(value: Value, key: String): Result[List[Bool], Error]
pub fn object_object(value: Value, key: String): Result[Option[Map[String, Value]], Error]
pub fn object_object_or(value: Value, key: String, default_value: Map[String, Value]): Result[Map[String, Value], Error]
pub fn object_object_required(value: Value, key: String): Result[Map[String, Value], Error]
pub fn object_bool(value: Value, key: String): Result[Option[Bool], Error]
pub fn object_bool_or(value: Value, key: String, default_value: Bool): Result[Bool, Error]
pub fn object_bool_required(value: Value, key: String): Result[Bool, Error]
pub fn object_string(value: Value, key: String): Result[Option[String], Error]
pub fn object_string_or(value: Value, key: String, default_value: String): Result[String, Error]
pub fn object_string_required(value: Value, key: String): Result[String, Error]
pub fn object_int(value: Value, key: String): Result[Option[Int], Error]
pub fn object_int_or(value: Value, key: String, default_value: Int): Result[Int, Error]
pub fn object_int_required(value: Value, key: String): Result[Int, Error]
```

Deferred from the first slice:

- pretty printing or configurable formatting;
- streaming parse/encode APIs;
- comments, trailing commas, JSON5, YAML, TOML, or config-file behavior;
- direct record/enum derive, reflection, macros, schema generation, OpenAPI, or
  client generation;
- `Float`, `Decimal`, arbitrary precision numbers, or scientific/numeric
  helper APIs beyond `number_as_int` and object integer accessors;
- binary `Bytes`, base64 helpers, HTTP, RPC, or service runtime APIs.

## Result Ergonomics

All recoverable failures return `Result[_, json::Error]`.

```muga
parsed: json::Value = try json::parse(text)
encoded: String = try json::encode(parsed)
```

The API should work naturally with existing `try expr`, `std::result` helpers,
and `muga test` functions that return `Result[Unit, E]`. It must not introduce
postfix propagation syntax, exceptions, panics for user data, or sentinel
strings.

`encode` returns `Result[String, Error]` instead of `String` because the public
`Number::Raw(String)` variant can be constructed by user code. Encoding must
validate raw number text rather than emitting invalid JSON.

`number_as_int` converts either `Number::Int(value)` or an integral
`Number::Raw(text)` into `Int`, returning `NumberOutOfRange` or
`InvalidNumber` for unsupported values.

The post-typed-cli accessor follow-up keeps the same error model. `as_*`
helpers extract typed payloads from `Value`, `object_get` extracts an optional
field from an object, and `object_*` helpers return `Option`, `_or` defaults,
or `_required` values for common configuration fields. The current
scalar/composite object-field helpers cover `Array`, `Object`, `Bool`,
`String`, and integral `Number` values. The scalar array projection helpers
`array_strings`, `array_ints`, and `array_bools` convert `List[Value]` into
typed scalar lists without adding direct object-field scalar-array helper
matrices or schema decoding. The direct scalar-array object-field helpers
`object_string_array*`, `object_int_array*`, and `object_bool_array*` compose
object lookup with scalar array projection while preserving optional/default/
required field behavior. Missing fields are not errors for lookup and `_or`
helpers. Missing required fields return `Error` with `UnexpectedToken`, a
message naming the object field, and `offset = -1` because no source byte
points at an absent key. Wrong base values, present fields with the wrong
shape, and scalar array items with the wrong shape return `Error` with
`UnexpectedToken`; object-field shape errors include the field key, scalar array
projection errors include the item index, and direct scalar-array field errors
include both the field key and item index with `offset = -1`. `object_int`,
`object_int_or`, `object_int_required`, `array_ints`, and `object_int_array*`
reuse `number_as_int`.

The post-repeated-cli-option JSON path follow-up keeps traversal explicit and
typed. `PathSegment::Field(String)` and `PathSegment::Index(Int)` describe a
path without a string path parser, JSON Pointer, JSONPath query syntax,
wildcards, filters, slices, or recursive descent. `at` returns
`Option::None` for a missing object field or out-of-range array index and
returns `json::Error` for wrong shapes while traversing. `at_required` maps
missing paths to `json::Error`. Path-aware diagnostics render deterministic
field/index paths such as `.metadata.owner` and `.items[0]`; these helpers do
not add schema decoding, typed path projection matrices, `std::config`, TOML,
or config-file discovery.

The typed JSON path scalar projection follow-up adds only the typed JSON path
scalar helper family: `at_string*`, `at_int*`, and `at_bool*`. These helpers preserve
optional/default/required missing-path behavior, add path-aware terminal scalar
shape errors such as `expected JSON String at path .metadata.owner`, and reuse
the existing integral-number policy for `Int`. Typed array/object path helper
matrices, schema decoding, `std::config`, TOML, and JSONPath syntax remain
outside this boundary.

The typed JSON path collection projection follow-up adds only raw collection
and scalar-array path helpers: `at_array*`, `at_object*`,
`at_string_array*`, `at_int_array*`, and `at_bool_array*`. These helpers
preserve optional/default/required missing-path behavior, add path-aware
terminal collection shape errors such as `expected JSON Array at path
.metadata.tags`, and report scalar-array item shape errors with the item index,
such as `expected JSON String at path .metadata.tags[1]`. Generic `List[T]`
decoding, typed object value matrices, schema decoding, `std::config`, TOML,
and JSONPath syntax remain outside this boundary.

## Scalar And Collection Mapping

JSON values map to Muga values as follows:

| JSON | Muga |
|---|---|
| `null` | `json::Value::Null` |
| boolean | `json::Value::Bool(Bool)` |
| number | `json::Value::Number(json::Number)` |
| string | `json::Value::String(String)` |
| array | `json::Value::Array(List[json::Value])` |
| object | `json::Value::Object(Map[String, json::Value])` |

Number policy:

- integer literals that fit in `Int` may parse as `Number::Int`;
- other JSON numbers parse as `Number::Raw` only if they are valid JSON number
  text;
- malformed number text returns `InvalidNumber`;
- integer overflow returns `NumberOutOfRange`;
- `Float`, `Decimal`, NaN, Infinity, and ordering/equality policy are deferred.

Object policy:

- object keys are `String`;
- duplicate object keys are rejected with `DuplicateKey`;
- object encoding is deterministic and sorts keys lexicographically by Unicode
  scalar value before emission, independent of any internal `Map` order;
- `Map[String, Value]` keeps the first slice aligned with existing scalar-key
  map support and avoids arbitrary-key or structural equality decisions.

String policy:

- JSON escapes decode to Muga `String` values;
- invalid escape sequences and invalid Unicode scalar values return
  `InvalidEscape`;
- encoding emits valid JSON string escapes and must not produce control
  characters unescaped.

Nesting policy:

- parsing and encoding should enforce an implementation-defined nesting limit
  and report `NestingLimitExceeded` instead of overflowing the Rust stack or VM
  stack;
- the initial implementation limit is 128 nested arrays or objects, covered by
  parse and encode tests.

## Schema Evolution

The `Value` and `Number` enum shapes are public `.mgi` API. Adding, removing,
or renaming variants is a compatibility decision.

The first slice deliberately uses `Number::Raw(String)` so future `Float`,
`Decimal`, arbitrary-precision, or scientific-number helpers can be added as
functions without changing the `Value` enum. Future helpers can parse
`Number::Raw` into richer numeric types after those types have their own
specifications.

Future schema/client generation must not infer record or enum mappings from
this package by reflection. It should be designed separately from `.mgi` public
interfaces and explicit schema rules.

## Diagnostics

Compiler diagnostics for using `std::json` should follow existing package
patterns:

- missing imports should suggest `import std::json`;
- type mismatches should name the expected public types, such as
  `json::Value`, `json::Number`, or `json::Error`;
- artifact-backed execution and checks must work through emitted `.mgi` /
  `.mgb` artifacts without reading private source bodies.

Runtime parse/encode failures are data errors, not compiler diagnostics. They
return `json::Error` with:

- `kind`: stable machine-readable `ErrorKind`;
- `message`: concise human text;
- `offset`: zero-based byte offset into the input string for parse errors, or
  `-1` when the failure is not tied to input text, such as encoding an invalid
  `Number::Raw`.

The first implementation should add focused tests for:

- valid scalar, array, and object parsing;
- valid scalar, array, and object encoding;
- duplicate keys;
- invalid escapes;
- invalid numbers and integer overflow;
- trailing characters;
- nesting limit behavior;
- `number_as_int`;
- missing import and type mismatch diagnostics;
- emitted artifact execution through `muga emit-artifacts` and
  `muga run --artifact-root`.

## First Implementation Boundary

The implemented code slice may include only this package contract plus focused
README/spec/test updates. It must not expand into schema generation or add:

- generalized JSON command output beyond existing CLI contracts;
- schema generation, OpenAPI, HTTP, RPC, service runtime, or client stubs;
- source-level derive/annotation syntax;
- `Float`, `Decimal`, `Bytes`, streaming APIs, or resource handles.
