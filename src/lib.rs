pub mod api_diff;
pub mod ast;
pub mod bytecode;
pub mod cache;
pub mod cli_schema;
pub mod diagnostic;
pub mod doc;
pub mod formatter;
pub mod identity;
pub mod implementation_artifact;
pub mod interface;
pub mod json_decode;
pub mod known_enum;
pub mod lexer;
pub mod mir;
pub mod package;
pub mod package_signature;
pub mod parser;
pub mod prelude;
pub mod project_template;
pub mod resolver;
pub mod runtime;
pub mod schema_export;
pub mod span;
pub(crate) mod std_package;
pub mod symbol;
pub mod token;
pub mod typed_hir;
pub mod types;
pub mod typing;

pub use package::{
    PackageArchive, PackageArchiveEntry, PackageArchiveMaterializationOutput, PackageArchiveOutput,
    PackageArchiveResourceEntry, PackageArchiveVerifyOutput,
};
pub use project_template::{
    ProjectTemplate, ProjectTemplateInfo, ProjectTemplateOutput, project_template_infos,
};
pub use schema_export::{SchemaDecodeMode, SchemaExportOptions};

use ast::Program;
use bytecode::Program as BytecodeProgram;
use diagnostic::Diagnostic;
use interface::PackageInterfaceGraph;
use mir::Program as MirProgram;
use runtime::RunOutcome;
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs, io,
    path::{Path, PathBuf},
    thread,
};
use typed_hir::Program as TypedHirProgram;

#[derive(Clone, Debug)]
pub struct PackageAwareCheck {
    pub packages: package::LoadedPackageGraph,
    pub signatures: package_signature::PackageSignatureEnvironment,
    pub module_checks: Vec<PackageModuleCheck>,
    pub typed_program: TypedHirProgram,
}

#[derive(Clone, Debug)]
pub struct PackageModuleCheck {
    pub package: identity::PackageId,
    pub module: identity::ModuleId,
    pub module_path: String,
    pub resolve_output: resolver::ResolveOutput,
    pub type_output: typing::TypeCheckOutput,
    pub typed_program: TypedHirProgram,
}

pub fn render_json_config_schema_for_check(
    check: &PackageAwareCheck,
    options: &SchemaExportOptions,
) -> Result<String, Vec<Diagnostic>> {
    let default_package = check
        .packages
        .package_graph
        .package(check.packages.entry_package)
        .map(|package| package.path.as_str())
        .unwrap_or("");
    if let Some(package) = options.package.as_deref()
        && let Some(loaded_interfaces) = &check.packages.interfaces
        && loaded_interfaces.graph.package_by_path(package).is_some()
    {
        return schema_export::render_json_config_schema_for_interfaces(
            &loaded_interfaces.graph,
            &loaded_interfaces.symbols,
            package,
            options,
        );
    }
    let interfaces = check.typed_program.package_interfaces();
    schema_export::render_json_config_schema_for_interfaces(
        &interfaces,
        &check.typed_program.symbols,
        default_package,
        options,
    )
}

#[derive(Clone, Debug)]
pub struct PackageBuildOutput {
    pub artifact_root: PathBuf,
    pub artifacts: Vec<PathBuf>,
    pub written_artifacts: Vec<PathBuf>,
    pub reused_artifacts: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppBundleOutput {
    pub root: PathBuf,
    pub entry: PathBuf,
    pub launcher: PathBuf,
    pub program: String,
    pub artifacts: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppBundleInstallOutput {
    pub launcher: PathBuf,
    pub metadata: PathBuf,
    pub program: String,
    pub files: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppBundleUninstallOutput {
    pub launcher: PathBuf,
    pub metadata: PathBuf,
    pub program: String,
    pub files: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstalledAppState {
    Ready,
    InvalidMetadata,
    MetadataMismatch,
    MissingLauncher,
    LauncherMismatch,
    MissingBundleLauncher,
}

impl InstalledAppState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::InvalidMetadata => "invalidMetadata",
            Self::MetadataMismatch => "metadataMismatch",
            Self::MissingLauncher => "missingLauncher",
            Self::LauncherMismatch => "launcherMismatch",
            Self::MissingBundleLauncher => "missingBundleLauncher",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledAppEntry {
    pub program: String,
    pub state: InstalledAppState,
    pub reason: String,
    pub launcher: PathBuf,
    pub metadata: PathBuf,
    pub bundle: Option<PathBuf>,
    pub bundle_launcher: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledAppInventoryOutput {
    pub output_dir: PathBuf,
    pub metadata_dir: PathBuf,
    pub apps: Vec<InstalledAppEntry>,
}

#[derive(Clone, Debug)]
pub struct AppBundleInterfaceOutput {
    pub entry_package: String,
    pub artifact_root: PathBuf,
    pub interfaces: PackageInterfaceGraph,
    pub symbols: symbol::SymbolTable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppBundleArchiveOutput {
    pub path: PathBuf,
    pub content_hash: String,
    pub program: String,
    pub files: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppBundleArchiveVerifyOutput {
    pub path: PathBuf,
    pub content_hash: String,
    pub files: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppBundleArchiveUnpackOutput {
    pub root: PathBuf,
    pub files: Vec<PathBuf>,
}

pub type PackageResourceRoots = Vec<(String, PathBuf)>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactCacheExplanation {
    pub artifact_root: PathBuf,
    pub artifact_root_selection: ArtifactRootSelection,
    pub lockfile: Option<ArtifactCacheLockfileExplanation>,
    pub archive_caches: Vec<ArtifactCacheArchiveCacheExplanation>,
    pub packages: Vec<ArtifactCachePackageExplanation>,
    pub artifacts: Vec<ArtifactCacheArtifactExplanation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactRootSelection {
    Built,
    ArtifactRoot,
}

impl ArtifactRootSelection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Built => "built",
            Self::ArtifactRoot => "artifactRoot",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactCachePackageExplanation {
    pub path: String,
    pub role: ArtifactCachePackageRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactCachePackageRole {
    Entry,
    Dependency,
}

impl ArtifactCachePackageRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Entry => "entry",
            Self::Dependency => "dependency",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactCacheArtifactExplanation {
    pub artifact_kind: ArtifactCacheArtifactKind,
    pub package_path: String,
    pub path: PathBuf,
    pub state: ArtifactCacheArtifactState,
    pub reason: String,
    pub hashes: Vec<ArtifactCacheHashExplanation>,
    pub regeneration_commands: Vec<ArtifactCacheRegenerationCommand>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactCacheArtifactKind {
    Interface,
    CheckCache,
    Implementation,
}

impl ArtifactCacheArtifactKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Interface => "interface",
            Self::CheckCache => "checkCache",
            Self::Implementation => "implementation",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactCacheArtifactState {
    Missing,
    Fresh,
    Stale,
    HashMismatch,
    Invalid,
    Unknown,
}

impl ArtifactCacheArtifactState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Fresh => "fresh",
            Self::Stale => "stale",
            Self::HashMismatch => "hashMismatch",
            Self::Invalid => "invalid",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactCacheHashExplanation {
    pub role: String,
    pub hash_kind: String,
    pub package_path: Option<String>,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactCacheRegenerationCommand {
    pub role: String,
    pub command: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactCacheLockfileExplanation {
    pub path: PathBuf,
    pub state: ArtifactCacheArtifactState,
    pub reason: String,
    pub dependencies: Vec<ArtifactCacheLockfileDependencyExplanation>,
    pub hashes: Vec<ArtifactCacheHashExplanation>,
    pub regeneration_commands: Vec<ArtifactCacheRegenerationCommand>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactCacheLockfileDependencyExplanation {
    pub package_path: String,
    pub source_kind: String,
    pub source: String,
    pub hash_kind: String,
    pub hash: String,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactCacheArchiveCacheExplanation {
    pub package_path: String,
    pub archive_path: PathBuf,
    pub path: PathBuf,
    pub state: ArtifactCacheArtifactState,
    pub reason: String,
    pub hashes: Vec<ArtifactCacheHashExplanation>,
    pub regeneration_commands: Vec<ArtifactCacheRegenerationCommand>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatPathOutcome {
    pub path: PathBuf,
    pub changed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestRunOutcome {
    pub tests: Vec<TestCaseResult>,
}

impl TestRunOutcome {
    pub fn passed_count(&self) -> usize {
        self.tests
            .iter()
            .filter(|test| test.status == TestStatus::Passed)
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.tests
            .iter()
            .filter(|test| test.status == TestStatus::Failed)
            .count()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestCaseResult {
    pub name: String,
    pub status: TestStatus,
    pub message: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
    pub output_text: String,
    pub stderr_text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestStatus {
    Passed,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PackageBuildArtifact {
    path: PathBuf,
    reused: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscoveredTest {
    name: String,
    runtime_name: String,
    package_item: Option<identity::PackageItemId>,
    span: span::Span,
}

pub fn check_source(source: &str) -> Result<Program, Vec<Diagnostic>> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;
    if program.package.is_some() {
        return Err(vec![Diagnostic::new(
            "PK001",
            "package mode requires a file-based entrypoint",
            Default::default(),
        )]);
    }

    let mut diagnostics = resolver::resolve(&program);
    diagnostics.extend(typing::typecheck(&program));

    if diagnostics.is_empty() {
        Ok(program)
    } else {
        Err(diagnostics)
    }
}

pub fn syntax_check_path(path: &Path) -> Result<(), Vec<Diagnostic>> {
    package::entry_package_path_from_entry(path).map(|_| ())
}

pub fn check_path(path: &Path) -> Result<Program, Vec<Diagnostic>> {
    if package::entry_package_path_from_entry(path)?.is_some() {
        let check = check_package_aware_path(path)?;
        return check.packages.entry_program().cloned().ok_or_else(|| {
            vec![Diagnostic::new(
                "PK018",
                "entry module was not loaded in package-aware checking",
                Default::default(),
            )]
        });
    }
    let program = package::load_flattened_program_from_entry(path)?;
    let mut diagnostics = resolver::resolve(&program);
    diagnostics.extend(typing::typecheck(&program));

    if diagnostics.is_empty() {
        Ok(program)
    } else {
        Err(diagnostics)
    }
}

pub fn format_source(source: &str) -> Result<String, Vec<Diagnostic>> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;
    Ok(formatter::format_program_preserving_comments(
        &program, source,
    ))
}

pub fn check_format_path(path: &Path) -> Result<FormatPathOutcome, Vec<Diagnostic>> {
    let source = read_format_source(path)?;
    let formatted = format_path_source(path, &source)?;
    Ok(FormatPathOutcome {
        path: path.to_path_buf(),
        changed: source != formatted,
    })
}

pub fn format_path(path: &Path) -> Result<FormatPathOutcome, Vec<Diagnostic>> {
    let source = read_format_source(path)?;
    let formatted = format_path_source(path, &source)?;
    let changed = source != formatted;
    if changed {
        fs::write(path, formatted).map_err(|error| {
            vec![Diagnostic::new(
                "FMT002",
                format!(
                    "failed to write formatted source `{}`: {error}",
                    path.display()
                ),
                Default::default(),
            )]
        })?;
    }
    Ok(FormatPathOutcome {
        path: path.to_path_buf(),
        changed,
    })
}

fn format_path_source(path: &Path, source: &str) -> Result<String, Vec<Diagnostic>> {
    if let Some(package_path) = package::entry_package_path_from_entry(path)? {
        let tokens = lexer::lex(source)?;
        let has_explicit_package = tokens
            .iter()
            .find(|token| !matches!(token.kind, token::TokenKind::Newline))
            .is_some_and(|token| matches!(token.kind, token::TokenKind::Package));
        let mut program = parser::parse_inferred_package(tokens, package_path)?;
        if !has_explicit_package {
            program.package = None;
        }
        return Ok(formatter::format_program_preserving_comments(
            &program, source,
        ));
    }
    format_source(source)
}

fn read_format_source(path: &Path) -> Result<String, Vec<Diagnostic>> {
    fs::read_to_string(path).map_err(|error| {
        vec![Diagnostic::new(
            "FMT002",
            format!("failed to read source file `{}`: {error}", path.display()),
            Default::default(),
        )]
    })
}

pub fn compile_source(source: &str) -> Result<MirProgram, Vec<Diagnostic>> {
    compile_mir_source(source)
}

pub fn compile_path(path: &Path) -> Result<MirProgram, Vec<Diagnostic>> {
    compile_mir_path(path)
}

pub fn compile_mir_source(source: &str) -> Result<MirProgram, Vec<Diagnostic>> {
    let program = compile_typed_source(source)?;
    Ok(mir::lower_typed(&program))
}

pub fn compile_mir_path(path: &Path) -> Result<MirProgram, Vec<Diagnostic>> {
    let program = compile_typed_path(path)?;
    Ok(mir::lower_typed(&program))
}

pub fn compile_typed_source(source: &str) -> Result<TypedHirProgram, Vec<Diagnostic>> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;
    if program.package.is_some() {
        return Err(vec![Diagnostic::new(
            "PK001",
            "package mode requires a file-based entrypoint",
            Default::default(),
        )]);
    }

    let resolve_output = resolver::resolve_program(&program);
    let type_output = typing::typecheck_program(&program);
    let mut diagnostics = resolve_output.diagnostics;
    diagnostics.extend(type_output.diagnostics.clone());
    if diagnostics.is_empty() {
        Ok(typed_hir::lower(
            &program,
            &type_output,
            package::PackageSymbolGraph::default(),
        ))
    } else {
        Err(diagnostics)
    }
}

pub fn compile_typed_path(path: &Path) -> Result<TypedHirProgram, Vec<Diagnostic>> {
    if package::entry_package_path_from_entry(path)?.is_some() {
        return check_package_aware_path(path).map(|check| check.typed_program);
    }
    let loaded = package::load_flattened_from_entry(path)?;
    compile_flattened_typed_program(loaded)
}

pub fn check_package_aware_path(path: &Path) -> Result<PackageAwareCheck, Vec<Diagnostic>> {
    let packages = package::load_package_graph_from_entry(path)?;
    let diagnostics = package::validate_loaded_package_graph(&packages);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let signatures = package_signature::PackageSignatureEnvironment::from_loaded_graph(&packages)?;
    let module_checks = typecheck_loaded_package_modules(&packages, &signatures)?;
    let typed_program = package_typed_program(&packages, &module_checks)?;
    Ok(PackageAwareCheck {
        packages,
        signatures,
        module_checks,
        typed_program,
    })
}

pub fn check_package_aware_path_against_loaded_interfaces(
    path: &Path,
    interfaces: &PackageInterfaceGraph,
    interface_symbols: &symbol::SymbolTable,
) -> Result<PackageAwareCheck, Vec<Diagnostic>> {
    let packages = package::load_package_graph_from_entry_against_interfaces(
        path,
        interfaces,
        interface_symbols,
    )?;
    let diagnostics = package::validate_loaded_package_graph(&packages);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let signatures = package_signature::PackageSignatureEnvironment::from_loaded_graph(&packages)?;
    let module_checks = typecheck_loaded_package_modules(&packages, &signatures)?;
    let typed_program = package_typed_program(&packages, &module_checks)?;
    Ok(PackageAwareCheck {
        packages,
        signatures,
        module_checks,
        typed_program,
    })
}

fn typecheck_loaded_package_modules(
    packages: &package::LoadedPackageGraph,
    signatures: &package_signature::PackageSignatureEnvironment,
) -> Result<Vec<PackageModuleCheck>, Vec<Diagnostic>> {
    let mut module_checks = Vec::new();
    let mut diagnostics = Vec::new();
    for package in &packages.packages {
        if packages.is_loaded_interface_package_path(&package.path) {
            continue;
        }
        let Some(package_id) = packages.package_graph.package_id(&package.path) else {
            continue;
        };
        for file in &package.files {
            let Some(module_id) = packages
                .package_graph
                .module_id(package_id, &file.module_path)
            else {
                continue;
            };
            let resolve_output =
                resolver::resolve_package_module(&file.program, signatures, module_id);
            let has_resolve_diagnostics = !resolve_output.diagnostics.is_empty();
            diagnostics.extend(resolve_output.diagnostics.clone());
            if has_resolve_diagnostics {
                continue;
            }
            let type_output =
                typing::typecheck_package_module(&file.program, signatures, module_id);
            let has_diagnostics = !type_output.diagnostics.is_empty();
            diagnostics.extend(type_output.diagnostics.clone());
            if has_diagnostics {
                continue;
            }
            let mut typed_program =
                typed_hir::lower(&file.program, &type_output, packages.package_graph.clone());
            attach_package_items_to_module_typed_program(
                &mut typed_program,
                &packages.package_graph,
                module_id,
            );
            module_checks.push(PackageModuleCheck {
                package: package_id,
                module: module_id,
                module_path: file.module_path.clone(),
                resolve_output,
                type_output,
                typed_program,
            });
        }
    }
    if diagnostics.is_empty() {
        Ok(module_checks)
    } else {
        Err(diagnostics)
    }
}

fn package_typed_program(
    packages: &package::LoadedPackageGraph,
    module_checks: &[PackageModuleCheck],
) -> Result<TypedHirProgram, Vec<Diagnostic>> {
    let entry_exists = module_checks.iter().any(|check| {
        check.package == packages.entry_package && check.module == packages.entry_module
    });
    if !entry_exists {
        return Err(vec![Diagnostic::new(
            "PK018",
            "entry module was not typechecked in package-aware checking",
            Default::default(),
        )]);
    }
    let modules = module_checks
        .iter()
        .map(|check| check.typed_program.clone())
        .collect::<Vec<_>>();
    Ok(typed_hir::merge_modules(
        &modules,
        packages.package_graph.clone(),
    ))
}

fn attach_package_items_to_module_typed_program(
    program: &mut TypedHirProgram,
    package_graph: &package::PackageSymbolGraph,
    module: identity::ModuleId,
) {
    for statement in &mut program.statements {
        match statement {
            typed_hir::Stmt::Record(record) => {
                record.package_item = record.package_item.or_else(|| {
                    package_graph.item_id_in_module(
                        module,
                        &record.name,
                        package::PackageItemKind::Record,
                    )
                });
            }
            typed_hir::Stmt::Enum(enumeration) => {
                enumeration.package_item = enumeration.package_item.or_else(|| {
                    package_graph.item_id_in_module(
                        module,
                        &enumeration.name,
                        package::PackageItemKind::Enum,
                    )
                });
            }
            typed_hir::Stmt::OpaqueType(opaque) => {
                opaque.package_item = opaque.package_item.or_else(|| {
                    package_graph.item_id_in_module(
                        module,
                        &opaque.name,
                        package::PackageItemKind::OpaqueType,
                    )
                });
            }
            typed_hir::Stmt::Function(function) => {
                function.package_item = function.package_item.or_else(|| {
                    package_graph.item_id_in_module(
                        module,
                        &function.name,
                        package::PackageItemKind::Function,
                    )
                });
            }
            _ => {}
        }
    }
}

pub fn compile_typed_path_against_loaded_interfaces(
    path: &Path,
    interfaces: &PackageInterfaceGraph,
    interface_symbols: &symbol::SymbolTable,
) -> Result<TypedHirProgram, Vec<Diagnostic>> {
    check_package_aware_path_against_loaded_interfaces(path, interfaces, interface_symbols)
        .map(|check| check.typed_program)
}

pub fn compile_typed_path_against_interface_artifacts(
    path: &Path,
    interface_root: &Path,
) -> Result<TypedHirProgram, Vec<Diagnostic>> {
    let package_paths = package::import_paths_from_entry(path)?;
    let mut symbols = symbol::SymbolTable::default();
    let interfaces = PackageInterfaceGraph::read_persisted_artifacts(
        interface_root,
        &package_paths,
        &mut symbols,
    )?;
    compile_typed_path_against_loaded_interfaces(path, &interfaces, &symbols)
}

pub fn check_package_aware_path_against_interface_artifacts(
    path: &Path,
    interface_root: &Path,
) -> Result<PackageAwareCheck, Vec<Diagnostic>> {
    let package_paths = package::import_paths_from_entry(path)?;
    let mut symbols = symbol::SymbolTable::default();
    let interfaces = PackageInterfaceGraph::read_persisted_artifacts(
        interface_root,
        &package_paths,
        &mut symbols,
    )?;
    check_package_aware_path_against_loaded_interfaces(path, &interfaces, &symbols)
}

pub fn package_check_cache_key(
    path: &Path,
    interface_root: &Path,
) -> Result<cache::PackageCheckCacheKey, Vec<Diagnostic>> {
    cache::compute_package_check_cache_key(path, interface_root)
}

pub fn write_package_check_cache_artifact(
    path: &Path,
    key: &cache::PackageCheckCacheKey,
) -> Result<(), Diagnostic> {
    cache::write_package_check_artifact(path, key)
}

pub fn write_package_check_cache_artifact_for_root(
    path: &Path,
    artifact_root: &Path,
) -> Result<PathBuf, Vec<Diagnostic>> {
    check_package_aware_path_against_interface_artifacts(path, artifact_root)?;
    let key = cache::compute_package_check_cache_key(path, artifact_root)?;
    let artifact_path = cache::package_check_artifact_path_from_entry(artifact_root, path)?;
    cache::write_package_check_artifact(&artifact_path, &key)
        .map_err(|diagnostic| vec![diagnostic])?;
    Ok(artifact_path)
}

pub fn package_check_cache_artifact_path(
    root: &Path,
    entry_path: &Path,
) -> Result<std::path::PathBuf, Vec<Diagnostic>> {
    cache::package_check_artifact_path_from_entry(root, entry_path)
}

pub fn write_package_interface_artifacts(
    path: &Path,
    artifact_root: &Path,
    package_paths: &[String],
) -> Result<Vec<PathBuf>, Vec<Diagnostic>> {
    let check = check_package_aware_path(path)?;
    build_package_interface_artifacts_from_check(&check, artifact_root, package_paths)
        .map(package_artifact_paths)
}

pub fn render_package_docs(path: &Path) -> Result<String, Vec<Diagnostic>> {
    let check = check_package_aware_path(path)?;
    Ok(doc::render_package_docs(
        &check.typed_program.package_interfaces(),
        &check.typed_program.symbols,
    ))
}

pub fn create_project_template(
    root: &Path,
    template: ProjectTemplate,
) -> Result<ProjectTemplateOutput, Vec<Diagnostic>> {
    project_template::create_project_template(root, template)
}

fn build_package_interface_artifacts_from_check(
    check: &PackageAwareCheck,
    artifact_root: &Path,
    package_paths: &[String],
) -> Result<Vec<PackageBuildArtifact>, Vec<Diagnostic>> {
    let program = &check.typed_program;
    let interfaces = program.package_interfaces();
    let requested_packages = if package_paths.is_empty() {
        interfaces
            .packages
            .iter()
            .map(|package| package.path.clone())
            .collect::<Vec<_>>()
    } else {
        package_paths.to_vec()
    };

    let mut artifacts = Vec::new();
    let mut diagnostics = Vec::new();
    for package_path in requested_packages {
        match write_or_reuse_package_interface_artifact(
            artifact_root,
            &package_path,
            &interfaces,
            &program.symbols,
        ) {
            Ok(artifact) => artifacts.push(artifact),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    if diagnostics.is_empty() {
        Ok(artifacts)
    } else {
        Err(diagnostics)
    }
}

pub fn write_package_implementation_artifacts(
    path: &Path,
    artifact_root: &Path,
) -> Result<Vec<PathBuf>, Vec<Diagnostic>> {
    let check = check_package_aware_path(path)?;
    build_package_implementation_artifacts_from_check(&check, artifact_root)
        .map(package_artifact_paths)
}

fn build_package_implementation_artifacts_from_check(
    check: &PackageAwareCheck,
    artifact_root: &Path,
) -> Result<Vec<PackageBuildArtifact>, Vec<Diagnostic>> {
    let interfaces = check.typed_program.package_interfaces();
    let mut artifacts = Vec::new();
    let mut diagnostics = Vec::new();

    for package in &check.packages.packages {
        match build_package_implementation_artifact_from_check(
            check,
            artifact_root,
            &interfaces,
            &package.path,
        ) {
            Ok(Some(artifact)) => artifacts.push(artifact),
            Ok(None) => {}
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    if diagnostics.is_empty() {
        Ok(artifacts)
    } else {
        Err(diagnostics)
    }
}

fn build_package_implementation_artifact_from_check(
    check: &PackageAwareCheck,
    artifact_root: &Path,
    interfaces: &PackageInterfaceGraph,
    package_path: &str,
) -> Result<Option<PackageBuildArtifact>, Diagnostic> {
    let Some(package) = check
        .packages
        .packages
        .iter()
        .find(|package| package.path == package_path)
    else {
        return Ok(None);
    };
    let Some(package_id) = check.packages.package_graph.package_id(package_path) else {
        return Ok(None);
    };
    let modules = check
        .module_checks
        .iter()
        .filter(|module| module.package == package_id)
        .map(|module| module.typed_program.clone())
        .collect::<Vec<_>>();
    if modules.is_empty() {
        return Ok(None);
    }
    let typed_program = typed_hir::merge_modules(&modules, check.packages.package_graph.clone());
    let bytecode_program = bytecode::compile(mir::lower_typed(&typed_program));
    implementation_artifact::PackageImplementationArtifact::from_bytecode_package(
        package_path,
        package_source_hash(package),
        interfaces,
        &check.typed_program.symbols,
        &check.packages.package_graph,
        bytecode_program,
    )
    .and_then(|artifact| {
        let path = implementation_artifact::persisted_file_path(artifact_root, package_path);
        write_package_build_artifact_text(
            path,
            artifact.to_persisted_text(),
            "package implementation artifact",
            "PK022",
        )
    })
    .map(Some)
}

fn package_source_hash(package: &package::LoadedPackage) -> String {
    let mut files = package.files.iter().collect::<Vec<_>>();
    files.sort_by(|left, right| left.module_path.cmp(&right.module_path));

    let mut input = format!("package\t{}\n", package.path);
    for file in files {
        input.push_str(&format!(
            "file\t{}\t{}\n{}\n",
            file.module_path,
            file.source.len(),
            file.source
        ));
    }
    interface::stable_hash_hex(&input)
}

pub fn write_package_artifacts(
    path: &Path,
    artifact_root: &Path,
) -> Result<Vec<PathBuf>, Vec<Diagnostic>> {
    build_package_artifact_records(path, artifact_root).map(package_artifact_paths)
}

pub fn default_build_artifact_root(path: &Path) -> Result<PathBuf, Vec<Diagnostic>> {
    package::default_build_artifact_root_from_entry(path)
}

pub fn package_content_hash(path: &Path) -> Result<Option<String>, Vec<Diagnostic>> {
    package::package_content_hash_from_entry(path)
}

pub fn write_package_archive(
    path: &Path,
    archive_root: &Path,
) -> Result<PackageArchiveOutput, Vec<Diagnostic>> {
    package::write_package_archive_from_entry(path, archive_root)
}

pub fn read_package_archive(
    path: &Path,
    expected_content_hash: Option<&str>,
) -> Result<PackageArchive, Vec<Diagnostic>> {
    package::read_package_archive(path, expected_content_hash)
}

pub fn verify_package_archive(path: &Path) -> Result<PackageArchiveVerifyOutput, Vec<Diagnostic>> {
    package::verify_package_archive(path)
}

pub fn verify_package_archive_with_expected_hash(
    path: &Path,
    expected_content_hash: &str,
) -> Result<PackageArchiveVerifyOutput, Vec<Diagnostic>> {
    package::verify_package_archive_with_expected_hash(path, expected_content_hash)
}

pub fn validate_package_archive_bytes(
    bytes: &[u8],
    expected_content_hash: Option<&str>,
) -> Result<PackageArchive, Vec<Diagnostic>> {
    package::validate_package_archive_bytes(bytes, expected_content_hash)
}

pub fn materialize_package_archive(
    path: &Path,
    expected_content_hash: Option<&str>,
    destination_root: &Path,
) -> Result<PackageArchiveMaterializationOutput, Vec<Diagnostic>> {
    package::materialize_package_archive(path, expected_content_hash, destination_root)
}

pub fn unpack_package_archive(
    path: &Path,
    destination_root: &Path,
) -> Result<PackageArchiveMaterializationOutput, Vec<Diagnostic>> {
    package::unpack_package_archive(path, destination_root)
}

pub fn unpack_package_archive_with_expected_hash(
    path: &Path,
    expected_content_hash: &str,
    destination_root: &Path,
) -> Result<PackageArchiveMaterializationOutput, Vec<Diagnostic>> {
    package::unpack_package_archive_with_expected_hash(
        path,
        expected_content_hash,
        destination_root,
    )
}

pub fn materialize_package_archive_bytes(
    bytes: &[u8],
    expected_content_hash: Option<&str>,
    destination_root: &Path,
) -> Result<PackageArchiveMaterializationOutput, Vec<Diagnostic>> {
    package::materialize_package_archive_bytes(bytes, expected_content_hash, destination_root)
}

pub fn build_package_artifacts(path: &Path) -> Result<PackageBuildOutput, Vec<Diagnostic>> {
    let artifact_root = default_build_artifact_root(path)?;
    let artifacts = build_package_artifact_records(path, &artifact_root)?;
    package::write_lockfile_from_entry(path)?;
    let written_artifacts = artifacts
        .iter()
        .filter(|artifact| !artifact.reused)
        .map(|artifact| artifact.path.clone())
        .collect::<Vec<_>>();
    let reused_artifacts = artifacts
        .iter()
        .filter(|artifact| artifact.reused)
        .map(|artifact| artifact.path.clone())
        .collect::<Vec<_>>();
    let artifacts = package_artifact_paths(artifacts);
    Ok(PackageBuildOutput {
        artifact_root,
        artifacts,
        written_artifacts,
        reused_artifacts,
    })
}

pub fn emit_app_bundle(
    path: &Path,
    output_dir: &Path,
    program_name: Option<&str>,
) -> Result<AppBundleOutput, Vec<Diagnostic>> {
    emit_app_bundle_with_source_mode(
        path,
        output_dir,
        program_name,
        AppBundleSourceMode::SourceBacked,
    )
}

pub fn emit_source_free_app_bundle(
    path: &Path,
    output_dir: &Path,
    program_name: Option<&str>,
) -> Result<AppBundleOutput, Vec<Diagnostic>> {
    emit_app_bundle_with_source_mode(
        path,
        output_dir,
        program_name,
        AppBundleSourceMode::SourceFree,
    )
}

fn emit_app_bundle_with_source_mode(
    path: &Path,
    output_dir: &Path,
    program_name: Option<&str>,
    source_mode: AppBundleSourceMode,
) -> Result<AppBundleOutput, Vec<Diagnostic>> {
    let Some(project) = package::project_manifest_metadata_from_entry(path)? else {
        return Err(vec![
            app_bundle_diagnostic("app bundle emission requires a muga.toml manifest")
                .with_suggestion("run `emit-app-bundle` from a manifest project entrypoint"),
        ]);
    };
    let entry_package = package::entry_package_path_from_entry(path)?.ok_or_else(|| {
        vec![
            app_bundle_diagnostic("app bundle entrypoint must be a package-mode source file")
                .with_suggestion("use a source file inside a muga.toml manifest project"),
        ]
    })?;

    let program = match program_name {
        Some(program_name) => validate_app_bundle_program_name(program_name)?,
        None => default_app_bundle_program_name(&project.package_path),
    };
    let dependency_roots = app_bundle_dependency_roots(&project.dependencies)?;
    let source_relative =
        app_bundle_project_relative_path(&project.root, &project.source_root, "source root")?;
    let entry_relative = app_bundle_project_relative_path(&project.root, path, "entry source")?;
    let resource_relative = project
        .resource_root
        .as_ref()
        .map(|root| app_bundle_project_relative_path(&project.root, root, "resource root"))
        .transpose()?;
    let artifact_root = default_build_artifact_root(path)?;
    validate_app_bundle_output_location(
        output_dir,
        &project.source_root,
        project.resource_root.as_deref(),
        &artifact_root,
    )?;
    validate_app_bundle_dependency_output_locations(output_dir, &project.dependencies)?;
    ensure_empty_app_bundle_root(output_dir)?;

    package::package_content_hash_from_entry(path)?;
    let build = build_package_artifacts(path)?;

    fs::create_dir_all(output_dir).map_err(|error| {
        vec![app_bundle_diagnostic(format!(
            "failed to create app bundle root `{}`: {error}",
            output_dir.display()
        ))]
    })?;

    let mut files = Vec::new();
    if project.dependencies.is_empty() {
        copy_app_bundle_file(
            &project.manifest_path,
            &output_dir.join("muga.toml"),
            "manifest",
            &mut files,
        )?;
    } else {
        let manifest = app_bundle_manifest_text(
            &project.package_path,
            &source_relative,
            resource_relative.as_deref(),
            &project.direct_dependencies,
            Path::new(""),
            &dependency_roots,
        )?;
        write_app_bundle_text_file(
            &output_dir.join("muga.toml"),
            &manifest,
            "manifest",
            &mut files,
        )?;
    }
    if source_mode.includes_sources() {
        copy_app_bundle_tree(
            &project.source_root,
            &output_dir.join(&source_relative),
            AppBundleCopyMode::MugaSources,
            "source tree",
            &mut files,
        )?;
    }
    if let (Some(resource_root), Some(resource_relative)) =
        (project.resource_root.as_ref(), resource_relative.as_ref())
    {
        copy_app_bundle_tree(
            resource_root,
            &output_dir.join(resource_relative),
            AppBundleCopyMode::AllFiles,
            "resource tree",
            &mut files,
        )?;
    }
    for dependency in &project.dependencies {
        copy_app_bundle_dependency(
            dependency,
            output_dir,
            &dependency_roots,
            source_mode,
            &mut files,
        )?;
    }
    if source_mode.includes_sources() {
        if project.dependencies.is_empty() {
            let lockfile = project.root.join("muga.lock");
            if lockfile.is_file() {
                copy_app_bundle_file(
                    &lockfile,
                    &output_dir.join("muga.lock"),
                    "lockfile",
                    &mut files,
                )?;
            }
        } else if let Some(lockfile) =
            package::write_lockfile_from_entry(&output_dir.join(&entry_relative))?
        {
            files.push(lockfile);
        }
    }

    let bundle_artifact_root = output_dir.join(".muga").join("build");
    copy_app_bundle_tree(
        &build.artifact_root,
        &bundle_artifact_root,
        AppBundleCopyMode::AllFiles,
        "build artifacts",
        &mut files,
    )?;
    write_app_bundle_text_file(
        &output_dir.join(".muga").join(APP_BUNDLE_METADATA_FILE),
        &app_bundle_metadata_text(&entry_package),
        "metadata",
        &mut files,
    )?;
    let artifacts = app_bundle_artifact_paths(
        &build.artifact_root,
        &bundle_artifact_root,
        &build.artifacts,
    )?;

    let launcher = output_dir.join("bin").join(&program);
    write_app_bundle_text_file(
        &launcher,
        &app_bundle_launcher_text(),
        "launcher",
        &mut files,
    )?;
    make_app_bundle_launcher_executable(&launcher)?;
    write_app_bundle_text_file(
        &output_dir.join("README.md"),
        &app_bundle_readme_text(&program, &entry_relative, source_mode),
        "README",
        &mut files,
    )?;

    Ok(AppBundleOutput {
        root: output_dir.to_path_buf(),
        entry: output_dir.join(entry_relative),
        launcher,
        program,
        artifacts,
        files,
    })
}

pub fn install_app_bundle(
    bundle_dir: &Path,
    output_dir: &Path,
    program_name: Option<&str>,
) -> Result<AppBundleInstallOutput, Vec<Diagnostic>> {
    install_app_bundle_with_replace_owned(bundle_dir, output_dir, program_name, false)
}

pub fn install_app_bundle_replace_owned(
    bundle_dir: &Path,
    output_dir: &Path,
    program_name: Option<&str>,
) -> Result<AppBundleInstallOutput, Vec<Diagnostic>> {
    install_app_bundle_with_replace_owned(bundle_dir, output_dir, program_name, true)
}

fn install_app_bundle_with_replace_owned(
    bundle_dir: &Path,
    output_dir: &Path,
    program_name: Option<&str>,
    replace_owned: bool,
) -> Result<AppBundleInstallOutput, Vec<Diagnostic>> {
    if !bundle_dir.is_dir() {
        return Err(vec![
            app_bundle_diagnostic(format!(
                "app bundle `{}` is not a directory",
                bundle_dir.display()
            ))
            .with_suggestion("pass the directory created by `muga emit-app-bundle`"),
        ]);
    }
    let program = match program_name {
        Some(program_name) => validate_app_bundle_program_name(program_name)?,
        None => discover_app_bundle_program(bundle_dir)?,
    };
    let bundle_launcher = bundle_dir.join("bin").join(&program);
    if !bundle_launcher.is_file() {
        return Err(vec![
            app_bundle_diagnostic(format!(
                "app bundle launcher `{}` is missing",
                bundle_launcher.display()
            ))
            .with_suggestion("pass --program for the launcher name or re-emit the app bundle"),
        ]);
    }
    validate_app_bundle_artifacts(bundle_dir)?;

    fs::create_dir_all(output_dir).map_err(|error| {
        vec![app_bundle_diagnostic(format!(
            "failed to create app install directory `{}`: {error}",
            output_dir.display()
        ))]
    })?;
    let launcher = output_dir.join(&program);
    let metadata = app_bundle_install_metadata_path(output_dir, &program);
    let launcher_absolute = absolute_normalized_path(&launcher)?;
    if replace_owned {
        if launcher.exists() || metadata.exists() {
            validate_app_bundle_install_metadata(&metadata, &program, &launcher_absolute)?;
        }
    } else {
        if launcher.exists() {
            return Err(vec![
                app_bundle_diagnostic(format!(
                    "app install launcher `{}` already exists",
                    launcher.display()
                ))
                .with_suggestion("choose another --output-dir or --program"),
            ]);
        }
        if metadata.exists() {
            return Err(vec![
                app_bundle_diagnostic(format!(
                    "app install metadata `{}` already exists",
                    metadata.display()
                ))
                .with_suggestion("choose another --output-dir or --program"),
            ]);
        }
    }

    let mut files = Vec::new();
    let bundle_dir = absolute_normalized_path(bundle_dir)?;
    let bundle_launcher = absolute_normalized_path(&bundle_launcher)?;
    write_app_bundle_text_file(
        &launcher,
        &app_bundle_install_launcher_text(&bundle_launcher),
        "install launcher",
        &mut files,
    )?;
    make_app_bundle_launcher_executable(&launcher)?;
    write_app_bundle_text_file(
        &metadata,
        &app_bundle_install_metadata_text(
            &program,
            &launcher_absolute,
            &bundle_dir,
            &bundle_launcher,
        ),
        "install metadata",
        &mut files,
    )?;

    Ok(AppBundleInstallOutput {
        launcher,
        metadata,
        program,
        files,
    })
}

pub fn uninstall_app_bundle(
    output_dir: &Path,
    program_name: &str,
) -> Result<AppBundleUninstallOutput, Vec<Diagnostic>> {
    let program = validate_app_bundle_program_name(program_name)?;
    let launcher = output_dir.join(&program);
    let metadata_path = app_bundle_install_metadata_path(output_dir, &program);
    let launcher_absolute = absolute_normalized_path(&launcher)?;
    let metadata =
        validate_app_bundle_install_metadata(&metadata_path, &program, &launcher_absolute)?;

    let mut files = Vec::new();
    if launcher.exists() {
        let launcher_text = fs::read_to_string(&launcher).map_err(|error| {
            vec![app_bundle_diagnostic(format!(
                "failed to read app install launcher `{}`: {error}",
                launcher.display()
            ))]
        })?;
        let expected_launcher =
            app_bundle_install_launcher_text(Path::new(&metadata.bundle_launcher));
        if launcher_text != expected_launcher {
            return Err(vec![
                app_bundle_diagnostic(format!(
                    "app install launcher `{}` does not match install metadata",
                    launcher.display()
                ))
                .with_suggestion(
                    "uninstall only removes launchers still owned by muga install-app",
                ),
            ]);
        }
        fs::remove_file(&launcher).map_err(|error| {
            vec![app_bundle_diagnostic(format!(
                "failed to remove app install launcher `{}`: {error}",
                launcher.display()
            ))]
        })?;
        files.push(launcher.clone());
    }
    fs::remove_file(&metadata_path).map_err(|error| {
        vec![app_bundle_diagnostic(format!(
            "failed to remove app install metadata `{}`: {error}",
            metadata_path.display()
        ))]
    })?;
    files.push(metadata_path.clone());

    Ok(AppBundleUninstallOutput {
        launcher,
        metadata: metadata_path,
        program,
        files,
    })
}

pub fn list_installed_app_bundles(
    output_dir: &Path,
) -> Result<InstalledAppInventoryOutput, Vec<Diagnostic>> {
    let metadata_dir = output_dir.join(".muga").join("installed-apps");
    if !metadata_dir.exists() {
        return Ok(InstalledAppInventoryOutput {
            output_dir: output_dir.to_path_buf(),
            metadata_dir,
            apps: Vec::new(),
        });
    }
    if !metadata_dir.is_dir() {
        return Err(vec![app_bundle_diagnostic(format!(
            "app install metadata directory `{}` is not a directory",
            metadata_dir.display()
        ))]);
    }

    let mut apps = Vec::new();
    for entry in fs::read_dir(&metadata_dir).map_err(|error| {
        vec![app_bundle_diagnostic(format!(
            "failed to read app install metadata directory `{}`: {error}",
            metadata_dir.display()
        ))]
    })? {
        let entry = entry.map_err(|error| {
            vec![app_bundle_diagnostic(format!(
                "failed to read app install metadata entry in `{}`: {error}",
                metadata_dir.display()
            ))]
        })?;
        let path = entry.path();
        if !path.is_file()
            || path.extension().and_then(|extension| extension.to_str()) != Some("toml")
        {
            continue;
        }
        apps.push(installed_app_entry_from_metadata(output_dir, &path)?);
    }
    apps.sort_by(|left, right| {
        left.program
            .cmp(&right.program)
            .then_with(|| left.metadata.cmp(&right.metadata))
    });

    Ok(InstalledAppInventoryOutput {
        output_dir: output_dir.to_path_buf(),
        metadata_dir,
        apps,
    })
}

pub fn read_app_bundle_interfaces(
    bundle_dir: &Path,
) -> Result<AppBundleInterfaceOutput, Vec<Diagnostic>> {
    if !bundle_dir.is_dir() {
        return Err(vec![
            app_bundle_diagnostic(format!(
                "app bundle `{}` is not a directory",
                bundle_dir.display()
            ))
            .with_suggestion("pass the directory created by `muga emit-app-bundle`"),
        ]);
    }

    let entry_package = read_app_bundle_entry_package(bundle_dir)?;
    let artifact_root = bundle_dir.join(".muga").join("build");
    let mut symbols = symbol::SymbolTable::default();
    let interfaces = PackageInterfaceGraph::read_persisted_artifacts(
        &artifact_root,
        std::slice::from_ref(&entry_package),
        &mut symbols,
    )
    .map_err(|mut diagnostics| {
        add_app_bundle_artifact_guidance(&mut diagnostics, bundle_dir);
        diagnostics
    })?;

    Ok(AppBundleInterfaceOutput {
        entry_package,
        artifact_root,
        interfaces,
        symbols,
    })
}

pub fn app_bundle_program(
    bundle_dir: &Path,
    program_name: Option<&str>,
) -> Result<String, Vec<Diagnostic>> {
    match program_name {
        Some(program_name) => validate_app_bundle_program_name(program_name),
        None => discover_app_bundle_program(bundle_dir),
    }
}

fn validate_app_bundle_artifacts(bundle_dir: &Path) -> Result<(), Vec<Diagnostic>> {
    compile_bytecode_app_bundle(bundle_dir).map(|_| ())
}

pub fn write_app_bundle_archive(
    bundle_dir: &Path,
    archive_root: &Path,
    program_name: Option<&str>,
) -> Result<AppBundleArchiveOutput, Vec<Diagnostic>> {
    if !bundle_dir.is_dir() {
        return Err(vec![
            app_bundle_diagnostic(format!(
                "app bundle `{}` is not a directory",
                bundle_dir.display()
            ))
            .with_suggestion("pass the directory created by `muga emit-app-bundle`"),
        ]);
    }
    let program = match program_name {
        Some(program_name) => validate_app_bundle_program_name(program_name)?,
        None => discover_app_bundle_program(bundle_dir)?,
    };
    let launcher = bundle_dir.join("bin").join(&program);
    if !launcher.is_file() {
        return Err(vec![
            app_bundle_diagnostic(format!(
                "app bundle launcher `{}` is missing",
                launcher.display()
            ))
            .with_suggestion("pass --program for the launcher name or re-emit the app bundle"),
        ]);
    }
    validate_app_bundle_archive_output_location(archive_root, bundle_dir)?;
    validate_app_bundle_artifacts(bundle_dir)?;

    let files = collect_app_bundle_archive_files(bundle_dir)?;
    let bytes = app_bundle_archive_bytes(&files);
    let content_hash = package::content_hash_for_bytes(&bytes);
    let path = app_bundle_archive_file_path(archive_root, &program, &content_hash);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            vec![app_bundle_diagnostic(format!(
                "failed to create app archive directory `{}`: {error}",
                parent.display()
            ))]
        })?;
    }
    fs::write(&path, bytes).map_err(|error| {
        vec![app_bundle_diagnostic(format!(
            "failed to write app archive `{}`: {error}",
            path.display()
        ))]
    })?;
    Ok(AppBundleArchiveOutput {
        path,
        content_hash,
        program,
        files: files
            .into_iter()
            .map(|file| bundle_dir.join(file.path))
            .collect(),
    })
}

pub fn verify_app_bundle_archive(
    archive_path: &Path,
) -> Result<AppBundleArchiveVerifyOutput, Vec<Diagnostic>> {
    let (content_hash, files) = read_verified_app_bundle_archive(archive_path)?;
    Ok(app_bundle_archive_verify_output(
        archive_path,
        content_hash,
        files,
    ))
}

pub fn verify_app_bundle_archive_with_expected_hash(
    archive_path: &Path,
    expected_hash: &str,
) -> Result<AppBundleArchiveVerifyOutput, Vec<Diagnostic>> {
    let (content_hash, files) =
        read_verified_app_bundle_archive_with_expected_hash(archive_path, expected_hash)?;
    Ok(app_bundle_archive_verify_output(
        archive_path,
        content_hash,
        files,
    ))
}

fn app_bundle_archive_verify_output(
    archive_path: &Path,
    content_hash: String,
    files: Vec<AppBundleArchiveFile>,
) -> AppBundleArchiveVerifyOutput {
    AppBundleArchiveVerifyOutput {
        path: archive_path.to_path_buf(),
        content_hash,
        files: files.into_iter().map(|file| file.path).collect(),
    }
}

pub fn unpack_app_bundle_archive(
    archive_path: &Path,
    output_dir: &Path,
) -> Result<AppBundleArchiveUnpackOutput, Vec<Diagnostic>> {
    let (_, files) = read_verified_app_bundle_archive(archive_path)?;
    unpack_verified_app_bundle_archive_files(files, output_dir)
}

pub fn unpack_app_bundle_archive_with_expected_hash(
    archive_path: &Path,
    expected_hash: &str,
    output_dir: &Path,
) -> Result<AppBundleArchiveUnpackOutput, Vec<Diagnostic>> {
    let (_, files) =
        read_verified_app_bundle_archive_with_expected_hash(archive_path, expected_hash)?;
    unpack_verified_app_bundle_archive_files(files, output_dir)
}

fn unpack_verified_app_bundle_archive_files(
    files: Vec<AppBundleArchiveFile>,
    output_dir: &Path,
) -> Result<AppBundleArchiveUnpackOutput, Vec<Diagnostic>> {
    ensure_empty_app_bundle_root(output_dir)?;
    fs::create_dir_all(output_dir).map_err(|error| {
        vec![app_bundle_diagnostic(format!(
            "failed to create app archive output `{}`: {error}",
            output_dir.display()
        ))]
    })?;

    let mut written = Vec::new();
    for file in files {
        let target = output_dir.join(&file.path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                vec![app_bundle_diagnostic(format!(
                    "failed to create app archive output directory `{}`: {error}",
                    parent.display()
                ))]
            })?;
        }
        fs::write(&target, &file.contents).map_err(|error| {
            vec![app_bundle_diagnostic(format!(
                "failed to write app archive output `{}`: {error}",
                target.display()
            ))]
        })?;
        if app_bundle_archive_path_is_bin_launcher(&file.path) {
            make_app_bundle_launcher_executable(&target)?;
        }
        written.push(target);
    }

    Ok(AppBundleArchiveUnpackOutput {
        root: output_dir.to_path_buf(),
        files: written,
    })
}

fn read_verified_app_bundle_archive(
    archive_path: &Path,
) -> Result<(String, Vec<AppBundleArchiveFile>), Vec<Diagnostic>> {
    let expected_hash = expected_app_bundle_archive_hash_from_path(archive_path)?;
    read_verified_app_bundle_archive_with_expected_hash(archive_path, &expected_hash)
}

fn read_verified_app_bundle_archive_with_expected_hash(
    archive_path: &Path,
    expected_hash: &str,
) -> Result<(String, Vec<AppBundleArchiveFile>), Vec<Diagnostic>> {
    validate_app_bundle_expected_archive_hash(expected_hash)?;
    let bytes = fs::read(archive_path).map_err(|error| {
        vec![app_bundle_diagnostic(format!(
            "failed to read app archive `{}`: {error}",
            archive_path.display()
        ))]
    })?;
    let content_hash = package::content_hash_for_bytes(&bytes);
    if content_hash != expected_hash {
        return Err(vec![
            app_bundle_diagnostic(format!(
                "app archive hash mismatch: expected `{expected_hash}`, got `{content_hash}`"
            ))
            .with_suggestion("fetch or emit the app archive again"),
        ]);
    }
    let files = parse_app_bundle_archive_bytes(&bytes)?;
    Ok((content_hash, files))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppBundleCopyMode {
    MugaSources,
    AllFiles,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppBundleSourceMode {
    SourceBacked,
    SourceFree,
}

impl AppBundleSourceMode {
    fn includes_sources(self) -> bool {
        self == Self::SourceBacked
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AppBundleArchiveFile {
    path: PathBuf,
    contents: Vec<u8>,
}

const APP_BUNDLE_ARCHIVE_HEADER: &[u8] = b"muga-app-bundle-archive-v1\n";
const APP_BUNDLE_METADATA_FILE: &str = "app-bundle";

fn app_bundle_archive_file_path(archive_root: &Path, program: &str, content_hash: &str) -> PathBuf {
    let hash = content_hash.strip_prefix("sha256:").unwrap_or(content_hash);
    archive_root.join(format!("{program}-sha256-{hash}.mga"))
}

fn expected_app_bundle_archive_hash_from_path(path: &Path) -> Result<String, Vec<Diagnostic>> {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return Err(vec![app_bundle_diagnostic(format!(
            "app archive path `{}` must have a valid UTF-8 file name",
            path.display()
        ))]);
    };
    let Some(stem) = file_name.strip_suffix(".mga") else {
        return Err(vec![
            app_bundle_diagnostic(format!(
                "app archive `{file_name}` must use the `.mga` extension"
            ))
            .with_suggestion("use the archive file written by `muga emit-app-archive`"),
        ]);
    };
    let Some((program, hash)) = stem.rsplit_once("-sha256-") else {
        return Err(vec![
            app_bundle_diagnostic(format!(
                "app archive `{file_name}` must use a `*-sha256-<hash>.mga` file name"
            ))
            .with_suggestion("use the archive file written by `muga emit-app-archive`"),
        ]);
    };
    validate_app_bundle_program_name(program)?;
    validate_app_bundle_archive_hash(hash, file_name)?;
    Ok(format!("sha256:{hash}"))
}

fn validate_app_bundle_archive_hash(hash: &str, file_name: &str) -> Result<(), Vec<Diagnostic>> {
    if hash.len() != 64 || !hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(vec![app_bundle_diagnostic(format!(
            "app archive `{file_name}` must contain 64 hexadecimal digits after `-sha256-`"
        ))]);
    }
    if hash.chars().any(|ch| ch.is_ascii_uppercase()) {
        return Err(vec![app_bundle_diagnostic(format!(
            "app archive `{file_name}` hash must use lower-case hexadecimal"
        ))]);
    }
    Ok(())
}

fn validate_app_bundle_expected_archive_hash(hash: &str) -> Result<(), Vec<Diagnostic>> {
    let Some(hex) = hash.strip_prefix("sha256:") else {
        return Err(vec![app_bundle_diagnostic(format!(
            "app archive expected hash `{hash}` must start with `sha256:`"
        ))]);
    };
    if hex.len() != 64 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(vec![app_bundle_diagnostic(format!(
            "app archive expected hash `{hash}` must be `sha256:` followed by 64 hexadecimal digits"
        ))]);
    }
    if hex.chars().any(|ch| ch.is_ascii_uppercase()) {
        return Err(vec![app_bundle_diagnostic(format!(
            "app archive expected hash `{hash}` must use lower-case hexadecimal"
        ))]);
    }
    Ok(())
}

fn validate_app_bundle_archive_output_location(
    archive_root: &Path,
    bundle_dir: &Path,
) -> Result<(), Vec<Diagnostic>> {
    let archive_root = absolute_normalized_path(archive_root)?;
    let bundle_dir = absolute_normalized_path(bundle_dir)?;
    if archive_root.starts_with(&bundle_dir) {
        return Err(vec![
            app_bundle_diagnostic(format!(
                "app archive output `{}` must not be inside app bundle `{}`",
                archive_root.display(),
                bundle_dir.display()
            ))
            .with_suggestion("choose an archive root outside the bundle directory"),
        ]);
    }
    Ok(())
}

fn collect_app_bundle_archive_files(
    bundle_dir: &Path,
) -> Result<Vec<AppBundleArchiveFile>, Vec<Diagnostic>> {
    let mut files = Vec::new();
    collect_app_bundle_archive_files_from(bundle_dir, bundle_dir, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn collect_app_bundle_archive_files_from(
    bundle_dir: &Path,
    current: &Path,
    files: &mut Vec<AppBundleArchiveFile>,
) -> Result<(), Vec<Diagnostic>> {
    let entries = fs::read_dir(current).map_err(|error| {
        vec![app_bundle_diagnostic(format!(
            "failed to read app bundle archive input `{}`: {error}",
            current.display()
        ))]
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            vec![app_bundle_diagnostic(format!(
                "failed to read app bundle archive input `{}`: {error}",
                current.display()
            ))]
        })?;
        paths.push(entry.path());
    }
    paths.sort();

    for path in paths {
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            vec![app_bundle_diagnostic(format!(
                "failed to read app bundle archive metadata `{}`: {error}",
                path.display()
            ))]
        })?;
        if metadata.file_type().is_symlink() {
            return Err(vec![app_bundle_diagnostic(format!(
                "app bundle archive input `{}` must not be a symlink",
                path.display()
            ))]);
        }
        if metadata.is_dir() {
            collect_app_bundle_archive_files_from(bundle_dir, &path, files)?;
        } else if metadata.is_file() {
            let relative = path.strip_prefix(bundle_dir).map_err(|_| {
                vec![app_bundle_diagnostic(format!(
                    "app bundle archive file `{}` is outside bundle root `{}`",
                    path.display(),
                    bundle_dir.display()
                ))]
            })?;
            validate_app_bundle_archive_path(relative)?;
            let contents = fs::read(&path).map_err(|error| {
                vec![app_bundle_diagnostic(format!(
                    "failed to read app bundle archive file `{}`: {error}",
                    path.display()
                ))]
            })?;
            files.push(AppBundleArchiveFile {
                path: relative.to_path_buf(),
                contents,
            });
        }
    }
    Ok(())
}

fn app_bundle_archive_bytes(files: &[AppBundleArchiveFile]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(APP_BUNDLE_ARCHIVE_HEADER);
    for file in files {
        out.extend_from_slice(b"file\t");
        out.extend_from_slice(app_bundle_slash_path(&file.path).as_bytes());
        out.extend_from_slice(b"\t");
        out.extend_from_slice(file.contents.len().to_string().as_bytes());
        out.extend_from_slice(b"\n");
        out.extend_from_slice(&file.contents);
        out.extend_from_slice(b"\n");
    }
    out
}

fn parse_app_bundle_archive_bytes(
    bytes: &[u8],
) -> Result<Vec<AppBundleArchiveFile>, Vec<Diagnostic>> {
    if !bytes.starts_with(APP_BUNDLE_ARCHIVE_HEADER) {
        return Err(vec![app_bundle_diagnostic(
            "invalid app bundle archive header",
        )]);
    }
    let mut index = APP_BUNDLE_ARCHIVE_HEADER.len();
    let mut files = Vec::new();
    let mut seen = BTreeSet::new();
    let mut previous_path: Option<PathBuf> = None;

    while index < bytes.len() {
        let line_start = index;
        while index < bytes.len() && bytes[index] != b'\n' {
            index += 1;
        }
        if index >= bytes.len() {
            return Err(vec![app_bundle_diagnostic(
                "app bundle archive contains an unterminated file header",
            )]);
        }
        let header = std::str::from_utf8(&bytes[line_start..index]).map_err(|_| {
            vec![app_bundle_diagnostic(
                "app bundle archive file header must be UTF-8",
            )]
        })?;
        index += 1;
        let Some(rest) = header.strip_prefix("file\t") else {
            return Err(vec![app_bundle_diagnostic(format!(
                "unknown app bundle archive entry header `{header}`"
            ))]);
        };
        let Some((path_text, len_text)) = rest.rsplit_once('\t') else {
            return Err(vec![app_bundle_diagnostic(format!(
                "malformed app bundle archive file header `{header}`"
            ))]);
        };
        let len = len_text.parse::<usize>().map_err(|_| {
            vec![app_bundle_diagnostic(format!(
                "app bundle archive file `{path_text}` has invalid length `{len_text}`"
            ))]
        })?;
        let path = PathBuf::from(path_text);
        validate_app_bundle_archive_path(&path)?;
        if !seen.insert(path.clone()) {
            return Err(vec![app_bundle_diagnostic(format!(
                "app bundle archive contains duplicate file `{}`",
                path.display()
            ))]);
        }
        if let Some(previous) = &previous_path
            && previous >= &path
        {
            return Err(vec![app_bundle_diagnostic(format!(
                "app bundle archive files must be sorted: `{}` appears after `{}`",
                path.display(),
                previous.display()
            ))]);
        }
        previous_path = Some(path.clone());

        let end = index.checked_add(len).ok_or_else(|| {
            vec![app_bundle_diagnostic(format!(
                "app bundle archive file `{}` length overflows",
                path.display()
            ))]
        })?;
        if end > bytes.len() {
            return Err(vec![app_bundle_diagnostic(format!(
                "app bundle archive file `{}` is truncated",
                path.display()
            ))]);
        }
        let contents = bytes[index..end].to_vec();
        index = end;
        if bytes.get(index).copied() != Some(b'\n') {
            return Err(vec![app_bundle_diagnostic(format!(
                "app bundle archive file `{}` is missing a trailing newline",
                path.display()
            ))]);
        }
        index += 1;
        files.push(AppBundleArchiveFile { path, contents });
    }
    Ok(files)
}

fn validate_app_bundle_archive_path(path: &Path) -> Result<(), Vec<Diagnostic>> {
    if path.as_os_str().is_empty() {
        return Err(vec![app_bundle_diagnostic(
            "app bundle archive file path must not be empty",
        )]);
    }
    for component in path.components() {
        match component {
            std::path::Component::Normal(part) if part.to_str().is_some() => {}
            std::path::Component::Normal(_) => {
                return Err(vec![app_bundle_diagnostic(format!(
                    "app bundle archive file path `{}` must be valid UTF-8",
                    path.display()
                ))]);
            }
            _ => {
                return Err(vec![app_bundle_diagnostic(format!(
                    "app bundle archive file path `{}` must be relative and must not contain `.` or `..` components",
                    path.display()
                ))]);
            }
        }
    }
    let path_text = app_bundle_slash_path(path);
    if path_text.contains('\\') || path_text.contains('\t') || path_text.contains('\n') {
        return Err(vec![app_bundle_diagnostic(format!(
            "app bundle archive file path `{}` must use plain slash-separated paths",
            path.display()
        ))]);
    }
    Ok(())
}

fn app_bundle_archive_path_is_bin_launcher(path: &Path) -> bool {
    let components = app_bundle_path_components(path);
    components.len() == 2
        && components
            .first()
            .is_some_and(|component| component == "bin")
}

fn app_bundle_dependency_roots(
    dependencies: &[package::ProjectManifestDependencyMetadata],
) -> Result<BTreeMap<String, PathBuf>, Vec<Diagnostic>> {
    let mut roots = BTreeMap::new();
    let mut seen_roots = BTreeMap::<PathBuf, String>::new();
    for dependency in dependencies {
        let root = app_bundle_dependency_root_relative(&dependency.package_path);
        if let Some(existing) = seen_roots.insert(root.clone(), dependency.package_path.clone()) {
            return Err(vec![app_bundle_diagnostic(format!(
                "app bundle dependency roots for `{}` and `{existing}` both resolve to `{}`",
                dependency.package_path,
                root.display()
            ))]);
        }
        roots.insert(dependency.package_path.clone(), root);
    }
    Ok(roots)
}

fn app_bundle_dependency_root_relative(package_path: &str) -> PathBuf {
    let mut root = PathBuf::from(".muga").join("bundle-deps");
    for segment in package_path.split("::") {
        root.push(segment);
    }
    root
}

fn copy_app_bundle_dependency(
    dependency: &package::ProjectManifestDependencyMetadata,
    output_dir: &Path,
    dependency_roots: &BTreeMap<String, PathBuf>,
    source_mode: AppBundleSourceMode,
    files: &mut Vec<PathBuf>,
) -> Result<(), Vec<Diagnostic>> {
    let dependency_root_relative =
        dependency_roots
            .get(&dependency.package_path)
            .ok_or_else(|| {
                vec![app_bundle_diagnostic(format!(
                    "app bundle dependency `{}` is missing a bundle root",
                    dependency.package_path
                ))]
            })?;
    let source_relative = app_bundle_project_relative_path(
        &dependency.root,
        &dependency.source_root,
        "dependency source root",
    )?;
    let resource_relative = dependency
        .resource_root
        .as_ref()
        .map(|root| {
            app_bundle_project_relative_path(&dependency.root, root, "dependency resource root")
        })
        .transpose()?;
    let dependency_output_root = output_dir.join(dependency_root_relative);
    let manifest = app_bundle_manifest_text(
        &dependency.package_path,
        &source_relative,
        resource_relative.as_deref(),
        &dependency.dependencies,
        dependency_root_relative,
        dependency_roots,
    )?;

    write_app_bundle_text_file(
        &dependency_output_root.join("muga.toml"),
        &manifest,
        "dependency manifest",
        files,
    )?;
    if source_mode.includes_sources() {
        copy_app_bundle_tree(
            &dependency.source_root,
            &dependency_output_root.join(&source_relative),
            AppBundleCopyMode::MugaSources,
            "dependency source tree",
            files,
        )?;
    }
    if let (Some(resource_root), Some(resource_relative)) = (
        dependency.resource_root.as_ref(),
        resource_relative.as_ref(),
    ) {
        copy_app_bundle_tree(
            resource_root,
            &dependency_output_root.join(resource_relative),
            AppBundleCopyMode::AllFiles,
            "dependency resource tree",
            files,
        )?;
    }
    Ok(())
}

fn app_bundle_manifest_text(
    package_path: &str,
    source_relative: &Path,
    resource_relative: Option<&Path>,
    direct_dependencies: &[String],
    package_root_relative: &Path,
    dependency_roots: &BTreeMap<String, PathBuf>,
) -> Result<String, Vec<Diagnostic>> {
    let mut out = String::new();
    out.push_str("[package]\n");
    out.push_str(&format!(
        "name = {}\n",
        app_bundle_manifest_string(package_path)
    ));
    out.push_str(&format!(
        "source = {}\n",
        app_bundle_manifest_string(&app_bundle_manifest_path_value(source_relative))
    ));
    if let Some(resource_relative) = resource_relative {
        out.push_str(&format!(
            "resources = {}\n",
            app_bundle_manifest_string(&app_bundle_manifest_path_value(resource_relative))
        ));
    }

    if !direct_dependencies.is_empty() {
        out.push_str("\n[dependencies]\n");
        let mut dependencies = direct_dependencies.to_vec();
        dependencies.sort();
        dependencies.dedup();
        for dependency in dependencies {
            let dependency_root = dependency_roots.get(&dependency).ok_or_else(|| {
                vec![app_bundle_diagnostic(format!(
                    "app bundle manifest for `{package_path}` references unknown dependency `{dependency}`"
                ))]
            })?;
            let relative_path =
                app_bundle_relative_path_between(package_root_relative, dependency_root);
            out.push_str(&format!(
                "{} = {{ path = {} }}\n",
                app_bundle_manifest_string(&dependency),
                app_bundle_manifest_string(&app_bundle_manifest_path_value(&relative_path))
            ));
        }
    }
    Ok(out)
}

fn app_bundle_relative_path_between(from: &Path, to: &Path) -> PathBuf {
    let from_components = app_bundle_path_components(from);
    let to_components = app_bundle_path_components(to);
    let mut common = 0;
    while common < from_components.len()
        && common < to_components.len()
        && from_components[common] == to_components[common]
    {
        common += 1;
    }

    let mut relative = PathBuf::new();
    for _ in common..from_components.len() {
        relative.push("..");
    }
    for component in to_components.iter().skip(common) {
        relative.push(component);
    }
    relative
}

fn app_bundle_path_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect()
}

fn app_bundle_manifest_path_value(path: &Path) -> String {
    let text = app_bundle_slash_path(path);
    if text.is_empty() {
        ".".to_string()
    } else {
        text
    }
}

fn app_bundle_slash_path(path: &Path) -> String {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => parts.push(".".to_string()),
            std::path::Component::ParentDir => parts.push("..".to_string()),
            std::path::Component::Normal(part) => {
                parts.push(part.to_string_lossy().into_owned());
            }
            std::path::Component::Prefix(prefix) => {
                parts.push(prefix.as_os_str().to_string_lossy().into_owned());
            }
            std::path::Component::RootDir => {}
        }
    }
    parts.join("/")
}

fn app_bundle_manifest_string(value: &str) -> String {
    let mut escaped = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped.push('"');
    escaped
}

fn discover_app_bundle_program(bundle_dir: &Path) -> Result<String, Vec<Diagnostic>> {
    let bin_dir = bundle_dir.join("bin");
    let entries = fs::read_dir(&bin_dir).map_err(|error| {
        vec![app_bundle_diagnostic(format!(
            "failed to read app bundle bin directory `{}`: {error}",
            bin_dir.display()
        ))]
    })?;
    let mut programs = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            vec![app_bundle_diagnostic(format!(
                "failed to read app bundle bin directory `{}`: {error}",
                bin_dir.display()
            ))]
        })?;
        let path = entry.path();
        if path.is_file()
            && let Some(name) = path.file_name().and_then(|name| name.to_str())
        {
            programs.push(name.to_string());
        }
    }
    programs.sort();
    match programs.as_slice() {
        [program] => validate_app_bundle_program_name(program),
        [] => Err(vec![
            app_bundle_diagnostic(format!(
                "app bundle bin directory `{}` does not contain a launcher",
                bin_dir.display()
            ))
            .with_suggestion("re-emit the app bundle or pass --program"),
        ]),
        _ => Err(vec![
            app_bundle_diagnostic(format!(
                "app bundle bin directory `{}` contains multiple launchers: {}",
                bin_dir.display(),
                programs.join(", ")
            ))
            .with_suggestion("pass --program to choose one launcher"),
        ]),
    }
}

fn validate_app_bundle_program_name(program: &str) -> Result<String, Vec<Diagnostic>> {
    if program.is_empty()
        || matches!(program, "." | "..")
        || program.starts_with('.')
        || program.starts_with('-')
        || program.chars().any(|ch| {
            !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
                || matches!(ch, '/' | '\\' | ':')
        })
    {
        return Err(vec![
            app_bundle_diagnostic(format!(
                "app bundle program name `{program}` is not a portable launcher name"
            ))
            .with_suggestion("use ASCII letters, digits, `_`, `-`, or `.`"),
        ]);
    }
    Ok(program.to_string())
}

fn default_app_bundle_program_name(package_path: &str) -> String {
    let mut name = String::new();
    let mut last_was_dash = false;
    for ch in package_path.chars() {
        let next = if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.') {
            ch
        } else {
            '-'
        };
        if next == '-' {
            if !last_was_dash {
                name.push(next);
            }
            last_was_dash = true;
        } else {
            name.push(next);
            last_was_dash = false;
        }
    }
    let name = name.trim_matches('-');
    if name.is_empty() {
        "muga-app".to_string()
    } else {
        name.to_string()
    }
}

fn app_bundle_project_relative_path(
    root: &Path,
    path: &Path,
    context: &str,
) -> Result<PathBuf, Vec<Diagnostic>> {
    let relative = path.strip_prefix(root).map_err(|_| {
        vec![
            app_bundle_diagnostic(format!(
                "app bundle {context} `{}` must live under project root `{}`",
                path.display(),
                root.display()
            ))
            .with_suggestion("use project-local source and resource roots for app bundles"),
        ]
    })?;
    for component in relative.components() {
        if !matches!(component, std::path::Component::Normal(_)) {
            return Err(vec![app_bundle_diagnostic(format!(
                "app bundle {context} `{}` must not contain `.` or `..` components",
                path.display()
            ))]);
        }
    }
    Ok(relative.to_path_buf())
}

fn validate_app_bundle_output_location(
    output_dir: &Path,
    source_root: &Path,
    resource_root: Option<&Path>,
    artifact_root: &Path,
) -> Result<(), Vec<Diagnostic>> {
    let output = absolute_normalized_path(output_dir)?;
    for (label, root) in [
        ("source root", Some(source_root)),
        ("resource root", resource_root),
        ("build artifact root", Some(artifact_root)),
    ] {
        let Some(root) = root else {
            continue;
        };
        validate_app_bundle_output_not_inside(output_dir, &output, label, root)?;
    }
    Ok(())
}

fn validate_app_bundle_dependency_output_locations(
    output_dir: &Path,
    dependencies: &[package::ProjectManifestDependencyMetadata],
) -> Result<(), Vec<Diagnostic>> {
    let output = absolute_normalized_path(output_dir)?;
    for dependency in dependencies {
        validate_app_bundle_output_not_inside(
            output_dir,
            &output,
            "dependency source root",
            &dependency.source_root,
        )?;
        if let Some(resource_root) = &dependency.resource_root {
            validate_app_bundle_output_not_inside(
                output_dir,
                &output,
                "dependency resource root",
                resource_root,
            )?;
        }
    }
    Ok(())
}

fn validate_app_bundle_output_not_inside(
    output_dir: &Path,
    output: &Path,
    label: &str,
    root: &Path,
) -> Result<(), Vec<Diagnostic>> {
    let root = absolute_normalized_path(root)?;
    if output.starts_with(&root) {
        return Err(vec![
            app_bundle_diagnostic(format!(
                "app bundle output `{}` must not be inside the {label} `{}`",
                output_dir.display(),
                root.display()
            ))
            .with_suggestion("choose an empty output directory outside copied inputs"),
        ]);
    }
    Ok(())
}

fn absolute_normalized_path(path: &Path) -> Result<PathBuf, Vec<Diagnostic>> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|error| {
                vec![app_bundle_diagnostic(format!(
                    "failed to resolve current directory for app bundle paths: {error}"
                ))]
            })?
            .join(path)
    };
    Ok(normalize_path_lexically(&path))
}

fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            std::path::Component::RootDir => normalized.push(component.as_os_str()),
            std::path::Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}

fn ensure_empty_app_bundle_root(output_dir: &Path) -> Result<(), Vec<Diagnostic>> {
    if !output_dir.exists() {
        return Ok(());
    }
    if !output_dir.is_dir() {
        return Err(vec![
            app_bundle_diagnostic(format!(
                "app bundle output `{}` already exists and is not a directory",
                output_dir.display()
            ))
            .with_suggestion("choose an empty output directory"),
        ]);
    }
    let mut entries = fs::read_dir(output_dir).map_err(|error| {
        vec![app_bundle_diagnostic(format!(
            "failed to read app bundle output `{}`: {error}",
            output_dir.display()
        ))]
    })?;
    if entries.next().is_some() {
        return Err(vec![
            app_bundle_diagnostic(format!(
                "app bundle output `{}` already exists and is not empty",
                output_dir.display()
            ))
            .with_suggestion("choose an empty output directory"),
        ]);
    }
    Ok(())
}

fn copy_app_bundle_tree(
    source_root: &Path,
    output_root: &Path,
    mode: AppBundleCopyMode,
    context: &str,
    files: &mut Vec<PathBuf>,
) -> Result<(), Vec<Diagnostic>> {
    let entries = fs::read_dir(source_root).map_err(|error| {
        vec![app_bundle_diagnostic(format!(
            "failed to read app bundle {context} `{}`: {error}",
            source_root.display()
        ))]
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            vec![app_bundle_diagnostic(format!(
                "failed to read app bundle {context} `{}`: {error}",
                source_root.display()
            ))]
        })?;
        paths.push(entry.path());
    }
    paths.sort();

    for source in paths {
        let file_name = source.file_name().ok_or_else(|| {
            vec![app_bundle_diagnostic(format!(
                "app bundle {context} entry `{}` has no file name",
                source.display()
            ))]
        })?;
        if file_name
            .to_str()
            .is_some_and(|name| matches!(name, ".git" | ".muga"))
        {
            continue;
        }
        let target = output_root.join(file_name);
        if source.is_dir() {
            copy_app_bundle_tree(&source, &target, mode, context, files)?;
        } else if source.is_file()
            && (mode == AppBundleCopyMode::AllFiles
                || source
                    .extension()
                    .is_some_and(|extension| extension == "muga"))
        {
            copy_app_bundle_file(&source, &target, context, files)?;
        }
    }
    Ok(())
}

fn copy_app_bundle_file(
    source: &Path,
    target: &Path,
    context: &str,
    files: &mut Vec<PathBuf>,
) -> Result<(), Vec<Diagnostic>> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            vec![app_bundle_diagnostic(format!(
                "failed to create app bundle {context} directory `{}`: {error}",
                parent.display()
            ))]
        })?;
    }
    fs::copy(source, target).map_err(|error| {
        vec![app_bundle_diagnostic(format!(
            "failed to copy app bundle {context} `{}` to `{}`: {error}",
            source.display(),
            target.display()
        ))]
    })?;
    files.push(target.to_path_buf());
    Ok(())
}

fn app_bundle_artifact_paths(
    source_artifact_root: &Path,
    bundle_artifact_root: &Path,
    artifacts: &[PathBuf],
) -> Result<Vec<PathBuf>, Vec<Diagnostic>> {
    let mut bundle_artifacts = Vec::new();
    for artifact in artifacts {
        let relative = artifact.strip_prefix(source_artifact_root).map_err(|_| {
            vec![app_bundle_diagnostic(format!(
                "app bundle build artifact `{}` is outside artifact root `{}`",
                artifact.display(),
                source_artifact_root.display()
            ))]
        })?;
        bundle_artifacts.push(bundle_artifact_root.join(relative));
    }
    Ok(bundle_artifacts)
}

fn write_app_bundle_text_file(
    target: &Path,
    text: &str,
    context: &str,
    files: &mut Vec<PathBuf>,
) -> Result<(), Vec<Diagnostic>> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            vec![app_bundle_diagnostic(format!(
                "failed to create app bundle {context} directory `{}`: {error}",
                parent.display()
            ))]
        })?;
    }
    fs::write(target, text).map_err(|error| {
        vec![app_bundle_diagnostic(format!(
            "failed to write app bundle {context} `{}`: {error}",
            target.display()
        ))]
    })?;
    files.push(target.to_path_buf());
    Ok(())
}

fn app_bundle_launcher_text() -> String {
    r#"#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
bundle_dir=$(CDPATH= cd "$script_dir/.." && pwd)
MUGA_BIN=${MUGA_BIN:-muga}

exec "$MUGA_BIN" run-app-bundle "$bundle_dir" -- "$@"
"#
    .to_string()
}

fn app_bundle_metadata_text(entry_package: &str) -> String {
    format!("muga-app-bundle-v1\nentry\t{entry_package}\n")
}

fn read_app_bundle_entry_package(bundle_dir: &Path) -> Result<String, Vec<Diagnostic>> {
    let path = bundle_dir.join(".muga").join(APP_BUNDLE_METADATA_FILE);
    let text = fs::read_to_string(&path).map_err(|error| {
        vec![
            app_bundle_diagnostic(format!(
                "failed to read app bundle metadata `{}`: {error}",
                path.display()
            ))
            .with_suggestion("re-emit the app bundle with `muga emit-app-bundle`"),
        ]
    })?;
    let mut lines = text.lines();
    if lines.next() != Some("muga-app-bundle-v1") {
        return Err(vec![app_bundle_diagnostic(format!(
            "invalid app bundle metadata `{}`",
            path.display()
        ))]);
    }
    let Some(entry_package) = lines.next().and_then(|line| line.strip_prefix("entry\t")) else {
        return Err(vec![app_bundle_diagnostic(format!(
            "app bundle metadata `{}` is missing an entry package",
            path.display()
        ))]);
    };
    if entry_package.is_empty() || entry_package.contains('\t') {
        return Err(vec![app_bundle_diagnostic(format!(
            "app bundle metadata `{}` contains an invalid entry package",
            path.display()
        ))]);
    }
    Ok(entry_package.to_string())
}

fn app_bundle_install_launcher_text(bundle_launcher: &Path) -> String {
    let launcher = shell_single_quoted_path(bundle_launcher);
    format!(
        r#"#!/usr/bin/env sh
set -eu

exec {launcher} "$@"
"#
    )
}

fn app_bundle_install_metadata_path(output_dir: &Path, program: &str) -> PathBuf {
    output_dir
        .join(".muga")
        .join("installed-apps")
        .join(format!("{program}.toml"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AppBundleInstallMetadata {
    bundle_launcher: String,
}

fn validate_app_bundle_install_metadata(
    metadata: &Path,
    program: &str,
    launcher: &Path,
) -> Result<AppBundleInstallMetadata, Vec<Diagnostic>> {
    let text = fs::read_to_string(metadata).map_err(|error| {
        vec![
            app_bundle_diagnostic(format!(
                "failed to read app install metadata `{}`: {error}",
                metadata.display()
            ))
            .with_suggestion(
                "use --replace-owned only for launchers previously installed by muga install-app",
            ),
        ]
    })?;
    let expected = [
        "format = \"muga-installed-app-v1\"".to_string(),
        format!("program = {}", app_bundle_manifest_string(program)),
        format!(
            "launcher = {}",
            app_bundle_manifest_string(&launcher.display().to_string())
        ),
    ];
    for line in expected {
        if !text.lines().any(|existing| existing == line) {
            return Err(vec![
                app_bundle_diagnostic(format!(
                    "app install metadata `{}` does not match this launcher",
                    metadata.display()
                ))
                .with_suggestion(
                    "use --replace-owned only for launchers previously installed by muga install-app",
                ),
            ]);
        }
    }
    let bundle_launcher = app_bundle_install_metadata_string_value(&text, "bundle_launcher")
        .map_err(|message| {
            vec![
                app_bundle_diagnostic(format!(
                    "invalid app install metadata `{}`: {message}",
                    metadata.display()
                ))
                .with_suggestion("reinstall the app with `muga install-app --replace-owned`"),
            ]
        })?;
    Ok(AppBundleInstallMetadata { bundle_launcher })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedAppBundleInstallMetadata {
    program: String,
    launcher: String,
    bundle: String,
    bundle_launcher: String,
}

fn parse_app_bundle_install_metadata_text(
    text: &str,
) -> Result<ParsedAppBundleInstallMetadata, String> {
    let format = app_bundle_install_metadata_string_value(text, "format")?;
    if format != "muga-installed-app-v1" {
        return Err(format!("unsupported format `{format}`"));
    }
    Ok(ParsedAppBundleInstallMetadata {
        program: app_bundle_install_metadata_string_value(text, "program")?,
        launcher: app_bundle_install_metadata_string_value(text, "launcher")?,
        bundle: app_bundle_install_metadata_string_value(text, "bundle")?,
        bundle_launcher: app_bundle_install_metadata_string_value(text, "bundle_launcher")?,
    })
}

fn installed_app_entry_from_metadata(
    output_dir: &Path,
    metadata: &Path,
) -> Result<InstalledAppEntry, Vec<Diagnostic>> {
    let fallback_program = installed_app_program_from_metadata_path(metadata);
    let fallback_launcher = output_dir.join(&fallback_program);
    let invalid_entry = |reason: String| InstalledAppEntry {
        program: fallback_program.clone(),
        state: InstalledAppState::InvalidMetadata,
        reason,
        launcher: fallback_launcher.clone(),
        metadata: metadata.to_path_buf(),
        bundle: None,
        bundle_launcher: None,
    };

    let text = match fs::read_to_string(metadata) {
        Ok(text) => text,
        Err(error) => {
            return Ok(invalid_entry(format!(
                "failed to read install metadata: {error}"
            )));
        }
    };
    let parsed = match parse_app_bundle_install_metadata_text(&text) {
        Ok(parsed) => parsed,
        Err(message) => {
            return Ok(invalid_entry(format!(
                "invalid install metadata: {message}"
            )));
        }
    };
    let launcher = PathBuf::from(&parsed.launcher);
    let bundle = PathBuf::from(&parsed.bundle);
    let bundle_launcher = PathBuf::from(&parsed.bundle_launcher);
    let expected_launcher = absolute_normalized_path(&output_dir.join(&parsed.program))?;
    if parsed.program != fallback_program || launcher != expected_launcher {
        return Ok(InstalledAppEntry {
            program: parsed.program,
            state: InstalledAppState::MetadataMismatch,
            reason: "metadata program or launcher does not match this output directory".to_string(),
            launcher,
            metadata: metadata.to_path_buf(),
            bundle: Some(bundle),
            bundle_launcher: Some(bundle_launcher),
        });
    }

    let (state, reason) = if !launcher.exists() {
        (
            InstalledAppState::MissingLauncher,
            "installed launcher is missing".to_string(),
        )
    } else {
        let expected_launcher_text = app_bundle_install_launcher_text(&bundle_launcher);
        match fs::read_to_string(&launcher) {
            Ok(text) if text == expected_launcher_text => {
                if bundle_launcher.is_file() {
                    (
                        InstalledAppState::Ready,
                        "launcher matches install metadata".to_string(),
                    )
                } else {
                    (
                        InstalledAppState::MissingBundleLauncher,
                        "bundle launcher is missing".to_string(),
                    )
                }
            }
            Ok(_) => (
                InstalledAppState::LauncherMismatch,
                "installed launcher does not match install metadata".to_string(),
            ),
            Err(error) => (
                InstalledAppState::LauncherMismatch,
                format!("failed to read installed launcher: {error}"),
            ),
        }
    };

    Ok(InstalledAppEntry {
        program: parsed.program,
        state,
        reason,
        launcher,
        metadata: metadata.to_path_buf(),
        bundle: Some(bundle),
        bundle_launcher: Some(bundle_launcher),
    })
}

fn installed_app_program_from_metadata_path(metadata: &Path) -> String {
    metadata
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn app_bundle_install_metadata_string_value(text: &str, key: &str) -> Result<String, String> {
    let prefix = format!("{key} = ");
    let Some(value) = text.lines().find_map(|line| line.strip_prefix(&prefix)) else {
        return Err(format!("missing `{key}`"));
    };
    parse_app_bundle_manifest_string(value).map_err(|error| format!("invalid `{key}`: {error}"))
}

fn parse_app_bundle_manifest_string(value: &str) -> Result<String, String> {
    let mut chars = value.chars();
    if chars.next() != Some('"') {
        return Err("expected string literal".to_string());
    }
    let mut out = String::new();
    let mut escaped = false;
    while let Some(ch) = chars.next() {
        if escaped {
            match ch {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                _ => return Err(format!("unsupported escape `\\{ch}`")),
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            if chars.next().is_some() {
                return Err("unexpected trailing characters".to_string());
            }
            return Ok(out);
        } else {
            out.push(ch);
        }
    }
    Err("unterminated string literal".to_string())
}

fn app_bundle_install_metadata_text(
    program: &str,
    launcher: &Path,
    bundle_dir: &Path,
    bundle_launcher: &Path,
) -> String {
    format!(
        "# muga install-app ownership metadata\nformat = \"muga-installed-app-v1\"\nprogram = {}\nlauncher = {}\nbundle = {}\nbundle_launcher = {}\n",
        app_bundle_manifest_string(program),
        app_bundle_manifest_string(&launcher.display().to_string()),
        app_bundle_manifest_string(&bundle_dir.display().to_string()),
        app_bundle_manifest_string(&bundle_launcher.display().to_string())
    )
}

fn app_bundle_readme_text(
    program: &str,
    entry_relative: &Path,
    source_mode: AppBundleSourceMode,
) -> String {
    let source_note = if source_mode.includes_sources() {
        format!(
            "The copied source entry is `{}` for inspection and `run --built` compatibility.",
            entry_relative.display()
        )
    } else {
        "This source-free bundle omits copied source files and is intended to run from artifacts."
            .to_string()
    };
    format!(
        r#"# Muga App Bundle

Run this bundle with:

```sh
bin/{program}
```

From this bundle directory, common distribution handoff commands are:

```sh
muga run-app-bundle .
muga install-app --output-dir <bin-dir> --program {program} .
muga list-installed-apps --output-dir <bin-dir>
muga uninstall-app --output-dir <bin-dir> --program {program}
muga emit-app-completions --format json --output-dir <completion-dir> --type <Type> .
muga emit-app-archive --archive-root <archive-dir> --program {program} .
muga verify-app-archive <archive-file>
muga unpack-app-archive [--format text|json] [--expected-hash sha256:<hex>] --output-dir <bundle-dir> <archive-file>
```

Use `--replace-owned` when updating a launcher already installed by Muga. These
commands never edit shell startup files.

The launcher uses `muga` from `PATH`, or `MUGA_BIN` when set. It executes
`muga run-app-bundle <bundle-root>` against the bundle-local manifest,
resources, and `.muga/build` artifacts. {}
"#,
        source_note
    )
}

fn shell_single_quoted_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    let mut quoted = String::from("'");
    for ch in text.chars() {
        if ch == '\'' {
            quoted.push_str("'\\''");
        } else {
            quoted.push(ch);
        }
    }
    quoted.push('\'');
    quoted
}

#[cfg(unix)]
fn make_app_bundle_launcher_executable(path: &Path) -> Result<(), Vec<Diagnostic>> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .map_err(|error| {
            vec![app_bundle_diagnostic(format!(
                "failed to read app bundle launcher metadata `{}`: {error}",
                path.display()
            ))]
        })?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).map_err(|error| {
        vec![app_bundle_diagnostic(format!(
            "failed to mark app bundle launcher `{}` executable: {error}",
            path.display()
        ))]
    })
}

#[cfg(not(unix))]
fn make_app_bundle_launcher_executable(_path: &Path) -> Result<(), Vec<Diagnostic>> {
    Ok(())
}

fn app_bundle_diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("PK031", message, Default::default())
}

pub fn explain_package_artifact_cache(
    path: &Path,
    artifact_root: &Path,
    artifact_root_selection: ArtifactRootSelection,
) -> Result<ArtifactCacheExplanation, Vec<Diagnostic>> {
    let lockfile_metadata = package::lockfile_metadata_from_entry(path)?;
    let lockfile = lockfile_metadata
        .as_ref()
        .map(explain_lockfile_metadata)
        .transpose()?;
    let archive_caches = lockfile_metadata
        .as_ref()
        .map(|metadata| explain_archive_cache_metadata(&metadata.archive_caches))
        .transpose()?
        .unwrap_or_default();
    let check = check_package_aware_path(path)?;
    let interfaces = check.typed_program.package_interfaces();
    let symbols = &check.typed_program.symbols;
    let entry_package = check
        .packages
        .package_graph
        .package(check.packages.entry_package)
        .map(|package| package.path.clone())
        .ok_or_else(|| {
            vec![Diagnostic::new(
                "PK018",
                "entry package was not loaded in package-aware checking",
                Default::default(),
            )]
        })?;

    let packages = check
        .packages
        .packages
        .iter()
        .map(|package| ArtifactCachePackageExplanation {
            path: package.path.clone(),
            role: if package.path == entry_package {
                ArtifactCachePackageRole::Entry
            } else {
                ArtifactCachePackageRole::Dependency
            },
        })
        .collect::<Vec<_>>();

    let mut artifacts = Vec::new();
    for package in &check.packages.packages {
        artifacts.push(explain_interface_artifact(
            artifact_root,
            &interfaces,
            symbols,
            &package.path,
        ));
        artifacts.push(explain_implementation_artifact(
            artifact_root,
            &interfaces,
            symbols,
            package,
        ));
    }

    artifacts.push(explain_check_cache_artifact(
        path,
        artifact_root,
        &interfaces,
        symbols,
        &entry_package,
    )?);

    Ok(ArtifactCacheExplanation {
        artifact_root: artifact_root.to_path_buf(),
        artifact_root_selection,
        lockfile,
        archive_caches,
        packages,
        artifacts,
    })
}

fn explain_lockfile_metadata(
    expected: &package::PackageLockfileMetadata,
) -> Result<ArtifactCacheLockfileExplanation, Vec<Diagnostic>> {
    let commands = lockfile_regeneration_commands();
    let mut explanation = ArtifactCacheLockfileExplanation {
        path: expected.path.clone(),
        state: ArtifactCacheArtifactState::Unknown,
        reason: String::new(),
        dependencies: expected
            .dependencies
            .iter()
            .map(|dependency| ArtifactCacheLockfileDependencyExplanation {
                package_path: dependency.package_path.clone(),
                source_kind: dependency.source_kind.as_str().to_string(),
                source: dependency.source.clone(),
                hash_kind: dependency.hash_kind.clone(),
                hash: dependency.hash.clone(),
                dependencies: dependency.dependencies.clone(),
            })
            .collect(),
        hashes: Vec::new(),
        regeneration_commands: Vec::new(),
    };

    if !expected.path.is_file() {
        explanation.state = ArtifactCacheArtifactState::Missing;
        explanation.reason = "expected package lockfile is missing".to_string();
        explanation.hashes.push(artifact_hash(
            "expected",
            "lockfile",
            None,
            expected.content_hash.clone(),
        ));
        explanation.regeneration_commands = commands;
        return Ok(explanation);
    }

    let actual = fs::read_to_string(&expected.path).map_err(|error| {
        vec![Diagnostic::new(
            "PK025",
            format!(
                "failed to read package lockfile `{}`: {error}",
                expected.path.display()
            ),
            Default::default(),
        )]
    })?;
    let actual_hash = artifact_hash(
        "actual",
        "lockfile",
        None,
        package::content_hash_for_bytes(actual.as_bytes()),
    );
    if actual == expected.text {
        explanation.state = ArtifactCacheArtifactState::Fresh;
        explanation.reason = "package lockfile metadata matches current dependencies".to_string();
        explanation.hashes.push(actual_hash);
        return Ok(explanation);
    }

    match package::validate_lockfile_text(&actual, &expected.path) {
        Ok(()) => {
            explanation.state = ArtifactCacheArtifactState::Stale;
            explanation.reason = "package lockfile metadata changed".to_string();
            explanation.hashes.push(artifact_hash(
                "expected",
                "lockfile",
                None,
                expected.content_hash.clone(),
            ));
            explanation.hashes.push(actual_hash);
        }
        Err(diagnostics) => {
            explanation.state = ArtifactCacheArtifactState::Invalid;
            explanation.reason = diagnostics
                .first()
                .map(|diagnostic| diagnostic.message.clone())
                .unwrap_or_else(|| "package lockfile failed to load".to_string());
            explanation.hashes.push(actual_hash);
        }
    }
    explanation.regeneration_commands = commands;
    Ok(explanation)
}

fn explain_archive_cache_metadata(
    metadata: &[package::PackageArchiveCacheMetadata],
) -> Result<Vec<ArtifactCacheArchiveCacheExplanation>, Vec<Diagnostic>> {
    metadata
        .iter()
        .map(explain_archive_cache_entry)
        .collect::<Result<Vec<_>, _>>()
}

fn explain_archive_cache_entry(
    metadata: &package::PackageArchiveCacheMetadata,
) -> Result<ArtifactCacheArchiveCacheExplanation, Vec<Diagnostic>> {
    let commands = archive_cache_regeneration_commands();
    let mut explanation = ArtifactCacheArchiveCacheExplanation {
        package_path: metadata.package_path.clone(),
        archive_path: metadata.archive_path.clone(),
        path: metadata.cache_root.clone(),
        state: ArtifactCacheArtifactState::Unknown,
        reason: String::new(),
        hashes: Vec::new(),
        regeneration_commands: Vec::new(),
    };

    if !metadata.cache_root.exists() {
        explanation.state = ArtifactCacheArtifactState::Missing;
        explanation.reason = "expected package archive dependency cache is missing".to_string();
        explanation.hashes.push(artifact_hash(
            "expected",
            "archiveCache",
            Some(&metadata.package_path),
            metadata.expected_content_hash.clone(),
        ));
        explanation.regeneration_commands = commands;
        return Ok(explanation);
    }

    if !metadata.cache_root.is_dir() {
        explanation.state = ArtifactCacheArtifactState::Invalid;
        explanation.reason = "package archive dependency cache path is not a directory".to_string();
        explanation.regeneration_commands = commands;
        return Ok(explanation);
    }

    match package::archive_dependency_cache_content_hash(&metadata.cache_root) {
        Ok(actual_hash) if actual_hash == metadata.expected_content_hash => {
            explanation.state = ArtifactCacheArtifactState::Fresh;
            explanation.reason =
                "package archive dependency cache matches declared archive hash".to_string();
            explanation.hashes.push(artifact_hash(
                "actual",
                "archiveCache",
                Some(&metadata.package_path),
                actual_hash,
            ));
        }
        Ok(actual_hash) => {
            explanation.state = ArtifactCacheArtifactState::HashMismatch;
            explanation.reason = "package archive dependency cache hash mismatch".to_string();
            explanation.hashes.push(artifact_hash(
                "expected",
                "archiveCache",
                Some(&metadata.package_path),
                metadata.expected_content_hash.clone(),
            ));
            explanation.hashes.push(artifact_hash(
                "actual",
                "archiveCache",
                Some(&metadata.package_path),
                actual_hash,
            ));
            explanation.regeneration_commands = commands;
        }
        Err(diagnostics) => {
            explanation.state = ArtifactCacheArtifactState::Invalid;
            explanation.reason = diagnostics
                .first()
                .map(|diagnostic| diagnostic.message.clone())
                .unwrap_or_else(|| "package archive dependency cache failed to load".to_string());
            explanation.regeneration_commands = commands;
        }
    }

    Ok(explanation)
}

fn explain_interface_artifact(
    artifact_root: &Path,
    interfaces: &PackageInterfaceGraph,
    symbols: &symbol::SymbolTable,
    package_path: &str,
) -> ArtifactCacheArtifactExplanation {
    let path = PackageInterfaceGraph::persisted_file_path(artifact_root, package_path);
    let expected_hash = interfaces.stable_hash_for_package(package_path, symbols);
    let commands = interface_regeneration_commands();
    let mut explanation =
        artifact_explanation(ArtifactCacheArtifactKind::Interface, package_path, path);

    let Some(expected_hash) = expected_hash else {
        explanation.state = ArtifactCacheArtifactState::Unknown;
        explanation.reason = "current package interface hash is unavailable".to_string();
        explanation.regeneration_commands = commands;
        return explanation;
    };

    if !explanation.path.is_file() {
        explanation.state = ArtifactCacheArtifactState::Missing;
        explanation.reason = "expected package interface artifact is missing".to_string();
        explanation.hashes.push(artifact_hash(
            "expected",
            "interface",
            Some(package_path),
            expected_hash,
        ));
        explanation.regeneration_commands = commands;
        return explanation;
    }

    let mut existing_symbols = symbol::SymbolTable::default();
    match PackageInterfaceGraph::read_persisted_file(&explanation.path, &mut existing_symbols) {
        Ok(existing) => match existing.stable_hash_for_package(package_path, &existing_symbols) {
            Some(actual_hash) if actual_hash == expected_hash => {
                explanation.state = ArtifactCacheArtifactState::Fresh;
                explanation.reason =
                    "artifact metadata matches current package interface".to_string();
                explanation.hashes.push(artifact_hash(
                    "actual",
                    "interface",
                    Some(package_path),
                    actual_hash,
                ));
            }
            Some(actual_hash) => {
                explanation.state = ArtifactCacheArtifactState::Stale;
                explanation.reason = "package interface hash changed".to_string();
                explanation.hashes.push(artifact_hash(
                    "expected",
                    "interface",
                    Some(package_path),
                    expected_hash,
                ));
                explanation.hashes.push(artifact_hash(
                    "actual",
                    "interface",
                    Some(package_path),
                    actual_hash,
                ));
                explanation.regeneration_commands = commands;
            }
            None => {
                explanation.state = ArtifactCacheArtifactState::Invalid;
                explanation.reason =
                    "package interface artifact does not contain the expected package".to_string();
                explanation.hashes.push(artifact_hash(
                    "expected",
                    "interface",
                    Some(package_path),
                    expected_hash,
                ));
                explanation.regeneration_commands = commands;
            }
        },
        Err(diagnostics) => {
            apply_artifact_diagnostics(&mut explanation, &diagnostics, commands);
        }
    }

    explanation
}

fn explain_implementation_artifact(
    artifact_root: &Path,
    interfaces: &PackageInterfaceGraph,
    symbols: &symbol::SymbolTable,
    package: &package::LoadedPackage,
) -> ArtifactCacheArtifactExplanation {
    let path = implementation_artifact::persisted_file_path(artifact_root, &package.path);
    let commands = implementation_regeneration_commands();
    let mut explanation = artifact_explanation(
        ArtifactCacheArtifactKind::Implementation,
        &package.path,
        path,
    );
    let expected_interface_hash = interfaces.stable_hash_for_package(&package.path, symbols);
    let expected_source_hash = package_source_hash(package);

    if !explanation.path.is_file() {
        explanation.state = ArtifactCacheArtifactState::Missing;
        explanation.reason = "expected package implementation artifact is missing".to_string();
        if let Some(expected_interface_hash) = expected_interface_hash {
            explanation.hashes.push(artifact_hash(
                "expected",
                "interface",
                Some(&package.path),
                expected_interface_hash,
            ));
        }
        explanation.hashes.push(artifact_hash(
            "expected",
            "source",
            Some(&package.path),
            expected_source_hash,
        ));
        explanation.regeneration_commands = commands;
        return explanation;
    }

    let artifact = match implementation_artifact::read_persisted_file(&explanation.path) {
        Ok(artifact) => artifact,
        Err(diagnostics) => {
            apply_artifact_diagnostics(&mut explanation, &diagnostics, commands);
            return explanation;
        }
    };

    if artifact.package_path != package.path {
        explanation.state = ArtifactCacheArtifactState::Invalid;
        explanation.reason = format!(
            "package implementation artifact contains `{}` instead of `{}`",
            artifact.package_path, package.path
        );
        explanation.regeneration_commands = commands;
        return explanation;
    }

    let mut stale_reasons = Vec::new();
    if let Some(expected_interface_hash) = expected_interface_hash {
        if artifact.interface_hash != expected_interface_hash {
            stale_reasons.push("interface hash changed".to_string());
            explanation.hashes.push(artifact_hash(
                "expected",
                "interface",
                Some(&package.path),
                expected_interface_hash,
            ));
            explanation.hashes.push(artifact_hash(
                "actual",
                "interface",
                Some(&package.path),
                artifact.interface_hash.clone(),
            ));
        } else {
            explanation.hashes.push(artifact_hash(
                "actual",
                "interface",
                Some(&package.path),
                artifact.interface_hash.clone(),
            ));
        }
    }

    if artifact.source_hash != expected_source_hash {
        stale_reasons.push("source hash changed".to_string());
        explanation.hashes.push(artifact_hash(
            "expected",
            "source",
            Some(&package.path),
            expected_source_hash,
        ));
        explanation.hashes.push(artifact_hash(
            "actual",
            "source",
            Some(&package.path),
            artifact.source_hash.clone(),
        ));
    } else {
        explanation.hashes.push(artifact_hash(
            "actual",
            "source",
            Some(&package.path),
            artifact.source_hash.clone(),
        ));
    }

    let expected_dependencies = interfaces
        .package_by_path(&package.path)
        .map(|interface| {
            let mut dependencies = interface.dependencies.clone();
            dependencies.sort();
            dependencies
        })
        .unwrap_or_default();
    let mut actual_dependencies = artifact
        .dependency_interfaces
        .iter()
        .map(|dependency| dependency.package_path.clone())
        .collect::<Vec<_>>();
    actual_dependencies.sort();
    let expected_dependency_hashes = expected_dependencies
        .iter()
        .filter_map(|dependency| {
            interfaces
                .stable_hash_for_package(dependency, symbols)
                .map(|hash| (dependency.clone(), hash))
        })
        .collect::<BTreeMap<_, _>>();
    let actual_dependency_hashes = artifact
        .dependency_interfaces
        .iter()
        .map(|dependency| {
            (
                dependency.package_path.clone(),
                dependency.interface_hash.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let expected_dependency_set = expected_dependencies
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let actual_dependency_set = actual_dependency_hashes
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    if actual_dependencies != expected_dependencies {
        stale_reasons.push("dependency interface set changed".to_string());
        for dependency in expected_dependency_set.difference(&actual_dependency_set) {
            if let Some(expected_hash) = expected_dependency_hashes.get(dependency) {
                explanation.hashes.push(artifact_hash(
                    "expected",
                    "dependencyInterface",
                    Some(dependency.as_str()),
                    expected_hash.clone(),
                ));
            }
        }
        for dependency in actual_dependency_set.difference(&expected_dependency_set) {
            if let Some(actual_hash) = actual_dependency_hashes.get(dependency) {
                explanation.hashes.push(artifact_hash(
                    "actual",
                    "dependencyInterface",
                    Some(dependency.as_str()),
                    actual_hash.clone(),
                ));
            }
        }
    }

    for dependency in &artifact.dependency_interfaces {
        match interfaces.stable_hash_for_package(&dependency.package_path, symbols) {
            Some(expected_hash) if expected_hash == dependency.interface_hash => {
                explanation.hashes.push(artifact_hash(
                    "actual",
                    "dependencyInterface",
                    Some(&dependency.package_path),
                    dependency.interface_hash.clone(),
                ));
            }
            Some(expected_hash) => {
                stale_reasons.push(format!(
                    "dependency interface `{}` changed",
                    dependency.package_path
                ));
                explanation.hashes.push(artifact_hash(
                    "expected",
                    "dependencyInterface",
                    Some(&dependency.package_path),
                    expected_hash,
                ));
                explanation.hashes.push(artifact_hash(
                    "actual",
                    "dependencyInterface",
                    Some(&dependency.package_path),
                    dependency.interface_hash.clone(),
                ));
            }
            None => {
                stale_reasons.push(format!(
                    "dependency interface `{}` is no longer loaded",
                    dependency.package_path
                ));
                explanation.hashes.push(artifact_hash(
                    "actual",
                    "dependencyInterface",
                    Some(&dependency.package_path),
                    dependency.interface_hash.clone(),
                ));
            }
        }
    }

    if stale_reasons.is_empty() {
        explanation.state = ArtifactCacheArtifactState::Fresh;
        explanation.reason =
            "artifact metadata matches current package implementation inputs".to_string();
    } else {
        stale_reasons.sort();
        stale_reasons.dedup();
        explanation.state = ArtifactCacheArtifactState::Stale;
        explanation.reason = stale_reasons.join("; ");
        explanation.regeneration_commands = commands;
    }

    explanation
}

fn explain_check_cache_artifact(
    entry_path: &Path,
    artifact_root: &Path,
    interfaces: &PackageInterfaceGraph,
    symbols: &symbol::SymbolTable,
    entry_package: &str,
) -> Result<ArtifactCacheArtifactExplanation, Vec<Diagnostic>> {
    let path = cache::package_check_artifact_path(artifact_root, entry_package);
    let commands = check_cache_regeneration_commands();
    let mut explanation =
        artifact_explanation(ArtifactCacheArtifactKind::CheckCache, entry_package, path);
    let expected = expected_check_cache_key(entry_path, interfaces, symbols, entry_package)?;

    if !explanation.path.is_file() {
        explanation.state = ArtifactCacheArtifactState::Missing;
        explanation.reason = "expected package check cache artifact is missing".to_string();
        explanation.hashes.push(artifact_hash(
            "expected",
            "artifact",
            None,
            expected.stable_hash(),
        ));
        explanation.regeneration_commands = commands;
        return Ok(explanation);
    }

    let actual = match cache::read_package_check_artifact(&explanation.path) {
        Ok(actual) => actual,
        Err(diagnostics) => {
            apply_artifact_diagnostics(&mut explanation, &diagnostics, commands);
            return Ok(explanation);
        }
    };

    let expected_hash = expected.stable_hash();
    let actual_hash = actual.stable_hash();
    if expected_hash == actual_hash {
        explanation.state = ArtifactCacheArtifactState::Fresh;
        explanation.reason = "artifact metadata matches current package check inputs".to_string();
        explanation
            .hashes
            .push(artifact_hash("actual", "artifact", None, actual_hash));
    } else {
        explanation.state = ArtifactCacheArtifactState::Stale;
        explanation.reason = check_cache_difference_details(&expected, &actual).join("; ");
        if explanation.reason.is_empty() {
            explanation.reason = "cache inputs changed".to_string();
        }
        explanation
            .hashes
            .push(artifact_hash("expected", "artifact", None, expected_hash));
        explanation
            .hashes
            .push(artifact_hash("actual", "artifact", None, actual_hash));
        add_check_cache_hash_differences(&mut explanation, &expected, &actual);
        explanation.regeneration_commands = commands;
    }

    Ok(explanation)
}

fn expected_check_cache_key(
    entry_path: &Path,
    interfaces: &PackageInterfaceGraph,
    symbols: &symbol::SymbolTable,
    entry_package: &str,
) -> Result<cache::PackageCheckCacheKey, Vec<Diagnostic>> {
    let source_input = package::source_fingerprint_input_from_entry(entry_path)?;
    let source_hash = interface::stable_hash_hex(&source_input);
    let mut dependency_interfaces = Vec::new();
    let mut diagnostics = Vec::new();
    for package in &interfaces.packages {
        if package.path == entry_package {
            continue;
        }
        match interfaces.stable_hash_for_package(&package.path, symbols) {
            Some(interface_hash) => {
                dependency_interfaces.push(cache::PackageDependencyInterfaceHash {
                    package_path: package.path.clone(),
                    interface_hash,
                });
            }
            None => diagnostics.push(Diagnostic::new(
                "PK016",
                format!("missing package interface hash for `{}`", package.path),
                Default::default(),
            )),
        }
    }

    if diagnostics.is_empty() {
        dependency_interfaces.sort_by(|left, right| left.package_path.cmp(&right.package_path));
        Ok(cache::PackageCheckCacheKey {
            source_hash,
            dependency_interfaces,
        })
    } else {
        Err(diagnostics)
    }
}

fn artifact_explanation(
    artifact_kind: ArtifactCacheArtifactKind,
    package_path: &str,
    path: PathBuf,
) -> ArtifactCacheArtifactExplanation {
    ArtifactCacheArtifactExplanation {
        artifact_kind,
        package_path: package_path.to_string(),
        path,
        state: ArtifactCacheArtifactState::Unknown,
        reason: String::new(),
        hashes: Vec::new(),
        regeneration_commands: Vec::new(),
    }
}

fn apply_artifact_diagnostics(
    explanation: &mut ArtifactCacheArtifactExplanation,
    diagnostics: &[Diagnostic],
    fallback_commands: Vec<ArtifactCacheRegenerationCommand>,
) {
    explanation.state = if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("hash mismatch"))
    {
        ArtifactCacheArtifactState::HashMismatch
    } else {
        ArtifactCacheArtifactState::Invalid
    };
    explanation.reason = diagnostics
        .first()
        .map(|diagnostic| diagnostic.message.clone())
        .unwrap_or_else(|| "artifact failed to load".to_string());
    explanation.hashes = artifact_hashes_from_diagnostics(diagnostics);
    explanation.regeneration_commands = regeneration_commands_from_diagnostics(diagnostics);
    if explanation.regeneration_commands.is_empty() {
        explanation.regeneration_commands = fallback_commands;
    }
}

fn artifact_hashes_from_diagnostics(
    diagnostics: &[Diagnostic],
) -> Vec<ArtifactCacheHashExplanation> {
    let mut hashes = Vec::new();
    for diagnostic in diagnostics {
        for context in diagnostic.context.iter() {
            if let diagnostic::DiagnosticContext::ArtifactHash {
                role,
                hash_kind,
                package_path,
                value,
            } = context
            {
                hashes.push(ArtifactCacheHashExplanation {
                    role: role.clone(),
                    hash_kind: hash_kind.clone(),
                    package_path: package_path.clone(),
                    value: value.clone(),
                });
            }
        }
    }
    hashes
}

fn regeneration_commands_from_diagnostics(
    diagnostics: &[Diagnostic],
) -> Vec<ArtifactCacheRegenerationCommand> {
    let mut commands = Vec::new();
    for diagnostic in diagnostics {
        for context in diagnostic.context.iter() {
            if let diagnostic::DiagnosticContext::RegenerationCommand { role, command } = context {
                commands.push(ArtifactCacheRegenerationCommand {
                    role: role.clone(),
                    command: command.clone(),
                });
            }
        }
    }
    commands
}

fn artifact_hash(
    role: &str,
    hash_kind: &str,
    package_path: Option<&str>,
    value: String,
) -> ArtifactCacheHashExplanation {
    ArtifactCacheHashExplanation {
        role: role.to_string(),
        hash_kind: hash_kind.to_string(),
        package_path: package_path.map(ToString::to_string),
        value,
    }
}

fn interface_regeneration_commands() -> Vec<ArtifactCacheRegenerationCommand> {
    vec![
        regeneration_command("default-build", "muga build <entry>"),
        regeneration_command(
            "artifact-root",
            "muga emit-artifacts --artifact-root <dir> <entry>",
        ),
        regeneration_command(
            "interface",
            "muga emit-interface --artifact-root <dir> <entry>",
        ),
    ]
}

fn implementation_regeneration_commands() -> Vec<ArtifactCacheRegenerationCommand> {
    vec![
        regeneration_command("default-build", "muga build <entry>"),
        regeneration_command(
            "artifact-root",
            "muga emit-artifacts --artifact-root <dir> <entry>",
        ),
    ]
}

fn check_cache_regeneration_commands() -> Vec<ArtifactCacheRegenerationCommand> {
    vec![
        regeneration_command("default-build", "muga build <entry>"),
        regeneration_command(
            "artifact-root",
            "muga emit-artifacts --artifact-root <dir> <entry>",
        ),
        regeneration_command(
            "check-cache",
            "muga emit-check-cache --artifact-root <dir> <entry>",
        ),
    ]
}

fn lockfile_regeneration_commands() -> Vec<ArtifactCacheRegenerationCommand> {
    vec![regeneration_command("default-build", "muga build <entry>")]
}

fn archive_cache_regeneration_commands() -> Vec<ArtifactCacheRegenerationCommand> {
    vec![regeneration_command("default-build", "muga build <entry>")]
}

fn regeneration_command(role: &str, command: &str) -> ArtifactCacheRegenerationCommand {
    ArtifactCacheRegenerationCommand {
        role: role.to_string(),
        command: command.to_string(),
    }
}

fn check_cache_difference_details(
    expected: &cache::PackageCheckCacheKey,
    actual: &cache::PackageCheckCacheKey,
) -> Vec<String> {
    let mut details = Vec::new();
    if expected.source_hash != actual.source_hash {
        details.push("entry package source changed".to_string());
    }

    let expected_dependencies = check_cache_dependency_hashes_by_path(expected);
    let actual_dependencies = check_cache_dependency_hashes_by_path(actual);

    for package_path in expected_dependencies.keys() {
        match actual_dependencies.get(package_path) {
            Some(actual_hash) if *actual_hash == expected_dependencies[package_path] => {}
            Some(_) => details.push(format!("dependency interface `{package_path}` changed")),
            None => details.push(format!("dependency interface `{package_path}` was added")),
        }
    }

    for package_path in actual_dependencies.keys() {
        if !expected_dependencies.contains_key(package_path) {
            details.push(format!("dependency interface `{package_path}` was removed"));
        }
    }

    details
}

fn add_check_cache_hash_differences(
    explanation: &mut ArtifactCacheArtifactExplanation,
    expected: &cache::PackageCheckCacheKey,
    actual: &cache::PackageCheckCacheKey,
) {
    if expected.source_hash != actual.source_hash {
        explanation.hashes.push(artifact_hash(
            "expected",
            "source",
            None,
            expected.source_hash.clone(),
        ));
        explanation.hashes.push(artifact_hash(
            "actual",
            "source",
            None,
            actual.source_hash.clone(),
        ));
    }

    let expected_dependencies = check_cache_dependency_hashes_by_path(expected);
    let actual_dependencies = check_cache_dependency_hashes_by_path(actual);
    for (package_path, expected_hash) in &expected_dependencies {
        match actual_dependencies.get(package_path) {
            Some(actual_hash) if actual_hash == expected_hash => {}
            Some(actual_hash) => {
                explanation.hashes.push(artifact_hash(
                    "expected",
                    "dependencyInterface",
                    Some(package_path),
                    (*expected_hash).to_string(),
                ));
                explanation.hashes.push(artifact_hash(
                    "actual",
                    "dependencyInterface",
                    Some(package_path),
                    (*actual_hash).to_string(),
                ));
            }
            None => {
                explanation.hashes.push(artifact_hash(
                    "expected",
                    "dependencyInterface",
                    Some(package_path),
                    (*expected_hash).to_string(),
                ));
            }
        }
    }

    for (package_path, actual_hash) in &actual_dependencies {
        if !expected_dependencies.contains_key(package_path) {
            explanation.hashes.push(artifact_hash(
                "actual",
                "dependencyInterface",
                Some(package_path),
                (*actual_hash).to_string(),
            ));
        }
    }
}

fn check_cache_dependency_hashes_by_path(
    key: &cache::PackageCheckCacheKey,
) -> BTreeMap<&str, &str> {
    key.dependency_interfaces
        .iter()
        .map(|dependency| {
            (
                dependency.package_path.as_str(),
                dependency.interface_hash.as_str(),
            )
        })
        .collect()
}

fn build_package_artifact_records(
    path: &Path,
    artifact_root: &Path,
) -> Result<Vec<PackageBuildArtifact>, Vec<Diagnostic>> {
    let check = check_package_aware_path(path)?;
    let mut artifacts = build_parallel_package_artifacts_from_check(&check, artifact_root)?;
    artifacts.push(build_package_check_cache_artifact_for_root(
        path,
        artifact_root,
    )?);
    Ok(artifacts)
}

fn build_parallel_package_artifacts_from_check(
    check: &PackageAwareCheck,
    artifact_root: &Path,
) -> Result<Vec<PackageBuildArtifact>, Vec<Diagnostic>> {
    let interfaces = check.typed_program.package_interfaces();
    let levels = package_build_levels(&check.packages)?;
    let mut artifacts = Vec::new();

    for level in levels {
        let level_results = thread::scope(|scope| {
            let mut handles = Vec::new();
            for package_path in level {
                let interfaces = &interfaces;
                handles.push((
                    package_path.clone(),
                    scope.spawn(move || {
                        build_package_artifacts_for_package(
                            check,
                            artifact_root,
                            interfaces,
                            &package_path,
                        )
                    }),
                ));
            }

            let mut results = Vec::new();
            for (package_path, handle) in handles {
                let result = match handle.join() {
                    Ok(result) => result,
                    Err(_) => Err(vec![Diagnostic::new(
                        "PK024",
                        format!("package build worker for `{package_path}` panicked"),
                        Default::default(),
                    )]),
                };
                results.push(result);
            }
            results
        });

        let mut diagnostics = Vec::new();
        for result in level_results {
            match result {
                Ok(mut package_artifacts) => artifacts.append(&mut package_artifacts),
                Err(mut package_diagnostics) => diagnostics.append(&mut package_diagnostics),
            }
        }
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
    }

    Ok(artifacts)
}

fn build_package_artifacts_for_package(
    check: &PackageAwareCheck,
    artifact_root: &Path,
    interfaces: &PackageInterfaceGraph,
    package_path: &str,
) -> Result<Vec<PackageBuildArtifact>, Vec<Diagnostic>> {
    let mut artifacts = Vec::new();
    let mut diagnostics = Vec::new();

    match write_or_reuse_package_interface_artifact(
        artifact_root,
        package_path,
        interfaces,
        &check.typed_program.symbols,
    ) {
        Ok(artifact) => artifacts.push(artifact),
        Err(diagnostic) => diagnostics.push(diagnostic),
    }

    match build_package_implementation_artifact_from_check(
        check,
        artifact_root,
        interfaces,
        package_path,
    ) {
        Ok(Some(artifact)) => artifacts.push(artifact),
        Ok(None) => {}
        Err(diagnostic) => diagnostics.push(diagnostic),
    }

    if diagnostics.is_empty() {
        Ok(artifacts)
    } else {
        Err(diagnostics)
    }
}

fn package_build_levels(
    loaded: &package::LoadedPackageGraph,
) -> Result<Vec<Vec<String>>, Vec<Diagnostic>> {
    let mut package_paths = loaded
        .packages
        .iter()
        .map(|package| package.path.clone())
        .collect::<Vec<_>>();
    package_paths.sort();

    let known_packages = package_paths.iter().cloned().collect::<BTreeSet<_>>();
    let mut remaining_dependencies = BTreeMap::<String, BTreeSet<String>>::new();
    let mut dependents = BTreeMap::<String, Vec<String>>::new();

    for package_path in &package_paths {
        let Some(package_id) = loaded.package_graph.package_id(package_path) else {
            continue;
        };
        let Some(package) = loaded.package_graph.package(package_id) else {
            continue;
        };
        let dependencies = package
            .imports
            .iter()
            .filter_map(|import| loaded.package_graph.package(import.package))
            .map(|dependency| dependency.path.clone())
            .filter(|dependency| dependency != package_path && known_packages.contains(dependency))
            .collect::<BTreeSet<_>>();
        for dependency in &dependencies {
            dependents
                .entry(dependency.clone())
                .or_default()
                .push(package_path.clone());
        }
        remaining_dependencies.insert(package_path.clone(), dependencies);
    }

    let mut ready = remaining_dependencies
        .iter()
        .filter(|(_, dependencies)| dependencies.is_empty())
        .map(|(package_path, _)| package_path.clone())
        .collect::<Vec<_>>();
    ready.sort();

    let mut levels = Vec::new();
    let mut built = BTreeSet::new();

    while !ready.is_empty() {
        let level = ready;
        ready = Vec::new();
        for package_path in &level {
            built.insert(package_path.clone());
            if let Some(package_dependents) = dependents.get(package_path) {
                for dependent in package_dependents {
                    let Some(dependencies) = remaining_dependencies.get_mut(dependent) else {
                        continue;
                    };
                    dependencies.remove(package_path);
                    if dependencies.is_empty() && !built.contains(dependent) {
                        ready.push(dependent.clone());
                    }
                }
            }
        }
        ready.sort();
        ready.dedup();
        levels.push(level);
    }

    if built.len() == package_paths.len() {
        Ok(levels)
    } else {
        let blocked = package_paths
            .into_iter()
            .filter(|package_path| !built.contains(package_path))
            .collect::<Vec<_>>();
        Err(vec![
            Diagnostic::new(
                "PK024",
                format!(
                    "package dependency graph contains a cycle involving {}",
                    blocked.join(", ")
                ),
                Default::default(),
            )
            .with_suggestion("remove cyclic package imports before building artifacts"),
        ])
    }
}

fn build_package_check_cache_artifact_for_root(
    path: &Path,
    artifact_root: &Path,
) -> Result<PackageBuildArtifact, Vec<Diagnostic>> {
    check_package_aware_path_against_interface_artifacts(path, artifact_root)?;
    let key = cache::compute_package_check_cache_key(path, artifact_root)?;
    let artifact_path = cache::package_check_artifact_path_from_entry(artifact_root, path)?;
    write_package_build_artifact_text(
        artifact_path,
        key.to_persisted_text(),
        "package check cache artifact",
        "PK020",
    )
    .map_err(|diagnostic| vec![diagnostic])
}

fn write_or_reuse_package_interface_artifact(
    artifact_root: &Path,
    package_path: &str,
    interfaces: &PackageInterfaceGraph,
    symbols: &symbol::SymbolTable,
) -> Result<PackageBuildArtifact, Diagnostic> {
    let path = PackageInterfaceGraph::persisted_file_path(artifact_root, package_path);
    let Some(expected_hash) = interfaces.stable_hash_for_package(package_path, symbols) else {
        return Err(Diagnostic::new(
            "PK016",
            format!("compiled package interfaces do not contain `{package_path}`"),
            Default::default(),
        )
        .with_suggestion("choose a package that is reachable from the entrypoint"));
    };

    if path.is_file() {
        let mut existing_symbols = symbol::SymbolTable::default();
        let should_reuse =
            match PackageInterfaceGraph::read_persisted_file(&path, &mut existing_symbols) {
                Ok(existing) => existing
                    .stable_hash_for_package(package_path, &existing_symbols)
                    .is_some_and(|actual_hash| actual_hash == expected_hash),
                Err(_) => false,
            };
        if should_reuse {
            return Ok(PackageBuildArtifact { path, reused: true });
        }
    }

    let text = interfaces.persisted_artifact_text(package_path, symbols)?;
    write_package_build_artifact_text(path, text, "package interface artifact", "PK018")
}

fn package_artifact_paths(artifacts: Vec<PackageBuildArtifact>) -> Vec<PathBuf> {
    artifacts
        .into_iter()
        .map(|artifact| artifact.path)
        .collect()
}

fn write_package_build_artifact_text(
    path: PathBuf,
    text: String,
    artifact_kind: &str,
    diagnostic_code: &'static str,
) -> Result<PackageBuildArtifact, Diagnostic> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            Diagnostic::new(
                diagnostic_code,
                format!(
                    "failed to create {artifact_kind} directory {}: {error}",
                    parent.display()
                ),
                Default::default(),
            )
        })?;
    }

    match fs::read_to_string(&path) {
        Ok(existing) if existing == text => {
            return Ok(PackageBuildArtifact { path, reused: true });
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(Diagnostic::new(
                diagnostic_code,
                format!(
                    "failed to read existing {artifact_kind} `{}`: {error}",
                    path.display()
                ),
                Default::default(),
            ));
        }
    }

    fs::write(&path, text).map_err(|error| {
        Diagnostic::new(
            diagnostic_code,
            format!(
                "failed to write {artifact_kind} `{}`: {error}",
                path.display()
            ),
            Default::default(),
        )
    })?;
    Ok(PackageBuildArtifact {
        path,
        reused: false,
    })
}

pub fn check_package_aware_path_against_default_build_artifacts(
    path: &Path,
) -> Result<PackageAwareCheck, Vec<Diagnostic>> {
    let artifact_root = default_build_artifact_root(path)?;
    check_package_aware_path_against_cached_artifact_root(path, &artifact_root).map_err(
        |mut diagnostics| {
            add_default_build_artifact_guidance(&mut diagnostics);
            diagnostics
        },
    )
}

pub fn compile_typed_path_against_cached_interface_artifacts(
    path: &Path,
    interface_root: &Path,
    checked_artifact_path: &Path,
) -> Result<TypedHirProgram, Vec<Diagnostic>> {
    let key = cache::compute_package_check_cache_key(path, interface_root)?;
    cache::validate_package_check_artifact(checked_artifact_path, &key)?;
    compile_typed_path_against_interface_artifacts(path, interface_root)
}

pub fn check_package_aware_path_against_cached_interface_artifacts(
    path: &Path,
    interface_root: &Path,
    checked_artifact_path: &Path,
) -> Result<PackageAwareCheck, Vec<Diagnostic>> {
    let key = cache::compute_package_check_cache_key(path, interface_root)?;
    cache::validate_package_check_artifact(checked_artifact_path, &key)?;
    check_package_aware_path_against_interface_artifacts(path, interface_root)
}

pub fn compile_typed_path_against_cached_artifact_root(
    path: &Path,
    artifact_root: &Path,
) -> Result<TypedHirProgram, Vec<Diagnostic>> {
    let checked_artifact_path = cache::package_check_artifact_path_from_entry(artifact_root, path)?;
    compile_typed_path_against_cached_interface_artifacts(
        path,
        artifact_root,
        &checked_artifact_path,
    )
}

pub fn check_package_aware_path_against_cached_artifact_root(
    path: &Path,
    artifact_root: &Path,
) -> Result<PackageAwareCheck, Vec<Diagnostic>> {
    let checked_artifact_path = cache::package_check_artifact_path_from_entry(artifact_root, path)?;
    check_package_aware_path_against_cached_interface_artifacts(
        path,
        artifact_root,
        &checked_artifact_path,
    )
}

fn compile_flattened_typed_program(
    loaded: package::LoadedFlattenedProgram,
) -> Result<TypedHirProgram, Vec<Diagnostic>> {
    let resolve_output = resolver::resolve_program(&loaded.program);
    let type_output = typing::typecheck_program(&loaded.program);
    let mut diagnostics = resolve_output.diagnostics;
    diagnostics.extend(type_output.diagnostics.clone());
    if diagnostics.is_empty() {
        let program = typed_hir::lower(&loaded.program, &type_output, loaded.package_graph);
        let interfaces = program.package_interfaces();
        diagnostics.extend(program.validate_package_references_against_interfaces(&interfaces));
        if diagnostics.is_empty() {
            Ok(program)
        } else {
            Err(diagnostics)
        }
    } else {
        Err(diagnostics)
    }
}

pub fn compile_bytecode_source(source: &str) -> Result<BytecodeProgram, Vec<Diagnostic>> {
    let program = compile_mir_source(source)?;
    Ok(bytecode::compile(program))
}

pub fn compile_bytecode_path(path: &Path) -> Result<BytecodeProgram, Vec<Diagnostic>> {
    let program = compile_mir_path(path)?;
    Ok(bytecode::compile(program))
}

pub fn compile_bytecode_path_for_run_against_artifact_root(
    path: &Path,
    artifact_root: &Path,
) -> Result<BytecodeProgram, Vec<Diagnostic>> {
    let entry = compile_typed_path_against_cached_artifact_root(path, artifact_root)?;
    let entry = bytecode::compile(mir::lower_typed(&entry));
    let package_paths = package::import_paths_from_entry(path)?;
    let mut symbols = symbol::SymbolTable::default();
    let interfaces = PackageInterfaceGraph::read_persisted_artifacts(
        artifact_root,
        &package_paths,
        &mut symbols,
    )?;
    let implementation_artifacts =
        implementation_artifact::read_persisted_artifacts_reserving_program_items(
            artifact_root,
            &interfaces,
            &symbols,
            &[&entry],
        )?;
    let dependencies = implementation_artifact::programs_from_artifacts(implementation_artifacts);
    Ok(bytecode::merge(entry, dependencies))
}

pub fn compile_bytecode_app_bundle(
    bundle_dir: &Path,
) -> Result<(BytecodeProgram, PackageResourceRoots), Vec<Diagnostic>> {
    let manifest = package::project_manifest_metadata_from_root(bundle_dir)?;
    let entry_package = read_app_bundle_entry_package(bundle_dir)?;
    let artifact_root = bundle_dir.join(".muga").join("build");
    let program = compile_bytecode_package_artifacts_for_run(&artifact_root, &entry_package)
        .map_err(|mut diagnostics| {
            add_app_bundle_artifact_guidance(&mut diagnostics, bundle_dir);
            diagnostics
        })?;
    Ok((program, package_resource_roots_from_manifest(&manifest)))
}

fn compile_bytecode_package_artifacts_for_run(
    artifact_root: &Path,
    entry_package: &str,
) -> Result<BytecodeProgram, Vec<Diagnostic>> {
    let mut symbols = symbol::SymbolTable::default();
    let interfaces = PackageInterfaceGraph::read_persisted_artifacts(
        artifact_root,
        &[entry_package.to_string()],
        &mut symbols,
    )?;
    let artifacts =
        implementation_artifact::read_persisted_artifacts(artifact_root, &interfaces, &symbols)?;
    let mut entry = None;
    let mut dependencies = Vec::new();
    for artifact in artifacts {
        if artifact.package_path == entry_package {
            entry = Some(artifact.program);
        } else {
            dependencies.push(artifact);
        }
    }
    let Some(entry) = entry else {
        return Err(vec![
            Diagnostic::new(
                "PK022",
                format!("missing package implementation artifact for `{entry_package}`"),
                Default::default(),
            )
            .with_suggestion("re-emit the app bundle with `muga emit-app-bundle`"),
        ]);
    };
    let dependencies = implementation_artifact::programs_from_artifacts(dependencies);
    Ok(bytecode::merge(entry, dependencies))
}

fn package_resource_roots_from_entry(path: &Path) -> Result<PackageResourceRoots, Vec<Diagnostic>> {
    let Some(project) = package::project_manifest_metadata_from_entry(path)? else {
        return Ok(Vec::new());
    };
    Ok(package_resource_roots_from_manifest(&project))
}

fn package_resource_roots_from_manifest(
    project: &package::ProjectManifestMetadata,
) -> PackageResourceRoots {
    let mut roots = Vec::new();
    if let Some(resource_root) = &project.resource_root {
        roots.push((project.package_path.clone(), resource_root.clone()));
    }
    for dependency in &project.dependencies {
        if let Some(resource_root) = &dependency.resource_root {
            roots.push((dependency.package_path.clone(), resource_root.clone()));
        }
    }
    roots.sort_by(|left, right| left.0.cmp(&right.0));
    roots.dedup_by(|left, right| left.0 == right.0);
    roots
}

pub fn test_source(source: &str) -> Result<TestRunOutcome, Vec<Diagnostic>> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;
    if program.package.is_some() {
        return Err(vec![Diagnostic::new(
            "PK001",
            "package mode requires a file-based entrypoint",
            Default::default(),
        )]);
    }

    let tests = discover_tests_in_program(&program);
    let loaded = package::LoadedFlattenedProgram {
        program,
        package_graph: package::PackageSymbolGraph::default(),
        package_exports: interface::PackageExportGraph::default(),
    };
    let typed_program = compile_flattened_typed_program(loaded)?;
    validate_test_cases(&typed_program, &tests)?;
    let bytecode_program = bytecode::compile(mir::lower_typed(&typed_program));
    Ok(run_discovered_tests(&bytecode_program, &tests))
}

pub fn test_path(path: &Path) -> Result<TestRunOutcome, Vec<Diagnostic>> {
    if package::entry_package_path_from_entry(path)?.is_some() {
        let check = check_package_aware_path(path)?;
        let package_resource_roots = package_resource_roots_from_entry(path)?;
        let tests = discover_tests_in_loaded_packages(&check.packages);
        validate_test_cases(&check.typed_program, &tests)?;
        let bytecode_program = bytecode::compile(mir::lower_typed(&check.typed_program));
        return Ok(run_discovered_tests_with_package_resources(
            &bytecode_program,
            &tests,
            &package_resource_roots,
        ));
    }

    let loaded = package::load_flattened_from_entry(path)?;
    let tests = discover_tests_in_program(&loaded.program);
    let typed_program = compile_flattened_typed_program(loaded)?;
    validate_test_cases(&typed_program, &tests)?;
    let bytecode_program = bytecode::compile(mir::lower_typed(&typed_program));
    Ok(run_discovered_tests(&bytecode_program, &tests))
}

fn discover_tests_in_program(program: &Program) -> Vec<DiscoveredTest> {
    program
        .statements
        .iter()
        .filter_map(|statement| match statement {
            ast::Stmt::FuncDecl(function) if is_test_function(function) => Some(DiscoveredTest {
                name: function.name.clone(),
                runtime_name: function.name.clone(),
                package_item: function.package_item,
                span: function.span,
            }),
            _ => None,
        })
        .collect()
}

fn discover_tests_in_loaded_packages(loaded: &package::LoadedPackageGraph) -> Vec<DiscoveredTest> {
    let mut tests = Vec::new();
    for package in &loaded.packages {
        if loaded.is_loaded_interface_package_path(&package.path) {
            continue;
        }
        let package_id = loaded.package_graph.package_id(&package.path);
        for file in &package.files {
            let module_id = package_id.and_then(|package_id| {
                loaded
                    .package_graph
                    .module_id(package_id, &file.module_path)
            });
            for statement in &file.program.statements {
                let ast::Stmt::FuncDecl(function) = statement else {
                    continue;
                };
                if !is_test_function(function) {
                    continue;
                }
                let package_item = function.package_item.or_else(|| {
                    module_id.and_then(|module_id| {
                        loaded.package_graph.item_id_in_module(
                            module_id,
                            &function.name,
                            package::PackageItemKind::Function,
                        )
                    })
                });
                let item = package_item.and_then(|item| loaded.package_graph.item(item));
                let runtime_name = item
                    .map(|item| item.mangled_name.clone())
                    .unwrap_or_else(|| function.name.clone());
                let source_name = item
                    .map(|item| item.name.as_str())
                    .unwrap_or(&function.name);
                let module_name = file
                    .module_path
                    .strip_suffix(".muga")
                    .unwrap_or(&file.module_path)
                    .replace('/', "::");
                let name = if module_name == "main" {
                    format!("{}::{source_name}", package.path)
                } else {
                    format!("{}::{module_name}::{source_name}", package.path)
                };
                tests.push(DiscoveredTest {
                    name,
                    runtime_name,
                    package_item,
                    span: function.span,
                });
            }
        }
    }
    tests
}

fn is_test_function(function: &ast::FuncDecl) -> bool {
    function
        .attributes
        .iter()
        .any(|attribute| attribute.name == "test")
}

fn validate_test_cases(
    program: &TypedHirProgram,
    tests: &[DiscoveredTest],
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    for test in tests {
        let Some(function) = typed_function_for_test(program, test) else {
            diagnostics.push(Diagnostic::new(
                "T024",
                format!(
                    "test function `{}` was not found after type checking",
                    test.name
                ),
                test.span,
            ));
            continue;
        };
        if !function.params.is_empty() {
            diagnostics.push(
                Diagnostic::new(
                    "T024",
                    format!("test function `{}` must not have parameters", test.name),
                    function.span,
                )
                .with_suggestion("remove parameters from the `@test` function"),
            );
        }
        if !is_valid_test_return_type(&function.return_ty) {
            diagnostics.push(
                Diagnostic::new(
                    "T024",
                    format!(
                        "test function `{}` must return Unit or Result[Unit, E]",
                        test.name
                    ),
                    function.span,
                )
                .with_suggestion("return `()` for success or `Result[Unit, E]` for fallible tests"),
            );
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn typed_function_for_test<'a>(
    program: &'a TypedHirProgram,
    test: &DiscoveredTest,
) -> Option<&'a typed_hir::FunctionStmt> {
    program
        .statements
        .iter()
        .find_map(|statement| match statement {
            typed_hir::Stmt::Function(function)
                if test
                    .package_item
                    .is_some_and(|item| function.package_item == Some(item))
                    || function.name == test.runtime_name =>
            {
                Some(function)
            }
            _ => None,
        })
}

fn is_valid_test_return_type(ty: &types::TypeInfo) -> bool {
    match ty {
        types::TypeInfo::Unit => true,
        types::TypeInfo::Result(ok, _) => matches!(ok.as_ref(), types::TypeInfo::Unit),
        _ => false,
    }
}

fn run_discovered_tests(program: &BytecodeProgram, tests: &[DiscoveredTest]) -> TestRunOutcome {
    run_discovered_tests_with_package_resources(program, tests, &[])
}

fn run_discovered_tests_with_package_resources(
    program: &BytecodeProgram,
    tests: &[DiscoveredTest],
    package_resource_roots: &[(String, PathBuf)],
) -> TestRunOutcome {
    let tests = tests
        .iter()
        .map(|test| {
            let result = match test.package_item {
                Some(package_item) => {
                    runtime::run_package_function_with_args_and_package_resources(
                        program,
                        package_item,
                        &test.name,
                        &[],
                        package_resource_roots,
                    )
                }
                None => runtime::run_function_with_args(program, &test.runtime_name, &[]),
            };
            match result {
                Ok(outcome) => test_result_from_outcome(test, outcome),
                Err(diagnostics) => TestCaseResult {
                    name: test.name.clone(),
                    status: TestStatus::Failed,
                    message: None,
                    diagnostics,
                    output_text: String::new(),
                    stderr_text: String::new(),
                },
            }
        })
        .collect();
    TestRunOutcome { tests }
}

fn test_result_from_outcome(test: &DiscoveredTest, outcome: RunOutcome) -> TestCaseResult {
    let (status, message) = match outcome.main_result.as_ref() {
        Some(value) if value.is_unit() => (TestStatus::Passed, None),
        Some(value) => match value.result_unit_status() {
            Some(Ok(())) => (TestStatus::Passed, None),
            Some(Err(message)) => (TestStatus::Failed, Some(message)),
            None => (
                TestStatus::Failed,
                Some(format!(
                    "test returned unsupported value `{value}`; expected Unit or Result[Unit, E]"
                )),
            ),
        },
        None => (
            TestStatus::Failed,
            Some("test function did not produce a value".to_string()),
        ),
    };
    let diagnostics = test_failure_diagnostics(&outcome, &message);
    TestCaseResult {
        name: test.name.clone(),
        status,
        message,
        diagnostics,
        output_text: outcome.output_text,
        stderr_text: outcome.stderr_text,
    }
}

fn test_failure_diagnostics(outcome: &RunOutcome, message: &Option<String>) -> Vec<Diagnostic> {
    let Some(message) = message else {
        return Vec::new();
    };
    outcome
        .runtime_diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == "R021" && diagnostic.message.ends_with(message.as_str())
        })
        .cloned()
        .collect()
}

pub fn run_source(source: &str) -> Result<RunOutcome, Vec<Diagnostic>> {
    run_source_with_args(source, &[])
}

pub fn run_source_with_args(
    source: &str,
    program_args: &[String],
) -> Result<RunOutcome, Vec<Diagnostic>> {
    let program = compile_bytecode_source(source)?;
    runtime::run_with_args(&program, program_args)
}

pub fn run_path(path: &Path) -> Result<RunOutcome, Vec<Diagnostic>> {
    run_path_with_args(path, &[])
}

pub fn run_path_with_args(
    path: &Path,
    program_args: &[String],
) -> Result<RunOutcome, Vec<Diagnostic>> {
    if package::entry_package_path_from_entry(path)?.is_some() {
        let check = check_package_aware_path(path)?;
        let package_resource_roots = package_resource_roots_from_entry(path)?;
        let program = bytecode::compile(mir::lower_typed(&check.typed_program));
        return runtime::run_with_args_and_package_resources(
            &program,
            program_args,
            &package_resource_roots,
        );
    }
    let program = compile_bytecode_path(path)?;
    runtime::run_with_args(&program, program_args)
}

pub fn run_app_bundle(bundle_dir: &Path) -> Result<RunOutcome, Vec<Diagnostic>> {
    run_app_bundle_with_args(bundle_dir, &[])
}

pub fn run_app_bundle_with_args(
    bundle_dir: &Path,
    program_args: &[String],
) -> Result<RunOutcome, Vec<Diagnostic>> {
    let (program, package_resource_roots) = compile_bytecode_app_bundle(bundle_dir)?;
    runtime::run_with_args_and_package_resources(&program, program_args, &package_resource_roots)
}

pub fn run_path_against_artifact_root(
    path: &Path,
    artifact_root: &Path,
) -> Result<RunOutcome, Vec<Diagnostic>> {
    run_path_against_artifact_root_with_args(path, artifact_root, &[])
}

pub fn run_path_against_artifact_root_with_args(
    path: &Path,
    artifact_root: &Path,
    program_args: &[String],
) -> Result<RunOutcome, Vec<Diagnostic>> {
    let package_resource_roots = package_resource_roots_from_entry(path)?;
    let program = compile_bytecode_path_for_run_against_artifact_root(path, artifact_root)?;
    runtime::run_with_args_and_package_resources(&program, program_args, &package_resource_roots)
}

pub fn run_path_against_default_build_artifacts(
    path: &Path,
) -> Result<RunOutcome, Vec<Diagnostic>> {
    run_path_against_default_build_artifacts_with_args(path, &[])
}

pub fn run_path_against_default_build_artifacts_with_args(
    path: &Path,
    program_args: &[String],
) -> Result<RunOutcome, Vec<Diagnostic>> {
    let artifact_root = default_build_artifact_root(path)?;
    run_path_against_artifact_root_with_args(path, &artifact_root, program_args).map_err(
        |mut diagnostics| {
            add_default_build_artifact_guidance(&mut diagnostics);
            diagnostics
        },
    )
}

fn add_default_build_artifact_guidance(diagnostics: &mut [Diagnostic]) {
    const SUGGESTION: &str =
        "run `muga build <entry>` before using `--built` to create default `.muga/build` artifacts";

    for diagnostic in diagnostics {
        if !matches!(
            diagnostic.code.as_str(),
            "PK016" | "PK020" | "PK021" | "PK022" | "PK023"
        ) {
            continue;
        }
        if diagnostic
            .suggestions
            .iter()
            .any(|suggestion| suggestion.message == SUGGESTION)
        {
            continue;
        }
        diagnostic
            .suggestions
            .push(diagnostic::DiagnosticSuggestion {
                message: SUGGESTION.to_string(),
                span: None,
                replacement: None,
            });
    }
}

fn add_app_bundle_artifact_guidance(diagnostics: &mut [Diagnostic], bundle_dir: &Path) {
    let suggestion = format!(
        "re-emit the app bundle with `muga emit-app-bundle --output-dir {} <entry>`",
        bundle_dir.display()
    );

    for diagnostic in diagnostics {
        if !matches!(diagnostic.code.as_str(), "PK016" | "PK022" | "PK023") {
            continue;
        }
        if diagnostic
            .suggestions
            .iter()
            .any(|existing| existing.message == suggestion)
        {
            continue;
        }
        diagnostic
            .suggestions
            .push(diagnostic::DiagnosticSuggestion {
                message: suggestion.clone(),
                span: None,
                replacement: None,
            });
    }
}
