use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::identity::{ExprId, ModuleId, PackageId, PackageItemId, StmtId};
use crate::interface::{PackageExportGraph, PackageInterface, PackageInterfaceGraph};
use crate::span::Span;
use crate::symbol::{Symbol, SymbolTable};
use crate::types::{FunctionTypeInfo, TypeInfo};

pub fn load_program_from_entry(path: &Path) -> Result<Program, Vec<Diagnostic>> {
    Ok(load_from_entry(path)?.program)
}

pub fn import_paths_from_entry(path: &Path) -> Result<Vec<String>, Vec<Diagnostic>> {
    let (entry_program, manifest) = parse_entry_program(path)?;
    if entry_program.package.is_none() {
        return Ok(Vec::new());
    }

    let mut loader = PackageLoader::new(path.to_path_buf(), entry_program, manifest);
    loader.load_entry_import_paths()
}

pub fn source_fingerprint_input_from_entry(path: &Path) -> Result<String, Vec<Diagnostic>> {
    let (entry_program, manifest) = parse_entry_program(path)?;
    if entry_program.package.is_none() {
        let source = fs::read_to_string(path).map_err(|error| {
            vec![Diagnostic::new(
                "PK002",
                format!("failed to read {}: {error}", path.display()),
                Span::default(),
            )]
        })?;
        return Ok(format!("script\t{}\n{source}\n", source.len()));
    }

    let mut loader = PackageLoader::new(path.to_path_buf(), entry_program, manifest);
    loader.entry_package_source_fingerprint_input()
}

pub fn load_from_entry(path: &Path) -> Result<LoadedProgram, Vec<Diagnostic>> {
    let (entry_program, manifest) = parse_entry_program(path)?;
    if entry_program.package.is_none() {
        return Ok(LoadedProgram {
            program: entry_program,
            package_graph: PackageSymbolGraph::default(),
            package_exports: PackageExportGraph::default(),
        });
    }

    let mut loader = PackageLoader::new(path.to_path_buf(), entry_program, manifest);
    loader.load_and_flatten()
}

pub fn load_from_entry_against_interfaces(
    path: &Path,
    interfaces: &PackageInterfaceGraph,
    interface_symbols: &SymbolTable,
) -> Result<LoadedProgram, Vec<Diagnostic>> {
    let (entry_program, manifest) = parse_entry_program(path)?;
    if entry_program.package.is_none() {
        return Ok(LoadedProgram {
            program: entry_program,
            package_graph: PackageSymbolGraph::default(),
            package_exports: PackageExportGraph::default(),
        });
    }

    let mut loader = PackageLoader::new(path.to_path_buf(), entry_program, manifest);
    loader.load_and_flatten_against_interfaces(interfaces, interface_symbols)
}

fn parse_entry_program(path: &Path) -> Result<(Program, Option<ProjectManifest>), Vec<Diagnostic>> {
    let entry_source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            return Err(vec![Diagnostic::new(
                "PK002",
                format!("failed to read {}: {error}", path.display()),
                Span::default(),
            )]);
        }
    };
    let entry_tokens = crate::lexer::lex(&entry_source)?;
    let manifest = discover_manifest(path)?;
    let entry_program = if let Some(manifest) = &manifest {
        let inferred_package = infer_manifest_package_path(path, manifest)?;
        let program =
            crate::parser::parse_inferred_package(entry_tokens, inferred_package.clone())?;
        if let Some(package) = &program.package {
            if package.path != inferred_package {
                return Err(vec![Diagnostic::new(
                    "PK006",
                    format!(
                        "file {} declares package `{}` but manifest layout expects `{inferred_package}`",
                        path.display(),
                        package.path
                    ),
                    package.span,
                )]);
            }
        }
        program
    } else {
        crate::parser::parse(entry_tokens)?
    };
    Ok((entry_program, manifest))
}

#[derive(Clone, Debug)]
pub struct LoadedProgram {
    pub program: Program,
    pub package_graph: PackageSymbolGraph,
    pub package_exports: PackageExportGraph,
}

#[derive(Clone, Debug, Default)]
pub struct PackageSymbolGraph {
    pub packages: Vec<PackageInfo>,
    pub modules: Vec<PackageModuleInfo>,
    pub items: Vec<PackageItemInfo>,
}

impl PackageSymbolGraph {
    pub fn package(&self, id: PackageId) -> Option<&PackageInfo> {
        self.packages.get(id.as_u32() as usize)
    }

    pub fn item(&self, id: PackageItemId) -> Option<&PackageItemInfo> {
        self.items.get(id.as_u32() as usize)
    }

    pub fn module(&self, id: ModuleId) -> Option<&PackageModuleInfo> {
        self.modules.get(id.as_u32() as usize)
    }

    pub fn module_id(&self, package: PackageId, path: &str) -> Option<ModuleId> {
        self.modules
            .iter()
            .find(|module| module.package == package && module.path == path)
            .map(|module| module.id)
    }

    pub fn package_id(&self, path: &str) -> Option<PackageId> {
        self.packages
            .iter()
            .find(|package| package.path == path)
            .map(|package| package.id)
    }

    pub fn item_id(
        &self,
        package: PackageId,
        name: &str,
        kind: PackageItemKind,
    ) -> Option<PackageItemId> {
        self.items
            .iter()
            .find(|item| item.package == package && item.name == name && item.kind == kind)
            .map(|item| item.id)
    }

    pub fn item_id_in_module(
        &self,
        module: ModuleId,
        name: &str,
        kind: PackageItemKind,
    ) -> Option<PackageItemId> {
        self.items
            .iter()
            .find(|item| item.module == module && item.name == name && item.kind == kind)
            .map(|item| item.id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInfo {
    pub id: PackageId,
    pub path: String,
    pub modules: Vec<ModuleId>,
    pub imports: Vec<PackageImportInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageModuleInfo {
    pub id: ModuleId,
    pub package: PackageId,
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageImportInfo {
    pub alias: String,
    pub package: PackageId,
    pub path: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageItemInfo {
    pub id: PackageItemId,
    pub package: PackageId,
    pub module: ModuleId,
    pub name: String,
    pub kind: PackageItemKind,
    pub visibility: Visibility,
    pub span: Span,
    pub mangled_name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageItemKind {
    Record,
    Enum,
    Function,
}

struct ParsedFile {
    program: Program,
    module_path: String,
}

#[derive(Clone, Debug)]
struct PackageSourceFile {
    module_path: String,
    source: String,
}

#[derive(Clone, Debug)]
struct ProjectManifest {
    source_root: PathBuf,
    name: String,
}

struct PackageData {
    files: Vec<ParsedFile>,
    records: HashMap<String, Vec<PackageItemDecl>>,
    enums: HashMap<String, Vec<PackageItemDecl>>,
    functions: HashMap<String, Vec<PackageItemDecl>>,
}

#[derive(Clone, Debug)]
struct PackageItemDecl {
    visibility: Visibility,
    module_path: String,
    span: Span,
}

struct PackageItemDeclInput<'a> {
    name: &'a str,
    visibility: Visibility,
    module_path: &'a str,
    span: Span,
    kind: PackageItemKind,
    package_path: &'a str,
}

struct PackageLoader {
    entry_file: PathBuf,
    source_root: PathBuf,
    entry_package: String,
    manifest: Option<ProjectManifest>,
    packages: HashMap<String, PackageData>,
    loading: HashSet<String>,
    diagnostics: Vec<Diagnostic>,
}

impl PackageLoader {
    fn new(entry_file: PathBuf, entry_program: Program, manifest: Option<ProjectManifest>) -> Self {
        let entry_package = entry_program
            .package
            .as_ref()
            .expect("checked package mode")
            .path
            .clone();
        let source_root = manifest
            .as_ref()
            .map(|manifest| manifest.source_root.clone())
            .unwrap_or_else(|| {
                infer_source_root(&entry_file, &entry_package)
                    .unwrap_or_else(|_| entry_file.clone())
            });
        Self {
            entry_file,
            source_root,
            entry_package,
            manifest,
            packages: HashMap::new(),
            loading: HashSet::new(),
            diagnostics: Vec::new(),
        }
    }

    fn load_and_flatten(&mut self) -> Result<LoadedProgram, Vec<Diagnostic>> {
        if let Err(diagnostic) = self.ensure_source_root() {
            self.diagnostics.push(diagnostic);
            return Err(std::mem::take(&mut self.diagnostics));
        }

        self.load_package(self.entry_package.clone());

        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }

        let package_paths = self.sorted_package_paths();
        let package_graph = self.build_symbol_graph(&package_paths);
        let package_exports = PackageExportGraph::from_symbol_graph(&package_graph);
        self.flatten_packages(&package_paths, package_graph, package_exports)
    }

    fn load_and_flatten_against_interfaces(
        &mut self,
        interfaces: &PackageInterfaceGraph,
        interface_symbols: &SymbolTable,
    ) -> Result<LoadedProgram, Vec<Diagnostic>> {
        if let Err(diagnostic) = self.ensure_source_root() {
            self.diagnostics.push(diagnostic);
            return Err(std::mem::take(&mut self.diagnostics));
        }

        let entry_package = self.entry_package.clone();
        let files = self.load_package_files(&entry_package);
        let entry_data = collect_package_data(&entry_package, files, &mut self.diagnostics);
        let imported_paths = package_import_paths(&entry_data);
        self.packages.insert(entry_package.clone(), entry_data);

        for import_path in &imported_paths {
            if interfaces.package_by_path(import_path).is_none() {
                self.diagnostics.push(
                    Diagnostic::new(
                        "PK016",
                        format!("missing loaded package interface for `{import_path}`"),
                        Span::default(),
                    )
                    .with_suggestion("load or regenerate the package interface before checking"),
                );
            }
        }

        for interface in &interfaces.packages {
            if interface.path == entry_package {
                continue;
            }
            let package_data = stub_package_data_from_interface(
                interface,
                interfaces,
                interface_symbols,
                &mut self.diagnostics,
            );
            self.packages.insert(interface.path.clone(), package_data);
        }

        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }

        let package_paths = self.sorted_package_paths();
        let package_graph = self.build_symbol_graph_against_interfaces(&package_paths, interfaces);
        let package_exports = PackageExportGraph::from_interfaces(interfaces, &package_graph);
        self.flatten_packages(&package_paths, package_graph, package_exports)
    }

    fn load_entry_import_paths(&mut self) -> Result<Vec<String>, Vec<Diagnostic>> {
        if let Err(diagnostic) = self.ensure_source_root() {
            self.diagnostics.push(diagnostic);
            return Err(std::mem::take(&mut self.diagnostics));
        }

        let entry_package = self.entry_package.clone();
        let files = self.load_package_files(&entry_package);
        let entry_data = collect_package_data(&entry_package, files, &mut self.diagnostics);
        if self.diagnostics.is_empty() {
            Ok(package_import_paths(&entry_data))
        } else {
            Err(std::mem::take(&mut self.diagnostics))
        }
    }

    fn entry_package_source_fingerprint_input(&mut self) -> Result<String, Vec<Diagnostic>> {
        if let Err(diagnostic) = self.ensure_source_root() {
            self.diagnostics.push(diagnostic);
            return Err(std::mem::take(&mut self.diagnostics));
        }

        let entry_package = self.entry_package.clone();
        let files = self.load_package_source_files(&entry_package);
        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }

        let mut input = format!("package\t{entry_package}\n");
        for file in files {
            input.push_str(&format!(
                "file\t{}\t{}\n{}\n",
                file.module_path,
                file.source.len(),
                file.source
            ));
        }
        Ok(input)
    }

    fn ensure_source_root(&mut self) -> Result<(), Diagnostic> {
        if self.manifest.is_none() {
            self.source_root = infer_source_root(&self.entry_file, &self.entry_package)?;
        }
        Ok(())
    }

    fn flatten_packages(
        &mut self,
        package_paths: &[String],
        package_graph: PackageSymbolGraph,
        package_exports: PackageExportGraph,
    ) -> Result<LoadedProgram, Vec<Diagnostic>> {
        let mut statements = Vec::new();
        for package_path in package_paths {
            let Some(package) = self.packages.get(package_path) else {
                continue;
            };
            for file in &package.files {
                let import_aliases =
                    file_import_aliases(&file.program.imports, &mut self.diagnostics);
                let mut rewriter = PackageRewriter {
                    diagnostics: &mut self.diagnostics,
                    current_package: package_path.clone(),
                    current_module: file.module_path.clone(),
                    entry_package: self.entry_package.clone(),
                    imports: import_aliases,
                    current_package_data: package,
                    packages: &self.packages,
                    package_graph: &package_graph,
                    package_exports: &package_exports,
                    scopes: Vec::new(),
                };
                for statement in &file.program.statements {
                    statements.push(rewriter.rewrite_top_level_stmt(statement));
                }
            }
        }

        if self.diagnostics.is_empty() {
            let mut program = Program {
                package: None,
                imports: Vec::new(),
                statements,
            };
            renumber_node_ids(&mut program);
            Ok(LoadedProgram {
                program,
                package_graph,
                package_exports,
            })
        } else {
            Err(std::mem::take(&mut self.diagnostics))
        }
    }

    fn load_package(&mut self, package_path: String) {
        if self.packages.contains_key(&package_path) {
            return;
        }
        if !self.loading.insert(package_path.clone()) {
            self.diagnostics.push(Diagnostic::new(
                "PK008",
                format!("import cycle detected at package `{package_path}`"),
                Span::default(),
            ));
            return;
        }

        let files = self.load_package_files(&package_path);
        for file in &files {
            for import in &file.program.imports {
                self.load_package(import.path.clone());
            }
        }

        let package_data = collect_package_data(&package_path, files, &mut self.diagnostics);
        self.packages.insert(package_path.clone(), package_data);
        self.loading.remove(&package_path);
    }

    fn load_package_files(&mut self, package_path: &str) -> Vec<ParsedFile> {
        let package_dir = self.package_dir(package_path);
        let read_dir = match fs::read_dir(&package_dir) {
            Ok(read_dir) => read_dir,
            Err(error) => {
                self.diagnostics.push(Diagnostic::new(
                    "PK002",
                    format!(
                        "failed to read package directory {}: {error}",
                        package_dir.display()
                    ),
                    Span::default(),
                ));
                return Vec::new();
            }
        };

        let mut file_paths: Vec<PathBuf> = read_dir
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "muga"))
            .collect();
        file_paths.sort();

        if file_paths.is_empty() {
            self.diagnostics.push(Diagnostic::new(
                "PK004",
                format!("package `{package_path}` does not contain any `.muga` files"),
                Span::default(),
            ));
            return Vec::new();
        }

        let mut files = Vec::new();
        for file_path in file_paths {
            let source = match fs::read_to_string(&file_path) {
                Ok(source) => source,
                Err(error) => {
                    self.diagnostics.push(Diagnostic::new(
                        "PK002",
                        format!("failed to read {}: {error}", file_path.display()),
                        Span::default(),
                    ));
                    continue;
                }
            };
            let program = match self.parse_package_file(&source, package_path) {
                Ok(program) => program,
                Err(diagnostics) => {
                    self.diagnostics.extend(diagnostics);
                    continue;
                }
            };
            match &program.package {
                Some(package) if package.path == package_path => {}
                Some(package) => {
                    self.diagnostics.push(Diagnostic::new(
                        "PK006",
                        format!(
                            "file {} declares package `{}` but directory expects `{package_path}`",
                            file_path.display(),
                            package.path
                        ),
                        package.span,
                    ));
                    continue;
                }
                None => {
                    self.diagnostics.push(Diagnostic::new(
                        "PK005",
                        format!(
                            "package directory file {} must begin with `package {package_path}`",
                            file_path.display()
                        ),
                        Span::default(),
                    ));
                    continue;
                }
            }
            let module_path = module_path_for_file(&file_path);
            files.push(ParsedFile {
                program,
                module_path,
            });
        }
        files
    }

    fn load_package_source_files(&mut self, package_path: &str) -> Vec<PackageSourceFile> {
        let package_dir = self.package_dir(package_path);
        let read_dir = match fs::read_dir(&package_dir) {
            Ok(read_dir) => read_dir,
            Err(error) => {
                self.diagnostics.push(Diagnostic::new(
                    "PK002",
                    format!(
                        "failed to read package directory {}: {error}",
                        package_dir.display()
                    ),
                    Span::default(),
                ));
                return Vec::new();
            }
        };

        let mut file_paths: Vec<PathBuf> = read_dir
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "muga"))
            .collect();
        file_paths.sort();

        if file_paths.is_empty() {
            self.diagnostics.push(Diagnostic::new(
                "PK004",
                format!("package `{package_path}` does not contain any `.muga` files"),
                Span::default(),
            ));
            return Vec::new();
        }

        let mut files = Vec::new();
        for file_path in file_paths {
            let source = match fs::read_to_string(&file_path) {
                Ok(source) => source,
                Err(error) => {
                    self.diagnostics.push(Diagnostic::new(
                        "PK002",
                        format!("failed to read {}: {error}", file_path.display()),
                        Span::default(),
                    ));
                    continue;
                }
            };
            files.push(PackageSourceFile {
                module_path: module_path_for_file(&file_path),
                source,
            });
        }
        files
    }

    fn parse_package_file(
        &self,
        source: &str,
        package_path: &str,
    ) -> Result<Program, Vec<Diagnostic>> {
        let tokens = crate::lexer::lex(source)?;
        if self.manifest.is_some() {
            crate::parser::parse_inferred_package(tokens, package_path.to_string())
        } else {
            crate::parser::parse(tokens)
        }
    }

    fn package_dir(&self, package_path: &str) -> PathBuf {
        if let Some(manifest) = &self.manifest {
            if package_path == manifest.name {
                return self.source_root.clone();
            }
            if let Some(rest) = package_path.strip_prefix(&(manifest.name.clone() + "::")) {
                let mut path = self.source_root.clone();
                for segment in split_package_path(rest) {
                    path.push(segment);
                }
                return path;
            }
        }

        let mut path = self.source_root.clone();
        for segment in split_package_path(package_path) {
            path.push(segment);
        }
        path
    }

    fn sorted_package_paths(&self) -> Vec<String> {
        let mut package_paths: Vec<String> = self.packages.keys().cloned().collect();
        package_paths.sort();
        package_paths
    }

    fn build_symbol_graph(&self, package_paths: &[String]) -> PackageSymbolGraph {
        let package_ids: HashMap<&str, PackageId> = package_paths
            .iter()
            .enumerate()
            .map(|(index, path)| (path.as_str(), PackageId::new(index as u32)))
            .collect();

        let mut packages = Vec::with_capacity(package_paths.len());
        let mut modules = Vec::new();
        let mut items = Vec::new();

        for package_path in package_paths {
            let Some(package) = self.packages.get(package_path) else {
                continue;
            };
            let package_id = package_ids[package_path.as_str()];
            let mut package_modules = Vec::new();
            let mut file_modules = HashMap::new();
            for file in &package.files {
                let id = ModuleId::new(modules.len() as u32);
                package_modules.push(id);
                file_modules.insert(file.module_path.as_str(), id);
                modules.push(PackageModuleInfo {
                    id,
                    package: package_id,
                    path: file.module_path.clone(),
                });
            }
            let mut imports = Vec::new();
            for file in &package.files {
                for import in &file.program.imports {
                    if let Some(imported_package) = package_ids.get(import.path.as_str()) {
                        imports.push(PackageImportInfo {
                            alias: import.alias.clone(),
                            package: *imported_package,
                            path: import.path.clone(),
                            span: import.span,
                        });
                    }
                }
            }
            packages.push(PackageInfo {
                id: package_id,
                path: package_path.clone(),
                modules: package_modules,
                imports,
            });

            for file in &package.files {
                let module_id = file_modules[file.module_path.as_str()];
                for statement in &file.program.statements {
                    match statement {
                        Stmt::RecordDecl(record) => {
                            let id = PackageItemId::new(items.len() as u32);
                            items.push(PackageItemInfo {
                                id,
                                package: package_id,
                                module: module_id,
                                name: record.name.clone(),
                                kind: PackageItemKind::Record,
                                visibility: record.visibility,
                                span: record.span,
                                mangled_name: mangle_record_name_for_visibility(
                                    package_path,
                                    &file.module_path,
                                    &record.name,
                                    record.visibility,
                                ),
                            });
                        }
                        Stmt::EnumDecl(enumeration) => {
                            let id = PackageItemId::new(items.len() as u32);
                            items.push(PackageItemInfo {
                                id,
                                package: package_id,
                                module: module_id,
                                name: enumeration.name.clone(),
                                kind: PackageItemKind::Enum,
                                visibility: enumeration.visibility,
                                span: enumeration.span,
                                mangled_name: mangle_enum_name_for_visibility(
                                    package_path,
                                    &file.module_path,
                                    &enumeration.name,
                                    enumeration.visibility,
                                ),
                            });
                        }
                        Stmt::FuncDecl(func) => {
                            let id = PackageItemId::new(items.len() as u32);
                            items.push(PackageItemInfo {
                                id,
                                package: package_id,
                                module: module_id,
                                name: func.name.clone(),
                                kind: PackageItemKind::Function,
                                visibility: func.visibility,
                                span: func.span,
                                mangled_name: mangle_function_name_for_visibility(
                                    package_path,
                                    &file.module_path,
                                    &func.name,
                                    func.visibility,
                                    &self.entry_package,
                                ),
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        PackageSymbolGraph {
            packages,
            modules,
            items,
        }
    }

    fn build_symbol_graph_against_interfaces(
        &mut self,
        package_paths: &[String],
        interfaces: &PackageInterfaceGraph,
    ) -> PackageSymbolGraph {
        let mut package_ids: HashMap<String, PackageId> = interfaces
            .packages
            .iter()
            .map(|interface| (interface.path.clone(), interface.package))
            .collect();
        let mut next_package_id = interfaces
            .packages
            .iter()
            .map(|interface| interface.package.as_u32())
            .max()
            .map_or(0, |id| id + 1);
        for package_path in package_paths {
            package_ids.entry(package_path.clone()).or_insert_with(|| {
                let id = PackageId::new(next_package_id);
                next_package_id += 1;
                id
            });
        }

        let max_package_id = package_ids
            .values()
            .map(|id| id.as_u32())
            .max()
            .unwrap_or(0) as usize;
        let max_item_id = interfaces
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
            .map(|id| id.as_u32())
            .max()
            .map_or(0, |id| id + 1) as usize;

        let mut package_slots: Vec<Option<PackageInfo>> = vec![None; max_package_id + 1];
        let mut modules = Vec::new();
        let mut item_slots: Vec<Option<PackageItemInfo>> = vec![None; max_item_id];

        for package_path in package_paths {
            let Some(package) = self.packages.get(package_path) else {
                continue;
            };
            let package_id = package_ids[package_path];
            let mut package_modules = Vec::new();
            let mut file_modules = HashMap::new();
            for file in &package.files {
                let id = ModuleId::new(modules.len() as u32);
                package_modules.push(id);
                file_modules.insert(file.module_path.as_str(), id);
                modules.push(PackageModuleInfo {
                    id,
                    package: package_id,
                    path: file.module_path.clone(),
                });
            }
            let mut imports = Vec::new();
            for file in &package.files {
                for import in &file.program.imports {
                    if let Some(imported_package) = package_ids.get(&import.path) {
                        imports.push(PackageImportInfo {
                            alias: import.alias.clone(),
                            package: *imported_package,
                            path: import.path.clone(),
                            span: import.span,
                        });
                    }
                }
            }
            package_slots[package_id.as_u32() as usize] = Some(PackageInfo {
                id: package_id,
                path: package_path.clone(),
                modules: package_modules,
                imports,
            });

            let package_interface = interfaces.package_by_path(package_path);
            for file in &package.files {
                let module_id = file_modules[file.module_path.as_str()];
                for statement in &file.program.statements {
                    match statement {
                        Stmt::RecordDecl(record) => {
                            let id = interface_item_id(
                                package_interface,
                                &record.name,
                                PackageItemKind::Record,
                                record.visibility,
                            )
                            .unwrap_or_else(|| allocate_package_item_id(&mut item_slots));
                            insert_package_graph_item(
                                &mut item_slots,
                                PackageItemInfo {
                                    id,
                                    package: package_id,
                                    module: module_id,
                                    name: record.name.clone(),
                                    kind: PackageItemKind::Record,
                                    visibility: record.visibility,
                                    span: record.span,
                                    mangled_name: mangle_record_name_for_visibility(
                                        package_path,
                                        &file.module_path,
                                        &record.name,
                                        record.visibility,
                                    ),
                                },
                                &mut self.diagnostics,
                            );
                        }
                        Stmt::EnumDecl(enumeration) => {
                            let id = interface_item_id(
                                package_interface,
                                &enumeration.name,
                                PackageItemKind::Enum,
                                enumeration.visibility,
                            )
                            .unwrap_or_else(|| allocate_package_item_id(&mut item_slots));
                            insert_package_graph_item(
                                &mut item_slots,
                                PackageItemInfo {
                                    id,
                                    package: package_id,
                                    module: module_id,
                                    name: enumeration.name.clone(),
                                    kind: PackageItemKind::Enum,
                                    visibility: enumeration.visibility,
                                    span: enumeration.span,
                                    mangled_name: mangle_enum_name_for_visibility(
                                        package_path,
                                        &file.module_path,
                                        &enumeration.name,
                                        enumeration.visibility,
                                    ),
                                },
                                &mut self.diagnostics,
                            );
                        }
                        Stmt::FuncDecl(func) => {
                            let id = interface_item_id(
                                package_interface,
                                &func.name,
                                PackageItemKind::Function,
                                func.visibility,
                            )
                            .unwrap_or_else(|| allocate_package_item_id(&mut item_slots));
                            insert_package_graph_item(
                                &mut item_slots,
                                PackageItemInfo {
                                    id,
                                    package: package_id,
                                    module: module_id,
                                    name: func.name.clone(),
                                    kind: PackageItemKind::Function,
                                    visibility: func.visibility,
                                    span: func.span,
                                    mangled_name: mangle_function_name_for_visibility(
                                        package_path,
                                        &file.module_path,
                                        &func.name,
                                        func.visibility,
                                        &self.entry_package,
                                    ),
                                },
                                &mut self.diagnostics,
                            );
                        }
                        _ => {}
                    }
                }
            }
        }

        let fallback_package = package_ids
            .values()
            .copied()
            .next()
            .unwrap_or_else(|| PackageId::new(0));
        let fallback_module = modules
            .first()
            .map(|module| module.id)
            .unwrap_or(ModuleId::new(0));
        let packages = package_slots
            .into_iter()
            .enumerate()
            .map(|(index, package)| {
                package.unwrap_or_else(|| PackageInfo {
                    id: PackageId::new(index as u32),
                    path: format!("<interface-gap-{index}>"),
                    modules: Vec::new(),
                    imports: Vec::new(),
                })
            })
            .collect();
        let items = item_slots
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                item.unwrap_or_else(|| PackageItemInfo {
                    id: PackageItemId::new(index as u32),
                    package: fallback_package,
                    module: fallback_module,
                    name: format!("<interface-gap-{index}>"),
                    kind: PackageItemKind::Function,
                    visibility: Visibility::Private,
                    span: Span::default(),
                    mangled_name: format!("__muga_interface_gap_{index}"),
                })
            })
            .collect();

        PackageSymbolGraph {
            packages,
            modules,
            items,
        }
    }
}

struct PackageRewriter<'a> {
    diagnostics: &'a mut Vec<Diagnostic>,
    current_package: String,
    current_module: String,
    entry_package: String,
    imports: HashMap<String, String>,
    current_package_data: &'a PackageData,
    packages: &'a HashMap<String, PackageData>,
    package_graph: &'a PackageSymbolGraph,
    package_exports: &'a PackageExportGraph,
    scopes: Vec<HashSet<String>>,
}

impl<'a> PackageRewriter<'a> {
    fn rewrite_top_level_stmt(&mut self, statement: &Stmt) -> Stmt {
        match statement {
            Stmt::RecordDecl(record) => Stmt::RecordDecl(self.rewrite_record_decl(record)),
            Stmt::EnumDecl(enumeration) => Stmt::EnumDecl(self.rewrite_enum_decl(enumeration)),
            Stmt::FuncDecl(func) => Stmt::FuncDecl(self.rewrite_func_decl(func, true)),
            _ => statement.clone(),
        }
    }

    fn rewrite_record_decl(&mut self, record: &RecordDecl) -> RecordDecl {
        if record.visibility == Visibility::Public || record.visibility == Visibility::Package {
            for field in &record.fields {
                self.validate_visible_type(&field.type_name, record.visibility, field.span);
            }
        }

        RecordDecl {
            id: record.id,
            package_item: self.package_item_id(&record.name, PackageItemKind::Record),
            name: mangle_record_name_for_visibility(
                &self.current_package,
                &self.current_module,
                &record.name,
                record.visibility,
            ),
            visibility: Visibility::Private,
            fields: record
                .fields
                .iter()
                .map(|field| RecordFieldDecl {
                    name: field.name.clone(),
                    type_name: self.rewrite_type_expr(&field.type_name, field.span),
                    span: field.span,
                })
                .collect(),
            span: record.span,
        }
    }

    fn rewrite_enum_decl(&mut self, enumeration: &EnumDecl) -> EnumDecl {
        if enumeration.visibility == Visibility::Public
            || enumeration.visibility == Visibility::Package
        {
            for variant in &enumeration.variants {
                if let Some(payload) = &variant.payload {
                    self.validate_visible_type(payload, enumeration.visibility, variant.span);
                }
            }
        }

        EnumDecl {
            id: enumeration.id,
            package_item: self.package_item_id(&enumeration.name, PackageItemKind::Enum),
            name: mangle_enum_name_for_visibility(
                &self.current_package,
                &self.current_module,
                &enumeration.name,
                enumeration.visibility,
            ),
            visibility: Visibility::Private,
            type_params: enumeration.type_params.clone(),
            variants: enumeration
                .variants
                .iter()
                .map(|variant| EnumVariantDecl {
                    name: variant.name.clone(),
                    payload: variant
                        .payload
                        .as_ref()
                        .map(|payload| self.rewrite_type_expr(payload, variant.span)),
                    span: variant.span,
                })
                .collect(),
            span: enumeration.span,
        }
    }

    fn rewrite_func_decl(&mut self, func: &FuncDecl, top_level: bool) -> FuncDecl {
        if top_level && func.visibility == Visibility::Public {
            let has_full_signature = func.params.iter().all(|param| param.type_name.is_some())
                && func.return_type.is_some();
            if !has_full_signature {
                self.diagnostics.push(
                    Diagnostic::new(
                        "PK011",
                        "public functions must annotate every parameter and the return type",
                        func.span,
                    )
                    .with_suggestion("add parameter type annotations and an explicit return type"),
                );
            }
            for param in &func.params {
                if let Some(type_name) = &param.type_name {
                    self.validate_visible_type(type_name, Visibility::Public, param.span);
                }
            }
            if let Some(type_name) = &func.return_type {
                self.validate_visible_type(type_name, Visibility::Public, func.span);
            }
        } else if top_level && func.visibility == Visibility::Package {
            for param in &func.params {
                if let Some(type_name) = &param.type_name {
                    self.validate_visible_type(type_name, Visibility::Package, param.span);
                }
            }
            if let Some(type_name) = &func.return_type {
                self.validate_visible_type(type_name, Visibility::Package, func.span);
            }
        }

        let mut params = Vec::with_capacity(func.params.len());
        self.push_scope();
        for param in &func.params {
            self.insert_local(param.name.clone());
            params.push(Param {
                name: param.name.clone(),
                type_name: param
                    .type_name
                    .as_ref()
                    .map(|type_name| self.rewrite_type_expr(type_name, param.span)),
                span: param.span,
            });
        }

        let body = self.rewrite_value_block(&func.body);
        self.pop_scope();

        FuncDecl {
            id: func.id,
            package_item: if top_level {
                self.package_item_id(&func.name, PackageItemKind::Function)
            } else {
                None
            },
            name: if top_level {
                mangle_function_name_for_visibility(
                    &self.current_package,
                    &self.current_module,
                    &func.name,
                    func.visibility,
                    &self.entry_package,
                )
            } else {
                func.name.clone()
            },
            visibility: Visibility::Private,
            params,
            return_type: func
                .return_type
                .as_ref()
                .map(|type_name| self.rewrite_type_expr(type_name, func.span)),
            body,
            span: func.span,
        }
    }

    fn package_item_id(&self, name: &str, kind: PackageItemKind) -> Option<PackageItemId> {
        let package = self.package_graph.package_id(&self.current_package)?;
        let module = self
            .package_graph
            .module_id(package, &self.current_module)?;
        self.package_graph.item_id_in_module(module, name, kind)
    }

    fn rewrite_stmt(&mut self, statement: &Stmt) -> Stmt {
        match statement {
            Stmt::Assign(stmt) => {
                let value = self.rewrite_expr(&stmt.value);
                self.insert_local(stmt.name.clone());
                Stmt::Assign(AssignStmt {
                    id: stmt.id,
                    mutable: stmt.mutable,
                    name: stmt.name.clone(),
                    type_name: stmt
                        .type_name
                        .as_ref()
                        .map(|type_name| self.rewrite_type_expr(type_name, stmt.span)),
                    value,
                    span: stmt.span,
                })
            }
            Stmt::RecordDecl(record) => Stmt::RecordDecl(self.rewrite_record_decl(record)),
            Stmt::EnumDecl(enumeration) => Stmt::EnumDecl(self.rewrite_enum_decl(enumeration)),
            Stmt::FuncDecl(func) => Stmt::FuncDecl(self.rewrite_func_decl(func, false)),
            Stmt::If(stmt) => Stmt::If(IfStmt {
                id: stmt.id,
                condition: self.rewrite_expr(&stmt.condition),
                then_branch: self.rewrite_block(&stmt.then_branch),
                else_branch: stmt
                    .else_branch
                    .as_ref()
                    .map(|block| self.rewrite_block(block)),
                span: stmt.span,
            }),
            Stmt::While(stmt) => Stmt::While(WhileStmt {
                id: stmt.id,
                condition: self.rewrite_expr(&stmt.condition),
                body: self.rewrite_block(&stmt.body),
                span: stmt.span,
            }),
            Stmt::Expr(stmt) => Stmt::Expr(ExprStmt {
                id: stmt.id,
                expr: self.rewrite_expr(&stmt.expr),
                span: stmt.span,
            }),
        }
    }

    fn rewrite_block(&mut self, block: &Block) -> Block {
        self.push_scope();
        self.predeclare_nested_functions(&block.statements);
        let statements = block
            .statements
            .iter()
            .map(|statement| self.rewrite_stmt(statement))
            .collect();
        self.pop_scope();
        Block {
            statements,
            span: block.span,
        }
    }

    fn rewrite_value_block(&mut self, block: &ValueBlock) -> ValueBlock {
        self.push_scope();
        self.predeclare_nested_functions(&block.statements);
        let statements = block
            .statements
            .iter()
            .map(|statement| self.rewrite_stmt(statement))
            .collect();
        let expr = Box::new(self.rewrite_expr(&block.expr));
        self.pop_scope();
        ValueBlock {
            statements,
            expr,
            span: block.span,
        }
    }

    fn rewrite_expr(&mut self, expr: &Expr) -> Expr {
        match expr {
            Expr::Int(_) | Expr::Bool(_) | Expr::String(_) => expr.clone(),
            Expr::Ident(expr) => Expr::Ident(IdentExpr {
                id: expr.id,
                name: self.rewrite_value_name(&expr.name, expr.span),
                span: expr.span,
            }),
            Expr::ListLit(expr) => Expr::ListLit(ListLitExpr {
                id: expr.id,
                items: expr
                    .items
                    .iter()
                    .map(|item| self.rewrite_expr(item))
                    .collect(),
                span: expr.span,
            }),
            Expr::Index(expr) => Expr::Index(IndexExpr {
                id: expr.id,
                base: Box::new(self.rewrite_expr(&expr.base)),
                index: Box::new(self.rewrite_expr(&expr.index)),
                span: expr.span,
            }),
            Expr::RecordLit(expr) => Expr::RecordLit(RecordLitExpr {
                id: expr.id,
                type_name: self.rewrite_type_name(&expr.type_name, expr.span),
                fields: expr
                    .fields
                    .iter()
                    .map(|field| RecordFieldInit {
                        name: field.name.clone(),
                        value: self.rewrite_expr(&field.value),
                        span: field.span,
                    })
                    .collect(),
                span: expr.span,
            }),
            Expr::Field(expr) => Expr::Field(FieldExpr {
                id: expr.id,
                base: Box::new(self.rewrite_expr(&expr.base)),
                field: expr.field.clone(),
                span: expr.span,
            }),
            Expr::RecordUpdate(expr) => Expr::RecordUpdate(RecordUpdateExpr {
                id: expr.id,
                base: Box::new(self.rewrite_expr(&expr.base)),
                fields: expr
                    .fields
                    .iter()
                    .map(|field| RecordFieldInit {
                        name: field.name.clone(),
                        value: self.rewrite_expr(&field.value),
                        span: field.span,
                    })
                    .collect(),
                span: expr.span,
            }),
            Expr::Unary(expr) => Expr::Unary(UnaryExpr {
                id: expr.id,
                op: expr.op,
                expr: Box::new(self.rewrite_expr(&expr.expr)),
                span: expr.span,
            }),
            Expr::Binary(expr) => Expr::Binary(BinaryExpr {
                id: expr.id,
                op: expr.op,
                left: Box::new(self.rewrite_expr(&expr.left)),
                right: Box::new(self.rewrite_expr(&expr.right)),
                span: expr.span,
            }),
            Expr::Call(expr) => Expr::Call(CallExpr {
                id: expr.id,
                callee: Box::new(self.rewrite_expr(&expr.callee)),
                args: expr.args.iter().map(|arg| self.rewrite_expr(arg)).collect(),
                origin: expr.origin,
                span: expr.span,
            }),
            Expr::If(expr) => Expr::If(IfExpr {
                id: expr.id,
                condition: Box::new(self.rewrite_expr(&expr.condition)),
                then_branch: self.rewrite_value_block(&expr.then_branch),
                else_branch: self.rewrite_value_block(&expr.else_branch),
                span: expr.span,
            }),
            Expr::Match(expr) => Expr::Match(self.rewrite_match_expr(expr)),
            Expr::Fn(expr) => Expr::Fn(self.rewrite_fn_expr(expr)),
        }
    }

    fn rewrite_match_expr(&mut self, expr: &MatchExpr) -> MatchExpr {
        let value = Box::new(self.rewrite_expr(&expr.value));
        let arms = expr
            .arms
            .iter()
            .map(|arm| {
                self.push_scope();
                let MatchPattern::Variant(pattern) = &arm.pattern;
                if let Some(binding) = &pattern.binding {
                    self.insert_local(binding.clone());
                }
                let value = self.rewrite_expr(&arm.value);
                self.pop_scope();
                MatchArm {
                    pattern: MatchPattern::Variant(EnumVariantPattern {
                        enum_name: self.rewrite_type_name(&pattern.enum_name, pattern.span),
                        variant_name: pattern.variant_name.clone(),
                        binding: pattern.binding.clone(),
                        span: pattern.span,
                    }),
                    value,
                    span: arm.span,
                }
            })
            .collect();
        MatchExpr {
            id: expr.id,
            value,
            arms,
            span: expr.span,
        }
    }

    fn rewrite_fn_expr(&mut self, expr: &FnExpr) -> FnExpr {
        let mut params = Vec::with_capacity(expr.params.len());
        self.push_scope();
        for param in &expr.params {
            self.insert_local(param.name.clone());
            params.push(Param {
                name: param.name.clone(),
                type_name: param
                    .type_name
                    .as_ref()
                    .map(|type_name| self.rewrite_type_expr(type_name, param.span)),
                span: param.span,
            });
        }
        let body = self.rewrite_value_block(&expr.body);
        self.pop_scope();
        FnExpr {
            id: expr.id,
            params,
            return_type: expr
                .return_type
                .as_ref()
                .map(|type_name| self.rewrite_type_expr(type_name, expr.span)),
            body,
            span: expr.span,
        }
    }

    fn rewrite_type_expr(&mut self, type_expr: &TypeExpr, span: Span) -> TypeExpr {
        match type_expr {
            TypeExpr::Int => TypeExpr::Int,
            TypeExpr::Bool => TypeExpr::Bool,
            TypeExpr::String => TypeExpr::String,
            TypeExpr::Named(name) => TypeExpr::Named(self.rewrite_type_name(name, span)),
            TypeExpr::Generic(generic) => TypeExpr::Generic(GenericTypeExpr {
                name: self.rewrite_type_name(&generic.name, span),
                args: generic
                    .args
                    .iter()
                    .map(|arg| self.rewrite_type_expr(arg, span))
                    .collect(),
            }),
            TypeExpr::Function(function) => TypeExpr::Function(FunctionTypeExpr {
                params: function
                    .params
                    .iter()
                    .map(|param| self.rewrite_type_expr(param, span))
                    .collect(),
                ret: Box::new(self.rewrite_type_expr(&function.ret, span)),
            }),
        }
    }

    fn rewrite_type_name(&mut self, name: &str, span: Span) -> String {
        if let Some((alias, item)) = split_qualified_name(name) {
            return self.resolve_imported_type_item(alias, item, span);
        }
        if let Some(item) = resolve_package_item(
            &self.current_package_data.records,
            name,
            &self.current_module,
        ) {
            return mangle_record_name_for_visibility(
                &self.current_package,
                &item.module_path,
                name,
                item.visibility,
            );
        }
        if let Some(item) =
            resolve_package_item(&self.current_package_data.enums, name, &self.current_module)
        {
            return mangle_enum_name_for_visibility(
                &self.current_package,
                &item.module_path,
                name,
                item.visibility,
            );
        }
        if let Some(item) = inaccessible_package_item(&self.current_package_data.records, name) {
            self.diagnostics.push(
                Diagnostic::new(
                    "PK015",
                    format!(
                        "record `{name}` is not visible from module `{}`",
                        self.current_module
                    ),
                    span,
                )
                .with_related(
                    format!(
                        "record `{name}` is module-private to `{}`",
                        item.module_path
                    ),
                    item.span,
                )
                .with_suggestion("mark the declaration as `pkg` to share it within the package"),
            );
        }
        if let Some(item) = inaccessible_package_item(&self.current_package_data.enums, name) {
            self.diagnostics.push(
                Diagnostic::new(
                    "PK015",
                    format!(
                        "enum `{name}` is not visible from module `{}`",
                        self.current_module
                    ),
                    span,
                )
                .with_related(
                    format!("enum `{name}` is module-private to `{}`", item.module_path),
                    item.span,
                )
                .with_suggestion("mark the declaration as `pkg` to share it within the package"),
            );
        }
        name.to_string()
    }

    fn rewrite_value_name(&mut self, name: &str, span: Span) -> String {
        if name.contains("::") && is_builtin_name(name) {
            return name.to_string();
        }
        if let Some((enum_name, variant_name)) = split_variant_name(name) {
            if is_mangled_item_name(enum_name) {
                return name.to_string();
            }
            if let Some((alias, item)) = split_qualified_name(enum_name) {
                let resolved =
                    self.resolve_imported_item(alias, item, ImportedItemKind::Enum, span);
                return format!("{resolved}::{variant_name}");
            }
            if let Some(item) = resolve_package_item(
                &self.current_package_data.enums,
                enum_name,
                &self.current_module,
            ) {
                let resolved = mangle_enum_name_for_visibility(
                    &self.current_package,
                    &item.module_path,
                    enum_name,
                    item.visibility,
                );
                return format!("{resolved}::{variant_name}");
            }
        }
        if let Some((alias, item)) = split_qualified_name(name) {
            return self.resolve_imported_item(alias, item, ImportedItemKind::Function, span);
        }
        if self.lookup_local(name) {
            return name.to_string();
        }
        if let Some(item) = resolve_package_item(
            &self.current_package_data.functions,
            name,
            &self.current_module,
        ) {
            return mangle_function_name_for_visibility(
                &self.current_package,
                &item.module_path,
                name,
                item.visibility,
                &self.entry_package,
            );
        }
        if is_builtin_name(name) {
            return name.to_string();
        }
        if let Some(item) = inaccessible_package_item(&self.current_package_data.functions, name) {
            self.diagnostics.push(
                Diagnostic::new(
                    "PK015",
                    format!(
                        "function `{name}` is not visible from module `{}`",
                        self.current_module
                    ),
                    span,
                )
                .with_related(
                    format!(
                        "function `{name}` is module-private to `{}`",
                        item.module_path
                    ),
                    item.span,
                )
                .with_suggestion("mark the declaration as `pkg` to share it within the package"),
            );
        }
        name.to_string()
    }

    fn validate_visible_type(
        &mut self,
        type_expr: &TypeExpr,
        api_visibility: Visibility,
        span: Span,
    ) {
        match type_expr {
            TypeExpr::Int | TypeExpr::Bool | TypeExpr::String => {}
            TypeExpr::Named(name) => {
                if let Some((alias, item)) = split_qualified_name(name) {
                    let _ = self.resolve_imported_type_item(alias, item, span);
                    return;
                }
                if let Some(item) = resolve_package_item(
                    &self.current_package_data.records,
                    name,
                    &self.current_module,
                ) {
                    if !visibility_can_expose(item.visibility, api_visibility) {
                        let api = visibility_label(api_visibility);
                        let item_visibility = visibility_label(item.visibility);
                        self.diagnostics.push(
                            Diagnostic::new(
                                "PK012",
                                format!(
                                    "{api} API may not expose {item_visibility} record `{name}`"
                                ),
                                span,
                            )
                            .with_related(format!("record `{name}` is declared here"), item.span),
                        );
                    }
                }
                if let Some(item) = resolve_package_item(
                    &self.current_package_data.enums,
                    name,
                    &self.current_module,
                ) {
                    if !visibility_can_expose(item.visibility, api_visibility) {
                        let api = visibility_label(api_visibility);
                        let item_visibility = visibility_label(item.visibility);
                        self.diagnostics.push(
                            Diagnostic::new(
                                "PK012",
                                format!("{api} API may not expose {item_visibility} enum `{name}`"),
                                span,
                            )
                            .with_related(format!("enum `{name}` is declared here"), item.span),
                        );
                    }
                }
            }
            TypeExpr::Generic(generic) => {
                for arg in &generic.args {
                    self.validate_visible_type(arg, api_visibility, span);
                }
            }
            TypeExpr::Function(function) => {
                for param in &function.params {
                    self.validate_visible_type(param, api_visibility, span);
                }
                self.validate_visible_type(&function.ret, api_visibility, span);
            }
        }
    }

    fn resolve_imported_item(
        &mut self,
        alias: &str,
        item: &str,
        kind: ImportedItemKind,
        span: Span,
    ) -> String {
        let Some(package_path) = self.imports.get(alias) else {
            self.diagnostics.push(Diagnostic::new(
                "PK009",
                format!("unknown import alias `{alias}`"),
                span,
            ));
            return format!("{alias}::{item}");
        };
        let Some(package) = self.packages.get(package_path) else {
            self.diagnostics.push(Diagnostic::new(
                "PK010",
                format!("unknown imported package `{package_path}`"),
                span,
            ));
            return format!("{alias}::{item}");
        };
        let Some(package_id) = self.package_graph.package_id(package_path) else {
            self.diagnostics.push(Diagnostic::new(
                "PK010",
                format!("unknown imported package `{package_path}`"),
                span,
            ));
            return format!("{alias}::{item}");
        };

        match kind {
            ImportedItemKind::Enum => {
                if let Some(export) = self.package_exports.enum_by_name(package_id, item) {
                    export.mangled_name.clone()
                } else {
                    let diagnostic = missing_export_diagnostic(
                        package_path,
                        item,
                        "enum",
                        package_item_decl(&package.enums, item),
                        span,
                    );
                    self.diagnostics.push(diagnostic);
                    format!("{alias}::{item}")
                }
            }
            ImportedItemKind::Function => {
                if let Some(export) = self.package_exports.function_by_name(package_id, item) {
                    export.mangled_name.clone()
                } else {
                    let diagnostic = missing_export_diagnostic(
                        package_path,
                        item,
                        "function",
                        package_item_decl(&package.functions, item),
                        span,
                    );
                    self.diagnostics.push(diagnostic);
                    format!("{alias}::{item}")
                }
            }
        }
    }

    fn resolve_imported_type_item(&mut self, alias: &str, item: &str, span: Span) -> String {
        let Some(package_path) = self.imports.get(alias) else {
            self.diagnostics.push(Diagnostic::new(
                "PK009",
                format!("unknown import alias `{alias}`"),
                span,
            ));
            return format!("{alias}::{item}");
        };
        let Some(package) = self.packages.get(package_path) else {
            self.diagnostics.push(Diagnostic::new(
                "PK010",
                format!("unknown imported package `{package_path}`"),
                span,
            ));
            return format!("{alias}::{item}");
        };
        let Some(package_id) = self.package_graph.package_id(package_path) else {
            self.diagnostics.push(Diagnostic::new(
                "PK010",
                format!("unknown imported package `{package_path}`"),
                span,
            ));
            return format!("{alias}::{item}");
        };

        if let Some(export) = self.package_exports.record_by_name(package_id, item) {
            return export.mangled_name.clone();
        }
        if package_item_decl(&package.records, item).is_some() {
            self.diagnostics.push(missing_export_diagnostic(
                package_path,
                item,
                "record",
                package_item_decl(&package.records, item),
                span,
            ));
            return format!("{alias}::{item}");
        }
        if let Some(export) = self.package_exports.enum_by_name(package_id, item) {
            return export.mangled_name.clone();
        }
        self.diagnostics.push(missing_export_diagnostic(
            package_path,
            item,
            "enum",
            package_item_decl(&package.enums, item),
            span,
        ));
        format!("{alias}::{item}")
    }

    fn predeclare_nested_functions(&mut self, statements: &[Stmt]) {
        for statement in statements {
            if let Stmt::FuncDecl(func) = statement {
                self.insert_local(func.name.clone());
            }
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn insert_local(&mut self, name: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name);
        }
    }

    fn lookup_local(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains(name))
    }
}

#[derive(Clone, Copy)]
enum ImportedItemKind {
    Enum,
    Function,
}

fn collect_package_data(
    package_path: &str,
    files: Vec<ParsedFile>,
    diagnostics: &mut Vec<Diagnostic>,
) -> PackageData {
    let mut records: HashMap<String, Vec<PackageItemDecl>> = HashMap::new();
    let mut enums: HashMap<String, Vec<PackageItemDecl>> = HashMap::new();
    let mut functions: HashMap<String, Vec<PackageItemDecl>> = HashMap::new();

    for file in &files {
        for statement in &file.program.statements {
            match statement {
                Stmt::RecordDecl(record) => {
                    insert_package_item_decl(
                        &mut records,
                        PackageItemDeclInput {
                            name: &record.name,
                            visibility: record.visibility,
                            module_path: &file.module_path,
                            span: record.span,
                            kind: PackageItemKind::Record,
                            package_path,
                        },
                        diagnostics,
                    );
                }
                Stmt::EnumDecl(enumeration) => {
                    insert_package_item_decl(
                        &mut enums,
                        PackageItemDeclInput {
                            name: &enumeration.name,
                            visibility: enumeration.visibility,
                            module_path: &file.module_path,
                            span: enumeration.span,
                            kind: PackageItemKind::Enum,
                            package_path,
                        },
                        diagnostics,
                    );
                }
                Stmt::FuncDecl(func) => {
                    insert_package_item_decl(
                        &mut functions,
                        PackageItemDeclInput {
                            name: &func.name,
                            visibility: func.visibility,
                            module_path: &file.module_path,
                            span: func.span,
                            kind: PackageItemKind::Function,
                            package_path,
                        },
                        diagnostics,
                    );
                }
                _ => {}
            }
        }
    }

    PackageData {
        files,
        records,
        enums,
        functions,
    }
}

fn package_import_paths(package: &PackageData) -> Vec<String> {
    let mut paths: Vec<String> = package
        .files
        .iter()
        .flat_map(|file| {
            file.program
                .imports
                .iter()
                .map(|import| import.path.clone())
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    paths.sort();
    paths
}

const INTERFACE_STUB_MODULE: &str = "<interface>";

fn stub_package_data_from_interface(
    interface: &PackageInterface,
    interfaces: &PackageInterfaceGraph,
    symbols: &SymbolTable,
    diagnostics: &mut Vec<Diagnostic>,
) -> PackageData {
    let mut statements = Vec::new();
    for record in &interface.records {
        statements.push(Stmt::RecordDecl(RecordDecl {
            id: StmtId::new(0),
            name: record.name.clone(),
            package_item: None,
            visibility: Visibility::Public,
            fields: record
                .fields
                .iter()
                .map(|field| RecordFieldDecl {
                    name: field.name.clone(),
                    type_name: type_expr_from_type_info(
                        &field.ty,
                        symbols,
                        diagnostics,
                        field.span,
                    ),
                    span: field.span,
                })
                .collect(),
            span: record.span,
        }));
    }
    for enumeration in &interface.enums {
        statements.push(Stmt::EnumDecl(EnumDecl {
            id: StmtId::new(0),
            name: enumeration.name.clone(),
            package_item: None,
            visibility: Visibility::Public,
            type_params: enumeration.type_params.clone(),
            variants: enumeration
                .variants
                .iter()
                .map(|variant| EnumVariantDecl {
                    name: variant.name.clone(),
                    payload: variant.payload.as_ref().map(|payload| {
                        type_expr_from_type_info(payload, symbols, diagnostics, variant.span)
                    }),
                    span: variant.span,
                })
                .collect(),
            span: enumeration.span,
        }));
    }
    for function in &interface.functions {
        let return_type =
            type_expr_from_type_info(&function.ret, symbols, diagnostics, function.span);
        let body_expr = {
            let mut context = DummyExprContext {
                symbols,
                interfaces,
                diagnostics,
            };
            dummy_expr_for_type_info(&function.ret, &mut context, function.span, 0)
        };
        statements.push(Stmt::FuncDecl(FuncDecl {
            id: StmtId::new(0),
            name: function.name.clone(),
            package_item: None,
            visibility: Visibility::Public,
            params: function
                .params
                .iter()
                .map(|param| Param {
                    name: param.name.clone(),
                    type_name: Some(type_expr_from_type_info(
                        &param.ty,
                        symbols,
                        diagnostics,
                        param.span,
                    )),
                    span: param.span,
                })
                .collect(),
            return_type: Some(return_type),
            body: ValueBlock {
                statements: Vec::new(),
                expr: Box::new(body_expr),
                span: function.span,
            },
            span: function.span,
        }));
    }

    let files = vec![ParsedFile {
        program: Program {
            package: Some(PackageDecl {
                path: interface.path.clone(),
                span: Span::default(),
            }),
            imports: Vec::new(),
            statements,
        },
        module_path: INTERFACE_STUB_MODULE.to_string(),
    }];
    collect_package_data(&interface.path, files, diagnostics)
}

fn type_expr_from_type_info(
    ty: &TypeInfo,
    symbols: &SymbolTable,
    diagnostics: &mut Vec<Diagnostic>,
    span: Span,
) -> TypeExpr {
    match ty {
        TypeInfo::Int => TypeExpr::Int,
        TypeInfo::Bool => TypeExpr::Bool,
        TypeInfo::String => TypeExpr::String,
        TypeInfo::GenericParam(symbol) => {
            TypeExpr::Named(symbol_name(*symbol, symbols, diagnostics, span))
        }
        TypeInfo::Record(symbol) | TypeInfo::PackageRecord { symbol, .. } => {
            TypeExpr::Named(symbol_name(*symbol, symbols, diagnostics, span))
        }
        TypeInfo::Enum { symbol, args } | TypeInfo::PackageEnum { symbol, args, .. } => {
            named_or_generic_type_expr(*symbol, args, symbols, diagnostics, span)
        }
        TypeInfo::List(item) => TypeExpr::Generic(GenericTypeExpr {
            name: "List".to_string(),
            args: vec![type_expr_from_type_info(item, symbols, diagnostics, span)],
        }),
        TypeInfo::Map(key, value) => TypeExpr::Generic(GenericTypeExpr {
            name: "Map".to_string(),
            args: vec![
                type_expr_from_type_info(key, symbols, diagnostics, span),
                type_expr_from_type_info(value, symbols, diagnostics, span),
            ],
        }),
        TypeInfo::Option(item) => TypeExpr::Generic(GenericTypeExpr {
            name: "Option".to_string(),
            args: vec![type_expr_from_type_info(item, symbols, diagnostics, span)],
        }),
        TypeInfo::Result(ok, err) => TypeExpr::Generic(GenericTypeExpr {
            name: "Result".to_string(),
            args: vec![
                type_expr_from_type_info(ok, symbols, diagnostics, span),
                type_expr_from_type_info(err, symbols, diagnostics, span),
            ],
        }),
        TypeInfo::Function(function) => TypeExpr::Function(FunctionTypeExpr {
            params: function
                .params
                .iter()
                .map(|param| type_expr_from_type_info(param, symbols, diagnostics, span))
                .collect(),
            ret: Box::new(type_expr_from_type_info(
                &function.ret,
                symbols,
                diagnostics,
                span,
            )),
        }),
        TypeInfo::EnumConstructor { .. }
        | TypeInfo::Builtin(_)
        | TypeInfo::Unknown
        | TypeInfo::Error => {
            diagnostics.push(
                Diagnostic::new(
                    "PK018",
                    "package interface contains a value-only or unresolved type in a signature",
                    span,
                )
                .with_suggestion("regenerate the package interface"),
            );
            TypeExpr::Named("<invalid-interface-type>".to_string())
        }
    }
}

fn named_or_generic_type_expr(
    symbol: Symbol,
    args: &[TypeInfo],
    symbols: &SymbolTable,
    diagnostics: &mut Vec<Diagnostic>,
    span: Span,
) -> TypeExpr {
    let name = symbol_name(symbol, symbols, diagnostics, span);
    if args.is_empty() {
        TypeExpr::Named(name)
    } else {
        TypeExpr::Generic(GenericTypeExpr {
            name,
            args: args
                .iter()
                .map(|arg| type_expr_from_type_info(arg, symbols, diagnostics, span))
                .collect(),
        })
    }
}

struct DummyExprContext<'a, 'd> {
    symbols: &'a SymbolTable,
    interfaces: &'a PackageInterfaceGraph,
    diagnostics: &'d mut Vec<Diagnostic>,
}

fn dummy_expr_for_type_info(
    ty: &TypeInfo,
    context: &mut DummyExprContext<'_, '_>,
    span: Span,
    depth: usize,
) -> Expr {
    if depth > 8 {
        context.diagnostics.push(
            Diagnostic::new(
                "PK018",
                "package interface stub return type is recursively nested too deeply",
                span,
            )
            .with_suggestion("use a less recursive public signature or check against source"),
        );
        return int_expr(span);
    }

    match ty {
        TypeInfo::Int => int_expr(span),
        TypeInfo::Bool => Expr::Bool(BoolExpr {
            id: ExprId::new(0),
            value: false,
            span,
        }),
        TypeInfo::String => Expr::String(StringExpr {
            id: ExprId::new(0),
            value: String::new(),
            span,
        }),
        TypeInfo::List(_) => Expr::ListLit(ListLitExpr {
            id: ExprId::new(0),
            items: Vec::new(),
            span,
        }),
        TypeInfo::Map(_, _) => call_expr("Map.empty", Vec::new(), span),
        TypeInfo::Option(_) => ident_expr("Option::None", span),
        TypeInfo::Result(ok, _) => call_expr(
            "Result::Ok",
            vec![dummy_expr_for_type_info(ok, context, span, depth + 1)],
            span,
        ),
        TypeInfo::PackageRecord { symbol, item } => {
            let type_name = symbol_name(*symbol, context.symbols, context.diagnostics, span);
            let fields = context
                .interfaces
                .record(*item)
                .map(|record| {
                    record
                        .fields
                        .iter()
                        .map(|field| RecordFieldInit {
                            name: field.name.clone(),
                            value: dummy_expr_for_type_info(
                                &field.ty,
                                context,
                                field.span,
                                depth + 1,
                            ),
                            span: field.span,
                        })
                        .collect()
                })
                .unwrap_or_default();
            Expr::RecordLit(RecordLitExpr {
                id: ExprId::new(0),
                type_name,
                fields,
                span,
            })
        }
        TypeInfo::PackageEnum { symbol, item, args } => {
            dummy_package_enum_expr(*symbol, *item, args, context, span, depth)
        }
        TypeInfo::Function(function) => dummy_function_expr(function, context, span, depth),
        TypeInfo::GenericParam(_)
        | TypeInfo::Record(_)
        | TypeInfo::Enum { .. }
        | TypeInfo::EnumConstructor { .. }
        | TypeInfo::Builtin(_)
        | TypeInfo::Unknown
        | TypeInfo::Error => {
            context.diagnostics.push(
                Diagnostic::new(
                    "PK018",
                    "package interface stub cannot synthesize a value for this return type",
                    span,
                )
                .with_suggestion("check this package against source instead"),
            );
            int_expr(span)
        }
    }
}

fn dummy_package_enum_expr(
    enum_symbol: Symbol,
    item: PackageItemId,
    args: &[TypeInfo],
    context: &mut DummyExprContext<'_, '_>,
    span: Span,
    depth: usize,
) -> Expr {
    let enum_name = symbol_name(enum_symbol, context.symbols, context.diagnostics, span);
    let Some(enumeration) = interface_enum(context.interfaces, item) else {
        context.diagnostics.push(
            Diagnostic::new(
                "PK018",
                format!("package interface is missing enum item {:?}", item),
                span,
            )
            .with_suggestion("regenerate the package interface"),
        );
        return int_expr(span);
    };
    let Some(variant) = enumeration
        .variants
        .iter()
        .find(|variant| variant.payload.is_none())
        .or_else(|| enumeration.variants.first())
    else {
        context.diagnostics.push(
            Diagnostic::new(
                "PK018",
                format!(
                    "package interface enum `{}` has no variants",
                    enumeration.name
                ),
                span,
            )
            .with_suggestion("add at least one enum variant"),
        );
        return int_expr(span);
    };

    let variant_name = format!("{enum_name}::{}", variant.name);
    match &variant.payload {
        None => ident_expr(&variant_name, variant.span),
        Some(payload) => {
            let payload = substitute_interface_type_params(
                payload,
                &enumeration.type_params,
                args,
                context.symbols,
            );
            call_expr(
                &variant_name,
                vec![dummy_expr_for_type_info(
                    &payload,
                    context,
                    variant.span,
                    depth + 1,
                )],
                variant.span,
            )
        }
    }
}

fn dummy_function_expr(
    function: &FunctionTypeInfo,
    context: &mut DummyExprContext<'_, '_>,
    span: Span,
    depth: usize,
) -> Expr {
    let params = function
        .params
        .iter()
        .enumerate()
        .map(|(index, param)| Param {
            name: format!("__stub_arg_{index}"),
            type_name: Some(type_expr_from_type_info(
                param,
                context.symbols,
                context.diagnostics,
                span,
            )),
            span,
        })
        .collect();
    let ret = type_expr_from_type_info(&function.ret, context.symbols, context.diagnostics, span);
    let expr = dummy_expr_for_type_info(&function.ret, context, span, depth + 1);
    Expr::Fn(FnExpr {
        id: ExprId::new(0),
        params,
        return_type: Some(ret),
        body: ValueBlock {
            statements: Vec::new(),
            expr: Box::new(expr),
            span,
        },
        span,
    })
}

fn substitute_interface_type_params(
    ty: &TypeInfo,
    params: &[String],
    type_args: &[TypeInfo],
    symbols: &SymbolTable,
) -> TypeInfo {
    match ty {
        TypeInfo::GenericParam(symbol) => {
            if symbol.as_u32() < symbols.len() as u32 {
                let name = symbols.resolve(*symbol);
                if let Some(index) = params.iter().position(|param| param == name)
                    && let Some(arg) = type_args.get(index)
                {
                    return arg.clone();
                }
            }
            ty.clone()
        }
        TypeInfo::Enum {
            symbol,
            args: enum_args,
        } => TypeInfo::Enum {
            symbol: *symbol,
            args: enum_args
                .iter()
                .map(|arg| substitute_interface_type_params(arg, params, type_args, symbols))
                .collect(),
        },
        TypeInfo::PackageEnum {
            symbol,
            item,
            args: enum_args,
        } => TypeInfo::PackageEnum {
            symbol: *symbol,
            item: *item,
            args: enum_args
                .iter()
                .map(|arg| substitute_interface_type_params(arg, params, type_args, symbols))
                .collect(),
        },
        TypeInfo::List(item) => TypeInfo::List(Box::new(substitute_interface_type_params(
            item, params, type_args, symbols,
        ))),
        TypeInfo::Map(key, value) => TypeInfo::Map(
            Box::new(substitute_interface_type_params(
                key, params, type_args, symbols,
            )),
            Box::new(substitute_interface_type_params(
                value, params, type_args, symbols,
            )),
        ),
        TypeInfo::Option(item) => TypeInfo::Option(Box::new(substitute_interface_type_params(
            item, params, type_args, symbols,
        ))),
        TypeInfo::Result(ok, err) => TypeInfo::Result(
            Box::new(substitute_interface_type_params(
                ok, params, type_args, symbols,
            )),
            Box::new(substitute_interface_type_params(
                err, params, type_args, symbols,
            )),
        ),
        TypeInfo::Function(function) => TypeInfo::Function(FunctionTypeInfo {
            params: function
                .params
                .iter()
                .map(|param| substitute_interface_type_params(param, params, type_args, symbols))
                .collect(),
            ret: Box::new(substitute_interface_type_params(
                &function.ret,
                params,
                type_args,
                symbols,
            )),
        }),
        other => other.clone(),
    }
}

fn symbol_name(
    symbol: Symbol,
    symbols: &SymbolTable,
    diagnostics: &mut Vec<Diagnostic>,
    span: Span,
) -> String {
    if symbol.as_u32() < symbols.len() as u32 {
        symbols.resolve(symbol).to_string()
    } else {
        diagnostics.push(
            Diagnostic::new(
                "PK018",
                format!("package interface references unknown symbol {:?}", symbol),
                span,
            )
            .with_suggestion("read the interface with the same symbol table used for checking"),
        );
        "<invalid-interface-symbol>".to_string()
    }
}

fn interface_enum(
    interfaces: &PackageInterfaceGraph,
    item: PackageItemId,
) -> Option<&crate::interface::PackageInterfaceEnum> {
    interfaces
        .packages
        .iter()
        .flat_map(|package| package.enums.iter())
        .find(|enumeration| enumeration.item == item)
}

fn ident_expr(name: &str, span: Span) -> Expr {
    Expr::Ident(IdentExpr {
        id: ExprId::new(0),
        name: name.to_string(),
        span,
    })
}

fn call_expr(name: &str, args: Vec<Expr>, span: Span) -> Expr {
    Expr::Call(CallExpr {
        id: ExprId::new(0),
        callee: Box::new(ident_expr(name, span)),
        args,
        origin: CallOrigin::Ordinary,
        span,
    })
}

fn int_expr(span: Span) -> Expr {
    Expr::Int(IntExpr {
        id: ExprId::new(0),
        value: 0,
        span,
    })
}

fn interface_item_id(
    package_interface: Option<&PackageInterface>,
    name: &str,
    kind: PackageItemKind,
    visibility: Visibility,
) -> Option<PackageItemId> {
    if visibility != Visibility::Public {
        return None;
    }
    let interface = package_interface?;
    match kind {
        PackageItemKind::Record => interface
            .records
            .iter()
            .find(|record| record.name == name)
            .map(|record| record.item),
        PackageItemKind::Enum => interface
            .enums
            .iter()
            .find(|enumeration| enumeration.name == name)
            .map(|enumeration| enumeration.item),
        PackageItemKind::Function => interface
            .functions
            .iter()
            .find(|function| function.name == name)
            .map(|function| function.item),
    }
}

fn allocate_package_item_id(items: &mut Vec<Option<PackageItemInfo>>) -> PackageItemId {
    let id = PackageItemId::new(items.len() as u32);
    items.push(None);
    id
}

fn insert_package_graph_item(
    items: &mut Vec<Option<PackageItemInfo>>,
    item: PackageItemInfo,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let index = item.id.as_u32() as usize;
    if index >= items.len() {
        items.resize(index + 1, None);
    }
    if items[index].is_some() {
        diagnostics.push(
            Diagnostic::new(
                "PK017",
                format!("duplicate package interface item identity {:?}", item.id),
                item.span,
            )
            .with_suggestion("regenerate the package interface"),
        );
    } else {
        items[index] = Some(item);
    }
}

fn insert_package_item_decl(
    items: &mut HashMap<String, Vec<PackageItemDecl>>,
    decl: PackageItemDeclInput<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let existing = items.entry(decl.name.to_string()).or_default();
    let duplicate = existing.iter().find(|item| {
        item.module_path == decl.module_path
            || (decl.visibility != Visibility::Private && item.visibility != Visibility::Private)
    });

    if let Some(previous) = duplicate {
        let kind_name = match decl.kind {
            PackageItemKind::Record => "record",
            PackageItemKind::Enum => "enum",
            PackageItemKind::Function => "function",
        };
        diagnostics.push(
            Diagnostic::new(
                "PK013",
                format!(
                    "duplicate top-level {kind_name} `{}` in package `{}`",
                    decl.name, decl.package_path
                ),
                decl.span,
            )
            .with_related(
                format!("previous `{}` declaration is here", decl.name),
                previous.span,
            ),
        );
    }

    existing.push(PackageItemDecl {
        visibility: decl.visibility,
        module_path: decl.module_path.to_string(),
        span: decl.span,
    });
}

fn resolve_package_item<'a>(
    items: &'a HashMap<String, Vec<PackageItemDecl>>,
    name: &str,
    current_module: &str,
) -> Option<&'a PackageItemDecl> {
    let candidates = items.get(name)?;
    candidates
        .iter()
        .find(|item| item.module_path == current_module)
        .or_else(|| {
            candidates.iter().find(|item| {
                item.visibility == Visibility::Package || item.visibility == Visibility::Public
            })
        })
}

fn inaccessible_package_item<'a>(
    items: &'a HashMap<String, Vec<PackageItemDecl>>,
    name: &str,
) -> Option<&'a PackageItemDecl> {
    items
        .get(name)?
        .iter()
        .find(|item| item.visibility == Visibility::Private)
}

fn package_item_decl<'a>(
    items: &'a HashMap<String, Vec<PackageItemDecl>>,
    name: &str,
) -> Option<&'a PackageItemDecl> {
    items.get(name)?.first()
}

fn missing_export_diagnostic(
    package_path: &str,
    item: &str,
    kind: &str,
    declaration: Option<&PackageItemDecl>,
    span: Span,
) -> Diagnostic {
    let mut diagnostic = Diagnostic::new(
        "PK010",
        format!("package `{package_path}` does not export {kind} `{item}`"),
        span,
    );
    if let Some(declaration) = declaration {
        diagnostic = diagnostic
            .with_related(
                format!("{kind} `{item}` is declared here but is not public"),
                declaration.span,
            )
            .with_suggestion(format!(
                "mark the {kind} declaration as `pub` to export it from the package"
            ));
    }
    diagnostic
}

fn visibility_can_expose(item_visibility: Visibility, api_visibility: Visibility) -> bool {
    match api_visibility {
        Visibility::Public => item_visibility == Visibility::Public,
        Visibility::Package => {
            item_visibility == Visibility::Package || item_visibility == Visibility::Public
        }
        Visibility::Private => true,
    }
}

fn visibility_label(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Private => "module-private",
        Visibility::Package => "package-visible",
        Visibility::Public => "public",
    }
}

fn infer_source_root(entry_file: &Path, package_path: &str) -> Result<PathBuf, Diagnostic> {
    let package_segments = split_package_path(package_path);
    let Some(dir) = entry_file.parent() else {
        return Err(Diagnostic::new(
            "PK003",
            "entry file must live inside a package directory",
            Span::default(),
        ));
    };

    let dir_segments: Vec<String> = dir
        .iter()
        .map(|segment| segment.to_string_lossy().into_owned())
        .collect();
    if dir_segments.len() < package_segments.len()
        || dir_segments[dir_segments.len() - package_segments.len()..] != package_segments
    {
        return Err(Diagnostic::new(
            "PK003",
            format!(
                "package path `{package_path}` must match the directory layout of {}",
                entry_file.display()
            ),
            Span::default(),
        ));
    }

    let mut root = dir.to_path_buf();
    for _ in 0..package_segments.len() {
        root = root.parent().map(Path::to_path_buf).ok_or_else(|| {
            Diagnostic::new(
                "PK003",
                format!(
                    "package path `{package_path}` must match the directory layout of {}",
                    entry_file.display()
                ),
                Span::default(),
            )
        })?;
    }
    Ok(root)
}

fn discover_manifest(entry_file: &Path) -> Result<Option<ProjectManifest>, Vec<Diagnostic>> {
    let mut current = entry_file.parent();
    while let Some(dir) = current {
        let manifest_path = dir.join("muga.toml");
        if manifest_path.is_file() {
            return parse_manifest(&manifest_path).map(Some);
        }
        current = dir.parent();
    }
    Ok(None)
}

fn parse_manifest(path: &Path) -> Result<ProjectManifest, Vec<Diagnostic>> {
    let source = fs::read_to_string(path).map_err(|error| {
        vec![Diagnostic::new(
            "PK002",
            format!("failed to read {}: {error}", path.display()),
            Span::default(),
        )]
    })?;

    let mut in_package = false;
    let mut name = None;
    let mut source_dir = "src".to_string();

    for raw_line in source.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let Some(value) = parse_manifest_string(value.trim()) else {
            return Err(vec![Diagnostic::new(
                "PK014",
                format!(
                    "manifest field `{key}` in {} must be a string",
                    path.display()
                ),
                Span::default(),
            )]);
        };
        match key {
            "name" => name = Some(value),
            "source" => source_dir = value,
            _ => {}
        }
    }

    let Some(name) = name else {
        return Err(vec![Diagnostic::new(
            "PK014",
            format!("manifest {} must define [package] name", path.display()),
            Span::default(),
        )]);
    };
    if !is_valid_package_path(&name) {
        return Err(vec![Diagnostic::new(
            "PK014",
            format!("manifest package name `{name}` is not a valid package path"),
            Span::default(),
        )]);
    }

    let root = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let source_root = if Path::new(&source_dir).is_absolute() {
        PathBuf::from(source_dir)
    } else {
        root.join(source_dir)
    };

    Ok(ProjectManifest { source_root, name })
}

fn parse_manifest_string(value: &str) -> Option<String> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(ToString::to_string)
}

fn infer_manifest_package_path(
    entry_file: &Path,
    manifest: &ProjectManifest,
) -> Result<String, Vec<Diagnostic>> {
    let Some(package_dir) = entry_file.parent() else {
        return Err(vec![Diagnostic::new(
            "PK003",
            "entry file must live inside a package directory",
            Span::default(),
        )]);
    };
    let relative = package_dir
        .strip_prefix(&manifest.source_root)
        .map_err(|_| {
            vec![Diagnostic::new(
                "PK003",
                format!(
                    "entry file {} must live under manifest source root {}",
                    entry_file.display(),
                    manifest.source_root.display()
                ),
                Span::default(),
            )]
        })?;

    let mut segments = vec![manifest.name.clone()];
    for component in relative {
        let Some(segment) = component.to_str() else {
            return Err(vec![Diagnostic::new(
                "PK003",
                format!(
                    "package path for {} contains non-UTF-8 segment",
                    entry_file.display()
                ),
                Span::default(),
            )]);
        };
        if segment.is_empty() {
            continue;
        }
        if !is_valid_package_segment(segment) {
            return Err(vec![Diagnostic::new(
                "PK003",
                format!(
                    "directory segment `{segment}` in {} is not a valid package segment",
                    entry_file.display()
                ),
                Span::default(),
            )]);
        }
        segments.push(segment.to_string());
    }

    Ok(segments.join("::"))
}

fn is_valid_package_path(path: &str) -> bool {
    !path.is_empty() && path.split("::").all(is_valid_package_segment)
}

fn is_valid_package_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn split_package_path(path: &str) -> Vec<String> {
    path.split("::").map(ToString::to_string).collect()
}

fn module_path_for_file(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

fn split_qualified_name(name: &str) -> Option<(&str, &str)> {
    let mut parts = name.split("::");
    let first = parts.next()?;
    let second = parts.next()?;
    if parts.next().is_some() {
        None
    } else {
        Some((first, second))
    }
}

fn split_variant_name(name: &str) -> Option<(&str, &str)> {
    name.rsplit_once("::")
}

fn file_import_aliases(
    imports: &[ImportDecl],
    diagnostics: &mut Vec<Diagnostic>,
) -> HashMap<String, String> {
    let mut aliases = HashMap::new();
    for import in imports {
        if let Some((previous, previous_span)) = aliases.get(&import.alias) {
            diagnostics.push(
                Diagnostic::new(
                    "PK007",
                    format!(
                        "duplicate import alias `{}` for `{}` and `{}`",
                        import.alias, previous, import.path
                    ),
                    import.span,
                )
                .with_related("previous import using this alias is here", *previous_span)
                .with_suggestion("use `as` to give one import a distinct local alias"),
            );
        } else {
            aliases.insert(import.alias.clone(), (import.path.clone(), import.span));
        }
    }
    aliases
        .into_iter()
        .map(|(alias, (path, _))| (alias, path))
        .collect()
}

fn mangle_function_name(package_path: &str, name: &str, entry_package: &str) -> String {
    if package_path == entry_package && name == "main" {
        "main".to_string()
    } else {
        format!("__muga_pkg__{}__{}", package_path.replace("::", "__"), name)
    }
}

fn mangle_function_name_for_visibility(
    package_path: &str,
    module_path: &str,
    name: &str,
    visibility: Visibility,
    entry_package: &str,
) -> String {
    if package_path == entry_package && name == "main" {
        return "main".to_string();
    }
    match visibility {
        Visibility::Private => mangle_module_item_name(package_path, module_path, name),
        Visibility::Package | Visibility::Public => {
            mangle_function_name(package_path, name, entry_package)
        }
    }
}

fn mangle_record_name(package_path: &str, name: &str) -> String {
    format!("__muga_pkg__{}__{}", package_path.replace("::", "__"), name)
}

fn mangle_record_name_for_visibility(
    package_path: &str,
    module_path: &str,
    name: &str,
    visibility: Visibility,
) -> String {
    match visibility {
        Visibility::Private => mangle_module_item_name(package_path, module_path, name),
        Visibility::Package | Visibility::Public => mangle_record_name(package_path, name),
    }
}

fn mangle_enum_name(package_path: &str, name: &str) -> String {
    format!("__muga_pkg__{}__{}", package_path.replace("::", "__"), name)
}

fn mangle_enum_name_for_visibility(
    package_path: &str,
    module_path: &str,
    name: &str,
    visibility: Visibility,
) -> String {
    match visibility {
        Visibility::Private => mangle_module_item_name(package_path, module_path, name),
        Visibility::Package | Visibility::Public => mangle_enum_name(package_path, name),
    }
}

fn mangle_module_item_name(package_path: &str, module_path: &str, name: &str) -> String {
    format!(
        "__muga_mod__{}__{}__{}",
        package_path.replace("::", "__"),
        sanitize_mangle_segment(module_path),
        name
    )
}

fn sanitize_mangle_segment(segment: &str) -> String {
    segment
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn is_builtin_name(name: &str) -> bool {
    crate::prelude::is_builtin_name(name)
}

fn is_mangled_item_name(name: &str) -> bool {
    name.starts_with("__muga_pkg__") || name.starts_with("__muga_mod__")
}
