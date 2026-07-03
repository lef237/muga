use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::identity::{ModuleId, PackageId, PackageItemId};
use crate::interface::{PackageExportGraph, PackageInterface, PackageInterfaceGraph};
use crate::span::Span;
use crate::symbol::SymbolTable;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageArchiveOutput {
    pub path: PathBuf,
    pub content_hash: String,
    pub package_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageArchiveVerifyOutput {
    pub path: PathBuf,
    pub content_hash: String,
    pub manifest: String,
    pub sources: Vec<String>,
    pub resources: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageArchiveMaterializationOutput {
    pub root: PathBuf,
    pub content_hash: String,
    pub files: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectManifestMetadata {
    pub manifest_path: PathBuf,
    pub root: PathBuf,
    pub source_root: PathBuf,
    pub resource_root: Option<PathBuf>,
    pub package_path: String,
    pub direct_dependencies: Vec<String>,
    pub dependencies: Vec<ProjectManifestDependencyMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectManifestDependencyMetadata {
    pub package_path: String,
    pub root: PathBuf,
    pub source_root: PathBuf,
    pub resource_root: Option<PathBuf>,
    pub source_kind: PackageLockfileDependencySourceKind,
    pub source: String,
    pub hash: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageArchive {
    pub content_hash: String,
    pub manifest: PackageArchiveEntry,
    pub sources: Vec<PackageArchiveEntry>,
    pub resources: Vec<PackageArchiveResourceEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageArchiveEntry {
    pub path: String,
    pub contents: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageArchiveResourceEntry {
    pub path: String,
    pub contents: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageLockfileMetadata {
    pub path: PathBuf,
    pub text: String,
    pub content_hash: String,
    pub dependencies: Vec<PackageLockfileDependencyMetadata>,
    pub archive_caches: Vec<PackageArchiveCacheMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageLockfileDependencyMetadata {
    pub package_path: String,
    pub source_kind: PackageLockfileDependencySourceKind,
    pub source: String,
    pub hash_kind: String,
    pub hash: String,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageArchiveCacheMetadata {
    pub package_path: String,
    pub archive_path: PathBuf,
    pub cache_root: PathBuf,
    pub expected_content_hash: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageLockfileDependencySourceKind {
    Path,
    Archive,
}

impl PackageLockfileDependencySourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Archive => "archive",
        }
    }
}

pub fn load_flattened_program_from_entry(path: &Path) -> Result<Program, Vec<Diagnostic>> {
    Ok(load_flattened_from_entry(path)?.program)
}

pub fn import_paths_from_entry(path: &Path) -> Result<Vec<String>, Vec<Diagnostic>> {
    let (entry_program, manifest) = parse_entry_program(path)?;
    if entry_program.package.is_none() {
        return Ok(Vec::new());
    }

    let mut loader = PackageLoader::new(path.to_path_buf(), entry_program, manifest);
    loader.load_entry_import_paths()
}

pub fn entry_package_path_from_entry(path: &Path) -> Result<Option<String>, Vec<Diagnostic>> {
    let (entry_program, _) = parse_entry_program(path)?;
    Ok(entry_program.package.map(|package| package.path))
}

pub fn default_build_artifact_root_from_entry(path: &Path) -> Result<PathBuf, Vec<Diagnostic>> {
    let project_root = discover_manifest(path)?
        .map(|manifest| manifest.root)
        .unwrap_or_else(|| {
            path.parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        });
    Ok(project_root.join(".muga").join("build"))
}

pub fn project_manifest_metadata_from_entry(
    path: &Path,
) -> Result<Option<ProjectManifestMetadata>, Vec<Diagnostic>> {
    let Some(manifest) = discover_manifest(path)? else {
        return Ok(None);
    };
    Ok(Some(project_manifest_metadata(&manifest)))
}

pub fn project_manifest_metadata_from_root(
    root: &Path,
) -> Result<ProjectManifestMetadata, Vec<Diagnostic>> {
    let manifest_path = root.join("muga.toml");
    if !manifest_path.is_file() {
        return Err(vec![Diagnostic::new(
            "PK014",
            format!(
                "project root `{}` must contain a muga.toml manifest",
                root.display()
            ),
            Span::default(),
        )]);
    }
    parse_manifest(&manifest_path).map(|manifest| project_manifest_metadata(&manifest))
}

pub fn write_lockfile_from_entry(path: &Path) -> Result<Option<PathBuf>, Vec<Diagnostic>> {
    let Some(manifest) = discover_manifest(path)? else {
        return Ok(None);
    };
    let text = manifest_lockfile_text(&manifest)?;
    let lockfile_path = manifest.root.join("muga.lock");
    match fs::read_to_string(&lockfile_path) {
        Ok(existing) if existing == text => return Ok(Some(lockfile_path)),
        Ok(existing) => validate_existing_lockfile(&existing, &lockfile_path)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(vec![Diagnostic::new(
                "PK025",
                format!(
                    "failed to read package lockfile `{}`: {error}",
                    lockfile_path.display()
                ),
                Span::default(),
            )]);
        }
    }
    fs::write(&lockfile_path, text).map_err(|error| {
        vec![Diagnostic::new(
            "PK025",
            format!(
                "failed to write package lockfile `{}`: {error}",
                lockfile_path.display()
            ),
            Span::default(),
        )]
    })?;
    Ok(Some(lockfile_path))
}

pub fn lockfile_metadata_from_entry(
    path: &Path,
) -> Result<Option<PackageLockfileMetadata>, Vec<Diagnostic>> {
    let Some(manifest) = discover_manifest(path)? else {
        return Ok(None);
    };
    let text = manifest_lockfile_text(&manifest)?;
    let content_hash = format!("sha256:{}", sha256_hex(text.as_bytes()));
    let dependencies = manifest_lockfile_dependency_metadata(&manifest)?;
    let archive_caches = manifest_archive_cache_metadata(&manifest);
    Ok(Some(PackageLockfileMetadata {
        path: manifest.root.join("muga.lock"),
        text,
        content_hash,
        dependencies,
        archive_caches,
    }))
}

pub fn validate_lockfile_text(text: &str, path: &Path) -> Result<(), Vec<Diagnostic>> {
    validate_existing_lockfile(text, path)
}

pub fn content_hash_for_bytes(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256_hex(bytes))
}

pub fn archive_dependency_cache_content_hash(cache_root: &Path) -> Result<String, Vec<Diagnostic>> {
    package_archive_dependency_cache_content_hash(cache_root)
}

pub fn package_content_hash_from_entry(path: &Path) -> Result<Option<String>, Vec<Diagnostic>> {
    let Some(manifest) = discover_manifest(path)? else {
        return Ok(None);
    };
    let input = package_source_content_input(
        &manifest.root,
        &manifest.source_root,
        manifest.resource_root.as_deref(),
        "package content hash",
    )?;
    Ok(Some(format!("sha256:{}", sha256_hex(&input))))
}

pub fn write_package_archive_from_entry(
    path: &Path,
    archive_root: &Path,
) -> Result<PackageArchiveOutput, Vec<Diagnostic>> {
    let Some(manifest) = discover_manifest(path)? else {
        return Err(vec![
            Diagnostic::new(
                "PK027",
                "package archive emission requires a muga.toml manifest",
                Span::default(),
            )
            .with_suggestion("run `emit-package-archive` from a manifest project entrypoint"),
        ]);
    };
    validate_package_archive_emission_manifest_roots(&manifest)?;
    validate_package_archive_output_location(archive_root, &manifest)?;
    let archive_bytes = package_source_content_input(
        &manifest.root,
        &manifest.source_root,
        manifest.resource_root.as_deref(),
        "package archive",
    )?;
    let content_hash = format!("sha256:{}", sha256_hex(&archive_bytes));
    let path = package_archive_file_path(archive_root, &manifest.name, &content_hash);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            vec![Diagnostic::new(
                "PK027",
                format!(
                    "failed to create package archive directory {}: {error}",
                    parent.display()
                ),
                Span::default(),
            )]
        })?;
    }
    fs::write(&path, archive_bytes).map_err(|error| {
        vec![Diagnostic::new(
            "PK027",
            format!(
                "failed to write package archive `{}`: {error}",
                path.display()
            ),
            Span::default(),
        )]
    })?;
    Ok(PackageArchiveOutput {
        path,
        content_hash,
        package_name: manifest.name,
    })
}

pub fn read_package_archive(
    path: &Path,
    expected_content_hash: Option<&str>,
) -> Result<PackageArchive, Vec<Diagnostic>> {
    let bytes = fs::read(path).map_err(|error| {
        vec![package_archive_validation_diagnostic(format!(
            "failed to read package archive `{}`: {error}",
            path.display()
        ))]
    })?;
    validate_package_archive_bytes(&bytes, expected_content_hash)
}

pub fn verify_package_archive(path: &Path) -> Result<PackageArchiveVerifyOutput, Vec<Diagnostic>> {
    let expected_content_hash = expected_package_archive_hash_from_path(path)?;
    verify_package_archive_with_expected_hash(path, &expected_content_hash)
}

pub fn verify_package_archive_with_expected_hash(
    path: &Path,
    expected_content_hash: &str,
) -> Result<PackageArchiveVerifyOutput, Vec<Diagnostic>> {
    let archive = read_package_archive(path, Some(expected_content_hash))?;
    Ok(PackageArchiveVerifyOutput {
        path: path.to_path_buf(),
        content_hash: archive.content_hash,
        manifest: archive.manifest.path,
        sources: archive
            .sources
            .into_iter()
            .map(|entry| entry.path)
            .collect(),
        resources: archive
            .resources
            .into_iter()
            .map(|entry| entry.path)
            .collect(),
    })
}

pub fn validate_package_archive_bytes(
    bytes: &[u8],
    expected_content_hash: Option<&str>,
) -> Result<PackageArchive, Vec<Diagnostic>> {
    let content_hash = format!("sha256:{}", sha256_hex(bytes));
    if let Some(expected) = expected_content_hash {
        validate_expected_package_archive_hash(expected)?;
        if expected != content_hash {
            return Err(vec![
                package_archive_validation_diagnostic(format!(
                    "package archive hash mismatch: expected `{expected}`, got `{content_hash}`"
                ))
                .with_suggestion("fetch or emit the package archive again"),
            ]);
        }
    }

    let mut parser = PackageArchiveParser::new(bytes);
    let mut manifest = None;
    let mut sources = Vec::new();
    let mut resources = Vec::new();
    let mut seen_source_paths = HashSet::new();
    let mut seen_resource_paths = HashSet::new();
    let mut previous_source_path: Option<String> = None;
    let mut previous_resource_path: Option<String> = None;

    while let Some(raw_entry) = parser.next_entry()? {
        match raw_entry.kind.as_str() {
            "manifest" => {
                if manifest.is_some() || !sources.is_empty() || !resources.is_empty() {
                    return Err(vec![package_archive_validation_diagnostic(
                        "package archive must contain exactly one leading manifest entry",
                    )]);
                }
                if raw_entry.path != "muga.toml" {
                    return Err(vec![package_archive_validation_diagnostic(format!(
                        "package archive manifest entry must be `muga.toml`, got `{}`",
                        raw_entry.path
                    ))]);
                }
                let contents =
                    package_archive_utf8_entry_contents(&raw_entry.path, &raw_entry.contents)?;
                manifest = Some(PackageArchiveEntry {
                    path: raw_entry.path,
                    contents,
                });
            }
            "file" => {
                if manifest.is_none() {
                    return Err(vec![package_archive_validation_diagnostic(
                        "package archive file entries must follow the manifest entry",
                    )]);
                }
                if !resources.is_empty() {
                    return Err(vec![package_archive_validation_diagnostic(
                        "package archive source entries must precede resource entries",
                    )]);
                }
                validate_package_archive_source_path(&raw_entry.path)?;
                if !seen_source_paths.insert(raw_entry.path.clone()) {
                    return Err(vec![package_archive_validation_diagnostic(format!(
                        "package archive contains duplicate source entry `{}`",
                        raw_entry.path
                    ))]);
                }
                if let Some(previous) = &previous_source_path
                    && previous >= &raw_entry.path
                {
                    return Err(vec![package_archive_validation_diagnostic(format!(
                        "package archive source entries must be sorted: `{}` appears after `{previous}`",
                        raw_entry.path
                    ))]);
                }
                previous_source_path = Some(raw_entry.path.clone());
                let contents =
                    package_archive_utf8_entry_contents(&raw_entry.path, &raw_entry.contents)?;
                sources.push(PackageArchiveEntry {
                    path: raw_entry.path,
                    contents,
                });
            }
            "resource" => {
                if manifest.is_none() {
                    return Err(vec![package_archive_validation_diagnostic(
                        "package archive resource entries must follow the manifest entry",
                    )]);
                }
                validate_package_archive_resource_path(&raw_entry.path)?;
                if !seen_resource_paths.insert(raw_entry.path.clone()) {
                    return Err(vec![package_archive_validation_diagnostic(format!(
                        "package archive contains duplicate resource entry `{}`",
                        raw_entry.path
                    ))]);
                }
                if let Some(previous) = &previous_resource_path
                    && previous >= &raw_entry.path
                {
                    return Err(vec![package_archive_validation_diagnostic(format!(
                        "package archive resource entries must be sorted: `{}` appears after `{previous}`",
                        raw_entry.path
                    ))]);
                }
                previous_resource_path = Some(raw_entry.path.clone());
                resources.push(PackageArchiveResourceEntry {
                    path: raw_entry.path,
                    contents: raw_entry.contents,
                });
            }
            other => {
                return Err(vec![package_archive_validation_diagnostic(format!(
                    "unknown package archive entry kind `{other}`"
                ))]);
            }
        }
    }

    let Some(manifest) = manifest else {
        return Err(vec![package_archive_validation_diagnostic(
            "package archive is missing manifest entry `muga.toml`",
        )]);
    };
    package_archive_manifest_source_dir_with_diagnostic(
        &manifest.contents,
        package_archive_validation_diagnostic,
    )?;
    let declared_resource_dir = package_archive_manifest_resource_dir_with_diagnostic(
        &manifest.contents,
        package_archive_validation_diagnostic,
    )?;
    if !resources.is_empty() && declared_resource_dir.is_none() {
        return Err(vec![package_archive_validation_diagnostic(
            "package archive resource entries require [package] resources",
        )]);
    }

    Ok(PackageArchive {
        content_hash,
        manifest,
        sources,
        resources,
    })
}

pub fn materialize_package_archive(
    path: &Path,
    expected_content_hash: Option<&str>,
    destination_root: &Path,
) -> Result<PackageArchiveMaterializationOutput, Vec<Diagnostic>> {
    let archive = read_package_archive(path, expected_content_hash)?;
    materialize_validated_package_archive(&archive, destination_root)
}

pub fn unpack_package_archive(
    path: &Path,
    destination_root: &Path,
) -> Result<PackageArchiveMaterializationOutput, Vec<Diagnostic>> {
    let expected_content_hash = expected_package_archive_hash_from_path(path)?;
    unpack_package_archive_with_expected_hash(path, &expected_content_hash, destination_root)
}

pub fn unpack_package_archive_with_expected_hash(
    path: &Path,
    expected_content_hash: &str,
    destination_root: &Path,
) -> Result<PackageArchiveMaterializationOutput, Vec<Diagnostic>> {
    materialize_package_archive(path, Some(expected_content_hash), destination_root)
}

pub fn materialize_package_archive_bytes(
    bytes: &[u8],
    expected_content_hash: Option<&str>,
    destination_root: &Path,
) -> Result<PackageArchiveMaterializationOutput, Vec<Diagnostic>> {
    let archive = validate_package_archive_bytes(bytes, expected_content_hash)?;
    materialize_validated_package_archive(&archive, destination_root)
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

pub fn load_flattened_from_entry(path: &Path) -> Result<LoadedFlattenedProgram, Vec<Diagnostic>> {
    let (entry_program, manifest) = parse_entry_program(path)?;
    if entry_program.package.is_none() {
        return Ok(LoadedFlattenedProgram {
            program: entry_program,
            package_graph: PackageSymbolGraph::default(),
            package_exports: PackageExportGraph::default(),
        });
    }

    let mut loader = PackageLoader::new(path.to_path_buf(), entry_program, manifest);
    loader.load_and_flatten()
}

pub fn load_package_graph_from_entry(path: &Path) -> Result<LoadedPackageGraph, Vec<Diagnostic>> {
    let (entry_program, manifest) = parse_entry_program(path)?;
    if entry_program.package.is_none() {
        return Err(vec![
            Diagnostic::new(
                "PK001",
                "package graph loading requires a package-mode entrypoint",
                Span::default(),
            )
            .with_suggestion("use a file that starts with `package`, or check scripts directly"),
        ]);
    }

    let mut loader = PackageLoader::new(path.to_path_buf(), entry_program, manifest);
    loader.load_unflattened_graph()
}

pub fn load_package_graph_from_entry_against_interfaces(
    path: &Path,
    interfaces: &PackageInterfaceGraph,
    interface_symbols: &SymbolTable,
) -> Result<LoadedPackageGraph, Vec<Diagnostic>> {
    let (entry_program, manifest) = parse_entry_program(path)?;
    if entry_program.package.is_none() {
        return Err(vec![
            Diagnostic::new(
                "PK001",
                "package graph loading requires a package-mode entrypoint",
                Span::default(),
            )
            .with_suggestion("use a file that starts with `package`, or check scripts directly"),
        ]);
    }

    let mut loader = PackageLoader::new(path.to_path_buf(), entry_program, manifest);
    loader.load_unflattened_graph_against_interfaces(interfaces, interface_symbols)
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
    let mut entry_program = if let Some(manifest) = &manifest {
        let inferred_package = infer_manifest_package_path(path, manifest)?;
        let program =
            crate::parser::parse_inferred_package(entry_tokens, inferred_package.clone())?;
        if let Some(package) = &program.package
            && package.path != inferred_package
        {
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
        program
    } else {
        crate::parser::parse(entry_tokens)?
    };
    if manifest.is_none()
        && let Some(package) = &entry_program.package
        && is_reserved_standard_package_path(&package.path)
    {
        return Err(vec![reserved_standard_package_diagnostic(
            &package.path,
            package.span,
        )]);
    }
    attach_doc_comments_from_source(&mut entry_program, &entry_source);
    Ok((entry_program, manifest))
}

fn attach_doc_comments_from_source(program: &mut Program, source: &str) {
    let lines = source.lines().collect::<Vec<_>>();
    for statement in &mut program.statements {
        let doc_comments = doc_comments_before_line(&lines, statement.span().start.line);
        match statement {
            Stmt::RecordDecl(record) => record.doc_comments = doc_comments,
            Stmt::EnumDecl(enumeration) => enumeration.doc_comments = doc_comments,
            Stmt::OpaqueTypeDecl(opaque) => opaque.doc_comments = doc_comments,
            Stmt::FuncDecl(function) => function.doc_comments = doc_comments,
            _ => {}
        }
    }
}

fn doc_comments_before_line(lines: &[&str], line: usize) -> Vec<String> {
    let mut docs = Vec::new();
    let mut index = line.saturating_sub(1);
    while index > 0 {
        let text = lines[index - 1].trim_start();
        if let Some(comment) = public_doc_comment_text(text) {
            docs.push(comment.to_string());
            index -= 1;
            continue;
        }
        if text.starts_with('@') {
            index -= 1;
            continue;
        }
        break;
    }
    docs.reverse();
    docs
}

fn public_doc_comment_text(text: &str) -> Option<&str> {
    let comment = text.strip_prefix("///")?;
    if comment.starts_with('/') {
        return None;
    }
    Some(comment.strip_prefix(' ').unwrap_or(comment))
}

#[derive(Clone, Debug)]
pub struct LoadedFlattenedProgram {
    pub program: Program,
    pub package_graph: PackageSymbolGraph,
    pub package_exports: PackageExportGraph,
}

#[derive(Clone, Debug)]
pub struct LoadedPackageGraph {
    pub packages: Vec<LoadedPackage>,
    pub package_graph: PackageSymbolGraph,
    pub package_exports: PackageExportGraph,
    pub interfaces: Option<LoadedPackageInterfaces>,
    pub entry_package: PackageId,
    pub entry_module: ModuleId,
}

impl LoadedPackageGraph {
    pub fn is_loaded_interface_package_path(&self, path: &str) -> bool {
        let Some(interfaces) = &self.interfaces else {
            return false;
        };
        if self
            .package_graph
            .package(self.entry_package)
            .is_some_and(|package| package.path == path)
        {
            return false;
        }
        interfaces.graph.package_by_path(path).is_some()
    }

    pub fn entry_program(&self) -> Option<&Program> {
        let entry_package = self.package_graph.package(self.entry_package)?;
        let entry_module = self.package_graph.module(self.entry_module)?;
        self.packages
            .iter()
            .find(|package| package.path == entry_package.path)?
            .files
            .iter()
            .find(|file| file.module_path == entry_module.path)
            .map(|file| &file.program)
    }
}

#[derive(Clone, Debug)]
pub struct LoadedPackageInterfaces {
    pub graph: PackageInterfaceGraph,
    pub symbols: SymbolTable,
}

#[derive(Clone, Debug)]
pub struct LoadedPackage {
    pub path: String,
    pub files: Vec<LoadedPackageFile>,
}

#[derive(Clone, Debug)]
pub struct LoadedPackageFile {
    pub path: Option<PathBuf>,
    pub module_path: String,
    pub source: String,
    pub program: Program,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackageItemKind {
    Record,
    Enum,
    OpaqueType,
    Function,
}

pub fn validate_loaded_package_graph(loaded: &LoadedPackageGraph) -> Vec<Diagnostic> {
    let mut checker = PackageAwareChecker::new(loaded);
    checker.check();
    checker.diagnostics
}

struct PackageAwareChecker<'a> {
    loaded: &'a LoadedPackageGraph,
    diagnostics: Vec<Diagnostic>,
    imports: HashMap<String, String>,
    current_package: PackageId,
    current_module: ModuleId,
    scopes: Vec<HashSet<String>>,
}

impl<'a> PackageAwareChecker<'a> {
    fn new(loaded: &'a LoadedPackageGraph) -> Self {
        Self {
            loaded,
            diagnostics: Vec::new(),
            imports: HashMap::new(),
            current_package: PackageId::new(0),
            current_module: ModuleId::new(0),
            scopes: Vec::new(),
        }
    }

    fn check(&mut self) {
        for package in &self.loaded.packages {
            let Some(package_id) = self.loaded.package_graph.package_id(&package.path) else {
                continue;
            };
            for file in &package.files {
                let Some(module_id) = self
                    .loaded
                    .package_graph
                    .module_id(package_id, &file.module_path)
                else {
                    continue;
                };
                self.current_package = package_id;
                self.current_module = module_id;
                self.imports = file_import_aliases(&file.program.imports, &mut self.diagnostics);
                for statement in &file.program.statements {
                    self.check_top_level_stmt(statement);
                }
            }
        }
    }

    fn check_top_level_stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::RecordDecl(record) => {
                if matches!(record.visibility, Visibility::Public | Visibility::Package) {
                    for field in &record.fields {
                        self.validate_visible_type_with_params(
                            &field.type_name,
                            record.visibility,
                            field.span,
                            &record.type_params,
                        );
                    }
                }
                self.scan_record_decl(record);
            }
            Stmt::EnumDecl(enumeration) => {
                if matches!(
                    enumeration.visibility,
                    Visibility::Public | Visibility::Package
                ) {
                    for variant in &enumeration.variants {
                        if let Some(payload) = &variant.payload {
                            self.validate_visible_type_with_params(
                                payload,
                                enumeration.visibility,
                                variant.span,
                                &enumeration.type_params,
                            );
                        }
                    }
                }
                self.scan_enum_decl(enumeration);
            }
            Stmt::FuncDecl(function) => {
                if function.visibility == Visibility::Public {
                    let has_full_signature = function
                        .params
                        .iter()
                        .all(|param| param.type_name.is_some())
                        && function.return_type.is_some();
                    if !has_full_signature {
                        self.diagnostics.push(
                            Diagnostic::new(
                                "PK011",
                                "public functions must annotate every parameter and the return type",
                                function.span,
                            )
                            .with_suggestion(
                                "add parameter type annotations and an explicit return type",
                            ),
                        );
                    }
                }
                if matches!(
                    function.visibility,
                    Visibility::Public | Visibility::Package
                ) {
                    for param in &function.params {
                        if let Some(type_name) = &param.type_name {
                            self.validate_visible_type_with_params(
                                type_name,
                                function.visibility,
                                param.span,
                                &function.type_params,
                            );
                        }
                    }
                    if let Some(type_name) = &function.return_type {
                        self.validate_visible_type_with_params(
                            type_name,
                            function.visibility,
                            function.span,
                            &function.type_params,
                        );
                    }
                }
                self.scan_func_decl(function);
            }
            Stmt::OpaqueTypeDecl(_) => {}
            _ => self.scan_stmt(statement),
        }
    }

    fn scan_stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Assign(stmt) => {
                if let Some(type_name) = &stmt.type_name {
                    self.scan_type_expr(type_name, stmt.span);
                }
                self.scan_expr(&stmt.value);
                self.insert_local(stmt.name.clone());
            }
            Stmt::RecordDecl(record) => self.scan_record_decl(record),
            Stmt::EnumDecl(enumeration) => self.scan_enum_decl(enumeration),
            Stmt::OpaqueTypeDecl(_) => {}
            Stmt::FuncDecl(function) => self.scan_func_decl(function),
            Stmt::If(stmt) => {
                self.scan_expr(&stmt.condition);
                self.scan_block(&stmt.then_branch);
                if let Some(else_branch) = &stmt.else_branch {
                    self.scan_block(else_branch);
                }
            }
            Stmt::While(stmt) => {
                self.scan_expr(&stmt.condition);
                self.scan_block(&stmt.body);
            }
            Stmt::For(stmt) => {
                self.scan_expr(&stmt.iterable);
                self.push_scope();
                self.insert_local(stmt.item.clone());
                self.predeclare_nested_functions(&stmt.body.statements);
                for statement in &stmt.body.statements {
                    self.scan_stmt(statement);
                }
                self.pop_scope();
            }
            Stmt::Using(stmt) => {
                self.scan_expr(&stmt.value);
                self.push_scope();
                self.insert_local(stmt.name.clone());
                self.predeclare_nested_functions(&stmt.body.statements);
                for statement in &stmt.body.statements {
                    self.scan_stmt(statement);
                }
                self.pop_scope();
            }
            Stmt::Break(_) | Stmt::Continue(_) => {}
            Stmt::Return(stmt) => self.scan_expr(&stmt.value),
            Stmt::Expr(stmt) => self.scan_expr(&stmt.expr),
        }
    }

    fn scan_record_decl(&mut self, record: &RecordDecl) {
        for field in &record.fields {
            self.scan_type_expr(&field.type_name, field.span);
        }
    }

    fn scan_enum_decl(&mut self, enumeration: &EnumDecl) {
        for variant in &enumeration.variants {
            if let Some(payload) = &variant.payload {
                self.scan_type_expr(payload, variant.span);
            }
        }
    }

    fn scan_func_decl(&mut self, function: &FuncDecl) {
        for param in &function.params {
            if let Some(type_name) = &param.type_name {
                self.scan_type_expr(type_name, param.span);
            }
        }
        if let Some(type_name) = &function.return_type {
            self.scan_type_expr(type_name, function.span);
        }

        self.push_scope();
        for param in &function.params {
            self.insert_local(param.name.clone());
        }
        self.predeclare_nested_functions(&function.body.statements);
        for statement in &function.body.statements {
            self.scan_stmt(statement);
        }
        self.scan_expr(&function.body.expr);
        self.pop_scope();
    }

    fn scan_block(&mut self, block: &Block) {
        self.push_scope();
        self.predeclare_nested_functions(&block.statements);
        for statement in &block.statements {
            self.scan_stmt(statement);
        }
        self.pop_scope();
    }

    fn scan_value_block(&mut self, block: &ValueBlock) {
        self.push_scope();
        self.predeclare_nested_functions(&block.statements);
        for statement in &block.statements {
            self.scan_stmt(statement);
        }
        self.scan_expr(&block.expr);
        self.pop_scope();
    }

    fn scan_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Int(_) | Expr::Bool(_) | Expr::String(_) | Expr::Unit(_) => {}
            Expr::Ident(expr) => {
                if !self.lookup_local(&expr.name) {
                    self.check_value_name(&expr.name, expr.span);
                }
            }
            Expr::ListLit(expr) => {
                for item in &expr.items {
                    self.scan_expr(item);
                }
            }
            Expr::Index(expr) => {
                self.scan_expr(&expr.base);
                self.scan_expr(&expr.index);
            }
            Expr::RecordLit(expr) => {
                self.check_type_name(&expr.type_name, expr.span);
                for field in &expr.fields {
                    self.scan_expr(&field.value);
                }
            }
            Expr::Field(expr) => self.scan_expr(&expr.base),
            Expr::RecordUpdate(expr) => {
                self.scan_expr(&expr.base);
                for field in &expr.fields {
                    self.scan_expr(&field.value);
                }
            }
            Expr::Unary(expr) => self.scan_expr(&expr.expr),
            Expr::Binary(expr) => {
                self.scan_expr(&expr.left);
                self.scan_expr(&expr.right);
            }
            Expr::Call(expr) => {
                self.scan_expr(&expr.callee);
                for arg in &expr.args {
                    self.scan_expr(arg);
                }
            }
            Expr::Try(expr) => self.scan_expr(&expr.expr),
            Expr::If(expr) => {
                self.scan_expr(&expr.condition);
                self.scan_value_block(&expr.then_branch);
                self.scan_value_block(&expr.else_branch);
            }
            Expr::Match(expr) => {
                self.scan_expr(&expr.value);
                for arm in &expr.arms {
                    let MatchPattern::Variant(pattern) = &arm.pattern;
                    self.check_type_name(&pattern.enum_name, pattern.span);
                    self.push_scope();
                    if let EnumVariantPatternPayload::Binding(binding) = &pattern.payload {
                        self.insert_local(binding.clone());
                    }
                    self.scan_expr(&arm.value);
                    self.pop_scope();
                }
            }
            Expr::Fn(expr) => {
                for param in &expr.params {
                    if let Some(type_name) = &param.type_name {
                        self.scan_type_expr(type_name, param.span);
                    }
                }
                if let Some(type_name) = &expr.return_type {
                    self.scan_type_expr(type_name, expr.span);
                }
                self.push_scope();
                for param in &expr.params {
                    self.insert_local(param.name.clone());
                }
                self.scan_value_block(&expr.body);
                self.pop_scope();
            }
            Expr::Group(expr) => self.scan_value_block(&expr.body),
            Expr::Spawn(expr) => self.scan_expr(&expr.expr),
        }
    }

    fn scan_type_expr(&mut self, type_expr: &TypeExpr, span: Span) {
        match type_expr {
            TypeExpr::Int | TypeExpr::Bool | TypeExpr::String | TypeExpr::Unit => {}
            TypeExpr::Named(name) => self.check_type_name(name, span),
            TypeExpr::Generic(generic) => {
                if !is_known_generic_type_name(&generic.name) {
                    self.check_type_name(&generic.name, span);
                }
                for arg in &generic.args {
                    self.scan_type_expr(arg, span);
                }
            }
            TypeExpr::Function(function) => {
                for param in &function.params {
                    self.scan_type_expr(param, span);
                }
                self.scan_type_expr(&function.ret, span);
            }
        }
    }

    fn validate_visible_type_with_params(
        &mut self,
        type_expr: &TypeExpr,
        api_visibility: Visibility,
        span: Span,
        type_params: &[String],
    ) {
        match type_expr {
            TypeExpr::Int | TypeExpr::Bool | TypeExpr::String | TypeExpr::Unit => {}
            TypeExpr::Named(name) => {
                if !type_params.iter().any(|param| param == name) {
                    self.validate_visible_type_name(name, api_visibility, span);
                }
            }
            TypeExpr::Generic(generic) => {
                if !is_known_generic_type_name(&generic.name)
                    && !type_params.iter().any(|param| param == &generic.name)
                {
                    self.validate_visible_type_name(&generic.name, api_visibility, span);
                }
                for arg in &generic.args {
                    self.validate_visible_type_with_params(arg, api_visibility, span, type_params);
                }
            }
            TypeExpr::Function(function) => {
                for param in &function.params {
                    self.validate_visible_type_with_params(
                        param,
                        api_visibility,
                        span,
                        type_params,
                    );
                }
                self.validate_visible_type_with_params(
                    &function.ret,
                    api_visibility,
                    span,
                    type_params,
                );
            }
        }
    }

    fn validate_visible_type_name(&mut self, name: &str, api_visibility: Visibility, span: Span) {
        if let Some((alias, item)) = split_qualified_name(name) {
            let _ = self.resolve_imported_type_item(alias, item, span);
            return;
        }

        for kind in [
            PackageItemKind::Record,
            PackageItemKind::Enum,
            PackageItemKind::OpaqueType,
        ] {
            if let Some(item) = self.visible_same_package_item(name, kind)
                && !visibility_can_expose(item.visibility, api_visibility)
            {
                let api = visibility_label(api_visibility);
                let item_visibility = visibility_label(item.visibility);
                self.diagnostics.push(
                    Diagnostic::new(
                        "PK012",
                        format!(
                            "{api} API may not expose {item_visibility} {} `{name}`",
                            package_item_kind_label(kind)
                        ),
                        span,
                    )
                    .with_related(
                        format!(
                            "{} `{name}` is declared here",
                            package_item_kind_label(kind)
                        ),
                        item.span,
                    ),
                );
                return;
            }
        }
    }

    fn check_type_name(&mut self, name: &str, span: Span) {
        if let Some(diagnostic) = fully_qualified_std_type_diagnostic(name, span) {
            self.diagnostics.push(diagnostic);
            return;
        }
        if let Some((alias, item)) = split_qualified_name(name) {
            let _ = self.resolve_imported_type_item(alias, item, span);
            return;
        }
        if self
            .visible_same_package_item(name, PackageItemKind::Record)
            .is_some()
            || self
                .visible_same_package_item(name, PackageItemKind::Enum)
                .is_some()
            || self
                .visible_same_package_item(name, PackageItemKind::OpaqueType)
                .is_some()
        {
            return;
        }
        if let Some(item) = self
            .inaccessible_same_package_item(name, PackageItemKind::Record)
            .cloned()
        {
            self.push_inaccessible_same_package_diagnostic(name, &item, "record", span);
        } else if let Some(item) = self
            .inaccessible_same_package_item(name, PackageItemKind::Enum)
            .cloned()
        {
            self.push_inaccessible_same_package_diagnostic(name, &item, "enum", span);
        } else if let Some(item) = self
            .inaccessible_same_package_item(name, PackageItemKind::OpaqueType)
            .cloned()
        {
            self.push_inaccessible_same_package_diagnostic(name, &item, "opaque type", span);
        }
    }

    fn check_value_name(&mut self, name: &str, span: Span) {
        if name.contains("::") && is_builtin_name(name) {
            return;
        }
        if let Some((enum_name, _variant_name)) = split_variant_name(name) {
            if let Some((alias, item)) = split_qualified_name(enum_name) {
                let _ = self.resolve_imported_item(alias, item, PackageItemKind::Enum, span);
                return;
            }
            if self
                .visible_same_package_item(enum_name, PackageItemKind::Enum)
                .is_some()
            {
                return;
            }
            if let Some(item) = self
                .inaccessible_same_package_item(enum_name, PackageItemKind::Enum)
                .cloned()
            {
                self.push_inaccessible_same_package_diagnostic(enum_name, &item, "enum", span);
                return;
            }
        }
        if let Some((alias, item)) = split_qualified_name(name) {
            let _ = self.resolve_imported_item(alias, item, PackageItemKind::Function, span);
            return;
        }
        if self
            .visible_same_package_item(name, PackageItemKind::Function)
            .is_some()
        {
            return;
        }
        if let Some(item) = self
            .inaccessible_same_package_item(name, PackageItemKind::Function)
            .cloned()
        {
            self.push_inaccessible_same_package_diagnostic(name, &item, "function", span);
        }
    }

    fn resolve_imported_type_item(&mut self, alias: &str, item: &str, span: Span) -> bool {
        let Some(package_id) = self.imported_package(alias, span) else {
            return false;
        };
        if self
            .loaded
            .package_exports
            .record_by_name(package_id, item)
            .is_some()
            || self
                .loaded
                .package_exports
                .enum_by_name(package_id, item)
                .is_some()
            || self
                .loaded
                .package_exports
                .opaque_type_by_name(package_id, item)
                .is_some()
        {
            return true;
        }
        if let Some(record) = self
            .package_item(package_id, item, PackageItemKind::Record)
            .cloned()
        {
            self.push_missing_export_diagnostic(&record, "record", span);
        } else if let Some(enumeration) = self
            .package_item(package_id, item, PackageItemKind::Enum)
            .cloned()
        {
            self.push_missing_export_diagnostic(&enumeration, "enum", span);
        } else if let Some(opaque) = self
            .package_item(package_id, item, PackageItemKind::OpaqueType)
            .cloned()
        {
            self.push_missing_export_diagnostic(&opaque, "opaque type", span);
        } else {
            self.push_missing_export_name_diagnostic(package_id, item, "type", span);
        }
        false
    }

    fn resolve_imported_item(
        &mut self,
        alias: &str,
        item: &str,
        kind: PackageItemKind,
        span: Span,
    ) -> bool {
        let Some(package_id) = self.imported_package(alias, span) else {
            return false;
        };
        let exported = match kind {
            PackageItemKind::Record => self.loaded.package_exports.record_by_name(package_id, item),
            PackageItemKind::Enum => self.loaded.package_exports.enum_by_name(package_id, item),
            PackageItemKind::OpaqueType => self
                .loaded
                .package_exports
                .opaque_type_by_name(package_id, item),
            PackageItemKind::Function => self
                .loaded
                .package_exports
                .function_by_name(package_id, item),
        };
        if exported.is_some() {
            return true;
        }
        if let Some(declaration) = self.package_item(package_id, item, kind).cloned() {
            self.push_missing_export_diagnostic(&declaration, package_item_kind_label(kind), span);
        } else {
            self.push_missing_export_name_diagnostic(
                package_id,
                item,
                package_item_kind_label(kind),
                span,
            );
        }
        false
    }

    fn imported_package(&mut self, alias: &str, span: Span) -> Option<PackageId> {
        let Some(package_path) = self.imports.get(alias).cloned() else {
            self.diagnostics
                .push(unknown_import_alias_diagnostic(alias, span));
            return None;
        };
        let Some(package_id) = self.loaded.package_graph.package_id(&package_path) else {
            self.diagnostics.push(Diagnostic::new(
                "PK010",
                format!("unknown imported package `{package_path}`"),
                span,
            ));
            return None;
        };
        Some(package_id)
    }

    fn visible_same_package_item(
        &self,
        name: &str,
        kind: PackageItemKind,
    ) -> Option<&PackageItemInfo> {
        self.loaded
            .package_graph
            .items
            .iter()
            .find(|item| {
                item.package == self.current_package
                    && item.module == self.current_module
                    && item.name == name
                    && item.kind == kind
            })
            .or_else(|| {
                self.loaded.package_graph.items.iter().find(|item| {
                    item.package == self.current_package
                        && item.name == name
                        && item.kind == kind
                        && matches!(item.visibility, Visibility::Package | Visibility::Public)
                })
            })
    }

    fn inaccessible_same_package_item(
        &self,
        name: &str,
        kind: PackageItemKind,
    ) -> Option<&PackageItemInfo> {
        self.loaded.package_graph.items.iter().find(|item| {
            item.package == self.current_package
                && item.module != self.current_module
                && item.name == name
                && item.kind == kind
                && item.visibility == Visibility::Private
        })
    }

    fn package_item(
        &self,
        package_id: PackageId,
        name: &str,
        kind: PackageItemKind,
    ) -> Option<&PackageItemInfo> {
        self.loaded
            .package_graph
            .items
            .iter()
            .find(|item| item.package == package_id && item.name == name && item.kind == kind)
    }

    fn push_missing_export_diagnostic(
        &mut self,
        declaration: &PackageItemInfo,
        kind: &str,
        span: Span,
    ) {
        let package_path = self
            .loaded
            .package_graph
            .package(declaration.package)
            .map(|package| package.path.as_str())
            .unwrap_or("<unknown>");
        self.diagnostics.push(
            Diagnostic::new(
                "PK010",
                format!(
                    "package `{package_path}` does not export {kind} `{}`",
                    declaration.name
                ),
                span,
            )
            .with_related(
                format!(
                    "{kind} `{}` is declared here but is not public",
                    declaration.name
                ),
                declaration.span,
            )
            .with_suggestion(format!(
                "mark the {kind} declaration as `pub` to export it from the package"
            )),
        );
    }

    fn push_missing_export_name_diagnostic(
        &mut self,
        package_id: PackageId,
        item: &str,
        kind: &str,
        span: Span,
    ) {
        let package_path = self
            .loaded
            .package_graph
            .package(package_id)
            .map(|package| package.path.as_str())
            .unwrap_or("<unknown>");
        self.diagnostics.push(Diagnostic::new(
            "PK010",
            format!("package `{package_path}` does not export {kind} `{item}`"),
            span,
        ));
    }

    fn push_inaccessible_same_package_diagnostic(
        &mut self,
        name: &str,
        item: &PackageItemInfo,
        kind: &str,
        span: Span,
    ) {
        let module = self
            .loaded
            .package_graph
            .module(self.current_module)
            .map(|module| module.path.as_str())
            .unwrap_or("<unknown>");
        let private_module = self
            .loaded
            .package_graph
            .module(item.module)
            .map(|module| module.path.as_str())
            .unwrap_or("<unknown>");
        self.diagnostics.push(
            Diagnostic::new(
                "PK015",
                format!("{kind} `{name}` is not visible from module `{module}`"),
                span,
            )
            .with_related(
                format!("{kind} `{name}` is module-private to `{private_module}`"),
                item.span,
            )
            .with_suggestion("mark the declaration as `pkg` to share it within the package"),
        );
    }

    fn predeclare_nested_functions(&mut self, statements: &[Stmt]) {
        for statement in statements {
            if let Stmt::FuncDecl(function) = statement {
                self.insert_local(function.name.clone());
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

struct ParsedFile {
    path: Option<PathBuf>,
    program: Program,
    module_path: String,
    source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SourceFingerprintFile {
    pub module_path: String,
    pub source: String,
}

#[derive(Clone, Debug)]
struct ProjectManifest {
    root: PathBuf,
    source_root: PathBuf,
    resource_root: Option<PathBuf>,
    name: String,
    direct_dependencies: Vec<String>,
    dependencies: HashMap<String, ProjectDependency>,
}

#[derive(Clone, Debug)]
struct ProjectDependency {
    root: PathBuf,
    source_root: PathBuf,
    resource_root: Option<PathBuf>,
    name: String,
    source: ProjectDependencySource,
    dependencies: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ProjectDependencySource {
    Path,
    Archive {
        archive_path: PathBuf,
        content_hash: String,
    },
}

#[derive(Clone, Debug)]
enum ManifestDependencySource {
    Path(String),
    Archive { archive: String, hash: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedLockfilePackage {
    alias: String,
    dependencies: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ParsedLockfileSource {
    Path(String),
    Archive(String),
}

#[derive(Debug)]
struct LockfilePackageBuilder {
    line: usize,
    alias: Option<String>,
    path: Option<String>,
    source: Option<ParsedLockfileSource>,
    source_hash: Option<String>,
    hash: Option<String>,
    dependencies: Option<Vec<String>>,
}

struct PackageData {
    files: Vec<ParsedFile>,
    records: HashMap<String, Vec<PackageItemDecl>>,
    enums: HashMap<String, Vec<PackageItemDecl>>,
    opaque_types: HashMap<String, Vec<PackageItemDecl>>,
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

    fn load_and_flatten(&mut self) -> Result<LoadedFlattenedProgram, Vec<Diagnostic>> {
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

    fn load_unflattened_graph(&mut self) -> Result<LoadedPackageGraph, Vec<Diagnostic>> {
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
        let packages = self.loaded_packages_from_paths(&package_paths);
        let (entry_package, entry_module) = self.entry_ids(&package_graph);

        Ok(LoadedPackageGraph {
            packages,
            package_graph,
            package_exports,
            interfaces: None,
            entry_package,
            entry_module,
        })
    }

    fn load_unflattened_graph_against_interfaces(
        &mut self,
        interfaces: &PackageInterfaceGraph,
        interface_symbols: &SymbolTable,
    ) -> Result<LoadedPackageGraph, Vec<Diagnostic>> {
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

        if !self.diagnostics.is_empty() {
            return Err(std::mem::take(&mut self.diagnostics));
        }

        let package_paths = self.sorted_package_paths();
        let package_graph = self.build_symbol_graph_against_interfaces(&package_paths, interfaces);
        let package_exports = PackageExportGraph::from_interfaces(interfaces, &package_graph);
        let packages = self.loaded_packages_from_paths(&package_paths);
        let (entry_package, entry_module) = self.entry_ids(&package_graph);

        Ok(LoadedPackageGraph {
            packages,
            package_graph,
            package_exports,
            interfaces: Some(LoadedPackageInterfaces {
                graph: interfaces.clone(),
                symbols: interface_symbols.clone(),
            }),
            entry_package,
            entry_module,
        })
    }

    fn loaded_packages_from_paths(&self, package_paths: &[String]) -> Vec<LoadedPackage> {
        package_paths
            .iter()
            .filter_map(|package_path| {
                let package = self.packages.get(package_path)?;
                Some(LoadedPackage {
                    path: package_path.clone(),
                    files: package
                        .files
                        .iter()
                        .map(|file| LoadedPackageFile {
                            path: file.path.clone(),
                            module_path: file.module_path.clone(),
                            source: file.source.clone(),
                            program: file.program.clone(),
                        })
                        .collect(),
                })
            })
            .collect()
    }

    fn entry_ids(&self, package_graph: &PackageSymbolGraph) -> (PackageId, ModuleId) {
        let package = package_graph
            .package_id(&self.entry_package)
            .expect("entry package should exist in loaded package graph");
        let module_path = module_path_for_file(&self.entry_file);
        let module = package_graph
            .module_id(package, &module_path)
            .expect("entry module should exist in loaded package graph");
        (package, module)
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
    ) -> Result<LoadedFlattenedProgram, Vec<Diagnostic>> {
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
            Ok(LoadedFlattenedProgram {
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
        if let Some(files) = crate::std_package::virtual_package_files(package_path) {
            return self.load_virtual_package_files(package_path, files);
        }
        if is_reserved_standard_package_path(package_path) {
            self.diagnostics.push(reserved_standard_package_diagnostic(
                package_path,
                Span::default(),
            ));
            return Vec::new();
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
                path: Some(file_path),
                program,
                module_path,
                source,
            });
        }
        files
    }

    fn load_virtual_package_files(
        &mut self,
        package_path: &str,
        files: &[crate::std_package::VirtualPackageFile],
    ) -> Vec<ParsedFile> {
        let mut parsed = Vec::with_capacity(files.len());
        for file in files {
            let source = file.source.trim_start().to_string();
            let program = match self.parse_package_file(&source, package_path) {
                Ok(program) => program,
                Err(diagnostics) => {
                    self.diagnostics.extend(diagnostics);
                    continue;
                }
            };
            parsed.push(ParsedFile {
                path: None,
                program,
                module_path: file.module_path.to_string(),
                source,
            });
        }
        parsed
    }

    fn load_package_source_files(&mut self, package_path: &str) -> Vec<SourceFingerprintFile> {
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
            files.push(SourceFingerprintFile {
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
        let mut program = if self.manifest.is_some() {
            crate::parser::parse_inferred_package(tokens, package_path.to_string())?
        } else {
            crate::parser::parse(tokens)?
        };
        attach_doc_comments_from_source(&mut program, source);
        Ok(program)
    }

    fn package_dir(&self, package_path: &str) -> PathBuf {
        if let Some(manifest) = &self.manifest {
            if let Some(path) =
                package_dir_under_root(package_path, &manifest.name, &manifest.source_root)
            {
                return path;
            }
            for dependency in manifest.dependencies.values() {
                if let Some(path) =
                    package_dir_under_root(package_path, &dependency.name, &dependency.source_root)
                {
                    return path;
                }
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
                        Stmt::OpaqueTypeDecl(opaque) => {
                            let id = PackageItemId::new(items.len() as u32);
                            items.push(PackageItemInfo {
                                id,
                                package: package_id,
                                module: module_id,
                                name: opaque.name.clone(),
                                kind: PackageItemKind::OpaqueType,
                                visibility: opaque.visibility,
                                span: opaque.span,
                                mangled_name: mangle_opaque_type_name_for_visibility(
                                    package_path,
                                    &file.module_path,
                                    &opaque.name,
                                    opaque.visibility,
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
                    .chain(interface.opaque_types.iter().map(|opaque| opaque.item))
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
                        Stmt::OpaqueTypeDecl(opaque) => {
                            let id = interface_item_id(
                                package_interface,
                                &opaque.name,
                                PackageItemKind::OpaqueType,
                                opaque.visibility,
                            )
                            .unwrap_or_else(|| allocate_package_item_id(&mut item_slots));
                            insert_package_graph_item(
                                &mut item_slots,
                                PackageItemInfo {
                                    id,
                                    package: package_id,
                                    module: module_id,
                                    name: opaque.name.clone(),
                                    kind: PackageItemKind::OpaqueType,
                                    visibility: opaque.visibility,
                                    span: opaque.span,
                                    mangled_name: mangle_opaque_type_name_for_visibility(
                                        package_path,
                                        &file.module_path,
                                        &opaque.name,
                                        opaque.visibility,
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

        for interface in &interfaces.packages {
            if self.packages.contains_key(&interface.path) {
                continue;
            }
            let package_id = package_ids[&interface.path];
            let module_id = ModuleId::new(modules.len() as u32);
            modules.push(PackageModuleInfo {
                id: module_id,
                package: package_id,
                path: INTERFACE_MODULE.to_string(),
            });
            let imports = interface
                .dependencies
                .iter()
                .filter_map(|dependency| {
                    package_ids
                        .get(dependency)
                        .map(|package| PackageImportInfo {
                            alias: dependency
                                .rsplit("::")
                                .next()
                                .unwrap_or(dependency)
                                .to_string(),
                            package: *package,
                            path: dependency.clone(),
                            span: Span::default(),
                        })
                })
                .collect();
            package_slots[package_id.as_u32() as usize] = Some(PackageInfo {
                id: package_id,
                path: interface.path.clone(),
                modules: vec![module_id],
                imports,
            });
            for record in &interface.records {
                insert_package_graph_item(
                    &mut item_slots,
                    PackageItemInfo {
                        id: record.item,
                        package: package_id,
                        module: module_id,
                        name: record.name.clone(),
                        kind: PackageItemKind::Record,
                        visibility: Visibility::Public,
                        span: record.span,
                        mangled_name: mangle_record_name_for_visibility(
                            &interface.path,
                            INTERFACE_MODULE,
                            &record.name,
                            Visibility::Public,
                        ),
                    },
                    &mut self.diagnostics,
                );
            }
            for enumeration in &interface.enums {
                insert_package_graph_item(
                    &mut item_slots,
                    PackageItemInfo {
                        id: enumeration.item,
                        package: package_id,
                        module: module_id,
                        name: enumeration.name.clone(),
                        kind: PackageItemKind::Enum,
                        visibility: Visibility::Public,
                        span: enumeration.span,
                        mangled_name: mangle_enum_name_for_visibility(
                            &interface.path,
                            INTERFACE_MODULE,
                            &enumeration.name,
                            Visibility::Public,
                        ),
                    },
                    &mut self.diagnostics,
                );
            }
            for opaque in &interface.opaque_types {
                insert_package_graph_item(
                    &mut item_slots,
                    PackageItemInfo {
                        id: opaque.item,
                        package: package_id,
                        module: module_id,
                        name: opaque.name.clone(),
                        kind: PackageItemKind::OpaqueType,
                        visibility: Visibility::Public,
                        span: opaque.span,
                        mangled_name: mangle_opaque_type_name_for_visibility(
                            &interface.path,
                            INTERFACE_MODULE,
                            &opaque.name,
                            Visibility::Public,
                        ),
                    },
                    &mut self.diagnostics,
                );
            }
            for function in &interface.functions {
                insert_package_graph_item(
                    &mut item_slots,
                    PackageItemInfo {
                        id: function.item,
                        package: package_id,
                        module: module_id,
                        name: function.name.clone(),
                        kind: PackageItemKind::Function,
                        visibility: Visibility::Public,
                        span: function.span,
                        mangled_name: mangle_function_name_for_visibility(
                            &interface.path,
                            INTERFACE_MODULE,
                            &function.name,
                            Visibility::Public,
                            &self.entry_package,
                        ),
                    },
                    &mut self.diagnostics,
                );
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
            Stmt::OpaqueTypeDecl(opaque) => {
                Stmt::OpaqueTypeDecl(self.rewrite_opaque_type_decl(opaque))
            }
            Stmt::FuncDecl(func) => Stmt::FuncDecl(self.rewrite_func_decl(func, true)),
            _ => statement.clone(),
        }
    }

    fn rewrite_record_decl(&mut self, record: &RecordDecl) -> RecordDecl {
        if record.visibility == Visibility::Public || record.visibility == Visibility::Package {
            for field in &record.fields {
                self.validate_visible_type_with_params(
                    &field.type_name,
                    record.visibility,
                    field.span,
                    &record.type_params,
                );
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
            attributes: record.attributes.clone(),
            doc_comments: record.doc_comments.clone(),
            type_params: record.type_params.clone(),
            fields: record
                .fields
                .iter()
                .map(|field| RecordFieldDecl {
                    attributes: field.attributes.clone(),
                    name: field.name.clone(),
                    type_name: self.rewrite_type_expr_with_params(
                        &field.type_name,
                        field.span,
                        &record.type_params,
                    ),
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
                    self.validate_visible_type_with_params(
                        payload,
                        enumeration.visibility,
                        variant.span,
                        &enumeration.type_params,
                    );
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
            attributes: enumeration.attributes.clone(),
            doc_comments: enumeration.doc_comments.clone(),
            type_params: enumeration.type_params.clone(),
            variants: enumeration
                .variants
                .iter()
                .map(|variant| EnumVariantDecl {
                    attributes: variant.attributes.clone(),
                    name: variant.name.clone(),
                    payload: variant.payload.as_ref().map(|payload| {
                        self.rewrite_type_expr_with_params(
                            payload,
                            variant.span,
                            &enumeration.type_params,
                        )
                    }),
                    span: variant.span,
                })
                .collect(),
            span: enumeration.span,
        }
    }

    fn rewrite_opaque_type_decl(&mut self, opaque: &OpaqueTypeDecl) -> OpaqueTypeDecl {
        OpaqueTypeDecl {
            id: opaque.id,
            package_item: self.package_item_id(&opaque.name, PackageItemKind::OpaqueType),
            name: mangle_opaque_type_name_for_visibility(
                &self.current_package,
                &self.current_module,
                &opaque.name,
                opaque.visibility,
            ),
            visibility: Visibility::Private,
            doc_comments: opaque.doc_comments.clone(),
            span: opaque.span,
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
                    self.validate_visible_type_with_params(
                        type_name,
                        Visibility::Public,
                        param.span,
                        &func.type_params,
                    );
                }
            }
            if let Some(type_name) = &func.return_type {
                self.validate_visible_type_with_params(
                    type_name,
                    Visibility::Public,
                    func.span,
                    &func.type_params,
                );
            }
        } else if top_level && func.visibility == Visibility::Package {
            for param in &func.params {
                if let Some(type_name) = &param.type_name {
                    self.validate_visible_type_with_params(
                        type_name,
                        Visibility::Package,
                        param.span,
                        &func.type_params,
                    );
                }
            }
            if let Some(type_name) = &func.return_type {
                self.validate_visible_type_with_params(
                    type_name,
                    Visibility::Package,
                    func.span,
                    &func.type_params,
                );
            }
        }

        let mut params = Vec::with_capacity(func.params.len());
        self.push_scope();
        for param in &func.params {
            self.insert_local(param.name.clone());
            params.push(Param {
                name: param.name.clone(),
                type_name: param.type_name.as_ref().map(|type_name| {
                    self.rewrite_type_expr_with_params(type_name, param.span, &func.type_params)
                }),
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
            attributes: func.attributes.clone(),
            doc_comments: func.doc_comments.clone(),
            type_params: func.type_params.clone(),
            params,
            return_type: func.return_type.as_ref().map(|type_name| {
                self.rewrite_type_expr_with_params(type_name, func.span, &func.type_params)
            }),
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
            Stmt::OpaqueTypeDecl(opaque) => {
                Stmt::OpaqueTypeDecl(self.rewrite_opaque_type_decl(opaque))
            }
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
            Stmt::For(stmt) => {
                let iterable = self.rewrite_expr(&stmt.iterable);
                self.push_scope();
                self.insert_local(stmt.item.clone());
                self.predeclare_nested_functions(&stmt.body.statements);
                let body_statements = stmt
                    .body
                    .statements
                    .iter()
                    .map(|statement| self.rewrite_stmt(statement))
                    .collect();
                self.pop_scope();
                Stmt::For(ForStmt {
                    id: stmt.id,
                    item: stmt.item.clone(),
                    item_span: stmt.item_span,
                    iterable,
                    body: Block {
                        statements: body_statements,
                        span: stmt.body.span,
                    },
                    span: stmt.span,
                })
            }
            Stmt::Using(stmt) => {
                let value = self.rewrite_expr(&stmt.value);
                self.push_scope();
                self.insert_local(stmt.name.clone());
                self.predeclare_nested_functions(&stmt.body.statements);
                let body_statements = stmt
                    .body
                    .statements
                    .iter()
                    .map(|statement| self.rewrite_stmt(statement))
                    .collect();
                self.pop_scope();
                Stmt::Using(UsingStmt {
                    id: stmt.id,
                    name: stmt.name.clone(),
                    name_span: stmt.name_span,
                    value,
                    body: Block {
                        statements: body_statements,
                        span: stmt.body.span,
                    },
                    span: stmt.span,
                })
            }
            Stmt::Break(stmt) => Stmt::Break(BreakStmt {
                id: stmt.id,
                span: stmt.span,
            }),
            Stmt::Continue(stmt) => Stmt::Continue(ContinueStmt {
                id: stmt.id,
                span: stmt.span,
            }),
            Stmt::Return(stmt) => Stmt::Return(ReturnStmt {
                id: stmt.id,
                value: self.rewrite_expr(&stmt.value),
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
            terminal_return: block.terminal_return,
            span: block.span,
        }
    }

    fn rewrite_expr(&mut self, expr: &Expr) -> Expr {
        match expr {
            Expr::Int(_) | Expr::Bool(_) | Expr::String(_) | Expr::Unit(_) => expr.clone(),
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
                type_args: expr
                    .type_args
                    .iter()
                    .map(|arg| self.rewrite_type_expr(arg, expr.span))
                    .collect(),
                args: expr.args.iter().map(|arg| self.rewrite_expr(arg)).collect(),
                origin: expr.origin,
                span: expr.span,
            }),
            Expr::Try(expr) => Expr::Try(TryExpr {
                id: expr.id,
                expr: Box::new(self.rewrite_expr(&expr.expr)),
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
            Expr::Group(expr) => Expr::Group(GroupExpr {
                id: expr.id,
                body: self.rewrite_value_block(&expr.body),
                span: expr.span,
            }),
            Expr::Spawn(expr) => Expr::Spawn(SpawnExpr {
                id: expr.id,
                expr: Box::new(self.rewrite_expr(&expr.expr)),
                span: expr.span,
            }),
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
                if let EnumVariantPatternPayload::Binding(binding) = &pattern.payload {
                    self.insert_local(binding.clone());
                }
                let value = self.rewrite_expr(&arm.value);
                self.pop_scope();
                MatchArm {
                    pattern: MatchPattern::Variant(EnumVariantPattern {
                        enum_name: self.rewrite_type_name(&pattern.enum_name, pattern.span),
                        variant_name: pattern.variant_name.clone(),
                        payload: pattern.payload.clone(),
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
        self.rewrite_type_expr_with_params(type_expr, span, &[])
    }

    fn rewrite_type_expr_with_params(
        &mut self,
        type_expr: &TypeExpr,
        span: Span,
        type_params: &[String],
    ) -> TypeExpr {
        match type_expr {
            TypeExpr::Int => TypeExpr::Int,
            TypeExpr::Bool => TypeExpr::Bool,
            TypeExpr::String => TypeExpr::String,
            TypeExpr::Unit => TypeExpr::Unit,
            TypeExpr::Named(name) if type_params.iter().any(|param| param == name) => {
                TypeExpr::Named(name.clone())
            }
            TypeExpr::Named(name) => TypeExpr::Named(self.rewrite_type_name(name, span)),
            TypeExpr::Generic(generic) => TypeExpr::Generic(GenericTypeExpr {
                name: if type_params.iter().any(|param| param == &generic.name) {
                    generic.name.clone()
                } else {
                    self.rewrite_type_name(&generic.name, span)
                },
                args: generic
                    .args
                    .iter()
                    .map(|arg| self.rewrite_type_expr_with_params(arg, span, type_params))
                    .collect(),
            }),
            TypeExpr::Function(function) => TypeExpr::Function(FunctionTypeExpr {
                params: function
                    .params
                    .iter()
                    .map(|param| self.rewrite_type_expr_with_params(param, span, type_params))
                    .collect(),
                ret: Box::new(self.rewrite_type_expr_with_params(&function.ret, span, type_params)),
            }),
        }
    }

    fn rewrite_type_name(&mut self, name: &str, span: Span) -> String {
        if let Some(diagnostic) = fully_qualified_std_type_diagnostic(name, span) {
            self.diagnostics.push(diagnostic);
            return name.to_string();
        }
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
        if let Some(item) = resolve_package_item(
            &self.current_package_data.opaque_types,
            name,
            &self.current_module,
        ) {
            return mangle_opaque_type_name_for_visibility(
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
        if let Some(item) = inaccessible_package_item(&self.current_package_data.opaque_types, name)
        {
            self.diagnostics.push(
                Diagnostic::new(
                    "PK015",
                    format!(
                        "opaque type `{name}` is not visible from module `{}`",
                        self.current_module
                    ),
                    span,
                )
                .with_related(
                    format!(
                        "opaque type `{name}` is module-private to `{}`",
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

    fn validate_visible_type_with_params(
        &mut self,
        type_expr: &TypeExpr,
        api_visibility: Visibility,
        span: Span,
        type_params: &[String],
    ) {
        match type_expr {
            TypeExpr::Int | TypeExpr::Bool | TypeExpr::String | TypeExpr::Unit => {}
            TypeExpr::Named(name) => {
                if type_params.iter().any(|param| param == name) {
                    return;
                }
                if let Some((alias, item)) = split_qualified_name(name) {
                    let _ = self.resolve_imported_type_item(alias, item, span);
                    return;
                }
                if let Some(item) = resolve_package_item(
                    &self.current_package_data.records,
                    name,
                    &self.current_module,
                ) && !visibility_can_expose(item.visibility, api_visibility)
                {
                    let api = visibility_label(api_visibility);
                    let item_visibility = visibility_label(item.visibility);
                    self.diagnostics.push(
                        Diagnostic::new(
                            "PK012",
                            format!("{api} API may not expose {item_visibility} record `{name}`"),
                            span,
                        )
                        .with_related(format!("record `{name}` is declared here"), item.span),
                    );
                }
                if let Some(item) = resolve_package_item(
                    &self.current_package_data.enums,
                    name,
                    &self.current_module,
                ) && !visibility_can_expose(item.visibility, api_visibility)
                {
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
                if let Some(item) = resolve_package_item(
                    &self.current_package_data.opaque_types,
                    name,
                    &self.current_module,
                ) && !visibility_can_expose(item.visibility, api_visibility)
                {
                    let api = visibility_label(api_visibility);
                    let item_visibility = visibility_label(item.visibility);
                    self.diagnostics.push(
                        Diagnostic::new(
                            "PK012",
                            format!(
                                "{api} API may not expose {item_visibility} opaque type `{name}`"
                            ),
                            span,
                        )
                        .with_related(format!("opaque type `{name}` is declared here"), item.span),
                    );
                }
            }
            TypeExpr::Generic(generic) => {
                if !type_params.iter().any(|param| param == &generic.name)
                    && !is_known_generic_type_name(&generic.name)
                {
                    self.validate_visible_type_with_params(
                        &TypeExpr::Named(generic.name.clone()),
                        api_visibility,
                        span,
                        type_params,
                    );
                }
                for arg in &generic.args {
                    self.validate_visible_type_with_params(arg, api_visibility, span, type_params);
                }
            }
            TypeExpr::Function(function) => {
                for param in &function.params {
                    self.validate_visible_type_with_params(
                        param,
                        api_visibility,
                        span,
                        type_params,
                    );
                }
                self.validate_visible_type_with_params(
                    &function.ret,
                    api_visibility,
                    span,
                    type_params,
                );
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
            self.diagnostics
                .push(unknown_import_alias_diagnostic(alias, span));
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
            self.diagnostics
                .push(unknown_import_alias_diagnostic(alias, span));
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
        if package_item_decl(&package.enums, item).is_some() {
            self.diagnostics.push(missing_export_diagnostic(
                package_path,
                item,
                "enum",
                package_item_decl(&package.enums, item),
                span,
            ));
            return format!("{alias}::{item}");
        }
        if let Some(export) = self.package_exports.opaque_type_by_name(package_id, item) {
            return export.mangled_name.clone();
        }
        if package_item_decl(&package.opaque_types, item).is_some() {
            self.diagnostics.push(missing_export_diagnostic(
                package_path,
                item,
                "opaque type",
                package_item_decl(&package.opaque_types, item),
                span,
            ));
            return format!("{alias}::{item}");
        }
        self.diagnostics.push(missing_export_diagnostic(
            package_path,
            item,
            "type",
            None,
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
    let mut opaque_types: HashMap<String, Vec<PackageItemDecl>> = HashMap::new();
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
                Stmt::OpaqueTypeDecl(opaque) => {
                    insert_package_item_decl(
                        &mut opaque_types,
                        PackageItemDeclInput {
                            name: &opaque.name,
                            visibility: opaque.visibility,
                            module_path: &file.module_path,
                            span: opaque.span,
                            kind: PackageItemKind::OpaqueType,
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
        opaque_types,
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

const INTERFACE_MODULE: &str = "<interface>";

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
        PackageItemKind::OpaqueType => interface
            .opaque_types
            .iter()
            .find(|opaque| opaque.name == name)
            .map(|opaque| opaque.item),
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
            PackageItemKind::OpaqueType => "opaque type",
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

fn package_item_kind_label(kind: PackageItemKind) -> &'static str {
    match kind {
        PackageItemKind::Record => "record",
        PackageItemKind::Enum => "enum",
        PackageItemKind::OpaqueType => "opaque type",
        PackageItemKind::Function => "function",
    }
}

fn is_known_generic_type_name(name: &str) -> bool {
    matches!(name, "List" | "Map")
        || name == crate::known_enum::OPTION_NAME
        || name == crate::known_enum::RESULT_NAME
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
    let mut stack = Vec::new();
    parse_manifest_inner(path, &mut stack)
}

fn parse_manifest_inner(
    path: &Path,
    stack: &mut Vec<PathBuf>,
) -> Result<ProjectManifest, Vec<Diagnostic>> {
    let identity = manifest_identity(path);
    if stack.contains(&identity) {
        return Err(vec![Diagnostic::new(
            "PK014",
            format!("local dependency cycle includes {}", path.display()),
            Span::default(),
        )]);
    }
    stack.push(identity);
    let result = parse_manifest_inner_impl(path, stack);
    stack.pop();
    result
}

fn parse_manifest_inner_impl(
    path: &Path,
    stack: &mut Vec<PathBuf>,
) -> Result<ProjectManifest, Vec<Diagnostic>> {
    let source = fs::read_to_string(path).map_err(|error| {
        vec![Diagnostic::new(
            "PK002",
            format!("failed to read {}: {error}", path.display()),
            Span::default(),
        )]
    })?;

    let mut in_package = false;
    let mut in_dependencies = false;
    let mut name = None;
    let mut source_dir = "src".to_string();
    let mut resource_dir = None;
    let mut dependency_sources = Vec::new();

    for raw_line in source.lines() {
        let line = strip_manifest_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_package = line == "[package]";
            in_dependencies = line == "[dependencies]";
            continue;
        }
        if !in_package && !in_dependencies {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if in_dependencies {
            let Some(package_name) = parse_manifest_key(key) else {
                return Err(vec![Diagnostic::new(
                    "PK014",
                    format!(
                        "manifest dependency key `{key}` in {} is invalid",
                        path.display()
                    ),
                    Span::default(),
                )]);
            };
            if !is_valid_package_path(&package_name) {
                return Err(vec![Diagnostic::new(
                    "PK014",
                    format!(
                        "manifest dependency name `{package_name}` is not a valid package path"
                    ),
                    Span::default(),
                )]);
            }
            if is_reserved_standard_package_path(&package_name) {
                return Err(vec![reserved_standard_package_diagnostic(
                    &package_name,
                    Span::default(),
                )]);
            }
            let dependency_source =
                parse_manifest_dependency_source(value.trim(), path, &package_name)?;
            dependency_sources.push((package_name, dependency_source));
            continue;
        }
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
            "source" => {
                validate_manifest_source_dir(&value, path)?;
                source_dir = value;
            }
            "resources" => {
                validate_manifest_resource_dir(&value, path)?;
                resource_dir = Some(value);
            }
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
    if is_reserved_standard_package_path(&name) {
        return Err(vec![reserved_standard_package_diagnostic(
            &name,
            Span::default(),
        )]);
    }

    let root = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let source_root = root.join(source_dir);
    let resource_root = resource_dir.map(|resource_dir| root.join(resource_dir));
    let mut direct_dependencies = dependency_sources
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    direct_dependencies.sort();
    let dependencies = parse_manifest_dependencies(path, &root, &name, dependency_sources, stack)?;

    Ok(ProjectManifest {
        root,
        source_root,
        resource_root,
        name,
        direct_dependencies,
        dependencies,
    })
}

fn validate_manifest_source_dir(value: &str, manifest_path: &Path) -> Result<(), Vec<Diagnostic>> {
    if value.is_empty() {
        return Err(vec![
            Diagnostic::new(
                "PK014",
                format!(
                    "manifest field `source` in {} must name a source directory",
                    manifest_path.display()
                ),
                Span::default(),
            )
            .with_suggestion("use `source = \"src\"` for package source files"),
        ]);
    }
    if Path::new(value).is_absolute() || value.contains('\\') || value.contains(':') {
        return Err(vec![Diagnostic::new(
            "PK014",
            format!(
                "manifest field `source` in {} must be a relative slash-separated path",
                manifest_path.display()
            ),
            Span::default(),
        )]);
    }
    for segment in value.split('/') {
        if segment.is_empty() || segment == ".." {
            return Err(vec![Diagnostic::new(
                "PK014",
                format!(
                    "manifest field `source` in {} must stay inside the package root",
                    manifest_path.display()
                ),
                Span::default(),
            )]);
        }
        if matches!(segment, ".git" | ".muga") {
            return Err(vec![Diagnostic::new(
                "PK014",
                format!(
                    "manifest field `source` in {} must not use tool metadata directories",
                    manifest_path.display()
                ),
                Span::default(),
            )]);
        }
    }
    Ok(())
}

fn validate_manifest_resource_dir(
    value: &str,
    manifest_path: &Path,
) -> Result<(), Vec<Diagnostic>> {
    if value.is_empty() || value == "." {
        return Err(vec![
            Diagnostic::new(
                "PK014",
                format!(
                    "manifest field `resources` in {} must name a resource directory",
                    manifest_path.display()
                ),
                Span::default(),
            )
            .with_suggestion("use `resources = \"resources\"` for package resource files"),
        ]);
    }
    if Path::new(value).is_absolute() || value.contains('\\') || value.contains(':') {
        return Err(vec![Diagnostic::new(
            "PK014",
            format!(
                "manifest field `resources` in {} must be a relative slash-separated path",
                manifest_path.display()
            ),
            Span::default(),
        )]);
    }
    for segment in value.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(vec![Diagnostic::new(
                "PK014",
                format!(
                    "manifest field `resources` in {} must stay inside the package root",
                    manifest_path.display()
                ),
                Span::default(),
            )]);
        }
        if matches!(segment, ".git" | ".muga") {
            return Err(vec![Diagnostic::new(
                "PK014",
                format!(
                    "manifest field `resources` in {} must not use tool metadata directories",
                    manifest_path.display()
                ),
                Span::default(),
            )]);
        }
    }
    Ok(())
}

fn parse_manifest_dependencies(
    manifest_path: &Path,
    root: &Path,
    root_name: &str,
    dependency_sources: Vec<(String, ManifestDependencySource)>,
    stack: &mut Vec<PathBuf>,
) -> Result<HashMap<String, ProjectDependency>, Vec<Diagnostic>> {
    let mut dependencies = HashMap::new();
    let mut diagnostics = Vec::new();

    for (declared_name, dependency_source) in dependency_sources {
        let (dependency_root, source) = match resolve_manifest_dependency_source(
            root,
            manifest_path,
            &declared_name,
            dependency_source,
        ) {
            Ok(resolved) => resolved,
            Err(mut source_diagnostics) => {
                diagnostics.append(&mut source_diagnostics);
                continue;
            }
        };
        let dependency_manifest_path = dependency_root.join("muga.toml");
        if !dependency_manifest_path.is_file() {
            diagnostics.push(Diagnostic::new(
                "PK014",
                format!(
                    "local dependency `{declared_name}` in {} must point to a directory containing muga.toml",
                    manifest_path.display()
                ),
                Span::default(),
            ));
            continue;
        }

        let dependency_manifest = match parse_manifest_inner(&dependency_manifest_path, stack) {
            Ok(manifest) => manifest,
            Err(mut dependency_diagnostics) => {
                diagnostics.append(&mut dependency_diagnostics);
                continue;
            }
        };
        if dependency_manifest.name != declared_name {
            diagnostics.push(Diagnostic::new(
                "PK014",
                format!(
                    "local dependency `{declared_name}` in {} points to package `{}`",
                    manifest_path.display(),
                    dependency_manifest.name
                ),
                Span::default(),
            ));
            continue;
        }

        insert_manifest_dependency(
            &mut dependencies,
            ProjectDependency {
                root: dependency_manifest.root.clone(),
                source_root: dependency_manifest.source_root.clone(),
                resource_root: dependency_manifest.resource_root.clone(),
                name: dependency_manifest.name.clone(),
                source,
                dependencies: dependency_manifest.direct_dependencies.clone(),
            },
            &mut diagnostics,
        );
        for dependency in dependency_manifest.dependencies.values() {
            insert_manifest_dependency(&mut dependencies, dependency.clone(), &mut diagnostics);
        }
    }

    validate_manifest_dependency_prefixes(root_name, &dependencies, &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(dependencies)
    } else {
        Err(diagnostics)
    }
}

fn resolve_manifest_dependency_source(
    root: &Path,
    manifest_path: &Path,
    declared_name: &str,
    source: ManifestDependencySource,
) -> Result<(PathBuf, ProjectDependencySource), Vec<Diagnostic>> {
    match source {
        ManifestDependencySource::Path(path_value) => {
            let dependency_root = if Path::new(&path_value).is_absolute() {
                PathBuf::from(path_value)
            } else {
                root.join(path_value)
            };
            Ok((dependency_root, ProjectDependencySource::Path))
        }
        ManifestDependencySource::Archive { archive, hash } => {
            if archive.is_empty() {
                return Err(vec![Diagnostic::new(
                    "PK014",
                    format!(
                        "manifest archive dependency `{declared_name}` in {} must not be empty",
                        manifest_path.display()
                    ),
                    Span::default(),
                )]);
            }
            let archive_path = if Path::new(&archive).is_absolute() {
                PathBuf::from(&archive)
            } else {
                root.join(&archive)
            };
            let cache_root = package_archive_dependency_cache_root(root, declared_name, &hash);
            ensure_archive_dependency_cache(&archive_path, &hash, &cache_root)?;
            Ok((
                cache_root,
                ProjectDependencySource::Archive {
                    archive_path,
                    content_hash: hash,
                },
            ))
        }
    }
}

fn ensure_archive_dependency_cache(
    archive_path: &Path,
    content_hash: &str,
    cache_root: &Path,
) -> Result<(), Vec<Diagnostic>> {
    if cache_root.exists() {
        if !cache_root.is_dir() {
            return Err(vec![package_archive_dependency_diagnostic(format!(
                "package archive dependency cache `{}` already exists and is not a directory",
                cache_root.display()
            ))]);
        }
        if !package_archive_dependency_cache_is_empty(cache_root)? {
            return validate_archive_dependency_cache(cache_root, content_hash);
        }
    }

    materialize_package_archive(archive_path, Some(content_hash), cache_root).map(|_| ())
}

fn package_archive_dependency_cache_is_empty(cache_root: &Path) -> Result<bool, Vec<Diagnostic>> {
    let mut entries = fs::read_dir(cache_root).map_err(|error| {
        vec![package_archive_dependency_diagnostic(format!(
            "failed to read package archive dependency cache `{}`: {error}",
            cache_root.display()
        ))]
    })?;
    match entries.next() {
        Some(entry) => {
            entry.map_err(|error| {
                vec![package_archive_dependency_diagnostic(format!(
                    "failed to read package archive dependency cache `{}`: {error}",
                    cache_root.display()
                ))]
            })?;
            Ok(false)
        }
        None => Ok(true),
    }
}

fn validate_archive_dependency_cache(
    cache_root: &Path,
    expected_content_hash: &str,
) -> Result<(), Vec<Diagnostic>> {
    let actual_content_hash = package_archive_dependency_cache_content_hash(cache_root)?;
    if actual_content_hash != expected_content_hash {
        return Err(vec![package_archive_dependency_diagnostic(format!(
            "package archive dependency cache `{}` hash mismatch: expected `{expected_content_hash}`, got `{actual_content_hash}`",
            cache_root.display()
        ))]);
    }
    Ok(())
}

fn package_archive_dependency_cache_content_hash(
    cache_root: &Path,
) -> Result<String, Vec<Diagnostic>> {
    let manifest = fs::read_to_string(cache_root.join("muga.toml")).map_err(|error| {
        vec![package_archive_dependency_diagnostic(format!(
            "failed to read package archive dependency cache manifest `{}`: {error}",
            cache_root.join("muga.toml").display()
        ))]
    })?;
    let source_dir = package_archive_manifest_source_dir(&manifest)?;
    let resource_dir = package_archive_manifest_resource_dir(&manifest)?;
    let resource_root = resource_dir
        .as_ref()
        .map(|resource_dir| cache_root.join(resource_dir));
    let input = package_source_content_input(
        cache_root,
        &cache_root.join(source_dir),
        resource_root.as_deref(),
        "package archive dependency cache",
    )?;
    Ok(format!("sha256:{}", sha256_hex(&input)))
}

fn insert_manifest_dependency(
    dependencies: &mut HashMap<String, ProjectDependency>,
    dependency: ProjectDependency,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(existing) = dependencies.get(&dependency.name) {
        if same_manifest_dependency(existing, &dependency) {
            return;
        }
        diagnostics.push(Diagnostic::new(
            "PK014",
            format!(
                "ambiguous local dependency package `{}` is declared at both {} and {}",
                dependency.name,
                existing.root.display(),
                dependency.root.display()
            ),
            Span::default(),
        ));
        return;
    }
    dependencies.insert(dependency.name.clone(), dependency);
}

fn validate_manifest_dependency_prefixes(
    root_name: &str,
    dependencies: &HashMap<String, ProjectDependency>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut names = vec![root_name.to_string()];
    names.extend(dependencies.keys().cloned());
    names.sort();
    for index in 0..names.len() {
        for other in &names[index + 1..] {
            if package_path_prefixes_overlap(&names[index], other) {
                diagnostics.push(Diagnostic::new(
                    "PK014",
                    format!(
                        "ambiguous local package roots `{}` and `{other}` overlap",
                        names[index]
                    ),
                    Span::default(),
                ));
            }
        }
    }
}

fn same_manifest_dependency(left: &ProjectDependency, right: &ProjectDependency) -> bool {
    left.root == right.root
        && left.source_root == right.source_root
        && left.resource_root == right.resource_root
        && left.source == right.source
}

fn project_manifest_metadata(manifest: &ProjectManifest) -> ProjectManifestMetadata {
    let mut direct_dependencies = manifest.direct_dependencies.clone();
    direct_dependencies.sort();
    let mut dependencies = manifest
        .dependencies
        .values()
        .map(|dependency| project_manifest_dependency_metadata(manifest, dependency))
        .collect::<Vec<_>>();
    dependencies.sort_by(|left, right| left.package_path.cmp(&right.package_path));
    ProjectManifestMetadata {
        manifest_path: manifest.root.join("muga.toml"),
        root: manifest.root.clone(),
        source_root: manifest.source_root.clone(),
        resource_root: manifest.resource_root.clone(),
        package_path: manifest.name.clone(),
        direct_dependencies,
        dependencies,
    }
}

fn project_manifest_dependency_metadata(
    manifest: &ProjectManifest,
    dependency: &ProjectDependency,
) -> ProjectManifestDependencyMetadata {
    let mut dependencies = dependency.dependencies.clone();
    dependencies.sort();
    let (source_kind, source, hash) = match &dependency.source {
        ProjectDependencySource::Path => (
            PackageLockfileDependencySourceKind::Path,
            lockfile_dependency_path(&manifest.root, &dependency.root),
            None,
        ),
        ProjectDependencySource::Archive {
            archive_path,
            content_hash,
        } => (
            PackageLockfileDependencySourceKind::Archive,
            lockfile_dependency_path(&manifest.root, archive_path),
            Some(content_hash.clone()),
        ),
    };
    ProjectManifestDependencyMetadata {
        package_path: dependency.name.clone(),
        root: dependency.root.clone(),
        source_root: dependency.source_root.clone(),
        resource_root: dependency.resource_root.clone(),
        source_kind,
        source,
        hash,
        dependencies,
    }
}

fn manifest_lockfile_text(manifest: &ProjectManifest) -> Result<String, Vec<Diagnostic>> {
    let mut out = String::new();
    out.push_str("# muga.lock -- generated by muga; do not edit by hand\n");
    out.push_str("lockfile_version = 1\n");
    out.push_str(&format!(
        "muga_version = \"{}\"\n",
        escape_lockfile_string(env!("CARGO_PKG_VERSION"))
    ));

    let mut dependencies = manifest.dependencies.values().collect::<Vec<_>>();
    dependencies.sort_by(|left, right| left.name.cmp(&right.name));
    for dependency in dependencies {
        out.push('\n');
        out.push_str("[[package]]\n");
        out.push_str(&format!(
            "alias = \"{}\"\n",
            escape_lockfile_string(&dependency.name)
        ));
        out.push_str(&format!(
            "path = \"{}\"\n",
            escape_lockfile_string(&dependency.name)
        ));
        match &dependency.source {
            ProjectDependencySource::Path => {
                out.push_str(&format!(
                    "source = {{ path = \"{}\" }}\n",
                    escape_lockfile_string(&lockfile_dependency_path(
                        &manifest.root,
                        &dependency.root
                    ))
                ));
                out.push_str(&format!(
                    "source_hash = \"{}\"\n",
                    local_dependency_source_hash(dependency)?
                ));
            }
            ProjectDependencySource::Archive {
                archive_path,
                content_hash,
            } => {
                out.push_str(&format!(
                    "source = {{ archive = \"{}\" }}\n",
                    escape_lockfile_string(&lockfile_dependency_path(&manifest.root, archive_path))
                ));
                out.push_str(&format!(
                    "hash = \"{}\"\n",
                    escape_lockfile_string(content_hash)
                ));
            }
        }
        out.push_str(&format!(
            "dependencies = [{}]\n",
            lockfile_dependency_list(&dependency.dependencies)
        ));
    }

    Ok(out)
}

fn manifest_lockfile_dependency_metadata(
    manifest: &ProjectManifest,
) -> Result<Vec<PackageLockfileDependencyMetadata>, Vec<Diagnostic>> {
    let mut dependencies = manifest.dependencies.values().collect::<Vec<_>>();
    dependencies.sort_by(|left, right| left.name.cmp(&right.name));
    dependencies
        .into_iter()
        .map(|dependency| {
            let mut dependency_paths = dependency.dependencies.clone();
            dependency_paths.sort();
            match &dependency.source {
                ProjectDependencySource::Path => Ok(PackageLockfileDependencyMetadata {
                    package_path: dependency.name.clone(),
                    source_kind: PackageLockfileDependencySourceKind::Path,
                    source: lockfile_dependency_path(&manifest.root, &dependency.root),
                    hash_kind: "source".to_string(),
                    hash: local_dependency_source_hash(dependency)?,
                    dependencies: dependency_paths,
                }),
                ProjectDependencySource::Archive {
                    archive_path,
                    content_hash,
                } => Ok(PackageLockfileDependencyMetadata {
                    package_path: dependency.name.clone(),
                    source_kind: PackageLockfileDependencySourceKind::Archive,
                    source: lockfile_dependency_path(&manifest.root, archive_path),
                    hash_kind: "archive".to_string(),
                    hash: content_hash.clone(),
                    dependencies: dependency_paths,
                }),
            }
        })
        .collect()
}

fn manifest_archive_cache_metadata(manifest: &ProjectManifest) -> Vec<PackageArchiveCacheMetadata> {
    let mut dependencies = manifest.dependencies.values().collect::<Vec<_>>();
    dependencies.sort_by(|left, right| left.name.cmp(&right.name));
    dependencies
        .into_iter()
        .filter_map(|dependency| match &dependency.source {
            ProjectDependencySource::Archive {
                archive_path,
                content_hash,
            } => Some(PackageArchiveCacheMetadata {
                package_path: dependency.name.clone(),
                archive_path: archive_path.clone(),
                cache_root: dependency.root.clone(),
                expected_content_hash: content_hash.clone(),
            }),
            ProjectDependencySource::Path => None,
        })
        .collect()
}

fn validate_existing_lockfile(text: &str, path: &Path) -> Result<(), Vec<Diagnostic>> {
    parse_lockfile_text(text, path)
}

fn parse_lockfile_text(text: &str, path: &Path) -> Result<(), Vec<Diagnostic>> {
    let mut lockfile_version = None;
    let mut muga_version = None;
    let mut current_package = None;
    let mut packages = Vec::new();

    for (line_index, raw_line) in text.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line == "[[package]]" {
            if let Some(package) = current_package.take() {
                packages.push(finish_lockfile_package(package, path)?);
            }
            current_package = Some(LockfilePackageBuilder {
                line: line_number,
                alias: None,
                path: None,
                source: None,
                source_hash: None,
                hash: None,
                dependencies: None,
            });
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            return Err(lockfile_diagnostic(
                path,
                line_number,
                "expected `key = value` entry",
            ));
        };
        let key = key.trim();
        let value = value.trim();
        if let Some(package) = current_package.as_mut() {
            parse_lockfile_package_field(package, key, value, path, line_number)?;
        } else {
            match key {
                "lockfile_version" => {
                    if lockfile_version.replace(value.to_string()).is_some() {
                        return Err(lockfile_diagnostic(
                            path,
                            line_number,
                            "duplicate `lockfile_version`",
                        ));
                    }
                    if value != "1" {
                        return Err(lockfile_diagnostic(
                            path,
                            line_number,
                            format!("unsupported `lockfile_version` `{value}`"),
                        ));
                    }
                }
                "muga_version" => {
                    let version = parse_lockfile_string(value).ok_or_else(|| {
                        lockfile_diagnostic(path, line_number, "`muga_version` must be a string")
                    })?;
                    if muga_version.replace(version).is_some() {
                        return Err(lockfile_diagnostic(
                            path,
                            line_number,
                            "duplicate `muga_version`",
                        ));
                    }
                }
                _ => {
                    return Err(lockfile_diagnostic(
                        path,
                        line_number,
                        format!("unsupported lockfile header field `{key}`"),
                    ));
                }
            }
        }
    }

    if let Some(package) = current_package {
        packages.push(finish_lockfile_package(package, path)?);
    }
    if lockfile_version.is_none() {
        return Err(lockfile_diagnostic(
            path,
            1,
            "missing `lockfile_version = 1`",
        ));
    }
    if muga_version.is_none() {
        return Err(lockfile_diagnostic(path, 1, "missing `muga_version`"));
    }

    validate_lockfile_package_graph(&packages, path)?;
    Ok(())
}

fn parse_lockfile_package_field(
    package: &mut LockfilePackageBuilder,
    key: &str,
    value: &str,
    path: &Path,
    line_number: usize,
) -> Result<(), Vec<Diagnostic>> {
    match key {
        "alias" => {
            let value = parse_lockfile_string(value).ok_or_else(|| {
                lockfile_diagnostic(path, line_number, "`alias` must be a string")
            })?;
            set_lockfile_field(&mut package.alias, value, path, line_number, "alias")
        }
        "path" => {
            let value = parse_lockfile_string(value)
                .ok_or_else(|| lockfile_diagnostic(path, line_number, "`path` must be a string"))?;
            set_lockfile_field(&mut package.path, value, path, line_number, "path")
        }
        "source" => {
            let source = parse_lockfile_source(value, path, line_number)?;
            set_lockfile_field(&mut package.source, source, path, line_number, "source")
        }
        "source_hash" => {
            let source_hash = parse_lockfile_string(value).ok_or_else(|| {
                lockfile_diagnostic(path, line_number, "`source_hash` must be a string")
            })?;
            validate_lockfile_hash(&source_hash, path, line_number, "source_hash")?;
            set_lockfile_field(
                &mut package.source_hash,
                source_hash,
                path,
                line_number,
                "source_hash",
            )
        }
        "hash" => {
            let hash = parse_lockfile_string(value)
                .ok_or_else(|| lockfile_diagnostic(path, line_number, "`hash` must be a string"))?;
            validate_lockfile_hash(&hash, path, line_number, "hash")?;
            set_lockfile_field(&mut package.hash, hash, path, line_number, "hash")
        }
        "dependencies" => {
            let dependencies = parse_lockfile_dependency_list(value, path, line_number)?;
            set_lockfile_field(
                &mut package.dependencies,
                dependencies,
                path,
                line_number,
                "dependencies",
            )
        }
        _ => Err(lockfile_diagnostic(
            path,
            line_number,
            format!("unsupported package field `{key}`"),
        )),
    }
}

fn finish_lockfile_package(
    package: LockfilePackageBuilder,
    path: &Path,
) -> Result<ParsedLockfilePackage, Vec<Diagnostic>> {
    let alias = required_lockfile_field(package.alias, path, package.line, "alias")?;
    let package_path = required_lockfile_field(package.path, path, package.line, "path")?;
    let source = required_lockfile_field(package.source, path, package.line, "source")?;
    let dependencies =
        required_lockfile_field(package.dependencies, path, package.line, "dependencies")?;

    if !is_valid_package_path(&alias) {
        return Err(lockfile_diagnostic(
            path,
            package.line,
            format!("package alias `{alias}` is not a valid package path"),
        ));
    }
    if !is_valid_package_path(&package_path) {
        return Err(lockfile_diagnostic(
            path,
            package.line,
            format!("package path `{package_path}` is not a valid package path"),
        ));
    }
    if alias != package_path {
        return Err(lockfile_diagnostic(
            path,
            package.line,
            format!("local path lockfile entry uses alias `{alias}` but path `{package_path}`"),
        ));
    }
    match source {
        ParsedLockfileSource::Path(source_path) => {
            let _source_hash =
                required_lockfile_field(package.source_hash, path, package.line, "source_hash")?;
            if package.hash.is_some() {
                return Err(lockfile_diagnostic(
                    path,
                    package.line,
                    "local path lockfile entry must use `source_hash`, not `hash`",
                ));
            }
            if source_path.is_empty() {
                return Err(lockfile_diagnostic(
                    path,
                    package.line,
                    "local path lockfile entry has an empty source path",
                ));
            }
        }
        ParsedLockfileSource::Archive(archive_path) => {
            let _hash = required_lockfile_field(package.hash, path, package.line, "hash")?;
            if package.source_hash.is_some() {
                return Err(lockfile_diagnostic(
                    path,
                    package.line,
                    "archive lockfile entry must use `hash`, not `source_hash`",
                ));
            }
            if archive_path.is_empty() {
                return Err(lockfile_diagnostic(
                    path,
                    package.line,
                    "archive lockfile entry has an empty archive path",
                ));
            }
        }
    }
    for dependency in &dependencies {
        if !is_valid_package_path(dependency) {
            return Err(lockfile_diagnostic(
                path,
                package.line,
                format!("dependency `{dependency}` is not a valid package path"),
            ));
        }
    }

    Ok(ParsedLockfilePackage {
        alias,
        dependencies,
    })
}

fn validate_lockfile_package_graph(
    packages: &[ParsedLockfilePackage],
    path: &Path,
) -> Result<(), Vec<Diagnostic>> {
    let mut aliases = HashSet::new();
    for package in packages {
        if !aliases.insert(package.alias.clone()) {
            return Err(lockfile_diagnostic(
                path,
                1,
                format!("duplicate package alias `{}`", package.alias),
            ));
        }
    }

    for package in packages {
        let mut dependencies = HashSet::new();
        for dependency in &package.dependencies {
            if !aliases.contains(dependency) {
                return Err(lockfile_diagnostic(
                    path,
                    1,
                    format!(
                        "package `{}` depends on missing lockfile package `{dependency}`",
                        package.alias
                    ),
                ));
            }
            if !dependencies.insert(dependency) {
                return Err(lockfile_diagnostic(
                    path,
                    1,
                    format!(
                        "package `{}` lists dependency `{dependency}` more than once",
                        package.alias
                    ),
                ));
            }
        }
    }

    Ok(())
}

fn set_lockfile_field<T>(
    slot: &mut Option<T>,
    value: T,
    path: &Path,
    line_number: usize,
    field: &str,
) -> Result<(), Vec<Diagnostic>> {
    if slot.replace(value).is_some() {
        Err(lockfile_diagnostic(
            path,
            line_number,
            format!("duplicate `{field}` field"),
        ))
    } else {
        Ok(())
    }
}

fn required_lockfile_field<T>(
    value: Option<T>,
    path: &Path,
    line_number: usize,
    field: &str,
) -> Result<T, Vec<Diagnostic>> {
    value.ok_or_else(|| lockfile_diagnostic(path, line_number, format!("missing `{field}` field")))
}

fn parse_lockfile_source(
    value: &str,
    path: &Path,
    line_number: usize,
) -> Result<ParsedLockfileSource, Vec<Diagnostic>> {
    let Some(body) = value
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
    else {
        return Err(lockfile_diagnostic(
            path,
            line_number,
            "`source` must use `{ path = \"...\" }` or `{ archive = \"...\" }`",
        ));
    };

    let mut path_value = None;
    let mut archive_value = None;
    for field in split_manifest_inline_fields(body) {
        let Some((key, value)) = field.split_once('=') else {
            return Err(lockfile_diagnostic(
                path,
                line_number,
                "`source` must contain `path = \"...\"` or `archive = \"...\"`",
            ));
        };
        let key = key.trim();
        let value = parse_lockfile_string(value.trim()).ok_or_else(|| {
            lockfile_diagnostic(
                path,
                line_number,
                format!("`source.{key}` must be a string"),
            )
        })?;
        match key {
            "path" => {
                if path_value.replace(value).is_some() {
                    return Err(lockfile_diagnostic(
                        path,
                        line_number,
                        "duplicate `source.path` field",
                    ));
                }
            }
            "archive" => {
                if archive_value.replace(value).is_some() {
                    return Err(lockfile_diagnostic(
                        path,
                        line_number,
                        "duplicate `source.archive` field",
                    ));
                }
            }
            _ => {
                return Err(lockfile_diagnostic(
                    path,
                    line_number,
                    format!("unsupported source field `{key}`"),
                ));
            }
        }
    }

    match (path_value, archive_value) {
        (Some(path_value), None) => Ok(ParsedLockfileSource::Path(path_value)),
        (None, Some(archive_value)) => Ok(ParsedLockfileSource::Archive(archive_value)),
        (None, None) => Err(lockfile_diagnostic(
            path,
            line_number,
            "`source` must contain `path = \"...\"` or `archive = \"...\"`",
        )),
        (Some(_), Some(_)) => Err(lockfile_diagnostic(
            path,
            line_number,
            "`source` must not contain both `path` and `archive`",
        )),
    }
}

fn parse_lockfile_dependency_list(
    value: &str,
    path: &Path,
    line_number: usize,
) -> Result<Vec<String>, Vec<Diagnostic>> {
    let Some(body) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Err(lockfile_diagnostic(
            path,
            line_number,
            "`dependencies` must be a list of strings",
        ));
    };
    let body = body.trim();
    if body.is_empty() {
        return Ok(Vec::new());
    }

    let mut dependencies = Vec::new();
    for value in split_manifest_inline_fields(body) {
        let dependency = parse_lockfile_string(value.trim()).ok_or_else(|| {
            lockfile_diagnostic(
                path,
                line_number,
                "`dependencies` must contain only strings",
            )
        })?;
        dependencies.push(dependency);
    }
    Ok(dependencies)
}

fn parse_lockfile_string(value: &str) -> Option<String> {
    let body = value.strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::new();
    let mut escaped = false;
    for ch in body.chars() {
        if escaped {
            match ch {
                '\\' | '"' => out.push(ch),
                _ => return None,
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return None;
        } else {
            out.push(ch);
        }
    }
    if escaped { None } else { Some(out) }
}

fn validate_lockfile_hash(
    value: &str,
    path: &Path,
    line_number: usize,
    field: &str,
) -> Result<(), Vec<Diagnostic>> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(lockfile_diagnostic(
            path,
            line_number,
            format!("`{field}` must start with `sha256:`"),
        ));
    };
    if hex.len() != 64
        || !hex
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
    {
        return Err(lockfile_diagnostic(
            path,
            line_number,
            format!("`{field}` must be `sha256:` followed by 64 lowercase hexadecimal digits"),
        ));
    }
    Ok(())
}

fn lockfile_diagnostic(
    path: &Path,
    line_number: usize,
    message: impl Into<String>,
) -> Vec<Diagnostic> {
    vec![
        Diagnostic::new(
            "PK026",
            format!(
                "invalid package lockfile `{}` at line {}: {}",
                path.display(),
                line_number,
                message.into()
            ),
            Span::default(),
        )
        .with_suggestion("restore the generated dependency lockfile format or delete muga.lock"),
    ]
}

fn local_dependency_source_hash(dependency: &ProjectDependency) -> Result<String, Vec<Diagnostic>> {
    let input = local_dependency_source_input(dependency)?;
    Ok(format!("sha256:{}", sha256_hex(&input)))
}

fn local_dependency_source_input(
    dependency: &ProjectDependency,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    package_source_content_input(
        &dependency.root,
        &dependency.source_root,
        dependency.resource_root.as_deref(),
        "lockfile",
    )
}

fn validate_package_archive_emission_manifest_roots(
    manifest: &ProjectManifest,
) -> Result<(), Vec<Diagnostic>> {
    let manifest_path = manifest.root.join("muga.toml");
    let manifest_source = fs::read_to_string(&manifest_path).map_err(|error| {
        vec![Diagnostic::new(
            "PK027",
            format!(
                "failed to read package manifest `{}` for package archive: {error}",
                manifest_path.display()
            ),
            Span::default(),
        )]
    })?;
    package_archive_manifest_source_dir_with_diagnostic(
        &manifest_source,
        package_archive_emission_diagnostic,
    )?;
    package_archive_manifest_resource_dir_with_diagnostic(
        &manifest_source,
        package_archive_emission_diagnostic,
    )?;
    Ok(())
}

fn validate_package_archive_output_location(
    archive_root: &Path,
    manifest: &ProjectManifest,
) -> Result<(), Vec<Diagnostic>> {
    let archive_root = package_absolute_normalized_path(archive_root)?;
    let source_root = package_absolute_normalized_path(&manifest.source_root)?;
    if archive_root.starts_with(&source_root) {
        return Err(vec![package_archive_emission_diagnostic(format!(
            "package archive output `{}` must not be inside package source root `{}`",
            archive_root.display(),
            source_root.display()
        ))]);
    }
    if let Some(resource_root) = &manifest.resource_root {
        let resource_root = package_absolute_normalized_path(resource_root)?;
        if archive_root.starts_with(&resource_root) {
            return Err(vec![package_archive_emission_diagnostic(format!(
                "package archive output `{}` must not be inside package resource root `{}`",
                archive_root.display(),
                resource_root.display()
            ))]);
        }
    }
    Ok(())
}

fn package_absolute_normalized_path(path: &Path) -> Result<PathBuf, Vec<Diagnostic>> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|error| {
                vec![package_archive_emission_diagnostic(format!(
                    "failed to resolve current directory for package archive paths: {error}"
                ))]
            })?
            .join(path)
    };
    Ok(normalize_package_path_lexically(&path))
}

fn normalize_package_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}

fn package_source_content_input(
    root: &Path,
    source_root: &Path,
    resource_root: Option<&Path>,
    context: &str,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let manifest_path = root.join("muga.toml");
    let manifest_source = fs::read_to_string(&manifest_path).map_err(|error| {
        vec![Diagnostic::new(
            "PK025",
            format!(
                "failed to read package manifest `{}` for {context}: {error}",
                manifest_path.display()
            ),
            Span::default(),
        )]
    })?;

    let mut out = format!(
        "manifest\tmuga.toml\t{}\n{}\n",
        manifest_source.len(),
        manifest_source
    )
    .into_bytes();
    let mut files = Vec::new();
    validate_package_content_root(source_root, "source", context)?;
    collect_package_source_files(source_root, source_root, &mut files, context)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    for (relative_path, source) in files {
        out.extend_from_slice(
            format!("file\t{}\t{}\n{}\n", relative_path, source.len(), source).as_bytes(),
        );
    }
    if let Some(resource_root) = resource_root {
        let mut resources = Vec::new();
        validate_package_content_root(resource_root, "resource", context)?;
        collect_package_resource_files(resource_root, resource_root, &mut resources, context)?;
        resources.sort_by(|left, right| left.0.cmp(&right.0));
        for (relative_path, contents) in resources {
            out.extend_from_slice(
                format!("resource\t{}\t{}\n", relative_path, contents.len()).as_bytes(),
            );
            out.extend_from_slice(&contents);
            out.push(b'\n');
        }
    }
    Ok(out)
}

fn validate_package_content_root(
    root: &Path,
    kind: &str,
    context: &str,
) -> Result<(), Vec<Diagnostic>> {
    let metadata = fs::symlink_metadata(root).map_err(|error| {
        vec![Diagnostic::new(
            "PK025",
            format!(
                "failed to read package {kind} root metadata `{}` for {context}: {error}",
                root.display()
            ),
            Span::default(),
        )]
    })?;
    if metadata.file_type().is_symlink() {
        return Err(vec![Diagnostic::new(
            "PK025",
            format!(
                "package {kind} root `{}` for {context} must not be a symlink",
                root.display()
            ),
            Span::default(),
        )]);
    }
    Ok(())
}

fn materialize_validated_package_archive(
    archive: &PackageArchive,
    destination_root: &Path,
) -> Result<PackageArchiveMaterializationOutput, Vec<Diagnostic>> {
    let source_dir = package_archive_manifest_source_dir(&archive.manifest.contents)?;
    let resource_dir = package_archive_manifest_resource_dir(&archive.manifest.contents)?;
    let mut outputs = Vec::with_capacity(archive.sources.len() + archive.resources.len() + 1);
    outputs.push((
        destination_root.join("muga.toml"),
        archive.manifest.contents.as_bytes().to_vec(),
    ));
    for source in &archive.sources {
        outputs.push((
            destination_root.join(&source_dir).join(&source.path),
            source.contents.as_bytes().to_vec(),
        ));
    }
    if let Some(resource_dir) = resource_dir {
        for resource in &archive.resources {
            outputs.push((
                destination_root.join(&resource_dir).join(&resource.path),
                resource.contents.clone(),
            ));
        }
    } else if !archive.resources.is_empty() {
        return Err(vec![package_archive_materialization_diagnostic(
            "package archive resource entries require [package] resources",
        )]);
    }

    preflight_package_archive_materialization(destination_root, &outputs)?;
    fs::create_dir_all(destination_root).map_err(|error| {
        vec![package_archive_materialization_diagnostic(format!(
            "failed to create package archive materialization root `{}`: {error}",
            destination_root.display()
        ))]
    })?;
    for (path, contents) in &outputs {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                vec![package_archive_materialization_diagnostic(format!(
                    "failed to create package archive materialization directory `{}`: {error}",
                    parent.display()
                ))]
            })?;
        }
        fs::write(path, contents).map_err(|error| {
            vec![package_archive_materialization_diagnostic(format!(
                "failed to write package archive materialized file `{}`: {error}",
                path.display()
            ))]
        })?;
    }

    Ok(PackageArchiveMaterializationOutput {
        root: destination_root.to_path_buf(),
        content_hash: archive.content_hash.clone(),
        files: outputs.into_iter().map(|(path, _)| path).collect(),
    })
}

fn preflight_package_archive_materialization(
    destination_root: &Path,
    outputs: &[(PathBuf, Vec<u8>)],
) -> Result<(), Vec<Diagnostic>> {
    if destination_root.exists() && !destination_root.is_dir() {
        return Err(vec![package_archive_materialization_diagnostic(format!(
            "package archive materialization root `{}` already exists and is not a directory",
            destination_root.display()
        ))]);
    }
    if destination_root.is_dir() {
        let mut entries = fs::read_dir(destination_root).map_err(|error| {
            vec![package_archive_materialization_diagnostic(format!(
                "failed to read package archive materialization root `{}`: {error}",
                destination_root.display()
            ))]
        })?;
        if let Some(entry) = entries.next() {
            entry.map_err(|error| {
                vec![package_archive_materialization_diagnostic(format!(
                    "failed to read package archive materialization root `{}`: {error}",
                    destination_root.display()
                ))]
            })?;
            return Err(vec![package_archive_materialization_diagnostic(format!(
                "package archive materialization root `{}` must be empty",
                destination_root.display()
            ))]);
        }
    }

    let mut seen = HashSet::new();
    for (path, _) in outputs {
        if !seen.insert(path.clone()) {
            return Err(vec![package_archive_materialization_diagnostic(format!(
                "package archive materialization would write `{}` more than once",
                path.display()
            ))]);
        }
        if path.exists() {
            return Err(vec![package_archive_materialization_diagnostic(format!(
                "package archive materialized file `{}` already exists",
                path.display()
            ))]);
        }
    }
    Ok(())
}

fn package_archive_manifest_source_dir(manifest: &str) -> Result<PathBuf, Vec<Diagnostic>> {
    package_archive_manifest_source_dir_with_diagnostic(
        manifest,
        package_archive_materialization_diagnostic,
    )
}

fn package_archive_manifest_source_dir_with_diagnostic<F>(
    manifest: &str,
    diagnostic: F,
) -> Result<PathBuf, Vec<Diagnostic>>
where
    F: Fn(String) -> Diagnostic,
{
    let source_dir =
        package_archive_manifest_string_field_with_diagnostic(manifest, "source", &diagnostic)?
            .unwrap_or_else(|| "src".to_string());
    package_archive_manifest_relative_dir_with_diagnostic("source", &source_dir, diagnostic)
}

fn package_archive_manifest_resource_dir(
    manifest: &str,
) -> Result<Option<PathBuf>, Vec<Diagnostic>> {
    package_archive_manifest_resource_dir_with_diagnostic(
        manifest,
        package_archive_materialization_diagnostic,
    )
}

fn package_archive_manifest_resource_dir_with_diagnostic<F>(
    manifest: &str,
    diagnostic: F,
) -> Result<Option<PathBuf>, Vec<Diagnostic>>
where
    F: Fn(String) -> Diagnostic,
{
    let Some(resource_dir) =
        package_archive_manifest_string_field_with_diagnostic(manifest, "resources", &diagnostic)?
    else {
        return Ok(None);
    };
    package_archive_manifest_relative_dir_with_diagnostic("resources", &resource_dir, diagnostic)
        .map(Some)
}

fn package_archive_manifest_string_field_with_diagnostic<F>(
    manifest: &str,
    field_name: &str,
    diagnostic: F,
) -> Result<Option<String>, Vec<Diagnostic>>
where
    F: Fn(String) -> Diagnostic,
{
    let mut in_package = false;

    for raw_line in manifest.lines() {
        let line = strip_manifest_comment(raw_line).trim();
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
        if key.trim() != field_name {
            continue;
        }
        let Some(value) = parse_manifest_string(value.trim()) else {
            return Err(vec![diagnostic(format!(
                "package archive manifest field `{field_name}` must be a string"
            ))]);
        };
        return Ok(Some(value));
    }

    Ok(None)
}

fn package_archive_manifest_relative_dir_with_diagnostic<F>(
    field_name: &str,
    value: &str,
    diagnostic: F,
) -> Result<PathBuf, Vec<Diagnostic>>
where
    F: Fn(String) -> Diagnostic,
{
    if value.is_empty() {
        return Err(vec![diagnostic(format!(
            "package archive manifest field `{field_name}` must not be empty"
        ))]);
    }
    if value.starts_with('/') || value.contains('\\') || value.contains(':') {
        return Err(vec![diagnostic(format!(
            "package archive manifest {field_name} `{value}` must be a relative slash-separated path"
        ))]);
    }
    let mut path = PathBuf::new();
    for segment in value.split('/') {
        if segment.is_empty() || segment == ".." || (field_name == "resources" && segment == ".") {
            return Err(vec![diagnostic(format!(
                "package archive manifest {field_name} `{value}` must stay inside the materialization root"
            ))]);
        }
        if segment == "." {
            continue;
        }
        if matches!(segment, ".git" | ".muga") {
            return Err(vec![diagnostic(format!(
                "package archive manifest {field_name} `{value}` must not use tool metadata directories"
            ))]);
        }
        path.push(segment);
    }
    Ok(path)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RawPackageArchiveEntry {
    kind: String,
    path: String,
    contents: Vec<u8>,
}

struct PackageArchiveParser<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> PackageArchiveParser<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn next_entry(&mut self) -> Result<Option<RawPackageArchiveEntry>, Vec<Diagnostic>> {
        if self.offset == self.bytes.len() {
            return Ok(None);
        }

        let header_end = self.bytes[self.offset..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|relative| self.offset + relative)
            .ok_or_else(|| {
                vec![package_archive_validation_diagnostic(
                    "package archive entry header is missing a newline",
                )]
            })?;
        let header = &self.bytes[self.offset..header_end];
        self.offset = header_end + 1;

        let fields = header.split(|byte| *byte == b'\t').collect::<Vec<_>>();
        if fields.len() != 3 {
            return Err(vec![package_archive_validation_diagnostic(
                "package archive entry header must have kind, path, and byte length",
            )]);
        }

        let kind = archive_header_field(fields[0], "kind")?;
        let path = archive_header_path(fields[1])?;
        let byte_len = archive_header_byte_len(fields[2], &path)?;
        let content_end = self.offset.checked_add(byte_len).ok_or_else(|| {
            vec![package_archive_validation_diagnostic(format!(
                "package archive entry `{path}` byte length is too large"
            ))]
        })?;
        if content_end > self.bytes.len() {
            return Err(vec![package_archive_validation_diagnostic(format!(
                "package archive entry `{path}` declares {byte_len} bytes but the archive ends early"
            ))]);
        }

        let contents = self.bytes[self.offset..content_end].to_vec();
        self.offset = content_end;

        if self.bytes.get(self.offset) != Some(&b'\n') {
            return Err(vec![package_archive_validation_diagnostic(format!(
                "package archive entry `{path}` is missing the newline after its contents"
            ))]);
        }
        self.offset += 1;

        Ok(Some(RawPackageArchiveEntry {
            kind,
            path,
            contents,
        }))
    }
}

fn package_archive_utf8_entry_contents(
    path: &str,
    contents: &[u8],
) -> Result<String, Vec<Diagnostic>> {
    std::str::from_utf8(contents)
        .map(str::to_string)
        .map_err(|_| {
            vec![package_archive_validation_diagnostic(format!(
                "package archive entry `{path}` contents are not valid UTF-8"
            ))]
        })
}

fn archive_header_field(bytes: &[u8], field: &str) -> Result<String, Vec<Diagnostic>> {
    std::str::from_utf8(bytes)
        .map(ToString::to_string)
        .map_err(|_| {
            vec![package_archive_validation_diagnostic(format!(
                "package archive entry {field} is not valid UTF-8"
            ))]
        })
}

fn archive_header_path(bytes: &[u8]) -> Result<String, Vec<Diagnostic>> {
    std::str::from_utf8(bytes)
        .map(ToString::to_string)
        .map_err(|_| {
            vec![package_archive_validation_diagnostic(
                "package archive entry path is not valid UTF-8",
            )]
        })
}

fn archive_header_byte_len(bytes: &[u8], path: &str) -> Result<usize, Vec<Diagnostic>> {
    if bytes.is_empty() || !bytes.iter().all(|byte| byte.is_ascii_digit()) {
        return Err(vec![package_archive_validation_diagnostic(format!(
            "package archive entry `{path}` has an invalid byte length"
        ))]);
    }
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|text| text.parse::<usize>().ok())
        .ok_or_else(|| {
            vec![package_archive_validation_diagnostic(format!(
                "package archive entry `{path}` byte length is too large"
            ))]
        })
}

fn validate_expected_package_archive_hash(hash: &str) -> Result<(), Vec<Diagnostic>> {
    let Some(hex) = hash.strip_prefix("sha256:") else {
        return Err(vec![package_archive_validation_diagnostic(format!(
            "package archive expected hash `{hash}` must start with `sha256:`"
        ))]);
    };
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(vec![package_archive_validation_diagnostic(format!(
            "package archive expected hash `{hash}` must be `sha256:` followed by 64 hexadecimal digits"
        ))]);
    }
    if !hex
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(vec![package_archive_validation_diagnostic(format!(
            "package archive expected hash `{hash}` must use lower-case hexadecimal"
        ))]);
    }
    Ok(())
}

fn expected_package_archive_hash_from_path(path: &Path) -> Result<String, Vec<Diagnostic>> {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return Err(vec![package_archive_validation_diagnostic(format!(
            "package archive path `{}` must have a valid UTF-8 file name",
            path.display()
        ))]);
    };
    let Some(stem) = file_name.strip_suffix(".mgp") else {
        return Err(vec![
            package_archive_validation_diagnostic(format!(
                "package archive `{file_name}` must use the `.mgp` extension"
            ))
            .with_suggestion("use the archive file written by `muga emit-package-archive`"),
        ]);
    };
    let Some((package, hash)) = stem.rsplit_once("-sha256-") else {
        return Err(vec![
            package_archive_validation_diagnostic(format!(
                "package archive `{file_name}` must use a `*-sha256-<hash>.mgp` file name"
            ))
            .with_suggestion("use the archive file written by `muga emit-package-archive`"),
        ]);
    };
    if package.is_empty() {
        return Err(vec![
            package_archive_validation_diagnostic(format!(
                "package archive `{file_name}` must include a package name before `-sha256-`"
            ))
            .with_suggestion("use the archive file written by `muga emit-package-archive`"),
        ]);
    }
    validate_package_archive_file_hash(hash, file_name)?;
    Ok(format!("sha256:{hash}"))
}

fn validate_package_archive_file_hash(hash: &str, file_name: &str) -> Result<(), Vec<Diagnostic>> {
    if hash.len() != 64 || !hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(vec![package_archive_validation_diagnostic(format!(
            "package archive `{file_name}` must contain 64 hexadecimal digits after `-sha256-`"
        ))]);
    }
    if hash.chars().any(|ch| ch.is_ascii_uppercase()) {
        return Err(vec![package_archive_validation_diagnostic(format!(
            "package archive `{file_name}` hash must use lower-case hexadecimal"
        ))]);
    }
    Ok(())
}

fn validate_package_archive_source_path(path: &str) -> Result<(), Vec<Diagnostic>> {
    if path.is_empty() {
        return Err(vec![package_archive_validation_diagnostic(
            "package archive source entry path must not be empty",
        )]);
    }
    if path == "muga.toml" || !path.ends_with(".muga") {
        return Err(vec![package_archive_validation_diagnostic(format!(
            "package archive source entry `{path}` must be a `.muga` source file"
        ))]);
    }
    if path.starts_with('/') || path.contains('\\') {
        return Err(vec![package_archive_validation_diagnostic(format!(
            "package archive source entry `{path}` must be a relative slash-separated path"
        ))]);
    }
    for segment in path.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(vec![package_archive_validation_diagnostic(format!(
                "package archive source entry `{path}` must stay inside the package source root"
            ))]);
        }
        if matches!(segment, ".git" | ".muga") {
            return Err(vec![package_archive_validation_diagnostic(format!(
                "package archive source entry `{path}` must not include tool metadata directories"
            ))]);
        }
    }
    Ok(())
}

fn validate_package_archive_resource_path(path: &str) -> Result<(), Vec<Diagnostic>> {
    if path.is_empty() {
        return Err(vec![package_archive_validation_diagnostic(
            "package archive resource entry path must not be empty",
        )]);
    }
    if path.starts_with('/') || path.contains('\\') || path.contains(':') {
        return Err(vec![package_archive_validation_diagnostic(format!(
            "package archive resource entry `{path}` must be a relative slash-separated path"
        ))]);
    }
    for segment in path.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(vec![package_archive_validation_diagnostic(format!(
                "package archive resource entry `{path}` must stay inside the package resource root"
            ))]);
        }
        if matches!(segment, ".git" | ".muga") {
            return Err(vec![package_archive_validation_diagnostic(format!(
                "package archive resource entry `{path}` must not include tool metadata directories"
            ))]);
        }
    }
    Ok(())
}

fn package_archive_validation_diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("PK028", message, Span::default())
        .with_suggestion("recreate the archive with `emit-package-archive`")
}

fn package_archive_emission_diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("PK027", message, Span::default())
        .with_suggestion("choose archive inputs and outputs that stay inside the package boundary")
}

fn package_archive_materialization_diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("PK029", message, Span::default())
        .with_suggestion("materialize into an empty destination or recreate the archive")
}

fn package_archive_dependency_diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("PK030", message, Span::default())
        .with_suggestion("delete the archive dependency cache entry or update the dependency hash")
}

fn collect_package_source_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<(String, String)>,
    context: &str,
) -> Result<(), Vec<Diagnostic>> {
    let entries = fs::read_dir(current).map_err(|error| {
        vec![Diagnostic::new(
            "PK025",
            format!(
                "failed to read package source directory `{}` for {context}: {error}",
                current.display()
            ),
            Span::default(),
        )]
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            vec![Diagnostic::new(
                "PK025",
                format!(
                    "failed to read package source directory `{}` for {context}: {error}",
                    current.display()
                ),
                Span::default(),
            )]
        })?;
        paths.push(entry.path());
    }
    paths.sort();

    for path in paths {
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            vec![Diagnostic::new(
                "PK025",
                format!(
                    "failed to read package source metadata `{}` for {context}: {error}",
                    path.display()
                ),
                Span::default(),
            )]
        })?;
        if metadata.file_type().is_symlink() {
            return Err(vec![Diagnostic::new(
                "PK025",
                format!(
                    "package source entry `{}` for {context} must not be a symlink",
                    path.display()
                ),
                Span::default(),
            )]);
        }
        if metadata.is_dir() {
            if should_skip_package_source_directory(&path) {
                continue;
            }
            collect_package_source_files(root, &path, files, context)?;
        } else if metadata.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == "muga")
        {
            let relative_path = package_relative_source_path(root, &path)?;
            let source = fs::read_to_string(&path).map_err(|error| {
                vec![Diagnostic::new(
                    "PK025",
                    format!(
                        "failed to read package source file `{}` for {context}: {error}",
                        path.display()
                    ),
                    Span::default(),
                )]
            })?;
            files.push((relative_path, source));
        }
    }

    Ok(())
}

fn collect_package_resource_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
    context: &str,
) -> Result<(), Vec<Diagnostic>> {
    let entries = fs::read_dir(current).map_err(|error| {
        vec![Diagnostic::new(
            "PK025",
            format!(
                "failed to read package resource directory `{}` for {context}: {error}",
                current.display()
            ),
            Span::default(),
        )]
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            vec![Diagnostic::new(
                "PK025",
                format!(
                    "failed to read package resource directory `{}` for {context}: {error}",
                    current.display()
                ),
                Span::default(),
            )]
        })?;
        paths.push(entry.path());
    }
    paths.sort();

    for path in paths {
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            vec![Diagnostic::new(
                "PK025",
                format!(
                    "failed to read package resource metadata `{}` for {context}: {error}",
                    path.display()
                ),
                Span::default(),
            )]
        })?;
        if metadata.file_type().is_symlink() {
            return Err(vec![Diagnostic::new(
                "PK025",
                format!(
                    "package resource entry `{}` for {context} must not be a symlink",
                    path.display()
                ),
                Span::default(),
            )]);
        }
        if metadata.is_dir() {
            if should_skip_package_source_directory(&path) {
                continue;
            }
            collect_package_resource_files(root, &path, files, context)?;
        } else if metadata.is_file() {
            let relative_path = package_relative_resource_path(root, &path)?;
            let contents = fs::read(&path).map_err(|error| {
                vec![Diagnostic::new(
                    "PK025",
                    format!(
                        "failed to read package resource file `{}` for {context}: {error}",
                        path.display()
                    ),
                    Span::default(),
                )]
            })?;
            files.push((relative_path, contents));
        }
    }

    Ok(())
}

fn should_skip_package_source_directory(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| matches!(name, ".git" | ".muga"))
}

fn lockfile_dependency_path(project_root: &Path, dependency_root: &Path) -> String {
    let project_root = canonical_or_self(project_root);
    let dependency_root = canonical_or_self(dependency_root);
    relative_path_string(&project_root, &dependency_root)
        .unwrap_or_else(|| dependency_root.display().to_string())
}

fn package_relative_source_path(root: &Path, path: &Path) -> Result<String, Vec<Diagnostic>> {
    let relative = path.strip_prefix(root).map_err(|_| {
        vec![Diagnostic::new(
            "PK025",
            format!(
                "package source file `{}` is outside source root `{}`",
                path.display(),
                root.display()
            ),
            Span::default(),
        )]
    })?;
    path_components_as_slashes(relative).ok_or_else(|| {
        vec![Diagnostic::new(
            "PK025",
            format!(
                "package source file `{}` contains non-UTF-8 path components",
                path.display()
            ),
            Span::default(),
        )]
    })
}

fn package_relative_resource_path(root: &Path, path: &Path) -> Result<String, Vec<Diagnostic>> {
    let relative = path.strip_prefix(root).map_err(|_| {
        vec![Diagnostic::new(
            "PK025",
            format!(
                "package resource file `{}` is outside resource root `{}`",
                path.display(),
                root.display()
            ),
            Span::default(),
        )]
    })?;
    path_components_as_slashes(relative).ok_or_else(|| {
        vec![Diagnostic::new(
            "PK025",
            format!(
                "package resource file `{}` contains non-UTF-8 path components",
                path.display()
            ),
            Span::default(),
        )]
    })
}

fn canonical_or_self(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn relative_path_string(from: &Path, to: &Path) -> Option<String> {
    let from = path_component_strings(from)?;
    let to = path_component_strings(to)?;
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    if common == 0 {
        return None;
    }

    let mut parts = Vec::new();
    parts.extend(std::iter::repeat_n(
        "..".to_string(),
        from.len().saturating_sub(common),
    ));
    parts.extend(to[common..].iter().cloned());
    if parts.is_empty() {
        Some(".".to_string())
    } else {
        Some(parts.join("/"))
    }
}

fn path_component_strings(path: &Path) -> Option<Vec<String>> {
    path.components()
        .map(|component| Some(component.as_os_str().to_str()?.to_string()))
        .collect()
}

fn path_components_as_slashes(path: &Path) -> Option<String> {
    let parts = path_component_strings(path)?;
    if parts.is_empty() {
        Some(".".to_string())
    } else {
        Some(parts.join("/"))
    }
}

fn lockfile_dependency_list(dependencies: &[String]) -> String {
    let mut dependencies = dependencies.to_vec();
    dependencies.sort();
    dependencies
        .iter()
        .map(|dependency| format!("\"{}\"", escape_lockfile_string(dependency)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn escape_lockfile_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn package_archive_file_path(root: &Path, package_name: &str, content_hash: &str) -> PathBuf {
    let hash = content_hash
        .strip_prefix("sha256:")
        .unwrap_or(content_hash)
        .replace(':', "-");
    root.join(format!(
        "{}-sha256-{}.mgp",
        package_name.replace("::", "__"),
        hash
    ))
}

fn package_archive_dependency_cache_root(
    project_root: &Path,
    dependency_name: &str,
    content_hash: &str,
) -> PathBuf {
    let hash = content_hash.strip_prefix("sha256:").unwrap_or(content_hash);
    project_root.join(".muga").join("packages").join(format!(
        "{}-sha256-{}",
        dependency_name.replace("::", "__"),
        hash
    ))
}

pub(crate) fn sha256_hex(input: &[u8]) -> String {
    const H0: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut message = input.to_vec();
    let bit_len = (message.len() as u64) * 8;
    message.push(0x80);
    while (message.len() % 64) != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    let mut hash = H0;
    for chunk in message.chunks_exact(64) {
        let mut words = [0_u32; 64];
        for (index, word) in words.iter_mut().take(16).enumerate() {
            let start = index * 4;
            *word = u32::from_be_bytes([
                chunk[start],
                chunk[start + 1],
                chunk[start + 2],
                chunk[start + 3],
            ]);
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }

        let mut a = hash[0];
        let mut b = hash[1];
        let mut c = hash[2];
        let mut d = hash[3];
        let mut e = hash[4];
        let mut f = hash[5];
        let mut g = hash[6];
        let mut h = hash[7];

        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        hash[0] = hash[0].wrapping_add(a);
        hash[1] = hash[1].wrapping_add(b);
        hash[2] = hash[2].wrapping_add(c);
        hash[3] = hash[3].wrapping_add(d);
        hash[4] = hash[4].wrapping_add(e);
        hash[5] = hash[5].wrapping_add(f);
        hash[6] = hash[6].wrapping_add(g);
        hash[7] = hash[7].wrapping_add(h);
    }

    hash.iter()
        .map(|word| format!("{word:08x}"))
        .collect::<Vec<_>>()
        .join("")
}

fn package_path_prefixes_overlap(left: &str, right: &str) -> bool {
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|rest| rest.starts_with("::"))
        || right
            .strip_prefix(left)
            .is_some_and(|rest| rest.starts_with("::"))
}

fn parse_manifest_dependency_source(
    value: &str,
    manifest_path: &Path,
    dependency_name: &str,
) -> Result<ManifestDependencySource, Vec<Diagnostic>> {
    let Some(body) = value
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
    else {
        return Err(vec![unsupported_manifest_dependency_diagnostic(
            manifest_path,
            dependency_name,
        )]);
    };

    let mut path_value = None;
    let mut archive_value = None;
    let mut hash_value = None;
    for field in split_manifest_inline_fields(body) {
        let Some((key, value)) = field.split_once('=') else {
            return Err(vec![unsupported_manifest_dependency_diagnostic(
                manifest_path,
                dependency_name,
            )]);
        };
        let key = key.trim();
        let Some(value) = parse_manifest_string(value.trim()) else {
            return Err(vec![Diagnostic::new(
                "PK014",
                format!(
                    "manifest dependency `{dependency_name}` field `{key}` in {} must be a string",
                    manifest_path.display()
                ),
                Span::default(),
            )]);
        };
        match key {
            "path" => {
                if path_value.replace(value).is_some() {
                    return Err(vec![Diagnostic::new(
                        "PK014",
                        format!(
                            "manifest dependency `{dependency_name}` in {} has duplicate `path` fields",
                            manifest_path.display()
                        ),
                        Span::default(),
                    )]);
                }
            }
            "archive" => {
                if archive_value.replace(value).is_some() {
                    return Err(vec![Diagnostic::new(
                        "PK014",
                        format!(
                            "manifest dependency `{dependency_name}` in {} has duplicate `archive` fields",
                            manifest_path.display()
                        ),
                        Span::default(),
                    )]);
                }
            }
            "hash" => {
                validate_manifest_archive_dependency_hash(&value, manifest_path, dependency_name)?;
                if hash_value.replace(value).is_some() {
                    return Err(vec![Diagnostic::new(
                        "PK014",
                        format!(
                            "manifest dependency `{dependency_name}` in {} has duplicate `hash` fields",
                            manifest_path.display()
                        ),
                        Span::default(),
                    )]);
                }
            }
            _ => {
                return Err(vec![Diagnostic::new(
                    "PK014",
                    format!(
                        "manifest dependency `{dependency_name}` in {} currently supports only `path`, or `archive` with `hash`",
                        manifest_path.display()
                    ),
                    Span::default(),
                )]);
            }
        }
    }

    match (path_value, archive_value, hash_value) {
        (Some(path_value), None, None) => Ok(ManifestDependencySource::Path(path_value)),
        (None, Some(archive), Some(hash)) => {
            Ok(ManifestDependencySource::Archive { archive, hash })
        }
        (Some(_), Some(_), _) => Err(vec![Diagnostic::new(
            "PK014",
            format!(
                "manifest dependency `{dependency_name}` in {} must not combine `path` and `archive`",
                manifest_path.display()
            ),
            Span::default(),
        )]),
        (Some(_), None, Some(_)) => Err(vec![Diagnostic::new(
            "PK014",
            format!(
                "manifest dependency `{dependency_name}` in {} must not combine `path` and `hash`",
                manifest_path.display()
            ),
            Span::default(),
        )]),
        (None, Some(_), None) => Err(vec![Diagnostic::new(
            "PK014",
            format!(
                "manifest dependency `{dependency_name}` in {} archive form requires `hash`",
                manifest_path.display()
            ),
            Span::default(),
        )]),
        (None, None, Some(_)) => Err(vec![Diagnostic::new(
            "PK014",
            format!(
                "manifest dependency `{dependency_name}` in {} hash requires `archive`",
                manifest_path.display()
            ),
            Span::default(),
        )]),
        (None, None, None) => Err(vec![unsupported_manifest_dependency_diagnostic(
            manifest_path,
            dependency_name,
        )]),
    }
}

fn validate_manifest_archive_dependency_hash(
    value: &str,
    manifest_path: &Path,
    dependency_name: &str,
) -> Result<(), Vec<Diagnostic>> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(vec![Diagnostic::new(
            "PK014",
            format!(
                "manifest dependency `{dependency_name}` hash in {} must start with `sha256:`",
                manifest_path.display()
            ),
            Span::default(),
        )]);
    };
    if hex.len() != 64
        || !hex
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
    {
        return Err(vec![Diagnostic::new(
            "PK014",
            format!(
                "manifest dependency `{dependency_name}` hash in {} must be `sha256:` followed by 64 lowercase hexadecimal digits",
                manifest_path.display()
            ),
            Span::default(),
        )]);
    }
    Ok(())
}

fn unsupported_manifest_dependency_diagnostic(
    manifest_path: &Path,
    dependency_name: &str,
) -> Diagnostic {
    Diagnostic::new(
        "PK014",
        format!(
            "manifest dependency `{dependency_name}` in {} currently supports only local path or local archive forms",
            manifest_path.display()
        ),
        Span::default(),
    )
    .with_suggestion(format!(
        "use `{dependency_name} = {{ path = \"../{dependency_name}\" }}` or a local archive with `hash`"
    ))
}

fn split_manifest_inline_fields(value: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut start = 0;
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in value.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
        } else if ch == ',' {
            let field = value[start..index].trim();
            if !field.is_empty() {
                fields.push(field);
            }
            start = index + ch.len_utf8();
        }
    }
    let field = value[start..].trim();
    if !field.is_empty() {
        fields.push(field);
    }
    fields
}

fn parse_manifest_key(value: &str) -> Option<String> {
    if value.starts_with('"') {
        parse_manifest_string(value)
    } else {
        Some(value.to_string())
    }
}

fn strip_manifest_comment(value: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in value.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
        } else if ch == '#' {
            return &value[..index];
        }
    }
    value
}

fn parse_manifest_string(value: &str) -> Option<String> {
    let body = value.strip_prefix('"')?.strip_suffix('"')?;
    let mut parsed = String::new();
    let mut chars = body.chars();
    while let Some(ch) = chars.next() {
        if ch == '"' {
            return None;
        }
        if ch != '\\' {
            parsed.push(ch);
            continue;
        }
        let escaped = chars.next()?;
        match escaped {
            '"' => parsed.push('"'),
            '\\' => parsed.push('\\'),
            'n' => parsed.push('\n'),
            'r' => parsed.push('\r'),
            't' => parsed.push('\t'),
            _ => return None,
        }
    }
    Some(parsed)
}

fn manifest_identity(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn package_dir_under_root(
    package_path: &str,
    root_name: &str,
    source_root: &Path,
) -> Option<PathBuf> {
    if package_path == root_name {
        return Some(source_root.to_path_buf());
    }
    let rest = package_path.strip_prefix(&(root_name.to_string() + "::"))?;
    let mut path = source_root.to_path_buf();
    for segment in split_package_path(rest) {
        path.push(segment);
    }
    Some(path)
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

fn is_reserved_standard_package_path(path: &str) -> bool {
    path == "std" || path.starts_with("std::")
}

fn reserved_standard_package_diagnostic(path: &str, span: Span) -> Diagnostic {
    Diagnostic::new(
        "PK014",
        format!("package path `{path}` is reserved for the standard library"),
        span,
    )
    .with_suggestion("choose a non-`std` package path for user code")
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

fn mangle_opaque_type_name(package_path: &str, name: &str) -> String {
    format!("__muga_pkg__{}__{}", package_path.replace("::", "__"), name)
}

fn mangle_opaque_type_name_for_visibility(
    package_path: &str,
    module_path: &str,
    name: &str,
    visibility: Visibility,
) -> String {
    match visibility {
        Visibility::Private => mangle_module_item_name(package_path, module_path, name),
        Visibility::Package | Visibility::Public => mangle_opaque_type_name(package_path, name),
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

fn unknown_import_alias_diagnostic(alias: &str, span: Span) -> Diagnostic {
    let diagnostic = Diagnostic::new("PK009", format!("unknown import alias `{alias}`"), span);
    match alias {
        "cli" => diagnostic.with_suggestion("add `import std::cli` before using `cli::...`"),
        "env" => diagnostic.with_suggestion("add `import std::env` before using `env::...`"),
        "fmt" => diagnostic.with_suggestion("add `import std::fmt` before using `fmt::...`"),
        "fs" => diagnostic.with_suggestion("add `import std::fs` before using `fs::...`"),
        "io" => diagnostic.with_suggestion("add `import std::io` before using `io::...`"),
        "json" => diagnostic.with_suggestion("add `import std::json` before using `json::...`"),
        "path" => diagnostic.with_suggestion("add `import std::path` before using `path::...`"),
        "string" => {
            diagnostic.with_suggestion("add `import std::string` before using `string::...`")
        }
        "time" => diagnostic.with_suggestion("add `import std::time` before using `time::...`"),
        _ => {
            diagnostic.with_suggestion("add an import declaration or use an existing import alias")
        }
    }
}

fn fully_qualified_std_type_diagnostic(name: &str, span: Span) -> Option<Diagnostic> {
    let (module, item) = split_fully_qualified_std_item(name)?;
    Some(
        Diagnostic::new(
            "PK009",
            format!("cannot use full package path `{name}` as a type name"),
            span,
        )
        .with_suggestion(format!(
            "add `import std::{module}` and use `{module}::{item}`"
        )),
    )
}

fn split_fully_qualified_std_item(name: &str) -> Option<(&str, &str)> {
    let mut parts = name.split("::");
    let first = parts.next()?;
    let module = parts.next()?;
    let item = parts.next()?;
    if first == "std" && is_std_import_alias(module) && parts.next().is_none() {
        Some((module, item))
    } else {
        None
    }
}

fn is_std_import_alias(alias: &str) -> bool {
    matches!(alias, "env" | "fs" | "io" | "path" | "time")
}

fn is_builtin_name(name: &str) -> bool {
    crate::prelude::is_builtin_name(name)
}

fn is_mangled_item_name(name: &str) -> bool {
    name.starts_with("__muga_pkg__") || name.starts_with("__muga_mod__")
}

#[cfg(test)]
mod tests {
    use super::sha256_hex;

    #[test]
    fn sha256_hex_matches_known_digest() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
