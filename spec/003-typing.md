# Typing Specification

Status: current language specification; see
[LANGUAGE.md](../LANGUAGE.md#specification-status).

This document defines the current typing policy, with emphasis on inference-first ergonomics and the limited cases where annotations are mandatory.

## 1. Typing Policy

The language prefers omission of type annotations.

- local bindings should infer their type from the right-hand side
- function parameter and return types should be inferred when the result is unique
- annotations are required only when inference cannot determine a unique type

## 2. Built-in Types and Source Type Expressions

The minimal built-in types are:

- `Int`
- `Bool`
- `String`
- `Unit`

In addition, Muga provides:

- user-defined nominal record types introduced by `record`
- source-level function types written with `->`
- generic type expressions written with `[]`

Therefore, source `type_expr` is:

```ebnf
type_expr          := function_type
function_type      := function_domain "->" type_expr
                    | non_function_type
function_domain    := non_function_type
                    | "(" type_expr_list? ")"
non_function_type  := type_primary type_args?
type_primary       := "Int"
                    | "Bool"
                    | "String"
                    | "Unit"
                    | IDENT
type_args          := "[" type_expr_list "]"
type_expr_list     := type_expr ("," type_expr)*
```

Examples:

- `Int -> Int`
- `(Int, String) -> Bool`
- `() -> Int`
- `List[Int]`
- `Map[String, Int]`
- `Option[User]`
- `Result[Unit, io::IOError]`

`Unit` has exactly one source value, written `()`. It is the preferred success value for effect-only fallible APIs such as future file writes, closes, and directory operations: `Result[Unit, E]`.

The language includes a restricted generics MVP.

Examples:

```txt
record Box[T] {
  value: T
}

fn id[T](value: T): T {
  value
}
```

The current Rust implementation supports generic type expressions for compiler-known `List[T]`, `Option[T]`, `Result[T, E]`, and `Map[K, V]`, plus explicit user-defined generic records and functions. The generics MVP is specified in [009-generics.md](./009-generics.md).

## 3. Prelude Built-ins

The prelude currently provides:

- `print`
- `println`
- `len`, `is_empty`, `push`, `get`, and `set` for `List[T]`
- `Map.empty`, `len`, `is_empty`, `contains`, `get`, `insert`, and `remove` for `Map[K, V]`
- `to_string` for `Int`, `Bool`, and `String`
- `is_empty`, `contains`, `trim`, `char_count`, `starts_with`, `ends_with`, `replace`, `split`, `concat`, `slice_chars`, `parse_int`, and `parse_bool` for `String`

`print` accepts exactly one argument of type `Int`, `Bool`, or `String`, writes its textual representation to standard output without a trailing newline, and returns that same value.

`println` accepts exactly one argument of type `Int`, `Bool`, or `String`, writes its textual representation to standard output as one line, and returns that same value.

When the `print` or `println` argument type is ambiguous, diagnostics should suggest annotating the argument as `Int`, `Bool`, or `String`.

When the direct `len` argument type is ambiguous, diagnostics should suggest annotating the argument as `List[T]` or `Map[K, V]`. When the direct `is_empty` argument type is ambiguous, diagnostics should suggest annotating the argument as `String`, `List[T]`, or `Map[K, V]`.

When the direct `get` receiver type is ambiguous, diagnostics should suggest annotating the receiver as `List[T]` or `Map[K, V]`. When the direct `contains` receiver type is ambiguous, diagnostics should suggest annotating the receiver as `String` or `Map[K, V]`. When the direct `insert` or `remove` receiver type is ambiguous, diagnostics should suggest annotating the receiver as `Map[K, V]`.

`Int.to_string()`, `Bool.to_string()`, and `String.to_string()` return `String`. `to_string` is explicit and intentionally does not introduce implicit string conversion. When the receiver type is ambiguous, diagnostics should suggest annotating the receiver as `Int`, `Bool`, or `String`.

`String.is_empty()` returns `Bool`, `String.contains(needle)` returns `Bool`, `String.trim()` returns `String`, `String.char_count()` returns `Int`, `String.starts_with(prefix)` / `String.ends_with(suffix)` return `Bool`, `String.replace(old, new)` returns `String`, `String.split(separator)` returns `List[String]`, `String.concat(other)` returns `String`, `String.slice_chars(start, count)` returns `Result[String, String]`, `String.parse_int()` returns `Result[Int, String]`, and `String.parse_bool()` returns `Result[Bool, String]`. `String.char_count()` and `String.slice_chars(start, count)` count and index Unicode scalar values, not UTF-8 bytes or user-perceived grapheme clusters. `slice_chars` accepts zero-based `start` plus `count`; negative values or ranges beyond the string return `Result::Err("invalid slice range")`. `replace("", new)` returns the original string unchanged, and `split("")` returns a one-item list containing the original string.

`String.len()` is intentionally not part of this string-helper slice. Future length/indexing APIs should stay explicit: add `String.byte_len()` when bytes or I/O APIs need byte size, keep any range syntax or substring aliases aligned with `slice_chars` before adding them, and reserve grapheme-cluster APIs until the standard library has a Unicode segmentation dependency/versioning policy. Fallible string helpers currently return `Result[_, String]`; richer string-specific error records or enums should be introduced only after several string APIs need a shared error shape.

## 4. Standard Library Package Slice

The first compiler-provided standard packages are:

```muga
import std::fs
import std::io
import std::json
import std::list
import std::map
import std::option
import std::path
import std::process
import std::result
import std::string
import std::task
```

`std::io` exports:

```muga
pub record IOError {
  operation: String
  path: String
  kind: String
  message: String
  raw_code: Option[Int]
}

pub record PathPairError {
  operation: String
  from_path: String
  to_path: String
  kind: String
  message: String
  raw_code: Option[Int]
}
```

Source files name these records through an import alias, for example `import std::io` followed by `io::IOError` or `io::PathPairError`. Full package paths such as `std::io::IOError` are not valid type names in source; diagnostics suggest importing `std::io` and using the local alias form.

`std::path` exports:

```muga
pub record Path {
  text: String
}

pub fn from_string(text: String): Path
pub fn as_string(path: Path): String
pub fn join(base: Path, child: String): Path
pub fn normalize(path: Path): Path
pub fn file_name(path: Path): Option[String]
pub fn with_file_name(path: Path, new_file_name: String): Path
pub fn parent(path: Path): Option[Path]
pub fn strip_prefix(path: Path, base: Path): Option[Path]
pub fn extension(path: Path): Option[String]
pub fn file_stem(path: Path): Option[String]
pub fn with_extension(path: Path, new_extension: String): Path
pub fn is_absolute(path: Path): Bool
```

`std::process` exports:

```muga
pub enum ErrorKind {
  Spawn
  Wait
  StdoutUtf8
  StderrUtf8
}

pub record Error {
  kind: ErrorKind
  command: String
  message: String
}

pub record EnvVar {
  name: String
  value: String
}

pub record Options {
  cwd: Option[path::Path]
  env: List[EnvVar]
}

pub record Output {
  status: Int
  success: Bool
  stdout: String
  stderr: String
}

pub fn default_options(): Options
pub fn run(command: String, args: List[String]): Result[Output, Error]
pub fn run_with(command: String, args: List[String], options: Options): Result[Output, Error]
```

`process::run` executes `command` directly with explicit argument values. It
does not add shell interpolation or a shell helper; users can run a shell as the
command when that is what they want. `process::run_with` accepts an optional
working directory as `path::Path` and explicit environment overrides through
`List[EnvVar]`; the host environment is otherwise inherited. A nonzero child
exit is captured as `Result::Ok(Output { success: false, ... })`, not as an
error. `Result::Err(process::Error)` is reserved for spawn, wait, stdout/stderr
UTF-8 conversion, and related recoverable host failures.

`std::task` exports:

```muga
pub fn join[T](task: Task[T]): T
```

`Task[T]` is the internal task-handle type produced by `spawn` inside a
`group` expression. Only the compiler-provided `std::task` package spells
`Task[T]` in a signature; user source cannot write it in annotations, record
fields, or function signatures (`T013` unknown generic type). `task::join`
returns the completed child value, and the qualified chained form
`handle.task::join()` is the same call. See
[spec/007-concurrency-draft.md](./007-concurrency-draft.md) section 5 for the
full structured task group rules.

`std::fs` exports:

```muga
pub opaque type File

pub record FileMetadata { ... }
pub record PathStatus { ... }
pub enum PathKind { Missing, File, Directory, Other }
pub record PathInfo { ... }
pub record PathMetadata { ... }
pub record PathSizeMetadata { ... }
pub record DirectorySizeMetadata { ... }

pub fn read_text(path: String): Result[String, io::IOError]
pub fn read_bytes(path: String): Result[bytes::Bytes, io::IOError]
pub fn read_resource_text(package_path: String, resource_path: String): Result[String, io::IOError]
pub fn read_resource_bytes(package_path: String, resource_path: String): Result[bytes::Bytes, io::IOError]
pub fn write_text(path: String, text: String): Result[Unit, io::IOError]
pub fn write_bytes(path: String, data: bytes::Bytes): Result[Unit, io::IOError]
pub fn read_text_path(path: path::Path): Result[String, io::IOError]
pub fn read_bytes_path(path: path::Path): Result[bytes::Bytes, io::IOError]
pub fn write_text_path(path: path::Path, text: String): Result[Unit, io::IOError]
pub fn write_bytes_path(path: path::Path, data: bytes::Bytes): Result[Unit, io::IOError]
pub fn open_text(path: path::Path): Result[File, io::IOError]
pub fn create_text(path: path::Path): Result[File, io::IOError]
pub fn append_text(path: path::Path): Result[File, io::IOError]
pub fn read_text_from(file: File): Result[String, io::IOError]
pub fn write_text_to(file: File, text: String): Result[Unit, io::IOError]
pub fn flush(file: File): Result[Unit, io::IOError]
pub fn close(file: File): Result[Unit, io::IOError]
pub fn read_dir_path(path: path::Path): Result[List[path::Path], io::IOError]
pub fn read_dir_recursive_path(path: path::Path): Result[List[path::Path], io::IOError]
pub fn directory_size_metadata_path(path: path::Path): Result[DirectorySizeMetadata, io::IOError]
pub fn canonicalize_path(path: path::Path): Result[path::Path, io::IOError]
pub fn create_dir_path(path: path::Path): Result[Unit, io::IOError]
pub fn create_dir_all_path(path: path::Path): Result[Unit, io::IOError]
pub fn remove_file_path(path: path::Path): Result[Unit, io::IOError]
pub fn remove_dir_path(path: path::Path): Result[Unit, io::IOError]
pub fn remove_dir_all_path(path: path::Path): Result[Unit, io::IOError]
pub fn copy_file_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]
pub fn copy_dir_all_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]
pub fn move_dir_all_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]
pub fn rename_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]
pub fn file_size_path(path: path::Path): Result[Int, io::IOError]
pub fn modified_unix_millis_path(path: path::Path): Result[time::UnixMillis, io::IOError]
pub fn file_metadata_path(path: path::Path): Result[FileMetadata, io::IOError]
pub fn path_status(path: path::Path): PathStatus
pub fn path_kind(path: path::Path): PathKind
pub fn path_info(path: path::Path): PathInfo
pub fn path_metadata_path(path: path::Path): Result[PathMetadata, io::IOError]
pub fn path_size_metadata_path(path: path::Path): Result[PathSizeMetadata, io::IOError]
pub fn exists_path(path: path::Path): Bool
pub fn is_file_path(path: path::Path): Bool
pub fn is_dir_path(path: path::Path): Bool
```

`std::option` exports ordinary value helpers:

```muga
pub fn is_some[T](value: Option[T]): Bool
pub fn is_none[T](value: Option[T]): Bool
pub fn map[T, U](value: Option[T], f: T -> U): Option[U]
pub fn and_then[T, U](value: Option[T], f: T -> Option[U]): Option[U]
pub fn value_or[T](value: Option[T], fallback: T): T
```

`std::result` exports ordinary value helpers:

```muga
pub fn is_ok[T, E](value: Result[T, E]): Bool
pub fn is_err[T, E](value: Result[T, E]): Bool
pub fn map[T, E, U](value: Result[T, E], f: T -> U): Result[U, E]
pub fn map_err[T, E, F](value: Result[T, E], f: E -> F): Result[T, F]
pub fn and_then[T, E, U](value: Result[T, E], f: T -> Result[U, E]): Result[U, E]
pub fn value_or[T, E](value: Result[T, E], fallback: T): T
```

`std::string` exports pure text assembly helpers over explicit string lists:

```muga
pub fn concat_all(parts: List[String]): String
pub fn join(parts: List[String], separator: String): String
```

`string::concat_all([])` returns `""`. `string::join(parts, separator)` inserts the
separator only between parts and also returns `""` for an empty list. These
helpers do not implicitly convert non-string values; callers use the existing
`to_string()` helpers before adding values to the `List[String]`.

`std::list` exports narrow value helpers:

```muga
pub fn map[T, U](items: List[T], f: T -> U): List[U]
pub fn filter[T](items: List[T], predicate: T -> Bool): List[T]
pub fn fold[T, U](items: List[T], initial: U, f: (U, T) -> U): U
pub fn any[T](items: List[T], predicate: T -> Bool): Bool
pub fn all[T](items: List[T], predicate: T -> Bool): Bool
```

`std::map` exports narrow key/value extraction helpers:

```muga
pub fn keys[K, V](items: Map[K, V]): List[K]
pub fn values[K, V](items: Map[K, V]): List[V]
```

`std::json` exports explicit JSON data shapes, parse/encode helpers, integer
number conversion, pure scalar/composite value/object-field
accessor/default/required helpers, scalar array projection helpers, and typed
JSON path traversal helpers:

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
  kind: ErrorKind
  message: String
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
pub fn decode_or[T](value: Value, fallback: T): Result[T, Error]
pub fn decode[T](value: Value): Result[T, Error]
```

Source files name these values through an import alias, for example
`import std::json` followed by `json::Value` or `json::Error`. Value and
object-field accessor helpers return `json::Error` for wrong JSON shapes and
missing required object fields. Scalar array projection helpers return
`json::Error` with index-specific messages for wrong array item shapes. Direct
scalar-array object-field helpers mirror the optional/default/required object
field family and include both the object field key and item index in wrong-item
shape diagnostics. JSON path helpers traverse objects and arrays by typed
`Field` / `Index` segments, return `Option::None` for missing fields or
out-of-range indexes, and return `json::Error` for wrong shapes or required
missing paths with deterministic rendered paths such as `.metadata.owner` and
`.items[0]`. String path parsing and JSONPath queries remain outside this
explicit helper family. The compiler-owned `json::decode_or[T]` and
`json::decode[T]` schema decoding helpers support `String`, `Int`, `Bool`,
`Option[T]`, recursive `List[T]`, typed `Map[String, T]`,
`Map[String, json::Value]`, concrete non-generic records over supported fields,
and concrete non-generic enums over supported payloads. `json::decode_or[T]`
preserves fallback fields for missing default-overlay record fields, while
explicit JSON `null` decodes optional fields to `Option::None`.
`json::decode[T]` requires an expected `Result[T, json::Error]` target and
reports missing non-optional record fields as path-aware `json::Error` values.
Concrete enum decoding uses zero-payload string tags and one-payload single-key
objects. Record fields and enum variants can use `@json(rename: "...")` and
input-only `@json(alias: "...")` metadata for external wire names, and records
can use `@json(deny_unknown_fields)` for strict object decoding. Record fields
can use narrow `@validate(...)` metadata: `non_empty`, `min_len`, and `max_len`
for `String` / `Option[String]`, plus `min` and `max` for `Int` /
`Option[Int]`. Validation failures are reported as path-aware
`json::ErrorKind::Validation` values through `std::json`; through
`std::config`, the same decode failure is wrapped as `config::ErrorKind::Decode`
with the validation message and offset. Nested `Option[Option[T]]`, generic
record schemas, generic enum schemas, non-string map keys, record-level or
cross-field validation, user-defined validators, TOML, and config-file
discovery remain deferred. The `std::json`
helpers for typed JSON path scalar projection (`at_string*`, `at_int*`, and
`at_bool*`) preserve the same
optional/default/required missing-path behavior and report terminal scalar
shape errors with the rendered path. Additional typed object value helper
matrices, generated schema metadata, TOML, and config-file discovery remain
deferred.
The `std::json` helpers for typed JSON path collection projection (`at_array*`,
`at_object*`, `at_string_array*`, `at_int_array*`, and `at_bool_array*`)
preserve the same optional/default/required missing-path behavior, report
terminal collection shape errors with the rendered path, and report scalar-array
item shape errors with an appended index such as `.metadata.tags[1]`. Generic
TOML and config-file discovery remain deferred.

`std::config` exports JSON file loading over the same compiler-owned schema
target set:

```muga
pub enum ErrorKind {
  Read
  Parse
  Decode
}

pub record Error {
  kind: ErrorKind
  path: path::Path
  message: String
  offset: Int
  raw_code: Option[Int]
}

pub fn load_json_or[T](file_path: path::Path, fallback: T): Result[T, Error]
```

`config::load_json_or[T]` reads a UTF-8 JSON file, parses it as `json::Value`,
and decodes it with the same default-overlay semantics and structural target
set as `json::decode_or[T]`. Decode failures map to `config::ErrorKind::Decode`.

`std::cli` exports pure lookup helpers plus compiler-owned typed parser,
request, usage, and help helpers over explicit argument lists:

```muga
pub enum ErrorKind {
  UnknownArgument
  MissingArgument
  MissingValue
  InvalidValue
  Validation
  UnsupportedTarget
}

pub record Error {
  kind: ErrorKind
  argument: String
  message: String
}

pub enum Request[T] {
  Help(String)
  Parsed(T)
}

pub fn parse_or[T](args: List[String], defaults: T): Result[T, Error]
pub fn parse[T](args: List[String]): Result[T, Error]
pub fn parse_request[T](args: List[String], program: String): Result[Request[T], Error]
pub fn parse_request_or[T](args: List[String], program: String, defaults: T): Result[Request[T], Error]
pub fn usage_for[T](program: String, defaults: T): String
pub fn usage_for_required[T](program: String): String
pub fn help_for[T](program: String, defaults: T): String
pub fn help_for_required[T](program: String): String
pub fn positional(args: List[String], index: Int): Option[String]
pub fn positional_or(args: List[String], index: Int, default_value: String): String
pub fn has_flag(args: List[String], name: String): Bool
pub fn has_short_flag(args: List[String], name: String): Bool
pub fn help_requested(args: List[String]): Bool
pub fn option(args: List[String], name: String): Option[String]
pub fn option_or(args: List[String], name: String, default_value: String): String
pub fn option_values(args: List[String], name: String): List[String]
pub fn option_values_or(args: List[String], name: String, default_value: List[String]): List[String]
pub fn positional_int(args: List[String], index: Int): Result[Option[Int], String]
pub fn positional_int_or(args: List[String], index: Int, default_value: Int): Result[Int, String]
pub fn option_int(args: List[String], name: String): Result[Option[Int], String]
pub fn option_int_or(args: List[String], name: String, default_value: Int): Result[Int, String]
pub fn positional_bool(args: List[String], index: Int): Result[Option[Bool], String]
pub fn positional_bool_or(args: List[String], index: Int, default_value: Bool): Result[Bool, String]
pub fn option_bool(args: List[String], name: String): Result[Option[Bool], String]
pub fn option_bool_or(args: List[String], name: String, default_value: Bool): Result[Bool, String]
```

`list::map`, `list::filter`, and `map::keys` / `map::values` allocate new lists at the source level. `list::map`, `list::filter`, and `list::fold` process items in list order. `list::any` and `list::all` return `Bool` and may stop once the result is known. `map::keys` and `map::values` return lists in the map's deterministic entry order; replacing an existing key does not move that key. `List.contains` remains deferred while equality is scalar-only. `map::entries` is active work once the public `map::Entry[K, V]` record shape in [008-collections.md](./008-collections.md) is validated; it does not require structural equality.

`cli::has_flag(args, "verbose")` matches `--verbose`; `cli::has_short_flag(args, "v")` matches `-v`; `cli::help_requested(args)` matches `--help` and `-h`; `cli::option(args, "output")` matches `--output value` and `--output=value`; `cli::option_values(args, "tag")` collects repeated `--tag value` and `--tag=value` occurrences in encounter order. `--` stops flag and option parsing and makes later values positional. Before `--`, `cli::positional` counts values that do not start with `--`. A separate `--name` option with no non-option following value is skipped by the lookup helpers. The schema helpers are lowered by the compiler for supported concrete record and enum targets, including command enums, wrapper records, value sources, validation metadata, generated usage/help text, and recoverable `cli::Error` values. Global parser state and shell integration remain outside the source language.

`path::join(base, child)` combines a `Path` with a child path string using host path semantics and returns a new `Path`. `path::parent(path)` returns the parent path as `Option[Path]`; paths without a meaningful parent return `Option::None`. `path::file_name(path)` returns the final path component as `Option[String]`; paths without a file name or with a non-Unicode file name return `Option::None`. `path::extension(path)` returns the final extension as `Option[String]`; paths without an extension or with a non-Unicode extension return `Option::None`. `path::file_stem(path)` returns the final file stem as `Option[String]`; paths without a file stem or with a non-Unicode file stem return `Option::None`. `path::is_absolute(path)` classifies the path using host path semantics. `read_text` and `read_text_path` read a UTF-8 text file into a `String`. `read_bytes` and `read_bytes_path` read a binary file into opaque `std::bytes::Bytes`. `write_bytes` and `write_bytes_path` write opaque `Bytes` as full-file binary output. `read_resource_text(package_path, resource_path)` reads a UTF-8 text resource from a manifest-declared package resource root without returning a host path. `read_resource_bytes(package_path, resource_path)` reads bytes from that same resource-root map and returns opaque `std::bytes::Bytes`; the first `std::bytes` helpers are `bytes::size`, `bytes::empty`, and zero-based `bytes::at(bytes, index): Option[Int]`. `hash::sha256_hex(bytes)` returns a 64-character lowercase SHA-256 hex digest for `Bytes`. `read_dir_path` lists direct directory entries as `path::Path` values in deterministic sorted order, and `read_dir_recursive_path` returns deterministic read-only descendants without recursing into symlink directories. `directory_size_metadata_path`, `file_metadata_path`, `path_metadata_path`, and `path_size_metadata_path` expose the current metadata slice. `write_text`, `write_text_path`, `write_bytes`, `write_bytes_path`, `create_dir_path`, `create_dir_all_path`, `remove_file_path`, `remove_dir_path`, `remove_dir_all_path`, `copy_file_path`, `copy_dir_all_path`, `move_dir_all_path`, and `rename_path` use `Unit` as the success payload. Single-path recoverable filesystem failures return `Result::Err(io::IOError)`. Two-path recoverable filesystem failures return `Result::Err(io::PathPairError)` with `from_path` and `to_path` populated. Recursive directory copy and move are no-overwrite operations in the current slice. `exists_path`, `is_file_path`, and `is_dir_path` are non-throwing metadata predicates and return `false` for missing or inaccessible paths. `std::fs::File` is the current runtime-backed opaque handle. `open_text`, `create_text`, and `append_text` acquire read, write, and append handles; `read_text_from`, `write_text_to`, and `flush` borrow a handle and return recoverable `io::IOError` values for host IO or wrong-mode failures; `close` consumes the handle and returns `Result[Unit, io::IOError]`. Statement-form `using` may manage such handles when the enclosing function returns a compatible `Result[T, io::IOError]`. The current slice intentionally does not add binary file handles, byte mutation, codecs, streaming hash handles, broader cryptographic APIs, stdout/stderr handles, permissions APIs, network APIs, streaming APIs, or asynchronous IO.

### 4.1 Standard-Library Surface Consolidation Target

The implemented package slices contain transitional overlap that should not be
frozen accidentally. Before these APIs become stable:

- filesystem operations should use `path::Path` as their one canonical path
  input; String-path twins and `_path` suffixes should be removed unless real
  programs demonstrate distinct semantics
- schema-driven `cli::parse[T]` should be the primary CLI API; manual
  positional/option/flag scanning should move to a clearly low-level namespace
  or be removed when redundant
- the combinatorial JSON accessor/default/required matrix should contract around
  parse/encode, typed decode/conversion, and a small composable dynamic `Value`
  traversal core
- `Bytes` should gain UTF-8/list conversion, slicing, concatenation, hex and
  Base64 codecs, an efficient builder, and binary file handles without implying
  a broad cryptography framework
- time should gain `Duration`, a monotonic clock, sleep, and checked arithmetic;
  OS-backed secure random bytes should be a narrow recoverable API separate
  from any future seeded PRNG

Removal or renaming requires sample, template, completion, generated-schema,
artifact, and migration coverage. Keeping compatibility aliases throughout the
entire `0.x` series is not required, but the stable language must not expose two preferred
spellings for the same operation.

Because `print`, `println`, `len`, and `is_empty` accept several concrete types, none of them by itself makes an unconstrained parameter uniquely inferable.

Example:

```txt
fn show_int(x) {
  print(x + 1)
}
```

This is valid because `x + 1` constrains the argument to `Int`.

By contrast:

```txt
fn show(x) {
  print(x)
}
```

still requires an annotation.

## 5. Higher-Order Functions

Muga supports higher-order functions.

Allowed in principle:

- passing a named function as an argument
- passing an anonymous function as an argument
- storing a function in a local binding

Example:

```txt
fn inc(x) {
  x + 1
}

fn apply(x: Int, f): Int {
  f(x)
}

apply(10, inc)
apply(10, fn(n) {
  n + 1
})
```

If a higher-order parameter is used in a way that determines a unique function type inside the same function body, its function-type annotation may be omitted.

Examples:

```txt
fn apply(x: Int, f): Int {
  f(x)
}

fn offset(x: Int, f) {
  f(x) + 1
}
```

By contrast, this remains ambiguous:

```txt
fn apply(x, f) {
  f(x)
}
```

This also remains ambiguous:

```txt
fn show(x: Int, f) {
  println(f(x))
}
```

because `println` accepts `Int`, `Bool`, or `String`, so the callback result type is not uniquely determined.

When a function parameter or return type remains ambiguous after body checking, diagnostics should suggest adding parameter or return type annotations until the function signature is unique.

An explicit arrow annotation remains valid and useful:

```txt
fn show(x: Int, f: Int -> String): String {
  println(f(x))
}
```

## 6. Record Typing

For:

```txt
record User {
  name: String
}
```

`User` is a nominal type.

A record literal:

```txt
User {
  name: "Ada"
}
```

has type `User` if and only if:

- every declared field is provided exactly once
- no extra fields are present
- each field initializer has the declared field type
- every record field type must be a non-function type

## 7. Field Access and Chained Call Typing

For field access:

```txt
expr.name
```

`expr` must have a record type that declares a field `name`. The expression type is the declared type of that field.

For chained call:

```txt
expr.name(arg1, arg2)
expr.alias::name(arg1, arg2)
```

the receiver expression `expr` is typed first.

Then:

1. if the callee is plain `name` and `name` resolves to a receiver-style function, the call is typed as a call of that function with `expr` as the first argument
2. otherwise, if the corresponding ordinary call is valid, the chained call is typed as that UFCS-style desugaring
3. otherwise, the expression is a type error

Because record fields may not have function type, `expr.name(...)` and `expr.alias::name(...)` never mean a call through a function-valued field.

## 8. Record Update Typing

For:

```txt
expr.with(field1: value1, field2: value2)
```

the base expression `expr` must have a record type.

The expression type is the same record type as the base expression if and only if:

- every updated field name exists on that record type
- each replacement expression has exactly the declared type of that field
- no field name appears more than once in the same update

Unspecified fields are preserved from the base value.

The update is non-destructive. The result is a new record value rather than a mutation of the original record.

`expr.with(...)` is not typed as an ordinary chained call.

## 9. Operator Typing Rules

The built-in operator typing rules are:

- unary `-` : `Int -> Int`
- unary `!` : `Bool -> Bool`
- `+`, `-`, `*`, `/` : `Int -> Int -> Int`
- `<`, `<=`, `>`, `>=` : `Int -> Int -> Bool`
- `==`, `!=` : allowed only for identical primitive types among `Int`, `Bool`, and `String`
- `and`, `or` : `Bool -> Bool -> Bool`, evaluated left-to-right with short-circuiting

String concatenation uses explicit `String.concat(other)`. The `+` operator remains `Int`-only.

### 9.1 Equality Policy

The current equality policy is intentionally scalar-only:

- `Int == Int` / `Int != Int`
- `Bool == Bool` / `Bool != Bool`
- `String == String` / `String != String`

Both operands must have the same supported scalar type. Muga does not define implicit conversions, cross-type equality, pointer/reference identity, or dynamic equality.

Structural equality is not currently supported. `==` and `!=` are rejected for records, user-defined enums, `Option[T]`, `Result[T, E]`, `List[T]`, `Map[K, V]`, `Unit`, functions, and builtins. Compare scalar fields or payloads explicitly with `match`, field access, and scalar helpers. The `std::test` package follows the same policy by exposing only scalar equality assertions.

`List.contains`, structural `assert_eq`, `Set[T]`, arbitrary `Map` key types,
and any future derived equality/hash support must not be added merely by relying
on runtime value shape. They require an explicit spec update for structural
equality, hashing, package-interface persistence, diagnostics, and focused
tests. `map::entries` is independent: it exposes existing key/value pairs
through a deliberate public record shape and does not grant equality or hash
capabilities.

### 9.2 Derived Equality And Hashing Decision

Muga should evaluate opt-in compiler-derived equality and hashing as
a closed capability, not as a trait, protocol, overloaded operator, or dynamic
dispatch system. If accepted, derivation must be recorded in package
interfaces, recurse only through supported payloads, reject functions,
builtins, tasks, and runtime handles, and define any future `Float64` `NaN` and
signed-zero behavior. Only types with the recorded capability may unlock
structural assertions, `List.contains`, `Set[T]`, or non-scalar map keys.

Until that design is accepted, the scalar-only rules above remain normative.

## 10. Inference Sources

Current inference may use:

- literal types
- operator constraints
- branch result agreement
- expected types from the surrounding expression inside the same function body
- explicit annotations already present in the same declaration
- explicit function-type annotations on parameters

Muga does not use call sites in other functions or modules as an inference source.

In future module or package boundaries, explicit function-type annotations are expected to remain the preferred interface style even when a local implementation might be inferable.

Examples:

```txt
x = 1          // Int
name = "m"     // String
```

```txt
fn inc(x) {
  x + 1
}
```

Because `+` is defined here only as integer addition, `x` is inferred as `Int`.

## 11. Local Bindings

For a binding:

```txt
x = e
mut y = e
```

the binding type is inferred from the type of `e`.

For mutable bindings, every later update in the same scope must be type-compatible with the original inferred type.

Example:

```txt
mut total = 0
total = total + 1
```

`total` has type `Int`.

Mutable updates must preserve the original type exactly. Muga does not define implicit conversions or subtyping.

Local bindings may also hold function values.

Example:

```txt
inc = fn(x: Int): Int {
  x + 1
}
```

Collection literals and enum-like constructors sometimes need an expected type. Local binding annotations provide that type without introducing `let`.

Syntax:

```txt
items: List[Int] = []
mut names: List[String] = []
```

This is needed because an empty collection literal does not determine its element type by itself.

This syntax is implemented for local bindings and is used to give empty collection literals and `Option::None` an expected type.

## 12. Conditions and Branches

The condition expression of:

- `if`
- `while`

must have type `Bool`.

For an `if` expression, both branches must produce the same result type. `else if` chains are typed as nested `if` expressions, so each nested branch follows the same exact-match rule.

Example:

```txt
fn abs(n: Int) {
  if n < 0 {
    -n
  } else {
    n
  }
}
```

Both branches produce `Int`, so the `if` expression has type `Int`.

For an `if` expression, the branch result types must match exactly, and every value-producing `else if` chain must end in a final `else`.

`for item in list` requires the iterable expression after `in` to have type `List[T]`. The loop item is a fresh immutable binding of type `T` scoped to the loop body. It follows the normal no-shadowing rule for bindings. When the iterable type is ambiguous, diagnostics should suggest annotating it as `List[T]`.

`break` and `continue` are valid only inside a `while` or `for` loop. They target the nearest enclosing loop in the same function body. A nested named or anonymous function starts a new loop-control boundary, so loop-control statements inside that function do not target a loop in the caller.

## 13. Function Parameter Inference

A parameter annotation may be omitted when the parameter type is uniquely determined from the function body and surrounding constraints.

Example:

```txt
fn double(x) {
  x * 2
}
```

Because `*` is defined only for `Int`, `x` is inferred as `Int`.

Inference fails when a parameter remains unconstrained.

Example:

```txt
fn id(x) {
  x
}
```

This requires annotation because the type of `x` is not uniquely determined.

For higher-order functions, parameter annotation is often the intended source of the function shape.

Example:

```txt
fn apply(x: Int, f: Int -> Int): Int {
  f(x)
}
```

## 14. Function Return Inference

The return type of a function is inferred from the final expression in the body and from any explicit `return expr` statements.

When control flow branches, the return type is inferred from the unified branch result type.

If the body does not provide enough information to infer a unique return type, a return annotation is required.

`return expr` is allowed only inside a named or anonymous function. The expression must match the enclosing function return type. A `return expr` inside a nested anonymous function returns from that anonymous function, not from the caller.

## 15. Inference Boundary

Muga intentionally uses local-only inference.

Allowed:

- infer local binding types from the right-hand side
- infer function parameter types from operators and other constraints inside the same function body
- infer function return types from the function body
- infer `if` expression result types from branch agreement
- typecheck higher-order calls once explicit function-type annotations are present

Disallowed:

- inferring a callee parameter type from call sites alone
- propagating constraints across unrelated top-level declarations
- implicit polymorphic generalization of non-generic declarations
- inferring a complete higher-order parameter shape from distant call sites alone

This means:

```txt
fn inc(x) {
  x + 1
}
```

is valid, but:

```txt
fn id(x) {
  x
}
```

is not.

## 16. Mandatory Annotations

Annotations are required in the following cases:

1. a function parameter type is not uniquely inferable
2. a function return type is not uniquely inferable
3. a recursive function has neither an annotated parameter nor an annotated return type
4. a mutually recursive function participates in a recursive group without an explicit signature
5. a receiver parameter must have an explicit type annotation
6. a higher-order parameter shape is not uniquely inferable

An explicit function signature means:

- at least one parameter or the return type is annotated for direct recursion
- every function in a mutually recursive group has enough annotations to determine its full callable type before body checking

## 17. Direct Recursion Rule

For a directly recursive function, at least one of the following must be present:

- an annotation on one or more parameters
- an explicit return type annotation

Valid:

```txt
fn fact(n: Int) {
  if n == 0 {
    1
  } else {
    n * fact(n - 1)
  }
}
```

Also valid:

```txt
fn fact(n): Int {
  if n == 0 {
    1
  } else {
    n * fact(n - 1)
  }
}
```

Invalid:

```txt
fn fact(n) {
  if n == 0 {
    1
  } else {
    n * fact(n - 1)
  }
}
```

## 18. Mutual Recursion Rule

Mutually recursive functions require explicit signatures.

Currently, this means each function in the recursive group must carry enough annotations for its callable type to be known before any body in the group is checked.

Valid:

```txt
fn is_even(n: Int): Bool {
  if n == 0 {
    true
  } else {
    is_odd(n - 1)
  }
}

fn is_odd(n: Int): Bool {
  if n == 0 {
    false
  } else {
    is_even(n - 1)
  }
}
```

For implementation purposes, "explicit signature" for a mutually recursive group means that each function's full callable type is known before any body in the group is checked.
