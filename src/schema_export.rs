use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{
    diagnostic::Diagnostic,
    doc,
    identity::PackageItemId,
    interface::{
        PackageInterface, PackageInterfaceEnum, PackageInterfaceEnumVariant, PackageInterfaceField,
        PackageInterfaceGraph, PackageInterfaceRecord,
    },
    json_decode::JsonDecodeValidationRule,
    span::Span,
    std_package,
    symbol::SymbolTable,
    types::TypeInfo,
};

pub const JSON_SCHEMA_DRAFT_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SchemaDecodeMode {
    #[default]
    Required,
    Overlay,
}

impl SchemaDecodeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Overlay => "overlay",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SchemaExportOptions {
    pub package: Option<String>,
    pub type_name: Option<String>,
    pub decode_mode: SchemaDecodeMode,
}

pub fn render_json_config_schema_for_interfaces(
    interfaces: &PackageInterfaceGraph,
    symbols: &SymbolTable,
    default_package: &str,
    options: &SchemaExportOptions,
) -> Result<String, Vec<Diagnostic>> {
    let package_path = options.package.as_deref().unwrap_or(default_package);
    let package = interfaces
        .packages
        .iter()
        .find(|package| package.path == package_path)
        .ok_or_else(|| {
            vec![schema_export_error(
                format!("package `{package_path}` is not available for schema export"),
                Span::default(),
                "choose a package loaded by the entrypoint or emit/load its interface artifact",
            )]
        })?;

    let mut exporter = SchemaExporter::new(interfaces, symbols, options.decode_mode);
    let mut root_refs = Vec::new();
    if let Some(type_name) = &options.type_name {
        let Some(target) = find_schema_target(package, type_name) else {
            return Err(vec![schema_export_error(
                format!(
                    "public record or enum `{type_name}` was not found in package `{package_path}`"
                ),
                Span::default(),
                "export a public concrete record or enum, or pass --package for a dependency package",
            )]);
        };
        root_refs.push(exporter.ensure_target_definition(target)?);
    } else {
        let mut records = package.records.iter().collect::<Vec<_>>();
        records.sort_by(|left, right| left.name.cmp(&right.name));
        for record in records {
            if record.type_params.is_empty() {
                root_refs.push(
                    exporter
                        .ensure_record_definition(record.item)
                        .map_err(|diagnostic| vec![diagnostic])?,
                );
            }
        }
        let mut enums = package.enums.iter().collect::<Vec<_>>();
        enums.sort_by(|left, right| left.name.cmp(&right.name));
        for enumeration in enums {
            if enumeration.type_params.is_empty() {
                root_refs.push(
                    exporter
                        .ensure_enum_definition(enumeration.item)
                        .map_err(|diagnostic| vec![diagnostic])?,
                );
            }
        }
        if root_refs.is_empty() {
            return Err(vec![schema_export_error(
                format!(
                    "package `{package_path}` has no exportable public concrete records or enums"
                ),
                Span::default(),
                "export a public non-generic record or enum",
            )]);
        }
    }

    Ok(exporter.render_document(package_path, root_refs))
}

#[derive(Clone, Copy)]
enum SchemaTarget<'a> {
    Record(&'a PackageInterfaceRecord),
    Enum(&'a PackageInterfaceEnum),
}

fn find_schema_target<'a>(
    package: &'a PackageInterface,
    type_name: &str,
) -> Option<SchemaTarget<'a>> {
    let qualified = |name: &str| format!("{}::{name}", package.path);
    package
        .records
        .iter()
        .find(|record| record.name == type_name || qualified(&record.name) == type_name)
        .map(SchemaTarget::Record)
        .or_else(|| {
            package
                .enums
                .iter()
                .find(|enumeration| {
                    enumeration.name == type_name || qualified(&enumeration.name) == type_name
                })
                .map(SchemaTarget::Enum)
        })
}

#[derive(Default)]
struct SchemaObject {
    fields: Vec<(String, String)>,
    x_muga: Vec<(String, String)>,
}

impl SchemaObject {
    fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push((key.into(), value.into()));
        self
    }

    fn with_x_muga(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.x_muga.push((key.into(), value.into()));
        self
    }

    fn into_json(mut self) -> String {
        if !self.x_muga.is_empty() {
            self.fields
                .push(("x-muga".to_string(), json_object(self.x_muga)));
        }
        json_object(self.fields)
    }
}

struct SchemaExporter<'a> {
    symbols: &'a SymbolTable,
    decode_mode: SchemaDecodeMode,
    records_by_item: HashMap<PackageItemId, (&'a str, &'a PackageInterfaceRecord)>,
    enums_by_item: HashMap<PackageItemId, (&'a str, &'a PackageInterfaceEnum)>,
    definitions: BTreeMap<String, String>,
    defined_items: HashSet<PackageItemId>,
    visiting_items: HashSet<PackageItemId>,
}

impl<'a> SchemaExporter<'a> {
    fn new(
        interfaces: &'a PackageInterfaceGraph,
        symbols: &'a SymbolTable,
        decode_mode: SchemaDecodeMode,
    ) -> Self {
        let mut records_by_item = HashMap::new();
        let mut enums_by_item = HashMap::new();
        for package in &interfaces.packages {
            for record in &package.records {
                records_by_item.insert(record.item, (package.path.as_str(), record));
            }
            for enumeration in &package.enums {
                enums_by_item.insert(enumeration.item, (package.path.as_str(), enumeration));
            }
        }
        Self {
            symbols,
            decode_mode,
            records_by_item,
            enums_by_item,
            definitions: BTreeMap::new(),
            defined_items: HashSet::new(),
            visiting_items: HashSet::new(),
        }
    }

    fn ensure_target_definition(
        &mut self,
        target: SchemaTarget<'_>,
    ) -> Result<String, Vec<Diagnostic>> {
        match target {
            SchemaTarget::Record(record) => self.ensure_record_definition(record.item),
            SchemaTarget::Enum(enumeration) => self.ensure_enum_definition(enumeration.item),
        }
        .map_err(|diagnostic| vec![diagnostic])
    }

    fn ensure_record_definition(&mut self, item: PackageItemId) -> Result<String, Diagnostic> {
        let (package_path, record) = self.record_for_item(item)?;
        let key = definition_key(package_path, &record.name);
        if self.defined_items.contains(&item) || self.visiting_items.contains(&item) {
            return Ok(key);
        }
        if !record.type_params.is_empty() {
            return Err(schema_export_error(
                format!(
                    "schema export does not support generic record `{}`",
                    qualified_name(package_path, &record.name)
                ),
                record.span,
                "export a concrete public record without type parameters",
            ));
        }

        self.visiting_items.insert(item);
        let mut properties = Vec::new();
        let mut required = Vec::new();
        for field in &record.fields {
            let wire_name = field_wire_name(field);
            let mut schema = self.schema_for_type(&field.ty, &field.json_validation, field.span)?;
            schema = schema
                .with_x_muga("field", json_string(&field.name))
                .with_x_muga("wireName", json_string(wire_name))
                .with_x_muga("aliases", json_string_array(&field.json_aliases));
            if !field.json_validation.is_empty() {
                schema = schema.with_x_muga(
                    "validation",
                    json_string_array(
                        &field
                            .json_validation
                            .iter()
                            .map(JsonDecodeValidationRule::artifact_token)
                            .collect::<Vec<_>>(),
                    ),
                );
            }
            properties.push((wire_name.to_string(), schema.into_json()));
            if self.decode_mode == SchemaDecodeMode::Required && !is_option_type(&field.ty) {
                required.push(wire_name.to_string());
            }
        }

        let mut object = SchemaObject::default()
            .with_field("type", json_string("object"))
            .with_field("properties", json_object(properties))
            .with_x_muga("kind", json_string("record"))
            .with_x_muga(
                "qualifiedName",
                json_string(&qualified_name(package_path, &record.name)),
            )
            .with_x_muga("decodeMode", json_string(self.decode_mode.as_str()));
        if !required.is_empty() {
            object = object.with_field("required", json_string_array(&required));
        }
        if record.json_deny_unknown_fields {
            object = object.with_field("additionalProperties", "false");
        }
        self.visiting_items.remove(&item);
        self.defined_items.insert(item);
        self.definitions.insert(key.clone(), object.into_json());
        Ok(key)
    }

    fn ensure_enum_definition(&mut self, item: PackageItemId) -> Result<String, Diagnostic> {
        let (package_path, enumeration) = self.enum_for_item(item)?;
        let key = definition_key(package_path, &enumeration.name);
        if self.defined_items.contains(&item) || self.visiting_items.contains(&item) {
            return Ok(key);
        }
        if !enumeration.type_params.is_empty() {
            return Err(schema_export_error(
                format!(
                    "schema export does not support generic enum `{}`",
                    qualified_name(package_path, &enumeration.name)
                ),
                enumeration.span,
                "export a concrete public enum without type parameters",
            ));
        }

        self.visiting_items.insert(item);
        let variants_json = self.enum_variants_metadata(enumeration)?;
        let mut object = if enumeration
            .variants
            .iter()
            .all(|variant| variant.payload.is_none())
        {
            SchemaObject::default()
                .with_field("type", json_string("string"))
                .with_field(
                    "enum",
                    json_string_array(
                        &enumeration
                            .variants
                            .iter()
                            .map(|variant| variant_wire_name(variant).to_string())
                            .collect::<Vec<_>>(),
                    ),
                )
        } else {
            let mut alternatives = Vec::new();
            for variant in &enumeration.variants {
                let wire_name = variant_wire_name(variant);
                if let Some(payload) = &variant.payload {
                    let payload_schema = self
                        .schema_for_type(payload, &[], variant.span)?
                        .into_json();
                    alternatives.push(json_object(vec![
                        ("type".to_string(), json_string("object")),
                        (
                            "properties".to_string(),
                            json_object(vec![(wire_name.to_string(), payload_schema)]),
                        ),
                        (
                            "required".to_string(),
                            json_string_array(&[wire_name.to_string()]),
                        ),
                        ("additionalProperties".to_string(), "false".to_string()),
                    ]));
                } else {
                    alternatives.push(json_object(vec![(
                        "const".to_string(),
                        json_string(wire_name),
                    )]));
                }
            }
            SchemaObject::default().with_field("oneOf", json_array(alternatives))
        };
        object = object
            .with_x_muga("kind", json_string("enum"))
            .with_x_muga(
                "qualifiedName",
                json_string(&qualified_name(package_path, &enumeration.name)),
            )
            .with_x_muga("variants", variants_json);
        self.visiting_items.remove(&item);
        self.defined_items.insert(item);
        self.definitions.insert(key.clone(), object.into_json());
        Ok(key)
    }

    fn schema_for_type(
        &mut self,
        ty: &TypeInfo,
        validation: &[JsonDecodeValidationRule],
        span: Span,
    ) -> Result<SchemaObject, Diagnostic> {
        match ty {
            TypeInfo::String => Ok(apply_string_validation(
                SchemaObject::default()
                    .with_field("type", json_string("string"))
                    .with_x_muga("type", json_string("String")),
                validation,
            )),
            TypeInfo::Int => Ok(apply_int_validation(
                SchemaObject::default()
                    .with_field("type", json_string("integer"))
                    .with_x_muga("type", json_string("Int"))
                    .with_x_muga("intBits", "64"),
                validation,
            )),
            TypeInfo::Bool => Ok(SchemaObject::default()
                .with_field("type", json_string("boolean"))
                .with_x_muga("type", json_string("Bool"))),
            TypeInfo::Option(item) => {
                let item_schema = self.schema_for_type(item, validation, span)?.into_json();
                Ok(SchemaObject::default()
                    .with_field(
                        "anyOf",
                        json_array(vec![
                            item_schema,
                            json_object(vec![("type".to_string(), json_string("null"))]),
                        ]),
                    )
                    .with_x_muga("optional", "true"))
            }
            TypeInfo::List(item) => Ok(SchemaObject::default()
                .with_field("type", json_string("array"))
                .with_field("items", self.schema_for_type(item, &[], span)?.into_json())),
            TypeInfo::Map(key, value) => {
                if !matches!(key.as_ref(), TypeInfo::String) {
                    return Err(self.unsupported_type_error(ty, span));
                }
                Ok(SchemaObject::default()
                    .with_field("type", json_string("object"))
                    .with_field(
                        "additionalProperties",
                        self.schema_for_type(value, &[], span)?.into_json(),
                    )
                    .with_x_muga("mapKey", json_string("String")))
            }
            TypeInfo::PackageRecord { item, args, .. } => {
                if !args.is_empty() {
                    return Err(self.unsupported_type_error(ty, span));
                }
                Ok(SchemaObject::default().with_field(
                    "$ref",
                    json_string(&format!(
                        "#/$defs/{}",
                        self.ensure_record_definition(*item)?
                    )),
                ))
            }
            TypeInfo::PackageEnum { item, args, .. } => {
                if self.is_std_json_value(*item) {
                    return Ok(SchemaObject::default()
                        .with_x_muga("type", json_string("std::json::Value")));
                }
                if !args.is_empty() {
                    return Err(self.unsupported_type_error(ty, span));
                }
                Ok(SchemaObject::default().with_field(
                    "$ref",
                    json_string(&format!("#/$defs/{}", self.ensure_enum_definition(*item)?)),
                ))
            }
            _ => Err(self.unsupported_type_error(ty, span)),
        }
    }

    fn enum_variants_metadata(
        &mut self,
        enumeration: &PackageInterfaceEnum,
    ) -> Result<String, Diagnostic> {
        let mut variants = Vec::new();
        for variant in &enumeration.variants {
            let mut fields = vec![
                ("name".to_string(), json_string(&variant.name)),
                (
                    "wireName".to_string(),
                    json_string(variant_wire_name(variant)),
                ),
                (
                    "aliases".to_string(),
                    json_string_array(&variant.json_aliases),
                ),
            ];
            if let Some(payload) = &variant.payload {
                fields.push((
                    "payload".to_string(),
                    self.schema_for_type(payload, &[], variant.span)?
                        .into_json(),
                ));
            } else {
                fields.push(("payload".to_string(), "null".to_string()));
            }
            variants.push(json_object(fields));
        }
        Ok(json_array(variants))
    }

    fn render_document(&self, package_path: &str, root_refs: Vec<String>) -> String {
        let mut fields = vec![
            (
                "$schema".to_string(),
                json_string(JSON_SCHEMA_DRAFT_2020_12),
            ),
            (
                "$id".to_string(),
                json_string(&format!("muga:{package_path}")),
            ),
        ];
        if root_refs.len() == 1 {
            fields.push((
                "$ref".to_string(),
                json_string(&format!("#/$defs/{}", root_refs[0])),
            ));
        }
        fields.push((
            "$defs".to_string(),
            json_object(
                self.definitions
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            ),
        ));
        fields.push((
            "x-muga".to_string(),
            json_object(vec![
                ("package".to_string(), json_string(package_path)),
                (
                    "decodeMode".to_string(),
                    json_string(self.decode_mode.as_str()),
                ),
                ("exports".to_string(), json_string_array(&root_refs)),
            ]),
        ));
        json_object(fields)
    }

    fn record_for_item(
        &self,
        item: PackageItemId,
    ) -> Result<(&'a str, &'a PackageInterfaceRecord), Diagnostic> {
        self.records_by_item.get(&item).copied().ok_or_else(|| {
            schema_export_error(
                format!(
                    "record item {} is not available for schema export",
                    item.as_u32()
                ),
                Span::default(),
                "export a public concrete record available in package interfaces",
            )
        })
    }

    fn enum_for_item(
        &self,
        item: PackageItemId,
    ) -> Result<(&'a str, &'a PackageInterfaceEnum), Diagnostic> {
        self.enums_by_item.get(&item).copied().ok_or_else(|| {
            schema_export_error(
                format!(
                    "enum item {} is not available for schema export",
                    item.as_u32()
                ),
                Span::default(),
                "export a public concrete enum available in package interfaces",
            )
        })
    }

    fn is_std_json_value(&self, item: PackageItemId) -> bool {
        self.enums_by_item
            .get(&item)
            .is_some_and(|(package, enumeration)| {
                *package == std_package::JSON_PACKAGE && enumeration.name == "Value"
            })
    }

    fn unsupported_type_error(&self, ty: &TypeInfo, span: Span) -> Diagnostic {
        schema_export_error(
            format!(
                "type `{}` is not supported by JSON/config schema export",
                doc::render_type_info(ty, self.symbols)
            ),
            span,
            "export a concrete public record or enum composed of String, Int, Bool, Option, List, Map[String, T], std::json::Value, and supported concrete public records/enums",
        )
    }
}

fn apply_string_validation(
    mut schema: SchemaObject,
    validation: &[JsonDecodeValidationRule],
) -> SchemaObject {
    let mut min_len = None;
    let mut max_len = None;
    for rule in validation {
        match rule {
            JsonDecodeValidationRule::NonEmpty => {
                min_len = Some(min_len.unwrap_or(0).max(1));
            }
            JsonDecodeValidationRule::MinLen(value) => {
                min_len = Some(min_len.unwrap_or(*value).max(*value));
            }
            JsonDecodeValidationRule::MaxLen(value) => max_len = Some(*value),
            JsonDecodeValidationRule::Min(_) | JsonDecodeValidationRule::Max(_) => {}
        }
    }
    if let Some(value) = min_len {
        schema = schema.with_field("minLength", value.to_string());
    }
    if let Some(value) = max_len {
        schema = schema.with_field("maxLength", value.to_string());
    }
    schema
}

fn apply_int_validation(
    mut schema: SchemaObject,
    validation: &[JsonDecodeValidationRule],
) -> SchemaObject {
    for rule in validation {
        match rule {
            JsonDecodeValidationRule::Min(value) => {
                schema = schema.with_field("minimum", value.to_string());
            }
            JsonDecodeValidationRule::Max(value) => {
                schema = schema.with_field("maximum", value.to_string());
            }
            JsonDecodeValidationRule::NonEmpty
            | JsonDecodeValidationRule::MinLen(_)
            | JsonDecodeValidationRule::MaxLen(_) => {}
        }
    }
    schema
}

fn is_option_type(ty: &TypeInfo) -> bool {
    matches!(ty, TypeInfo::Option(_))
}

fn field_wire_name(field: &PackageInterfaceField) -> &str {
    field.json_rename.as_deref().unwrap_or(&field.name)
}

fn variant_wire_name(variant: &PackageInterfaceEnumVariant) -> &str {
    variant.json_rename.as_deref().unwrap_or(&variant.name)
}

fn definition_key(package_path: &str, name: &str) -> String {
    qualified_name(package_path, name)
}

fn qualified_name(package_path: &str, name: &str) -> String {
    format!("{package_path}::{name}")
}

fn schema_export_error(
    message: impl Into<String>,
    span: Span,
    suggestion: impl Into<String>,
) -> Diagnostic {
    Diagnostic::new("T029", message, span).with_suggestion(suggestion)
}

fn json_object(fields: Vec<(String, String)>) -> String {
    let mut output = String::new();
    output.push('{');
    for (index, (key, value)) in fields.into_iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&json_string(&key));
        output.push(':');
        output.push_str(&value);
    }
    output.push('}');
    output
}

fn json_array(values: Vec<String>) -> String {
    let mut output = String::new();
    output.push('[');
    for (index, value) in values.into_iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&value);
    }
    output.push(']');
    output
}

fn json_string_array(values: &[String]) -> String {
    json_array(values.iter().map(|value| json_string(value)).collect())
}

fn json_string(value: &str) -> String {
    let mut output = String::new();
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            ch if ch.is_control() => {
                output.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => output.push(ch),
        }
    }
    output.push('"');
    output
}
