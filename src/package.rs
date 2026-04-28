use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::identity::{PackageId, PackageItemId};
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
    let entry_program = crate::parser::parse(entry_tokens)?;
    if entry_program.package.is_none() {
        return Ok(LoadedProgram {
            program: entry_program,
            package_graph: PackageSymbolGraph::default(),
        });
    }

    let mut loader = PackageLoader::new(path.to_path_buf(), entry_program);
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
    pub items: Vec<PackageItemInfo>,
}

impl PackageSymbolGraph {
    pub fn package(&self, id: PackageId) -> Option<&PackageInfo> {
        self.packages.get(id.as_u32() as usize)
    }

    pub fn item(&self, id: PackageItemId) -> Option<&PackageItemInfo> {
        self.items.get(id.as_u32() as usize)
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInfo {
    pub id: PackageId,
    pub path: String,
    pub imports: Vec<PackageImportInfo>,
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
}

struct PackageData {
    files: Vec<ParsedFile>,
    public_records: HashSet<String>,
    public_functions: HashSet<String>,
    all_records: HashSet<String>,
    all_functions: HashSet<String>,
}

struct PackageLoader {
    entry_file: PathBuf,
    entry_program: Program,
    source_root: PathBuf,
    entry_package: String,
    packages: HashMap<String, PackageData>,
    loading: HashSet<String>,
    diagnostics: Vec<Diagnostic>,
}

impl PackageLoader {
    fn new(entry_file: PathBuf, entry_program: Program) -> Self {
        let entry_package = entry_program
            .package
            .as_ref()
            .expect("checked package mode")
            .path
            .clone();
        let source_root =
            infer_source_root(&entry_file, &entry_package).unwrap_or_else(|_| entry_file.clone());
        Self {
            entry_file,
            entry_program,
            source_root,
            entry_package,
            packages: HashMap::new(),
            loading: HashSet::new(),
            diagnostics: Vec::new(),
        }
    }

    fn load_and_flatten(&mut self) -> Result<LoadedProgram, Vec<Diagnostic>> {
        match infer_source_root(&self.entry_file, &self.entry_package) {
            Ok(source_root) => self.source_root = source_root,
            Err(diagnostic) => self.diagnostics.push(diagnostic),
        }

        self.load_package(
            self.entry_package.clone(),
            Some(ParsedFile {
                program: self.entry_program.clone(),
            }),
        );

        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }

        let package_paths = self.sorted_package_paths();

        let mut statements = Vec::new();
        for package_path in &package_paths {
            let Some(package) = self.packages.get(package_path) else {
                continue;
            };
            let all_records = package.all_records.clone();
            let all_functions = package.all_functions.clone();
            let public_records = package.public_records.clone();
            for file in &package.files {
                let import_aliases =
                    file_import_aliases(&file.program.imports, &mut self.diagnostics);
                let mut rewriter = PackageRewriter {
                    diagnostics: &mut self.diagnostics,
                    current_package: package_path.clone(),
                    entry_package: self.entry_package.clone(),
                    imports: import_aliases,
                    package_public_records: &public_records,
                    package_records: &all_records,
                    package_functions: &all_functions,
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

    fn load_package(&mut self, package_path: String, entry_file: Option<ParsedFile>) {
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

        let files = self.load_package_files(&package_path, entry_file);
        for file in &files {
            for import in &file.program.imports {
                self.load_package(import.path.clone(), None);
            }
        }

        let mut public_records = HashSet::new();
        let mut public_functions = HashSet::new();
        let mut all_records = HashSet::new();
        let mut all_functions = HashSet::new();

        for file in &files {
            for statement in &file.program.statements {
                match statement {
                    Stmt::RecordDecl(record) => {
                        if !all_records.insert(record.name.clone()) {
                            self.diagnostics.push(Diagnostic::new(
                                "PK013",
                                format!(
                                    "duplicate top-level record `{}` in package `{package_path}`",
                                    record.name
                                ),
                                record.span,
                            ));
                        }
                        if record.visibility == Visibility::Public {
                            public_records.insert(record.name.clone());
                        }
                    }
                    Stmt::FuncDecl(func) => {
                        if !all_functions.insert(func.name.clone()) {
                            self.diagnostics.push(Diagnostic::new(
                                "PK013",
                                format!(
                                    "duplicate top-level function `{}` in package `{package_path}`",
                                    func.name
                                ),
                                func.span,
                            ));
                        }
                        if func.visibility == Visibility::Public {
                            public_functions.insert(func.name.clone());
                        }
                    }
                    _ => {}
                }
            }
        }

        self.packages.insert(
            package_path.clone(),
            PackageData {
                files,
                public_records,
                public_functions,
                all_records,
                all_functions,
            },
        );
        self.loading.remove(&package_path);
    }

    fn load_package_files(
        &mut self,
        package_path: &str,
        entry_file: Option<ParsedFile>,
    ) -> Vec<ParsedFile> {
        if let Some(entry_file) = entry_file {
            return vec![entry_file];
        }

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
            let tokens = match crate::lexer::lex(&source) {
                Ok(tokens) => tokens,
                Err(diagnostics) => {
                    self.diagnostics.extend(diagnostics);
                    continue;
                }
            };
            let program = match crate::parser::parse(tokens) {
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
            files.push(ParsedFile { program });
        }
        files
    }

    fn package_dir(&self, package_path: &str) -> PathBuf {
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
        let mut items = Vec::new();

        for package_path in package_paths {
            let Some(package) = self.packages.get(package_path) else {
                continue;
            };
            let package_id = package_ids[package_path.as_str()];
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
                imports,
            });

            for file in &package.files {
                for statement in &file.program.statements {
                    match statement {
                        Stmt::RecordDecl(record) => {
                            let id = PackageItemId::new(items.len() as u32);
                            items.push(PackageItemInfo {
                                id,
                                package: package_id,
                                name: record.name.clone(),
                                kind: PackageItemKind::Record,
                                visibility: record.visibility,
                                span: record.span,
                                mangled_name: mangle_record_name(package_path, &record.name),
                            });
                        }
                        Stmt::FuncDecl(func) => {
                            let id = PackageItemId::new(items.len() as u32);
                            items.push(PackageItemInfo {
                                id,
                                package: package_id,
                                name: func.name.clone(),
                                kind: PackageItemKind::Function,
                                visibility: func.visibility,
                                span: func.span,
                                mangled_name: mangle_function_name(
                                    package_path,
                                    &func.name,
                                    &self.entry_package,
                                ),
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        PackageSymbolGraph { packages, items }
    }
}

struct PackageRewriter<'a> {
    diagnostics: &'a mut Vec<Diagnostic>,
    current_package: String,
    entry_package: String,
    imports: HashMap<String, String>,
    package_public_records: &'a HashSet<String>,
    package_records: &'a HashSet<String>,
    package_functions: &'a HashSet<String>,
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
        if record.visibility == Visibility::Public {
            for field in &record.fields {
                self.validate_public_type(&field.type_name, field.span);
            }
        }

        RecordDecl {
            id: record.id,
            name: mangle_record_name(&self.current_package, &record.name),
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
                    self.validate_public_type(type_name, param.span);
                }
            }
            if let Some(type_name) = &func.return_type {
                self.validate_public_type(type_name, func.span);
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
                mangle_function_name(&self.current_package, &func.name, &self.entry_package)
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
        if self.package_records.contains(name) {
            return mangle_record_name(&self.current_package, name);
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
        if self.package_functions.contains(name) {
            return mangle_function_name(&self.current_package, name, &self.entry_package);
        }
        name.to_string()
    }

    fn validate_public_type(&mut self, type_expr: &TypeExpr, span: Span) {
        match type_expr {
            TypeExpr::Int | TypeExpr::Bool | TypeExpr::String => {}
            TypeExpr::Named(name) => {
                if let Some((alias, item)) = split_qualified_name(name) {
                    let _ = self.resolve_imported_item(alias, item, ImportedItemKind::Record, span);
                    return;
                }
                if self.package_records.contains(name)
                    && !self.package_public_records.contains(name)
                {
                    self.diagnostics.push(Diagnostic::new(
                        "PK012",
                        format!("public API may not expose private record `{name}`"),
                        span,
                    ));
                }
            }
            TypeExpr::Function(function) => {
                for param in &function.params {
                    self.validate_public_type(param, span);
                }
                self.validate_public_type(&function.ret, span);
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
                if package.public_records.contains(item) {
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
                if package.public_functions.contains(item) {
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

fn split_package_path(path: &str) -> Vec<String> {
    path.split("::").map(ToString::to_string).collect()
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

fn mangle_record_name(package_path: &str, name: &str) -> String {
    format!("__muga_pkg__{}__{}", package_path.replace("::", "__"), name)
}

fn is_builtin_name(name: &str) -> bool {
    matches!(name, "print" | "println")
}
