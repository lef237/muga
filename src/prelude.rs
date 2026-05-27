use crate::known_enum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BuiltinId {
    Print,
    Println,
    Eprint,
    Eprintln,
    Len,
    IsEmpty,
    Push,
    Get,
    Set,
    MapEmpty,
    Contains,
    Insert,
    Remove,
    Trim,
    CharCount,
    ByteLen,
    StartsWith,
    EndsWith,
    Replace,
    Split,
    Concat,
    SliceChars,
    ToString,
    ParseInt,
    ParseBool,
    StdPathJoin,
    StdPathNormalize,
    StdPathFileName,
    StdPathWithFileName,
    StdPathParent,
    StdPathStripPrefix,
    StdPathExtension,
    StdPathFileStem,
    StdPathWithExtension,
    StdPathIsAbsolute,
    StdBytesSize,
    StdBytesIsEmpty,
    StdBytesAt,
    StdFsReadText,
    StdFsReadBytes,
    StdFsReadResourceText,
    StdFsReadResourceBytes,
    StdFsWriteText,
    StdFsWriteBytes,
    StdFsOpenText,
    StdFsCreateText,
    StdFsAppendText,
    StdFsReadTextFrom,
    StdFsWriteTextTo,
    StdFsFlush,
    StdFsClose,
    StdFsReadDir,
    StdFsReadDirRecursive,
    StdFsDirectorySizeMetadata,
    StdFsCanonicalize,
    StdFsCreateDir,
    StdFsCreateDirAll,
    StdFsRemoveFile,
    StdFsRemoveDir,
    StdFsRemoveDirAll,
    StdFsCopyFile,
    StdFsCopyDirAll,
    StdFsMoveDirAll,
    StdFsRename,
    StdFsFileSize,
    StdFsModifiedUnixMillis,
    StdFsExists,
    StdFsIsFile,
    StdFsIsDir,
    StdEnvGetVar,
    StdEnvArgs,
    StdEnvCurrentDir,
    StdEnvTempDir,
    StdTimeNowUnixMillis,
    StdHashSha256Hex,
    StdTestAssertTrue,
    StdTestAssertEqInt,
    StdTestAssertEqBool,
    StdTestAssertEqString,
    StdMapKeys,
    StdMapValues,
    StdJsonParse,
    StdJsonEncode,
    StdJsonNumberAsInt,
    OptionSome,
    OptionNone,
    ResultOk,
    ResultErr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinKind {
    Function,
    Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Builtin {
    pub id: BuiltinId,
    pub name: &'static str,
    pub kind: BuiltinKind,
}

pub const BUILTINS: &[Builtin] = &[
    Builtin {
        id: BuiltinId::Print,
        name: "print",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Println,
        name: "println",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Eprint,
        name: "eprint",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Eprintln,
        name: "eprintln",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Len,
        name: "len",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::IsEmpty,
        name: "is_empty",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Push,
        name: "push",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Get,
        name: "get",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Set,
        name: "set",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::MapEmpty,
        name: "Map.empty",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Contains,
        name: "contains",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Insert,
        name: "insert",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Remove,
        name: "remove",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Trim,
        name: "trim",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::CharCount,
        name: "char_count",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::ByteLen,
        name: "byte_len",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StartsWith,
        name: "starts_with",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::EndsWith,
        name: "ends_with",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Replace,
        name: "replace",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Split,
        name: "split",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Concat,
        name: "concat",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::SliceChars,
        name: "slice_chars",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::ToString,
        name: "to_string",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::ParseInt,
        name: "parse_int",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::ParseBool,
        name: "parse_bool",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::OptionSome,
        name: known_enum::OPTION_SOME_QUALIFIED,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::OptionNone,
        name: known_enum::OPTION_NONE_QUALIFIED,
        kind: BuiltinKind::Value,
    },
    Builtin {
        id: BuiltinId::ResultOk,
        name: known_enum::RESULT_OK_QUALIFIED,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::ResultErr,
        name: known_enum::RESULT_ERR_QUALIFIED,
        kind: BuiltinKind::Function,
    },
];

const INTERNAL_BUILTINS: &[Builtin] = &[
    Builtin {
        id: BuiltinId::StdPathJoin,
        name: crate::std_package::PATH_JOIN_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdPathNormalize,
        name: crate::std_package::PATH_NORMALIZE_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdPathFileName,
        name: crate::std_package::PATH_FILE_NAME_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdPathWithFileName,
        name: crate::std_package::PATH_WITH_FILE_NAME_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdPathParent,
        name: crate::std_package::PATH_PARENT_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdPathStripPrefix,
        name: crate::std_package::PATH_STRIP_PREFIX_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdPathExtension,
        name: crate::std_package::PATH_EXTENSION_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdPathFileStem,
        name: crate::std_package::PATH_FILE_STEM_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdPathWithExtension,
        name: crate::std_package::PATH_WITH_EXTENSION_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdPathIsAbsolute,
        name: crate::std_package::PATH_IS_ABSOLUTE_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsReadText,
        name: crate::std_package::FS_READ_TEXT_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdBytesSize,
        name: crate::std_package::BYTES_SIZE_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdBytesIsEmpty,
        name: crate::std_package::BYTES_IS_EMPTY_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdBytesAt,
        name: crate::std_package::BYTES_AT_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsReadBytes,
        name: crate::std_package::FS_READ_BYTES_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsReadResourceText,
        name: crate::std_package::FS_READ_RESOURCE_TEXT_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsReadResourceBytes,
        name: crate::std_package::FS_READ_RESOURCE_BYTES_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsWriteText,
        name: crate::std_package::FS_WRITE_TEXT_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsWriteBytes,
        name: crate::std_package::FS_WRITE_BYTES_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsOpenText,
        name: crate::std_package::FS_OPEN_TEXT_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsCreateText,
        name: crate::std_package::FS_CREATE_TEXT_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsAppendText,
        name: crate::std_package::FS_APPEND_TEXT_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsReadTextFrom,
        name: crate::std_package::FS_READ_TEXT_FROM_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsWriteTextTo,
        name: crate::std_package::FS_WRITE_TEXT_TO_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsFlush,
        name: crate::std_package::FS_FLUSH_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsClose,
        name: crate::std_package::FS_CLOSE_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsReadDir,
        name: crate::std_package::FS_READ_DIR_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsReadDirRecursive,
        name: crate::std_package::FS_READ_DIR_RECURSIVE_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsDirectorySizeMetadata,
        name: crate::std_package::FS_DIRECTORY_SIZE_METADATA_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsCanonicalize,
        name: crate::std_package::FS_CANONICALIZE_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsCreateDir,
        name: crate::std_package::FS_CREATE_DIR_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsCreateDirAll,
        name: crate::std_package::FS_CREATE_DIR_ALL_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsRemoveFile,
        name: crate::std_package::FS_REMOVE_FILE_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsRemoveDir,
        name: crate::std_package::FS_REMOVE_DIR_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsRemoveDirAll,
        name: crate::std_package::FS_REMOVE_DIR_ALL_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsCopyFile,
        name: crate::std_package::FS_COPY_FILE_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsCopyDirAll,
        name: crate::std_package::FS_COPY_DIR_ALL_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsMoveDirAll,
        name: crate::std_package::FS_MOVE_DIR_ALL_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsRename,
        name: crate::std_package::FS_RENAME_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsFileSize,
        name: crate::std_package::FS_FILE_SIZE_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsModifiedUnixMillis,
        name: crate::std_package::FS_MODIFIED_UNIX_MILLIS_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsExists,
        name: crate::std_package::FS_EXISTS_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsIsFile,
        name: crate::std_package::FS_IS_FILE_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsIsDir,
        name: crate::std_package::FS_IS_DIR_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdEnvGetVar,
        name: crate::std_package::ENV_GET_VAR_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdEnvArgs,
        name: crate::std_package::ENV_ARGS_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdEnvCurrentDir,
        name: crate::std_package::ENV_CURRENT_DIR_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdEnvTempDir,
        name: crate::std_package::ENV_TEMP_DIR_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdTimeNowUnixMillis,
        name: crate::std_package::TIME_NOW_UNIX_MILLIS_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdHashSha256Hex,
        name: crate::std_package::HASH_SHA256_HEX_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdTestAssertTrue,
        name: crate::std_package::TEST_ASSERT_TRUE_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdTestAssertEqInt,
        name: crate::std_package::TEST_ASSERT_EQ_INT_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdTestAssertEqBool,
        name: crate::std_package::TEST_ASSERT_EQ_BOOL_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdTestAssertEqString,
        name: crate::std_package::TEST_ASSERT_EQ_STRING_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdMapKeys,
        name: crate::std_package::MAP_KEYS_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdMapValues,
        name: crate::std_package::MAP_VALUES_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdJsonParse,
        name: crate::std_package::JSON_PARSE_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdJsonEncode,
        name: crate::std_package::JSON_ENCODE_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdJsonNumberAsInt,
        name: crate::std_package::JSON_NUMBER_AS_INT_BUILTIN,
        kind: BuiltinKind::Function,
    },
];

pub fn builtins() -> &'static [Builtin] {
    BUILTINS
}

pub(crate) fn internal_builtins() -> &'static [Builtin] {
    INTERNAL_BUILTINS
}

pub fn builtin_name(id: BuiltinId) -> &'static str {
    builtin_by_id(id).name
}

pub fn builtin_debug_label(id: BuiltinId) -> &'static str {
    match id {
        BuiltinId::Print => "Builtin(print)",
        BuiltinId::Println => "Builtin(println)",
        BuiltinId::Eprint => "Builtin(eprint)",
        BuiltinId::Eprintln => "Builtin(eprintln)",
        BuiltinId::Len => "Builtin(len)",
        BuiltinId::IsEmpty => "Builtin(is_empty)",
        BuiltinId::Push => "Builtin(push)",
        BuiltinId::Get => "Builtin(get)",
        BuiltinId::Set => "Builtin(set)",
        BuiltinId::MapEmpty => "Builtin(Map.empty)",
        BuiltinId::Contains => "Builtin(contains)",
        BuiltinId::Insert => "Builtin(insert)",
        BuiltinId::Remove => "Builtin(remove)",
        BuiltinId::Trim => "Builtin(trim)",
        BuiltinId::CharCount => "Builtin(char_count)",
        BuiltinId::ByteLen => "Builtin(byte_len)",
        BuiltinId::StartsWith => "Builtin(starts_with)",
        BuiltinId::EndsWith => "Builtin(ends_with)",
        BuiltinId::Replace => "Builtin(replace)",
        BuiltinId::Split => "Builtin(split)",
        BuiltinId::Concat => "Builtin(concat)",
        BuiltinId::SliceChars => "Builtin(slice_chars)",
        BuiltinId::ToString => "Builtin(to_string)",
        BuiltinId::ParseInt => "Builtin(parse_int)",
        BuiltinId::ParseBool => "Builtin(parse_bool)",
        BuiltinId::StdPathJoin => "Builtin(__muga_std_path_join)",
        BuiltinId::StdPathNormalize => "Builtin(__muga_std_path_normalize)",
        BuiltinId::StdPathFileName => "Builtin(__muga_std_path_file_name)",
        BuiltinId::StdPathWithFileName => "Builtin(__muga_std_path_with_file_name)",
        BuiltinId::StdPathParent => "Builtin(__muga_std_path_parent)",
        BuiltinId::StdPathStripPrefix => "Builtin(__muga_std_path_strip_prefix)",
        BuiltinId::StdPathExtension => "Builtin(__muga_std_path_extension)",
        BuiltinId::StdPathFileStem => "Builtin(__muga_std_path_file_stem)",
        BuiltinId::StdPathWithExtension => "Builtin(__muga_std_path_with_extension)",
        BuiltinId::StdPathIsAbsolute => "Builtin(__muga_std_path_is_absolute)",
        BuiltinId::StdBytesSize => "Builtin(__muga_std_bytes_size)",
        BuiltinId::StdBytesIsEmpty => "Builtin(__muga_std_bytes_is_empty)",
        BuiltinId::StdBytesAt => "Builtin(__muga_std_bytes_at)",
        BuiltinId::StdFsReadText => "Builtin(__muga_std_fs_read_text)",
        BuiltinId::StdFsReadBytes => "Builtin(__muga_std_fs_read_bytes)",
        BuiltinId::StdFsReadResourceText => "Builtin(__muga_std_fs_read_resource_text)",
        BuiltinId::StdFsReadResourceBytes => "Builtin(__muga_std_fs_read_resource_bytes)",
        BuiltinId::StdFsWriteText => "Builtin(__muga_std_fs_write_text)",
        BuiltinId::StdFsWriteBytes => "Builtin(__muga_std_fs_write_bytes)",
        BuiltinId::StdFsOpenText => "Builtin(__muga_std_fs_open_text)",
        BuiltinId::StdFsCreateText => "Builtin(__muga_std_fs_create_text)",
        BuiltinId::StdFsAppendText => "Builtin(__muga_std_fs_append_text)",
        BuiltinId::StdFsReadTextFrom => "Builtin(__muga_std_fs_read_text_from)",
        BuiltinId::StdFsWriteTextTo => "Builtin(__muga_std_fs_write_text_to)",
        BuiltinId::StdFsFlush => "Builtin(__muga_std_fs_flush)",
        BuiltinId::StdFsClose => "Builtin(__muga_std_fs_close)",
        BuiltinId::StdFsReadDir => "Builtin(__muga_std_fs_read_dir)",
        BuiltinId::StdFsReadDirRecursive => "Builtin(__muga_std_fs_read_dir_recursive)",
        BuiltinId::StdFsDirectorySizeMetadata => "Builtin(__muga_std_fs_directory_size_metadata)",
        BuiltinId::StdFsCanonicalize => "Builtin(__muga_std_fs_canonicalize)",
        BuiltinId::StdFsCreateDir => "Builtin(__muga_std_fs_create_dir)",
        BuiltinId::StdFsCreateDirAll => "Builtin(__muga_std_fs_create_dir_all)",
        BuiltinId::StdFsRemoveFile => "Builtin(__muga_std_fs_remove_file)",
        BuiltinId::StdFsRemoveDir => "Builtin(__muga_std_fs_remove_dir)",
        BuiltinId::StdFsRemoveDirAll => "Builtin(__muga_std_fs_remove_dir_all)",
        BuiltinId::StdFsCopyFile => "Builtin(__muga_std_fs_copy_file)",
        BuiltinId::StdFsCopyDirAll => "Builtin(__muga_std_fs_copy_dir_all)",
        BuiltinId::StdFsMoveDirAll => "Builtin(__muga_std_fs_move_dir_all)",
        BuiltinId::StdFsRename => "Builtin(__muga_std_fs_rename)",
        BuiltinId::StdFsFileSize => "Builtin(__muga_std_fs_file_size)",
        BuiltinId::StdFsModifiedUnixMillis => "Builtin(__muga_std_fs_modified_unix_millis)",
        BuiltinId::StdFsExists => "Builtin(__muga_std_fs_exists)",
        BuiltinId::StdFsIsFile => "Builtin(__muga_std_fs_is_file)",
        BuiltinId::StdFsIsDir => "Builtin(__muga_std_fs_is_dir)",
        BuiltinId::StdEnvGetVar => "Builtin(__muga_std_env_get_var)",
        BuiltinId::StdEnvArgs => "Builtin(__muga_std_env_args)",
        BuiltinId::StdEnvCurrentDir => "Builtin(__muga_std_env_current_dir)",
        BuiltinId::StdEnvTempDir => "Builtin(__muga_std_env_temp_dir)",
        BuiltinId::StdTimeNowUnixMillis => "Builtin(__muga_std_time_now_unix_millis)",
        BuiltinId::StdHashSha256Hex => "Builtin(__muga_std_hash_sha256_hex)",
        BuiltinId::StdTestAssertTrue => "Builtin(__muga_std_test_assert_true)",
        BuiltinId::StdTestAssertEqInt => "Builtin(__muga_std_test_assert_eq_int)",
        BuiltinId::StdTestAssertEqBool => "Builtin(__muga_std_test_assert_eq_bool)",
        BuiltinId::StdTestAssertEqString => "Builtin(__muga_std_test_assert_eq_string)",
        BuiltinId::StdMapKeys => "Builtin(__muga_std_map_keys)",
        BuiltinId::StdMapValues => "Builtin(__muga_std_map_values)",
        BuiltinId::StdJsonParse => "Builtin(__muga_std_json_parse)",
        BuiltinId::StdJsonEncode => "Builtin(__muga_std_json_encode)",
        BuiltinId::StdJsonNumberAsInt => "Builtin(__muga_std_json_number_as_int)",
        BuiltinId::OptionSome => "Builtin(Option::Some)",
        BuiltinId::OptionNone => "Builtin(Option::None)",
        BuiltinId::ResultOk => "Builtin(Result::Ok)",
        BuiltinId::ResultErr => "Builtin(Result::Err)",
    }
}

pub fn builtin_by_id(id: BuiltinId) -> Builtin {
    BUILTINS
        .iter()
        .chain(INTERNAL_BUILTINS.iter())
        .copied()
        .find(|builtin| builtin.id == id)
        .expect("every builtin id must be present in the catalog")
}

pub fn builtin_by_name(name: &str) -> Option<Builtin> {
    BUILTINS
        .iter()
        .copied()
        .find(|builtin| builtin.name == name)
}

pub(crate) fn builtin_by_any_name(name: &str) -> Option<Builtin> {
    BUILTINS
        .iter()
        .chain(INTERNAL_BUILTINS.iter())
        .copied()
        .find(|builtin| builtin.name == name)
}

pub fn is_builtin_name(name: &str) -> bool {
    builtin_by_name(name).is_some()
}
