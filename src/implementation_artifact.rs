use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{
    ast::Visibility,
    bytecode::{
        self, BinaryOp, BindingDef, Chunk, Function, Instruction, LocalDef, LocalKind, NameRef,
    },
    diagnostic::Diagnostic,
    identity::{BindingId, BindingKind, LocalId, PackageItemId},
    interface::{PackageInterfaceGraph, stable_hash_hex},
    package::{PackageItemKind, PackageSymbolGraph},
    span::{Position, Span},
    symbol::{Symbol, SymbolTable},
};

const PERSISTED_IMPLEMENTATION_HEADER: &str = "muga-package-implementation-bytecode-v1";

#[derive(Clone, Debug)]
pub struct PackageImplementationArtifact {
    pub package_path: String,
    pub interface_hash: String,
    pub dependency_interfaces: Vec<PackageImplementationDependencyHash>,
    pub item_refs: Vec<PackageImplementationItemRef>,
    pub program: bytecode::Program,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageImplementationDependencyHash {
    pub package_path: String,
    pub interface_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageImplementationItemRef {
    pub local_item: PackageItemId,
    pub package_path: String,
    pub module_path: String,
    pub kind: PackageItemKind,
    pub visibility: Visibility,
    pub name: String,
}

impl PackageImplementationArtifact {
    pub fn from_bytecode_package(
        package_path: &str,
        interfaces: &PackageInterfaceGraph,
        interface_symbols: &SymbolTable,
        package_graph: &PackageSymbolGraph,
        program: bytecode::Program,
    ) -> Result<Self, Diagnostic> {
        let Some(interface) = interfaces.package_by_path(package_path) else {
            return Err(implementation_artifact_diagnostic(format!(
                "compiled package interfaces do not contain `{package_path}`"
            )));
        };
        let Some(interface_hash) =
            interfaces.stable_hash_for_package(package_path, interface_symbols)
        else {
            return Err(implementation_artifact_diagnostic(format!(
                "missing package interface hash for `{package_path}`"
            )));
        };

        let mut dependency_interfaces = Vec::with_capacity(interface.dependencies.len());
        for dependency in &interface.dependencies {
            let Some(interface_hash) =
                interfaces.stable_hash_for_package(dependency, interface_symbols)
            else {
                return Err(implementation_artifact_diagnostic(format!(
                    "missing dependency interface hash for `{dependency}`"
                )));
            };
            dependency_interfaces.push(PackageImplementationDependencyHash {
                package_path: dependency.clone(),
                interface_hash,
            });
        }
        dependency_interfaces.sort_by(|left, right| left.package_path.cmp(&right.package_path));

        let item_refs = implementation_item_refs(package_graph, &program)?;

        Ok(Self {
            package_path: package_path.to_string(),
            interface_hash,
            dependency_interfaces,
            item_refs,
            program,
        })
    }

    pub fn to_persisted_text(&self) -> String {
        let body = self.body_text();
        format!(
            "{PERSISTED_IMPLEMENTATION_HEADER}\nhash\t{}\n{body}",
            stable_hash_hex(&body)
        )
    }

    pub fn from_persisted_text(text: &str) -> Result<Self, Vec<Diagnostic>> {
        let lines = text.lines().collect::<Vec<_>>();
        if lines.first().copied() != Some(PERSISTED_IMPLEMENTATION_HEADER) {
            return Err(vec![implementation_artifact_diagnostic(
                "invalid package implementation artifact header",
            )]);
        }
        let Some(expected_hash) = lines.get(1).and_then(|line| line.strip_prefix("hash\t")) else {
            return Err(vec![implementation_artifact_diagnostic(
                "package implementation artifact is missing a hash",
            )]);
        };
        let body = if lines.len() > 2 {
            format!("{}\n", lines[2..].join("\n"))
        } else {
            String::new()
        };
        let actual_hash = stable_hash_hex(&body);
        if expected_hash != actual_hash {
            return Err(vec![
                Diagnostic::new(
                    "PK022",
                    format!(
                        "package implementation artifact hash mismatch: expected `{expected_hash}` but found `{actual_hash}`"
                    ),
                    Span::default(),
                )
                .with_suggestion("regenerate the package implementation artifact"),
            ]);
        }

        let artifact = Self::from_body_lines(&lines[2..])?;
        let diagnostics = validate_artifact_structure(&artifact);
        if diagnostics.is_empty() {
            Ok(artifact)
        } else {
            Err(diagnostics)
        }
    }

    pub fn write_persisted_artifact(&self, root: &Path) -> Result<PathBuf, Diagnostic> {
        let path = persisted_file_path(root, &self.package_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                implementation_artifact_diagnostic(format!(
                    "failed to create package implementation artifact directory {}: {error}",
                    parent.display()
                ))
            })?;
        }
        fs::write(&path, self.to_persisted_text()).map_err(|error| {
            implementation_artifact_diagnostic(format!(
                "failed to write package implementation artifact `{}`: {error}",
                path.display()
            ))
        })?;
        Ok(path)
    }

    pub fn validate_against_interfaces(
        &self,
        interfaces: &PackageInterfaceGraph,
        symbols: &SymbolTable,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        match interfaces.stable_hash_for_package(&self.package_path, symbols) {
            Some(expected) if expected == self.interface_hash => {}
            Some(expected) => diagnostics.push(stale_implementation_artifact_diagnostic(format!(
                "stale package implementation artifact for `{}`: expected interface hash `{expected}` but found `{}`",
                self.package_path, self.interface_hash
            ))),
            None => diagnostics.push(implementation_artifact_diagnostic(format!(
                "missing loaded package interface for `{}`",
                self.package_path
            ))),
        }

        let expected_dependencies = interfaces
            .package_by_path(&self.package_path)
            .map(|interface| interface.dependencies.clone())
            .unwrap_or_default();
        let actual_dependencies = self
            .dependency_interfaces
            .iter()
            .map(|dependency| dependency.package_path.clone())
            .collect::<Vec<_>>();
        if expected_dependencies != actual_dependencies {
            diagnostics.push(stale_implementation_artifact_diagnostic(format!(
                "stale package implementation artifact for `{}`: dependency interface set changed",
                self.package_path
            )));
        }

        for dependency in &self.dependency_interfaces {
            match interfaces.stable_hash_for_package(&dependency.package_path, symbols) {
                Some(expected) if expected == dependency.interface_hash => {}
                Some(expected) => {
                    diagnostics.push(stale_implementation_artifact_diagnostic(format!(
                        "stale package implementation artifact for `{}`: dependency `{}` expected interface hash `{expected}` but found `{}`",
                        self.package_path, dependency.package_path, dependency.interface_hash
                    )));
                }
                None => diagnostics.push(implementation_artifact_diagnostic(format!(
                    "missing loaded dependency interface for `{}`",
                    dependency.package_path
                ))),
            }
        }

        diagnostics
    }

    fn remap_package_items(
        mut self,
        interfaces: &PackageInterfaceGraph,
        next_private_item: &mut u32,
    ) -> Result<Self, Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let mut item_map = HashMap::new();
        for item_ref in &self.item_refs {
            let remapped = if item_ref.visibility == Visibility::Public {
                interface_item_id(interfaces, item_ref).or_else(|| {
                    diagnostics.push(stale_implementation_artifact_diagnostic(format!(
                        "package implementation artifact for `{}` references public {} `{}` that is not in the loaded interface",
                        self.package_path,
                        package_item_kind_label(item_ref.kind),
                        item_ref.name
                    )));
                    None
                })
            } else {
                let id = PackageItemId::new(*next_private_item);
                *next_private_item += 1;
                Some(id)
            };
            if let Some(remapped) = remapped {
                item_map.insert(item_ref.local_item, remapped);
            }
        }
        remap_program_package_items(&mut self.program, &item_map, &mut diagnostics);
        if diagnostics.is_empty() {
            Ok(self)
        } else {
            Err(diagnostics)
        }
    }

    fn from_body_lines(lines: &[&str]) -> Result<Self, Vec<Diagnostic>> {
        let mut parser = ArtifactParser::new(lines);
        parser.parse()
    }

    fn body_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("package\t{}\n", escape_field(&self.package_path)));
        out.push_str(&format!("interface\t{}\n", self.interface_hash));
        out.push_str(&format!("deps\t{}\n", self.dependency_interfaces.len()));
        for dependency in &self.dependency_interfaces {
            out.push_str(&format!(
                "dep\t{}\t{}\n",
                escape_field(&dependency.package_path),
                dependency.interface_hash
            ));
        }
        out.push_str(&format!("items\t{}\n", self.item_refs.len()));
        for item in &self.item_refs {
            out.push_str(&format!(
                "item\t{}\t{}\t{}\t{}\t{}\t{}\n",
                item.local_item.as_u32(),
                escape_field(&item.package_path),
                escape_field(&item.module_path),
                package_item_kind_text(item.kind),
                visibility_text(item.visibility),
                escape_field(&item.name)
            ));
        }
        push_program(&mut out, &self.program);
        out
    }
}

pub fn read_persisted_file(path: &Path) -> Result<PackageImplementationArtifact, Vec<Diagnostic>> {
    let text = fs::read_to_string(path).map_err(|error| {
        vec![implementation_artifact_diagnostic(format!(
            "failed to read package implementation artifact `{}`: {error}",
            path.display()
        ))]
    })?;
    PackageImplementationArtifact::from_persisted_text(&text)
}

pub fn read_persisted_artifacts(
    root: &Path,
    interfaces: &PackageInterfaceGraph,
    symbols: &SymbolTable,
) -> Result<Vec<PackageImplementationArtifact>, Vec<Diagnostic>> {
    read_persisted_artifacts_reserving_program_items(root, interfaces, symbols, &[])
}

pub fn read_persisted_artifacts_reserving_program_items(
    root: &Path,
    interfaces: &PackageInterfaceGraph,
    symbols: &SymbolTable,
    reserved_programs: &[&bytecode::Program],
) -> Result<Vec<PackageImplementationArtifact>, Vec<Diagnostic>> {
    let mut artifacts = Vec::new();
    let mut diagnostics = Vec::new();
    let mut next_private_item = next_private_package_item_id(interfaces);
    for program in reserved_programs {
        next_private_item = next_private_item.max(next_package_item_id_in_program(program));
    }

    for interface in &interfaces.packages {
        let artifact_path = persisted_file_path(root, &interface.path);
        if !artifact_path.is_file() {
            diagnostics.push(
                Diagnostic::new(
                    "PK022",
                    format!(
                        "missing package implementation artifact `{}` for `{}`",
                        artifact_path.display(),
                        interface.path
                    ),
                    Span::default(),
                )
                .with_suggestion("regenerate package artifacts with `emit-artifacts`"),
            );
            continue;
        }

        match read_persisted_file(&artifact_path) {
            Ok(artifact) => {
                if artifact.package_path != interface.path {
                    diagnostics.push(implementation_artifact_diagnostic(format!(
                        "package implementation artifact `{}` contains `{}` instead of `{}`",
                        artifact_path.display(),
                        artifact.package_path,
                        interface.path
                    )));
                    continue;
                }
                diagnostics.extend(artifact.validate_against_interfaces(interfaces, symbols));
                match artifact.remap_package_items(interfaces, &mut next_private_item) {
                    Ok(artifact) => artifacts.push(artifact),
                    Err(mut errors) => diagnostics.append(&mut errors),
                }
            }
            Err(errors) => {
                for mut error in errors {
                    error.message = format!(
                        "{} in `{}` for `{}`",
                        error.message,
                        artifact_path.display(),
                        interface.path
                    );
                    diagnostics.push(error);
                }
            }
        }
    }

    if diagnostics.is_empty() {
        Ok(artifacts)
    } else {
        Err(diagnostics)
    }
}

pub fn programs_from_artifacts(
    artifacts: Vec<PackageImplementationArtifact>,
) -> Vec<bytecode::Program> {
    let mut artifacts = artifacts
        .into_iter()
        .map(|artifact| (artifact.package_path.clone(), artifact))
        .collect::<HashMap<_, _>>();
    let mut paths = artifacts.keys().cloned().collect::<Vec<_>>();
    paths.sort();

    let mut ordered_paths = Vec::new();
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    for path in paths {
        order_artifact_dependencies(
            &path,
            &artifacts,
            &mut visiting,
            &mut visited,
            &mut ordered_paths,
        );
    }

    ordered_paths
        .into_iter()
        .filter_map(|path| artifacts.remove(&path).map(|artifact| artifact.program))
        .collect()
}

pub fn persisted_file_path(root: &Path, package_path: &str) -> PathBuf {
    root.join(format!("{}.mgb", package_path.replace("::", "__")))
}

fn implementation_item_refs(
    graph: &PackageSymbolGraph,
    program: &bytecode::Program,
) -> Result<Vec<PackageImplementationItemRef>, Diagnostic> {
    let item_ids = program
        .bindings
        .iter()
        .filter_map(|binding| binding.package_item)
        .chain(program.locals.iter().filter_map(|local| local.package_item))
        .collect::<HashSet<_>>();
    let mut item_refs = Vec::new();
    for item_id in item_ids {
        let Some(item) = graph.item(item_id) else {
            return Err(implementation_artifact_diagnostic(format!(
                "missing package item metadata for {:?}",
                item_id
            )));
        };
        let package_path = graph
            .package(item.package)
            .map(|package| package.path.clone())
            .unwrap_or_else(|| "<unknown>".to_string());
        let module_path = graph
            .module(item.module)
            .map(|module| module.path.clone())
            .unwrap_or_else(|| "<unknown>".to_string());
        item_refs.push(PackageImplementationItemRef {
            local_item: item_id,
            package_path,
            module_path,
            kind: item.kind,
            visibility: item.visibility,
            name: item.name.clone(),
        });
    }
    item_refs.sort_by_key(|item| item.local_item.as_u32());
    Ok(item_refs)
}

fn order_artifact_dependencies(
    path: &str,
    artifacts: &HashMap<String, PackageImplementationArtifact>,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
    ordered_paths: &mut Vec<String>,
) {
    if visited.contains(path) || !visiting.insert(path.to_string()) {
        return;
    }
    if let Some(artifact) = artifacts.get(path) {
        for dependency in &artifact.dependency_interfaces {
            order_artifact_dependencies(
                &dependency.package_path,
                artifacts,
                visiting,
                visited,
                ordered_paths,
            );
        }
    }
    visiting.remove(path);
    if visited.insert(path.to_string()) {
        ordered_paths.push(path.to_string());
    }
}

fn push_program(out: &mut String, program: &bytecode::Program) {
    out.push_str(&format!("symbols\t{}\n", program.symbols.len()));
    for name in program.symbols.names() {
        out.push_str(&format!("sym\t{}\n", escape_field(name)));
    }
    out.push_str(&format!("local_count\t{}\n", program.local_count));
    push_name_ref_option(out, "main", program.main);

    out.push_str(&format!("bindings\t{}\n", program.bindings.len()));
    for binding in &program.bindings {
        out.push_str(&format!(
            "binding\t{}\t{}\t{}\t{}\t{}\t{}\n",
            binding.id.as_u32(),
            binding.local.as_u32(),
            binding.name.as_u32(),
            binding_kind_text(binding.kind),
            package_item_option_text(binding.package_item),
            span_text(binding.span)
        ));
    }

    out.push_str(&format!("locals\t{}\n", program.locals.len()));
    for local in &program.locals {
        out.push_str(&format!(
            "local\t{}\t{}\t{}\t{}\t{}\t{}\n",
            local.id.as_u32(),
            binding_option_text(local.binding),
            local.name.as_u32(),
            local_kind_text(local.kind),
            package_item_option_text(local.package_item),
            span_text(local.span)
        ));
    }

    push_chunk(out, "entry", &program.entry);
    out.push_str(&format!("functions\t{}\n", program.functions.len()));
    for function in &program.functions {
        out.push_str(&format!(
            "function\t{}\t{}\t{}\t{}\n",
            function.id,
            symbol_option_text(function.name),
            function.params.len(),
            span_text(function.span)
        ));
        for param in &function.params {
            push_name_ref(out, "param", *param);
        }
        push_chunk(out, "chunk", &function.chunk);
    }
}

fn push_chunk(out: &mut String, label: &str, chunk: &Chunk) {
    out.push_str(&format!("{label}\t{}\n", chunk.instructions.len()));
    for instruction in &chunk.instructions {
        push_instruction(out, instruction);
    }
}

fn push_instruction(out: &mut String, instruction: &Instruction) {
    match instruction {
        Instruction::LoadInt(value) => out.push_str(&format!("ins\tLoadInt\t{value}\n")),
        Instruction::LoadBool(value) => out.push_str(&format!("ins\tLoadBool\t{value}\n")),
        Instruction::LoadString(value) => {
            out.push_str(&format!("ins\tLoadString\t{}\n", escape_field(value)));
        }
        Instruction::MakeRecord {
            type_name,
            fields,
            span,
        } => out.push_str(&format!(
            "ins\tMakeRecord\t{}\t{}\t{}\t{}\n",
            type_name.as_u32(),
            symbol_list_text(fields),
            fields.len(),
            span_text(*span)
        )),
        Instruction::MakeEnum {
            enum_name,
            variant_name,
            has_payload,
            span,
        } => out.push_str(&format!(
            "ins\tMakeEnum\t{}\t{}\t{}\t{}\n",
            enum_name.as_u32(),
            variant_name.as_u32(),
            has_payload,
            span_text(*span)
        )),
        Instruction::MakeList { len, span } => {
            out.push_str(&format!("ins\tMakeList\t{len}\t{}\n", span_text(*span)));
        }
        Instruction::LoadName { target, span } => {
            out.push_str(&format!("ins\tLoadName\t{}\n", span_text(*span)));
            push_name_ref(out, "target", *target);
        }
        Instruction::LoadField { field, span } => out.push_str(&format!(
            "ins\tLoadField\t{}\t{}\n",
            field.as_u32(),
            span_text(*span)
        )),
        Instruction::LoadIndex { span } => {
            out.push_str(&format!("ins\tLoadIndex\t{}\n", span_text(*span)));
        }
        Instruction::UpdateRecord { fields, span } => out.push_str(&format!(
            "ins\tUpdateRecord\t{}\t{}\t{}\n",
            symbol_list_text(fields),
            fields.len(),
            span_text(*span)
        )),
        Instruction::Assign {
            target,
            mutable,
            is_update,
            span,
        } => {
            out.push_str(&format!(
                "ins\tAssign\t{}\t{}\t{}\n",
                mutable,
                is_update,
                span_text(*span)
            ));
            push_name_ref(out, "target", *target);
        }
        Instruction::DefineFunction {
            target,
            function,
            span,
        } => {
            out.push_str(&format!(
                "ins\tDefineFunction\t{}\t{}\n",
                function,
                span_text(*span)
            ));
            push_name_ref(out, "target", *target);
        }
        Instruction::MakeClosure { function } => {
            out.push_str(&format!("ins\tMakeClosure\t{function}\n"));
        }
        Instruction::UnaryNeg { span } => {
            out.push_str(&format!("ins\tUnaryNeg\t{}\n", span_text(*span)));
        }
        Instruction::UnaryNot { span } => {
            out.push_str(&format!("ins\tUnaryNot\t{}\n", span_text(*span)));
        }
        Instruction::Binary { op, span } => out.push_str(&format!(
            "ins\tBinary\t{}\t{}\n",
            binary_op_text(*op),
            span_text(*span)
        )),
        Instruction::Call { argc, span } => {
            out.push_str(&format!("ins\tCall\t{argc}\t{}\n", span_text(*span)));
        }
        Instruction::JumpIfFalse { target, span } => out.push_str(&format!(
            "ins\tJumpIfFalse\t{}\t{}\n",
            target,
            span_text(*span)
        )),
        Instruction::JumpIfNotEnumVariant {
            enum_name,
            variant_name,
            target,
            span,
        } => out.push_str(&format!(
            "ins\tJumpIfNotEnumVariant\t{}\t{}\t{}\t{}\n",
            enum_name.as_u32(),
            variant_name.as_u32(),
            target,
            span_text(*span)
        )),
        Instruction::MatchExhausted { enum_name, span } => out.push_str(&format!(
            "ins\tMatchExhausted\t{}\t{}\n",
            enum_name.as_u32(),
            span_text(*span)
        )),
        Instruction::Jump { target } => out.push_str(&format!("ins\tJump\t{target}\n")),
        Instruction::PushScope => out.push_str("ins\tPushScope\n"),
        Instruction::PopScope => out.push_str("ins\tPopScope\n"),
        Instruction::Pop => out.push_str("ins\tPop\n"),
        Instruction::Return => out.push_str("ins\tReturn\n"),
    }
}

struct ArtifactParser<'a> {
    lines: &'a [&'a str],
    index: usize,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> ArtifactParser<'a> {
    fn new(lines: &'a [&'a str]) -> Self {
        Self {
            lines,
            index: 0,
            diagnostics: Vec::new(),
        }
    }

    fn parse(&mut self) -> Result<PackageImplementationArtifact, Vec<Diagnostic>> {
        let parsed = match self.parse_inner() {
            Ok(parsed) => parsed,
            Err(diagnostic) => {
                self.diagnostics.push(diagnostic);
                return Err(std::mem::take(&mut self.diagnostics));
            }
        };

        if self.diagnostics.is_empty() {
            Ok(parsed)
        } else {
            Err(std::mem::take(&mut self.diagnostics))
        }
    }

    fn parse_inner(&mut self) -> Result<PackageImplementationArtifact, Diagnostic> {
        let package_path = self.field("package")?;
        let interface_hash = self.field("interface")?;
        let dep_count = self.count("deps")?;
        let mut dependency_interfaces = Vec::with_capacity(dep_count);
        for _ in 0..dep_count {
            let parts = self.parts("dep", 3)?;
            dependency_interfaces.push(PackageImplementationDependencyHash {
                package_path: unescape_field(parts[1])?,
                interface_hash: parts[2].to_string(),
            });
        }

        let item_count = self.count("items")?;
        let mut item_refs = Vec::with_capacity(item_count);
        for _ in 0..item_count {
            let parts = self.parts("item", 7)?;
            item_refs.push(PackageImplementationItemRef {
                local_item: PackageItemId::new(parse_u32(parts[1], "item id")?),
                package_path: unescape_field(parts[2])?,
                module_path: unescape_field(parts[3])?,
                kind: parse_package_item_kind(parts[4])?,
                visibility: parse_visibility(parts[5])?,
                name: unescape_field(parts[6])?,
            });
        }

        let program = self.program()?;
        if self.index != self.lines.len() {
            self.diagnostics.push(implementation_artifact_diagnostic(
                "package implementation artifact contains trailing data",
            ));
        }

        Ok(PackageImplementationArtifact {
            package_path,
            interface_hash,
            dependency_interfaces,
            item_refs,
            program,
        })
    }

    fn program(&mut self) -> Result<bytecode::Program, Diagnostic> {
        let symbol_count = self.count("symbols")?;
        let mut symbols = SymbolTable::default();
        for _ in 0..symbol_count {
            let parts = self.parts("sym", 2)?;
            symbols.intern(&unescape_field(parts[1])?);
        }
        let local_count = self.count("local_count")?;
        let main = self.name_ref_option("main")?;

        let binding_count = self.count("bindings")?;
        let mut bindings = Vec::with_capacity(binding_count);
        for _ in 0..binding_count {
            let parts = self.parts("binding", 7)?;
            bindings.push(BindingDef {
                id: BindingId::new(parse_u32(parts[1], "binding id")?),
                local: LocalId::new(parse_u32(parts[2], "binding local")?),
                name: Symbol::new(parse_u32(parts[3], "binding name")?),
                kind: parse_binding_kind(parts[4])?,
                package_item: parse_package_item_option(parts[5])?,
                span: parse_span(parts[6])?,
            });
        }

        let local_def_count = self.count("locals")?;
        let mut locals = Vec::with_capacity(local_def_count);
        for _ in 0..local_def_count {
            let parts = self.parts("local", 7)?;
            locals.push(LocalDef {
                id: LocalId::new(parse_u32(parts[1], "local id")?),
                binding: parse_binding_option(parts[2])?,
                name: Symbol::new(parse_u32(parts[3], "local name")?),
                kind: parse_local_kind(parts[4])?,
                package_item: parse_package_item_option(parts[5])?,
                span: parse_span(parts[6])?,
            });
        }

        let entry = self.chunk("entry")?;
        let function_count = self.count("functions")?;
        let mut functions = Vec::with_capacity(function_count);
        for _ in 0..function_count {
            let parts = self.parts("function", 5)?;
            let id = parse_usize(parts[1], "function id")?;
            let name = parse_symbol_option(parts[2])?;
            let param_count = parse_usize(parts[3], "function parameter count")?;
            let span = parse_span(parts[4])?;
            let mut params = Vec::with_capacity(param_count);
            for _ in 0..param_count {
                params.push(self.name_ref("param")?);
            }
            let chunk = self.chunk("chunk")?;
            functions.push(Function {
                id,
                name,
                params,
                chunk,
                span,
            });
        }

        Ok(bytecode::Program {
            entry,
            functions,
            bindings,
            locals,
            main,
            local_count,
            symbols,
        })
    }

    fn chunk(&mut self, label: &str) -> Result<Chunk, Diagnostic> {
        let instruction_count = self.count(label)?;
        let mut instructions = Vec::with_capacity(instruction_count);
        for _ in 0..instruction_count {
            instructions.push(self.instruction()?);
        }
        Ok(Chunk { instructions })
    }

    fn instruction(&mut self) -> Result<Instruction, Diagnostic> {
        let line = self.next_line()?;
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 2 || parts[0] != "ins" {
            return Err(implementation_artifact_diagnostic(
                "invalid package implementation instruction line",
            ));
        }
        let instruction = match parts[1] {
            "LoadInt" if parts.len() == 3 => Instruction::LoadInt(parse_i64(parts[2], "int")?),
            "LoadBool" if parts.len() == 3 => Instruction::LoadBool(parse_bool(parts[2], "bool")?),
            "LoadString" if parts.len() == 3 => Instruction::LoadString(unescape_field(parts[2])?),
            "MakeRecord" if parts.len() == 6 => Instruction::MakeRecord {
                type_name: Symbol::new(parse_u32(parts[2], "record type symbol")?),
                fields: parse_symbol_list(parts[3], parse_usize(parts[4], "field count")?)?,
                span: parse_span(parts[5])?,
            },
            "MakeEnum" if parts.len() == 6 => Instruction::MakeEnum {
                enum_name: Symbol::new(parse_u32(parts[2], "enum symbol")?),
                variant_name: Symbol::new(parse_u32(parts[3], "variant symbol")?),
                has_payload: parse_bool(parts[4], "payload flag")?,
                span: parse_span(parts[5])?,
            },
            "MakeList" if parts.len() == 4 => Instruction::MakeList {
                len: parse_usize(parts[2], "list length")?,
                span: parse_span(parts[3])?,
            },
            "LoadName" if parts.len() == 3 => Instruction::LoadName {
                span: parse_span(parts[2])?,
                target: self.name_ref("target")?,
            },
            "LoadField" if parts.len() == 4 => Instruction::LoadField {
                field: Symbol::new(parse_u32(parts[2], "field symbol")?),
                span: parse_span(parts[3])?,
            },
            "LoadIndex" if parts.len() == 3 => Instruction::LoadIndex {
                span: parse_span(parts[2])?,
            },
            "UpdateRecord" if parts.len() == 5 => Instruction::UpdateRecord {
                fields: parse_symbol_list(parts[2], parse_usize(parts[3], "field count")?)?,
                span: parse_span(parts[4])?,
            },
            "Assign" if parts.len() == 5 => Instruction::Assign {
                mutable: parse_bool(parts[2], "mutable flag")?,
                is_update: parse_bool(parts[3], "update flag")?,
                span: parse_span(parts[4])?,
                target: self.name_ref("target")?,
            },
            "DefineFunction" if parts.len() == 4 => Instruction::DefineFunction {
                function: parse_usize(parts[2], "function id")?,
                span: parse_span(parts[3])?,
                target: self.name_ref("target")?,
            },
            "MakeClosure" if parts.len() == 3 => Instruction::MakeClosure {
                function: parse_usize(parts[2], "function id")?,
            },
            "UnaryNeg" if parts.len() == 3 => Instruction::UnaryNeg {
                span: parse_span(parts[2])?,
            },
            "UnaryNot" if parts.len() == 3 => Instruction::UnaryNot {
                span: parse_span(parts[2])?,
            },
            "Binary" if parts.len() == 4 => Instruction::Binary {
                op: parse_binary_op(parts[2])?,
                span: parse_span(parts[3])?,
            },
            "Call" if parts.len() == 4 => Instruction::Call {
                argc: parse_usize(parts[2], "argument count")?,
                span: parse_span(parts[3])?,
            },
            "JumpIfFalse" if parts.len() == 4 => Instruction::JumpIfFalse {
                target: parse_usize(parts[2], "jump target")?,
                span: parse_span(parts[3])?,
            },
            "JumpIfNotEnumVariant" if parts.len() == 6 => Instruction::JumpIfNotEnumVariant {
                enum_name: Symbol::new(parse_u32(parts[2], "enum symbol")?),
                variant_name: Symbol::new(parse_u32(parts[3], "variant symbol")?),
                target: parse_usize(parts[4], "jump target")?,
                span: parse_span(parts[5])?,
            },
            "MatchExhausted" if parts.len() == 4 => Instruction::MatchExhausted {
                enum_name: Symbol::new(parse_u32(parts[2], "enum symbol")?),
                span: parse_span(parts[3])?,
            },
            "Jump" if parts.len() == 3 => Instruction::Jump {
                target: parse_usize(parts[2], "jump target")?,
            },
            "PushScope" if parts.len() == 2 => Instruction::PushScope,
            "PopScope" if parts.len() == 2 => Instruction::PopScope,
            "Pop" if parts.len() == 2 => Instruction::Pop,
            "Return" if parts.len() == 2 => Instruction::Return,
            _ => {
                return Err(implementation_artifact_diagnostic(format!(
                    "invalid package implementation instruction `{}`",
                    parts[1]
                )));
            }
        };
        Ok(instruction)
    }

    fn field(&mut self, prefix: &str) -> Result<String, Diagnostic> {
        let parts = self.parts(prefix, 2)?;
        unescape_field(parts[1])
    }

    fn count(&mut self, prefix: &str) -> Result<usize, Diagnostic> {
        let parts = self.parts(prefix, 2)?;
        parse_usize(parts[1], prefix)
    }

    fn name_ref_option(&mut self, prefix: &str) -> Result<Option<NameRef>, Diagnostic> {
        let parts = self.parts(prefix, 2)?;
        if parts[1] == "-" {
            Ok(None)
        } else {
            Ok(Some(parse_name_ref(parts[1])?))
        }
    }

    fn name_ref(&mut self, prefix: &str) -> Result<NameRef, Diagnostic> {
        let parts = self.parts(prefix, 2)?;
        parse_name_ref(parts[1])
    }

    fn parts(&mut self, prefix: &str, expected: usize) -> Result<Vec<&'a str>, Diagnostic> {
        let line = self.next_line()?;
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() != expected || parts[0] != prefix {
            return Err(implementation_artifact_diagnostic(format!(
                "invalid package implementation `{prefix}` line"
            )));
        }
        Ok(parts)
    }

    fn next_line(&mut self) -> Result<&'a str, Diagnostic> {
        let Some(line) = self.lines.get(self.index).copied() else {
            return Err(implementation_artifact_diagnostic(
                "unexpected end of package implementation artifact",
            ));
        };
        self.index += 1;
        Ok(line)
    }
}

fn remap_program_package_items(
    program: &mut bytecode::Program,
    item_map: &HashMap<PackageItemId, PackageItemId>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for binding in &mut program.bindings {
        remap_package_item(&mut binding.package_item, item_map, diagnostics);
    }
    for local in &mut program.locals {
        remap_package_item(&mut local.package_item, item_map, diagnostics);
    }
}

fn remap_package_item(
    item: &mut Option<PackageItemId>,
    item_map: &HashMap<PackageItemId, PackageItemId>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(current) = *item else {
        return;
    };
    match item_map.get(&current).copied() {
        Some(remapped) => *item = Some(remapped),
        None => diagnostics.push(implementation_artifact_diagnostic(format!(
            "package implementation artifact references unknown package item {:?}",
            current
        ))),
    }
}

fn validate_artifact_structure(artifact: &PackageImplementationArtifact) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let program = &artifact.program;
    let symbol_count = program.symbols.len();
    let mut binding_ids = HashSet::new();
    let mut local_ids = HashSet::new();
    let mut function_ids = HashSet::new();
    let mut item_ids = HashSet::new();

    for item_ref in &artifact.item_refs {
        if !item_ids.insert(item_ref.local_item) {
            diagnostics.push(invalid_bytecode_diagnostic(format!(
                "duplicate package item reference {} for `{}`",
                item_ref.local_item.as_u32(),
                artifact.package_path
            )));
        }
    }

    for binding in &program.bindings {
        if !binding_ids.insert(binding.id) {
            diagnostics.push(invalid_bytecode_diagnostic(format!(
                "duplicate binding id {} in `{}`",
                binding.id.as_u32(),
                artifact.package_path
            )));
        }
    }
    for local in &program.locals {
        if !local_ids.insert(local.id) {
            diagnostics.push(invalid_bytecode_diagnostic(format!(
                "duplicate local id {} in `{}`",
                local.id.as_u32(),
                artifact.package_path
            )));
        }
    }
    for function in &program.functions {
        if !function_ids.insert(function.id) {
            diagnostics.push(invalid_bytecode_diagnostic(format!(
                "duplicate function id {} in `{}`",
                function.id, artifact.package_path
            )));
        }
    }

    let validation_context = BytecodeValidationContext {
        symbol_count,
        binding_ids: &binding_ids,
        local_ids: &local_ids,
        function_ids: &function_ids,
        local_count: program.local_count,
    };

    for binding in &program.bindings {
        let label = format!("binding {}", binding.id.as_u32());
        validate_local_id(
            binding.local,
            &format!("{label} local"),
            &local_ids,
            program.local_count,
            &mut diagnostics,
        );
        validate_symbol(
            binding.name,
            &format!("{label} name"),
            symbol_count,
            &mut diagnostics,
        );
        validate_package_item_ref(
            binding.package_item,
            &format!("{label} package item"),
            &item_ids,
            &mut diagnostics,
        );
    }

    for local in &program.locals {
        let label = format!("local {}", local.id.as_u32());
        validate_local_id(
            local.id,
            &label,
            &local_ids,
            program.local_count,
            &mut diagnostics,
        );
        if let Some(binding) = local.binding {
            validate_binding_id(
                binding,
                &format!("{label} binding"),
                &binding_ids,
                &mut diagnostics,
            );
        }
        validate_symbol(
            local.name,
            &format!("{label} name"),
            symbol_count,
            &mut diagnostics,
        );
        validate_package_item_ref(
            local.package_item,
            &format!("{label} package item"),
            &item_ids,
            &mut diagnostics,
        );
    }

    if let Some(main) = program.main {
        validate_name_ref(
            main,
            "main reference",
            &validation_context,
            &mut diagnostics,
        );
    }

    validate_chunk_structure(
        "entry chunk",
        &program.entry,
        &validation_context,
        &mut diagnostics,
    );

    for function in &program.functions {
        let label = format!("function {}", function.id);
        if let Some(name) = function.name {
            validate_symbol(
                name,
                &format!("{label} name"),
                symbol_count,
                &mut diagnostics,
            );
        }
        for (index, param) in function.params.iter().enumerate() {
            validate_name_ref(
                *param,
                &format!("{label} parameter {index}"),
                &validation_context,
                &mut diagnostics,
            );
        }
        validate_chunk_structure(
            &format!("{label} chunk"),
            &function.chunk,
            &validation_context,
            &mut diagnostics,
        );
    }

    diagnostics
}

struct BytecodeValidationContext<'a> {
    symbol_count: usize,
    binding_ids: &'a HashSet<BindingId>,
    local_ids: &'a HashSet<LocalId>,
    function_ids: &'a HashSet<usize>,
    local_count: usize,
}

fn validate_chunk_structure(
    label: &str,
    chunk: &Chunk,
    validation: &BytecodeValidationContext<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let instruction_count = chunk.instructions.len();
    for (index, instruction) in chunk.instructions.iter().enumerate() {
        let context = format!("{label} instruction {index}");
        match instruction {
            Instruction::LoadInt(_)
            | Instruction::LoadBool(_)
            | Instruction::LoadString(_)
            | Instruction::LoadIndex { .. }
            | Instruction::UnaryNeg { .. }
            | Instruction::UnaryNot { .. }
            | Instruction::Binary { .. }
            | Instruction::Call { .. }
            | Instruction::PushScope
            | Instruction::PopScope
            | Instruction::Pop
            | Instruction::Return
            | Instruction::MakeList { .. } => {}
            Instruction::MakeRecord {
                type_name, fields, ..
            } => {
                validate_symbol(
                    *type_name,
                    &format!("{context} record type symbol"),
                    validation.symbol_count,
                    diagnostics,
                );
                for (field_index, field) in fields.iter().enumerate() {
                    validate_symbol(
                        *field,
                        &format!("{context} record field symbol {field_index}"),
                        validation.symbol_count,
                        diagnostics,
                    );
                }
            }
            Instruction::MakeEnum {
                enum_name,
                variant_name,
                ..
            } => {
                validate_symbol(
                    *enum_name,
                    &format!("{context} enum symbol"),
                    validation.symbol_count,
                    diagnostics,
                );
                validate_symbol(
                    *variant_name,
                    &format!("{context} variant symbol"),
                    validation.symbol_count,
                    diagnostics,
                );
            }
            Instruction::JumpIfNotEnumVariant {
                enum_name,
                variant_name,
                target,
                ..
            } => {
                validate_symbol(
                    *enum_name,
                    &format!("{context} enum symbol"),
                    validation.symbol_count,
                    diagnostics,
                );
                validate_symbol(
                    *variant_name,
                    &format!("{context} variant symbol"),
                    validation.symbol_count,
                    diagnostics,
                );
                if *target > instruction_count {
                    diagnostics.push(invalid_bytecode_diagnostic(format!(
                        "{context} has jump target {target} outside chunk length {instruction_count}"
                    )));
                }
            }
            Instruction::LoadName { target, .. } => validate_name_ref(
                *target,
                &format!("{context} target"),
                validation,
                diagnostics,
            ),
            Instruction::LoadField { field, .. } => validate_symbol(
                *field,
                &format!("{context} field symbol"),
                validation.symbol_count,
                diagnostics,
            ),
            Instruction::UpdateRecord { fields, .. } => {
                for (field_index, field) in fields.iter().enumerate() {
                    validate_symbol(
                        *field,
                        &format!("{context} update field symbol {field_index}"),
                        validation.symbol_count,
                        diagnostics,
                    );
                }
            }
            Instruction::Assign { target, .. } => validate_name_ref(
                *target,
                &format!("{context} target"),
                validation,
                diagnostics,
            ),
            Instruction::DefineFunction {
                target, function, ..
            } => {
                validate_name_ref(
                    *target,
                    &format!("{context} target"),
                    validation,
                    diagnostics,
                );
                validate_function_id(
                    *function,
                    &format!("{context} function id"),
                    validation.function_ids,
                    diagnostics,
                );
            }
            Instruction::MakeClosure { function } => validate_function_id(
                *function,
                &format!("{context} function id"),
                validation.function_ids,
                diagnostics,
            ),
            Instruction::JumpIfFalse { target, .. } | Instruction::Jump { target } => {
                if *target > instruction_count {
                    diagnostics.push(invalid_bytecode_diagnostic(format!(
                        "{context} has jump target {target} outside chunk length {instruction_count}"
                    )));
                }
            }
            Instruction::MatchExhausted { enum_name, .. } => validate_symbol(
                *enum_name,
                &format!("{context} enum symbol"),
                validation.symbol_count,
                diagnostics,
            ),
        }
    }
}

fn validate_name_ref(
    value: NameRef,
    context: &str,
    validation: &BytecodeValidationContext<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(binding) = value.binding {
        validate_binding_id(
            binding,
            &format!("{context} binding"),
            validation.binding_ids,
            diagnostics,
        );
    }
    validate_local_id(
        value.local,
        &format!("{context} local"),
        validation.local_ids,
        validation.local_count,
        diagnostics,
    );
    validate_symbol(
        value.name,
        &format!("{context} name"),
        validation.symbol_count,
        diagnostics,
    );
}

fn validate_symbol(
    value: Symbol,
    context: &str,
    symbol_count: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let index = value.as_u32() as usize;
    if index >= symbol_count {
        diagnostics.push(invalid_bytecode_diagnostic(format!(
            "{context} references symbol {index} outside symbol table length {symbol_count}"
        )));
    }
}

fn validate_binding_id(
    value: BindingId,
    context: &str,
    binding_ids: &HashSet<BindingId>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !binding_ids.contains(&value) {
        diagnostics.push(invalid_bytecode_diagnostic(format!(
            "{context} references unknown binding id {}",
            value.as_u32()
        )));
    }
}

fn validate_local_id(
    value: LocalId,
    context: &str,
    local_ids: &HashSet<LocalId>,
    local_count: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let index = value.as_u32() as usize;
    if index >= local_count {
        diagnostics.push(invalid_bytecode_diagnostic(format!(
            "{context} references local {index} outside local_count {local_count}"
        )));
    }
    if !local_ids.contains(&value) {
        diagnostics.push(invalid_bytecode_diagnostic(format!(
            "{context} references unknown local id {index}"
        )));
    }
}

fn validate_function_id(
    value: usize,
    context: &str,
    function_ids: &HashSet<usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !function_ids.contains(&value) {
        diagnostics.push(invalid_bytecode_diagnostic(format!(
            "{context} references unknown function id {value}"
        )));
    }
}

fn validate_package_item_ref(
    value: Option<PackageItemId>,
    context: &str,
    item_ids: &HashSet<PackageItemId>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(value) = value else {
        return;
    };
    if !item_ids.contains(&value) {
        diagnostics.push(invalid_bytecode_diagnostic(format!(
            "{context} references unknown package item {}",
            value.as_u32()
        )));
    }
}

fn next_private_package_item_id(interfaces: &PackageInterfaceGraph) -> u32 {
    interfaces
        .packages
        .iter()
        .flat_map(|interface| {
            interface
                .records
                .iter()
                .map(|record| record.item)
                .chain(interface.enums.iter().map(|enumeration| enumeration.item))
                .chain(interface.functions.iter().map(|function| function.item))
        })
        .map(|item| item.as_u32())
        .max()
        .map_or(0, |item| item + 1)
}

fn next_package_item_id_in_program(program: &bytecode::Program) -> u32 {
    program
        .bindings
        .iter()
        .filter_map(|binding| binding.package_item)
        .chain(program.locals.iter().filter_map(|local| local.package_item))
        .map(|item| item.as_u32())
        .max()
        .map_or(0, |item| item + 1)
}

fn interface_item_id(
    interfaces: &PackageInterfaceGraph,
    item_ref: &PackageImplementationItemRef,
) -> Option<PackageItemId> {
    let interface = interfaces.package_by_path(&item_ref.package_path)?;
    match item_ref.kind {
        PackageItemKind::Record => interface
            .records
            .iter()
            .find(|record| record.name == item_ref.name)
            .map(|record| record.item),
        PackageItemKind::Enum => interface
            .enums
            .iter()
            .find(|enumeration| enumeration.name == item_ref.name)
            .map(|enumeration| enumeration.item),
        PackageItemKind::Function => interface
            .functions
            .iter()
            .find(|function| function.name == item_ref.name)
            .map(|function| function.item),
    }
}

fn push_name_ref_option(out: &mut String, label: &str, value: Option<NameRef>) {
    match value {
        Some(value) => push_name_ref(out, label, value),
        None => out.push_str(&format!("{label}\t-\n")),
    }
}

fn push_name_ref(out: &mut String, label: &str, value: NameRef) {
    out.push_str(&format!(
        "{label}\t{}:{}:{}\n",
        binding_option_text(value.binding),
        value.local.as_u32(),
        value.name.as_u32()
    ));
}

fn parse_name_ref(text: &str) -> Result<NameRef, Diagnostic> {
    let parts = text.split(':').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(implementation_artifact_diagnostic(
            "invalid package implementation name reference",
        ));
    }
    Ok(NameRef {
        binding: parse_binding_option(parts[0])?,
        local: LocalId::new(parse_u32(parts[1], "name local")?),
        name: Symbol::new(parse_u32(parts[2], "name symbol")?),
    })
}

fn span_text(span: Span) -> String {
    format!(
        "{}:{}:{}:{}",
        span.start.line, span.start.column, span.end.line, span.end.column
    )
}

fn parse_span(text: &str) -> Result<Span, Diagnostic> {
    let parts = text.split(':').collect::<Vec<_>>();
    if parts.len() != 4 {
        return Err(implementation_artifact_diagnostic(
            "invalid package implementation span",
        ));
    }
    Ok(Span::new(
        Position::new(
            parse_usize(parts[0], "span start line")?,
            parse_usize(parts[1], "span start column")?,
        ),
        Position::new(
            parse_usize(parts[2], "span end line")?,
            parse_usize(parts[3], "span end column")?,
        ),
    ))
}

fn symbol_list_text(symbols: &[Symbol]) -> String {
    symbols
        .iter()
        .map(|symbol| symbol.as_u32().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn parse_symbol_list(text: &str, expected: usize) -> Result<Vec<Symbol>, Diagnostic> {
    if text.is_empty() {
        if expected == 0 {
            return Ok(Vec::new());
        }
        return Err(implementation_artifact_diagnostic(
            "invalid package implementation symbol list",
        ));
    }
    let symbols = text
        .split(',')
        .map(|part| parse_u32(part, "symbol").map(Symbol::new))
        .collect::<Result<Vec<_>, _>>()?;
    if symbols.len() == expected {
        Ok(symbols)
    } else {
        Err(implementation_artifact_diagnostic(
            "package implementation symbol list length mismatch",
        ))
    }
}

fn package_item_option_text(item: Option<PackageItemId>) -> String {
    item.map(|item| item.as_u32().to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn parse_package_item_option(text: &str) -> Result<Option<PackageItemId>, Diagnostic> {
    if text == "-" {
        Ok(None)
    } else {
        Ok(Some(PackageItemId::new(parse_u32(text, "package item")?)))
    }
}

fn binding_option_text(binding: Option<BindingId>) -> String {
    binding
        .map(|binding| binding.as_u32().to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn parse_binding_option(text: &str) -> Result<Option<BindingId>, Diagnostic> {
    if text == "-" {
        Ok(None)
    } else {
        Ok(Some(BindingId::new(parse_u32(text, "binding")?)))
    }
}

fn symbol_option_text(symbol: Option<Symbol>) -> String {
    symbol
        .map(|symbol| symbol.as_u32().to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn parse_symbol_option(text: &str) -> Result<Option<Symbol>, Diagnostic> {
    if text == "-" {
        Ok(None)
    } else {
        Ok(Some(Symbol::new(parse_u32(text, "symbol")?)))
    }
}

fn binding_kind_text(kind: BindingKind) -> &'static str {
    match kind {
        BindingKind::Immutable => "immutable",
        BindingKind::Mutable => "mutable",
        BindingKind::Function => "function",
        BindingKind::Parameter => "parameter",
    }
}

fn parse_binding_kind(text: &str) -> Result<BindingKind, Diagnostic> {
    match text {
        "immutable" => Ok(BindingKind::Immutable),
        "mutable" => Ok(BindingKind::Mutable),
        "function" => Ok(BindingKind::Function),
        "parameter" => Ok(BindingKind::Parameter),
        _ => Err(implementation_artifact_diagnostic(
            "invalid package implementation binding kind",
        )),
    }
}

fn local_kind_text(kind: LocalKind) -> &'static str {
    match kind {
        LocalKind::Binding(BindingKind::Immutable) => "binding-immutable",
        LocalKind::Binding(BindingKind::Mutable) => "binding-mutable",
        LocalKind::Binding(BindingKind::Function) => "binding-function",
        LocalKind::Binding(BindingKind::Parameter) => "binding-parameter",
        LocalKind::Synthetic => "synthetic",
    }
}

fn parse_local_kind(text: &str) -> Result<LocalKind, Diagnostic> {
    match text {
        "binding-immutable" => Ok(LocalKind::Binding(BindingKind::Immutable)),
        "binding-mutable" => Ok(LocalKind::Binding(BindingKind::Mutable)),
        "binding-function" => Ok(LocalKind::Binding(BindingKind::Function)),
        "binding-parameter" => Ok(LocalKind::Binding(BindingKind::Parameter)),
        "synthetic" => Ok(LocalKind::Synthetic),
        _ => Err(implementation_artifact_diagnostic(
            "invalid package implementation local kind",
        )),
    }
}

fn binary_op_text(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "add",
        BinaryOp::Sub => "sub",
        BinaryOp::Mul => "mul",
        BinaryOp::Div => "div",
        BinaryOp::Lt => "lt",
        BinaryOp::LtEq => "lteq",
        BinaryOp::Gt => "gt",
        BinaryOp::GtEq => "gteq",
        BinaryOp::EqEq => "eqeq",
        BinaryOp::BangEq => "bangeq",
    }
}

fn parse_binary_op(text: &str) -> Result<BinaryOp, Diagnostic> {
    match text {
        "add" => Ok(BinaryOp::Add),
        "sub" => Ok(BinaryOp::Sub),
        "mul" => Ok(BinaryOp::Mul),
        "div" => Ok(BinaryOp::Div),
        "lt" => Ok(BinaryOp::Lt),
        "lteq" => Ok(BinaryOp::LtEq),
        "gt" => Ok(BinaryOp::Gt),
        "gteq" => Ok(BinaryOp::GtEq),
        "eqeq" => Ok(BinaryOp::EqEq),
        "bangeq" => Ok(BinaryOp::BangEq),
        _ => Err(implementation_artifact_diagnostic(
            "invalid package implementation binary operator",
        )),
    }
}

fn package_item_kind_text(kind: PackageItemKind) -> &'static str {
    match kind {
        PackageItemKind::Record => "record",
        PackageItemKind::Enum => "enum",
        PackageItemKind::Function => "function",
    }
}

fn package_item_kind_label(kind: PackageItemKind) -> &'static str {
    package_item_kind_text(kind)
}

fn parse_package_item_kind(text: &str) -> Result<PackageItemKind, Diagnostic> {
    match text {
        "record" => Ok(PackageItemKind::Record),
        "enum" => Ok(PackageItemKind::Enum),
        "function" => Ok(PackageItemKind::Function),
        _ => Err(implementation_artifact_diagnostic(
            "invalid package implementation item kind",
        )),
    }
}

fn visibility_text(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Private => "private",
        Visibility::Package => "package",
        Visibility::Public => "public",
    }
}

fn parse_visibility(text: &str) -> Result<Visibility, Diagnostic> {
    match text {
        "private" => Ok(Visibility::Private),
        "package" => Ok(Visibility::Package),
        "public" => Ok(Visibility::Public),
        _ => Err(implementation_artifact_diagnostic(
            "invalid package implementation item visibility",
        )),
    }
}

fn parse_bool(text: &str, label: &str) -> Result<bool, Diagnostic> {
    text.parse::<bool>().map_err(|_| {
        implementation_artifact_diagnostic(format!("invalid package implementation {label}"))
    })
}

fn parse_i64(text: &str, label: &str) -> Result<i64, Diagnostic> {
    text.parse::<i64>().map_err(|_| {
        implementation_artifact_diagnostic(format!("invalid package implementation {label}"))
    })
}

fn parse_u32(text: &str, label: &str) -> Result<u32, Diagnostic> {
    text.parse::<u32>().map_err(|_| {
        implementation_artifact_diagnostic(format!("invalid package implementation {label}"))
    })
}

fn parse_usize(text: &str, label: &str) -> Result<usize, Diagnostic> {
    text.parse::<usize>().map_err(|_| {
        implementation_artifact_diagnostic(format!("invalid package implementation {label}"))
    })
}

fn escape_field(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

fn unescape_field(value: &str) -> Result<String, Diagnostic> {
    let mut out = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        let Some(escaped) = chars.next() else {
            return Err(implementation_artifact_diagnostic(
                "invalid trailing escape in package implementation artifact",
            ));
        };
        match escaped {
            '\\' => out.push('\\'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            't' => out.push('\t'),
            _ => {
                return Err(implementation_artifact_diagnostic(
                    "invalid escape in package implementation artifact",
                ));
            }
        }
    }
    Ok(out)
}

fn implementation_artifact_diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("PK022", message, Span::default())
        .with_suggestion("regenerate the package implementation artifact")
}

fn invalid_bytecode_diagnostic(message: impl Into<String>) -> Diagnostic {
    implementation_artifact_diagnostic(format!(
        "invalid package implementation bytecode: {}",
        message.into()
    ))
}

fn stale_implementation_artifact_diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("PK023", message, Span::default())
        .with_suggestion("regenerate the package implementation artifact")
}
