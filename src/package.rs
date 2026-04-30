use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::identity::{ModuleId, PackageId, PackageItemId};
use crate::span::Span;

pub fn load_program_from_entry(path: &Path) -> Result<Program, Vec<Diagnostic>> {
    Ok(load_from_entry(path)?.program)
}

pub fn load_from_entry(path: &Path) -> Result<LoadedProgram, Vec<Diagnostic>> {
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
    if entry_program.package.is_none() {
        return Ok(LoadedProgram {
            program: entry_program,
            package_graph: PackageSymbolGraph::default(),
        });
    }

    let mut loader = PackageLoader::new(path.to_path_buf(), entry_program, manifest);
    loader.load_and_flatten()
}

#[derive(Clone, Debug)]
pub struct LoadedProgram {
    pub program: Program,
    pub package_graph: PackageSymbolGraph,
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
    Function,
}

struct ParsedFile {
    program: Program,
    module_path: String,
}

#[derive(Clone, Debug)]
struct ProjectManifest {
    source_root: PathBuf,
    name: String,
}

struct PackageData {
    files: Vec<ParsedFile>,
    records: HashMap<String, Vec<PackageItemDecl>>,
    functions: HashMap<String, Vec<PackageItemDecl>>,
}

#[derive(Clone, Debug)]
struct PackageItemDecl {
    visibility: Visibility,
    module_path: String,
    span: Span,
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
        if self.manifest.is_none() {
            match infer_source_root(&self.entry_file, &self.entry_package) {
                Ok(source_root) => self.source_root = source_root,
                Err(diagnostic) => {
                    self.diagnostics.push(diagnostic);
                    return Err(std::mem::take(&mut self.diagnostics));
                }
            }
        }

        self.load_package(self.entry_package.clone());

        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }

        let package_paths = self.sorted_package_paths();

        let mut statements = Vec::new();
        for package_path in &package_paths {
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
                    scopes: Vec::new(),
                };
                for statement in &file.program.statements {
                    statements.push(rewriter.rewrite_top_level_stmt(statement));
                }
            }
        }

        if self.diagnostics.is_empty() {
            let package_graph = self.build_symbol_graph(&package_paths);
            let mut program = Program {
                package: None,
                imports: Vec::new(),
                statements,
            };
            renumber_node_ids(&mut program);
            Ok(LoadedProgram {
                program,
                package_graph,
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

        let mut records: HashMap<String, Vec<PackageItemDecl>> = HashMap::new();
        let mut functions: HashMap<String, Vec<PackageItemDecl>> = HashMap::new();

        for file in &files {
            for statement in &file.program.statements {
                match statement {
                    Stmt::RecordDecl(record) => {
                        insert_package_item_decl(
                            &mut records,
                            &record.name,
                            record.visibility,
                            &file.module_path,
                            record.span,
                            PackageItemKind::Record,
                            package_path.as_str(),
                            &mut self.diagnostics,
                        );
                    }
                    Stmt::FuncDecl(func) => {
                        insert_package_item_decl(
                            &mut functions,
                            &func.name,
                            func.visibility,
                            &file.module_path,
                            func.span,
                            PackageItemKind::Function,
                            package_path.as_str(),
                            &mut self.diagnostics,
                        );
                    }
                    _ => {}
                }
            }
        }

        self.packages.insert(
            package_path.clone(),
            PackageData {
                files,
                records,
                functions,
            },
        );
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
}

struct PackageRewriter<'a> {
    diagnostics: &'a mut Vec<Diagnostic>,
    current_package: String,
    current_module: String,
    entry_package: String,
    imports: HashMap<String, String>,
    current_package_data: &'a PackageData,
    packages: &'a HashMap<String, PackageData>,
    scopes: Vec<HashSet<String>>,
}

impl<'a> PackageRewriter<'a> {
    fn rewrite_top_level_stmt(&mut self, statement: &Stmt) -> Stmt {
        match statement {
            Stmt::RecordDecl(record) => Stmt::RecordDecl(self.rewrite_record_decl(record)),
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

    fn rewrite_func_decl(&mut self, func: &FuncDecl, top_level: bool) -> FuncDecl {
        if top_level && func.visibility == Visibility::Public {
            let has_full_signature = func.params.iter().all(|param| param.type_name.is_some())
                && func.return_type.is_some();
            if !has_full_signature {
                self.diagnostics.push(Diagnostic::new(
                    "PK011",
                    "public functions must annotate every parameter and the return type",
                    func.span,
                ));
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

    fn rewrite_stmt(&mut self, statement: &Stmt) -> Stmt {
        match statement {
            Stmt::Assign(stmt) => {
                let value = self.rewrite_expr(&stmt.value);
                self.insert_local(stmt.name.clone());
                Stmt::Assign(AssignStmt {
                    id: stmt.id,
                    mutable: stmt.mutable,
                    name: stmt.name.clone(),
                    value,
                    span: stmt.span,
                })
            }
            Stmt::RecordDecl(record) => Stmt::RecordDecl(self.rewrite_record_decl(record)),
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
            Expr::Fn(expr) => Expr::Fn(self.rewrite_fn_expr(expr)),
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
            return self.resolve_imported_item(alias, item, ImportedItemKind::Record, span);
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
        name.to_string()
    }

    fn rewrite_value_name(&mut self, name: &str, span: Span) -> String {
        if let Some((alias, item)) = split_qualified_name(name) {
            return self.resolve_imported_item(alias, item, ImportedItemKind::Function, span);
        }
        if self.lookup_local(name) || is_builtin_name(name) {
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
                    let _ = self.resolve_imported_item(alias, item, ImportedItemKind::Record, span);
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

        match kind {
            ImportedItemKind::Record => {
                if package_item_is_public(&package.records, item) {
                    mangle_record_name(package_path, item)
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        "PK010",
                        format!("package `{package_path}` does not export record `{item}`"),
                        span,
                    ));
                    format!("{alias}::{item}")
                }
            }
            ImportedItemKind::Function => {
                if package_item_is_public(&package.functions, item) {
                    mangle_function_name(package_path, item, &self.entry_package)
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        "PK010",
                        format!("package `{package_path}` does not export function `{item}`"),
                        span,
                    ));
                    format!("{alias}::{item}")
                }
            }
        }
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
    Record,
    Function,
}

fn insert_package_item_decl(
    items: &mut HashMap<String, Vec<PackageItemDecl>>,
    name: &str,
    visibility: Visibility,
    module_path: &str,
    span: Span,
    kind: PackageItemKind,
    package_path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let existing = items.entry(name.to_string()).or_default();
    let duplicate = existing.iter().find(|item| {
        item.module_path == module_path
            || (visibility != Visibility::Private && item.visibility != Visibility::Private)
    });

    if let Some(previous) = duplicate {
        let kind_name = match kind {
            PackageItemKind::Record => "record",
            PackageItemKind::Function => "function",
        };
        diagnostics.push(
            Diagnostic::new(
                "PK013",
                format!("duplicate top-level {kind_name} `{name}` in package `{package_path}`"),
                span,
            )
            .with_related(
                format!("previous `{name}` declaration is here"),
                previous.span,
            ),
        );
    }

    existing.push(PackageItemDecl {
        visibility,
        module_path: module_path.to_string(),
        span,
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

fn package_item_is_public(items: &HashMap<String, Vec<PackageItemDecl>>, name: &str) -> bool {
    items.get(name).is_some_and(|items| {
        items
            .iter()
            .any(|item| item.visibility == Visibility::Public)
    })
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

fn file_import_aliases(
    imports: &[ImportDecl],
    diagnostics: &mut Vec<Diagnostic>,
) -> HashMap<String, String> {
    let mut aliases = HashMap::new();
    for import in imports {
        if let Some(previous) = aliases.insert(import.alias.clone(), import.path.clone()) {
            diagnostics.push(Diagnostic::new(
                "PK007",
                format!(
                    "duplicate import alias `{}` for `{}` and `{}`",
                    import.alias, previous, import.path
                ),
                import.span,
            ));
        }
    }
    aliases
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
    matches!(name, "print" | "println")
}
