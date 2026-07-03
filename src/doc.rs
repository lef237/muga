use std::fmt::Write;

use crate::{
    interface::{
        OpaqueHandleFacts, PackageInterfaceEnum, PackageInterfaceFunction, PackageInterfaceGraph,
        PackageInterfaceOpaqueType, PackageInterfaceParamMode, PackageInterfaceRecord,
    },
    prelude,
    symbol::SymbolTable,
    types::{FunctionTypeInfo, TypeInfo},
};

pub fn render_package_docs(interfaces: &PackageInterfaceGraph, symbols: &SymbolTable) -> String {
    let mut output = String::new();
    output.push_str("# Muga Package Documentation\n\n");
    output.push_str("Generated from public package interfaces.\n");

    let mut packages = interfaces
        .packages
        .iter()
        .filter(|package| {
            !package.records.is_empty()
                || !package.enums.is_empty()
                || !package.opaque_types.is_empty()
                || !package.functions.is_empty()
        })
        .collect::<Vec<_>>();
    packages.sort_by(|left, right| left.path.cmp(&right.path));

    if packages.is_empty() {
        output.push_str("\n_No public package interface items._\n");
        return output;
    }

    for package in packages {
        output.push('\n');
        writeln!(&mut output, "## Package `{}`", package.path)
            .expect("writing docs to String should not fail");

        let mut records = package.records.iter().collect::<Vec<_>>();
        records.sort_by(|left, right| left.name.cmp(&right.name));
        if !records.is_empty() {
            output.push_str("\n### Records\n");
            for record in records {
                push_record_docs(&mut output, record, symbols);
            }
        }

        let mut enums = package.enums.iter().collect::<Vec<_>>();
        enums.sort_by(|left, right| left.name.cmp(&right.name));
        if !enums.is_empty() {
            output.push_str("\n### Enums\n");
            for enumeration in enums {
                push_enum_docs(&mut output, enumeration, symbols);
            }
        }

        let mut opaque_types = package.opaque_types.iter().collect::<Vec<_>>();
        opaque_types.sort_by(|left, right| left.name.cmp(&right.name));
        if !opaque_types.is_empty() {
            output.push_str("\n### Opaque Types\n");
            for opaque in opaque_types {
                push_opaque_type_docs(&mut output, opaque);
            }
        }

        let mut functions = package.functions.iter().collect::<Vec<_>>();
        functions.sort_by(|left, right| left.name.cmp(&right.name));
        if !functions.is_empty() {
            output.push_str("\n### Functions\n");
            for function in functions {
                push_function_docs(&mut output, function, symbols);
            }
        }
    }

    output
}

fn push_record_docs(output: &mut String, record: &PackageInterfaceRecord, symbols: &SymbolTable) {
    push_doc_comments(output, &record.doc_comments);
    output.push_str("\n```muga\n");
    writeln!(
        output,
        "pub record {}{} {{",
        record.name,
        render_type_params(&record.type_params)
    )
    .expect("writing docs to String should not fail");
    for field in &record.fields {
        writeln!(
            output,
            "  {}: {}",
            field.name,
            render_type_info(&field.ty, symbols)
        )
        .expect("writing docs to String should not fail");
    }
    output.push_str("}\n```\n");
}

fn push_enum_docs(output: &mut String, enumeration: &PackageInterfaceEnum, symbols: &SymbolTable) {
    push_doc_comments(output, &enumeration.doc_comments);
    output.push_str("\n```muga\n");
    writeln!(
        output,
        "pub enum {}{} {{",
        enumeration.name,
        render_type_params(&enumeration.type_params)
    )
    .expect("writing docs to String should not fail");
    for variant in &enumeration.variants {
        if let Some(payload) = &variant.payload {
            writeln!(
                output,
                "  {}({})",
                variant.name,
                render_type_info(payload, symbols)
            )
            .expect("writing docs to String should not fail");
        } else {
            writeln!(output, "  {}", variant.name).expect("writing docs to String should not fail");
        }
    }
    output.push_str("}\n```\n");
}

fn push_opaque_type_docs(output: &mut String, opaque: &PackageInterfaceOpaqueType) {
    push_doc_comments(output, &opaque.doc_comments);
    output.push_str("\n```muga\n");
    writeln!(output, "pub opaque type {}", opaque.name)
        .expect("writing docs to String should not fail");
    output.push_str("```\n");
    writeln!(
        output,
        "\nMetadata:\n- handleFacts: {}",
        render_opaque_handle_facts(&opaque.handle_facts)
    )
    .expect("writing docs to String should not fail");
}

fn push_function_docs(
    output: &mut String,
    function: &PackageInterfaceFunction,
    symbols: &SymbolTable,
) {
    push_doc_comments(output, &function.doc_comments);
    output.push_str("\n```muga\n");
    write!(
        output,
        "pub fn {}{}(",
        function.name,
        render_type_params(&function.type_params)
    )
    .expect("writing docs to String should not fail");
    for (index, param) in function.params.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        write!(
            output,
            "{}: {}",
            param.name,
            render_type_info(&param.ty, symbols)
        )
        .expect("writing docs to String should not fail");
    }
    writeln!(output, "): {}", render_type_info(&function.ret, symbols))
        .expect("writing docs to String should not fail");
    output.push_str("```\n");
    if !function.params.is_empty() {
        output.push_str("\nMetadata:\n- paramMode: ");
        for (index, param) in function.params.iter().enumerate() {
            if index > 0 {
                output.push_str(", ");
            }
            write!(output, "{}={}", param.name, render_param_mode(param.mode))
                .expect("writing docs to String should not fail");
        }
        output.push('\n');
    }
}

fn render_opaque_handle_facts(facts: &OpaqueHandleFacts) -> String {
    let close_function = facts
        .close_function
        .map(|item| item.as_u32().to_string())
        .unwrap_or_else(|| "null".to_string());
    format!(
        "runtimeBacked={}, copyable={}, cloneable={}, sendable={}, shareable={}, structurallyComparable={}, serializable={}, closeable={}, closeFunction={}",
        facts.runtime_backed,
        facts.copyable,
        facts.cloneable,
        facts.sendable,
        facts.shareable,
        facts.structurally_comparable,
        facts.serializable,
        facts.closeable,
        close_function
    )
}

fn render_param_mode(mode: PackageInterfaceParamMode) -> &'static str {
    mode.as_str()
}

fn push_doc_comments(output: &mut String, comments: &[String]) {
    if comments.is_empty() {
        return;
    }
    output.push('\n');
    for comment in comments {
        output.push_str(comment);
        output.push('\n');
    }
}

fn render_type_params(type_params: &[String]) -> String {
    if type_params.is_empty() {
        String::new()
    } else {
        format!("[{}]", type_params.join(", "))
    }
}

pub fn render_type_info(ty: &TypeInfo, symbols: &SymbolTable) -> String {
    match ty {
        TypeInfo::Int => "Int".to_string(),
        TypeInfo::Bool => "Bool".to_string(),
        TypeInfo::String => "String".to_string(),
        TypeInfo::Unit => "Unit".to_string(),
        TypeInfo::GenericParam(symbol) => symbols.resolve(*symbol).to_string(),
        TypeInfo::Record(symbol, args)
        | TypeInfo::PackageRecord { symbol, args, .. }
        | TypeInfo::Enum { symbol, args }
        | TypeInfo::PackageEnum { symbol, args, .. } => {
            render_named_type(symbols.resolve(*symbol), args, symbols)
        }
        TypeInfo::PackageOpaque { symbol, .. } => symbols.resolve(*symbol).to_string(),
        TypeInfo::List(item) => format!("List[{}]", render_type_info(item, symbols)),
        TypeInfo::Map(key, value) => format!(
            "Map[{}, {}]",
            render_type_info(key, symbols),
            render_type_info(value, symbols)
        ),
        TypeInfo::Option(item) => format!("Option[{}]", render_type_info(item, symbols)),
        TypeInfo::Result(ok, err) => format!(
            "Result[{}, {}]",
            render_type_info(ok, symbols),
            render_type_info(err, symbols)
        ),
        TypeInfo::Task(item) => format!("Task[{}]", render_type_info(item, symbols)),
        TypeInfo::EnumConstructor {
            enum_symbol,
            variant,
            ..
        } => format!(
            "{}::{}",
            symbols.resolve(*enum_symbol),
            symbols.resolve(*variant)
        ),
        TypeInfo::Function(function) => render_function_type(function, symbols),
        TypeInfo::Builtin(builtin) => prelude::builtin_name(*builtin).to_string(),
        TypeInfo::Unknown => "Unknown".to_string(),
        TypeInfo::Error => "Error".to_string(),
    }
}

fn render_named_type(name: &str, args: &[TypeInfo], symbols: &SymbolTable) -> String {
    if args.is_empty() {
        name.to_string()
    } else {
        let args = args
            .iter()
            .map(|arg| render_type_info(arg, symbols))
            .collect::<Vec<_>>();
        format!("{name}[{}]", args.join(", "))
    }
}

fn render_function_type(function: &FunctionTypeInfo, symbols: &SymbolTable) -> String {
    let params = match function.params.as_slice() {
        [] => "()".to_string(),
        [param] => render_function_param_type(param, symbols),
        params => {
            let params = params
                .iter()
                .map(|param| render_type_info(param, symbols))
                .collect::<Vec<_>>();
            format!("({})", params.join(", "))
        }
    };
    format!("{} -> {}", params, render_type_info(&function.ret, symbols))
}

fn render_function_param_type(ty: &TypeInfo, symbols: &SymbolTable) -> String {
    match ty {
        TypeInfo::Function(_) => format!("({})", render_type_info(ty, symbols)),
        _ => render_type_info(ty, symbols),
    }
}
