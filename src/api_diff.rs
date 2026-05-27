use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::{
    interface::{
        OpaqueHandleFacts, PackageInterface, PackageInterfaceEnum, PackageInterfaceEnumVariant,
        PackageInterfaceField, PackageInterfaceFunction, PackageInterfaceGraph,
        PackageInterfaceOpaqueType, PackageInterfaceParam, PackageInterfaceParamMode,
        PackageInterfaceRecord,
    },
    package::PackageItemKind,
    symbol::{Symbol, SymbolTable},
    types::{FunctionTypeInfo, TypeInfo},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageApiDiff {
    pub package: String,
    pub status: PackageApiDiffStatus,
    pub summary: PackageApiDiffSummary,
    pub changes: Vec<PackageApiDiffChange>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageApiDiffStatus {
    Compatible,
    SourceCompatible,
    Breaking,
    Unknown,
}

impl PackageApiDiffStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compatible => "compatible",
            Self::SourceCompatible => "sourceCompatible",
            Self::Breaking => "breaking",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageApiDiffSummary {
    pub compatible: usize,
    pub source_compatible: usize,
    pub breaking: usize,
    pub unknown: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageApiDiffChange {
    pub classification: PackageApiDiffClassification,
    pub kind: String,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageApiDiffClassification {
    SourceCompatible,
    Breaking,
    Unknown,
}

impl PackageApiDiffClassification {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceCompatible => "sourceCompatible",
            Self::Breaking => "breaking",
            Self::Unknown => "unknown",
        }
    }
}

pub fn diff_package_interfaces(
    old: &PackageInterfaceGraph,
    new: &PackageInterfaceGraph,
    package_path: &str,
    symbols: &SymbolTable,
) -> PackageApiDiff {
    let old_context = ApiDiffContext::new(old, symbols);
    let new_context = ApiDiffContext::new(new, symbols);
    let mut changes = Vec::new();

    let old_package = old.package_by_path(package_path);
    let new_package = new.package_by_path(package_path);

    match (old_package, new_package) {
        (None, None) => changes.push(PackageApiDiffChange {
            classification: PackageApiDiffClassification::Unknown,
            kind: "package-not-found".to_string(),
            path: package_path.to_string(),
            message: format!("package `{package_path}` is not present in either interface graph"),
        }),
        (None, Some(_)) => changes.push(PackageApiDiffChange {
            classification: PackageApiDiffClassification::SourceCompatible,
            kind: "package-added".to_string(),
            path: package_path.to_string(),
            message: format!("public package `{package_path}` was added"),
        }),
        (Some(_), None) => changes.push(PackageApiDiffChange {
            classification: PackageApiDiffClassification::Breaking,
            kind: "package-removed".to_string(),
            path: package_path.to_string(),
            message: format!("public package `{package_path}` was removed"),
        }),
        (Some(old_package), Some(new_package)) => {
            if old.stable_hash_for_package(package_path, symbols)
                == new.stable_hash_for_package(package_path, symbols)
            {
                return PackageApiDiff {
                    package: package_path.to_string(),
                    status: PackageApiDiffStatus::Compatible,
                    summary: PackageApiDiffSummary {
                        compatible: 1,
                        ..PackageApiDiffSummary::default()
                    },
                    changes,
                };
            }

            diff_package(
                old_package,
                new_package,
                &old_context,
                &new_context,
                &mut changes,
            );

            if changes.is_empty() {
                changes.push(PackageApiDiffChange {
                    classification: PackageApiDiffClassification::SourceCompatible,
                    kind: "public-metadata-changed".to_string(),
                    path: package_path.to_string(),
                    message: format!(
                        "public package `{package_path}` metadata changed without a source-breaking shape change"
                    ),
                });
            }
        }
    }

    changes.sort_by(|left, right| {
        change_rank(left.classification)
            .cmp(&change_rank(right.classification))
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.kind.cmp(&right.kind))
    });
    let summary = PackageApiDiffSummary::from_changes(&changes);
    let status = summary.status();

    PackageApiDiff {
        package: package_path.to_string(),
        status,
        summary,
        changes,
    }
}

impl PackageApiDiffSummary {
    fn from_changes(changes: &[PackageApiDiffChange]) -> Self {
        let mut summary = Self::default();
        for change in changes {
            match change.classification {
                PackageApiDiffClassification::SourceCompatible => summary.source_compatible += 1,
                PackageApiDiffClassification::Breaking => summary.breaking += 1,
                PackageApiDiffClassification::Unknown => summary.unknown += 1,
            }
        }
        summary
    }

    fn status(&self) -> PackageApiDiffStatus {
        if self.unknown > 0 {
            PackageApiDiffStatus::Unknown
        } else if self.breaking > 0 {
            PackageApiDiffStatus::Breaking
        } else if self.source_compatible > 0 {
            PackageApiDiffStatus::SourceCompatible
        } else {
            PackageApiDiffStatus::Compatible
        }
    }
}

fn change_rank(classification: PackageApiDiffClassification) -> u8 {
    match classification {
        PackageApiDiffClassification::Unknown => 0,
        PackageApiDiffClassification::Breaking => 1,
        PackageApiDiffClassification::SourceCompatible => 2,
    }
}

fn diff_package(
    old: &PackageInterface,
    new: &PackageInterface,
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    changes: &mut Vec<PackageApiDiffChange>,
) {
    push_duplicate_name_changes(old, "old", changes);
    push_duplicate_name_changes(new, "new", changes);
    push_kind_changed_changes(old, new, changes);
    diff_records(old, new, old_context, new_context, changes);
    diff_enums(old, new, old_context, new_context, changes);
    diff_opaque_types(old, new, old_context, new_context, changes);
    diff_functions(old, new, old_context, new_context, changes);
}

fn push_duplicate_name_changes(
    package: &PackageInterface,
    label: &str,
    changes: &mut Vec<PackageApiDiffChange>,
) {
    for (kind, names) in [
        (
            "record",
            item_names(package.records.iter().map(|record| &record.name)),
        ),
        (
            "enum",
            item_names(package.enums.iter().map(|item| &item.name)),
        ),
        (
            "opaque-type",
            item_names(package.opaque_types.iter().map(|item| &item.name)),
        ),
        (
            "function",
            item_names(package.functions.iter().map(|item| &item.name)),
        ),
    ] {
        for (name, count) in names {
            if count > 1 {
                let path = item_path(&package.path, &name);
                changes.push(PackageApiDiffChange {
                    classification: PackageApiDiffClassification::Unknown,
                    kind: "duplicate-public-name".to_string(),
                    path,
                    message: format!(
                        "{label} interface for `{}` contains duplicate public {kind} `{name}`",
                        package.path
                    ),
                });
            }
        }
    }
}

fn item_names<'a>(names: impl Iterator<Item = &'a String>) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for name in names {
        *out.entry(name.clone()).or_insert(0) += 1;
    }
    out
}

fn push_kind_changed_changes(
    old: &PackageInterface,
    new: &PackageInterface,
    changes: &mut Vec<PackageApiDiffChange>,
) {
    let old_names = public_item_kinds_by_name(old);
    let new_names = public_item_kinds_by_name(new);
    for (name, old_kinds) in old_names {
        let Some(new_kinds) = new_names.get(&name) else {
            continue;
        };
        if old_kinds.is_disjoint(new_kinds) {
            changes.push(PackageApiDiffChange {
                classification: PackageApiDiffClassification::Breaking,
                kind: "public-item-kind-changed".to_string(),
                path: item_path(&old.path, &name),
                message: format!(
                    "public item `{}` changed kind from {} to {}",
                    item_path(&old.path, &name),
                    kind_set_label(&old_kinds),
                    kind_set_label(new_kinds)
                ),
            });
        }
    }
}

fn public_item_kinds_by_name(
    package: &PackageInterface,
) -> BTreeMap<String, BTreeSet<&'static str>> {
    let mut out: BTreeMap<String, BTreeSet<&'static str>> = BTreeMap::new();
    for record in &package.records {
        out.entry(record.name.clone()).or_default().insert("record");
    }
    for item in &package.enums {
        out.entry(item.name.clone()).or_default().insert("enum");
    }
    for item in &package.opaque_types {
        out.entry(item.name.clone())
            .or_default()
            .insert("opaque type");
    }
    for item in &package.functions {
        out.entry(item.name.clone()).or_default().insert("function");
    }
    out
}

fn kind_set_label(kinds: &BTreeSet<&'static str>) -> String {
    kinds.iter().copied().collect::<Vec<_>>().join(", ")
}

fn diff_records(
    old: &PackageInterface,
    new: &PackageInterface,
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    changes: &mut Vec<PackageApiDiffChange>,
) {
    diff_named_items(
        "record",
        &old.records,
        &new.records,
        |record| &record.name,
        |name| item_path(&old.path, name),
        changes,
        |old_record, new_record, changes| {
            diff_record(
                old_record,
                new_record,
                old_context,
                new_context,
                &old.path,
                changes,
            );
        },
    );
}

fn diff_record(
    old: &PackageInterfaceRecord,
    new: &PackageInterfaceRecord,
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    package_path: &str,
    changes: &mut Vec<PackageApiDiffChange>,
) {
    let path = item_path(package_path, &old.name);
    diff_type_params(
        "record-type-parameters",
        &path,
        &old.type_params,
        &new.type_params,
        changes,
    );
    let type_context = TypeParamContext::new(&old.type_params, &new.type_params);

    diff_named_items(
        "record-field",
        &old.fields,
        &new.fields,
        |field| &field.name,
        |name| member_path(&path, name),
        changes,
        |old_field, new_field, changes| {
            diff_record_field(
                old_field,
                new_field,
                old_context,
                new_context,
                &type_context,
                &path,
                changes,
            );
        },
    );

    let old_order = old
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<Vec<_>>();
    let new_order = new
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<Vec<_>>();
    let old_names = old_order.iter().copied().collect::<BTreeSet<_>>();
    let new_names = new_order.iter().copied().collect::<BTreeSet<_>>();
    if old_names == new_names && old_order != new_order {
        changes.push(PackageApiDiffChange {
            classification: PackageApiDiffClassification::SourceCompatible,
            kind: "record-fields-reordered".to_string(),
            path,
            message: format!("public record `{}` reordered fields", old.name),
        });
    }
}

fn diff_record_field(
    old: &PackageInterfaceField,
    new: &PackageInterfaceField,
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    type_context: &TypeParamContext<'_>,
    record_path: &str,
    changes: &mut Vec<PackageApiDiffChange>,
) {
    let field_path = member_path(record_path, &old.name);
    if !same_type_info(&old.ty, &new.ty, old_context, new_context, type_context) {
        changes.push(PackageApiDiffChange {
            classification: PackageApiDiffClassification::Breaking,
            kind: "record-field-type-changed".to_string(),
            path: field_path,
            message: format!(
                "public record field `{}` type changed from `{}` to `{}`",
                old.name,
                render_type_info(&old.ty, old_context, type_context.old_type_params),
                render_type_info(&new.ty, new_context, type_context.new_type_params)
            ),
        });
    }
}

fn diff_enums(
    old: &PackageInterface,
    new: &PackageInterface,
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    changes: &mut Vec<PackageApiDiffChange>,
) {
    diff_named_items(
        "enum",
        &old.enums,
        &new.enums,
        |item| &item.name,
        |name| item_path(&old.path, name),
        changes,
        |old_enum, new_enum, changes| {
            diff_enum(
                old_enum,
                new_enum,
                old_context,
                new_context,
                &old.path,
                changes,
            );
        },
    );
}

fn diff_enum(
    old: &PackageInterfaceEnum,
    new: &PackageInterfaceEnum,
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    package_path: &str,
    changes: &mut Vec<PackageApiDiffChange>,
) {
    let path = item_path(package_path, &old.name);
    diff_type_params(
        "enum-type-parameters",
        &path,
        &old.type_params,
        &new.type_params,
        changes,
    );
    let type_context = TypeParamContext::new(&old.type_params, &new.type_params);

    diff_named_items(
        "enum-variant",
        &old.variants,
        &new.variants,
        |variant| &variant.name,
        |name| member_path(&path, name),
        changes,
        |old_variant, new_variant, changes| {
            diff_enum_variant(
                old_variant,
                new_variant,
                old_context,
                new_context,
                &type_context,
                &path,
                changes,
            );
        },
    );

    let old_order = old
        .variants
        .iter()
        .map(|variant| variant.name.as_str())
        .collect::<Vec<_>>();
    let new_order = new
        .variants
        .iter()
        .map(|variant| variant.name.as_str())
        .collect::<Vec<_>>();
    let old_names = old_order.iter().copied().collect::<BTreeSet<_>>();
    let new_names = new_order.iter().copied().collect::<BTreeSet<_>>();
    if old_names == new_names && old_order != new_order {
        changes.push(PackageApiDiffChange {
            classification: PackageApiDiffClassification::SourceCompatible,
            kind: "enum-variants-reordered".to_string(),
            path,
            message: format!("public enum `{}` reordered variants", old.name),
        });
    }
}

fn diff_enum_variant(
    old: &PackageInterfaceEnumVariant,
    new: &PackageInterfaceEnumVariant,
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    type_context: &TypeParamContext<'_>,
    enum_path: &str,
    changes: &mut Vec<PackageApiDiffChange>,
) {
    let same_payload = match (&old.payload, &new.payload) {
        (None, None) => true,
        (Some(old_payload), Some(new_payload)) => same_type_info(
            old_payload,
            new_payload,
            old_context,
            new_context,
            type_context,
        ),
        _ => false,
    };
    if !same_payload {
        let old_payload = old
            .payload
            .as_ref()
            .map(|payload| render_type_info(payload, old_context, type_context.old_type_params))
            .unwrap_or_else(|| "none".to_string());
        let new_payload = new
            .payload
            .as_ref()
            .map(|payload| render_type_info(payload, new_context, type_context.new_type_params))
            .unwrap_or_else(|| "none".to_string());
        changes.push(PackageApiDiffChange {
            classification: PackageApiDiffClassification::Breaking,
            kind: "enum-variant-payload-changed".to_string(),
            path: member_path(enum_path, &old.name),
            message: format!(
                "public enum variant `{}` payload changed from `{old_payload}` to `{new_payload}`",
                old.name
            ),
        });
    }
}

fn diff_opaque_types(
    old: &PackageInterface,
    new: &PackageInterface,
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    changes: &mut Vec<PackageApiDiffChange>,
) {
    diff_named_items(
        "opaque-type",
        &old.opaque_types,
        &new.opaque_types,
        |item| &item.name,
        |name| item_path(&old.path, name),
        changes,
        |old_opaque, new_opaque, changes| {
            diff_opaque_type(
                old_opaque,
                new_opaque,
                old_context,
                new_context,
                &old.path,
                changes,
            );
        },
    );
}

fn diff_opaque_type(
    old: &PackageInterfaceOpaqueType,
    new: &PackageInterfaceOpaqueType,
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    package_path: &str,
    changes: &mut Vec<PackageApiDiffChange>,
) {
    let path = item_path(package_path, &old.name);
    diff_handle_facts(
        &old.handle_facts,
        &new.handle_facts,
        old_context,
        new_context,
        &path,
        changes,
    );
}

fn diff_handle_facts(
    old: &OpaqueHandleFacts,
    new: &OpaqueHandleFacts,
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    path: &str,
    changes: &mut Vec<PackageApiDiffChange>,
) {
    for (name, old_value, new_value) in [
        ("copyable", old.copyable, new.copyable),
        ("cloneable", old.cloneable, new.cloneable),
        ("sendable", old.sendable, new.sendable),
        ("shareable", old.shareable, new.shareable),
    ] {
        if !old_value && new_value {
            changes.push(PackageApiDiffChange {
                classification: PackageApiDiffClassification::SourceCompatible,
                kind: "opaque-handle-capability-added".to_string(),
                path: path.to_string(),
                message: format!("public opaque type `{path}` added `{name}` capability"),
            });
        } else if old_value && !new_value {
            changes.push(PackageApiDiffChange {
                classification: PackageApiDiffClassification::Breaking,
                kind: "opaque-handle-capability-removed".to_string(),
                path: path.to_string(),
                message: format!("public opaque type `{path}` removed `{name}` capability"),
            });
        }
    }

    if old.closeable != new.closeable
        || close_function_identity(old.close_function, old_context)
            != close_function_identity(new.close_function, new_context)
    {
        changes.push(PackageApiDiffChange {
            classification: PackageApiDiffClassification::Breaking,
            kind: "opaque-handle-close-policy-changed".to_string(),
            path: path.to_string(),
            message: format!("public opaque type `{path}` changed close policy"),
        });
    }

    for (name, old_value, new_value) in [
        ("runtime_backed", old.runtime_backed, new.runtime_backed),
        (
            "structurally_comparable",
            old.structurally_comparable,
            new.structurally_comparable,
        ),
        ("serializable", old.serializable, new.serializable),
    ] {
        if old_value != new_value {
            changes.push(PackageApiDiffChange {
                classification: PackageApiDiffClassification::Unknown,
                kind: "opaque-handle-fact-changed".to_string(),
                path: path.to_string(),
                message: format!(
                    "public opaque type `{path}` changed `{name}` handle fact without an API-diff compatibility rule"
                ),
            });
        }
    }
}

fn close_function_identity(
    item: Option<crate::identity::PackageItemId>,
    context: &ApiDiffContext<'_>,
) -> Option<ApiItemIdentity> {
    item.and_then(|item| context.item_identity(PackageItemKind::Function, item))
        .cloned()
}

fn diff_functions(
    old: &PackageInterface,
    new: &PackageInterface,
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    changes: &mut Vec<PackageApiDiffChange>,
) {
    diff_named_items(
        "function",
        &old.functions,
        &new.functions,
        |function| &function.name,
        |name| item_path(&old.path, name),
        changes,
        |old_function, new_function, changes| {
            diff_function(
                old_function,
                new_function,
                old_context,
                new_context,
                &old.path,
                changes,
            );
        },
    );
}

fn diff_function(
    old: &PackageInterfaceFunction,
    new: &PackageInterfaceFunction,
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    package_path: &str,
    changes: &mut Vec<PackageApiDiffChange>,
) {
    let path = item_path(package_path, &old.name);
    diff_type_params(
        "function-type-parameters",
        &path,
        &old.type_params,
        &new.type_params,
        changes,
    );
    let type_context = TypeParamContext::new(&old.type_params, &new.type_params);
    if old.params.len() != new.params.len() {
        changes.push(PackageApiDiffChange {
            classification: PackageApiDiffClassification::Breaking,
            kind: "function-parameter-count-changed".to_string(),
            path: path.clone(),
            message: format!(
                "public function `{}` parameter count changed from {} to {}",
                old.name,
                old.params.len(),
                new.params.len()
            ),
        });
    }

    for (index, (old_param, new_param)) in old.params.iter().zip(&new.params).enumerate() {
        diff_function_param(
            old_param,
            new_param,
            index,
            (old_context, new_context, &type_context),
            &path,
            changes,
        );
    }

    if !same_type_info(&old.ret, &new.ret, old_context, new_context, &type_context) {
        changes.push(PackageApiDiffChange {
            classification: PackageApiDiffClassification::Breaking,
            kind: "function-return-type-changed".to_string(),
            path,
            message: format!(
                "public function `{}` return type changed from `{}` to `{}`",
                old.name,
                render_type_info(&old.ret, old_context, &old.type_params),
                render_type_info(&new.ret, new_context, &new.type_params)
            ),
        });
    }
}

fn diff_function_param(
    old: &PackageInterfaceParam,
    new: &PackageInterfaceParam,
    index: usize,
    contexts: (
        &ApiDiffContext<'_>,
        &ApiDiffContext<'_>,
        &TypeParamContext<'_>,
    ),
    function_path: &str,
    changes: &mut Vec<PackageApiDiffChange>,
) {
    let (old_context, new_context, type_context) = contexts;
    let path = member_path(function_path, &format!("param{}", index + 1));
    if old.name != new.name {
        changes.push(PackageApiDiffChange {
            classification: PackageApiDiffClassification::SourceCompatible,
            kind: "function-parameter-renamed".to_string(),
            path: path.clone(),
            message: format!(
                "public function parameter {} renamed from `{}` to `{}`",
                index + 1,
                old.name,
                new.name
            ),
        });
    }
    if !same_type_info(&old.ty, &new.ty, old_context, new_context, type_context) {
        changes.push(PackageApiDiffChange {
            classification: PackageApiDiffClassification::Breaking,
            kind: "function-parameter-type-changed".to_string(),
            path: path.clone(),
            message: format!(
                "public function parameter {} type changed from `{}` to `{}`",
                index + 1,
                render_type_info(&old.ty, old_context, type_context.old_type_params),
                render_type_info(&new.ty, new_context, type_context.new_type_params)
            ),
        });
    }
    match (old.mode, new.mode) {
        (PackageInterfaceParamMode::Borrow, PackageInterfaceParamMode::Consume) => {
            changes.push(PackageApiDiffChange {
                classification: PackageApiDiffClassification::Breaking,
                kind: "function-parameter-mode-tightened".to_string(),
                path,
                message: format!(
                    "public function parameter {} changed from `borrow` to `consume`",
                    index + 1
                ),
            });
        }
        (PackageInterfaceParamMode::Consume, PackageInterfaceParamMode::Borrow) => {
            changes.push(PackageApiDiffChange {
                classification: PackageApiDiffClassification::SourceCompatible,
                kind: "function-parameter-mode-relaxed".to_string(),
                path,
                message: format!(
                    "public function parameter {} changed from `consume` to `borrow`",
                    index + 1
                ),
            });
        }
        _ => {}
    }
}

fn diff_type_params(
    kind: &str,
    path: &str,
    old: &[String],
    new: &[String],
    changes: &mut Vec<PackageApiDiffChange>,
) {
    if old.len() != new.len() {
        changes.push(PackageApiDiffChange {
            classification: PackageApiDiffClassification::Breaking,
            kind: format!("{kind}-arity-changed"),
            path: path.to_string(),
            message: format!(
                "public item `{path}` type parameter arity changed from {} to {}",
                old.len(),
                new.len()
            ),
        });
    } else if old != new {
        changes.push(PackageApiDiffChange {
            classification: PackageApiDiffClassification::SourceCompatible,
            kind: format!("{kind}-renamed"),
            path: path.to_string(),
            message: format!("public item `{path}` renamed type parameters"),
        });
    }
}

fn diff_named_items<T>(
    item_kind: &str,
    old: &[T],
    new: &[T],
    name: impl Fn(&T) -> &String,
    path_for_name: impl Fn(&str) -> String,
    changes: &mut Vec<PackageApiDiffChange>,
    mut compare: impl FnMut(&T, &T, &mut Vec<PackageApiDiffChange>),
) {
    let old_by_name = old
        .iter()
        .map(|item| (name(item).clone(), item))
        .collect::<BTreeMap<_, _>>();
    let new_by_name = new
        .iter()
        .map(|item| (name(item).clone(), item))
        .collect::<BTreeMap<_, _>>();
    for old_name in old_by_name.keys() {
        if !new_by_name.contains_key(old_name) {
            changes.push(PackageApiDiffChange {
                classification: PackageApiDiffClassification::Breaking,
                kind: format!("{item_kind}-removed"),
                path: path_for_name(old_name),
                message: format!(
                    "public {item_kind} `{}` was removed",
                    path_for_name(old_name)
                ),
            });
        }
    }
    for new_name in new_by_name.keys() {
        if !old_by_name.contains_key(new_name) {
            let classification = if item_kind == "record-field" || item_kind == "enum-variant" {
                PackageApiDiffClassification::Breaking
            } else {
                PackageApiDiffClassification::SourceCompatible
            };
            changes.push(PackageApiDiffChange {
                classification,
                kind: format!("{item_kind}-added"),
                path: path_for_name(new_name),
                message: format!("public {item_kind} `{}` was added", path_for_name(new_name)),
            });
        }
    }
    for (old_name, old_item) in old_by_name {
        if let Some(new_item) = new_by_name.get(&old_name) {
            compare(old_item, new_item, changes);
        }
    }
}

fn same_type_info(
    old: &TypeInfo,
    new: &TypeInfo,
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    type_context: &TypeParamContext<'_>,
) -> bool {
    match (old, new) {
        (TypeInfo::Int, TypeInfo::Int)
        | (TypeInfo::Bool, TypeInfo::Bool)
        | (TypeInfo::String, TypeInfo::String)
        | (TypeInfo::Unit, TypeInfo::Unit)
        | (TypeInfo::Unknown, TypeInfo::Unknown)
        | (TypeInfo::Error, TypeInfo::Error) => true,
        (TypeInfo::GenericParam(old_symbol), TypeInfo::GenericParam(new_symbol)) => {
            same_generic_param(
                *old_symbol,
                *new_symbol,
                old_context,
                new_context,
                type_context,
            )
        }
        (TypeInfo::Record(old_symbol, old_args), TypeInfo::Record(new_symbol, new_args))
        | (
            TypeInfo::Enum {
                symbol: old_symbol,
                args: old_args,
            },
            TypeInfo::Enum {
                symbol: new_symbol,
                args: new_args,
            },
        ) => {
            old_context.symbol_name(*old_symbol) == new_context.symbol_name(*new_symbol)
                && same_type_args(old_args, new_args, old_context, new_context, type_context)
        }
        (
            TypeInfo::PackageRecord {
                item: old_item,
                args: old_args,
                ..
            },
            TypeInfo::PackageRecord {
                item: new_item,
                args: new_args,
                ..
            },
        ) => {
            old_context.item_identity(PackageItemKind::Record, *old_item)
                == new_context.item_identity(PackageItemKind::Record, *new_item)
                && same_type_args(old_args, new_args, old_context, new_context, type_context)
        }
        (
            TypeInfo::PackageEnum {
                item: old_item,
                args: old_args,
                ..
            },
            TypeInfo::PackageEnum {
                item: new_item,
                args: new_args,
                ..
            },
        ) => {
            old_context.item_identity(PackageItemKind::Enum, *old_item)
                == new_context.item_identity(PackageItemKind::Enum, *new_item)
                && same_type_args(old_args, new_args, old_context, new_context, type_context)
        }
        (
            TypeInfo::PackageOpaque { item: old_item, .. },
            TypeInfo::PackageOpaque { item: new_item, .. },
        ) => {
            old_context.item_identity(PackageItemKind::OpaqueType, *old_item)
                == new_context.item_identity(PackageItemKind::OpaqueType, *new_item)
        }
        (TypeInfo::List(old_item), TypeInfo::List(new_item))
        | (TypeInfo::Option(old_item), TypeInfo::Option(new_item)) => {
            same_type_info(old_item, new_item, old_context, new_context, type_context)
        }
        (TypeInfo::Map(old_key, old_value), TypeInfo::Map(new_key, new_value))
        | (TypeInfo::Result(old_key, old_value), TypeInfo::Result(new_key, new_value)) => {
            same_type_info(old_key, new_key, old_context, new_context, type_context)
                && same_type_info(old_value, new_value, old_context, new_context, type_context)
        }
        (TypeInfo::Function(old_function), TypeInfo::Function(new_function)) => {
            same_function_type_info(
                old_function,
                new_function,
                old_context,
                new_context,
                type_context,
            )
        }
        (TypeInfo::Builtin(old_builtin), TypeInfo::Builtin(new_builtin)) => {
            old_builtin == new_builtin
        }
        (
            TypeInfo::EnumConstructor {
                enum_symbol: old_symbol,
                enum_item: old_item,
                variant: old_variant,
            },
            TypeInfo::EnumConstructor {
                enum_symbol: new_symbol,
                enum_item: new_item,
                variant: new_variant,
            },
        ) => {
            let same_enum = match (old_item, new_item) {
                (Some(old_item), Some(new_item)) => {
                    old_context.item_identity(PackageItemKind::Enum, *old_item)
                        == new_context.item_identity(PackageItemKind::Enum, *new_item)
                }
                (None, None) => {
                    old_context.symbol_name(*old_symbol) == new_context.symbol_name(*new_symbol)
                }
                _ => false,
            };
            same_enum
                && old_context.symbol_name(*old_variant) == new_context.symbol_name(*new_variant)
        }
        _ => false,
    }
}

fn same_function_type_info(
    old: &FunctionTypeInfo,
    new: &FunctionTypeInfo,
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    type_context: &TypeParamContext<'_>,
) -> bool {
    old.params.len() == new.params.len()
        && old
            .params
            .iter()
            .zip(&new.params)
            .all(|(old_param, new_param)| {
                same_type_info(old_param, new_param, old_context, new_context, type_context)
            })
        && same_type_info(&old.ret, &new.ret, old_context, new_context, type_context)
}

fn same_type_args(
    old: &[TypeInfo],
    new: &[TypeInfo],
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    type_context: &TypeParamContext<'_>,
) -> bool {
    old.len() == new.len()
        && old.iter().zip(new).all(|(old_arg, new_arg)| {
            same_type_info(old_arg, new_arg, old_context, new_context, type_context)
        })
}

fn same_generic_param(
    old: Symbol,
    new: Symbol,
    old_context: &ApiDiffContext<'_>,
    new_context: &ApiDiffContext<'_>,
    type_context: &TypeParamContext<'_>,
) -> bool {
    let old_name = old_context.symbol_name(old);
    let new_name = new_context.symbol_name(new);
    match (
        type_context
            .old_type_params
            .iter()
            .position(|name| name == old_name),
        type_context
            .new_type_params
            .iter()
            .position(|name| name == new_name),
    ) {
        (Some(old_position), Some(new_position)) => old_position == new_position,
        _ => old_name == new_name,
    }
}

fn render_type_info(ty: &TypeInfo, context: &ApiDiffContext<'_>, type_params: &[String]) -> String {
    match ty {
        TypeInfo::Int => "Int".to_string(),
        TypeInfo::Bool => "Bool".to_string(),
        TypeInfo::String => "String".to_string(),
        TypeInfo::Unit => "Unit".to_string(),
        TypeInfo::GenericParam(symbol) => context.symbol_name(*symbol).to_string(),
        TypeInfo::Record(symbol, args) | TypeInfo::Enum { symbol, args } => {
            render_type_application(context.symbol_name(*symbol), args, context, type_params)
        }
        TypeInfo::PackageRecord { item, args, .. } => {
            let name = context
                .item_identity(PackageItemKind::Record, *item)
                .map(ApiItemIdentity::path)
                .unwrap_or_else(|| format!("unknown-record#{}", item.as_u32()));
            render_type_application(&name, args, context, type_params)
        }
        TypeInfo::PackageEnum { item, args, .. } => {
            let name = context
                .item_identity(PackageItemKind::Enum, *item)
                .map(ApiItemIdentity::path)
                .unwrap_or_else(|| format!("unknown-enum#{}", item.as_u32()));
            render_type_application(&name, args, context, type_params)
        }
        TypeInfo::PackageOpaque { item, .. } => context
            .item_identity(PackageItemKind::OpaqueType, *item)
            .map(ApiItemIdentity::path)
            .unwrap_or_else(|| format!("unknown-opaque#{}", item.as_u32())),
        TypeInfo::List(item) => format!("List[{}]", render_type_info(item, context, type_params)),
        TypeInfo::Map(key, value) => format!(
            "Map[{}, {}]",
            render_type_info(key, context, type_params),
            render_type_info(value, context, type_params)
        ),
        TypeInfo::Option(item) => {
            format!("Option[{}]", render_type_info(item, context, type_params))
        }
        TypeInfo::Result(value, error) => format!(
            "Result[{}, {}]",
            render_type_info(value, context, type_params),
            render_type_info(error, context, type_params)
        ),
        TypeInfo::EnumConstructor {
            enum_symbol,
            variant,
            ..
        } => format!(
            "{}::{}",
            context.symbol_name(*enum_symbol),
            context.symbol_name(*variant)
        ),
        TypeInfo::Function(function) => format!(
            "fn({}) -> {}",
            function
                .params
                .iter()
                .map(|param| render_type_info(param, context, type_params))
                .collect::<Vec<_>>()
                .join(", "),
            render_type_info(&function.ret, context, type_params)
        ),
        TypeInfo::Builtin(builtin) => format!("{builtin:?}"),
        TypeInfo::Unknown => "Unknown".to_string(),
        TypeInfo::Error => "Error".to_string(),
    }
}

fn render_type_application(
    name: &str,
    args: &[TypeInfo],
    context: &ApiDiffContext<'_>,
    type_params: &[String],
) -> String {
    if args.is_empty() {
        name.to_string()
    } else {
        format!(
            "{}[{}]",
            name,
            args.iter()
                .map(|arg| render_type_info(arg, context, type_params))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

struct TypeParamContext<'a> {
    old_type_params: &'a [String],
    new_type_params: &'a [String],
}

impl<'a> TypeParamContext<'a> {
    fn new(old_type_params: &'a [String], new_type_params: &'a [String]) -> Self {
        Self {
            old_type_params,
            new_type_params,
        }
    }
}

struct ApiDiffContext<'a> {
    symbols: &'a SymbolTable,
    items: HashMap<(PackageItemKind, crate::identity::PackageItemId), ApiItemIdentity>,
}

impl<'a> ApiDiffContext<'a> {
    fn new(graph: &'a PackageInterfaceGraph, symbols: &'a SymbolTable) -> Self {
        let mut items = HashMap::new();
        for package in &graph.packages {
            for record in &package.records {
                items.insert(
                    (PackageItemKind::Record, record.item),
                    ApiItemIdentity::new(&package.path, PackageItemKind::Record, &record.name),
                );
            }
            for item in &package.enums {
                items.insert(
                    (PackageItemKind::Enum, item.item),
                    ApiItemIdentity::new(&package.path, PackageItemKind::Enum, &item.name),
                );
            }
            for item in &package.opaque_types {
                items.insert(
                    (PackageItemKind::OpaqueType, item.item),
                    ApiItemIdentity::new(&package.path, PackageItemKind::OpaqueType, &item.name),
                );
            }
            for item in &package.functions {
                items.insert(
                    (PackageItemKind::Function, item.item),
                    ApiItemIdentity::new(&package.path, PackageItemKind::Function, &item.name),
                );
            }
        }
        Self { symbols, items }
    }

    fn symbol_name(&self, symbol: Symbol) -> &str {
        self.symbols.resolve(symbol)
    }

    fn item_identity(
        &self,
        kind: PackageItemKind,
        item: crate::identity::PackageItemId,
    ) -> Option<&ApiItemIdentity> {
        self.items.get(&(kind, item))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ApiItemIdentity {
    package_path: String,
    kind: PackageItemKind,
    name: String,
}

impl ApiItemIdentity {
    fn new(package_path: &str, kind: PackageItemKind, name: &str) -> Self {
        Self {
            package_path: package_path.to_string(),
            kind,
            name: name.to_string(),
        }
    }

    fn path(&self) -> String {
        item_path(&self.package_path, &self.name)
    }
}

fn item_path(package_path: &str, name: &str) -> String {
    if package_path.is_empty() {
        name.to_string()
    } else {
        format!("{package_path}::{name}")
    }
}

fn member_path(item_path: &str, name: &str) -> String {
    format!("{item_path}.{name}")
}
