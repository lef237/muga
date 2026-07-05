pub const IO_PACKAGE: &str = "std::io";
pub const FS_PACKAGE: &str = "std::fs";
pub const PATH_PACKAGE: &str = "std::path";
pub const ENV_PACKAGE: &str = "std::env";
pub const PROCESS_PACKAGE: &str = "std::process";
pub const CLI_PACKAGE: &str = "std::cli";
pub const TIME_PACKAGE: &str = "std::time";
pub const BYTES_PACKAGE: &str = "std::bytes";
pub const HASH_PACKAGE: &str = "std::hash";
pub const TEST_PACKAGE: &str = "std::test";
pub const OPTION_PACKAGE: &str = "std::option";
pub const RESULT_PACKAGE: &str = "std::result";
pub const STRING_PACKAGE: &str = "std::string";
pub const FMT_PACKAGE: &str = "std::fmt";
pub const LIST_PACKAGE: &str = "std::list";
pub const MAP_PACKAGE: &str = "std::map";
pub const JSON_PACKAGE: &str = "std::json";
pub const CONFIG_PACKAGE: &str = "std::config";
pub const TASK_PACKAGE: &str = "std::task";
pub const IO_ERROR_MANGLED_NAME: &str = "__muga_pkg__std__io__IOError";
pub const IO_ERROR_VISIBLE_NAME_IN_FS: &str = "io::IOError";
pub const PATH_PAIR_ERROR_MANGLED_NAME: &str = "__muga_pkg__std__io__PathPairError";
pub const PATH_PAIR_ERROR_VISIBLE_NAME_IN_FS: &str = "io::PathPairError";
pub const PATH_MANGLED_NAME: &str = "__muga_pkg__std__path__Path";
pub const PATH_VISIBLE_NAME_IN_FS: &str = "path::Path";
pub const BYTES_VISIBLE_NAME_IN_FS: &str = "bytes::Bytes";
pub const FS_DIRECTORY_SIZE_METADATA_MANGLED_NAME: &str =
    "__muga_pkg__std__fs__DirectorySizeMetadata";
pub const FS_DIRECTORY_SIZE_METADATA_VISIBLE_NAME_IN_FS: &str = "DirectorySizeMetadata";
pub const JSON_VALUE_MANGLED_NAME: &str = "__muga_pkg__std__json__Value";
pub const JSON_NUMBER_MANGLED_NAME: &str = "__muga_pkg__std__json__Number";
pub const JSON_ERROR_KIND_MANGLED_NAME: &str = "__muga_pkg__std__json__ErrorKind";
pub const JSON_ERROR_MANGLED_NAME: &str = "__muga_pkg__std__json__Error";
pub const CONFIG_ERROR_KIND_MANGLED_NAME: &str = "__muga_pkg__std__config__ErrorKind";
pub const CONFIG_ERROR_MANGLED_NAME: &str = "__muga_pkg__std__config__Error";
pub const CLI_ERROR_KIND_MANGLED_NAME: &str = "__muga_pkg__std__cli__ErrorKind";
pub const CLI_ERROR_MANGLED_NAME: &str = "__muga_pkg__std__cli__Error";
pub const CLI_REQUEST_MANGLED_NAME: &str = "__muga_pkg__std__cli__Request";
pub const PROCESS_ERROR_KIND_MANGLED_NAME: &str = "__muga_pkg__std__process__ErrorKind";
pub const PROCESS_ERROR_MANGLED_NAME: &str = "__muga_pkg__std__process__Error";
pub const PROCESS_ENV_VAR_MANGLED_NAME: &str = "__muga_pkg__std__process__EnvVar";
pub const PROCESS_OPTIONS_MANGLED_NAME: &str = "__muga_pkg__std__process__Options";
pub const PROCESS_OUTPUT_MANGLED_NAME: &str = "__muga_pkg__std__process__Output";
pub const PROCESS_ERROR_VISIBLE_NAME: &str = "Error";
pub const PROCESS_OPTIONS_VISIBLE_NAME: &str = "Options";
pub const PROCESS_OUTPUT_VISIBLE_NAME: &str = "Output";
pub const TASK_JOIN_BUILTIN: &str = "__muga_std_task_join";
pub const PATH_JOIN_BUILTIN: &str = "__muga_std_path_join";
pub const PATH_NORMALIZE_BUILTIN: &str = "__muga_std_path_normalize";
pub const PATH_FILE_NAME_BUILTIN: &str = "__muga_std_path_file_name";
pub const PATH_WITH_FILE_NAME_BUILTIN: &str = "__muga_std_path_with_file_name";
pub const PATH_PARENT_BUILTIN: &str = "__muga_std_path_parent";
pub const PATH_STRIP_PREFIX_BUILTIN: &str = "__muga_std_path_strip_prefix";
pub const PATH_EXTENSION_BUILTIN: &str = "__muga_std_path_extension";
pub const PATH_FILE_STEM_BUILTIN: &str = "__muga_std_path_file_stem";
pub const PATH_WITH_EXTENSION_BUILTIN: &str = "__muga_std_path_with_extension";
pub const PATH_IS_ABSOLUTE_BUILTIN: &str = "__muga_std_path_is_absolute";
pub const BYTES_SIZE_BUILTIN: &str = "__muga_std_bytes_size";
pub const BYTES_IS_EMPTY_BUILTIN: &str = "__muga_std_bytes_is_empty";
pub const BYTES_AT_BUILTIN: &str = "__muga_std_bytes_at";
pub const FS_READ_TEXT_BUILTIN: &str = "__muga_std_fs_read_text";
pub const FS_READ_BYTES_BUILTIN: &str = "__muga_std_fs_read_bytes";
pub const FS_READ_RESOURCE_TEXT_BUILTIN: &str = "__muga_std_fs_read_resource_text";
pub const FS_READ_RESOURCE_BYTES_BUILTIN: &str = "__muga_std_fs_read_resource_bytes";
pub const FS_WRITE_TEXT_BUILTIN: &str = "__muga_std_fs_write_text";
pub const FS_WRITE_BYTES_BUILTIN: &str = "__muga_std_fs_write_bytes";
pub const FS_OPEN_TEXT_BUILTIN: &str = "__muga_std_fs_open_text";
pub const FS_CREATE_TEXT_BUILTIN: &str = "__muga_std_fs_create_text";
pub const FS_APPEND_TEXT_BUILTIN: &str = "__muga_std_fs_append_text";
pub const FS_READ_TEXT_FROM_BUILTIN: &str = "__muga_std_fs_read_text_from";
pub const FS_WRITE_TEXT_TO_BUILTIN: &str = "__muga_std_fs_write_text_to";
pub const FS_FLUSH_BUILTIN: &str = "__muga_std_fs_flush";
pub const FS_CLOSE_BUILTIN: &str = "__muga_std_fs_close";
pub const FS_READ_DIR_BUILTIN: &str = "__muga_std_fs_read_dir";
pub const FS_READ_DIR_RECURSIVE_BUILTIN: &str = "__muga_std_fs_read_dir_recursive";
pub const FS_DIRECTORY_SIZE_METADATA_BUILTIN: &str = "__muga_std_fs_directory_size_metadata";
pub const FS_CANONICALIZE_BUILTIN: &str = "__muga_std_fs_canonicalize";
pub const FS_CREATE_DIR_BUILTIN: &str = "__muga_std_fs_create_dir";
pub const FS_CREATE_DIR_ALL_BUILTIN: &str = "__muga_std_fs_create_dir_all";
pub const FS_REMOVE_FILE_BUILTIN: &str = "__muga_std_fs_remove_file";
pub const FS_REMOVE_DIR_BUILTIN: &str = "__muga_std_fs_remove_dir";
pub const FS_REMOVE_DIR_ALL_BUILTIN: &str = "__muga_std_fs_remove_dir_all";
pub const FS_COPY_FILE_BUILTIN: &str = "__muga_std_fs_copy_file";
pub const FS_COPY_DIR_ALL_BUILTIN: &str = "__muga_std_fs_copy_dir_all";
pub const FS_MOVE_DIR_ALL_BUILTIN: &str = "__muga_std_fs_move_dir_all";
pub const FS_RENAME_BUILTIN: &str = "__muga_std_fs_rename";
pub const FS_FILE_SIZE_BUILTIN: &str = "__muga_std_fs_file_size";
pub const FS_MODIFIED_UNIX_MILLIS_BUILTIN: &str = "__muga_std_fs_modified_unix_millis";
pub const FS_EXISTS_BUILTIN: &str = "__muga_std_fs_exists";
pub const FS_IS_FILE_BUILTIN: &str = "__muga_std_fs_is_file";
pub const FS_IS_DIR_BUILTIN: &str = "__muga_std_fs_is_dir";
pub const ENV_GET_VAR_BUILTIN: &str = "__muga_std_env_get_var";
pub const ENV_ARGS_BUILTIN: &str = "__muga_std_env_args";
pub const ENV_CURRENT_DIR_BUILTIN: &str = "__muga_std_env_current_dir";
pub const ENV_TEMP_DIR_BUILTIN: &str = "__muga_std_env_temp_dir";
pub const PROCESS_RUN_BUILTIN: &str = "__muga_std_process_run";
pub const TIME_NOW_UNIX_MILLIS_BUILTIN: &str = "__muga_std_time_now_unix_millis";
pub const HASH_SHA256_HEX_BUILTIN: &str = "__muga_std_hash_sha256_hex";
pub const TEST_ASSERT_TRUE_BUILTIN: &str = "__muga_std_test_assert_true";
pub const TEST_ASSERT_EQ_INT_BUILTIN: &str = "__muga_std_test_assert_eq_int";
pub const TEST_ASSERT_EQ_BOOL_BUILTIN: &str = "__muga_std_test_assert_eq_bool";
pub const TEST_ASSERT_EQ_STRING_BUILTIN: &str = "__muga_std_test_assert_eq_string";
pub const MAP_KEYS_BUILTIN: &str = "__muga_std_map_keys";
pub const MAP_VALUES_BUILTIN: &str = "__muga_std_map_values";
pub const JSON_PARSE_BUILTIN: &str = "__muga_std_json_parse";
pub const JSON_ENCODE_BUILTIN: &str = "__muga_std_json_encode";
pub const JSON_NUMBER_AS_INT_BUILTIN: &str = "__muga_std_json_number_as_int";

#[derive(Clone, Copy, Debug)]
pub struct VirtualPackageFile {
    pub module_path: &'static str,
    pub source: &'static str,
}

pub fn virtual_package_files(package_path: &str) -> Option<&'static [VirtualPackageFile]> {
    match package_path {
        IO_PACKAGE => Some(IO_FILES),
        FS_PACKAGE => Some(FS_FILES),
        PATH_PACKAGE => Some(PATH_FILES),
        ENV_PACKAGE => Some(ENV_FILES),
        PROCESS_PACKAGE => Some(PROCESS_FILES),
        CLI_PACKAGE => Some(CLI_FILES),
        TIME_PACKAGE => Some(TIME_FILES),
        BYTES_PACKAGE => Some(BYTES_FILES),
        HASH_PACKAGE => Some(HASH_FILES),
        TEST_PACKAGE => Some(TEST_FILES),
        OPTION_PACKAGE => Some(OPTION_FILES),
        RESULT_PACKAGE => Some(RESULT_FILES),
        STRING_PACKAGE => Some(STRING_FILES),
        FMT_PACKAGE => Some(FMT_FILES),
        LIST_PACKAGE => Some(LIST_FILES),
        MAP_PACKAGE => Some(MAP_FILES),
        JSON_PACKAGE => Some(JSON_FILES),
        CONFIG_PACKAGE => Some(CONFIG_FILES),
        TASK_PACKAGE => Some(TASK_FILES),
        _ => None,
    }
}

pub fn allows_internal_builtins(package_path: &str) -> bool {
    matches!(
        package_path,
        FS_PACKAGE
            | PATH_PACKAGE
            | ENV_PACKAGE
            | PROCESS_PACKAGE
            | TIME_PACKAGE
            | BYTES_PACKAGE
            | HASH_PACKAGE
            | TEST_PACKAGE
            | MAP_PACKAGE
            | JSON_PACKAGE
            | TASK_PACKAGE
    )
}

const JSON_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "json.muga",
    source: r#"
package std::json

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
  Validation
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

fn shape_error(expected: String): Error {
  Error {
    kind: ErrorKind::UnexpectedToken
    message: "expected JSON ".concat(expected)
    offset: -1
  }
}

fn field_shape_error(key: String, expected: String): Error {
  Error {
    kind: ErrorKind::UnexpectedToken
    message: "expected JSON ".concat(expected).concat(" for object field `").concat(key).concat("`")
    offset: -1
  }
}

fn missing_field_error(key: String): Error {
  Error {
    kind: ErrorKind::UnexpectedToken
    message: "missing JSON object field `".concat(key).concat("`")
    offset: -1
  }
}

fn array_item_shape_error(index: Int, expected: String): Error {
  Error {
    kind: ErrorKind::UnexpectedToken
    message: "expected JSON ".concat(expected).concat(" for array item at index ").concat(index.to_string())
    offset: -1
  }
}

fn array_field_item_shape_error(key: String, index: Int, expected: String): Error {
  Error {
    kind: ErrorKind::UnexpectedToken
    message: "expected JSON ".concat(expected).concat(" for object field `").concat(key).concat("` array item at index ").concat(index.to_string())
    offset: -1
  }
}

fn path_label(path: String): String {
  if path == "" {
    "<root>"
  } else {
    path
  }
}

fn append_path_field(path: String, key: String): String {
  path.concat(".").concat(key)
}

fn append_path_index(path: String, index: Int): String {
  path.concat("[").concat(index.to_string()).concat("]")
}

fn path_shape_error(path: String, expected: String): Error {
  Error {
    kind: ErrorKind::UnexpectedToken
    message: "expected JSON ".concat(expected).concat(" at path ").concat(path_label(path))
    offset: -1
  }
}

fn missing_path_error(path: String): Error {
  Error {
    kind: ErrorKind::UnexpectedToken
    message: "missing JSON value at path ".concat(path_label(path))
    offset: -1
  }
}

fn render_path(path: List[PathSegment]): String {
  mut rendered = ""
  for segment in path {
    rendered = match segment {
      PathSegment::Field(key) => append_path_field(rendered, key)
      PathSegment::Index(index) => append_path_index(rendered, index)
    }
  }
  path_label(rendered)
}

fn at_from_index(value: Value, path: List[PathSegment], index: Int, rendered: String): Result[Option[Value], Error] {
  if index < path.len() {
    match path.get(index) {
      Option::Some(segment) => match segment {
        PathSegment::Field(key) => match value {
          Value::Object(fields) => match fields.get(key) {
            Option::Some(item) => at_from_index(item, path, index + 1, append_path_field(rendered, key))
            Option::None => Result::Ok(Option::None)
          }
          Value::Null => Result::Err(path_shape_error(append_path_field(rendered, key), "Object"))
          Value::Bool(_) => Result::Err(path_shape_error(append_path_field(rendered, key), "Object"))
          Value::Number(_) => Result::Err(path_shape_error(append_path_field(rendered, key), "Object"))
          Value::String(_) => Result::Err(path_shape_error(append_path_field(rendered, key), "Object"))
          Value::Array(_) => Result::Err(path_shape_error(append_path_field(rendered, key), "Object"))
        }
        PathSegment::Index(item_index) => match value {
          Value::Array(items) => match items.get(item_index) {
            Option::Some(item) => at_from_index(item, path, index + 1, append_path_index(rendered, item_index))
            Option::None => Result::Ok(Option::None)
          }
          Value::Null => Result::Err(path_shape_error(append_path_index(rendered, item_index), "Array"))
          Value::Bool(_) => Result::Err(path_shape_error(append_path_index(rendered, item_index), "Array"))
          Value::Number(_) => Result::Err(path_shape_error(append_path_index(rendered, item_index), "Array"))
          Value::String(_) => Result::Err(path_shape_error(append_path_index(rendered, item_index), "Array"))
          Value::Object(_) => Result::Err(path_shape_error(append_path_index(rendered, item_index), "Array"))
        }
      }
      Option::None => Result::Ok(Option::None)
    }
  } else {
    Result::Ok(Option::Some(value))
  }
}

pub fn parse(text: String): Result[Value, Error] {
  __muga_std_json_parse(text)
}

pub fn encode(value: Value): Result[String, Error] {
  __muga_std_json_encode(value)
}

pub fn number_as_int(number: Number): Result[Int, Error] {
  __muga_std_json_number_as_int(number)
}

pub fn decode_or[T](value: Value, fallback: T): Result[T, Error] {
  Result::Err(Error {
    kind: ErrorKind::UnexpectedToken,
    message: "json::decode_or requires compiler schema lowering",
    offset: -1
  })
}

pub fn decode[T](value: Value): Result[T, Error] {
  Result::Err(Error {
    kind: ErrorKind::UnexpectedToken,
    message: "json::decode requires compiler schema lowering",
    offset: -1
  })
}

pub fn to_value[T](value: T): Result[Value, Error] {
  Result::Err(Error {
    kind: ErrorKind::UnexpectedToken,
    message: "json::to_value requires compiler schema lowering",
    offset: -1
  })
}

pub fn encode_typed[T](value: T): Result[String, Error] {
  Result::Err(Error {
    kind: ErrorKind::UnexpectedToken,
    message: "json::encode_typed requires compiler schema lowering",
    offset: -1
  })
}

pub fn int(value: Int): Value {
  Value::Number(Number::Int(value))
}

pub fn as_bool(value: Value): Result[Bool, Error] {
  match value {
    Value::Bool(item) => Result::Ok(item)
    Value::Null => Result::Err(shape_error("Bool"))
    Value::Number(_) => Result::Err(shape_error("Bool"))
    Value::String(_) => Result::Err(shape_error("Bool"))
    Value::Array(_) => Result::Err(shape_error("Bool"))
    Value::Object(_) => Result::Err(shape_error("Bool"))
  }
}

pub fn as_string(value: Value): Result[String, Error] {
  match value {
    Value::String(item) => Result::Ok(item)
    Value::Null => Result::Err(shape_error("String"))
    Value::Bool(_) => Result::Err(shape_error("String"))
    Value::Number(_) => Result::Err(shape_error("String"))
    Value::Array(_) => Result::Err(shape_error("String"))
    Value::Object(_) => Result::Err(shape_error("String"))
  }
}

pub fn as_number(value: Value): Result[Number, Error] {
  match value {
    Value::Number(item) => Result::Ok(item)
    Value::Null => Result::Err(shape_error("Number"))
    Value::Bool(_) => Result::Err(shape_error("Number"))
    Value::String(_) => Result::Err(shape_error("Number"))
    Value::Array(_) => Result::Err(shape_error("Number"))
    Value::Object(_) => Result::Err(shape_error("Number"))
  }
}

pub fn as_int(value: Value): Result[Int, Error] {
  match as_number(value) {
    Result::Ok(number) => number_as_int(number)
    Result::Err(error) => Result::Err(error)
  }
}

pub fn as_array(value: Value): Result[List[Value], Error] {
  match value {
    Value::Array(items) => Result::Ok(items)
    Value::Null => Result::Err(shape_error("Array"))
    Value::Bool(_) => Result::Err(shape_error("Array"))
    Value::Number(_) => Result::Err(shape_error("Array"))
    Value::String(_) => Result::Err(shape_error("Array"))
    Value::Object(_) => Result::Err(shape_error("Array"))
  }
}

pub fn as_object(value: Value): Result[Map[String, Value], Error] {
  match value {
    Value::Object(fields) => Result::Ok(fields)
    Value::Null => Result::Err(shape_error("Object"))
    Value::Bool(_) => Result::Err(shape_error("Object"))
    Value::Number(_) => Result::Err(shape_error("Object"))
    Value::String(_) => Result::Err(shape_error("Object"))
    Value::Array(_) => Result::Err(shape_error("Object"))
  }
}

pub fn at(value: Value, path: List[PathSegment]): Result[Option[Value], Error] {
  at_from_index(value, path, 0, "")
}

pub fn at_required(value: Value, path: List[PathSegment]): Result[Value, Error] {
  match at(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Err(missing_path_error(render_path(path)))
    }
    Result::Err(error) => Result::Err(error)
  }
}

fn at_string_value(value: Value, path: List[PathSegment]): Result[String, Error] {
  match value {
    Value::String(item) => Result::Ok(item)
    Value::Null => Result::Err(path_shape_error(render_path(path), "String"))
    Value::Bool(_) => Result::Err(path_shape_error(render_path(path), "String"))
    Value::Number(_) => Result::Err(path_shape_error(render_path(path), "String"))
    Value::Array(_) => Result::Err(path_shape_error(render_path(path), "String"))
    Value::Object(_) => Result::Err(path_shape_error(render_path(path), "String"))
  }
}

fn at_int_value(value: Value, path: List[PathSegment]): Result[Int, Error] {
  match value {
    Value::Number(number) => match number_as_int(number) {
      Result::Ok(item) => Result::Ok(item)
      Result::Err(_) => Result::Err(path_shape_error(render_path(path), "Int"))
    }
    Value::Null => Result::Err(path_shape_error(render_path(path), "Int"))
    Value::Bool(_) => Result::Err(path_shape_error(render_path(path), "Int"))
    Value::String(_) => Result::Err(path_shape_error(render_path(path), "Int"))
    Value::Array(_) => Result::Err(path_shape_error(render_path(path), "Int"))
    Value::Object(_) => Result::Err(path_shape_error(render_path(path), "Int"))
  }
}

fn at_bool_value(value: Value, path: List[PathSegment]): Result[Bool, Error] {
  match value {
    Value::Bool(item) => Result::Ok(item)
    Value::Null => Result::Err(path_shape_error(render_path(path), "Bool"))
    Value::Number(_) => Result::Err(path_shape_error(render_path(path), "Bool"))
    Value::String(_) => Result::Err(path_shape_error(render_path(path), "Bool"))
    Value::Array(_) => Result::Err(path_shape_error(render_path(path), "Bool"))
    Value::Object(_) => Result::Err(path_shape_error(render_path(path), "Bool"))
  }
}

fn at_array_value(value: Value, path: List[PathSegment]): Result[List[Value], Error] {
  match value {
    Value::Array(items) => Result::Ok(items)
    Value::Null => Result::Err(path_shape_error(render_path(path), "Array"))
    Value::Bool(_) => Result::Err(path_shape_error(render_path(path), "Array"))
    Value::Number(_) => Result::Err(path_shape_error(render_path(path), "Array"))
    Value::String(_) => Result::Err(path_shape_error(render_path(path), "Array"))
    Value::Object(_) => Result::Err(path_shape_error(render_path(path), "Array"))
  }
}

fn at_object_value(value: Value, path: List[PathSegment]): Result[Map[String, Value], Error] {
  match value {
    Value::Object(fields) => Result::Ok(fields)
    Value::Null => Result::Err(path_shape_error(render_path(path), "Object"))
    Value::Bool(_) => Result::Err(path_shape_error(render_path(path), "Object"))
    Value::Number(_) => Result::Err(path_shape_error(render_path(path), "Object"))
    Value::String(_) => Result::Err(path_shape_error(render_path(path), "Object"))
    Value::Array(_) => Result::Err(path_shape_error(render_path(path), "Object"))
  }
}

fn at_string_array_value(value: Value, path: List[PathSegment]): Result[List[String], Error] {
  values = try at_array_value(value, path)
  mut out: List[String] = []
  mut index = 0
  for item in values {
    parsed = try at_string_value(item, path.push(PathSegment::Index(index)))
    out = out.push(parsed)
    index = index + 1
  }
  Result::Ok(out)
}

fn at_int_array_value(value: Value, path: List[PathSegment]): Result[List[Int], Error] {
  values = try at_array_value(value, path)
  mut out: List[Int] = []
  mut index = 0
  for item in values {
    parsed = try at_int_value(item, path.push(PathSegment::Index(index)))
    out = out.push(parsed)
    index = index + 1
  }
  Result::Ok(out)
}

fn at_bool_array_value(value: Value, path: List[PathSegment]): Result[List[Bool], Error] {
  values = try at_array_value(value, path)
  mut out: List[Bool] = []
  mut index = 0
  for item in values {
    parsed = try at_bool_value(item, path.push(PathSegment::Index(index)))
    out = out.push(parsed)
    index = index + 1
  }
  Result::Ok(out)
}

pub fn at_string(value: Value, path: List[PathSegment]): Result[Option[String], Error] {
  match at(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => match at_string_value(item, path) {
        Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
        Result::Err(error) => Result::Err(error)
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_string_or(value: Value, path: List[PathSegment], default_value: String): Result[String, Error] {
  match at_string(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_string_required(value: Value, path: List[PathSegment]): Result[String, Error] {
  match at_string(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Err(missing_path_error(render_path(path)))
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_int(value: Value, path: List[PathSegment]): Result[Option[Int], Error] {
  match at(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => match at_int_value(item, path) {
        Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
        Result::Err(error) => Result::Err(error)
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_int_or(value: Value, path: List[PathSegment], default_value: Int): Result[Int, Error] {
  match at_int(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_int_required(value: Value, path: List[PathSegment]): Result[Int, Error] {
  match at_int(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Err(missing_path_error(render_path(path)))
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_bool(value: Value, path: List[PathSegment]): Result[Option[Bool], Error] {
  match at(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => match at_bool_value(item, path) {
        Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
        Result::Err(error) => Result::Err(error)
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_bool_or(value: Value, path: List[PathSegment], default_value: Bool): Result[Bool, Error] {
  match at_bool(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_bool_required(value: Value, path: List[PathSegment]): Result[Bool, Error] {
  match at_bool(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Err(missing_path_error(render_path(path)))
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_array(value: Value, path: List[PathSegment]): Result[Option[List[Value]], Error] {
  match at(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => match at_array_value(item, path) {
        Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
        Result::Err(error) => Result::Err(error)
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_array_or(value: Value, path: List[PathSegment], default_value: List[Value]): Result[List[Value], Error] {
  match at_array(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_array_required(value: Value, path: List[PathSegment]): Result[List[Value], Error] {
  match at_array(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Err(missing_path_error(render_path(path)))
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_object(value: Value, path: List[PathSegment]): Result[Option[Map[String, Value]], Error] {
  match at(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => match at_object_value(item, path) {
        Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
        Result::Err(error) => Result::Err(error)
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_object_or(value: Value, path: List[PathSegment], default_value: Map[String, Value]): Result[Map[String, Value], Error] {
  match at_object(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_object_required(value: Value, path: List[PathSegment]): Result[Map[String, Value], Error] {
  match at_object(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Err(missing_path_error(render_path(path)))
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_string_array(value: Value, path: List[PathSegment]): Result[Option[List[String]], Error] {
  match at(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => match at_string_array_value(item, path) {
        Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
        Result::Err(error) => Result::Err(error)
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_string_array_or(value: Value, path: List[PathSegment], default_value: List[String]): Result[List[String], Error] {
  match at_string_array(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_string_array_required(value: Value, path: List[PathSegment]): Result[List[String], Error] {
  match at_string_array(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Err(missing_path_error(render_path(path)))
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_int_array(value: Value, path: List[PathSegment]): Result[Option[List[Int]], Error] {
  match at(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => match at_int_array_value(item, path) {
        Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
        Result::Err(error) => Result::Err(error)
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_int_array_or(value: Value, path: List[PathSegment], default_value: List[Int]): Result[List[Int], Error] {
  match at_int_array(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_int_array_required(value: Value, path: List[PathSegment]): Result[List[Int], Error] {
  match at_int_array(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Err(missing_path_error(render_path(path)))
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_bool_array(value: Value, path: List[PathSegment]): Result[Option[List[Bool]], Error] {
  match at(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => match at_bool_array_value(item, path) {
        Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
        Result::Err(error) => Result::Err(error)
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_bool_array_or(value: Value, path: List[PathSegment], default_value: List[Bool]): Result[List[Bool], Error] {
  match at_bool_array(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn at_bool_array_required(value: Value, path: List[PathSegment]): Result[List[Bool], Error] {
  match at_bool_array(value, path) {
    Result::Ok(maybe) => match maybe {
      Option::Some(item) => Result::Ok(item)
      Option::None => Result::Err(missing_path_error(render_path(path)))
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn array_strings(values: List[Value]): Result[List[String], Error] {
  mut out: List[String] = []
  mut index = 0
  for value in values {
    converted: Result[String, Error] = match value {
      Value::String(item) => Result::Ok(item)
      Value::Null => Result::Err(array_item_shape_error(index, "String"))
      Value::Bool(_) => Result::Err(array_item_shape_error(index, "String"))
      Value::Number(_) => Result::Err(array_item_shape_error(index, "String"))
      Value::Array(_) => Result::Err(array_item_shape_error(index, "String"))
      Value::Object(_) => Result::Err(array_item_shape_error(index, "String"))
    }
    parsed = try converted
    out = out.push(parsed)
    index = index + 1
  }
  Result::Ok(out)
}

pub fn array_ints(values: List[Value]): Result[List[Int], Error] {
  mut out: List[Int] = []
  mut index = 0
  for value in values {
    converted: Result[Int, Error] = match value {
      Value::Number(number) => match number_as_int(number) {
        Result::Ok(item) => Result::Ok(item)
        Result::Err(_) => Result::Err(array_item_shape_error(index, "Int"))
      }
      Value::Null => Result::Err(array_item_shape_error(index, "Int"))
      Value::Bool(_) => Result::Err(array_item_shape_error(index, "Int"))
      Value::String(_) => Result::Err(array_item_shape_error(index, "Int"))
      Value::Array(_) => Result::Err(array_item_shape_error(index, "Int"))
      Value::Object(_) => Result::Err(array_item_shape_error(index, "Int"))
    }
    parsed = try converted
    out = out.push(parsed)
    index = index + 1
  }
  Result::Ok(out)
}

pub fn array_bools(values: List[Value]): Result[List[Bool], Error] {
  mut out: List[Bool] = []
  mut index = 0
  for value in values {
    converted: Result[Bool, Error] = match value {
      Value::Bool(item) => Result::Ok(item)
      Value::Null => Result::Err(array_item_shape_error(index, "Bool"))
      Value::Number(_) => Result::Err(array_item_shape_error(index, "Bool"))
      Value::String(_) => Result::Err(array_item_shape_error(index, "Bool"))
      Value::Array(_) => Result::Err(array_item_shape_error(index, "Bool"))
      Value::Object(_) => Result::Err(array_item_shape_error(index, "Bool"))
    }
    parsed = try converted
    out = out.push(parsed)
    index = index + 1
  }
  Result::Ok(out)
}

fn field_array_strings(key: String, values: List[Value]): Result[List[String], Error] {
  mut out: List[String] = []
  mut index = 0
  for value in values {
    converted: Result[String, Error] = match value {
      Value::String(item) => Result::Ok(item)
      Value::Null => Result::Err(array_field_item_shape_error(key, index, "String"))
      Value::Bool(_) => Result::Err(array_field_item_shape_error(key, index, "String"))
      Value::Number(_) => Result::Err(array_field_item_shape_error(key, index, "String"))
      Value::Array(_) => Result::Err(array_field_item_shape_error(key, index, "String"))
      Value::Object(_) => Result::Err(array_field_item_shape_error(key, index, "String"))
    }
    parsed = try converted
    out = out.push(parsed)
    index = index + 1
  }
  Result::Ok(out)
}

fn field_array_ints(key: String, values: List[Value]): Result[List[Int], Error] {
  mut out: List[Int] = []
  mut index = 0
  for value in values {
    converted: Result[Int, Error] = match value {
      Value::Number(number) => match number_as_int(number) {
        Result::Ok(item) => Result::Ok(item)
        Result::Err(_) => Result::Err(array_field_item_shape_error(key, index, "Int"))
      }
      Value::Null => Result::Err(array_field_item_shape_error(key, index, "Int"))
      Value::Bool(_) => Result::Err(array_field_item_shape_error(key, index, "Int"))
      Value::String(_) => Result::Err(array_field_item_shape_error(key, index, "Int"))
      Value::Array(_) => Result::Err(array_field_item_shape_error(key, index, "Int"))
      Value::Object(_) => Result::Err(array_field_item_shape_error(key, index, "Int"))
    }
    parsed = try converted
    out = out.push(parsed)
    index = index + 1
  }
  Result::Ok(out)
}

fn field_array_bools(key: String, values: List[Value]): Result[List[Bool], Error] {
  mut out: List[Bool] = []
  mut index = 0
  for value in values {
    converted: Result[Bool, Error] = match value {
      Value::Bool(item) => Result::Ok(item)
      Value::Null => Result::Err(array_field_item_shape_error(key, index, "Bool"))
      Value::Number(_) => Result::Err(array_field_item_shape_error(key, index, "Bool"))
      Value::String(_) => Result::Err(array_field_item_shape_error(key, index, "Bool"))
      Value::Array(_) => Result::Err(array_field_item_shape_error(key, index, "Bool"))
      Value::Object(_) => Result::Err(array_field_item_shape_error(key, index, "Bool"))
    }
    parsed = try converted
    out = out.push(parsed)
    index = index + 1
  }
  Result::Ok(out)
}

pub fn object_get(value: Value, key: String): Result[Option[Value], Error] {
  match value {
    Value::Object(fields) => Result::Ok(fields.get(key))
    Value::Null => Result::Err(shape_error("Object"))
    Value::Bool(_) => Result::Err(shape_error("Object"))
    Value::Number(_) => Result::Err(shape_error("Object"))
    Value::String(_) => Result::Err(shape_error("Object"))
    Value::Array(_) => Result::Err(shape_error("Object"))
  }
}

pub fn object_array(value: Value, key: String): Result[Option[List[Value]], Error] {
  match object_get(value, key) {
    Result::Ok(field) => match field {
      Option::Some(item) => match item {
        Value::Array(parsed) => Result::Ok(Option::Some(parsed))
        Value::Null => Result::Err(field_shape_error(key, "Array"))
        Value::Bool(_) => Result::Err(field_shape_error(key, "Array"))
        Value::Number(_) => Result::Err(field_shape_error(key, "Array"))
        Value::String(_) => Result::Err(field_shape_error(key, "Array"))
        Value::Object(_) => Result::Err(field_shape_error(key, "Array"))
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_array_or(value: Value, key: String, default_value: List[Value]): Result[List[Value], Error] {
  match object_array(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_array_required(value: Value, key: String): Result[List[Value], Error] {
  match object_array(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Err(missing_field_error(key))
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_string_array(value: Value, key: String): Result[Option[List[String]], Error] {
  match object_array(value, key) {
    Result::Ok(field) => match field {
      Option::Some(values) => match field_array_strings(key, values) {
        Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
        Result::Err(error) => Result::Err(error)
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_string_array_or(value: Value, key: String, default_value: List[String]): Result[List[String], Error] {
  match object_string_array(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_string_array_required(value: Value, key: String): Result[List[String], Error] {
  match object_string_array(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Err(missing_field_error(key))
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_int_array(value: Value, key: String): Result[Option[List[Int]], Error] {
  match object_array(value, key) {
    Result::Ok(field) => match field {
      Option::Some(values) => match field_array_ints(key, values) {
        Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
        Result::Err(error) => Result::Err(error)
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_int_array_or(value: Value, key: String, default_value: List[Int]): Result[List[Int], Error] {
  match object_int_array(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_int_array_required(value: Value, key: String): Result[List[Int], Error] {
  match object_int_array(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Err(missing_field_error(key))
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_bool_array(value: Value, key: String): Result[Option[List[Bool]], Error] {
  match object_array(value, key) {
    Result::Ok(field) => match field {
      Option::Some(values) => match field_array_bools(key, values) {
        Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
        Result::Err(error) => Result::Err(error)
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_bool_array_or(value: Value, key: String, default_value: List[Bool]): Result[List[Bool], Error] {
  match object_bool_array(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_bool_array_required(value: Value, key: String): Result[List[Bool], Error] {
  match object_bool_array(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Err(missing_field_error(key))
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_object(value: Value, key: String): Result[Option[Map[String, Value]], Error] {
  match object_get(value, key) {
    Result::Ok(field) => match field {
      Option::Some(item) => match item {
        Value::Object(parsed) => Result::Ok(Option::Some(parsed))
        Value::Null => Result::Err(field_shape_error(key, "Object"))
        Value::Bool(_) => Result::Err(field_shape_error(key, "Object"))
        Value::Number(_) => Result::Err(field_shape_error(key, "Object"))
        Value::String(_) => Result::Err(field_shape_error(key, "Object"))
        Value::Array(_) => Result::Err(field_shape_error(key, "Object"))
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_object_or(value: Value, key: String, default_value: Map[String, Value]): Result[Map[String, Value], Error] {
  match object_object(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_object_required(value: Value, key: String): Result[Map[String, Value], Error] {
  match object_object(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Err(missing_field_error(key))
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_bool(value: Value, key: String): Result[Option[Bool], Error] {
  match object_get(value, key) {
    Result::Ok(field) => match field {
      Option::Some(item) => match item {
        Value::Bool(parsed) => Result::Ok(Option::Some(parsed))
        Value::Null => Result::Err(field_shape_error(key, "Bool"))
        Value::Number(_) => Result::Err(field_shape_error(key, "Bool"))
        Value::String(_) => Result::Err(field_shape_error(key, "Bool"))
        Value::Array(_) => Result::Err(field_shape_error(key, "Bool"))
        Value::Object(_) => Result::Err(field_shape_error(key, "Bool"))
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_bool_or(value: Value, key: String, default_value: Bool): Result[Bool, Error] {
  match object_bool(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_bool_required(value: Value, key: String): Result[Bool, Error] {
  match object_bool(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Err(missing_field_error(key))
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_string(value: Value, key: String): Result[Option[String], Error] {
  match object_get(value, key) {
    Result::Ok(field) => match field {
      Option::Some(item) => match item {
        Value::String(parsed) => Result::Ok(Option::Some(parsed))
        Value::Null => Result::Err(field_shape_error(key, "String"))
        Value::Bool(_) => Result::Err(field_shape_error(key, "String"))
        Value::Number(_) => Result::Err(field_shape_error(key, "String"))
        Value::Array(_) => Result::Err(field_shape_error(key, "String"))
        Value::Object(_) => Result::Err(field_shape_error(key, "String"))
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_string_or(value: Value, key: String, default_value: String): Result[String, Error] {
  match object_string(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_string_required(value: Value, key: String): Result[String, Error] {
  match object_string(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Err(missing_field_error(key))
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_int(value: Value, key: String): Result[Option[Int], Error] {
  match object_get(value, key) {
    Result::Ok(field) => match field {
      Option::Some(item) => match item {
        Value::Number(number) => match number_as_int(number) {
          Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
          Result::Err(error) => Result::Err(error)
        }
        Value::Null => Result::Err(field_shape_error(key, "Number"))
        Value::Bool(_) => Result::Err(field_shape_error(key, "Number"))
        Value::String(_) => Result::Err(field_shape_error(key, "Number"))
        Value::Array(_) => Result::Err(field_shape_error(key, "Number"))
        Value::Object(_) => Result::Err(field_shape_error(key, "Number"))
      }
      Option::None => Result::Ok(Option::None)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_int_or(value: Value, key: String, default_value: Int): Result[Int, Error] {
  match object_int(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(error) => Result::Err(error)
  }
}

pub fn object_int_required(value: Value, key: String): Result[Int, Error] {
  match object_int(value, key) {
    Result::Ok(field) => match field {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Err(missing_field_error(key))
    }
    Result::Err(error) => Result::Err(error)
  }
}
"#,
}];

const LIST_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "list.muga",
    source: r#"
package std::list

pub fn map[T, U](items: List[T], f: T -> U): List[U] {
  mut out: List[U] = []
  for item in items {
    out = out.push(f(item))
  }
  out
}

pub fn filter[T](items: List[T], predicate: T -> Bool): List[T] {
  mut out: List[T] = []
  for item in items {
    if predicate(item) {
      out = out.push(item)
    }
  }
  out
}

pub fn fold[T, U](items: List[T], initial: U, f: (U, T) -> U): U {
  mut acc: U = initial
  for item in items {
    acc = f(acc, item)
  }
  acc
}

pub fn any[T](items: List[T], predicate: T -> Bool): Bool {
  for item in items {
    if predicate(item) {
      return true
    }
  }
  false
}

pub fn all[T](items: List[T], predicate: T -> Bool): Bool {
  for item in items {
    if predicate(item) {
    } else {
      return false
    }
  }
  true
}
"#,
}];

const CONFIG_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "config.muga",
    source: r#"
package std::config

import std::path

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

pub fn load_json_or[T](file_path: path::Path, fallback: T): Result[T, Error] {
  Result::Err(Error {
    kind: ErrorKind::Decode,
    path: file_path,
    message: "config::load_json_or requires compiler schema lowering",
    offset: -1,
    raw_code: Option::None
  })
}

pub fn load_json[T](file_path: path::Path): Result[T, Error] {
  Result::Err(Error {
    kind: ErrorKind::Decode,
    path: file_path,
    message: "config::load_json requires compiler schema lowering",
    offset: -1,
    raw_code: Option::None
  })
}
"#,
}];

const MAP_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "map.muga",
    source: r#"
package std::map

pub fn keys[K, V](items: Map[K, V]): List[K] {
  __muga_std_map_keys(items)
}

pub fn values[K, V](items: Map[K, V]): List[V] {
  __muga_std_map_values(items)
}
"#,
}];

const TASK_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "task.muga",
    source: r#"
package std::task

pub fn join[T](task: Task[T]): T {
  __muga_std_task_join(task)
}

pub fn spawn_map[T, U](items: List[T], f: T -> U): List[U] {
  group {
    mut tasks: List[Task[U]] = []
    for item in items {
      tasks = tasks.push(spawn f(item))
    }
    mut out: List[U] = []
    for item_task in tasks {
      out = out.push(item_task.join())
    }
    out
  }
}
"#,
}];

const OPTION_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "option.muga",
    source: r#"
package std::option

pub fn is_some[T](value: Option[T]): Bool {
  match value {
    Option::Some(item) => true
    Option::None => false
  }
}

pub fn is_none[T](value: Option[T]): Bool {
  match value {
    Option::Some(item) => false
    Option::None => true
  }
}

pub fn map[T, U](value: Option[T], f: T -> U): Option[U] {
  match value {
    Option::Some(item) => Option::Some(f(item))
    Option::None => Option::None
  }
}

pub fn and_then[T, U](value: Option[T], f: T -> Option[U]): Option[U] {
  match value {
    Option::Some(item) => f(item)
    Option::None => Option::None
  }
}

pub fn value_or[T](value: Option[T], fallback: T): T {
  match value {
    Option::Some(item) => item
    Option::None => fallback
  }
}
"#,
}];

const RESULT_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "result.muga",
    source: r#"
package std::result

pub fn is_ok[T, E](value: Result[T, E]): Bool {
  match value {
    Result::Ok(item) => true
    Result::Err(error) => false
  }
}

pub fn is_err[T, E](value: Result[T, E]): Bool {
  match value {
    Result::Ok(item) => false
    Result::Err(error) => true
  }
}

pub fn map[T, E, U](value: Result[T, E], f: T -> U): Result[U, E] {
  match value {
    Result::Ok(item) => Result::Ok(f(item))
    Result::Err(error) => Result::Err(error)
  }
}

pub fn map_err[T, E, F](value: Result[T, E], f: E -> F): Result[T, F] {
  match value {
    Result::Ok(item) => Result::Ok(item)
    Result::Err(error) => Result::Err(f(error))
  }
}

pub fn and_then[T, E, U](value: Result[T, E], f: T -> Result[U, E]): Result[U, E] {
  match value {
    Result::Ok(item) => f(item)
    Result::Err(error) => Result::Err(error)
  }
}

pub fn value_or[T, E](value: Result[T, E], fallback: T): T {
  match value {
    Result::Ok(item) => item
    Result::Err(error) => fallback
  }
}
"#,
}];

const STRING_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "string.muga",
    source: r#"
package std::string

pub fn concat_all(parts: List[String]): String {
  mut out = ""
  for part in parts {
    out = out.concat(part)
  }
  out
}

pub fn join(parts: List[String], separator: String): String {
  mut out = ""
  mut first = true
  for part in parts {
    if first {
      out = part
      first = false
    } else {
      out = out.concat(separator).concat(part)
    }
  }
  out
}
"#,
}];

const FMT_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "fmt.muga",
    source: r#"
package std::fmt

pub enum FormatError {
  MissingValue(Int)
  UnclosedPlaceholder(Int)
  UnexpectedClose(Int)
}

pub fn repeat(text: String, count: Int): String {
  mut out = ""
  mut index = 0
  while index < count {
    out = out.concat(text)
    index = index + 1
  }
  out
}

fn first_fill_scalar(fill: String): String {
  if fill.is_empty() {
    ""
  } else {
    match fill.slice_chars(0, 1) {
      Result::Ok(value) => value
      Result::Err(message) => ""
    }
  }
}

fn missing_width(text: String, width: Int): Int {
  length = text.char_count()
  if width > length {
    width - length
  } else {
    0
  }
}

pub fn pad_left(text: String, width: Int, fill: String): String {
  padding = repeat(first_fill_scalar(fill), missing_width(text, width))
  padding.concat(text)
}

pub fn pad_right(text: String, width: Int, fill: String): String {
  padding = repeat(first_fill_scalar(fill), missing_width(text, width))
  text.concat(padding)
}

pub fn truncate_chars(text: String, max_chars: Int): String {
  if max_chars < 1 {
    ""
  } else if text.char_count() < max_chars {
    text
  } else {
    match text.slice_chars(0, max_chars) {
      Result::Ok(value) => value
      Result::Err(message) => ""
    }
  }
}

fn char_at(text: String, index: Int): String {
  match text.slice_chars(index, 1) {
    Result::Ok(value) => value
    Result::Err(message) => ""
  }
}

fn value_at(values: List[String], index: Int): Result[String, FormatError] {
  match values.get(index) {
    Option::Some(value) => Result::Ok(value)
    Option::None => Result::Err(FormatError::MissingValue(index))
  }
}

pub fn format_values(template: String, values: List[String]): Result[String, FormatError] {
  mut out = ""
  mut index = 0
  mut value_index = 0
  length = template.char_count()
  while index < length {
    ch = char_at(template, index)
    next_index = index + 1
    next = if next_index < length {
      char_at(template, next_index)
    } else {
      ""
    }
    if ch == "{" {
      if next == "{" {
        out = out.concat("{")
        index = index + 2
      } else if next == "}" {
        value = try value_at(values, value_index)
        out = out.concat(value)
        value_index = value_index + 1
        index = index + 2
      } else {
        return Result::Err(FormatError::UnclosedPlaceholder(index))
      }
    } else if ch == "}" {
      if next == "}" {
        out = out.concat("}")
        index = index + 2
      } else {
        return Result::Err(FormatError::UnexpectedClose(index))
      }
    } else {
      out = out.concat(ch)
      index = index + 1
    }
  }
  Result::Ok(out)
}
"#,
}];

const IO_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "io.muga",
    source: r#"
package std::io

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
"#,
}];

const PATH_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "path.muga",
    source: r#"
package std::path

pub record Path {
  text: String
}

pub fn from_string(text: String): Path {
  Path {
    text: text
  }
}

pub fn as_string(path: Path): String {
  path.text
}

pub fn join(base: Path, child: String): Path {
  Path {
    text: __muga_std_path_join(base.text, child)
  }
}

pub fn normalize(path: Path): Path {
  Path {
    text: __muga_std_path_normalize(path.text)
  }
}

pub fn file_name(path: Path): Option[String] {
  __muga_std_path_file_name(path.text)
}

pub fn with_file_name(path: Path, new_file_name: String): Path {
  Path {
    text: __muga_std_path_with_file_name(path.text, new_file_name)
  }
}

pub fn parent(path: Path): Option[Path] {
  match __muga_std_path_parent(path.text) {
    Option::Some(value) => Option::Some(Path {
      text: value
    })
    Option::None => Option::None
  }
}

pub fn strip_prefix(path: Path, base: Path): Option[Path] {
  match __muga_std_path_strip_prefix(path.text, base.text) {
    Option::Some(value) => Option::Some(Path {
      text: value
    })
    Option::None => Option::None
  }
}

pub fn extension(path: Path): Option[String] {
  __muga_std_path_extension(path.text)
}

pub fn file_stem(path: Path): Option[String] {
  __muga_std_path_file_stem(path.text)
}

pub fn with_extension(path: Path, new_extension: String): Path {
  Path {
    text: __muga_std_path_with_extension(path.text, new_extension)
  }
}

pub fn is_absolute(path: Path): Bool {
  __muga_std_path_is_absolute(path.text)
}
"#,
}];

const BYTES_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "bytes.muga",
    source: r#"
package std::bytes

pub opaque type Bytes

pub fn size(bytes: Bytes): Int {
  __muga_std_bytes_size(bytes)
}

pub fn empty(bytes: Bytes): Bool {
  __muga_std_bytes_is_empty(bytes)
}

pub fn at(bytes: Bytes, index: Int): Option[Int] {
  __muga_std_bytes_at(bytes, index)
}
"#,
}];

const HASH_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "hash.muga",
    source: r#"
package std::hash

import std::bytes

pub fn sha256_hex(bytes: bytes::Bytes): String {
  __muga_std_hash_sha256_hex(bytes)
}
"#,
}];

const FS_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "fs.muga",
    source: r#"
package std::fs

import std::bytes
import std::io
import std::path
import std::time

pub opaque type File

pub record FileMetadata {
  size: Int
  modified: time::UnixMillis
}

pub record PathStatus {
  exists: Bool
  is_file: Bool
  is_dir: Bool
}

pub enum PathKind {
  Missing
  File
  Directory
  Other
}

pub record PathInfo {
  status: PathStatus
  kind: PathKind
}

pub record PathMetadata {
  status: PathStatus
  kind: PathKind
  modified: time::UnixMillis
}

pub record PathSizeMetadata {
  status: PathStatus
  kind: PathKind
  modified: time::UnixMillis
  size: Option[Int]
}

pub record DirectorySizeMetadata {
  size: Int
  file_count: Int
  directory_count: Int
  other_count: Int
}

pub fn read_text(path: String): Result[String, io::IOError] {
  __muga_std_fs_read_text(path)
}

pub fn read_bytes(path: String): Result[bytes::Bytes, io::IOError] {
  __muga_std_fs_read_bytes(path)
}

pub fn read_resource_text(package_path: String, resource_path: String): Result[String, io::IOError] {
  __muga_std_fs_read_resource_text(package_path, resource_path)
}

pub fn read_resource_bytes(package_path: String, resource_path: String): Result[bytes::Bytes, io::IOError] {
  __muga_std_fs_read_resource_bytes(package_path, resource_path)
}

pub fn write_text(path: String, text: String): Result[Unit, io::IOError] {
  __muga_std_fs_write_text(path, text)
}

pub fn write_bytes(path: String, data: bytes::Bytes): Result[Unit, io::IOError] {
  __muga_std_fs_write_bytes(path, data)
}

pub fn read_text_path(file_path: path::Path): Result[String, io::IOError] {
  __muga_std_fs_read_text(path::as_string(file_path))
}

pub fn read_bytes_path(file_path: path::Path): Result[bytes::Bytes, io::IOError] {
  __muga_std_fs_read_bytes(path::as_string(file_path))
}

pub fn write_text_path(file_path: path::Path, text: String): Result[Unit, io::IOError] {
  __muga_std_fs_write_text(path::as_string(file_path), text)
}

pub fn write_bytes_path(file_path: path::Path, data: bytes::Bytes): Result[Unit, io::IOError] {
  __muga_std_fs_write_bytes(path::as_string(file_path), data)
}

pub fn open_text(file_path: path::Path): Result[File, io::IOError] {
  __muga_std_fs_open_text(path::as_string(file_path))
}

pub fn create_text(file_path: path::Path): Result[File, io::IOError] {
  __muga_std_fs_create_text(path::as_string(file_path))
}

pub fn append_text(file_path: path::Path): Result[File, io::IOError] {
  __muga_std_fs_append_text(path::as_string(file_path))
}

pub fn read_text_from(file: File): Result[String, io::IOError] {
  __muga_std_fs_read_text_from(file)
}

pub fn write_text_to(file: File, text: String): Result[Unit, io::IOError] {
  __muga_std_fs_write_text_to(file, text)
}

pub fn flush(file: File): Result[Unit, io::IOError] {
  __muga_std_fs_flush(file)
}

pub fn close(file: File): Result[Unit, io::IOError] {
  __muga_std_fs_close(file)
}

pub fn read_dir_path(dir_path: path::Path): Result[List[path::Path], io::IOError] {
  __muga_std_fs_read_dir(path::as_string(dir_path))
}

pub fn read_dir_recursive_path(root_path: path::Path): Result[List[path::Path], io::IOError] {
  __muga_std_fs_read_dir_recursive(path::as_string(root_path))
}

pub fn directory_size_metadata_path(root_path: path::Path): Result[DirectorySizeMetadata, io::IOError] {
  __muga_std_fs_directory_size_metadata(path::as_string(root_path))
}

pub fn canonicalize_path(target_path: path::Path): Result[path::Path, io::IOError] {
  match __muga_std_fs_canonicalize(path::as_string(target_path)) {
    Result::Ok(text) => Result::Ok(path::from_string(text))
    Result::Err(error) => Result::Err(error)
  }
}

pub fn create_dir_path(dir_path: path::Path): Result[Unit, io::IOError] {
  __muga_std_fs_create_dir(path::as_string(dir_path))
}

pub fn create_dir_all_path(dir_path: path::Path): Result[Unit, io::IOError] {
  __muga_std_fs_create_dir_all(path::as_string(dir_path))
}

pub fn remove_file_path(file_path: path::Path): Result[Unit, io::IOError] {
  __muga_std_fs_remove_file(path::as_string(file_path))
}

pub fn remove_dir_path(dir_path: path::Path): Result[Unit, io::IOError] {
  __muga_std_fs_remove_dir(path::as_string(dir_path))
}

pub fn remove_dir_all_path(dir_path: path::Path): Result[Unit, io::IOError] {
  __muga_std_fs_remove_dir_all(path::as_string(dir_path))
}

pub fn copy_file_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError] {
  __muga_std_fs_copy_file(path::as_string(from_path), path::as_string(to_path))
}

pub fn copy_dir_all_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError] {
  __muga_std_fs_copy_dir_all(path::as_string(from_path), path::as_string(to_path))
}

pub fn move_dir_all_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError] {
  __muga_std_fs_move_dir_all(path::as_string(from_path), path::as_string(to_path))
}

pub fn rename_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError] {
  __muga_std_fs_rename(path::as_string(from_path), path::as_string(to_path))
}

pub fn file_size_path(file_path: path::Path): Result[Int, io::IOError] {
  __muga_std_fs_file_size(path::as_string(file_path))
}

pub fn modified_unix_millis_path(target_path: path::Path): Result[time::UnixMillis, io::IOError] {
  match __muga_std_fs_modified_unix_millis(path::as_string(target_path)) {
    Result::Ok(value) => Result::Ok(time::UnixMillis {
      value: value
    })
    Result::Err(error) => Result::Err(error)
  }
}

pub fn file_metadata_path(file_path: path::Path): Result[FileMetadata, io::IOError] {
  size = try file_size_path(file_path)
  modified = try modified_unix_millis_path(file_path)
  Result::Ok(FileMetadata {
    size: size,
    modified: modified
  })
}

pub fn path_status(target_path: path::Path): PathStatus {
  PathStatus {
    exists: exists_path(target_path),
    is_file: is_file_path(target_path),
    is_dir: is_dir_path(target_path)
  }
}

fn path_kind_from_status(status: PathStatus): PathKind {
  if !status.exists {
    PathKind::Missing
  } else if status.is_file {
    PathKind::File
  } else if status.is_dir {
    PathKind::Directory
  } else {
    PathKind::Other
  }
}

pub fn path_kind(target_path: path::Path): PathKind {
  path_kind_from_status(path_status(target_path))
}

pub fn path_info(target_path: path::Path): PathInfo {
  status = path_status(target_path)
  kind = path_kind_from_status(status)
  PathInfo { status: status, kind: kind }
}

pub fn path_metadata_path(target_path: path::Path): Result[PathMetadata, io::IOError] {
  modified = try modified_unix_millis_path(target_path)
  info = path_info(target_path)
  Result::Ok(PathMetadata {
    status: info.status,
    kind: info.kind,
    modified: modified
  })
}

pub fn path_size_metadata_path(target_path: path::Path): Result[PathSizeMetadata, io::IOError] {
  metadata = try path_metadata_path(target_path)
  size: Option[Int] = if metadata.status.is_file {
    file_size = try file_size_path(target_path)
    Option::Some(file_size)
  } else {
    Option::None
  }
  Result::Ok(PathSizeMetadata {
    status: metadata.status,
    kind: metadata.kind,
    modified: metadata.modified,
    size: size
  })
}

pub fn exists_path(file_path: path::Path): Bool {
  __muga_std_fs_exists(path::as_string(file_path))
}

pub fn is_file_path(file_path: path::Path): Bool {
  __muga_std_fs_is_file(path::as_string(file_path))
}

pub fn is_dir_path(file_path: path::Path): Bool {
  __muga_std_fs_is_dir(path::as_string(file_path))
}
"#,
}];

const ENV_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "env.muga",
    source: r#"
package std::env

import std::io
import std::path

pub fn get_var(name: String): Option[String] {
  __muga_std_env_get_var(name)
}

pub fn args(): List[String] {
  __muga_std_env_args()
}

pub fn current_dir(): Result[path::Path, io::IOError] {
  match __muga_std_env_current_dir() {
    Result::Ok(text) => Result::Ok(path::from_string(text))
    Result::Err(error) => Result::Err(error)
  }
}

pub fn temp_dir(): Result[path::Path, io::IOError] {
  match __muga_std_env_temp_dir() {
    Result::Ok(text) => Result::Ok(path::from_string(text))
    Result::Err(error) => Result::Err(error)
  }
}
"#,
}];

const PROCESS_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "process.muga",
    source: r#"
package std::process

import std::path

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

pub fn default_options(): Options {
  Options {
    cwd: Option::None
    env: []
  }
}

pub fn run(command: String, args: List[String]): Result[Output, Error] {
  run_with(command, args, default_options())
}

pub fn run_with(command: String, args: List[String], options: Options): Result[Output, Error] {
  __muga_std_process_run(command, args, options)
}
"#,
}];

const CLI_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "cli.muga",
    source: r#"
package std::cli

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

pub fn parse_or[T](args: List[String], defaults: T): Result[T, Error] {
  Result::Err(Error {
    kind: ErrorKind::UnsupportedTarget,
    argument: "",
    message: "cli::parse_or requires compiler schema lowering"
  })
}

pub fn parse[T](args: List[String]): Result[T, Error] {
  Result::Err(Error {
    kind: ErrorKind::UnsupportedTarget,
    argument: "",
    message: "cli::parse requires compiler schema lowering"
  })
}

pub fn parse_request[T](args: List[String], program: String): Result[Request[T], Error] {
  Result::Err(Error {
    kind: ErrorKind::UnsupportedTarget,
    argument: "",
    message: "cli::parse_request requires compiler schema lowering"
  })
}

pub fn parse_request_or[T](args: List[String], program: String, defaults: T): Result[Request[T], Error] {
  Result::Err(Error {
    kind: ErrorKind::UnsupportedTarget,
    argument: "",
    message: "cli::parse_request_or requires compiler schema lowering"
  })
}

pub fn usage_for[T](program: String, defaults: T): String {
  "cli::usage_for requires compiler schema lowering"
}

pub fn usage_for_required[T](program: String): String {
  "cli::usage_for_required requires compiler schema lowering"
}

pub fn help_for[T](program: String, defaults: T): String {
  "cli::help_for requires compiler schema lowering"
}

pub fn help_for_required[T](program: String): String {
  "cli::help_for_required requires compiler schema lowering"
}

fn long_marker(name: String): String {
  "--".concat(name)
}

fn short_marker(name: String): String {
  "-".concat(name)
}

fn equals_marker(name: String): String {
  long_marker(name).concat("=")
}

fn value_after_prefix(value: String, prefix: String): Option[String] {
  if value.starts_with(prefix) {
    start = prefix.char_count()
    count = value.char_count() - start
    match value.slice_chars(start, count) {
      Result::Ok(item) => Option::Some(item)
      Result::Err(message) => Option::None
    }
  } else {
    Option::None
  }
}

fn separate_option_value(value: String): Option[String] {
  if value.starts_with("--") {
    Option::None
  } else {
    Option::Some(value)
  }
}

fn separate_option_at(args: List[String], index: Int): Option[String] {
  match args.get(index + 1) {
    Option::Some(value) => separate_option_value(value)
    Option::None => Option::None
  }
}

fn option_value_for_arg(args: List[String], index: Int, arg: String, marker: String, equals: String): Option[String] {
  if arg == marker {
    separate_option_at(args, index)
  } else {
    value_after_prefix(arg, equals)
  }
}

fn option_from_index(args: List[String], index: Int, marker: String, equals: String): Option[String] {
  if index < args.len() {
    match args.get(index) {
      Option::Some(arg) => if arg == "--" {
        Option::None
      } else {
        match option_value_for_arg(args, index, arg, marker, equals) {
          Option::Some(value) => Option::Some(value)
          Option::None => option_from_index(args, index + 1, marker, equals)
        }
      }
      Option::None => Option::None
    }
  } else {
    Option::None
  }
}

fn option_values_from_args(args: List[String], marker: String, equals: String): List[String] {
  mut out: List[String] = []
  mut index = 0
  for arg in args {
    if arg == "--" {
      return out
    }
    out = match option_value_for_arg(args, index, arg, marker, equals) {
      Option::Some(value) => out.push(value)
      Option::None => out
    }
    index = index + 1
  }
  out
}

fn positional_label(index: Int): String {
  "argument ".concat(index.to_string())
}

fn option_label(name: String): String {
  long_marker(name)
}

fn parse_int_value(label: String, value: String): Result[Int, String] {
  match value.parse_int() {
    Result::Ok(parsed) => Result::Ok(parsed)
    Result::Err(message) => Result::Err("invalid Int for ".concat(label).concat(": ").concat(value).concat(" (").concat(message).concat(")"))
  }
}

fn parse_bool_value(label: String, value: String): Result[Bool, String] {
  match value.parse_bool() {
    Result::Ok(parsed) => Result::Ok(parsed)
    Result::Err(message) => Result::Err("invalid Bool for ".concat(label).concat(": ").concat(value).concat(" (").concat(message).concat(")"))
  }
}

pub fn positional(args: List[String], index: Int): Option[String] {
  mut seen = 0
  mut parsing_options = true
  for arg in args {
    if parsing_options {
      if arg == "--" {
        parsing_options = false
      } else {
        if arg.starts_with("--") {
        } else {
          if seen == index {
            return Option::Some(arg)
          }
          seen = seen + 1
        }
      }
    } else {
      if seen == index {
        return Option::Some(arg)
      }
      seen = seen + 1
    }
  }
  Option::None
}

pub fn positional_or(args: List[String], index: Int, default_value: String): String {
  match positional(args, index) {
    Option::Some(value) => value
    Option::None => default_value
  }
}

pub fn positional_int(args: List[String], index: Int): Result[Option[Int], String] {
  match positional(args, index) {
    Option::Some(value) => match parse_int_value(positional_label(index), value) {
      Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
      Result::Err(message) => Result::Err(message)
    }
    Option::None => Result::Ok(Option::None)
  }
}

pub fn positional_int_or(args: List[String], index: Int, default_value: Int): Result[Int, String] {
  match positional_int(args, index) {
    Result::Ok(value) => match value {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(message) => Result::Err(message)
  }
}

pub fn positional_bool(args: List[String], index: Int): Result[Option[Bool], String] {
  match positional(args, index) {
    Option::Some(value) => match parse_bool_value(positional_label(index), value) {
      Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
      Result::Err(message) => Result::Err(message)
    }
    Option::None => Result::Ok(Option::None)
  }
}

pub fn positional_bool_or(args: List[String], index: Int, default_value: Bool): Result[Bool, String] {
  match positional_bool(args, index) {
    Result::Ok(value) => match value {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(message) => Result::Err(message)
  }
}

pub fn has_flag(args: List[String], name: String): Bool {
  marker = long_marker(name)
  for arg in args {
    if arg == "--" {
      return false
    }
    if arg == marker {
      return true
    }
  }
  false
}

pub fn has_short_flag(args: List[String], name: String): Bool {
  marker = short_marker(name)
  for arg in args {
    if arg == "--" {
      return false
    }
    if arg == marker {
      return true
    }
  }
  false
}

pub fn help_requested(args: List[String]): Bool {
  has_flag(args, "help") or has_short_flag(args, "h")
}

pub fn option(args: List[String], name: String): Option[String] {
  marker = long_marker(name)
  equals = equals_marker(name)
  option_from_index(args, 0, marker, equals)
}

pub fn option_or(args: List[String], name: String, default_value: String): String {
  match option(args, name) {
    Option::Some(value) => value
    Option::None => default_value
  }
}

pub fn option_values(args: List[String], name: String): List[String] {
  marker = long_marker(name)
  equals = equals_marker(name)
  option_values_from_args(args, marker, equals)
}

pub fn option_values_or(args: List[String], name: String, default_value: List[String]): List[String] {
  values = option_values(args, name)
  if values.is_empty() {
    default_value
  } else {
    values
  }
}

pub fn option_int(args: List[String], name: String): Result[Option[Int], String] {
  match option(args, name) {
    Option::Some(value) => match parse_int_value(option_label(name), value) {
      Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
      Result::Err(message) => Result::Err(message)
    }
    Option::None => Result::Ok(Option::None)
  }
}

pub fn option_int_or(args: List[String], name: String, default_value: Int): Result[Int, String] {
  match option_int(args, name) {
    Result::Ok(value) => match value {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(message) => Result::Err(message)
  }
}

pub fn option_bool(args: List[String], name: String): Result[Option[Bool], String] {
  match option(args, name) {
    Option::Some(value) => match parse_bool_value(option_label(name), value) {
      Result::Ok(parsed) => Result::Ok(Option::Some(parsed))
      Result::Err(message) => Result::Err(message)
    }
    Option::None => Result::Ok(Option::None)
  }
}

pub fn option_bool_or(args: List[String], name: String, default_value: Bool): Result[Bool, String] {
  match option_bool(args, name) {
    Result::Ok(value) => match value {
      Option::Some(parsed) => Result::Ok(parsed)
      Option::None => Result::Ok(default_value)
    }
    Result::Err(message) => Result::Err(message)
  }
}
"#,
}];

const TIME_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "time.muga",
    source: r#"
package std::time

pub record UnixMillis {
  value: Int
}

pub fn now_unix_millis(): UnixMillis {
  UnixMillis {
    value: __muga_std_time_now_unix_millis()
  }
}
"#,
}];

const TEST_FILES: &[VirtualPackageFile] = &[VirtualPackageFile {
    module_path: "test.muga",
    source: r#"
package std::test

pub fn assert_true(value: Bool): Result[Unit, String] {
  __muga_std_test_assert_true(value)
}

pub fn assert_eq_int(expected: Int, actual: Int): Result[Unit, String] {
  __muga_std_test_assert_eq_int(expected, actual)
}

pub fn assert_eq_bool(expected: Bool, actual: Bool): Result[Unit, String] {
  __muga_std_test_assert_eq_bool(expected, actual)
}

pub fn assert_eq_string(expected: String, actual: String): Result[Unit, String] {
  __muga_std_test_assert_eq_string(expected, actual)
}
"#,
}];
