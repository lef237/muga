use std::{
    cell::RefCell,
    collections::HashMap,
    env as process_env, fmt, fs, io,
    path::{Component, Path, PathBuf},
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    bytecode::*,
    cli_schema::{CliCommandVariantSchema, CliFieldSchema, CliSchema, CliValueSchema},
    diagnostic::{Diagnostic, RelatedNote},
    identity::{BindingKind, LocalId, PackageItemId},
    json_decode::{
        JsonDecodeFieldSchema, JsonDecodeSchema, JsonDecodeValidationRule, JsonDecodeVariantSchema,
    },
    known_enum::{self, KnownEnum, KnownEnumVariant},
    prelude::{self, BuiltinId},
    span::Span,
    symbol::Symbol,
};

type EnvRef = Rc<RefCell<Env>>;
type OutputRef = Rc<RefCell<String>>;
type RuntimeDiagnosticsRef = Rc<RefCell<Vec<Diagnostic>>>;
type RuntimeCallStackRef = Rc<RefCell<Vec<RuntimeCallFrame>>>;
type RuntimeHandlesRef = Rc<RefCell<RuntimeHandles>>;
type PackageResourceRootsRef = Rc<HashMap<String, PathBuf>>;

const STD_FS_FILE_HANDLE_FAMILY: &str = "std::fs::File";

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Bool(bool),
    String(String),
    Bytes(Vec<u8>),
    Unit,
    List(Vec<Value>),
    Map(MapValue),
    Enum(EnumValue),
    Record(RecordValue),
    RuntimeHandle(RuntimeHandleValue),
    Function(Rc<ClosureValue>),
    Builtin(BuiltinId),
}

#[derive(Clone, Debug)]
pub struct RuntimeHandleValue {
    family: &'static str,
    slot: usize,
    generation: u64,
}

#[derive(Debug, Default)]
struct RuntimeHandles {
    std_fs_files: Vec<StdFsFileSlot>,
}

#[derive(Debug)]
enum StdFsFileSlot {
    Open {
        path: String,
        file: fs::File,
        mode: StdFsFileMode,
        generation: u64,
    },
    Closed {
        generation: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StdFsFileMode {
    Read,
    Write,
    Append,
}

impl StdFsFileMode {
    fn can_read(self) -> bool {
        matches!(self, Self::Read)
    }

    fn can_write(self) -> bool {
        matches!(self, Self::Write | Self::Append)
    }
}

#[derive(Clone, Debug)]
pub struct RecordValue {
    type_name: String,
    fields: Vec<RecordFieldValue>,
}

#[derive(Clone, Debug)]
pub struct MapValue {
    entries: Vec<MapEntryValue>,
}

#[derive(Clone, Debug)]
pub struct EnumValue {
    type_name: String,
    variant_name: String,
    payload: Option<Box<Value>>,
}

#[derive(Clone, Debug)]
struct MapEntryValue {
    key: MapKey,
    value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MapKey {
    Int(i64),
    Bool(bool),
    String(String),
}

impl MapKey {
    fn into_value(self) -> Value {
        match self {
            Self::Int(value) => Value::Int(value),
            Self::Bool(value) => Value::Bool(value),
            Self::String(value) => Value::String(value),
        }
    }
}

#[derive(Clone, Debug)]
struct RecordFieldValue {
    name: String,
    value: Value,
}

fn enum_value(known: &KnownEnum, variant: KnownEnumVariant, payload: Option<Value>) -> Value {
    Value::Enum(EnumValue {
        type_name: known.name.to_string(),
        variant_name: variant.name.to_string(),
        payload: payload.map(Box::new),
    })
}

fn option_some(value: Value) -> Value {
    let option = known_enum::option_enum();
    let some = option
        .variant(known_enum::OPTION_SOME_NAME)
        .expect("known Option enum should define Some");
    enum_value(option, some, Some(value))
}

fn option_none() -> Value {
    let option = known_enum::option_enum();
    let none = option
        .variant(known_enum::OPTION_NONE_NAME)
        .expect("known Option enum should define None");
    enum_value(option, none, None)
}

fn result_ok(value: Value) -> Value {
    let result = known_enum::result_enum();
    let ok = result
        .variant(known_enum::RESULT_OK_NAME)
        .expect("known Result enum should define Ok");
    enum_value(result, ok, Some(value))
}

fn result_err(value: Value) -> Value {
    let result = known_enum::result_enum();
    let err = result
        .variant(known_enum::RESULT_ERR_NAME)
        .expect("known Result enum should define Err");
    enum_value(result, err, Some(value))
}

fn cli_request_help(value: String) -> Value {
    Value::Enum(EnumValue {
        type_name: crate::std_package::CLI_REQUEST_MANGLED_NAME.to_string(),
        variant_name: "Help".to_string(),
        payload: Some(Box::new(Value::String(value))),
    })
}

fn cli_request_parsed(value: Value) -> Value {
    Value::Enum(EnumValue {
        type_name: crate::std_package::CLI_REQUEST_MANGLED_NAME.to_string(),
        variant_name: "Parsed".to_string(),
        payload: Some(Box::new(value)),
    })
}

fn io_error_value(operation: &str, path: &str, error: &io::Error) -> Value {
    Value::Record(RecordValue {
        type_name: crate::std_package::IO_ERROR_MANGLED_NAME.to_string(),
        fields: vec![
            RecordFieldValue {
                name: "operation".to_string(),
                value: Value::String(operation.to_string()),
            },
            RecordFieldValue {
                name: "path".to_string(),
                value: Value::String(path.to_string()),
            },
            RecordFieldValue {
                name: "kind".to_string(),
                value: Value::String(format!("{:?}", error.kind())),
            },
            RecordFieldValue {
                name: "message".to_string(),
                value: Value::String(error.to_string()),
            },
            RecordFieldValue {
                name: "raw_code".to_string(),
                value: error
                    .raw_os_error()
                    .map(|code| option_some(Value::Int(i64::from(code))))
                    .unwrap_or_else(option_none),
            },
        ],
    })
}

fn path_buf_into_string(path: PathBuf, error_message: &'static str) -> io::Result<String> {
    path.into_os_string()
        .into_string()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, error_message))
}

fn read_dir_recursive_paths(root_path: &str) -> Result<Vec<String>, (String, io::Error)> {
    let mut paths = Vec::new();
    collect_read_dir_recursive_paths(root_path, &mut paths)?;
    Ok(paths)
}

fn collect_read_dir_recursive_paths(
    dir_path: &str,
    paths: &mut Vec<String>,
) -> Result<(), (String, io::Error)> {
    let entries = fs::read_dir(dir_path).map_err(|error| (dir_path.to_string(), error))?;
    let mut children = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| (dir_path.to_string(), error))?;
        let child_path =
            path_buf_into_string(entry.path(), "directory entry path is not valid Unicode")
                .map_err(|error| (dir_path.to_string(), error))?;
        let is_dir = entry
            .file_type()
            .map_err(|error| (child_path.clone(), error))?
            .is_dir();
        children.push((child_path, is_dir));
    }
    children.sort_by(|left, right| left.0.cmp(&right.0));
    for (child_path, is_dir) in children {
        paths.push(child_path.clone());
        if is_dir {
            collect_read_dir_recursive_paths(&child_path, paths)?;
        }
    }
    Ok(())
}

#[derive(Debug, Default)]
struct DirectorySizeMetadataRaw {
    size: i64,
    file_count: i64,
    directory_count: i64,
    other_count: i64,
}

#[derive(Debug)]
enum DirectorySizeEntryKind {
    File { size: i64 },
    Directory,
    Other,
}

#[derive(Debug)]
struct CopyDirEntry {
    from_path: PathBuf,
    to_path: PathBuf,
    from_text: String,
    to_text: String,
    kind: CopyDirEntryKind,
}

#[derive(Debug)]
enum CopyDirEntryKind {
    File,
    Directory,
    Other,
}

fn read_directory_size_metadata(
    root_path: &str,
) -> Result<DirectorySizeMetadataRaw, (String, io::Error)> {
    let mut metadata = DirectorySizeMetadataRaw::default();
    collect_directory_size_metadata(root_path, &mut metadata)?;
    Ok(metadata)
}

fn collect_directory_size_metadata(
    dir_path: &str,
    metadata: &mut DirectorySizeMetadataRaw,
) -> Result<(), (String, io::Error)> {
    let entries = fs::read_dir(dir_path).map_err(|error| (dir_path.to_string(), error))?;
    let mut children = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| (dir_path.to_string(), error))?;
        let child_path =
            path_buf_into_string(entry.path(), "directory entry path is not valid Unicode")
                .map_err(|error| (dir_path.to_string(), error))?;
        let file_type = entry
            .file_type()
            .map_err(|error| (child_path.clone(), error))?;
        let kind = if file_type.is_dir() {
            DirectorySizeEntryKind::Directory
        } else if file_type.is_file() {
            let size = entry
                .metadata()
                .and_then(|metadata| {
                    if !metadata.is_file() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "directory entry is not a file",
                        ));
                    }
                    i64::try_from(metadata.len()).map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidData, "file size does not fit in Int")
                    })
                })
                .map_err(|error| (child_path.clone(), error))?;
            DirectorySizeEntryKind::File { size }
        } else {
            DirectorySizeEntryKind::Other
        };
        children.push((child_path, kind));
    }
    children.sort_by(|left, right| left.0.cmp(&right.0));

    for (child_path, kind) in children {
        match kind {
            DirectorySizeEntryKind::File { size } => {
                add_directory_size_value(
                    &mut metadata.size,
                    size,
                    &child_path,
                    "directory size does not fit in Int",
                )?;
                increment_directory_size_count(
                    &mut metadata.file_count,
                    &child_path,
                    "file count does not fit in Int",
                )?;
            }
            DirectorySizeEntryKind::Directory => {
                increment_directory_size_count(
                    &mut metadata.directory_count,
                    &child_path,
                    "directory count does not fit in Int",
                )?;
                collect_directory_size_metadata(&child_path, metadata)?;
            }
            DirectorySizeEntryKind::Other => {
                increment_directory_size_count(
                    &mut metadata.other_count,
                    &child_path,
                    "other entry count does not fit in Int",
                )?;
            }
        }
    }
    Ok(())
}

fn increment_directory_size_count(
    count: &mut i64,
    path: &str,
    error_message: &'static str,
) -> Result<(), (String, io::Error)> {
    add_directory_size_value(count, 1, path, error_message)
}

fn add_directory_size_value(
    value: &mut i64,
    increment: i64,
    path: &str,
    error_message: &'static str,
) -> Result<(), (String, io::Error)> {
    let Some(next) = value.checked_add(increment) else {
        return Err((
            path.to_string(),
            io::Error::new(io::ErrorKind::InvalidData, error_message),
        ));
    };
    *value = next;
    Ok(())
}

fn directory_size_metadata_value(metadata: DirectorySizeMetadataRaw) -> Value {
    Value::Record(RecordValue {
        type_name: crate::std_package::FS_DIRECTORY_SIZE_METADATA_MANGLED_NAME.to_string(),
        fields: vec![
            RecordFieldValue {
                name: "size".to_string(),
                value: Value::Int(metadata.size),
            },
            RecordFieldValue {
                name: "file_count".to_string(),
                value: Value::Int(metadata.file_count),
            },
            RecordFieldValue {
                name: "directory_count".to_string(),
                value: Value::Int(metadata.directory_count),
            },
            RecordFieldValue {
                name: "other_count".to_string(),
                value: Value::Int(metadata.other_count),
            },
        ],
    })
}

fn copy_dir_all_paths(from_path: &str, to_path: &str) -> Result<(), (String, String, io::Error)> {
    reject_copy_dir_target_inside_source(from_path, to_path)?;
    copy_dir_all_paths_after_target_check(from_path, to_path)
}

fn move_dir_all_paths(from_path: &str, to_path: &str) -> Result<(), (String, String, io::Error)> {
    reject_move_dir_target_inside_source(from_path, to_path)?;
    copy_dir_all_paths_after_target_check(from_path, to_path)?;
    fs::remove_dir_all(from_path)
        .map_err(|error| (from_path.to_string(), to_path.to_string(), error))
}

fn copy_dir_all_paths_after_target_check(
    from_path: &str,
    to_path: &str,
) -> Result<(), (String, String, io::Error)> {
    let from_root = Path::new(from_path);
    let to_root = Path::new(to_path);
    let entries = read_copy_dir_entries(from_root, to_root, from_path, to_path)?;
    fs::create_dir(to_root).map_err(|error| (from_path.to_string(), to_path.to_string(), error))?;
    copy_dir_entries(entries)
}

fn reject_copy_dir_target_inside_source(
    from_path: &str,
    to_path: &str,
) -> Result<(), (String, String, io::Error)> {
    reject_dir_target_inside_source(
        from_path,
        to_path,
        "directory copy destination must not be the source or inside the source",
    )
}

fn reject_move_dir_target_inside_source(
    from_path: &str,
    to_path: &str,
) -> Result<(), (String, String, io::Error)> {
    reject_dir_target_inside_source(
        from_path,
        to_path,
        "directory move destination must not be the source or inside the source",
    )
}

fn reject_dir_target_inside_source(
    from_path: &str,
    to_path: &str,
    error_message: &'static str,
) -> Result<(), (String, String, io::Error)> {
    let normalized_from = normalize_path_lexically_for_std(from_path);
    let normalized_to = normalize_path_lexically_for_std(to_path);
    let normalized_from_path = Path::new(&normalized_from);
    let normalized_to_path = Path::new(&normalized_to);
    if normalized_to_path == normalized_from_path
        || normalized_to_path.starts_with(normalized_from_path)
    {
        return Err((
            from_path.to_string(),
            to_path.to_string(),
            io::Error::new(io::ErrorKind::InvalidInput, error_message),
        ));
    }

    let canonical_from = fs::canonicalize(from_path)
        .map_err(|error| (from_path.to_string(), to_path.to_string(), error))?;
    let to = Path::new(to_path);
    let to_parent = match to.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    let canonical_to_parent = fs::canonicalize(to_parent)
        .map_err(|error| (from_path.to_string(), to_path.to_string(), error))?;
    let canonical_to = to
        .file_name()
        .map(|file_name| canonical_to_parent.join(file_name))
        .unwrap_or(canonical_to_parent);
    if canonical_to == canonical_from || canonical_to.starts_with(&canonical_from) {
        return Err((
            from_path.to_string(),
            to_path.to_string(),
            io::Error::new(io::ErrorKind::InvalidInput, error_message),
        ));
    }

    Ok(())
}

fn read_copy_dir_entries(
    from_dir: &Path,
    to_dir: &Path,
    from_dir_text: &str,
    to_dir_text: &str,
) -> Result<Vec<CopyDirEntry>, (String, String, io::Error)> {
    let entries = fs::read_dir(from_dir)
        .map_err(|error| (from_dir_text.to_string(), to_dir_text.to_string(), error))?;
    let mut children = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|error| (from_dir_text.to_string(), to_dir_text.to_string(), error))?;
        let from_child_path = entry.path();
        let to_child_path = to_dir.join(entry.file_name());
        let from_child_text = path_buf_into_string(
            from_child_path.clone(),
            "directory entry path is not valid Unicode",
        )
        .map_err(|error| (from_dir_text.to_string(), to_dir_text.to_string(), error))?;
        let to_child_text = path_buf_into_string(
            to_child_path.clone(),
            "target directory entry path is not valid Unicode",
        )
        .map_err(|error| (from_child_text.clone(), to_dir_text.to_string(), error))?;
        let file_type = entry
            .file_type()
            .map_err(|error| (from_child_text.clone(), to_child_text.clone(), error))?;
        let kind = if file_type.is_dir() {
            CopyDirEntryKind::Directory
        } else if file_type.is_file() {
            CopyDirEntryKind::File
        } else {
            CopyDirEntryKind::Other
        };
        children.push(CopyDirEntry {
            from_path: from_child_path,
            to_path: to_child_path,
            from_text: from_child_text,
            to_text: to_child_text,
            kind,
        });
    }
    children.sort_by(|left, right| left.from_text.cmp(&right.from_text));
    Ok(children)
}

fn copy_dir_entries(entries: Vec<CopyDirEntry>) -> Result<(), (String, String, io::Error)> {
    for entry in entries {
        match entry.kind {
            CopyDirEntryKind::File => {
                fs::copy(&entry.from_path, &entry.to_path)
                    .map(|_| ())
                    .map_err(|error| (entry.from_text.clone(), entry.to_text.clone(), error))?;
            }
            CopyDirEntryKind::Directory => {
                let child_entries = read_copy_dir_entries(
                    &entry.from_path,
                    &entry.to_path,
                    &entry.from_text,
                    &entry.to_text,
                )?;
                fs::create_dir(&entry.to_path)
                    .map_err(|error| (entry.from_text.clone(), entry.to_text.clone(), error))?;
                copy_dir_entries(child_entries)?;
            }
            CopyDirEntryKind::Other => {
                return Err((
                    entry.from_text,
                    entry.to_text,
                    io::Error::new(
                        io::ErrorKind::Unsupported,
                        "directory copy supports only regular files and directories",
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn normalize_path_lexically_for_std(path: &str) -> String {
    let mut normalized = PathBuf::new();

    for component in Path::new(path).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => match normalized.components().next_back() {
                Some(Component::Normal(_)) => {
                    normalized.pop();
                }
                Some(Component::RootDir | Component::Prefix(_)) => {}
                Some(Component::ParentDir) | Some(Component::CurDir) | None => {
                    normalized.push("..");
                }
            },
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::Normal(part) => normalized.push(part),
        }
    }

    if normalized.as_os_str().is_empty() && !path.is_empty() {
        ".".to_string()
    } else {
        normalized.to_string_lossy().into_owned()
    }
}

fn path_pair_error_value(
    operation: &str,
    from_path: &str,
    to_path: &str,
    error: &io::Error,
) -> Value {
    Value::Record(RecordValue {
        type_name: crate::std_package::PATH_PAIR_ERROR_MANGLED_NAME.to_string(),
        fields: vec![
            RecordFieldValue {
                name: "operation".to_string(),
                value: Value::String(operation.to_string()),
            },
            RecordFieldValue {
                name: "from_path".to_string(),
                value: Value::String(from_path.to_string()),
            },
            RecordFieldValue {
                name: "to_path".to_string(),
                value: Value::String(to_path.to_string()),
            },
            RecordFieldValue {
                name: "kind".to_string(),
                value: Value::String(format!("{:?}", error.kind())),
            },
            RecordFieldValue {
                name: "message".to_string(),
                value: Value::String(error.to_string()),
            },
            RecordFieldValue {
                name: "raw_code".to_string(),
                value: error
                    .raw_os_error()
                    .map(|code| option_some(Value::Int(i64::from(code))))
                    .unwrap_or_else(option_none),
            },
        ],
    })
}

fn resource_display_path(package_path: &str, resource_path: &str) -> String {
    format!("{package_path}:{resource_path}")
}

fn read_package_resource_text(
    package_resource_roots: &HashMap<String, PathBuf>,
    package_path: &str,
    resource_path: &str,
) -> io::Result<String> {
    let bytes = read_package_resource_bytes(package_resource_roots, package_path, resource_path)?;
    String::from_utf8(bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn read_package_resource_bytes(
    package_resource_roots: &HashMap<String, PathBuf>,
    package_path: &str,
    resource_path: &str,
) -> io::Result<Vec<u8>> {
    validate_runtime_resource_path(resource_path)?;
    let Some(resource_root) = package_resource_roots.get(package_path) else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("package `{package_path}` does not declare runtime resources"),
        ));
    };

    let canonical_root = resource_root.canonicalize()?;
    let mut candidate = canonical_root.clone();
    for segment in resource_path.split('/') {
        candidate.push(segment);
    }
    let canonical_candidate = candidate.canonicalize()?;
    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "resource path escapes the package resource root",
        ));
    }

    fs::read(canonical_candidate)
}

fn validate_runtime_resource_path(resource_path: &str) -> io::Result<()> {
    if resource_path.is_empty() {
        return Err(invalid_runtime_resource_path_error(
            "resource path must not be empty",
        ));
    }
    if Path::new(resource_path).is_absolute()
        || resource_path.contains('\\')
        || resource_path.contains(':')
    {
        return Err(invalid_runtime_resource_path_error(
            "resource path must be a relative slash-separated path",
        ));
    }
    for segment in resource_path.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(invalid_runtime_resource_path_error(
                "resource path must stay inside the package resource root",
            ));
        }
        if matches!(segment, ".git" | ".muga") {
            return Err(invalid_runtime_resource_path_error(
                "resource path must not use tool metadata directories",
            ));
        }
    }
    Ok(())
}

fn invalid_runtime_resource_path_error(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn path_value(path: String) -> Value {
    Value::Record(RecordValue {
        type_name: crate::std_package::PATH_MANGLED_NAME.to_string(),
        fields: vec![RecordFieldValue {
            name: "text".to_string(),
            value: Value::String(path),
        }],
    })
}

const JSON_NESTING_LIMIT: usize = 128;

#[derive(Clone, Copy, Debug)]
enum JsonErrorKind {
    UnexpectedEnd,
    UnexpectedToken,
    InvalidEscape,
    InvalidNumber,
    NumberOutOfRange,
    DuplicateKey,
    TrailingCharacters,
    NestingLimitExceeded,
    Validation,
}

#[derive(Clone, Copy, Debug)]
enum ConfigErrorKind {
    Read,
    Parse,
    Decode,
}

#[derive(Clone, Copy, Debug)]
enum CliErrorKind {
    UnknownArgument,
    MissingArgument,
    MissingValue,
    InvalidValue,
    Validation,
    UnsupportedTarget,
}

impl ConfigErrorKind {
    fn variant_name(self) -> &'static str {
        match self {
            Self::Read => "Read",
            Self::Parse => "Parse",
            Self::Decode => "Decode",
        }
    }
}

impl JsonErrorKind {
    fn variant_name(self) -> &'static str {
        match self {
            Self::UnexpectedEnd => "UnexpectedEnd",
            Self::UnexpectedToken => "UnexpectedToken",
            Self::InvalidEscape => "InvalidEscape",
            Self::InvalidNumber => "InvalidNumber",
            Self::NumberOutOfRange => "NumberOutOfRange",
            Self::DuplicateKey => "DuplicateKey",
            Self::TrailingCharacters => "TrailingCharacters",
            Self::NestingLimitExceeded => "NestingLimitExceeded",
            Self::Validation => "Validation",
        }
    }
}

impl CliErrorKind {
    fn variant_name(self) -> &'static str {
        match self {
            Self::UnknownArgument => "UnknownArgument",
            Self::MissingArgument => "MissingArgument",
            Self::MissingValue => "MissingValue",
            Self::InvalidValue => "InvalidValue",
            Self::Validation => "Validation",
            Self::UnsupportedTarget => "UnsupportedTarget",
        }
    }
}

#[derive(Clone, Debug)]
struct JsonDataError {
    kind: JsonErrorKind,
    message: String,
    offset: i64,
}

#[derive(Clone, Debug)]
struct CliInputError {
    kind: CliErrorKind,
    argument: String,
    message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JsonNumberShape {
    Integral,
    NonIntegral,
}

fn json_value_variant(variant_name: &str, payload: Option<Value>) -> Value {
    Value::Enum(EnumValue {
        type_name: crate::std_package::JSON_VALUE_MANGLED_NAME.to_string(),
        variant_name: variant_name.to_string(),
        payload: payload.map(Box::new),
    })
}

fn json_number_variant(variant_name: &str, payload: Value) -> Value {
    Value::Enum(EnumValue {
        type_name: crate::std_package::JSON_NUMBER_MANGLED_NAME.to_string(),
        variant_name: variant_name.to_string(),
        payload: Some(Box::new(payload)),
    })
}

fn json_error_kind_value(kind: JsonErrorKind) -> Value {
    Value::Enum(EnumValue {
        type_name: crate::std_package::JSON_ERROR_KIND_MANGLED_NAME.to_string(),
        variant_name: kind.variant_name().to_string(),
        payload: None,
    })
}

fn json_error_value(error: JsonDataError) -> Value {
    Value::Record(RecordValue {
        type_name: crate::std_package::JSON_ERROR_MANGLED_NAME.to_string(),
        fields: vec![
            RecordFieldValue {
                name: "kind".to_string(),
                value: json_error_kind_value(error.kind),
            },
            RecordFieldValue {
                name: "message".to_string(),
                value: Value::String(error.message),
            },
            RecordFieldValue {
                name: "offset".to_string(),
                value: Value::Int(error.offset),
            },
        ],
    })
}

fn config_error_kind_value(kind: ConfigErrorKind) -> Value {
    Value::Enum(EnumValue {
        type_name: crate::std_package::CONFIG_ERROR_KIND_MANGLED_NAME.to_string(),
        variant_name: kind.variant_name().to_string(),
        payload: None,
    })
}

fn config_error_value(
    kind: ConfigErrorKind,
    path: &str,
    message: impl Into<String>,
    offset: i64,
    raw_code: Option<i32>,
) -> Value {
    Value::Record(RecordValue {
        type_name: crate::std_package::CONFIG_ERROR_MANGLED_NAME.to_string(),
        fields: vec![
            RecordFieldValue {
                name: "kind".to_string(),
                value: config_error_kind_value(kind),
            },
            RecordFieldValue {
                name: "path".to_string(),
                value: path_value(path.to_string()),
            },
            RecordFieldValue {
                name: "message".to_string(),
                value: Value::String(message.into()),
            },
            RecordFieldValue {
                name: "offset".to_string(),
                value: Value::Int(offset),
            },
            RecordFieldValue {
                name: "raw_code".to_string(),
                value: raw_code
                    .map(|code| option_some(Value::Int(i64::from(code))))
                    .unwrap_or_else(option_none),
            },
        ],
    })
}

fn cli_input_error(
    kind: CliErrorKind,
    argument: impl Into<String>,
    message: impl Into<String>,
) -> CliInputError {
    CliInputError {
        kind,
        argument: argument.into(),
        message: message.into(),
    }
}

fn cli_error_kind_value(kind: CliErrorKind) -> Value {
    Value::Enum(EnumValue {
        type_name: crate::std_package::CLI_ERROR_KIND_MANGLED_NAME.to_string(),
        variant_name: kind.variant_name().to_string(),
        payload: None,
    })
}

fn cli_error_value(error: CliInputError) -> Value {
    Value::Record(RecordValue {
        type_name: crate::std_package::CLI_ERROR_MANGLED_NAME.to_string(),
        fields: vec![
            RecordFieldValue {
                name: "kind".to_string(),
                value: cli_error_kind_value(error.kind),
            },
            RecordFieldValue {
                name: "argument".to_string(),
                value: Value::String(error.argument),
            },
            RecordFieldValue {
                name: "message".to_string(),
                value: Value::String(error.message),
            },
        ],
    })
}

fn json_error(kind: JsonErrorKind, message: impl Into<String>, offset: usize) -> JsonDataError {
    JsonDataError {
        kind,
        message: message.into(),
        offset: i64::try_from(offset).unwrap_or(i64::MAX),
    }
}

fn json_value_error(kind: JsonErrorKind, message: impl Into<String>) -> JsonDataError {
    JsonDataError {
        kind,
        message: message.into(),
        offset: -1,
    }
}

fn make_enum_value(
    program: &Program,
    enum_name: Symbol,
    variant_name: Symbol,
    payload: Option<Value>,
) -> Value {
    Value::Enum(EnumValue {
        type_name: program.symbols.resolve(enum_name).to_string(),
        variant_name: program.symbols.resolve(variant_name).to_string(),
        payload: payload.map(Box::new),
    })
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "{value}"),
            Self::Bytes(bytes) => write!(f, "<bytes:{}>", bytes.len()),
            Self::Unit => write!(f, "()"),
            Self::List(items) => {
                write!(f, "[")?;
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Self::Map(map) => {
                write!(f, "Map {{")?;
                for (index, entry) in map.entries.iter().enumerate() {
                    if index == 0 {
                        write!(f, " ")?;
                    } else {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", entry.key, entry.value)?;
                }
                if !map.entries.is_empty() {
                    write!(f, " ")?;
                }
                write!(f, "}}")
            }
            Self::Enum(value) => {
                write!(f, "{}::{}", value.type_name, value.variant_name)?;
                if let Some(payload) = &value.payload {
                    write!(f, "({payload})")?;
                }
                Ok(())
            }
            Self::Record(record) => {
                write!(f, "{} {{ ", record.type_name)?;
                for (index, field) in record.fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field.name, field.value)?;
                }
                write!(f, " }}")
            }
            Self::RuntimeHandle(handle) => write!(f, "<opaque:{}>", handle.family),
            Self::Function(_) => write!(f, "<function>"),
            Self::Builtin(builtin) => write!(f, "<builtin:{}>", prelude::builtin_name(*builtin)),
        }
    }
}

impl Value {
    pub fn is_unit(&self) -> bool {
        matches!(self, Self::Unit)
    }

    pub fn result_unit_status(&self) -> Option<Result<(), String>> {
        let Self::Enum(value) = self else {
            return None;
        };
        if value.type_name != known_enum::RESULT_NAME {
            return None;
        }
        match value.variant_name.as_str() {
            known_enum::RESULT_OK_NAME => match value.payload.as_deref() {
                Some(Self::Unit) => Some(Ok(())),
                Some(payload) => Some(Err(format!(
                    "Result::Ok payload must be Unit for a test, got {payload}"
                ))),
                None => Some(Err(
                    "Result::Ok must carry Unit for a test, got no payload".to_string()
                )),
            },
            known_enum::RESULT_ERR_NAME => Some(Err(value.payload.as_deref().map_or_else(
                || "Result::Err returned without an error payload".to_string(),
                |payload| payload.to_string(),
            ))),
            _ => None,
        }
    }
}

impl fmt::Display for MapKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "\"{value}\""),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RunOutcome {
    pub main_result: Option<Value>,
    pub output_text: String,
    pub stderr_text: String,
    pub runtime_diagnostics: Vec<Diagnostic>,
}

pub fn run(program: &Program) -> Result<RunOutcome, Vec<Diagnostic>> {
    run_with_args(program, &[])
}

pub fn run_with_args(
    program: &Program,
    program_args: &[String],
) -> Result<RunOutcome, Vec<Diagnostic>> {
    run_with_args_and_package_resources(program, program_args, &[])
}

pub fn run_with_args_and_package_resources(
    program: &Program,
    program_args: &[String],
    package_resource_roots: &[(String, PathBuf)],
) -> Result<RunOutcome, Vec<Diagnostic>> {
    let root = execute_entry_with_args_and_package_resources(
        program,
        program_args,
        package_resource_roots,
    )?;
    match program.main {
        Some(main) => {
            call_zero_arg_function_by_ref(program, &root, main.local, "`main`", main.name)
        }
        None => Ok(run_outcome_from_env(&root, None)),
    }
}

pub fn run_function_with_args(
    program: &Program,
    function_name: &str,
    program_args: &[String],
) -> Result<RunOutcome, Vec<Diagnostic>> {
    let root = execute_entry_with_args(program, program_args)?;
    let Some(function_local) = program.locals.iter().find(|local| {
        matches!(local.kind, LocalKind::Binding(BindingKind::Function))
            && symbol_name(program, local.name) == function_name
    }) else {
        return Err(vec![Diagnostic::new(
            "R016",
            format!("test function `{function_name}` was not found at runtime"),
            Span::default(),
        )]);
    };

    call_zero_arg_function_by_ref(
        program,
        &root,
        function_local.id,
        function_name,
        function_local.name,
    )
}

pub fn run_package_function_with_args(
    program: &Program,
    package_item: PackageItemId,
    display_name: &str,
    program_args: &[String],
) -> Result<RunOutcome, Vec<Diagnostic>> {
    run_package_function_with_args_and_package_resources(
        program,
        package_item,
        display_name,
        program_args,
        &[],
    )
}

pub fn run_package_function_with_args_and_package_resources(
    program: &Program,
    package_item: PackageItemId,
    display_name: &str,
    program_args: &[String],
    package_resource_roots: &[(String, PathBuf)],
) -> Result<RunOutcome, Vec<Diagnostic>> {
    let root = execute_entry_with_args_and_package_resources(
        program,
        program_args,
        package_resource_roots,
    )?;
    let Some(function_local) = program.locals.iter().find(|local| {
        matches!(local.kind, LocalKind::Binding(BindingKind::Function))
            && local.package_item == Some(package_item)
    }) else {
        return Err(vec![Diagnostic::new(
            "R016",
            format!("test function `{display_name}` was not found at runtime"),
            Span::default(),
        )]);
    };

    call_zero_arg_function_by_ref(
        program,
        &root,
        function_local.id,
        display_name,
        function_local.name,
    )
}

fn execute_entry_with_args(
    program: &Program,
    program_args: &[String],
) -> Result<EnvRef, Vec<Diagnostic>> {
    execute_entry_with_args_and_package_resources(program, program_args, &[])
}

fn execute_entry_with_args_and_package_resources(
    program: &Program,
    program_args: &[String],
    package_resource_roots: &[(String, PathBuf)],
) -> Result<EnvRef, Vec<Diagnostic>> {
    let output = Rc::new(RefCell::new(String::new()));
    let stderr = Rc::new(RefCell::new(String::new()));
    let program_args = Rc::new(program_args.to_vec());
    let runtime_diagnostics = Rc::new(RefCell::new(Vec::new()));
    let call_stack = Rc::new(RefCell::new(Vec::new()));
    let runtime_handles = Rc::new(RefCell::new(RuntimeHandles::default()));
    let package_resource_roots = Rc::new(package_resource_roots.iter().cloned().collect());
    let context = RuntimeContext {
        output: output.clone(),
        stderr: stderr.clone(),
        program_args,
        runtime_diagnostics: runtime_diagnostics.clone(),
        call_stack,
        runtime_handles,
        package_resource_roots,
    };
    let root = Rc::new(RefCell::new(Env::new(
        None,
        true,
        context,
        program.local_count,
    )));
    install_prelude(program, &root);
    let _ = execute_chunk(program, &program.entry, root.clone())?;
    Ok(root)
}

fn call_zero_arg_function_by_ref(
    program: &Program,
    root: &EnvRef,
    local: LocalId,
    display_name: &str,
    name: Symbol,
) -> Result<RunOutcome, Vec<Diagnostic>> {
    match lookup_any(root, local) {
        None => Ok(run_outcome_from_env(root, None)),
        Some(Binding {
            value: Value::Function(function),
            ..
        }) => {
            let definition = function.definition(program);
            if !definition.params.is_empty() {
                return Err(vec![Diagnostic::new(
                    "R001",
                    format!(
                        "{display_name} must be a zero-argument function to be used as an entrypoint"
                    ),
                    definition.span,
                )]);
            }
            let entry_label = format!("while running {}", quoted_runtime_name(display_name));
            push_runtime_call_frame(root, entry_label.clone(), definition.span);
            let result = call_function(program, &function, Vec::new());
            pop_runtime_call_frame(root);
            let value = result.map_err(|diagnostics| {
                add_runtime_related_note(diagnostics, entry_label, definition.span)
            })?;
            Ok(run_outcome_from_env(root, Some(value)))
        }
        Some(binding) => Err(vec![Diagnostic::new(
            "R002",
            format!("`{}` must be a function", symbol_name(program, name)),
            binding.span,
        )]),
    }
}

fn run_outcome_from_env(root: &EnvRef, main_result: Option<Value>) -> RunOutcome {
    let borrowed = root.borrow();
    RunOutcome {
        main_result,
        output_text: borrowed.output.borrow().clone(),
        stderr_text: borrowed.stderr.borrow().clone(),
        runtime_diagnostics: borrowed.runtime_diagnostics.borrow().clone(),
    }
}

fn symbol_name(program: &Program, symbol: Symbol) -> &str {
    program.symbols.resolve(symbol)
}

#[derive(Clone, Debug)]
pub struct ClosureValue {
    function: FunctionId,
    env: EnvRef,
}

impl ClosureValue {
    fn definition<'a>(&self, program: &'a Program) -> &'a Function {
        &program.functions[self.function]
    }
}

#[derive(Clone, Debug)]
struct Binding {
    mutable: bool,
    value: Value,
    span: Span,
}

#[derive(Clone, Debug)]
struct RuntimeCallFrame {
    message: String,
    span: Span,
}

#[derive(Debug)]
struct RuntimeContext {
    output: OutputRef,
    stderr: OutputRef,
    program_args: Rc<Vec<String>>,
    runtime_diagnostics: RuntimeDiagnosticsRef,
    call_stack: RuntimeCallStackRef,
    runtime_handles: RuntimeHandlesRef,
    package_resource_roots: PackageResourceRootsRef,
}

#[derive(Debug)]
struct Env {
    bindings: Vec<Option<Binding>>,
    parent: Option<EnvRef>,
    function_boundary: bool,
    output: OutputRef,
    stderr: OutputRef,
    program_args: Rc<Vec<String>>,
    runtime_diagnostics: RuntimeDiagnosticsRef,
    call_stack: RuntimeCallStackRef,
    runtime_handles: RuntimeHandlesRef,
    package_resource_roots: PackageResourceRootsRef,
}

impl Env {
    fn new(
        parent: Option<EnvRef>,
        function_boundary: bool,
        context: RuntimeContext,
        local_count: usize,
    ) -> Self {
        Self {
            bindings: vec![None; local_count],
            parent,
            function_boundary,
            output: context.output,
            stderr: context.stderr,
            program_args: context.program_args,
            runtime_diagnostics: context.runtime_diagnostics,
            call_stack: context.call_stack,
            runtime_handles: context.runtime_handles,
            package_resource_roots: context.package_resource_roots,
        }
    }

    fn binding(&self, local: LocalId) -> Option<&Binding> {
        self.bindings
            .get(local.as_u32() as usize)
            .and_then(Option::as_ref)
    }

    fn binding_mut(&mut self, local: LocalId) -> Option<&mut Binding> {
        self.bindings
            .get_mut(local.as_u32() as usize)
            .and_then(Option::as_mut)
    }

    fn contains(&self, local: LocalId) -> bool {
        self.binding(local).is_some()
    }

    fn insert(&mut self, local: LocalId, binding: Binding) {
        let slot = self
            .bindings
            .get_mut(local.as_u32() as usize)
            .expect("bytecode local should be allocated in the program frame");
        *slot = Some(binding);
    }
}

fn execute_chunk(
    program: &Program,
    chunk: &Chunk,
    env: EnvRef,
) -> Result<Option<Value>, Vec<Diagnostic>> {
    let mut stack = Vec::<Value>::new();
    let mut current_env = env;
    let mut pc = 0usize;

    while let Some(instruction) = chunk.instructions.get(pc) {
        match instruction {
            Instruction::LoadInt(value) => stack.push(Value::Int(*value)),
            Instruction::LoadBool(value) => stack.push(Value::Bool(*value)),
            Instruction::LoadString(value) => stack.push(Value::String(value.clone())),
            Instruction::LoadUnit => stack.push(Value::Unit),
            Instruction::MakeRecord {
                type_name,
                fields,
                span,
            } => {
                let values = pop_args(&mut stack, fields.len(), *span)?;
                stack.push(make_record_value(program, *type_name, fields, values));
            }
            Instruction::MakeEnum {
                enum_name,
                variant_name,
                has_payload,
                span,
            } => {
                let payload = if *has_payload {
                    Some(pop_value(
                        &mut stack,
                        *span,
                        "R015",
                        "missing enum variant payload",
                    )?)
                } else {
                    None
                };
                stack.push(make_enum_value(program, *enum_name, *variant_name, payload));
            }
            Instruction::MakeList { len, span } => {
                let values = pop_args(&mut stack, *len, *span)?;
                stack.push(Value::List(values));
            }
            Instruction::LoadName { target, span } => {
                let Some(binding) = lookup_any(&current_env, target.local) else {
                    return Err(vec![Diagnostic::new(
                        "R008",
                        format!(
                            "unresolved runtime name `{}`",
                            symbol_name(program, target.name)
                        ),
                        *span,
                    )]);
                };
                stack.push(binding.value);
            }
            Instruction::LoadField { field, span } => {
                let base = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing record value for field access",
                )?;
                let value = load_record_field(program, base, *field, *span)?;
                stack.push(value);
            }
            Instruction::LoadIndex { span } => {
                let index = pop_value(&mut stack, *span, "R015", "missing list index")?;
                let base = pop_value(&mut stack, *span, "R015", "missing list value")?;
                let value = load_list_index(base, index, *span)?;
                stack.push(value);
            }
            Instruction::ListLen { span } => {
                let value = pop_value(&mut stack, *span, "R015", "missing list value")?;
                let Value::List(items) = value else {
                    return Err(vec![Diagnostic::new(
                        "R014",
                        "list length expects List[T]",
                        *span,
                    )]);
                };
                stack.push(Value::Int(items.len() as i64));
            }
            Instruction::UpdateRecord { fields, span } => {
                let values = pop_args(&mut stack, fields.len(), *span)?;
                let base = pop_value(&mut stack, *span, "R015", "missing record value for update")?;
                let value = update_record_value(program, base, fields, values, *span)?;
                stack.push(value);
            }
            Instruction::Assign {
                target,
                mutable,
                is_update,
                span,
            } => {
                let value = pop_value(&mut stack, *span, "R015", "missing value for assignment")?;
                execute_assign(
                    program,
                    &current_env,
                    *target,
                    *mutable,
                    *is_update,
                    value,
                    *span,
                )?;
            }
            Instruction::DefineFunction {
                target,
                function,
                span,
            } => {
                current_env.borrow_mut().insert(
                    target.local,
                    Binding {
                        mutable: false,
                        value: Value::Function(Rc::new(ClosureValue {
                            function: *function,
                            env: current_env.clone(),
                        })),
                        span: *span,
                    },
                );
            }
            Instruction::MakeClosure { function } => {
                stack.push(Value::Function(Rc::new(ClosureValue {
                    function: *function,
                    env: current_env.clone(),
                })));
            }
            Instruction::UnaryNeg { span } => {
                let value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing operand for unary operator",
                )?;
                match value {
                    Value::Int(value) => {
                        let Some(value) = value.checked_neg() else {
                            return Err(integer_overflow(*span));
                        };
                        stack.push(Value::Int(value));
                    }
                    _ => {
                        return Err(vec![Diagnostic::new(
                            "R009",
                            "invalid operand for unary operator",
                            *span,
                        )]);
                    }
                }
            }
            Instruction::UnaryNot { span } => {
                let value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing operand for unary operator",
                )?;
                match value {
                    Value::Bool(value) => stack.push(Value::Bool(!value)),
                    _ => {
                        return Err(vec![Diagnostic::new(
                            "R009",
                            "invalid operand for unary operator",
                            *span,
                        )]);
                    }
                }
            }
            Instruction::Binary { op, span } => {
                let right = pop_value(&mut stack, *span, "R015", "missing right operand")?;
                let left = pop_value(&mut stack, *span, "R015", "missing left operand")?;
                let value = eval_binary(*op, left, right, *span)?;
                stack.push(value);
            }
            Instruction::Call { argc, span } => {
                let args = pop_args(&mut stack, *argc, *span)?;
                let callee = pop_value(&mut stack, *span, "R015", "missing callee for call")?;
                let value = call_value(program, callee, args, &current_env, *span)?;
                stack.push(value);
            }
            Instruction::DecodeJson { schema, span } => {
                let fallback = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing fallback for JSON decode",
                )?;
                let value = pop_value(&mut stack, *span, "R015", "missing JSON value")?;
                let decoded = decode_json_value(program, schema, &value, &fallback, "");
                stack.push(match decoded {
                    Ok(value) => result_ok(value),
                    Err(error) => result_err(json_error_value(error)),
                });
            }
            Instruction::DecodeJsonRequired { schema, span } => {
                let value = pop_value(&mut stack, *span, "R015", "missing JSON value")?;
                let decoded = decode_json_value_required(program, schema, &value, "");
                stack.push(match decoded {
                    Ok(value) => result_ok(value),
                    Err(error) => result_err(json_error_value(error)),
                });
            }
            Instruction::JsonToValue { schema, span } => {
                let value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing value for typed JSON conversion",
                )?;
                let encoded = typed_value_to_json_value(program, schema, &value, "");
                stack.push(match encoded {
                    Ok(value) => result_ok(value),
                    Err(error) => result_err(json_error_value(error)),
                });
            }
            Instruction::JsonEncodeTyped { schema, span } => {
                let value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing value for typed JSON encoding",
                )?;
                let encoded = typed_value_to_json_value(program, schema, &value, "")
                    .and_then(|value| encode_json_value(&value, 0).map(Value::String));
                stack.push(match encoded {
                    Ok(value) => result_ok(value),
                    Err(error) => result_err(json_error_value(error)),
                });
            }
            Instruction::LoadJsonConfigRequired { schema, span } => {
                let path_value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing path for required JSON config load",
                )?;
                let path = expect_path_value(&path_value, *span, "config::load_json")?;
                let result = match fs::read_to_string(&path) {
                    Ok(text) => match JsonParser::new(&text).parse() {
                        Ok(json_value) => {
                            match decode_json_value_required(program, schema, &json_value, "") {
                                Ok(value) => result_ok(value),
                                Err(error) => result_err(config_error_value(
                                    ConfigErrorKind::Decode,
                                    &path,
                                    error.message,
                                    error.offset,
                                    None,
                                )),
                            }
                        }
                        Err(error) => result_err(config_error_value(
                            ConfigErrorKind::Parse,
                            &path,
                            error.message,
                            error.offset,
                            None,
                        )),
                    },
                    Err(error) => result_err(config_error_value(
                        ConfigErrorKind::Read,
                        &path,
                        error.to_string(),
                        -1,
                        error.raw_os_error(),
                    )),
                };
                stack.push(result);
            }
            Instruction::LoadJsonConfig { schema, span } => {
                let fallback = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing fallback for JSON config load",
                )?;
                let path_value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing path for JSON config load",
                )?;
                let path = expect_path_value(&path_value, *span, "config::load_json_or")?;
                let result = match fs::read_to_string(&path) {
                    Ok(text) => match JsonParser::new(&text).parse() {
                        Ok(json_value) => {
                            match decode_json_value(program, schema, &json_value, &fallback, "") {
                                Ok(value) => result_ok(value),
                                Err(error) => result_err(config_error_value(
                                    ConfigErrorKind::Decode,
                                    &path,
                                    error.message,
                                    error.offset,
                                    None,
                                )),
                            }
                        }
                        Err(error) => result_err(config_error_value(
                            ConfigErrorKind::Parse,
                            &path,
                            error.message,
                            error.offset,
                            None,
                        )),
                    },
                    Err(error) => result_err(config_error_value(
                        ConfigErrorKind::Read,
                        &path,
                        error.to_string(),
                        -1,
                        error.raw_os_error(),
                    )),
                };
                stack.push(result);
            }
            Instruction::CliParseOr { schema, span } => {
                let defaults =
                    pop_value(&mut stack, *span, "R015", "missing defaults for CLI parser")?;
                let args_value =
                    pop_value(&mut stack, *span, "R015", "missing args for CLI parser")?;
                let args = expect_string_list_value(&args_value, *span, "cli::parse_or")?;
                let parsed = cli_parse_or(program, schema, &args, &defaults);
                stack.push(match parsed {
                    Ok(value) => result_ok(value),
                    Err(error) => result_err(cli_error_value(error)),
                });
            }
            Instruction::CliParse { schema, span } => {
                let args_value =
                    pop_value(&mut stack, *span, "R015", "missing args for CLI parser")?;
                let args = expect_string_list_value(&args_value, *span, "cli::parse")?;
                let parsed = cli_parse(program, schema, &args);
                stack.push(match parsed {
                    Ok(value) => result_ok(value),
                    Err(error) => result_err(cli_error_value(error)),
                });
            }
            Instruction::CliParseRequest { schema, span } => {
                let program_value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing program name for CLI request parser",
                )?;
                let args_value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing args for CLI request parser",
                )?;
                let args = expect_string_list_value(&args_value, *span, "cli::parse_request")?;
                let program_name =
                    expect_string_value(&program_value, *span, "cli::parse_request")?;
                if cli_schema_is_command(schema) {
                    let request = cli_parse_command_request(program, schema, &args, &program_name);
                    stack.push(match request {
                        Ok(value) => result_ok(value),
                        Err(error) => result_err(cli_error_value(error)),
                    });
                } else if cli_schema_is_wrapper(schema) {
                    let request = cli_parse_wrapper_request(program, schema, &args, &program_name);
                    stack.push(match request {
                        Ok(value) => result_ok(value),
                        Err(error) => result_err(cli_error_value(error)),
                    });
                } else if cli_help_requested(&args) {
                    stack.push(result_ok(cli_request_help(cli_help_for_required(
                        program,
                        schema,
                        &program_name,
                    ))));
                } else {
                    let parsed = cli_parse(program, schema, &args);
                    stack.push(match parsed {
                        Ok(value) => result_ok(cli_request_parsed(value)),
                        Err(error) => result_err(cli_error_value(error)),
                    });
                }
            }
            Instruction::CliParseRequestOr { schema, span } => {
                let defaults = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing defaults for CLI request parser",
                )?;
                let program_value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing program name for CLI request parser",
                )?;
                let args_value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing args for CLI request parser",
                )?;
                let args = expect_string_list_value(&args_value, *span, "cli::parse_request_or")?;
                let program_name =
                    expect_string_value(&program_value, *span, "cli::parse_request_or")?;
                if cli_help_requested(&args) {
                    stack.push(result_ok(cli_request_help(cli_help_for(
                        program,
                        schema,
                        &program_name,
                        &defaults,
                    ))));
                } else {
                    let parsed = cli_parse_or(program, schema, &args, &defaults);
                    stack.push(match parsed {
                        Ok(value) => result_ok(cli_request_parsed(value)),
                        Err(error) => result_err(cli_error_value(error)),
                    });
                }
            }
            Instruction::CliUsageFor { schema, span } => {
                let defaults =
                    pop_value(&mut stack, *span, "R015", "missing defaults for CLI usage")?;
                let program_value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing program name for CLI usage",
                )?;
                let program_name = expect_string_value(&program_value, *span, "cli::usage_for")?;
                stack.push(Value::String(cli_usage_for(
                    program,
                    schema,
                    &program_name,
                    &defaults,
                )));
            }
            Instruction::CliUsageForRequired { schema, span } => {
                let program_value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing program name for strict CLI usage",
                )?;
                let program_name =
                    expect_string_value(&program_value, *span, "cli::usage_for_required")?;
                stack.push(Value::String(cli_usage_for_required(
                    program,
                    schema,
                    &program_name,
                )));
            }
            Instruction::CliHelpFor { schema, span } => {
                let defaults =
                    pop_value(&mut stack, *span, "R015", "missing defaults for CLI help")?;
                let program_value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing program name for CLI help",
                )?;
                let program_name = expect_string_value(&program_value, *span, "cli::help_for")?;
                stack.push(Value::String(cli_help_for(
                    program,
                    schema,
                    &program_name,
                    &defaults,
                )));
            }
            Instruction::CliHelpForRequired { schema, span } => {
                let program_value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing program name for strict CLI help",
                )?;
                let program_name =
                    expect_string_value(&program_value, *span, "cli::help_for_required")?;
                stack.push(Value::String(cli_help_for_required(
                    program,
                    schema,
                    &program_name,
                )));
            }
            Instruction::JumpIfFalse { target, span } => {
                let condition = pop_value(&mut stack, *span, "R015", "missing condition for jump")?;
                match condition {
                    Value::Bool(false) => {
                        pc = *target;
                        continue;
                    }
                    Value::Bool(true) => {}
                    _ => {
                        return Err(vec![Diagnostic::new(
                            "R003",
                            "`if`/`while` condition did not evaluate to Bool",
                            *span,
                        )]);
                    }
                }
            }
            Instruction::JumpIfNotEnumVariant {
                enum_name,
                variant_name,
                target,
                span,
            } => {
                let value = pop_value(&mut stack, *span, "R015", "missing enum value for match")?;
                let enum_name = program.symbols.resolve(*enum_name);
                let variant_name = program.symbols.resolve(*variant_name);
                match value {
                    Value::Enum(value)
                        if value.type_name == enum_name && value.variant_name == variant_name =>
                    {
                        if let Some(payload) = value.payload {
                            stack.push(*payload);
                        }
                    }
                    Value::Enum(value) if value.type_name == enum_name => {
                        pc = *target;
                        continue;
                    }
                    Value::Enum(value) => {
                        return Err(vec![Diagnostic::new(
                            "R019",
                            format!(
                                "`match` expected a {enum_name} value but found `{}::{}`",
                                value.type_name, value.variant_name
                            ),
                            *span,
                        )]);
                    }
                    _ => {
                        return Err(vec![Diagnostic::new(
                            "R019",
                            format!("`match` expected a {enum_name} value"),
                            *span,
                        )]);
                    }
                }
            }
            Instruction::MatchExhausted { enum_name, span } => {
                return Err(vec![Diagnostic::new(
                    "R019",
                    format!(
                        "`match` did not cover a {} variant",
                        program.symbols.resolve(*enum_name)
                    ),
                    *span,
                )]);
            }
            Instruction::Jump { target } => {
                pc = *target;
                continue;
            }
            Instruction::PushScope => {
                current_env = child_env(&current_env, false);
            }
            Instruction::PopScope => {
                let parent = current_env
                    .borrow()
                    .parent
                    .clone()
                    .expect("scope must have parent");
                current_env = parent;
            }
            Instruction::Pop => {
                let _ = pop_value(
                    &mut stack,
                    Span::default(),
                    "R015",
                    "missing value to discard",
                )?;
            }
            Instruction::Return => {
                let value = pop_value(
                    &mut stack,
                    Span::default(),
                    "R015",
                    "missing return value at end of function",
                )?;
                return Ok(Some(value));
            }
        }
        pc += 1;
    }

    Ok(None)
}

fn execute_assign(
    program: &Program,
    env: &EnvRef,
    target: NameRef,
    mutable: bool,
    is_update: bool,
    value: Value,
    span: Span,
) -> Result<(), Vec<Diagnostic>> {
    if is_update {
        return execute_update(program, env, target, value, span);
    }

    if env.borrow().contains(target.local) {
        return Err(vec![Diagnostic::new(
            "R004",
            format!(
                "duplicate binding `{}` in the current scope",
                symbol_name(program, target.name)
            ),
            span,
        )]);
    }

    env.borrow_mut().insert(
        target.local,
        Binding {
            mutable,
            value,
            span,
        },
    );
    Ok(())
}

fn execute_update(
    program: &Program,
    env: &EnvRef,
    target: NameRef,
    value: Value,
    span: Span,
) -> Result<(), Vec<Diagnostic>> {
    if let Some(target_env) = lookup_in_current_function_env(env, target.local) {
        let mut env = target_env.borrow_mut();
        let binding = env.binding_mut(target.local).expect("binding must exist");
        if binding.mutable {
            binding.value = value;
            binding.span = span;
            return Ok(());
        }
        return Err(vec![Diagnostic::new(
            "R006",
            format!(
                "cannot update immutable binding `{}`",
                symbol_name(program, target.name)
            ),
            span,
        )]);
    }

    if let Some(binding) = lookup_beyond_current_function(env, target.local) {
        let code = if binding.mutable { "R007" } else { "R005" };
        let message = if binding.mutable {
            format!(
                "cannot update outer-scope mutable binding `{}` in v1",
                symbol_name(program, target.name)
            )
        } else {
            format!(
                "shadowing is prohibited for `{}`",
                symbol_name(program, target.name)
            )
        };
        return Err(vec![Diagnostic::new(code, message, span)]);
    }

    Err(vec![Diagnostic::new(
        "R008",
        format!(
            "unresolved runtime name `{}`",
            symbol_name(program, target.name)
        ),
        span,
    )])
}

fn call_value(
    program: &Program,
    callee: Value,
    args: Vec<Value>,
    env: &EnvRef,
    span: Span,
) -> Result<Value, Vec<Diagnostic>> {
    match callee {
        Value::Function(function) => {
            let label = function_runtime_label(program, &function);
            let frame_message = format!("called {label} here");
            push_runtime_call_frame(env, frame_message.clone(), span);
            let result = call_function(program, &function, args);
            pop_runtime_call_frame(env);
            result.map_err(|diagnostics| add_runtime_related_note(diagnostics, frame_message, span))
        }
        Value::Builtin(builtin) => call_builtin(builtin, args, env, span),
        _ => Err(vec![Diagnostic::new(
            "R010",
            "attempted to call a non-function value",
            span,
        )]),
    }
}

fn call_function(
    program: &Program,
    function: &ClosureValue,
    args: Vec<Value>,
) -> Result<Value, Vec<Diagnostic>> {
    let definition = function.definition(program);
    if definition.params.len() != args.len() {
        return Err(arg_count_error(
            definition.params.len(),
            args.len(),
            definition.span,
        ));
    }

    let context = {
        let borrowed = function.env.borrow();
        RuntimeContext {
            output: borrowed.output.clone(),
            stderr: borrowed.stderr.clone(),
            program_args: borrowed.program_args.clone(),
            runtime_diagnostics: borrowed.runtime_diagnostics.clone(),
            call_stack: borrowed.call_stack.clone(),
            runtime_handles: borrowed.runtime_handles.clone(),
            package_resource_roots: borrowed.package_resource_roots.clone(),
        }
    };
    let env = Rc::new(RefCell::new(Env::new(
        Some(function.env.clone()),
        true,
        context,
        program.local_count,
    )));
    for (param, arg) in definition.params.iter().zip(args) {
        env.borrow_mut().insert(
            param.local,
            Binding {
                mutable: false,
                value: arg,
                span: definition.span,
            },
        );
    }

    execute_chunk(program, &definition.chunk, env)?.ok_or_else(|| {
        vec![Diagnostic::new(
            "R015",
            "function did not produce a value",
            definition.span,
        )]
    })
}

fn function_runtime_label(program: &Program, function: &ClosureValue) -> String {
    match function.definition(program).name {
        Some(name) => quoted_runtime_name(symbol_name(program, name)),
        None => "anonymous function".to_string(),
    }
}

fn quoted_runtime_name(name: &str) -> String {
    if name.starts_with('`') && name.ends_with('`') {
        name.to_string()
    } else {
        format!("`{name}`")
    }
}

fn push_runtime_call_frame(env: &EnvRef, message: String, span: Span) {
    let call_stack = env.borrow().call_stack.clone();
    call_stack
        .borrow_mut()
        .push(RuntimeCallFrame { message, span });
}

fn pop_runtime_call_frame(env: &EnvRef) {
    let call_stack = env.borrow().call_stack.clone();
    call_stack
        .borrow_mut()
        .pop()
        .expect("runtime call frame should be balanced");
}

fn add_runtime_related_note(
    mut diagnostics: Vec<Diagnostic>,
    message: impl Into<String>,
    span: Span,
) -> Vec<Diagnostic> {
    let message = message.into();
    for diagnostic in &mut diagnostics {
        diagnostic.related.push(RelatedNote {
            message: message.clone(),
            span,
        });
    }
    diagnostics
}

fn add_test_assertion_diagnostic(env: &EnvRef, failure_message: &str, fallback_span: Span) {
    let diagnostic = test_assertion_diagnostic(env, failure_message, fallback_span);
    let runtime_diagnostics = env.borrow().runtime_diagnostics.clone();
    runtime_diagnostics.borrow_mut().push(diagnostic);
}

fn test_assertion_diagnostic(
    env: &EnvRef,
    failure_message: &str,
    fallback_span: Span,
) -> Diagnostic {
    let frames = {
        let borrowed = env.borrow();
        borrowed.call_stack.borrow().clone()
    };
    let primary_span = frames
        .last()
        .map(|frame| frame.span)
        .unwrap_or(fallback_span);
    let mut diagnostic = Diagnostic::new(
        "R021",
        format!("test assertion failed: {failure_message}"),
        primary_span,
    );
    for frame in frames.iter().rev().skip(1) {
        diagnostic.related.push(RelatedNote {
            message: frame.message.clone(),
            span: frame.span,
        });
    }
    diagnostic
}

fn arg_count_error(expected: usize, actual: usize, span: Span) -> Vec<Diagnostic> {
    vec![Diagnostic::new(
        "R012",
        format!("expected {expected} arguments but found {actual}"),
        span,
    )]
}

fn expect_no_args(args: Vec<Value>, span: Span) -> Result<(), Vec<Diagnostic>> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(arg_count_error(0, args.len(), span))
    }
}

fn expect_one_arg(args: Vec<Value>, span: Span) -> Result<Value, Vec<Diagnostic>> {
    let actual = args.len();
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(value), None) => Ok(value),
        _ => Err(arg_count_error(1, actual, span)),
    }
}

fn expect_string_arg(
    args: Vec<Value>,
    span: Span,
    builtin_name: &str,
) -> Result<String, Vec<Diagnostic>> {
    let value = expect_one_arg(args, span)?;
    let Value::String(value) = value else {
        return Err(vec![Diagnostic::new(
            "R014",
            format!("`{builtin_name}` expects String as its first argument"),
            span,
        )]);
    };
    Ok(value)
}

fn expect_bytes_arg(
    args: Vec<Value>,
    span: Span,
    builtin_name: &str,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let value = expect_one_arg(args, span)?;
    let Value::Bytes(value) = value else {
        return Err(vec![Diagnostic::new(
            "R014",
            format!("`{builtin_name}` expects bytes::Bytes as its first argument"),
            span,
        )]);
    };
    Ok(value)
}

fn expect_runtime_handle_arg(
    args: Vec<Value>,
    span: Span,
    builtin_name: &str,
) -> Result<RuntimeHandleValue, Vec<Diagnostic>> {
    let value = expect_one_arg(args, span)?;
    let Value::RuntimeHandle(handle) = value else {
        return Err(vec![Diagnostic::new(
            "R014",
            format!("`{builtin_name}` expects an opaque runtime handle as its first argument"),
            span,
        )]);
    };
    Ok(handle)
}

fn expect_runtime_handle_and_string_args(
    args: Vec<Value>,
    span: Span,
    builtin_name: &str,
) -> Result<(RuntimeHandleValue, String), Vec<Diagnostic>> {
    let (first, second) = expect_two_args(args, span)?;
    let Value::RuntimeHandle(handle) = first else {
        return Err(vec![Diagnostic::new(
            "R014",
            format!("`{builtin_name}` expects an opaque runtime handle as its first argument"),
            span,
        )]);
    };
    let Value::String(text) = second else {
        return Err(vec![Diagnostic::new(
            "R014",
            format!("`{builtin_name}` expects String as its second argument"),
            span,
        )]);
    };
    Ok((handle, text))
}

fn expect_two_string_args(
    args: Vec<Value>,
    span: Span,
    builtin_name: &str,
) -> Result<(String, String), Vec<Diagnostic>> {
    let (first, second) = expect_two_args(args, span)?;
    let Value::String(first) = first else {
        return Err(vec![Diagnostic::new(
            "R014",
            format!("`{builtin_name}` expects String as its first argument"),
            span,
        )]);
    };
    let Value::String(second) = second else {
        return Err(vec![Diagnostic::new(
            "R014",
            format!("`{builtin_name}` expects String as its second argument"),
            span,
        )]);
    };
    Ok((first, second))
}

fn expect_two_args(args: Vec<Value>, span: Span) -> Result<(Value, Value), Vec<Diagnostic>> {
    let actual = args.len();
    let mut args = args.into_iter();
    match (args.next(), args.next(), args.next()) {
        (Some(first), Some(second), None) => Ok((first, second)),
        _ => Err(arg_count_error(2, actual, span)),
    }
}

fn expect_three_args(
    args: Vec<Value>,
    span: Span,
) -> Result<(Value, Value, Value), Vec<Diagnostic>> {
    let actual = args.len();
    let mut args = args.into_iter();
    match (args.next(), args.next(), args.next(), args.next()) {
        (Some(first), Some(second), Some(third), None) => Ok((first, second, third)),
        _ => Err(arg_count_error(3, actual, span)),
    }
}

struct JsonParser<'a> {
    text: &'a str,
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, pos: 0 }
    }

    fn parse(mut self) -> Result<Value, JsonDataError> {
        self.skip_ws();
        let value = self.parse_value(0)?;
        self.skip_ws();
        if self.pos == self.text.len() {
            Ok(value)
        } else {
            Err(json_error(
                JsonErrorKind::TrailingCharacters,
                "trailing characters after JSON value",
                self.pos,
            ))
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<Value, JsonDataError> {
        if depth > JSON_NESTING_LIMIT {
            return Err(json_error(
                JsonErrorKind::NestingLimitExceeded,
                "JSON nesting limit exceeded",
                self.pos,
            ));
        }

        self.skip_ws();
        let Some(byte) = self.peek_byte() else {
            return Err(json_error(
                JsonErrorKind::UnexpectedEnd,
                "unexpected end of JSON input",
                self.pos,
            ));
        };

        match byte {
            b'n' => {
                self.expect_literal("null")?;
                Ok(json_value_variant("Null", None))
            }
            b't' => {
                self.expect_literal("true")?;
                Ok(json_value_variant("Bool", Some(Value::Bool(true))))
            }
            b'f' => {
                self.expect_literal("false")?;
                Ok(json_value_variant("Bool", Some(Value::Bool(false))))
            }
            b'"' => {
                let text = self.parse_string()?;
                Ok(json_value_variant("String", Some(Value::String(text))))
            }
            b'[' => self.parse_array(depth),
            b'{' => self.parse_object(depth),
            b'-' | b'0'..=b'9' => self.parse_number(),
            _ => Err(json_error(
                JsonErrorKind::UnexpectedToken,
                "unexpected token while parsing JSON value",
                self.pos,
            )),
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<Value, JsonDataError> {
        self.pos += 1;
        self.skip_ws();
        let mut items = Vec::new();
        if self.consume_byte(b']') {
            return Ok(json_value_variant("Array", Some(Value::List(items))));
        }

        loop {
            items.push(self.parse_value(depth + 1)?);
            self.skip_ws();
            if self.consume_byte(b']') {
                return Ok(json_value_variant("Array", Some(Value::List(items))));
            }
            if !self.consume_byte(b',') {
                return Err(json_error(
                    JsonErrorKind::UnexpectedToken,
                    "expected `,` or `]` in JSON array",
                    self.pos,
                ));
            }
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<Value, JsonDataError> {
        self.pos += 1;
        self.skip_ws();
        let mut entries = Vec::new();
        if self.consume_byte(b'}') {
            return Ok(json_value_variant(
                "Object",
                Some(Value::Map(MapValue { entries })),
            ));
        }

        loop {
            self.skip_ws();
            if self.peek_byte() != Some(b'"') {
                return Err(json_error(
                    JsonErrorKind::UnexpectedToken,
                    "expected JSON object key",
                    self.pos,
                ));
            }
            let key_offset = self.pos;
            let key = self.parse_string()?;
            if entries
                .iter()
                .any(|entry: &MapEntryValue| entry.key == MapKey::String(key.clone()))
            {
                return Err(json_error(
                    JsonErrorKind::DuplicateKey,
                    "duplicate JSON object key",
                    key_offset,
                ));
            }

            self.skip_ws();
            if !self.consume_byte(b':') {
                return Err(json_error(
                    JsonErrorKind::UnexpectedToken,
                    "expected `:` after JSON object key",
                    self.pos,
                ));
            }
            let value = self.parse_value(depth + 1)?;
            entries.push(MapEntryValue {
                key: MapKey::String(key),
                value,
            });
            self.skip_ws();
            if self.consume_byte(b'}') {
                return Ok(json_value_variant(
                    "Object",
                    Some(Value::Map(MapValue { entries })),
                ));
            }
            if !self.consume_byte(b',') {
                return Err(json_error(
                    JsonErrorKind::UnexpectedToken,
                    "expected `,` or `}` in JSON object",
                    self.pos,
                ));
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, JsonDataError> {
        let start = self.pos;
        if !self.consume_byte(b'"') {
            return Err(json_error(
                JsonErrorKind::UnexpectedToken,
                "expected JSON string",
                self.pos,
            ));
        }

        let mut out = String::new();
        while self.pos < self.text.len() {
            let ch = self.text[self.pos..]
                .chars()
                .next()
                .expect("pos should be inside text");
            match ch {
                '"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                '\\' => {
                    let escape_offset = self.pos;
                    self.pos += 1;
                    let Some(escape) = self.next_char() else {
                        return Err(json_error(
                            JsonErrorKind::UnexpectedEnd,
                            "unexpected end in JSON string escape",
                            escape_offset,
                        ));
                    };
                    match escape {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        '/' => out.push('/'),
                        'b' => out.push('\u{0008}'),
                        'f' => out.push('\u{000c}'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => out.push(self.parse_unicode_escape(escape_offset)?),
                        _ => {
                            return Err(json_error(
                                JsonErrorKind::InvalidEscape,
                                "invalid JSON string escape",
                                escape_offset,
                            ));
                        }
                    }
                }
                '\u{0000}'..='\u{001f}' => {
                    return Err(json_error(
                        JsonErrorKind::InvalidEscape,
                        "unescaped control character in JSON string",
                        self.pos,
                    ));
                }
                _ => {
                    out.push(ch);
                    self.pos += ch.len_utf8();
                }
            }
        }

        Err(json_error(
            JsonErrorKind::UnexpectedEnd,
            "unterminated JSON string",
            start,
        ))
    }

    fn parse_unicode_escape(&mut self, escape_offset: usize) -> Result<char, JsonDataError> {
        let code = self.parse_hex_quad(escape_offset)?;
        if (0xd800..=0xdbff).contains(&code) {
            if !self.consume_byte(b'\\') || !self.consume_byte(b'u') {
                return Err(json_error(
                    JsonErrorKind::InvalidEscape,
                    "high surrogate must be followed by a low surrogate",
                    escape_offset,
                ));
            }
            let low = self.parse_hex_quad(escape_offset)?;
            if !(0xdc00..=0xdfff).contains(&low) {
                return Err(json_error(
                    JsonErrorKind::InvalidEscape,
                    "high surrogate must be followed by a low surrogate",
                    escape_offset,
                ));
            }
            let high_ten = u32::from(code) - 0xd800;
            let low_ten = u32::from(low) - 0xdc00;
            let scalar = 0x10000 + ((high_ten << 10) | low_ten);
            return char::from_u32(scalar).ok_or_else(|| {
                json_error(
                    JsonErrorKind::InvalidEscape,
                    "invalid Unicode scalar value",
                    escape_offset,
                )
            });
        }
        if (0xdc00..=0xdfff).contains(&code) {
            return Err(json_error(
                JsonErrorKind::InvalidEscape,
                "low surrogate without preceding high surrogate",
                escape_offset,
            ));
        }
        char::from_u32(u32::from(code)).ok_or_else(|| {
            json_error(
                JsonErrorKind::InvalidEscape,
                "invalid Unicode scalar value",
                escape_offset,
            )
        })
    }

    fn parse_hex_quad(&mut self, escape_offset: usize) -> Result<u16, JsonDataError> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let Some(byte) = self.peek_byte() else {
                return Err(json_error(
                    JsonErrorKind::UnexpectedEnd,
                    "unexpected end in Unicode escape",
                    escape_offset,
                ));
            };
            let Some(digit) = hex_digit(byte) else {
                return Err(json_error(
                    JsonErrorKind::InvalidEscape,
                    "invalid hex digit in Unicode escape",
                    self.pos,
                ));
            };
            self.pos += 1;
            value = (value << 4) | u16::from(digit);
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<Value, JsonDataError> {
        let start = self.pos;
        let Some((end, shape)) = consume_json_number(self.text, start) else {
            return Err(json_error(
                JsonErrorKind::InvalidNumber,
                "invalid JSON number",
                start,
            ));
        };
        self.pos = end;
        let raw = &self.text[start..end];
        let number = match shape {
            JsonNumberShape::Integral => match raw.parse::<i64>() {
                Ok(value) => json_number_variant("Int", Value::Int(value)),
                Err(_) => {
                    return Err(json_error(
                        JsonErrorKind::NumberOutOfRange,
                        "JSON integer is outside Int range",
                        start,
                    ));
                }
            },
            JsonNumberShape::NonIntegral => {
                json_number_variant("Raw", Value::String(raw.to_string()))
            }
        };
        Ok(json_value_variant("Number", Some(number)))
    }

    fn expect_literal(&mut self, literal: &str) -> Result<(), JsonDataError> {
        if self.text[self.pos..].starts_with(literal) {
            self.pos += literal.len();
            Ok(())
        } else if self.text.len() - self.pos < literal.len() {
            Err(json_error(
                JsonErrorKind::UnexpectedEnd,
                format!("expected `{literal}`"),
                self.pos,
            ))
        } else {
            Err(json_error(
                JsonErrorKind::UnexpectedToken,
                format!("expected `{literal}`"),
                self.pos,
            ))
        }
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek_byte(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.text.as_bytes().get(self.pos).copied()
    }

    fn consume_byte(&mut self, expected: u8) -> bool {
        if self.peek_byte() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.text[self.pos..].chars().next()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn consume_json_number(text: &str, start: usize) -> Option<(usize, JsonNumberShape)> {
    let bytes = text.as_bytes();
    let mut pos = start;
    if bytes.get(pos) == Some(&b'-') {
        pos += 1;
    }

    match bytes.get(pos).copied()? {
        b'0' => {
            pos += 1;
            if matches!(bytes.get(pos).copied(), Some(b'0'..=b'9')) {
                return None;
            }
        }
        b'1'..=b'9' => {
            pos += 1;
            while matches!(bytes.get(pos).copied(), Some(b'0'..=b'9')) {
                pos += 1;
            }
        }
        _ => return None,
    }

    let mut shape = JsonNumberShape::Integral;
    if bytes.get(pos) == Some(&b'.') {
        shape = JsonNumberShape::NonIntegral;
        pos += 1;
        if !matches!(bytes.get(pos).copied(), Some(b'0'..=b'9')) {
            return None;
        }
        while matches!(bytes.get(pos).copied(), Some(b'0'..=b'9')) {
            pos += 1;
        }
    }

    if matches!(bytes.get(pos).copied(), Some(b'e' | b'E')) {
        shape = JsonNumberShape::NonIntegral;
        pos += 1;
        if matches!(bytes.get(pos).copied(), Some(b'+' | b'-')) {
            pos += 1;
        }
        if !matches!(bytes.get(pos).copied(), Some(b'0'..=b'9')) {
            return None;
        }
        while matches!(bytes.get(pos).copied(), Some(b'0'..=b'9')) {
            pos += 1;
        }
    }

    Some((pos, shape))
}

fn validate_json_number_text(text: &str) -> Result<JsonNumberShape, ()> {
    match consume_json_number(text, 0) {
        Some((end, shape)) if end == text.len() => Ok(shape),
        _ => Err(()),
    }
}

fn encode_json_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0000}'..='\u{001f}' => out.push_str(&format!("\\u{:04x}", ch as u32)),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn encode_json_value(value: &Value, depth: usize) -> Result<String, JsonDataError> {
    if depth > JSON_NESTING_LIMIT {
        return Err(json_value_error(
            JsonErrorKind::NestingLimitExceeded,
            "JSON nesting limit exceeded",
        ));
    }

    let Value::Enum(value) = value else {
        return Err(json_value_error(
            JsonErrorKind::UnexpectedToken,
            "expected json::Value",
        ));
    };
    if value.type_name != crate::std_package::JSON_VALUE_MANGLED_NAME {
        return Err(json_value_error(
            JsonErrorKind::UnexpectedToken,
            "expected json::Value",
        ));
    }

    match value.variant_name.as_str() {
        "Null" => Ok("null".to_string()),
        "Bool" => match value.payload.as_deref() {
            Some(Value::Bool(value)) => Ok(value.to_string()),
            _ => Err(json_value_error(
                JsonErrorKind::UnexpectedToken,
                "json::Value::Bool requires Bool payload",
            )),
        },
        "Number" => match value.payload.as_deref() {
            Some(number) => encode_json_number(number),
            None => Err(json_value_error(
                JsonErrorKind::UnexpectedToken,
                "json::Value::Number requires Number payload",
            )),
        },
        "String" => match value.payload.as_deref() {
            Some(Value::String(text)) => Ok(encode_json_string(text)),
            _ => Err(json_value_error(
                JsonErrorKind::UnexpectedToken,
                "json::Value::String requires String payload",
            )),
        },
        "Array" => match value.payload.as_deref() {
            Some(Value::List(items)) => {
                let mut out = String::from("[");
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    out.push_str(&encode_json_value(item, depth + 1)?);
                }
                out.push(']');
                Ok(out)
            }
            _ => Err(json_value_error(
                JsonErrorKind::UnexpectedToken,
                "json::Value::Array requires List[json::Value] payload",
            )),
        },
        "Object" => match value.payload.as_deref() {
            Some(Value::Map(map)) => {
                let mut entries = Vec::new();
                for entry in &map.entries {
                    let MapKey::String(key) = &entry.key else {
                        return Err(json_value_error(
                            JsonErrorKind::UnexpectedToken,
                            "json::Value::Object requires String keys",
                        ));
                    };
                    entries.push((key.as_str(), &entry.value));
                }
                entries.sort_by(|left, right| left.0.cmp(right.0));

                let mut out = String::from("{");
                for (index, (key, item)) in entries.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    out.push_str(&encode_json_string(key));
                    out.push(':');
                    out.push_str(&encode_json_value(item, depth + 1)?);
                }
                out.push('}');
                Ok(out)
            }
            _ => Err(json_value_error(
                JsonErrorKind::UnexpectedToken,
                "json::Value::Object requires Map[String, json::Value] payload",
            )),
        },
        _ => Err(json_value_error(
            JsonErrorKind::UnexpectedToken,
            "unknown json::Value variant",
        )),
    }
}

fn encode_json_number(number: &Value) -> Result<String, JsonDataError> {
    let Value::Enum(number) = number else {
        return Err(json_value_error(
            JsonErrorKind::UnexpectedToken,
            "expected json::Number",
        ));
    };
    if number.type_name != crate::std_package::JSON_NUMBER_MANGLED_NAME {
        return Err(json_value_error(
            JsonErrorKind::UnexpectedToken,
            "expected json::Number",
        ));
    }

    match number.variant_name.as_str() {
        "Int" => match number.payload.as_deref() {
            Some(Value::Int(value)) => Ok(value.to_string()),
            _ => Err(json_value_error(
                JsonErrorKind::UnexpectedToken,
                "json::Number::Int requires Int payload",
            )),
        },
        "Raw" => match number.payload.as_deref() {
            Some(Value::String(raw)) => {
                validate_json_number_text(raw).map_err(|()| {
                    json_value_error(JsonErrorKind::InvalidNumber, "invalid raw JSON number")
                })?;
                Ok(raw.clone())
            }
            _ => Err(json_value_error(
                JsonErrorKind::UnexpectedToken,
                "json::Number::Raw requires String payload",
            )),
        },
        _ => Err(json_value_error(
            JsonErrorKind::UnexpectedToken,
            "unknown json::Number variant",
        )),
    }
}

fn json_number_as_int(number: Value) -> Result<i64, JsonDataError> {
    let Value::Enum(number) = number else {
        return Err(json_value_error(
            JsonErrorKind::UnexpectedToken,
            "expected json::Number",
        ));
    };
    if number.type_name != crate::std_package::JSON_NUMBER_MANGLED_NAME {
        return Err(json_value_error(
            JsonErrorKind::UnexpectedToken,
            "expected json::Number",
        ));
    }

    match number.variant_name.as_str() {
        "Int" => match number.payload.as_deref() {
            Some(Value::Int(value)) => Ok(*value),
            _ => Err(json_value_error(
                JsonErrorKind::UnexpectedToken,
                "json::Number::Int requires Int payload",
            )),
        },
        "Raw" => match number.payload.as_deref() {
            Some(Value::String(raw)) => match validate_json_number_text(raw) {
                Ok(JsonNumberShape::Integral) => raw.parse::<i64>().map_err(|_| {
                    json_value_error(
                        JsonErrorKind::NumberOutOfRange,
                        "JSON integer is outside Int range",
                    )
                }),
                Ok(JsonNumberShape::NonIntegral) => Err(json_value_error(
                    JsonErrorKind::InvalidNumber,
                    "JSON number is not an Int",
                )),
                Err(()) => Err(json_value_error(
                    JsonErrorKind::InvalidNumber,
                    "invalid raw JSON number",
                )),
            },
            _ => Err(json_value_error(
                JsonErrorKind::UnexpectedToken,
                "json::Number::Raw requires String payload",
            )),
        },
        _ => Err(json_value_error(
            JsonErrorKind::UnexpectedToken,
            "unknown json::Number variant",
        )),
    }
}

fn typed_value_to_json_value(
    program: &Program,
    schema: &JsonDecodeSchema,
    value: &Value,
    path: &str,
) -> Result<Value, JsonDataError> {
    match schema {
        JsonDecodeSchema::String => match value {
            Value::String(text) => Ok(json_value_variant(
                "String",
                Some(Value::String(text.clone())),
            )),
            _ => Err(typed_json_encode_shape_error(path, "String")),
        },
        JsonDecodeSchema::Int => match value {
            Value::Int(number) => Ok(json_value_variant(
                "Number",
                Some(json_number_variant("Int", Value::Int(*number))),
            )),
            _ => Err(typed_json_encode_shape_error(path, "Int")),
        },
        JsonDecodeSchema::Bool => match value {
            Value::Bool(boolean) => Ok(json_value_variant("Bool", Some(Value::Bool(*boolean)))),
            _ => Err(typed_json_encode_shape_error(path, "Bool")),
        },
        JsonDecodeSchema::JsonValue => Ok(value.clone()),
        JsonDecodeSchema::StringList => typed_scalar_list_to_json_array(
            program,
            value,
            path,
            "List[String]",
            &JsonDecodeSchema::String,
        ),
        JsonDecodeSchema::IntList => typed_scalar_list_to_json_array(
            program,
            value,
            path,
            "List[Int]",
            &JsonDecodeSchema::Int,
        ),
        JsonDecodeSchema::BoolList => typed_scalar_list_to_json_array(
            program,
            value,
            path,
            "List[Bool]",
            &JsonDecodeSchema::Bool,
        ),
        JsonDecodeSchema::JsonObjectMap => typed_json_object_map_to_json_object(value, path),
        JsonDecodeSchema::Option(item) => {
            if is_option_none(value) {
                return Ok(json_value_variant("Null", None));
            }
            let Some(payload) = option_some_payload(value) else {
                return Err(typed_json_encode_shape_error(path, "Option"));
            };
            typed_value_to_json_value(program, item, payload, path)
        }
        JsonDecodeSchema::List(item) => {
            typed_list_to_json_array(program, item, value, path, "List")
        }
        JsonDecodeSchema::TypedStringMap(item) => {
            typed_string_map_to_json_object(program, item, value, path)
        }
        JsonDecodeSchema::Record {
            type_name, fields, ..
        } => typed_record_to_json_object(program, *type_name, fields, value, path),
        JsonDecodeSchema::Enum {
            type_name,
            variants,
            ..
        } => typed_enum_to_json_value(program, *type_name, variants, value, path),
    }
}

fn typed_scalar_list_to_json_array(
    program: &Program,
    value: &Value,
    path: &str,
    expected: &str,
    item_schema: &JsonDecodeSchema,
) -> Result<Value, JsonDataError> {
    typed_list_to_json_array(program, item_schema, value, path, expected)
}

fn typed_list_to_json_array(
    program: &Program,
    item_schema: &JsonDecodeSchema,
    value: &Value,
    path: &str,
    expected: &str,
) -> Result<Value, JsonDataError> {
    let Value::List(items) = value else {
        return Err(typed_json_encode_shape_error(path, expected));
    };
    let mut encoded = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let item_path = append_json_decode_index_path(path, index);
        encoded.push(typed_value_to_json_value(
            program,
            item_schema,
            item,
            &item_path,
        )?);
    }
    Ok(json_value_variant("Array", Some(Value::List(encoded))))
}

fn typed_json_object_map_to_json_object(value: &Value, path: &str) -> Result<Value, JsonDataError> {
    let Value::Map(map) = value else {
        return Err(typed_json_encode_shape_error(
            path,
            "Map[String, json::Value]",
        ));
    };
    let mut entries = Vec::with_capacity(map.entries.len());
    for entry in &map.entries {
        let MapKey::String(key) = &entry.key else {
            return Err(typed_json_encode_shape_error(
                path,
                "Map[String, json::Value]",
            ));
        };
        entries.push(MapEntryValue {
            key: MapKey::String(key.clone()),
            value: entry.value.clone(),
        });
    }
    Ok(json_value_variant(
        "Object",
        Some(Value::Map(MapValue { entries })),
    ))
}

fn typed_string_map_to_json_object(
    program: &Program,
    item_schema: &JsonDecodeSchema,
    value: &Value,
    path: &str,
) -> Result<Value, JsonDataError> {
    let Value::Map(map) = value else {
        return Err(typed_json_encode_shape_error(path, "Map[String, T]"));
    };
    let mut entries = Vec::with_capacity(map.entries.len());
    for entry in &map.entries {
        let MapKey::String(key) = &entry.key else {
            return Err(typed_json_encode_shape_error(path, "Map[String, T]"));
        };
        let item_path = append_json_decode_field_path(path, key);
        entries.push(MapEntryValue {
            key: MapKey::String(key.clone()),
            value: typed_value_to_json_value(program, item_schema, &entry.value, &item_path)?,
        });
    }
    Ok(json_value_variant(
        "Object",
        Some(Value::Map(MapValue { entries })),
    ))
}

fn typed_record_to_json_object(
    program: &Program,
    type_name: Symbol,
    fields: &[JsonDecodeFieldSchema],
    value: &Value,
    path: &str,
) -> Result<Value, JsonDataError> {
    let Value::Record(record) = value else {
        return Err(typed_json_encode_shape_error(
            path,
            symbol_name(program, type_name),
        ));
    };
    let expected_type = symbol_name(program, type_name);
    if record.type_name != expected_type {
        return Err(typed_json_encode_shape_error(path, expected_type));
    }

    let mut entries = Vec::with_capacity(fields.len());
    for field in fields {
        let source_name = symbol_name(program, field.name);
        let field_value = record_field_value(record, source_name).ok_or_else(|| {
            json_value_error(
                JsonErrorKind::UnexpectedToken,
                format!("typed JSON encoding missing record field `{source_name}`"),
            )
        })?;
        let wire_name = json_decode_wire_name(program, field.name, field.wire_name);
        let field_path = append_json_decode_field_path(path, wire_name);
        if matches!(field.schema, JsonDecodeSchema::Option(_)) && is_option_none(field_value) {
            continue;
        }
        validate_json_decoded_field(field, field_value, &field_path)?;
        entries.push(MapEntryValue {
            key: MapKey::String(wire_name.to_string()),
            value: typed_value_to_json_value(program, &field.schema, field_value, &field_path)?,
        });
    }

    Ok(json_value_variant(
        "Object",
        Some(Value::Map(MapValue { entries })),
    ))
}

fn typed_enum_to_json_value(
    program: &Program,
    type_name: Symbol,
    variants: &[JsonDecodeVariantSchema],
    value: &Value,
    path: &str,
) -> Result<Value, JsonDataError> {
    let Value::Enum(enumeration) = value else {
        return Err(typed_json_encode_shape_error(
            path,
            symbol_name(program, type_name),
        ));
    };
    let expected_type = symbol_name(program, type_name);
    if enumeration.type_name != expected_type {
        return Err(typed_json_encode_shape_error(path, expected_type));
    }
    let variant = variants
        .iter()
        .find(|variant| symbol_name(program, variant.name) == enumeration.variant_name)
        .ok_or_else(|| {
            json_value_error(
                JsonErrorKind::UnexpectedToken,
                format!(
                    "typed JSON encoding found unknown enum variant `{}` at path {}",
                    enumeration.variant_name,
                    json_decode_path_label(path)
                ),
            )
        })?;
    let wire_name = json_decode_wire_name(program, variant.name, variant.wire_name);
    match (&variant.payload, enumeration.payload.as_deref()) {
        (None, None) => Ok(json_value_variant(
            "String",
            Some(Value::String(wire_name.to_string())),
        )),
        (Some(payload_schema), Some(payload)) => {
            let item_path = append_json_decode_field_path(path, wire_name);
            let encoded = typed_value_to_json_value(program, payload_schema, payload, &item_path)?;
            Ok(json_value_variant(
                "Object",
                Some(Value::Map(MapValue {
                    entries: vec![MapEntryValue {
                        key: MapKey::String(wire_name.to_string()),
                        value: encoded,
                    }],
                })),
            ))
        }
        (None, Some(_)) => Err(typed_json_encode_shape_error(
            path,
            "enum variant without payload",
        )),
        (Some(_), None) => Err(typed_json_encode_shape_error(path, "enum variant payload")),
    }
}

#[derive(Clone, Debug)]
struct ParsedCliField {
    name: String,
    value: Value,
}

enum CliCommandRequestOutcome {
    Help(String),
    Parsed(Value),
}

enum CliGlobalParseOutcome {
    Help,
    Parsed {
        parsed: Vec<ParsedCliField>,
        command_start: usize,
    },
}

fn cli_schema_is_command(schema: &CliSchema) -> bool {
    !schema.commands.is_empty()
}

fn cli_schema_is_wrapper(schema: &CliSchema) -> bool {
    schema.subcommand.is_some()
}

fn cli_parse_or(
    program: &Program,
    schema: &CliSchema,
    args: &[String],
    defaults: &Value,
) -> Result<Value, CliInputError> {
    if cli_schema_is_command(schema) {
        return Err(cli_input_error(
            CliErrorKind::UnsupportedTarget,
            "",
            "cli::parse_or does not support command enum schemas",
        ));
    }
    if cli_schema_is_wrapper(schema) {
        return Err(cli_input_error(
            CliErrorKind::UnsupportedTarget,
            "",
            "cli::parse_or does not support wrapper record schemas",
        ));
    }

    let type_name = schema.type_name;
    let Value::Record(default_record) = defaults else {
        return Err(cli_input_error(
            CliErrorKind::UnsupportedTarget,
            "",
            "cli::parse_or defaults must be a record value",
        ));
    };

    let parsed = cli_parse_args(program, schema, args)?;
    let mut out_fields = default_record.fields.clone();
    for item in parsed {
        match out_fields.iter_mut().find(|field| field.name == item.name) {
            Some(field) => field.value = item.value,
            None => out_fields.push(RecordFieldValue {
                name: item.name,
                value: item.value,
            }),
        }
    }

    Ok(Value::Record(RecordValue {
        type_name: symbol_name(program, type_name).to_string(),
        fields: out_fields,
    }))
}

fn cli_parse(
    program: &Program,
    schema: &CliSchema,
    args: &[String],
) -> Result<Value, CliInputError> {
    if cli_schema_is_command(schema) {
        return cli_parse_command(program, schema, args);
    }
    if cli_schema_is_wrapper(schema) {
        return cli_parse_wrapper(program, schema, args);
    }

    let parsed = cli_parse_args(program, schema, args)?;
    let mut out_fields = Vec::with_capacity(schema.fields.len());
    for field in &schema.fields {
        let name = symbol_name(program, field.name).to_string();
        let value = match parsed.iter().find(|parsed| parsed.name == name) {
            Some(parsed) => parsed.value.clone(),
            None => cli_synthesized_absent_value(&field.value).ok_or_else(|| {
                let argument = cli_missing_argument_label(program, field);
                let message = if field.position.is_some() {
                    format!("missing required CLI positional `{argument}`")
                } else {
                    format!("missing required CLI option `{argument}`")
                };
                cli_input_error(CliErrorKind::MissingArgument, argument.clone(), message)
            })?,
        };
        let argument = cli_missing_argument_label(program, field);
        validate_cli_parsed_field(field, &value, &argument)?;
        out_fields.push(RecordFieldValue { name, value });
    }

    Ok(Value::Record(RecordValue {
        type_name: symbol_name(program, schema.type_name).to_string(),
        fields: out_fields,
    }))
}

fn cli_parse_wrapper(
    program: &Program,
    schema: &CliSchema,
    args: &[String],
) -> Result<Value, CliInputError> {
    let Some(subcommand) = schema.subcommand.as_ref() else {
        return Err(cli_input_error(
            CliErrorKind::UnsupportedTarget,
            "",
            "missing CLI wrapper subcommand schema",
        ));
    };
    let CliGlobalParseOutcome::Parsed {
        parsed,
        command_start,
    } = cli_parse_global_options(program, schema, args, false)?
    else {
        return Err(cli_input_error(
            CliErrorKind::UnsupportedTarget,
            "--help",
            "cli::parse does not handle CLI help requests",
        ));
    };
    let command_value = cli_parse_command(program, &subcommand.schema, &args[command_start..])?;
    cli_build_wrapper_record(program, schema, parsed, command_value)
}

fn cli_parse_command(
    program: &Program,
    schema: &CliSchema,
    args: &[String],
) -> Result<Value, CliInputError> {
    let Some(token) = args.first() else {
        return Err(cli_input_error(
            CliErrorKind::MissingArgument,
            "<command>",
            "missing required CLI command `<command>`",
        ));
    };
    let Some(command) = cli_command_by_name(program, &schema.commands, token) else {
        return Err(cli_input_error(
            CliErrorKind::UnknownArgument,
            token.clone(),
            format!("unknown CLI command `{token}`"),
        ));
    };
    let payload = cli_parse(program, &command.payload, &args[1..])?;
    Ok(make_enum_value(
        program,
        schema.type_name,
        command.variant_name,
        Some(payload),
    ))
}

fn cli_parse_command_request(
    program: &Program,
    schema: &CliSchema,
    args: &[String],
    program_name: &str,
) -> Result<Value, CliInputError> {
    match cli_parse_command_request_outcome(program, schema, args, program_name)? {
        CliCommandRequestOutcome::Help(help) => Ok(cli_request_help(help)),
        CliCommandRequestOutcome::Parsed(value) => Ok(cli_request_parsed(value)),
    }
}

fn cli_parse_wrapper_request(
    program: &Program,
    schema: &CliSchema,
    args: &[String],
    program_name: &str,
) -> Result<Value, CliInputError> {
    match cli_parse_wrapper_request_outcome(program, schema, args, program_name)? {
        CliCommandRequestOutcome::Help(help) => Ok(cli_request_help(help)),
        CliCommandRequestOutcome::Parsed(value) => Ok(cli_request_parsed(value)),
    }
}

fn cli_parse_wrapper_request_outcome(
    program: &Program,
    schema: &CliSchema,
    args: &[String],
    program_name: &str,
) -> Result<CliCommandRequestOutcome, CliInputError> {
    let Some(subcommand) = schema.subcommand.as_ref() else {
        return Err(cli_input_error(
            CliErrorKind::UnsupportedTarget,
            "",
            "missing CLI wrapper subcommand schema",
        ));
    };
    let CliGlobalParseOutcome::Parsed {
        parsed,
        command_start,
    } = cli_parse_global_options(program, schema, args, true)?
    else {
        return Ok(CliCommandRequestOutcome::Help(cli_help_for_required(
            program,
            schema,
            program_name,
        )));
    };
    let payload_outcome = cli_parse_command_request_outcome(
        program,
        &subcommand.schema,
        &args[command_start..],
        program_name,
    )?;
    match payload_outcome {
        CliCommandRequestOutcome::Help(help) => Ok(CliCommandRequestOutcome::Help(help)),
        CliCommandRequestOutcome::Parsed(command_value) => Ok(CliCommandRequestOutcome::Parsed(
            cli_build_wrapper_record(program, schema, parsed, command_value)?,
        )),
    }
}

fn cli_parse_command_request_outcome(
    program: &Program,
    schema: &CliSchema,
    args: &[String],
    program_name: &str,
) -> Result<CliCommandRequestOutcome, CliInputError> {
    if cli_first_arg_is_help(args) {
        return Ok(CliCommandRequestOutcome::Help(cli_help_for_required(
            program,
            schema,
            program_name,
        )));
    }
    let Some(token) = args.first() else {
        return Err(cli_input_error(
            CliErrorKind::MissingArgument,
            "<command>",
            "missing required CLI command `<command>`",
        ));
    };
    let Some(command) = cli_command_by_name(program, &schema.commands, token) else {
        return Err(cli_input_error(
            CliErrorKind::UnknownArgument,
            token.clone(),
            format!("unknown CLI command `{token}`"),
        ));
    };

    let command_program_name = format!("{program_name} {token}");
    let payload_outcome = if cli_schema_is_command(&command.payload) {
        cli_parse_command_request_outcome(
            program,
            &command.payload,
            &args[1..],
            &command_program_name,
        )?
    } else if cli_help_requested(&args[1..]) {
        CliCommandRequestOutcome::Help(cli_help_for_required(
            program,
            &command.payload,
            &command_program_name,
        ))
    } else {
        CliCommandRequestOutcome::Parsed(cli_parse(program, &command.payload, &args[1..])?)
    };

    match payload_outcome {
        CliCommandRequestOutcome::Help(help) => Ok(CliCommandRequestOutcome::Help(help)),
        CliCommandRequestOutcome::Parsed(payload) => {
            Ok(CliCommandRequestOutcome::Parsed(make_enum_value(
                program,
                schema.type_name,
                command.variant_name,
                Some(payload),
            )))
        }
    }
}

fn cli_first_arg_is_help(args: &[String]) -> bool {
    matches!(args.first().map(String::as_str), Some("--help" | "-h"))
}

fn cli_help_requested(args: &[String]) -> bool {
    for arg in args {
        if arg == "--" {
            return false;
        }
        if arg == "--help" || arg == "-h" {
            return true;
        }
    }
    false
}

fn cli_build_wrapper_record(
    program: &Program,
    schema: &CliSchema,
    parsed: Vec<ParsedCliField>,
    command_value: Value,
) -> Result<Value, CliInputError> {
    let Some(subcommand) = schema.subcommand.as_ref() else {
        return Err(cli_input_error(
            CliErrorKind::UnsupportedTarget,
            "",
            "missing CLI wrapper subcommand schema",
        ));
    };

    let mut out_fields = Vec::with_capacity(schema.fields.len() + 1);
    for field in &schema.fields {
        let name = symbol_name(program, field.name).to_string();
        let value = match parsed.iter().find(|parsed| parsed.name == name) {
            Some(parsed) => parsed.value.clone(),
            None => cli_synthesized_absent_value(&field.value).ok_or_else(|| {
                let argument = cli_missing_argument_label(program, field);
                cli_input_error(
                    CliErrorKind::MissingArgument,
                    argument.clone(),
                    format!("missing required CLI option `{argument}`"),
                )
            })?,
        };
        let argument = cli_missing_argument_label(program, field);
        validate_cli_parsed_field(field, &value, &argument)?;
        out_fields.push(RecordFieldValue { name, value });
    }
    out_fields.push(RecordFieldValue {
        name: symbol_name(program, subcommand.field_name).to_string(),
        value: command_value,
    });

    Ok(Value::Record(RecordValue {
        type_name: symbol_name(program, schema.type_name).to_string(),
        fields: out_fields,
    }))
}

fn cli_parse_global_options(
    program: &Program,
    schema: &CliSchema,
    args: &[String],
    allow_help: bool,
) -> Result<CliGlobalParseOutcome, CliInputError> {
    let fields = &schema.fields;
    let mut parsed = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if allow_help && matches!(arg.as_str(), "--help" | "-h") {
            return Ok(CliGlobalParseOutcome::Help);
        }
        if arg == "--" {
            return Ok(CliGlobalParseOutcome::Parsed {
                parsed,
                command_start: args.len(),
            });
        }
        if arg.starts_with("--") && arg.len() > 2 {
            let marker = &arg[2..];
            let (option_name, inline_value) = match marker.split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (marker, None),
            };
            let argument = format!("--{option_name}");
            let Some(field) = cli_field_by_option_name(program, fields, option_name) else {
                return Err(cli_input_error(
                    CliErrorKind::UnknownArgument,
                    argument.clone(),
                    format!("unknown CLI option `{argument}`"),
                ));
            };

            let mut consumed_next = false;
            let raw_value = if let Some(value) = inline_value {
                value.to_string()
            } else if cli_schema_accepts_bare_bool(&field.value) {
                match args.get(index + 1).filter(|value| cli_bool_literal(value)) {
                    Some(next) => {
                        consumed_next = true;
                        next.clone()
                    }
                    None => "true".to_string(),
                }
            } else if let Some(next) = args
                .get(index + 1)
                .filter(|value| !cli_arg_looks_like_option_marker(value))
            {
                consumed_next = true;
                next.clone()
            } else {
                return Err(cli_input_error(
                    CliErrorKind::MissingValue,
                    argument.clone(),
                    format!("missing value for `{argument}`"),
                ));
            };

            cli_parse_and_merge_field(program, field, &mut parsed, &raw_value, &argument)?;
            index += if consumed_next { 2 } else { 1 };
            continue;
        }

        if arg.starts_with('-') && arg.len() > 1 {
            let consumed_next =
                cli_parse_short_token(program, fields, args, index, &mut parsed, false)?;
            index += if consumed_next { 2 } else { 1 };
            continue;
        }

        return Ok(CliGlobalParseOutcome::Parsed {
            parsed,
            command_start: index,
        });
    }

    Ok(CliGlobalParseOutcome::Parsed {
        parsed,
        command_start: args.len(),
    })
}

fn cli_parse_args(
    program: &Program,
    schema: &CliSchema,
    args: &[String],
) -> Result<Vec<ParsedCliField>, CliInputError> {
    let fields = &schema.fields;
    let mut parsed = Vec::new();
    let mut positionals = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--" {
            positionals.extend(args[index + 1..].iter().cloned());
            break;
        }
        if arg.starts_with("--") && arg.len() > 2 {
            let marker = &arg[2..];
            let (option_name, inline_value) = match marker.split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (marker, None),
            };
            let argument = format!("--{option_name}");
            let Some(field) = cli_field_by_option_name(program, fields, option_name) else {
                return Err(cli_input_error(
                    CliErrorKind::UnknownArgument,
                    argument.clone(),
                    format!("unknown CLI option `{argument}`"),
                ));
            };

            let mut consumed_next = false;
            let raw_value = if let Some(value) = inline_value {
                value.to_string()
            } else if cli_schema_accepts_bare_bool(&field.value) {
                match args
                    .get(index + 1)
                    .filter(|value| !cli_arg_looks_like_option_marker(value))
                {
                    Some(next) if cli_bool_literal(next) || !cli_schema_has_positionals(fields) => {
                        consumed_next = true;
                        next.clone()
                    }
                    _ => "true".to_string(),
                }
            } else if let Some(next) = args
                .get(index + 1)
                .filter(|value| !cli_arg_looks_like_option_marker(value))
            {
                consumed_next = true;
                next.clone()
            } else {
                return Err(cli_input_error(
                    CliErrorKind::MissingValue,
                    argument.clone(),
                    format!("missing value for `{argument}`"),
                ));
            };

            let value = cli_parse_field_value(program, field, &raw_value, &argument)?;
            validate_cli_parsed_field(field, &value, &argument)?;
            merge_parsed_cli_field(
                &mut parsed,
                symbol_name(program, field.name).to_string(),
                value,
                cli_list_item_schema(&field.value).is_some(),
            );

            index += if consumed_next { 2 } else { 1 };
            continue;
        }

        if arg.starts_with('-') && arg.len() > 1 {
            let consumed_next =
                cli_parse_short_token(program, fields, args, index, &mut parsed, true)?;

            index += if consumed_next { 2 } else { 1 };
            continue;
        }

        positionals.push(arg.clone());
        index += 1;
    }

    cli_assign_positionals(program, fields, &positionals, &mut parsed)?;
    Ok(parsed)
}

fn cli_parse_short_token(
    program: &Program,
    fields: &[CliFieldSchema],
    args: &[String],
    index: usize,
    parsed: &mut Vec<ParsedCliField>,
    consume_non_bool_without_positionals: bool,
) -> Result<bool, CliInputError> {
    let marker = &args[index][1..];
    let (short_run, explicit_value) = match marker.split_once('=') {
        Some((name, value)) => (name, Some(value)),
        None => (marker, None),
    };
    if short_run.chars().count() > 1
        && !cli_short_run_starts_with_known_short(program, fields, short_run)
    {
        let argument = format!("-{short_run}");
        return Err(cli_input_error(
            CliErrorKind::UnknownArgument,
            argument.clone(),
            format!("unknown CLI option `{argument}`"),
        ));
    }
    if let Some(value) = explicit_value {
        cli_parse_short_run_with_explicit_value(program, fields, short_run, value, parsed)?;
        return Ok(false);
    }
    if short_run.chars().count() == 1 {
        return cli_parse_exact_short_token(
            program,
            fields,
            args,
            index,
            short_run,
            parsed,
            consume_non_bool_without_positionals,
        );
    }
    cli_parse_compact_short_run(program, fields, args, index, short_run, parsed)
}

fn cli_short_run_starts_with_known_short(
    program: &Program,
    fields: &[CliFieldSchema],
    short_run: &str,
) -> bool {
    let Some(short) = short_run.chars().next() else {
        return false;
    };
    cli_field_by_short_name(program, fields, &short.to_string()).is_some()
}

fn cli_parse_exact_short_token(
    program: &Program,
    fields: &[CliFieldSchema],
    args: &[String],
    index: usize,
    short_name: &str,
    parsed: &mut Vec<ParsedCliField>,
    consume_non_bool_without_positionals: bool,
) -> Result<bool, CliInputError> {
    let argument = format!("-{short_name}");
    let Some(field) = cli_field_by_short_name(program, fields, short_name) else {
        return Err(cli_input_error(
            CliErrorKind::UnknownArgument,
            argument.clone(),
            format!("unknown CLI option `{argument}`"),
        ));
    };

    let mut consumed_next = false;
    let raw_value = if cli_schema_accepts_bare_bool(&field.value) {
        match args
            .get(index + 1)
            .filter(|value| !cli_arg_looks_like_option_marker(value))
        {
            Some(next)
                if cli_bool_literal(next)
                    || (consume_non_bool_without_positionals
                        && !cli_schema_has_positionals(fields)) =>
            {
                consumed_next = true;
                next.clone()
            }
            _ => "true".to_string(),
        }
    } else if let Some(next) = args
        .get(index + 1)
        .filter(|value| !cli_arg_looks_like_option_marker(value))
    {
        consumed_next = true;
        next.clone()
    } else {
        return Err(cli_input_error(
            CliErrorKind::MissingValue,
            argument.clone(),
            format!("missing value for `{argument}`"),
        ));
    };

    cli_parse_and_merge_field(program, field, parsed, &raw_value, &argument)?;
    Ok(consumed_next)
}

fn cli_parse_compact_short_run(
    program: &Program,
    fields: &[CliFieldSchema],
    args: &[String],
    index: usize,
    short_run: &str,
    parsed: &mut Vec<ParsedCliField>,
) -> Result<bool, CliInputError> {
    for (offset, short) in short_run.char_indices() {
        let short_name = short.to_string();
        let argument = format!("-{short_name}");
        let Some(field) = cli_field_by_short_name(program, fields, &short_name) else {
            return Err(cli_input_error(
                CliErrorKind::UnknownArgument,
                argument.clone(),
                format!("unknown CLI option `{argument}`"),
            ));
        };
        let rest_start = offset + short.len_utf8();
        let rest = &short_run[rest_start..];
        if cli_schema_accepts_bare_bool(&field.value) {
            cli_parse_and_merge_field(program, field, parsed, "true", &argument)?;
            continue;
        }
        if !rest.is_empty() {
            cli_parse_and_merge_field(program, field, parsed, rest, &argument)?;
            return Ok(false);
        }
        let Some(next) = args
            .get(index + 1)
            .filter(|value| !cli_arg_looks_like_option_marker(value))
        else {
            return Err(cli_input_error(
                CliErrorKind::MissingValue,
                argument.clone(),
                format!("missing value for `{argument}`"),
            ));
        };
        cli_parse_and_merge_field(program, field, parsed, next, &argument)?;
        return Ok(true);
    }
    Err(cli_input_error(
        CliErrorKind::UnknownArgument,
        "-".to_string(),
        "unknown CLI option `-`".to_string(),
    ))
}

fn cli_parse_short_run_with_explicit_value(
    program: &Program,
    fields: &[CliFieldSchema],
    short_run: &str,
    explicit_value: &str,
    parsed: &mut Vec<ParsedCliField>,
) -> Result<(), CliInputError> {
    let chars = short_run.chars().collect::<Vec<_>>();
    let Some((last, prefix)) = chars.split_last() else {
        return Err(cli_input_error(
            CliErrorKind::UnknownArgument,
            "-".to_string(),
            "unknown CLI option `-`".to_string(),
        ));
    };
    for short in prefix {
        let short_name = short.to_string();
        let argument = format!("-{short_name}");
        let Some(field) = cli_field_by_short_name(program, fields, &short_name) else {
            return Err(cli_input_error(
                CliErrorKind::UnknownArgument,
                argument.clone(),
                format!("unknown CLI option `{argument}`"),
            ));
        };
        if !cli_schema_accepts_bare_bool(&field.value) {
            return Err(cli_input_error(
                CliErrorKind::MissingValue,
                argument.clone(),
                format!("missing value for `{argument}`"),
            ));
        }
        cli_parse_and_merge_field(program, field, parsed, "true", &argument)?;
    }
    let short_name = last.to_string();
    let argument = format!("-{short_name}");
    let Some(field) = cli_field_by_short_name(program, fields, &short_name) else {
        return Err(cli_input_error(
            CliErrorKind::UnknownArgument,
            argument.clone(),
            format!("unknown CLI option `{argument}`"),
        ));
    };
    cli_parse_and_merge_field(program, field, parsed, explicit_value, &argument)
}

fn cli_parse_and_merge_field(
    program: &Program,
    field: &CliFieldSchema,
    parsed: &mut Vec<ParsedCliField>,
    raw_value: &str,
    argument: &str,
) -> Result<(), CliInputError> {
    let value = cli_parse_field_value(program, field, raw_value, argument)?;
    validate_cli_parsed_field(field, &value, argument)?;
    merge_parsed_cli_field(
        parsed,
        symbol_name(program, field.name).to_string(),
        value,
        cli_list_item_schema(&field.value).is_some(),
    );
    Ok(())
}

fn cli_assign_positionals(
    program: &Program,
    fields: &[CliFieldSchema],
    positionals: &[String],
    parsed: &mut Vec<ParsedCliField>,
) -> Result<(), CliInputError> {
    let mut positional_fields = fields
        .iter()
        .filter(|field| field.position.is_some())
        .collect::<Vec<_>>();
    positional_fields.sort_by_key(|field| field.position.unwrap_or(u32::MAX));
    if positional_fields.is_empty() {
        if let Some(raw) = positionals.first() {
            return Err(cli_input_error(
                CliErrorKind::UnknownArgument,
                raw.clone(),
                format!("unexpected CLI positional argument `{raw}`"),
            ));
        }
        return Ok(());
    }

    for field in &positional_fields {
        let Some(position) = field.position else {
            continue;
        };
        let start = position.saturating_sub(1) as usize;
        let argument = cli_positional_argument_label(program, field);
        let name = symbol_name(program, field.name).to_string();
        if let Some(item_schema) = cli_list_item_schema(&field.value) {
            if start >= positionals.len() {
                return Ok(());
            }
            for raw in &positionals[start..] {
                let value = cli_parse_scalar_value(program, &item_schema, raw, &argument)?;
                validate_cli_parsed_field(field, &value, &argument)?;
                merge_parsed_cli_field(parsed, name.clone(), value, true);
            }
            return Ok(());
        }
        let Some(raw) = positionals.get(start) else {
            continue;
        };
        let value = cli_parse_field_value(program, field, raw, &argument)?;
        validate_cli_parsed_field(field, &value, &argument)?;
        merge_parsed_cli_field(parsed, name, value, false);
    }

    let max_consumed = positional_fields
        .last()
        .and_then(|field| field.position)
        .unwrap_or(0) as usize;
    if positionals.len() > max_consumed {
        let raw = &positionals[max_consumed];
        return Err(cli_input_error(
            CliErrorKind::UnknownArgument,
            raw.clone(),
            format!("unexpected CLI positional argument `{raw}`"),
        ));
    }
    Ok(())
}

fn cli_usage_for(
    program: &Program,
    schema: &CliSchema,
    program_name: &str,
    defaults: &Value,
) -> String {
    if cli_schema_is_command(schema) {
        return cli_usage_for_command(program, schema, program_name);
    }

    let positional_fields = cli_visible_positional_fields(&schema.fields);
    if positional_fields.is_empty() {
        return cli_usage_for_options_only(program, schema, program_name, defaults);
    }

    let mut out = cli_usage_line(program, schema, program_name);
    if let Some(about) = schema.about {
        out.push_str("\n\n");
        out.push_str(symbol_name(program, about));
    }
    let default_record = match defaults {
        Value::Record(record) => Some(record),
        _ => None,
    };

    out.push_str("\n\nArguments:");
    for field in positional_fields {
        out.push_str("\n  ");
        out.push_str(&cli_usage_positional_cell(program, field));
        let annotations = cli_positional_usage_annotations(program, field, default_record, false);
        if !annotations.is_empty() {
            out.push_str("  ");
            out.push_str(&annotations.join("; "));
        }
    }

    out.push_str("\n\nOptions:");
    let visible_fields = cli_visible_option_fields(&schema.fields);
    if visible_fields.is_empty() {
        out.push_str("\n  (none)");
        return out;
    }

    for field in visible_fields {
        out.push_str("\n  ");
        out.push_str(&cli_usage_option_cell(
            program,
            field,
            cli_usage_metavar(program, &field.value),
        ));
        if cli_list_item_schema(&field.value).is_some() {
            out.push_str("  repeatable");
        }
        if !field.aliases.is_empty() {
            let aliases = field
                .aliases
                .iter()
                .map(|alias| format!("--{}", symbol_name(program, *alias)))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str("  aliases: ");
            out.push_str(&aliases);
        }
        if let Some(default_text) = default_record
            .and_then(|record| record_field_value(record, symbol_name(program, field.name)))
            .and_then(cli_default_label)
        {
            out.push_str("  default: ");
            out.push_str(&default_text);
        }
        if let Some(help) = field.help {
            out.push_str("  ");
            out.push_str(symbol_name(program, help));
        }
    }
    out
}

fn cli_usage_for_options_only(
    program: &Program,
    schema: &CliSchema,
    program_name: &str,
    defaults: &Value,
) -> String {
    let mut out = format!("Usage: {program_name} [options]");
    if let Some(about) = schema.about {
        out.push_str("\n\n");
        out.push_str(symbol_name(program, about));
    }
    out.push_str("\n\nOptions:");
    let visible_fields = cli_visible_option_fields(&schema.fields);
    if visible_fields.is_empty() {
        out.push_str("\n  (none)");
        return out;
    }

    let default_record = match defaults {
        Value::Record(record) => Some(record),
        _ => None,
    };
    for field in visible_fields {
        out.push_str("\n  ");
        out.push_str(&cli_usage_option_cell(
            program,
            field,
            cli_usage_metavar(program, &field.value),
        ));
        if cli_list_item_schema(&field.value).is_some() {
            out.push_str("  repeatable");
        }
        if !field.aliases.is_empty() {
            let aliases = field
                .aliases
                .iter()
                .map(|alias| format!("--{}", symbol_name(program, *alias)))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str("  aliases: ");
            out.push_str(&aliases);
        }
        if let Some(default_text) = default_record
            .and_then(|record| record_field_value(record, symbol_name(program, field.name)))
            .and_then(cli_default_label)
        {
            out.push_str("  default: ");
            out.push_str(&default_text);
        }
        if let Some(help) = field.help {
            out.push_str("  ");
            out.push_str(symbol_name(program, help));
        }
    }
    out
}

fn cli_usage_for_required(program: &Program, schema: &CliSchema, program_name: &str) -> String {
    if cli_schema_is_command(schema) {
        return cli_usage_for_command(program, schema, program_name);
    }
    if cli_schema_is_wrapper(schema) {
        return cli_usage_for_wrapper_required(program, schema, program_name);
    }

    let positional_fields = cli_visible_positional_fields(&schema.fields);
    if positional_fields.is_empty() {
        return cli_usage_for_required_options_only(program, schema, program_name);
    }

    let mut out = cli_usage_line(program, schema, program_name);
    if let Some(about) = schema.about {
        out.push_str("\n\n");
        out.push_str(symbol_name(program, about));
    }

    out.push_str("\n\nArguments:");
    for field in positional_fields {
        out.push_str("\n  ");
        out.push_str(&cli_usage_positional_cell(program, field));
        let annotations = cli_positional_usage_annotations(program, field, None, true);
        if !annotations.is_empty() {
            out.push_str("  ");
            out.push_str(&annotations.join("; "));
        }
    }

    out.push_str("\n\nOptions:");
    let visible_fields = cli_visible_option_fields(&schema.fields);
    if visible_fields.is_empty() {
        out.push_str("\n  (none)");
        return out;
    }

    for field in visible_fields {
        out.push_str("\n  ");
        out.push_str(&cli_usage_option_cell(
            program,
            field,
            cli_required_usage_metavar(program, &field.value),
        ));

        let annotations = cli_required_usage_annotations(program, field);
        if !annotations.is_empty() {
            out.push_str("  ");
            out.push_str(&annotations.join("; "));
        }
    }
    out
}

fn cli_usage_for_required_options_only(
    program: &Program,
    schema: &CliSchema,
    program_name: &str,
) -> String {
    let mut out = format!("Usage: {program_name} [options]");
    if let Some(about) = schema.about {
        out.push_str("\n\n");
        out.push_str(symbol_name(program, about));
    }
    let visible_fields = cli_visible_option_fields(&schema.fields);
    if visible_fields.is_empty() {
        out.push_str("\n  (none)");
        return out;
    }

    for field in visible_fields {
        out.push_str("\n  ");
        out.push_str(&cli_usage_option_cell(
            program,
            field,
            cli_required_usage_metavar(program, &field.value),
        ));

        let annotations = cli_required_usage_annotations(program, field);
        if !annotations.is_empty() {
            out.push_str("  ");
            out.push_str(&annotations.join("; "));
        }
    }
    out
}

fn cli_usage_for_wrapper_required(
    program: &Program,
    schema: &CliSchema,
    program_name: &str,
) -> String {
    let mut out = format!("Usage: {program_name} [global-options] <command> [args]");
    if let Some(about) = schema.about {
        out.push_str("\n\n");
        out.push_str(symbol_name(program, about));
    }
    if let Some(subcommand) = &schema.subcommand {
        cli_append_command_list(program, &subcommand.schema, &mut out);
    }
    cli_append_wrapper_global_options(program, schema, &mut out, false);
    out
}

fn cli_help_for(
    program: &Program,
    schema: &CliSchema,
    program_name: &str,
    defaults: &Value,
) -> String {
    if cli_schema_is_command(schema) {
        return cli_help_for_command(program, schema, program_name);
    }

    let mut out = cli_usage_line(program, schema, program_name);
    if let Some(about) = schema.about {
        out.push_str("\n\n");
        out.push_str(symbol_name(program, about));
    }
    let default_record = match defaults {
        Value::Record(record) => Some(record),
        _ => None,
    };

    let positional_fields = cli_visible_positional_fields(&schema.fields);
    if !positional_fields.is_empty() {
        out.push_str("\n\nArguments:");
        for field in positional_fields {
            out.push_str("\n  ");
            out.push_str(&cli_usage_positional_cell(program, field));
            let annotations =
                cli_positional_usage_annotations(program, field, default_record, false);
            if !annotations.is_empty() {
                out.push_str("  ");
                out.push_str(&annotations.join("; "));
            }
        }
    }

    out.push_str("\n\nOptions:");
    for field in cli_visible_option_fields(&schema.fields) {
        out.push_str("\n  ");
        out.push_str(&cli_usage_option_cell(
            program,
            field,
            cli_usage_metavar(program, &field.value),
        ));
        if cli_list_item_schema(&field.value).is_some() {
            out.push_str("  repeatable");
        }
        if !field.aliases.is_empty() {
            let aliases = field
                .aliases
                .iter()
                .map(|alias| format!("--{}", symbol_name(program, *alias)))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str("  aliases: ");
            out.push_str(&aliases);
        }
        if let Some(default_text) = default_record
            .and_then(|record| record_field_value(record, symbol_name(program, field.name)))
            .and_then(cli_default_label)
        {
            out.push_str("  default: ");
            out.push_str(&default_text);
        }
        if let Some(help) = field.help {
            out.push_str("  ");
            out.push_str(symbol_name(program, help));
        }
    }
    cli_append_help_option(&mut out);
    out
}

fn cli_help_for_required(program: &Program, schema: &CliSchema, program_name: &str) -> String {
    if cli_schema_is_command(schema) {
        return cli_help_for_command(program, schema, program_name);
    }
    if cli_schema_is_wrapper(schema) {
        return cli_help_for_wrapper_required(program, schema, program_name);
    }

    let mut out = cli_usage_line(program, schema, program_name);
    if let Some(about) = schema.about {
        out.push_str("\n\n");
        out.push_str(symbol_name(program, about));
    }

    let positional_fields = cli_visible_positional_fields(&schema.fields);
    if !positional_fields.is_empty() {
        out.push_str("\n\nArguments:");
        for field in positional_fields {
            out.push_str("\n  ");
            out.push_str(&cli_usage_positional_cell(program, field));
            let annotations = cli_positional_usage_annotations(program, field, None, true);
            if !annotations.is_empty() {
                out.push_str("  ");
                out.push_str(&annotations.join("; "));
            }
        }
    }

    out.push_str("\n\nOptions:");
    for field in cli_visible_option_fields(&schema.fields) {
        out.push_str("\n  ");
        out.push_str(&cli_usage_option_cell(
            program,
            field,
            cli_required_usage_metavar(program, &field.value),
        ));

        let annotations = cli_required_usage_annotations(program, field);
        if !annotations.is_empty() {
            out.push_str("  ");
            out.push_str(&annotations.join("; "));
        }
    }
    cli_append_help_option(&mut out);
    out
}

fn cli_help_for_wrapper_required(
    program: &Program,
    schema: &CliSchema,
    program_name: &str,
) -> String {
    let mut out = format!("Usage: {program_name} [global-options] <command> [args]");
    if let Some(about) = schema.about {
        out.push_str("\n\n");
        out.push_str(symbol_name(program, about));
    }
    if let Some(subcommand) = &schema.subcommand {
        cli_append_command_list(program, &subcommand.schema, &mut out);
    }
    cli_append_wrapper_global_options(program, schema, &mut out, true);
    out
}

fn cli_append_help_option(out: &mut String) {
    out.push_str("\n  -h, --help  Show this help");
}

fn cli_append_wrapper_global_options(
    program: &Program,
    schema: &CliSchema,
    out: &mut String,
    include_help: bool,
) {
    out.push_str("\n\nGlobal Options:");
    let visible_fields = cli_visible_option_fields(&schema.fields);
    if visible_fields.is_empty() && !include_help {
        out.push_str("\n  (none)");
        return;
    }
    for field in visible_fields {
        out.push_str("\n  ");
        out.push_str(&cli_usage_option_cell(
            program,
            field,
            cli_required_usage_metavar(program, &field.value),
        ));

        let annotations = cli_required_usage_annotations(program, field);
        if !annotations.is_empty() {
            out.push_str("  ");
            out.push_str(&annotations.join("; "));
        }
    }
    if include_help {
        cli_append_help_option(out);
    }
}

fn cli_usage_for_command(program: &Program, schema: &CliSchema, program_name: &str) -> String {
    let mut out = format!("Usage: {program_name} <command> [args]");
    if let Some(about) = schema.about {
        out.push_str("\n\n");
        out.push_str(symbol_name(program, about));
    }
    cli_append_command_list(program, schema, &mut out);
    out
}

fn cli_help_for_command(program: &Program, schema: &CliSchema, program_name: &str) -> String {
    let mut out = cli_usage_for_command(program, schema, program_name);
    out.push_str("\n\nOptions:");
    cli_append_help_option(&mut out);
    out
}

fn cli_append_command_list(program: &Program, schema: &CliSchema, out: &mut String) {
    out.push_str("\n\nCommands:");
    let visible_commands = cli_visible_commands(&schema.commands);
    if visible_commands.is_empty() {
        out.push_str("\n  (none)");
        return;
    }
    for command in visible_commands {
        out.push_str("\n  ");
        out.push_str(symbol_name(program, command.command_name));
        if !command.aliases.is_empty() {
            let aliases = command
                .aliases
                .iter()
                .map(|alias| symbol_name(program, *alias))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str("  aliases: ");
            out.push_str(&aliases);
        }
        if let Some(about) = command.about {
            out.push_str("  ");
            out.push_str(symbol_name(program, about));
        }
    }
}

fn cli_usage_line(program: &Program, schema: &CliSchema, program_name: &str) -> String {
    let mut out = format!("Usage: {program_name} [options]");
    for field in cli_visible_positional_fields(&schema.fields) {
        out.push(' ');
        out.push_str(&cli_usage_positional_cell(program, field));
    }
    out
}

fn cli_visible_option_fields(fields: &[CliFieldSchema]) -> Vec<&CliFieldSchema> {
    fields
        .iter()
        .filter(|field| !field.hidden && field.position.is_none())
        .collect()
}

fn cli_visible_positional_fields(fields: &[CliFieldSchema]) -> Vec<&CliFieldSchema> {
    let mut fields = fields
        .iter()
        .filter(|field| field.position.is_some())
        .collect::<Vec<_>>();
    fields.sort_by_key(|field| field.position.unwrap_or(u32::MAX));
    fields
}

fn cli_usage_positional_cell(program: &Program, field: &CliFieldSchema) -> String {
    let label = cli_positional_label(program, field);
    if cli_list_item_schema(&field.value).is_some() {
        format!("[{label}...]")
    } else if cli_required_usage_is_required(&field.value) {
        format!("<{label}>")
    } else {
        format!("[{label}]")
    }
}

fn cli_usage_option_cell(program: &Program, field: &CliFieldSchema, metavar: String) -> String {
    let long = format!("--{}", cli_field_option_name(program, field));
    let mut cell = match cli_field_short_name(program, field) {
        Some(short) => format!("-{short}, {long}"),
        None => long,
    };
    if cli_schema_accepts_bare_bool(&field.value) {
        cell.push_str("[=<Bool>]");
    } else {
        cell.push(' ');
        cell.push_str(&metavar);
    }
    cell
}

fn cli_positional_usage_annotations(
    program: &Program,
    field: &CliFieldSchema,
    default_record: Option<&RecordValue>,
    include_required: bool,
) -> Vec<String> {
    let mut annotations = Vec::new();
    if include_required && cli_required_usage_is_required(&field.value) {
        annotations.push("required".to_string());
    }
    if cli_list_item_schema(&field.value).is_some() {
        annotations.push("repeatable".to_string());
    }
    if let Some(default_text) = default_record
        .and_then(|record| record_field_value(record, symbol_name(program, field.name)))
        .and_then(cli_default_label)
    {
        annotations.push(format!("default: {default_text}"));
    }
    annotations.extend(cli_validation_usage_annotations(&field.validation));
    if let Some(values) = cli_usage_enum_values(program, &field.value) {
        annotations.push(format!("values: {values}"));
    }
    if let Some(help) = field.help {
        annotations.push(symbol_name(program, help).to_string());
    }
    annotations
}

fn cli_required_usage_annotations(program: &Program, field: &CliFieldSchema) -> Vec<String> {
    let mut annotations = Vec::new();
    if cli_required_usage_is_required(&field.value) {
        annotations.push("required".to_string());
    }
    if cli_list_item_schema(&field.value).is_some() {
        annotations.push("repeatable".to_string());
    }
    if !field.aliases.is_empty() {
        let aliases = field
            .aliases
            .iter()
            .map(|alias| format!("--{}", symbol_name(program, *alias)))
            .collect::<Vec<_>>()
            .join(", ");
        annotations.push(format!("aliases: {aliases}"));
    }
    annotations.extend(cli_validation_usage_annotations(&field.validation));
    if let Some(values) = cli_usage_enum_values(program, &field.value) {
        annotations.push(format!("values: {values}"));
    }
    if let Some(help) = field.help {
        annotations.push(symbol_name(program, help).to_string());
    }
    annotations
}

fn cli_required_usage_is_required(schema: &CliValueSchema) -> bool {
    matches!(
        schema,
        CliValueSchema::String | CliValueSchema::Int | CliValueSchema::Enum { .. }
    )
}

fn cli_validation_usage_annotations(validation: &[JsonDecodeValidationRule]) -> Vec<String> {
    let mut min = None;
    let mut max = None;
    let mut min_len = None;
    let mut max_len = None;
    let mut annotations = Vec::new();
    for rule in validation {
        match rule {
            JsonDecodeValidationRule::NonEmpty => annotations.push("non-empty".to_string()),
            JsonDecodeValidationRule::Min(value) => min = Some(*value),
            JsonDecodeValidationRule::Max(value) => max = Some(*value),
            JsonDecodeValidationRule::MinLen(value) => min_len = Some(*value),
            JsonDecodeValidationRule::MaxLen(value) => max_len = Some(*value),
        }
    }
    match (min, max) {
        (Some(min), Some(max)) => annotations.push(format!("range: {min}..{max}")),
        (Some(min), None) => annotations.push(format!("min: {min}")),
        (None, Some(max)) => annotations.push(format!("max: {max}")),
        (None, None) => {}
    }
    match (min_len, max_len) {
        (Some(min), Some(max)) => annotations.push(format!("length: {min}..{max}")),
        (Some(min), None) => annotations.push(format!("min length: {min}")),
        (None, Some(max)) => annotations.push(format!("max length: {max}")),
        (None, None) => {}
    }
    annotations
}

fn cli_usage_enum_values(program: &Program, schema: &CliValueSchema) -> Option<String> {
    match schema {
        CliValueSchema::Enum { variants, .. } | CliValueSchema::EnumList { variants, .. } => Some(
            variants
                .iter()
                .map(|variant| symbol_name(program, variant.tag))
                .collect::<Vec<_>>()
                .join(", "),
        ),
        CliValueSchema::Option(item) => cli_usage_enum_values(program, item),
        _ => None,
    }
}

fn cli_command_by_name<'a>(
    program: &Program,
    commands: &'a [CliCommandVariantSchema],
    name: &str,
) -> Option<&'a CliCommandVariantSchema> {
    commands.iter().find(|command| {
        symbol_name(program, command.command_name) == name
            || command
                .aliases
                .iter()
                .any(|alias| symbol_name(program, *alias) == name)
    })
}

fn cli_visible_commands(commands: &[CliCommandVariantSchema]) -> Vec<&CliCommandVariantSchema> {
    commands.iter().filter(|command| !command.hidden).collect()
}

fn cli_field_by_option_name<'a>(
    program: &Program,
    fields: &'a [CliFieldSchema],
    name: &str,
) -> Option<&'a CliFieldSchema> {
    fields
        .iter()
        .filter(|field| field.position.is_none())
        .find(|field| {
            cli_field_option_name(program, field) == name
                || field
                    .aliases
                    .iter()
                    .any(|alias| symbol_name(program, *alias) == name)
        })
}

fn cli_field_by_short_name<'a>(
    program: &Program,
    fields: &'a [CliFieldSchema],
    name: &str,
) -> Option<&'a CliFieldSchema> {
    fields
        .iter()
        .filter(|field| field.position.is_none())
        .find(|field| cli_field_short_name(program, field) == Some(name))
}

fn cli_field_option_name<'a>(program: &'a Program, field: &CliFieldSchema) -> &'a str {
    symbol_name(program, field.option_name)
}

fn cli_field_short_name<'a>(program: &'a Program, field: &CliFieldSchema) -> Option<&'a str> {
    field.short.map(|short| symbol_name(program, short))
}

fn cli_missing_argument_label(program: &Program, field: &CliFieldSchema) -> String {
    if field.position.is_some() {
        cli_positional_argument_label(program, field)
    } else {
        format!("--{}", cli_field_option_name(program, field))
    }
}

fn cli_positional_argument_label(program: &Program, field: &CliFieldSchema) -> String {
    format!("<{}>", cli_positional_label(program, field))
}

fn cli_positional_label(program: &Program, field: &CliFieldSchema) -> String {
    symbol_name(program, field.name).replace('_', "-")
}

fn cli_arg_looks_like_option_marker(value: &str) -> bool {
    value.starts_with('-') && value.len() > 1
}

fn cli_schema_has_positionals(fields: &[CliFieldSchema]) -> bool {
    fields.iter().any(|field| field.position.is_some())
}

fn cli_bool_literal(value: &str) -> bool {
    matches!(value, "true" | "false")
}

fn cli_schema_accepts_bare_bool(schema: &CliValueSchema) -> bool {
    match schema {
        CliValueSchema::Bool | CliValueSchema::BoolList => true,
        CliValueSchema::Option(item) => matches!(item.as_ref(), CliValueSchema::Bool),
        _ => false,
    }
}

fn cli_list_item_schema(schema: &CliValueSchema) -> Option<CliValueSchema> {
    match schema {
        CliValueSchema::StringList => Some(CliValueSchema::String),
        CliValueSchema::IntList => Some(CliValueSchema::Int),
        CliValueSchema::BoolList => Some(CliValueSchema::Bool),
        CliValueSchema::EnumList {
            type_name,
            package_item,
            variants,
        } => Some(CliValueSchema::Enum {
            type_name: *type_name,
            package_item: *package_item,
            variants: variants.clone(),
        }),
        _ => None,
    }
}

fn cli_synthesized_absent_value(schema: &CliValueSchema) -> Option<Value> {
    match schema {
        CliValueSchema::Bool => Some(Value::Bool(false)),
        CliValueSchema::Option(_) => Some(option_none()),
        CliValueSchema::StringList
        | CliValueSchema::IntList
        | CliValueSchema::BoolList
        | CliValueSchema::EnumList { .. } => Some(Value::List(Vec::new())),
        _ => None,
    }
}

fn cli_parse_field_value(
    program: &Program,
    field: &CliFieldSchema,
    raw: &str,
    argument: &str,
) -> Result<Value, CliInputError> {
    if let Some(item_schema) = cli_list_item_schema(&field.value) {
        return cli_parse_scalar_value(program, &item_schema, raw, argument);
    }
    cli_parse_scalar_value(program, &field.value, raw, argument)
}

fn cli_parse_scalar_value(
    program: &Program,
    schema: &CliValueSchema,
    raw: &str,
    argument: &str,
) -> Result<Value, CliInputError> {
    match schema {
        CliValueSchema::String => Ok(Value::String(raw.to_string())),
        CliValueSchema::Int => raw.parse::<i64>().map(Value::Int).map_err(|error| {
            cli_input_error(
                CliErrorKind::InvalidValue,
                argument.to_string(),
                format!("invalid Int for `{argument}`: {raw} ({error})"),
            )
        }),
        CliValueSchema::Bool => match raw {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(cli_input_error(
                CliErrorKind::InvalidValue,
                argument.to_string(),
                format!("invalid Bool for `{argument}`: {raw}"),
            )),
        },
        CliValueSchema::Option(item) => Ok(option_some(cli_parse_scalar_value(
            program, item, raw, argument,
        )?)),
        CliValueSchema::Enum {
            type_name,
            variants,
            ..
        } => {
            let Some(variant) = variants
                .iter()
                .find(|variant| symbol_name(program, variant.tag) == raw)
            else {
                let accepted = variants
                    .iter()
                    .map(|variant| symbol_name(program, variant.tag))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(cli_input_error(
                    CliErrorKind::InvalidValue,
                    argument.to_string(),
                    format!("invalid enum tag for `{argument}`: {raw}; expected one of {accepted}"),
                ));
            };
            Ok(make_enum_value(program, *type_name, variant.name, None))
        }
        _ => Err(cli_input_error(
            CliErrorKind::UnsupportedTarget,
            argument.to_string(),
            format!("unsupported CLI field schema for `{argument}`"),
        )),
    }
}

fn validate_cli_parsed_field(
    field: &CliFieldSchema,
    value: &Value,
    argument: &str,
) -> Result<(), CliInputError> {
    if field.validation.is_empty() {
        return Ok(());
    }
    if matches!(
        value,
        Value::Enum(EnumValue {
            type_name,
            variant_name,
            ..
        }) if type_name == known_enum::OPTION_NAME && variant_name == known_enum::OPTION_NONE_NAME
    ) {
        return Ok(());
    }
    let value = option_some_payload(value).unwrap_or(value);
    for rule in &field.validation {
        if let Err(error) = validate_json_rule(rule, value, argument) {
            return Err(cli_input_error(
                CliErrorKind::Validation,
                argument.to_string(),
                error.message,
            ));
        }
    }
    Ok(())
}

fn merge_parsed_cli_field(
    parsed: &mut Vec<ParsedCliField>,
    name: String,
    value: Value,
    is_list: bool,
) {
    let Some(existing) = parsed.iter_mut().find(|field| field.name == name) else {
        parsed.push(ParsedCliField {
            name,
            value: if is_list {
                Value::List(vec![value])
            } else {
                value
            },
        });
        return;
    };
    if is_list {
        match &mut existing.value {
            Value::List(items) => items.push(value),
            other => *other = Value::List(vec![value]),
        }
    } else {
        existing.value = value;
    }
}

fn cli_usage_metavar(program: &Program, schema: &CliValueSchema) -> String {
    match schema {
        CliValueSchema::String => "<String>".to_string(),
        CliValueSchema::Int => "<Int>".to_string(),
        CliValueSchema::Bool => "<Bool>".to_string(),
        CliValueSchema::StringList => "<String>".to_string(),
        CliValueSchema::IntList => "<Int>".to_string(),
        CliValueSchema::BoolList => "<Bool>".to_string(),
        CliValueSchema::EnumList { variants, .. } | CliValueSchema::Enum { variants, .. } => {
            let tags = variants
                .iter()
                .map(|variant| symbol_name(program, variant.tag))
                .collect::<Vec<_>>()
                .join("|");
            format!("<{tags}>")
        }
        CliValueSchema::Option(item) => cli_usage_metavar(program, item),
    }
}

fn cli_required_usage_metavar(program: &Program, schema: &CliValueSchema) -> String {
    match schema {
        CliValueSchema::String | CliValueSchema::StringList => "<String>".to_string(),
        CliValueSchema::Int | CliValueSchema::IntList => "<Int>".to_string(),
        CliValueSchema::Bool | CliValueSchema::BoolList => "<Bool>".to_string(),
        CliValueSchema::Enum { type_name, .. } | CliValueSchema::EnumList { type_name, .. } => {
            format!("<{}>", cli_usage_type_name(program, *type_name))
        }
        CliValueSchema::Option(item) => cli_required_usage_metavar(program, item),
    }
}

fn cli_usage_type_name(program: &Program, type_name: Symbol) -> String {
    let name = symbol_name(program, type_name);
    if name.starts_with("__muga_pkg__") || name.starts_with("__muga_mod__") {
        return name.rsplit("__").next().unwrap_or(name).to_string();
    }
    name.to_string()
}

fn cli_default_label(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(format!("{text:?}")),
        Value::Int(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::List(items) => {
            let labels = items
                .iter()
                .filter_map(cli_default_label)
                .collect::<Vec<_>>();
            Some(format!("[{}]", labels.join(", ")))
        }
        Value::Enum(value)
            if value.type_name == known_enum::OPTION_NAME
                && value.variant_name == known_enum::OPTION_NONE_NAME =>
        {
            Some("none".to_string())
        }
        Value::Enum(value)
            if value.type_name == known_enum::OPTION_NAME
                && value.variant_name == known_enum::OPTION_SOME_NAME =>
        {
            value.payload.as_deref().and_then(cli_default_label)
        }
        Value::Enum(value) if value.payload.is_none() => Some(value.variant_name.clone()),
        _ => None,
    }
}

fn typed_json_encode_shape_error(path: &str, expected: &str) -> JsonDataError {
    json_value_error(
        JsonErrorKind::UnexpectedToken,
        format!(
            "expected typed JSON {expected} at path {}",
            json_decode_path_label(path)
        ),
    )
}

fn decode_json_value(
    program: &Program,
    schema: &JsonDecodeSchema,
    value: &Value,
    fallback: &Value,
    path: &str,
) -> Result<Value, JsonDataError> {
    match schema {
        JsonDecodeSchema::String => Ok(Value::String(decode_json_string(value, path)?)),
        JsonDecodeSchema::Int => Ok(Value::Int(decode_json_int(value, path)?)),
        JsonDecodeSchema::Bool => Ok(Value::Bool(decode_json_bool(value, path)?)),
        JsonDecodeSchema::JsonValue => Ok(value.clone()),
        JsonDecodeSchema::StringList => decode_json_scalar_list(value, path, |item, item_path| {
            Ok(Value::String(decode_json_string(item, item_path)?))
        }),
        JsonDecodeSchema::IntList => decode_json_scalar_list(value, path, |item, item_path| {
            Ok(Value::Int(decode_json_int(item, item_path)?))
        }),
        JsonDecodeSchema::BoolList => decode_json_scalar_list(value, path, |item, item_path| {
            Ok(Value::Bool(decode_json_bool(item, item_path)?))
        }),
        JsonDecodeSchema::JsonObjectMap => Ok(Value::Map(decode_json_object_map(value, path)?)),
        JsonDecodeSchema::Option(item) => decode_json_option(program, item, value, fallback, path),
        JsonDecodeSchema::List(item) => decode_json_list(program, item, value, path),
        JsonDecodeSchema::TypedStringMap(item) => {
            decode_json_typed_string_map(program, item, value, path)
        }
        JsonDecodeSchema::Record {
            type_name,
            deny_unknown_fields,
            fields,
            ..
        } => decode_json_record(
            program,
            *type_name,
            *deny_unknown_fields,
            fields,
            value,
            fallback,
            path,
        ),
        JsonDecodeSchema::Enum {
            type_name,
            variants,
            ..
        } => decode_json_enum(program, *type_name, variants, value, Some(fallback), path),
    }
}

fn decode_json_value_required(
    program: &Program,
    schema: &JsonDecodeSchema,
    value: &Value,
    path: &str,
) -> Result<Value, JsonDataError> {
    match schema {
        JsonDecodeSchema::String => Ok(Value::String(decode_json_string(value, path)?)),
        JsonDecodeSchema::Int => Ok(Value::Int(decode_json_int(value, path)?)),
        JsonDecodeSchema::Bool => Ok(Value::Bool(decode_json_bool(value, path)?)),
        JsonDecodeSchema::JsonValue => Ok(value.clone()),
        JsonDecodeSchema::StringList => decode_json_scalar_list(value, path, |item, item_path| {
            Ok(Value::String(decode_json_string(item, item_path)?))
        }),
        JsonDecodeSchema::IntList => decode_json_scalar_list(value, path, |item, item_path| {
            Ok(Value::Int(decode_json_int(item, item_path)?))
        }),
        JsonDecodeSchema::BoolList => decode_json_scalar_list(value, path, |item, item_path| {
            Ok(Value::Bool(decode_json_bool(item, item_path)?))
        }),
        JsonDecodeSchema::JsonObjectMap => Ok(Value::Map(decode_json_object_map(value, path)?)),
        JsonDecodeSchema::Option(item) => decode_json_option_required(program, item, value, path),
        JsonDecodeSchema::List(item) => decode_json_list(program, item, value, path),
        JsonDecodeSchema::TypedStringMap(item) => {
            decode_json_typed_string_map(program, item, value, path)
        }
        JsonDecodeSchema::Record {
            type_name,
            deny_unknown_fields,
            fields,
            ..
        } => decode_json_record_required(
            program,
            *type_name,
            *deny_unknown_fields,
            fields,
            value,
            path,
        ),
        JsonDecodeSchema::Enum {
            type_name,
            variants,
            ..
        } => decode_json_enum(program, *type_name, variants, value, None, path),
    }
}

fn decode_json_record(
    program: &Program,
    type_name: Symbol,
    deny_unknown_fields: bool,
    fields: &[JsonDecodeFieldSchema],
    value: &Value,
    fallback: &Value,
    path: &str,
) -> Result<Value, JsonDataError> {
    let object = decode_json_object_ref(value, path)?;
    if deny_unknown_fields {
        reject_unknown_json_record_fields(program, object, fields, path)?;
    }
    let Value::Record(fallback_record) = fallback else {
        return Err(json_decode_shape_error(path, "Object"));
    };

    let mut decoded_fields = Vec::with_capacity(fields.len());
    for field in fields {
        let name = symbol_name(program, field.name).to_string();
        let (decoded, validation_path) = if let Some((item, matched_key)) =
            json_object_field_for_decode(program, object, field, path)?
        {
            let field_path = append_json_decode_field_path(path, matched_key);
            let field_fallback = record_field_value(fallback_record, &name).unwrap_or(&Value::Unit);
            (
                decode_json_value(program, &field.schema, item, field_fallback, &field_path)?,
                field_path,
            )
        } else {
            let field_path = append_json_decode_field_path(
                path,
                json_decode_wire_name(program, field.name, field.wire_name),
            );
            (
                record_field_value(fallback_record, &name)
                    .cloned()
                    .ok_or_else(|| {
                        json_value_error(
                            JsonErrorKind::UnexpectedToken,
                            format!("fallback record missing field `{name}`"),
                        )
                    })?,
                field_path,
            )
        };
        validate_json_decoded_field(field, &decoded, &validation_path)?;
        decoded_fields.push(RecordFieldValue {
            name,
            value: decoded,
        });
    }

    Ok(Value::Record(RecordValue {
        type_name: symbol_name(program, type_name).to_string(),
        fields: decoded_fields,
    }))
}

fn decode_json_record_required(
    program: &Program,
    type_name: Symbol,
    deny_unknown_fields: bool,
    fields: &[JsonDecodeFieldSchema],
    value: &Value,
    path: &str,
) -> Result<Value, JsonDataError> {
    let object = decode_json_object_ref(value, path)?;
    if deny_unknown_fields {
        reject_unknown_json_record_fields(program, object, fields, path)?;
    }

    let mut decoded_fields = Vec::with_capacity(fields.len());
    for field in fields {
        let name = symbol_name(program, field.name).to_string();
        let (decoded, validation_path) = if let Some((item, matched_key)) =
            json_object_field_for_decode(program, object, field, path)?
        {
            let field_path = append_json_decode_field_path(path, matched_key);
            (
                decode_json_value_required(program, &field.schema, item, &field_path)?,
                field_path,
            )
        } else if matches!(field.schema, JsonDecodeSchema::Option(_)) {
            (
                option_none(),
                append_json_decode_field_path(
                    path,
                    json_decode_wire_name(program, field.name, field.wire_name),
                ),
            )
        } else {
            let wire_name = json_decode_wire_name(program, field.name, field.wire_name);
            let field_path = append_json_decode_field_path(path, wire_name);
            return Err(json_value_error(
                JsonErrorKind::UnexpectedToken,
                format!(
                    "missing required JSON field at path {}",
                    json_decode_path_label(&field_path)
                ),
            ));
        };
        validate_json_decoded_field(field, &decoded, &validation_path)?;
        decoded_fields.push(RecordFieldValue {
            name,
            value: decoded,
        });
    }

    Ok(Value::Record(RecordValue {
        type_name: symbol_name(program, type_name).to_string(),
        fields: decoded_fields,
    }))
}

fn reject_unknown_json_record_fields(
    program: &Program,
    object: &MapValue,
    fields: &[JsonDecodeFieldSchema],
    path: &str,
) -> Result<(), JsonDataError> {
    for entry in &object.entries {
        let MapKey::String(key) = &entry.key else {
            continue;
        };
        let accepted = fields
            .iter()
            .any(|field| json_decode_field_accepts_name(program, field, key));
        if !accepted {
            let field_path = append_json_decode_field_path(path, key);
            return Err(json_value_error(
                JsonErrorKind::UnexpectedToken,
                format!(
                    "unexpected JSON field `{key}` at path {}",
                    json_decode_path_label(&field_path)
                ),
            ));
        }
    }
    Ok(())
}

fn json_object_field_for_decode<'a>(
    program: &Program,
    object: &'a MapValue,
    field: &JsonDecodeFieldSchema,
    path: &str,
) -> Result<Option<(&'a Value, &'a str)>, JsonDataError> {
    let mut matched = None;
    for entry in &object.entries {
        let MapKey::String(key) = &entry.key else {
            continue;
        };
        if !json_decode_field_accepts_name(program, field, key) {
            continue;
        }
        if matched.is_some() {
            let field_name = symbol_name(program, field.name);
            let field_path = append_json_decode_field_path(path, key);
            return Err(json_value_error(
                JsonErrorKind::UnexpectedToken,
                format!(
                    "multiple JSON fields match `{field_name}` at path {}",
                    json_decode_path_label(&field_path)
                ),
            ));
        }
        matched = Some((&entry.value, key.as_str()));
    }
    Ok(matched)
}

fn validate_json_decoded_field(
    field: &JsonDecodeFieldSchema,
    value: &Value,
    path: &str,
) -> Result<(), JsonDataError> {
    if field.validation.is_empty() {
        return Ok(());
    }
    if matches!(
        value,
        Value::Enum(EnumValue {
            type_name,
            variant_name,
            ..
        }) if type_name == known_enum::OPTION_NAME && variant_name == known_enum::OPTION_NONE_NAME
    ) {
        return Ok(());
    }
    let value = option_some_payload(value).unwrap_or(value);
    for rule in &field.validation {
        validate_json_rule(rule, value, path)?;
    }
    Ok(())
}

fn validate_json_rule(
    rule: &JsonDecodeValidationRule,
    value: &Value,
    path: &str,
) -> Result<(), JsonDataError> {
    match rule {
        JsonDecodeValidationRule::NonEmpty => {
            let Value::String(text) = value else {
                return Ok(());
            };
            if text.is_empty() {
                return Err(json_validation_error(path, "expected non-empty String"));
            }
        }
        JsonDecodeValidationRule::Min(limit) => {
            let Value::Int(value) = value else {
                return Ok(());
            };
            if value < limit {
                return Err(json_validation_error(
                    path,
                    format!("expected Int >= {limit}"),
                ));
            }
        }
        JsonDecodeValidationRule::Max(limit) => {
            let Value::Int(value) = value else {
                return Ok(());
            };
            if value > limit {
                return Err(json_validation_error(
                    path,
                    format!("expected Int <= {limit}"),
                ));
            }
        }
        JsonDecodeValidationRule::MinLen(limit) => {
            let Value::String(text) = value else {
                return Ok(());
            };
            if (text.chars().count() as i64) < *limit {
                return Err(json_validation_error(
                    path,
                    format!("expected String length >= {limit}"),
                ));
            }
        }
        JsonDecodeValidationRule::MaxLen(limit) => {
            let Value::String(text) = value else {
                return Ok(());
            };
            if (text.chars().count() as i64) > *limit {
                return Err(json_validation_error(
                    path,
                    format!("expected String length <= {limit}"),
                ));
            }
        }
    }
    Ok(())
}

fn json_validation_error(path: &str, expectation: impl Into<String>) -> JsonDataError {
    json_value_error(
        JsonErrorKind::Validation,
        format!(
            "validation failed at path {}: {}",
            json_decode_path_label(path),
            expectation.into()
        ),
    )
}

fn decode_json_enum(
    program: &Program,
    type_name: Symbol,
    variants: &[JsonDecodeVariantSchema],
    value: &Value,
    fallback: Option<&Value>,
    path: &str,
) -> Result<Value, JsonDataError> {
    let Value::Enum(json_value) = value else {
        return Err(json_decode_shape_error(path, "String or Object"));
    };
    if json_value.type_name != crate::std_package::JSON_VALUE_MANGLED_NAME {
        return Err(json_decode_shape_error(path, "String or Object"));
    }

    match json_value.variant_name.as_str() {
        "String" => {
            let Some(Value::String(tag)) = json_value.payload.as_deref() else {
                return Err(json_decode_shape_error(path, "String"));
            };
            let variant = json_decode_variant_by_name(program, variants, tag)
                .ok_or_else(|| json_decode_unknown_enum_variant_error(path, tag))?;
            if variant.payload.is_some() {
                return Err(json_decode_shape_error(path, "Object"));
            }
            Ok(make_enum_value(program, type_name, variant.name, None))
        }
        "Object" => {
            let Some(Value::Map(map)) = json_value.payload.as_deref() else {
                return Err(json_decode_shape_error(path, "Object"));
            };
            decode_json_enum_object(program, type_name, variants, map, fallback, path)
        }
        _ => Err(json_decode_shape_error(path, "String or Object")),
    }
}

fn decode_json_enum_object(
    program: &Program,
    type_name: Symbol,
    variants: &[JsonDecodeVariantSchema],
    map: &MapValue,
    fallback: Option<&Value>,
    path: &str,
) -> Result<Value, JsonDataError> {
    if map.entries.len() != 1 {
        return Err(json_value_error(
            JsonErrorKind::UnexpectedToken,
            format!(
                "expected single-key JSON Object at path {}",
                json_decode_path_label(path)
            ),
        ));
    }

    let entry = &map.entries[0];
    let MapKey::String(tag) = &entry.key else {
        return Err(json_value_error(
            JsonErrorKind::UnexpectedToken,
            format!(
                "expected JSON object key String at path {}",
                json_decode_path_label(path)
            ),
        ));
    };
    let variant = json_decode_variant_by_name(program, variants, tag)
        .ok_or_else(|| json_decode_unknown_enum_variant_error(path, tag))?;
    let Some(payload_schema) = &variant.payload else {
        return Err(json_decode_shape_error(path, "String"));
    };

    let item_path = append_json_decode_field_path(path, tag);
    let payload_fallback = fallback.and_then(|fallback| {
        json_decode_enum_payload_fallback(program, type_name, variant.name, fallback)
    });
    let payload = match payload_fallback {
        Some(item_fallback) => decode_json_value(
            program,
            payload_schema,
            &entry.value,
            item_fallback,
            &item_path,
        )?,
        None => decode_json_value_required(program, payload_schema, &entry.value, &item_path)?,
    };
    Ok(make_enum_value(
        program,
        type_name,
        variant.name,
        Some(payload),
    ))
}

fn json_decode_variant_by_name<'a>(
    program: &Program,
    variants: &'a [JsonDecodeVariantSchema],
    name: &str,
) -> Option<&'a JsonDecodeVariantSchema> {
    variants
        .iter()
        .find(|variant| json_decode_variant_accepts_name(program, variant, name))
}

fn json_decode_wire_name(
    program: &Program,
    source_name: Symbol,
    wire_name: Option<Symbol>,
) -> &str {
    symbol_name(program, wire_name.unwrap_or(source_name))
}

fn json_decode_field_accepts_name(
    program: &Program,
    field: &JsonDecodeFieldSchema,
    name: &str,
) -> bool {
    json_decode_wire_name(program, field.name, field.wire_name) == name
        || field
            .aliases
            .iter()
            .any(|alias| symbol_name(program, *alias) == name)
}

fn json_decode_variant_accepts_name(
    program: &Program,
    variant: &JsonDecodeVariantSchema,
    name: &str,
) -> bool {
    json_decode_wire_name(program, variant.name, variant.wire_name) == name
        || variant
            .aliases
            .iter()
            .any(|alias| symbol_name(program, *alias) == name)
}

fn json_decode_enum_payload_fallback<'a>(
    program: &Program,
    type_name: Symbol,
    variant_name: Symbol,
    fallback: &'a Value,
) -> Option<&'a Value> {
    let Value::Enum(fallback) = fallback else {
        return None;
    };
    if fallback.type_name != symbol_name(program, type_name)
        || fallback.variant_name != symbol_name(program, variant_name)
    {
        return None;
    }
    fallback.payload.as_deref()
}

fn json_decode_unknown_enum_variant_error(path: &str, tag: &str) -> JsonDataError {
    json_value_error(
        JsonErrorKind::UnexpectedToken,
        format!(
            "unknown JSON enum variant `{tag}` at path {}",
            json_decode_path_label(path)
        ),
    )
}

fn decode_json_string(value: &Value, path: &str) -> Result<String, JsonDataError> {
    match json_value_payload(value, "String", path, "String")? {
        Some(Value::String(text)) => Ok(text.clone()),
        _ => Err(json_decode_shape_error(path, "String")),
    }
}

fn decode_json_int(value: &Value, path: &str) -> Result<i64, JsonDataError> {
    match json_value_payload(value, "Number", path, "Int")? {
        Some(number) => {
            json_number_as_int(number.clone()).map_err(|_| json_decode_shape_error(path, "Int"))
        }
        _ => Err(json_decode_shape_error(path, "Int")),
    }
}

fn decode_json_bool(value: &Value, path: &str) -> Result<bool, JsonDataError> {
    match json_value_payload(value, "Bool", path, "Bool")? {
        Some(Value::Bool(value)) => Ok(*value),
        _ => Err(json_decode_shape_error(path, "Bool")),
    }
}

fn decode_json_scalar_list<F>(
    value: &Value,
    path: &str,
    mut decode_item: F,
) -> Result<Value, JsonDataError>
where
    F: FnMut(&Value, &str) -> Result<Value, JsonDataError>,
{
    let items = match json_value_payload(value, "Array", path, "Array")? {
        Some(Value::List(items)) => items,
        _ => return Err(json_decode_shape_error(path, "Array")),
    };
    let mut decoded = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let item_path = append_json_decode_index_path(path, index);
        decoded.push(decode_item(item, &item_path)?);
    }
    Ok(Value::List(decoded))
}

fn decode_json_option(
    program: &Program,
    item_schema: &JsonDecodeSchema,
    value: &Value,
    fallback: &Value,
    path: &str,
) -> Result<Value, JsonDataError> {
    if is_json_null(value) {
        return Ok(option_none());
    }

    let decoded = match option_some_payload(fallback) {
        Some(item_fallback) => decode_json_value(program, item_schema, value, item_fallback, path)?,
        None => decode_json_value_required(program, item_schema, value, path)?,
    };
    Ok(option_some(decoded))
}

fn decode_json_option_required(
    program: &Program,
    item_schema: &JsonDecodeSchema,
    value: &Value,
    path: &str,
) -> Result<Value, JsonDataError> {
    if is_json_null(value) {
        return Ok(option_none());
    }
    Ok(option_some(decode_json_value_required(
        program,
        item_schema,
        value,
        path,
    )?))
}

fn decode_json_list(
    program: &Program,
    item_schema: &JsonDecodeSchema,
    value: &Value,
    path: &str,
) -> Result<Value, JsonDataError> {
    let items = match json_value_payload(value, "Array", path, "Array")? {
        Some(Value::List(items)) => items,
        _ => return Err(json_decode_shape_error(path, "Array")),
    };
    let mut decoded = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let item_path = append_json_decode_index_path(path, index);
        decoded.push(decode_json_value_required(
            program,
            item_schema,
            item,
            &item_path,
        )?);
    }
    Ok(Value::List(decoded))
}

fn decode_json_typed_string_map(
    program: &Program,
    item_schema: &JsonDecodeSchema,
    value: &Value,
    path: &str,
) -> Result<Value, JsonDataError> {
    let map = decode_json_object_ref(value, path)?;
    let mut entries = Vec::with_capacity(map.entries.len());
    for entry in &map.entries {
        let MapKey::String(key) = &entry.key else {
            return Err(json_value_error(
                JsonErrorKind::UnexpectedToken,
                format!(
                    "expected JSON object key String at path {}",
                    json_decode_path_label(path)
                ),
            ));
        };
        let item_path = append_json_decode_field_path(path, key);
        entries.push(MapEntryValue {
            key: MapKey::String(key.clone()),
            value: decode_json_value_required(program, item_schema, &entry.value, &item_path)?,
        });
    }
    Ok(Value::Map(MapValue { entries }))
}

fn decode_json_object_map(value: &Value, path: &str) -> Result<MapValue, JsonDataError> {
    Ok(decode_json_object_ref(value, path)?.clone())
}

fn decode_json_object_ref<'a>(value: &'a Value, path: &str) -> Result<&'a MapValue, JsonDataError> {
    match json_value_payload(value, "Object", path, "Object")? {
        Some(Value::Map(map)) => Ok(map),
        _ => Err(json_decode_shape_error(path, "Object")),
    }
}

fn is_json_null(value: &Value) -> bool {
    matches!(
        value,
        Value::Enum(EnumValue {
            type_name,
            variant_name,
            ..
        }) if type_name == crate::std_package::JSON_VALUE_MANGLED_NAME && variant_name == "Null"
    )
}

fn option_some_payload(value: &Value) -> Option<&Value> {
    let Value::Enum(value) = value else {
        return None;
    };
    if value.type_name != known_enum::OPTION_NAME
        || value.variant_name != known_enum::OPTION_SOME_NAME
    {
        return None;
    }
    value.payload.as_deref()
}

fn is_option_none(value: &Value) -> bool {
    matches!(
        value,
        Value::Enum(EnumValue {
            type_name,
            variant_name,
            ..
        }) if type_name == known_enum::OPTION_NAME && variant_name == known_enum::OPTION_NONE_NAME
    )
}

fn json_value_payload<'a>(
    value: &'a Value,
    variant: &str,
    path: &str,
    expected: &str,
) -> Result<Option<&'a Value>, JsonDataError> {
    let Value::Enum(value) = value else {
        return Err(json_decode_shape_error(path, expected));
    };
    if value.type_name != crate::std_package::JSON_VALUE_MANGLED_NAME {
        return Err(json_decode_shape_error(path, expected));
    }
    if value.variant_name == variant {
        Ok(value.payload.as_deref())
    } else {
        Err(json_decode_shape_error(path, expected))
    }
}

fn record_field_value<'a>(record: &'a RecordValue, name: &str) -> Option<&'a Value> {
    record
        .fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| &field.value)
}

fn expect_path_value(value: &Value, span: Span, context: &str) -> Result<String, Vec<Diagnostic>> {
    let Value::Record(record) = value else {
        return Err(vec![Diagnostic::new(
            "R014",
            format!("`{context}` expects path::Path as its first argument"),
            span,
        )]);
    };
    if record.type_name != crate::std_package::PATH_MANGLED_NAME {
        return Err(vec![Diagnostic::new(
            "R014",
            format!("`{context}` expects path::Path as its first argument"),
            span,
        )]);
    }
    match record_field_value(record, "text") {
        Some(Value::String(path)) => Ok(path.clone()),
        _ => Err(vec![Diagnostic::new(
            "R014",
            format!("`{context}` received an invalid path::Path value"),
            span,
        )]),
    }
}

fn expect_string_value(
    value: &Value,
    span: Span,
    context: &str,
) -> Result<String, Vec<Diagnostic>> {
    let Value::String(text) = value else {
        return Err(vec![Diagnostic::new(
            "R014",
            format!("`{context}` expects String as its first argument"),
            span,
        )]);
    };
    Ok(text.clone())
}

fn expect_string_list_value(
    value: &Value,
    span: Span,
    context: &str,
) -> Result<Vec<String>, Vec<Diagnostic>> {
    let Value::List(items) = value else {
        return Err(vec![Diagnostic::new(
            "R014",
            format!("`{context}` expects List[String] as its first argument"),
            span,
        )]);
    };
    let mut strings = Vec::with_capacity(items.len());
    for item in items {
        let Value::String(text) = item else {
            return Err(vec![Diagnostic::new(
                "R014",
                format!("`{context}` expects List[String] as its first argument"),
                span,
            )]);
        };
        strings.push(text.clone());
    }
    Ok(strings)
}

fn json_decode_shape_error(path: &str, expected: &str) -> JsonDataError {
    json_value_error(
        JsonErrorKind::UnexpectedToken,
        format!(
            "expected JSON {expected} at path {}",
            json_decode_path_label(path)
        ),
    )
}

fn json_decode_path_label(path: &str) -> &str {
    if path.is_empty() { "<root>" } else { path }
}

fn append_json_decode_field_path(path: &str, field: &str) -> String {
    format!("{path}.{field}")
}

fn append_json_decode_index_path(path: &str, index: usize) -> String {
    format!("{path}[{index}]")
}

impl RuntimeHandles {
    fn open_std_fs_file(&mut self, path: &str) -> io::Result<RuntimeHandleValue> {
        let file = fs::File::open(path)?;
        Ok(self.push_std_fs_file(path, file, StdFsFileMode::Read))
    }

    fn create_std_fs_file(&mut self, path: &str) -> io::Result<RuntimeHandleValue> {
        let file = fs::File::create(path)?;
        Ok(self.push_std_fs_file(path, file, StdFsFileMode::Write))
    }

    fn append_std_fs_file(&mut self, path: &str) -> io::Result<RuntimeHandleValue> {
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(self.push_std_fs_file(path, file, StdFsFileMode::Append))
    }

    fn push_std_fs_file(
        &mut self,
        path: &str,
        file: fs::File,
        mode: StdFsFileMode,
    ) -> RuntimeHandleValue {
        let slot = self.std_fs_files.len();
        let generation = 0;
        self.std_fs_files.push(StdFsFileSlot::Open {
            path: path.to_string(),
            file,
            mode,
            generation,
        });
        RuntimeHandleValue {
            family: STD_FS_FILE_HANDLE_FAMILY,
            slot,
            generation,
        }
    }

    fn read_std_fs_file_text(
        &mut self,
        handle: &RuntimeHandleValue,
        span: Span,
    ) -> Result<(String, io::Result<String>), Vec<Diagnostic>> {
        let slot = self.std_fs_file_slot_mut(handle, span)?;
        let StdFsFileSlot::Open {
            path, file, mode, ..
        } = slot
        else {
            unreachable!("std_fs_file_slot_mut returns only open slots");
        };
        let path = path.clone();
        if !mode.can_read() {
            return Ok((
                path,
                Err(wrong_mode_io_error("std::fs::File is not open for reading")),
            ));
        }
        if let Err(error) = io::Seek::seek(file, io::SeekFrom::Start(0)) {
            return Ok((path, Err(error)));
        }
        let mut text = String::new();
        match io::Read::read_to_string(file, &mut text) {
            Ok(_) => Ok((path, Ok(text))),
            Err(error) => Ok((path, Err(error))),
        }
    }

    fn write_std_fs_file_text(
        &mut self,
        handle: &RuntimeHandleValue,
        text: &str,
        span: Span,
    ) -> Result<(String, io::Result<()>), Vec<Diagnostic>> {
        let slot = self.std_fs_file_slot_mut(handle, span)?;
        let StdFsFileSlot::Open {
            path, file, mode, ..
        } = slot
        else {
            unreachable!("std_fs_file_slot_mut returns only open slots");
        };
        let path = path.clone();
        if !mode.can_write() {
            return Ok((
                path,
                Err(wrong_mode_io_error("std::fs::File is not open for writing")),
            ));
        }
        match io::Write::write_all(file, text.as_bytes()) {
            Ok(()) => Ok((path, Ok(()))),
            Err(error) => Ok((path, Err(error))),
        }
    }

    fn flush_std_fs_file(
        &mut self,
        handle: &RuntimeHandleValue,
        span: Span,
    ) -> Result<(String, io::Result<()>), Vec<Diagnostic>> {
        let slot = self.std_fs_file_slot_mut(handle, span)?;
        let StdFsFileSlot::Open {
            path, file, mode, ..
        } = slot
        else {
            unreachable!("std_fs_file_slot_mut returns only open slots");
        };
        let path = path.clone();
        if !mode.can_write() {
            return Ok((
                path,
                Err(wrong_mode_io_error("std::fs::File is not open for writing")),
            ));
        }
        match io::Write::flush(file) {
            Ok(()) => Ok((path, Ok(()))),
            Err(error) => Ok((path, Err(error))),
        }
    }

    fn close_std_fs_file(
        &mut self,
        handle: &RuntimeHandleValue,
        span: Span,
    ) -> Result<(String, io::Result<()>), Vec<Diagnostic>> {
        if handle.family != STD_FS_FILE_HANDLE_FAMILY {
            return Err(vec![invalid_runtime_handle_diagnostic(
                "wrong family for std::fs::File handle",
                span,
            )]);
        }
        let Some(slot) = self.std_fs_files.get_mut(handle.slot) else {
            return Err(vec![invalid_runtime_handle_diagnostic(
                "stale slot for std::fs::File handle",
                span,
            )]);
        };
        match slot {
            StdFsFileSlot::Open {
                path,
                file,
                mode,
                generation,
            } if *generation == handle.generation => {
                let path = path.clone();
                let flush_result = if mode.can_write() {
                    io::Write::flush(file)
                } else {
                    Ok(())
                };
                let next_generation = generation.saturating_add(1);
                *slot = StdFsFileSlot::Closed {
                    generation: next_generation,
                };
                Ok((path, flush_result))
            }
            StdFsFileSlot::Open { .. } => Err(vec![invalid_runtime_handle_diagnostic(
                "stale slot for std::fs::File handle",
                span,
            )]),
            StdFsFileSlot::Closed { generation } => {
                let _ = *generation;
                Err(vec![invalid_runtime_handle_diagnostic(
                    "std::fs::File handle was already closed",
                    span,
                )])
            }
        }
    }

    fn std_fs_file_slot_mut(
        &mut self,
        handle: &RuntimeHandleValue,
        span: Span,
    ) -> Result<&mut StdFsFileSlot, Vec<Diagnostic>> {
        if handle.family != STD_FS_FILE_HANDLE_FAMILY {
            return Err(vec![invalid_runtime_handle_diagnostic(
                "wrong family for std::fs::File handle",
                span,
            )]);
        }
        let Some(slot) = self.std_fs_files.get_mut(handle.slot) else {
            return Err(vec![invalid_runtime_handle_diagnostic(
                "stale slot for std::fs::File handle",
                span,
            )]);
        };
        match slot {
            StdFsFileSlot::Open { generation, .. } if *generation == handle.generation => Ok(slot),
            StdFsFileSlot::Open { .. } => Err(vec![invalid_runtime_handle_diagnostic(
                "stale slot for std::fs::File handle",
                span,
            )]),
            StdFsFileSlot::Closed { generation } if *generation == handle.generation => {
                Err(vec![invalid_runtime_handle_diagnostic(
                    "std::fs::File handle was already closed",
                    span,
                )])
            }
            StdFsFileSlot::Closed { .. } => Err(vec![invalid_runtime_handle_diagnostic(
                "stale slot for std::fs::File handle",
                span,
            )]),
        }
    }
}

fn invalid_runtime_handle_diagnostic(message: &'static str, span: Span) -> Diagnostic {
    Diagnostic::new("R022", message, span)
}

fn wrong_mode_io_error(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn call_builtin(
    builtin: BuiltinId,
    args: Vec<Value>,
    env: &EnvRef,
    span: Span,
) -> Result<Value, Vec<Diagnostic>> {
    match builtin {
        BuiltinId::Print => {
            let value = expect_one_arg(args, span)?;
            match &value {
                Value::Int(_) | Value::Bool(_) | Value::String(_) => {
                    env.borrow()
                        .output
                        .borrow_mut()
                        .push_str(&value.to_string());
                    Ok(value)
                }
                Value::Unit
                | Value::Bytes(_)
                | Value::List(_)
                | Value::Map(_)
                | Value::Enum(_)
                | Value::Record(_)
                | Value::RuntimeHandle(_)
                | Value::Function(_)
                | Value::Builtin(_) => Err(vec![Diagnostic::new(
                    "R014",
                    "`print` accepts only Int, Bool, or String",
                    span,
                )]),
            }
        }
        BuiltinId::Println => {
            let value = expect_one_arg(args, span)?;
            match &value {
                Value::Int(_) | Value::Bool(_) | Value::String(_) => {
                    let borrowed_env = env.borrow();
                    let mut output = borrowed_env.output.borrow_mut();
                    output.push_str(&value.to_string());
                    output.push('\n');
                    Ok(value)
                }
                Value::Unit
                | Value::Bytes(_)
                | Value::List(_)
                | Value::Map(_)
                | Value::Enum(_)
                | Value::Record(_)
                | Value::RuntimeHandle(_)
                | Value::Function(_)
                | Value::Builtin(_) => Err(vec![Diagnostic::new(
                    "R014",
                    "`println` accepts only Int, Bool, or String",
                    span,
                )]),
            }
        }
        BuiltinId::Eprint => {
            let value = expect_one_arg(args, span)?;
            match &value {
                Value::Int(_) | Value::Bool(_) | Value::String(_) => {
                    env.borrow()
                        .stderr
                        .borrow_mut()
                        .push_str(&value.to_string());
                    Ok(value)
                }
                Value::Unit
                | Value::Bytes(_)
                | Value::List(_)
                | Value::Map(_)
                | Value::Enum(_)
                | Value::Record(_)
                | Value::RuntimeHandle(_)
                | Value::Function(_)
                | Value::Builtin(_) => Err(vec![Diagnostic::new(
                    "R014",
                    "`eprint` accepts only Int, Bool, or String",
                    span,
                )]),
            }
        }
        BuiltinId::Eprintln => {
            let value = expect_one_arg(args, span)?;
            match &value {
                Value::Int(_) | Value::Bool(_) | Value::String(_) => {
                    let borrowed_env = env.borrow();
                    let mut stderr = borrowed_env.stderr.borrow_mut();
                    stderr.push_str(&value.to_string());
                    stderr.push('\n');
                    Ok(value)
                }
                Value::Unit
                | Value::Bytes(_)
                | Value::List(_)
                | Value::Map(_)
                | Value::Enum(_)
                | Value::Record(_)
                | Value::RuntimeHandle(_)
                | Value::Function(_)
                | Value::Builtin(_) => Err(vec![Diagnostic::new(
                    "R014",
                    "`eprintln` accepts only Int, Bool, or String",
                    span,
                )]),
            }
        }
        BuiltinId::Len => {
            let value = expect_one_arg(args, span)?;
            match value {
                Value::List(items) => Ok(Value::Int(items.len() as i64)),
                Value::Map(map) => Ok(Value::Int(map.entries.len() as i64)),
                _ => Err(vec![Diagnostic::new(
                    "R014",
                    "`len` expects List[T] or Map[K, V] as its first argument",
                    span,
                )]),
            }
        }
        BuiltinId::IsEmpty => {
            let value = expect_one_arg(args, span)?;
            match value {
                Value::String(text) => Ok(Value::Bool(text.is_empty())),
                Value::List(items) => Ok(Value::Bool(items.is_empty())),
                Value::Map(map) => Ok(Value::Bool(map.entries.is_empty())),
                _ => Err(vec![Diagnostic::new(
                    "R014",
                    "`is_empty` expects String, List[T], or Map[K, V] as its first argument",
                    span,
                )]),
            }
        }
        BuiltinId::Push => {
            let (list, value) = expect_two_args(args, span)?;
            match list {
                Value::List(mut items) => {
                    items.push(value);
                    Ok(Value::List(items))
                }
                _ => Err(vec![Diagnostic::new(
                    "R014",
                    "`push` expects List[T] as its first argument",
                    span,
                )]),
            }
        }
        BuiltinId::Get => {
            let (collection, key_or_index) = expect_two_args(args, span)?;
            match collection {
                Value::List(items) => {
                    let Value::Int(index) = key_or_index else {
                        return Err(vec![Diagnostic::new(
                            "R014",
                            "`get` expects Int as its second argument for List[T]",
                            span,
                        )]);
                    };
                    if index < 0 {
                        return Ok(option_none());
                    }
                    match items.get(index as usize).cloned() {
                        Some(value) => Ok(option_some(value)),
                        None => Ok(option_none()),
                    }
                }
                Value::Map(map) => {
                    let key = map_key(key_or_index, span, "get")?;
                    match map.entries.iter().find(|entry| entry.key == key) {
                        Some(entry) => Ok(option_some(entry.value.clone())),
                        None => Ok(option_none()),
                    }
                }
                _ => Err(vec![Diagnostic::new(
                    "R014",
                    "`get` expects List[T] or Map[K, V] as its first argument",
                    span,
                )]),
            }
        }
        BuiltinId::Set => {
            let (list, index, value) = expect_three_args(args, span)?;
            let Value::List(mut items) = list else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`set` expects List[T] as its first argument",
                    span,
                )]);
            };
            let index = list_index(index, items.len(), span)?;
            items[index] = value;
            Ok(Value::List(items))
        }
        BuiltinId::MapEmpty => {
            expect_no_args(args, span)?;
            Ok(Value::Map(MapValue {
                entries: Vec::new(),
            }))
        }
        BuiltinId::Contains => {
            let (collection, key_or_needle) = expect_two_args(args, span)?;
            match collection {
                Value::String(text) => {
                    let Value::String(needle) = key_or_needle else {
                        return Err(vec![Diagnostic::new(
                            "R014",
                            "`contains` expects String as its second argument for String",
                            span,
                        )]);
                    };
                    Ok(Value::Bool(text.contains(&needle)))
                }
                Value::Map(map) => {
                    let key = map_key(key_or_needle, span, "contains")?;
                    Ok(Value::Bool(
                        map.entries.iter().any(|entry| entry.key == key),
                    ))
                }
                _ => Err(vec![Diagnostic::new(
                    "R014",
                    "`contains` expects String or Map[K, V] as its first argument",
                    span,
                )]),
            }
        }
        BuiltinId::Insert => {
            let (map, key, value) = expect_three_args(args, span)?;
            let Value::Map(mut map) = map else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`insert` expects Map[K, V] as its first argument",
                    span,
                )]);
            };
            let key = map_key(key, span, "insert")?;
            if let Some(entry) = map.entries.iter_mut().find(|entry| entry.key == key) {
                entry.value = value;
            } else {
                map.entries.push(MapEntryValue { key, value });
            }
            Ok(Value::Map(map))
        }
        BuiltinId::Remove => {
            let (map, key) = expect_two_args(args, span)?;
            let Value::Map(mut map) = map else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`remove` expects Map[K, V] as its first argument",
                    span,
                )]);
            };
            let key = map_key(key, span, "remove")?;
            map.entries.retain(|entry| entry.key != key);
            Ok(Value::Map(map))
        }
        BuiltinId::StdMapKeys => {
            let value = expect_one_arg(args, span)?;
            let Value::Map(map) = value else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_map_keys` expects Map[K, V] as its first argument",
                    span,
                )]);
            };
            Ok(Value::List(
                map.entries
                    .into_iter()
                    .map(|entry| entry.key.into_value())
                    .collect(),
            ))
        }
        BuiltinId::StdMapValues => {
            let value = expect_one_arg(args, span)?;
            let Value::Map(map) = value else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_map_values` expects Map[K, V] as its first argument",
                    span,
                )]);
            };
            Ok(Value::List(
                map.entries.into_iter().map(|entry| entry.value).collect(),
            ))
        }
        BuiltinId::StdJsonParse => {
            let text = expect_string_arg(args, span, "__muga_std_json_parse")?;
            Ok(match JsonParser::new(&text).parse() {
                Ok(value) => result_ok(value),
                Err(error) => result_err(json_error_value(error)),
            })
        }
        BuiltinId::StdJsonEncode => {
            let value = expect_one_arg(args, span)?;
            Ok(match encode_json_value(&value, 0) {
                Ok(text) => result_ok(Value::String(text)),
                Err(error) => result_err(json_error_value(error)),
            })
        }
        BuiltinId::StdJsonNumberAsInt => {
            let number = expect_one_arg(args, span)?;
            Ok(match json_number_as_int(number) {
                Ok(value) => result_ok(Value::Int(value)),
                Err(error) => result_err(json_error_value(error)),
            })
        }
        BuiltinId::Trim => {
            let value = expect_one_arg(args, span)?;
            let Value::String(text) = value else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`trim` expects String as its first argument",
                    span,
                )]);
            };
            Ok(Value::String(text.trim().to_string()))
        }
        BuiltinId::CharCount => {
            let value = expect_one_arg(args, span)?;
            let Value::String(text) = value else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`char_count` expects String as its first argument",
                    span,
                )]);
            };
            Ok(Value::Int(text.chars().count() as i64))
        }
        BuiltinId::ByteLen => {
            let value = expect_one_arg(args, span)?;
            let Value::String(text) = value else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`byte_len` expects String as its first argument",
                    span,
                )]);
            };
            Ok(Value::Int(text.len() as i64))
        }
        BuiltinId::StartsWith => {
            let (text, prefix) = expect_two_args(args, span)?;
            let Value::String(text) = text else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`starts_with` expects String as its first argument",
                    span,
                )]);
            };
            let Value::String(prefix) = prefix else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`starts_with` expects String as its second argument",
                    span,
                )]);
            };
            Ok(Value::Bool(text.starts_with(&prefix)))
        }
        BuiltinId::EndsWith => {
            let (text, suffix) = expect_two_args(args, span)?;
            let Value::String(text) = text else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`ends_with` expects String as its first argument",
                    span,
                )]);
            };
            let Value::String(suffix) = suffix else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`ends_with` expects String as its second argument",
                    span,
                )]);
            };
            Ok(Value::Bool(text.ends_with(&suffix)))
        }
        BuiltinId::Replace => {
            let (text, old, new) = expect_three_args(args, span)?;
            let Value::String(text) = text else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`replace` expects String as its first argument",
                    span,
                )]);
            };
            let Value::String(old) = old else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`replace` expects String as its second argument",
                    span,
                )]);
            };
            let Value::String(new) = new else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`replace` expects String as its third argument",
                    span,
                )]);
            };
            if old.is_empty() {
                Ok(Value::String(text))
            } else {
                Ok(Value::String(text.replace(&old, &new)))
            }
        }
        BuiltinId::Split => {
            let (text, separator) = expect_two_args(args, span)?;
            let Value::String(text) = text else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`split` expects String as its first argument",
                    span,
                )]);
            };
            let Value::String(separator) = separator else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`split` expects String as its second argument",
                    span,
                )]);
            };
            let parts = if separator.is_empty() {
                vec![Value::String(text)]
            } else {
                text.split(&separator)
                    .map(|part| Value::String(part.to_string()))
                    .collect()
            };
            Ok(Value::List(parts))
        }
        BuiltinId::Concat => {
            let (left, right) = expect_two_args(args, span)?;
            let Value::String(mut left) = left else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`concat` expects String as its first argument",
                    span,
                )]);
            };
            let Value::String(right) = right else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`concat` expects String as its second argument",
                    span,
                )]);
            };
            left.push_str(&right);
            Ok(Value::String(left))
        }
        BuiltinId::SliceChars => {
            let (text, start, count) = expect_three_args(args, span)?;
            let Value::String(text) = text else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`slice_chars` expects String as its first argument",
                    span,
                )]);
            };
            let Value::Int(start) = start else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`slice_chars` expects Int as its second argument",
                    span,
                )]);
            };
            let Value::Int(count) = count else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`slice_chars` expects Int as its third argument",
                    span,
                )]);
            };
            let Some(end) = start.checked_add(count) else {
                return Ok(result_err(Value::String("invalid slice range".to_string())));
            };
            let char_count = text.chars().count() as i64;
            if start < 0 || count < 0 || end > char_count {
                return Ok(result_err(Value::String("invalid slice range".to_string())));
            }
            let slice = text
                .chars()
                .skip(start as usize)
                .take(count as usize)
                .collect();
            Ok(result_ok(Value::String(slice)))
        }
        BuiltinId::ToString => {
            let value = expect_one_arg(args, span)?;
            match value {
                Value::Int(value) => Ok(Value::String(value.to_string())),
                Value::Bool(value) => Ok(Value::String(value.to_string())),
                Value::String(value) => Ok(Value::String(value)),
                _ => Err(vec![Diagnostic::new(
                    "R014",
                    "`to_string` accepts only Int, Bool, or String",
                    span,
                )]),
            }
        }
        BuiltinId::ParseInt => {
            let value = expect_one_arg(args, span)?;
            let Value::String(text) = value else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`parse_int` expects String as its first argument",
                    span,
                )]);
            };
            match text.parse::<i64>() {
                Ok(value) => Ok(result_ok(Value::Int(value))),
                Err(_) => Ok(result_err(Value::String("invalid Int".to_string()))),
            }
        }
        BuiltinId::ParseBool => {
            let value = expect_one_arg(args, span)?;
            let Value::String(text) = value else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`parse_bool` expects String as its first argument",
                    span,
                )]);
            };
            match text.as_str() {
                "true" => Ok(result_ok(Value::Bool(true))),
                "false" => Ok(result_ok(Value::Bool(false))),
                _ => Ok(result_err(Value::String("invalid Bool".to_string()))),
            }
        }
        BuiltinId::StdPathJoin => {
            let (base, child) = expect_two_string_args(args, span, "__muga_std_path_join")?;
            Ok(Value::String(
                std::path::Path::new(&base)
                    .join(child)
                    .to_string_lossy()
                    .into_owned(),
            ))
        }
        BuiltinId::StdPathNormalize => {
            let path = expect_string_arg(args, span, "__muga_std_path_normalize")?;
            Ok(Value::String(normalize_path_lexically_for_std(&path)))
        }
        BuiltinId::StdPathFileName => {
            let path = expect_string_arg(args, span, "__muga_std_path_file_name")?;
            Ok(std::path::Path::new(&path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| option_some(Value::String(name.to_string())))
                .unwrap_or_else(option_none))
        }
        BuiltinId::StdPathWithFileName => {
            let (path, file_name) =
                expect_two_string_args(args, span, "__muga_std_path_with_file_name")?;
            Ok(Value::String(
                std::path::Path::new(&path)
                    .with_file_name(file_name)
                    .to_string_lossy()
                    .into_owned(),
            ))
        }
        BuiltinId::StdPathParent => {
            let path = expect_string_arg(args, span, "__muga_std_path_parent")?;
            Ok(std::path::Path::new(&path)
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .and_then(|parent| parent.to_str())
                .map(|parent| option_some(Value::String(parent.to_string())))
                .unwrap_or_else(option_none))
        }
        BuiltinId::StdPathStripPrefix => {
            let (path, base) = expect_two_string_args(args, span, "__muga_std_path_strip_prefix")?;
            Ok(std::path::Path::new(&path)
                .strip_prefix(std::path::Path::new(&base))
                .ok()
                .and_then(|path| path.to_str())
                .map(|path| option_some(Value::String(path.to_string())))
                .unwrap_or_else(option_none))
        }
        BuiltinId::StdPathExtension => {
            let path = expect_string_arg(args, span, "__muga_std_path_extension")?;
            Ok(std::path::Path::new(&path)
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| option_some(Value::String(extension.to_string())))
                .unwrap_or_else(option_none))
        }
        BuiltinId::StdPathFileStem => {
            let path = expect_string_arg(args, span, "__muga_std_path_file_stem")?;
            Ok(std::path::Path::new(&path)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(|stem| option_some(Value::String(stem.to_string())))
                .unwrap_or_else(option_none))
        }
        BuiltinId::StdPathWithExtension => {
            let (path, extension) =
                expect_two_string_args(args, span, "__muga_std_path_with_extension")?;
            Ok(Value::String(
                std::path::Path::new(&path)
                    .with_extension(extension)
                    .to_string_lossy()
                    .into_owned(),
            ))
        }
        BuiltinId::StdPathIsAbsolute => {
            let path = expect_string_arg(args, span, "__muga_std_path_is_absolute")?;
            Ok(Value::Bool(std::path::Path::new(&path).is_absolute()))
        }
        BuiltinId::StdBytesSize => {
            let bytes = expect_bytes_arg(args, span, "__muga_std_bytes_size")?;
            Ok(Value::Int(bytes.len() as i64))
        }
        BuiltinId::StdBytesIsEmpty => {
            let bytes = expect_bytes_arg(args, span, "__muga_std_bytes_is_empty")?;
            Ok(Value::Bool(bytes.is_empty()))
        }
        BuiltinId::StdBytesAt => {
            let (bytes, index) = expect_two_args(args, span)?;
            let Value::Bytes(bytes) = bytes else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_bytes_at` expects bytes::Bytes as its first argument",
                    span,
                )]);
            };
            let Value::Int(index) = index else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_bytes_at` expects Int as its second argument",
                    span,
                )]);
            };
            let Some(index) = usize::try_from(index).ok() else {
                return Ok(option_none());
            };
            Ok(bytes
                .get(index)
                .map(|byte| option_some(Value::Int(i64::from(*byte))))
                .unwrap_or_else(option_none))
        }
        BuiltinId::StdFsReadText => {
            let value = expect_one_arg(args, span)?;
            let Value::String(path) = value else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_fs_read_text` expects String as its first argument",
                    span,
                )]);
            };
            match fs::read_to_string(&path) {
                Ok(text) => Ok(result_ok(Value::String(text))),
                Err(error) => Ok(result_err(io_error_value("read_text", &path, &error))),
            }
        }
        BuiltinId::StdFsReadBytes => {
            let path = expect_string_arg(args, span, "__muga_std_fs_read_bytes")?;
            match fs::read(&path) {
                Ok(bytes) => Ok(result_ok(Value::Bytes(bytes))),
                Err(error) => Ok(result_err(io_error_value("read_bytes", &path, &error))),
            }
        }
        BuiltinId::StdHashSha256Hex => {
            let bytes = expect_bytes_arg(args, span, "__muga_std_hash_sha256_hex")?;
            Ok(Value::String(crate::package::sha256_hex(&bytes)))
        }
        BuiltinId::StdFsReadResourceText => {
            let (package_path, resource_path) =
                expect_two_string_args(args, span, "__muga_std_fs_read_resource_text")?;
            let display_path = resource_display_path(&package_path, &resource_path);
            let roots = env.borrow().package_resource_roots.clone();
            match read_package_resource_text(&roots, &package_path, &resource_path) {
                Ok(text) => Ok(result_ok(Value::String(text))),
                Err(error) => Ok(result_err(io_error_value(
                    "read_resource_text",
                    &display_path,
                    &error,
                ))),
            }
        }
        BuiltinId::StdFsReadResourceBytes => {
            let (package_path, resource_path) =
                expect_two_string_args(args, span, "__muga_std_fs_read_resource_bytes")?;
            let display_path = resource_display_path(&package_path, &resource_path);
            let roots = env.borrow().package_resource_roots.clone();
            match read_package_resource_bytes(&roots, &package_path, &resource_path) {
                Ok(bytes) => Ok(result_ok(Value::Bytes(bytes))),
                Err(error) => Ok(result_err(io_error_value(
                    "read_resource_bytes",
                    &display_path,
                    &error,
                ))),
            }
        }
        BuiltinId::StdFsWriteText => {
            let (path, text) = expect_two_args(args, span)?;
            let Value::String(path) = path else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_fs_write_text` expects String as its first argument",
                    span,
                )]);
            };
            let Value::String(text) = text else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_fs_write_text` expects String as its second argument",
                    span,
                )]);
            };
            match fs::write(&path, text) {
                Ok(()) => Ok(result_ok(Value::Unit)),
                Err(error) => Ok(result_err(io_error_value("write_text", &path, &error))),
            }
        }
        BuiltinId::StdFsWriteBytes => {
            let (path, data) = expect_two_args(args, span)?;
            let Value::String(path) = path else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_fs_write_bytes` expects String as its first argument",
                    span,
                )]);
            };
            let Value::Bytes(data) = data else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_fs_write_bytes` expects bytes::Bytes as its second argument",
                    span,
                )]);
            };
            match fs::write(&path, data) {
                Ok(()) => Ok(result_ok(Value::Unit)),
                Err(error) => Ok(result_err(io_error_value("write_bytes", &path, &error))),
            }
        }
        BuiltinId::StdFsOpenText => {
            let path = expect_string_arg(args, span, "__muga_std_fs_open_text")?;
            match env
                .borrow()
                .runtime_handles
                .borrow_mut()
                .open_std_fs_file(&path)
            {
                Ok(handle) => Ok(result_ok(Value::RuntimeHandle(handle))),
                Err(error) => Ok(result_err(io_error_value("open_text", &path, &error))),
            }
        }
        BuiltinId::StdFsCreateText => {
            let path = expect_string_arg(args, span, "__muga_std_fs_create_text")?;
            match env
                .borrow()
                .runtime_handles
                .borrow_mut()
                .create_std_fs_file(&path)
            {
                Ok(handle) => Ok(result_ok(Value::RuntimeHandle(handle))),
                Err(error) => Ok(result_err(io_error_value("create_text", &path, &error))),
            }
        }
        BuiltinId::StdFsAppendText => {
            let path = expect_string_arg(args, span, "__muga_std_fs_append_text")?;
            match env
                .borrow()
                .runtime_handles
                .borrow_mut()
                .append_std_fs_file(&path)
            {
                Ok(handle) => Ok(result_ok(Value::RuntimeHandle(handle))),
                Err(error) => Ok(result_err(io_error_value("append_text", &path, &error))),
            }
        }
        BuiltinId::StdFsReadTextFrom => {
            let handle = expect_runtime_handle_arg(args, span, "__muga_std_fs_read_text_from")?;
            let result = env
                .borrow()
                .runtime_handles
                .borrow_mut()
                .read_std_fs_file_text(&handle, span)?;
            match result {
                (_path, Ok(text)) => Ok(result_ok(Value::String(text))),
                (path, Err(error)) => {
                    Ok(result_err(io_error_value("read_text_from", &path, &error)))
                }
            }
        }
        BuiltinId::StdFsWriteTextTo => {
            let (handle, text) =
                expect_runtime_handle_and_string_args(args, span, "__muga_std_fs_write_text_to")?;
            let result = env
                .borrow()
                .runtime_handles
                .borrow_mut()
                .write_std_fs_file_text(&handle, &text, span)?;
            match result {
                (_path, Ok(())) => Ok(result_ok(Value::Unit)),
                (path, Err(error)) => {
                    Ok(result_err(io_error_value("write_text_to", &path, &error)))
                }
            }
        }
        BuiltinId::StdFsFlush => {
            let handle = expect_runtime_handle_arg(args, span, "__muga_std_fs_flush")?;
            let result = env
                .borrow()
                .runtime_handles
                .borrow_mut()
                .flush_std_fs_file(&handle, span)?;
            match result {
                (_path, Ok(())) => Ok(result_ok(Value::Unit)),
                (path, Err(error)) => Ok(result_err(io_error_value("flush", &path, &error))),
            }
        }
        BuiltinId::StdFsClose => {
            let handle = expect_runtime_handle_arg(args, span, "__muga_std_fs_close")?;
            let result = env
                .borrow()
                .runtime_handles
                .borrow_mut()
                .close_std_fs_file(&handle, span)?;
            match result {
                (_path, Ok(())) => Ok(result_ok(Value::Unit)),
                (path, Err(error)) => Ok(result_err(io_error_value("close", &path, &error))),
            }
        }
        BuiltinId::StdFsReadDir => {
            let path = expect_string_arg(args, span, "__muga_std_fs_read_dir")?;
            match fs::read_dir(&path) {
                Ok(entries) => {
                    let mut paths = Vec::new();
                    for entry in entries {
                        let entry = match entry {
                            Ok(entry) => entry,
                            Err(error) => {
                                return Ok(result_err(io_error_value("read_dir", &path, &error)));
                            }
                        };
                        let Some(entry_path) = entry.path().to_str().map(str::to_string) else {
                            let error = io::Error::new(
                                io::ErrorKind::InvalidData,
                                "directory entry path is not valid Unicode",
                            );
                            return Ok(result_err(io_error_value("read_dir", &path, &error)));
                        };
                        paths.push(entry_path);
                    }
                    paths.sort();
                    Ok(result_ok(Value::List(
                        paths.into_iter().map(path_value).collect(),
                    )))
                }
                Err(error) => Ok(result_err(io_error_value("read_dir", &path, &error))),
            }
        }
        BuiltinId::StdFsReadDirRecursive => {
            let path = expect_string_arg(args, span, "__muga_std_fs_read_dir_recursive")?;
            match read_dir_recursive_paths(&path) {
                Ok(paths) => Ok(result_ok(Value::List(
                    paths.into_iter().map(path_value).collect(),
                ))),
                Err((error_path, error)) => {
                    Ok(result_err(io_error_value("read_dir", &error_path, &error)))
                }
            }
        }
        BuiltinId::StdFsDirectorySizeMetadata => {
            let path = expect_string_arg(args, span, "__muga_std_fs_directory_size_metadata")?;
            match read_directory_size_metadata(&path) {
                Ok(metadata) => Ok(result_ok(directory_size_metadata_value(metadata))),
                Err((error_path, error)) => Ok(result_err(io_error_value(
                    "directory_size_metadata",
                    &error_path,
                    &error,
                ))),
            }
        }
        BuiltinId::StdFsCanonicalize => {
            let path = expect_string_arg(args, span, "__muga_std_fs_canonicalize")?;
            match fs::canonicalize(&path)
                .and_then(|path| path_buf_into_string(path, "canonical path is not valid Unicode"))
            {
                Ok(path) => Ok(result_ok(Value::String(path))),
                Err(error) => Ok(result_err(io_error_value("canonicalize", &path, &error))),
            }
        }
        BuiltinId::StdFsCreateDir => {
            let path = expect_string_arg(args, span, "__muga_std_fs_create_dir")?;
            match fs::create_dir(&path) {
                Ok(()) => Ok(result_ok(Value::Unit)),
                Err(error) => Ok(result_err(io_error_value("create_dir", &path, &error))),
            }
        }
        BuiltinId::StdFsCreateDirAll => {
            let path = expect_string_arg(args, span, "__muga_std_fs_create_dir_all")?;
            match fs::create_dir_all(&path) {
                Ok(()) => Ok(result_ok(Value::Unit)),
                Err(error) => Ok(result_err(io_error_value("create_dir_all", &path, &error))),
            }
        }
        BuiltinId::StdFsRemoveFile => {
            let path = expect_string_arg(args, span, "__muga_std_fs_remove_file")?;
            match fs::remove_file(&path) {
                Ok(()) => Ok(result_ok(Value::Unit)),
                Err(error) => Ok(result_err(io_error_value("remove_file", &path, &error))),
            }
        }
        BuiltinId::StdFsRemoveDir => {
            let path = expect_string_arg(args, span, "__muga_std_fs_remove_dir")?;
            match fs::remove_dir(&path) {
                Ok(()) => Ok(result_ok(Value::Unit)),
                Err(error) => Ok(result_err(io_error_value("remove_dir", &path, &error))),
            }
        }
        BuiltinId::StdFsRemoveDirAll => {
            let path = expect_string_arg(args, span, "__muga_std_fs_remove_dir_all")?;
            match fs::remove_dir_all(&path) {
                Ok(()) => Ok(result_ok(Value::Unit)),
                Err(error) => Ok(result_err(io_error_value("remove_dir_all", &path, &error))),
            }
        }
        BuiltinId::StdFsCopyFile => {
            let (from_path, to_path) =
                expect_two_string_args(args, span, "__muga_std_fs_copy_file")?;
            match fs::copy(&from_path, &to_path) {
                Ok(_) => Ok(result_ok(Value::Unit)),
                Err(error) => Ok(result_err(path_pair_error_value(
                    "copy_file",
                    &from_path,
                    &to_path,
                    &error,
                ))),
            }
        }
        BuiltinId::StdFsCopyDirAll => {
            let (from_path, to_path) =
                expect_two_string_args(args, span, "__muga_std_fs_copy_dir_all")?;
            match copy_dir_all_paths(&from_path, &to_path) {
                Ok(()) => Ok(result_ok(Value::Unit)),
                Err((error_from_path, error_to_path, error)) => Ok(result_err(
                    path_pair_error_value("copy_dir_all", &error_from_path, &error_to_path, &error),
                )),
            }
        }
        BuiltinId::StdFsMoveDirAll => {
            let (from_path, to_path) =
                expect_two_string_args(args, span, "__muga_std_fs_move_dir_all")?;
            match move_dir_all_paths(&from_path, &to_path) {
                Ok(()) => Ok(result_ok(Value::Unit)),
                Err((error_from_path, error_to_path, error)) => Ok(result_err(
                    path_pair_error_value("move_dir_all", &error_from_path, &error_to_path, &error),
                )),
            }
        }
        BuiltinId::StdFsRename => {
            let (from_path, to_path) = expect_two_string_args(args, span, "__muga_std_fs_rename")?;
            match fs::rename(&from_path, &to_path) {
                Ok(()) => Ok(result_ok(Value::Unit)),
                Err(error) => Ok(result_err(path_pair_error_value(
                    "rename", &from_path, &to_path, &error,
                ))),
            }
        }
        BuiltinId::StdFsFileSize => {
            let path = expect_string_arg(args, span, "__muga_std_fs_file_size")?;
            match fs::metadata(&path).and_then(|metadata| {
                if !metadata.is_file() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "path is not a file",
                    ));
                }
                i64::try_from(metadata.len()).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "file size does not fit in Int")
                })
            }) {
                Ok(size) => Ok(result_ok(Value::Int(size))),
                Err(error) => Ok(result_err(io_error_value("file_size", &path, &error))),
            }
        }
        BuiltinId::StdFsModifiedUnixMillis => {
            let path = expect_string_arg(args, span, "__muga_std_fs_modified_unix_millis")?;
            match fs::metadata(&path)
                .and_then(|metadata| metadata.modified())
                .and_then(|modified| {
                    let duration = modified.duration_since(UNIX_EPOCH).map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "modified time is before Unix epoch",
                        )
                    })?;
                    i64::try_from(duration.as_millis()).map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "modified time does not fit in Int",
                        )
                    })
                }) {
                Ok(millis) => Ok(result_ok(Value::Int(millis))),
                Err(error) => Ok(result_err(io_error_value(
                    "modified_unix_millis",
                    &path,
                    &error,
                ))),
            }
        }
        BuiltinId::StdFsExists => {
            let path = expect_string_arg(args, span, "__muga_std_fs_exists")?;
            Ok(Value::Bool(std::path::Path::new(&path).exists()))
        }
        BuiltinId::StdFsIsFile => {
            let path = expect_string_arg(args, span, "__muga_std_fs_is_file")?;
            Ok(Value::Bool(std::path::Path::new(&path).is_file()))
        }
        BuiltinId::StdFsIsDir => {
            let path = expect_string_arg(args, span, "__muga_std_fs_is_dir")?;
            Ok(Value::Bool(std::path::Path::new(&path).is_dir()))
        }
        BuiltinId::StdEnvGetVar => {
            let value = expect_one_arg(args, span)?;
            let Value::String(name) = value else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_env_get_var` expects String as its first argument",
                    span,
                )]);
            };
            match process_env::var(&name) {
                Ok(value) => Ok(option_some(Value::String(value))),
                Err(_) => Ok(option_none()),
            }
        }
        BuiltinId::StdEnvArgs => {
            expect_no_args(args, span)?;
            let args = env
                .borrow()
                .program_args
                .iter()
                .cloned()
                .map(Value::String)
                .collect();
            Ok(Value::List(args))
        }
        BuiltinId::StdEnvCurrentDir => {
            expect_no_args(args, span)?;
            match process_env::current_dir().and_then(|path| {
                path_buf_into_string(path, "current directory is not valid Unicode")
            }) {
                Ok(path) => Ok(result_ok(Value::String(path))),
                Err(error) => Ok(result_err(io_error_value("current_dir", ".", &error))),
            }
        }
        BuiltinId::StdEnvTempDir => {
            expect_no_args(args, span)?;
            match path_buf_into_string(
                process_env::temp_dir(),
                "temporary directory path is not valid Unicode",
            ) {
                Ok(path) => Ok(result_ok(Value::String(path))),
                Err(error) => Ok(result_err(io_error_value("temp_dir", ".", &error))),
            }
        }
        BuiltinId::StdTimeNowUnixMillis => {
            expect_no_args(args, span)?;
            let duration = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| {
                vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_time_now_unix_millis` requires system time after Unix epoch",
                    span,
                )]
            })?;
            let millis = i64::try_from(duration.as_millis()).map_err(|_| {
                vec![Diagnostic::new(
                    "R019",
                    "`__muga_std_time_now_unix_millis` overflowed Int",
                    span,
                )]
            })?;
            Ok(Value::Int(millis))
        }
        BuiltinId::StdTestAssertTrue => {
            let value = expect_one_arg(args, span)?;
            let Value::Bool(value) = value else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_test_assert_true` expects Bool as its first argument",
                    span,
                )]);
            };
            Ok(test_assert_result(
                value,
                "assert_true failed: expected true but got false".to_string(),
                env,
                span,
            ))
        }
        BuiltinId::StdTestAssertEqInt => {
            let (expected, actual) = expect_two_args(args, span)?;
            let Value::Int(expected) = expected else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_test_assert_eq_int` expects Int as its first argument",
                    span,
                )]);
            };
            let Value::Int(actual) = actual else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_test_assert_eq_int` expects Int as its second argument",
                    span,
                )]);
            };
            Ok(test_assert_result(
                expected == actual,
                format!("assert_eq_int failed: expected {expected} but got {actual}"),
                env,
                span,
            ))
        }
        BuiltinId::StdTestAssertEqBool => {
            let (expected, actual) = expect_two_args(args, span)?;
            let Value::Bool(expected) = expected else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_test_assert_eq_bool` expects Bool as its first argument",
                    span,
                )]);
            };
            let Value::Bool(actual) = actual else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_test_assert_eq_bool` expects Bool as its second argument",
                    span,
                )]);
            };
            Ok(test_assert_result(
                expected == actual,
                format!("assert_eq_bool failed: expected {expected} but got {actual}"),
                env,
                span,
            ))
        }
        BuiltinId::StdTestAssertEqString => {
            let (expected, actual) = expect_two_args(args, span)?;
            let Value::String(expected) = expected else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_test_assert_eq_string` expects String as its first argument",
                    span,
                )]);
            };
            let Value::String(actual) = actual else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`__muga_std_test_assert_eq_string` expects String as its second argument",
                    span,
                )]);
            };
            Ok(test_assert_result(
                expected == actual,
                format!("assert_eq_string failed: expected `{expected}` but got `{actual}`"),
                env,
                span,
            ))
        }
        BuiltinId::OptionSome => {
            let value = expect_one_arg(args, span)?;
            Ok(option_some(value))
        }
        BuiltinId::ResultOk => {
            let value = expect_one_arg(args, span)?;
            Ok(result_ok(value))
        }
        BuiltinId::ResultErr => {
            let value = expect_one_arg(args, span)?;
            Ok(result_err(value))
        }
        BuiltinId::OptionNone => Err(vec![Diagnostic::new(
            "R010",
            "attempted to call a non-function value",
            span,
        )]),
    }
}

fn test_assert_result(passed: bool, failure_message: String, env: &EnvRef, span: Span) -> Value {
    if passed {
        result_ok(Value::Unit)
    } else {
        add_test_assertion_diagnostic(env, &failure_message, span);
        result_err(Value::String(failure_message))
    }
}

fn map_key(value: Value, span: Span, builtin_name: &str) -> Result<MapKey, Vec<Diagnostic>> {
    match value {
        Value::Int(value) => Ok(MapKey::Int(value)),
        Value::Bool(value) => Ok(MapKey::Bool(value)),
        Value::String(value) => Ok(MapKey::String(value)),
        _ => Err(vec![Diagnostic::new(
            "R014",
            format!("`{builtin_name}` expects an Int, Bool, or String Map key"),
            span,
        )]),
    }
}

fn eval_binary(
    op: BinaryOp,
    left: Value,
    right: Value,
    span: Span,
) -> Result<Value, Vec<Diagnostic>> {
    match (op, left, right) {
        (BinaryOp::Add, Value::Int(left), Value::Int(right)) => {
            checked_int(left.checked_add(right), span)
        }
        (BinaryOp::Sub, Value::Int(left), Value::Int(right)) => {
            checked_int(left.checked_sub(right), span)
        }
        (BinaryOp::Mul, Value::Int(left), Value::Int(right)) => {
            checked_int(left.checked_mul(right), span)
        }
        (BinaryOp::Div, Value::Int(_), Value::Int(0)) => {
            Err(vec![Diagnostic::new("R013", "division by zero", span)])
        }
        (BinaryOp::Div, Value::Int(left), Value::Int(right)) => {
            checked_int(left.checked_div(right), span)
        }
        (BinaryOp::Lt, Value::Int(left), Value::Int(right)) => Ok(Value::Bool(left < right)),
        (BinaryOp::LtEq, Value::Int(left), Value::Int(right)) => Ok(Value::Bool(left <= right)),
        (BinaryOp::Gt, Value::Int(left), Value::Int(right)) => Ok(Value::Bool(left > right)),
        (BinaryOp::GtEq, Value::Int(left), Value::Int(right)) => Ok(Value::Bool(left >= right)),
        (BinaryOp::EqEq, Value::Int(left), Value::Int(right)) => Ok(Value::Bool(left == right)),
        (BinaryOp::EqEq, Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(left == right)),
        (BinaryOp::EqEq, Value::String(left), Value::String(right)) => {
            Ok(Value::Bool(left == right))
        }
        (BinaryOp::BangEq, Value::Int(left), Value::Int(right)) => Ok(Value::Bool(left != right)),
        (BinaryOp::BangEq, Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(left != right)),
        (BinaryOp::BangEq, Value::String(left), Value::String(right)) => {
            Ok(Value::Bool(left != right))
        }
        _ => Err(vec![Diagnostic::new(
            "R011",
            "invalid operands for binary operator",
            span,
        )]),
    }
}

fn checked_int(value: Option<i64>, span: Span) -> Result<Value, Vec<Diagnostic>> {
    value.map(Value::Int).ok_or_else(|| integer_overflow(span))
}

fn integer_overflow(span: Span) -> Vec<Diagnostic> {
    vec![Diagnostic::new("R019", "integer overflow", span)]
}

fn pop_args(
    stack: &mut Vec<Value>,
    argc: usize,
    span: Span,
) -> Result<Vec<Value>, Vec<Diagnostic>> {
    if stack.len() < argc {
        return Err(vec![Diagnostic::new(
            "R015",
            "missing call arguments on stack",
            span,
        )]);
    }
    let mut args = Vec::with_capacity(argc);
    for _ in 0..argc {
        args.push(stack.pop().expect("checked length"));
    }
    args.reverse();
    Ok(args)
}

fn pop_value(
    stack: &mut Vec<Value>,
    span: Span,
    code: &'static str,
    message: &'static str,
) -> Result<Value, Vec<Diagnostic>> {
    stack
        .pop()
        .ok_or_else(|| vec![Diagnostic::new(code, message, span)])
}

fn make_record_value(
    program: &Program,
    type_name: Symbol,
    fields: &[Symbol],
    values: Vec<Value>,
) -> Value {
    Value::Record(RecordValue {
        type_name: symbol_name(program, type_name).to_string(),
        fields: fields
            .iter()
            .zip(values)
            .map(|(field, value)| RecordFieldValue {
                name: symbol_name(program, *field).to_string(),
                value,
            })
            .collect(),
    })
}

fn load_record_field(
    program: &Program,
    base: Value,
    field: Symbol,
    span: Span,
) -> Result<Value, Vec<Diagnostic>> {
    let field_name = symbol_name(program, field);
    let Value::Record(record) = base else {
        return Err(vec![Diagnostic::new(
            "R016",
            "field access requires a record value",
            span,
        )]);
    };
    let Some(field_value) = record
        .fields
        .iter()
        .find(|candidate| candidate.name == field_name)
    else {
        return Err(vec![Diagnostic::new(
            "R017",
            format!("unknown field `{field_name}`"),
            span,
        )]);
    };
    Ok(field_value.value.clone())
}

fn load_list_index(base: Value, index: Value, span: Span) -> Result<Value, Vec<Diagnostic>> {
    let Value::List(items) = base else {
        return Err(vec![Diagnostic::new(
            "R014",
            "list indexing expects List[T] as its base",
            span,
        )]);
    };
    let index = list_index(index, items.len(), span)?;
    Ok(items[index].clone())
}

fn list_index(index: Value, len: usize, span: Span) -> Result<usize, Vec<Diagnostic>> {
    let Value::Int(index) = index else {
        return Err(vec![Diagnostic::new(
            "R014",
            "list index must be Int",
            span,
        )]);
    };
    if index < 0 {
        return Err(list_index_out_of_bounds(span));
    }
    let index = usize::try_from(index).map_err(|_| list_index_out_of_bounds(span))?;
    if index >= len {
        return Err(list_index_out_of_bounds(span));
    }
    Ok(index)
}

fn list_index_out_of_bounds(span: Span) -> Vec<Diagnostic> {
    vec![Diagnostic::new("R020", "list index out of bounds", span)]
}

fn update_record_value(
    program: &Program,
    base: Value,
    fields: &[Symbol],
    values: Vec<Value>,
    span: Span,
) -> Result<Value, Vec<Diagnostic>> {
    let Value::Record(mut record) = base else {
        return Err(vec![Diagnostic::new("R018", "invalid record update", span)]);
    };

    for (field, value) in fields.iter().zip(values) {
        let field_name = symbol_name(program, *field);
        let Some(existing) = record
            .fields
            .iter_mut()
            .find(|candidate| candidate.name == field_name)
        else {
            return Err(vec![Diagnostic::new("R018", "invalid record update", span)]);
        };
        existing.value = value;
    }

    Ok(Value::Record(record))
}

fn install_prelude(program: &Program, env: &EnvRef) {
    for binding in &program.bindings {
        let name = symbol_name(program, binding.name);
        let Some(builtin) = prelude::builtin_by_any_name(name) else {
            continue;
        };
        let value = if builtin.id == BuiltinId::OptionNone {
            option_none()
        } else {
            Value::Builtin(builtin.id)
        };
        env.borrow_mut().insert(
            binding.local,
            Binding {
                mutable: false,
                value,
                span: Span::default(),
            },
        );
    }
}

fn child_env(parent: &EnvRef, function_boundary: bool) -> EnvRef {
    let (local_count, context) = {
        let borrowed = parent.borrow();
        (
            borrowed.bindings.len(),
            RuntimeContext {
                output: borrowed.output.clone(),
                stderr: borrowed.stderr.clone(),
                program_args: borrowed.program_args.clone(),
                runtime_diagnostics: borrowed.runtime_diagnostics.clone(),
                call_stack: borrowed.call_stack.clone(),
                runtime_handles: borrowed.runtime_handles.clone(),
                package_resource_roots: borrowed.package_resource_roots.clone(),
            },
        )
    };
    Rc::new(RefCell::new(Env::new(
        Some(parent.clone()),
        function_boundary,
        context,
        local_count,
    )))
}

fn lookup_any(env: &EnvRef, local: LocalId) -> Option<Binding> {
    let mut current = Some(env.clone());
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if let Some(found) = borrowed.binding(local) {
            return Some(found.clone());
        }
        current = borrowed.parent.clone();
    }
    None
}

fn lookup_in_current_function_env(env: &EnvRef, local: LocalId) -> Option<EnvRef> {
    let mut current = Some(env.clone());
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if borrowed.contains(local) {
            return Some(candidate.clone());
        }
        let stop = borrowed.function_boundary;
        let parent = borrowed.parent.clone();
        drop(borrowed);
        if stop {
            break;
        }
        current = parent;
    }
    None
}

fn lookup_beyond_current_function(env: &EnvRef, local: LocalId) -> Option<Binding> {
    let mut first_boundary_seen = false;
    let mut current = Some(env.clone());
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if first_boundary_seen && let Some(found) = borrowed.binding(local) {
            return Some(found.clone());
        }
        if borrowed.function_boundary {
            first_boundary_seen = true;
        }
        current = borrowed.parent.clone();
    }
    None
}
