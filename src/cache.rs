use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    diagnostic::{
        Diagnostic, artifact_file_context, artifact_hash_context, regeneration_command_context,
    },
    interface::{PackageInterfaceGraph, stable_hash_hex},
    package,
    span::Span,
    symbol::SymbolTable,
};

const PERSISTED_CHECK_HEADER: &str = "muga-package-check-v1";
const REGENERATE_CHECK_CACHE_SUGGESTION: &str = "regenerate package check caches with `muga build`, `muga emit-artifacts`, or `muga emit-check-cache`";
const REGENERATE_CHECK_CACHE_COMMANDS: [(&str, &str); 3] = [
    ("default-build", "muga build <entry>"),
    (
        "artifact-root",
        "muga emit-artifacts --artifact-root <dir> <entry>",
    ),
    (
        "check-cache",
        "muga emit-check-cache --artifact-root <dir> <entry>",
    ),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageCheckCacheKey {
    pub source_hash: String,
    pub dependency_interfaces: Vec<PackageDependencyInterfaceHash>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageDependencyInterfaceHash {
    pub package_path: String,
    pub interface_hash: String,
}

impl PackageCheckCacheKey {
    pub fn stable_hash(&self) -> String {
        stable_hash_hex(&self.body_text())
    }

    pub fn to_persisted_text(&self) -> String {
        let body = self.body_text();
        format!(
            "{PERSISTED_CHECK_HEADER}\nhash\t{}\n{body}",
            stable_hash_hex(&body)
        )
    }

    pub fn from_persisted_text(text: &str) -> Result<Self, Vec<Diagnostic>> {
        let lines = text.lines().collect::<Vec<_>>();
        if lines.first().copied() != Some(PERSISTED_CHECK_HEADER) {
            return Err(vec![cache_artifact_diagnostic(
                "invalid package check cache artifact header",
            )]);
        }
        let Some(expected_hash) = lines.get(1).and_then(|line| line.strip_prefix("hash\t")) else {
            return Err(vec![cache_artifact_diagnostic(
                "package check cache artifact is missing a hash",
            )]);
        };
        let body = if lines.len() > 2 {
            format!("{}\n", lines[2..].join("\n"))
        } else {
            String::new()
        };
        let actual_hash = stable_hash_hex(&body);
        if expected_hash != actual_hash {
            let mut diagnostic = Diagnostic::new(
                "PK020",
                format!(
                    "package check cache artifact hash mismatch: expected `{expected_hash}` but found `{actual_hash}`"
                ),
                Span::default(),
            )
            .with_suggestion(REGENERATE_CHECK_CACHE_SUGGESTION)
            .with_context(artifact_hash_context(
                "expected",
                "artifact",
                None,
                expected_hash,
            ))
            .with_context(artifact_hash_context(
                "actual",
                "artifact",
                None,
                actual_hash,
            ));
            add_check_cache_regeneration_context(&mut diagnostic);
            return Err(vec![diagnostic]);
        }

        Self::from_body_lines(&lines[2..])
    }

    fn from_body_lines(lines: &[&str]) -> Result<Self, Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let Some(source_parts) = lines
            .first()
            .map(|line| line.split('\t').collect::<Vec<_>>())
        else {
            return Err(vec![cache_artifact_diagnostic(
                "package check cache artifact is missing a source hash",
            )]);
        };
        if source_parts.len() != 2 || source_parts[0] != "source" {
            return Err(vec![cache_artifact_diagnostic(
                "invalid package check cache source line",
            )]);
        }
        let Some(deps_parts) = lines
            .get(1)
            .map(|line| line.split('\t').collect::<Vec<_>>())
        else {
            return Err(vec![cache_artifact_diagnostic(
                "package check cache artifact is missing dependency count",
            )]);
        };
        if deps_parts.len() != 2 || deps_parts[0] != "deps" {
            return Err(vec![cache_artifact_diagnostic(
                "invalid package check cache dependency count line",
            )]);
        }
        let dep_count = match deps_parts[1].parse::<usize>() {
            Ok(count) => count,
            Err(_) => {
                return Err(vec![cache_artifact_diagnostic(
                    "invalid package check cache dependency count",
                )]);
            }
        };
        if lines.len() != dep_count + 2 {
            diagnostics.push(cache_artifact_diagnostic(
                "package check cache artifact dependency count does not match its body",
            ));
        }

        let mut dependency_interfaces = Vec::with_capacity(dep_count);
        for line in lines.iter().skip(2).take(dep_count) {
            let parts = line.split('\t').collect::<Vec<_>>();
            if parts.len() != 3 || parts[0] != "dep" {
                diagnostics.push(cache_artifact_diagnostic(
                    "invalid package check cache dependency line",
                ));
                continue;
            }
            dependency_interfaces.push(PackageDependencyInterfaceHash {
                package_path: parts[1].to_string(),
                interface_hash: parts[2].to_string(),
            });
        }

        if diagnostics.is_empty() {
            Ok(Self {
                source_hash: source_parts[1].to_string(),
                dependency_interfaces,
            })
        } else {
            Err(diagnostics)
        }
    }

    fn body_text(&self) -> String {
        let mut dependency_interfaces = self.dependency_interfaces.clone();
        dependency_interfaces.sort_by(|left, right| left.package_path.cmp(&right.package_path));

        let mut out = String::new();
        out.push_str(&format!("source\t{}\n", self.source_hash));
        out.push_str(&format!("deps\t{}\n", dependency_interfaces.len()));
        for dependency in dependency_interfaces {
            out.push_str(&format!(
                "dep\t{}\t{}\n",
                dependency.package_path, dependency.interface_hash
            ));
        }
        out
    }
}

pub fn compute_package_check_cache_key(
    entry_path: &Path,
    interface_root: &Path,
) -> Result<PackageCheckCacheKey, Vec<Diagnostic>> {
    let source_input = package::source_fingerprint_input_from_entry(entry_path)?;
    let source_hash = stable_hash_hex(&source_input);
    let package_paths = package::import_paths_from_entry(entry_path)?;
    let mut symbols = SymbolTable::default();
    let interfaces = PackageInterfaceGraph::read_persisted_artifacts(
        interface_root,
        &package_paths,
        &mut symbols,
    )?;

    let mut dependency_interfaces = Vec::with_capacity(interfaces.packages.len());
    let mut diagnostics = Vec::new();
    for package_path in interfaces
        .packages
        .iter()
        .map(|interface| interface.path.clone())
    {
        match interfaces.stable_hash_for_package(&package_path, &symbols) {
            Some(interface_hash) => dependency_interfaces.push(PackageDependencyInterfaceHash {
                package_path,
                interface_hash,
            }),
            None => diagnostics.push(
                Diagnostic::new(
                    "PK016",
                    format!("missing loaded package interface for `{package_path}`"),
                    Span::default(),
                )
                .with_suggestion("load or regenerate the package interface before checking"),
            ),
        }
    }

    if diagnostics.is_empty() {
        dependency_interfaces.sort_by(|left, right| left.package_path.cmp(&right.package_path));
        Ok(PackageCheckCacheKey {
            source_hash,
            dependency_interfaces,
        })
    } else {
        Err(diagnostics)
    }
}

pub fn package_check_artifact_path(root: &Path, package_path: &str) -> PathBuf {
    root.join(format!("{}.mgc", package_path.replace("::", "__")))
}

pub fn package_check_artifact_path_from_entry(
    root: &Path,
    entry_path: &Path,
) -> Result<PathBuf, Vec<Diagnostic>> {
    let package_path = package::entry_package_path_from_entry(entry_path)?;
    match package_path {
        Some(package_path) => Ok(package_check_artifact_path(root, &package_path)),
        None => Err(vec![
            Diagnostic::new(
                "PK001",
                "artifact-backed checking requires a package-mode entrypoint",
                Span::default(),
            )
            .with_suggestion("remove `--artifact-root` or check a package entrypoint"),
        ]),
    }
}

pub fn write_package_check_artifact(
    path: &Path,
    key: &PackageCheckCacheKey,
) -> Result<(), Diagnostic> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            Diagnostic::new(
                "PK020",
                format!(
                    "failed to create package check cache artifact directory {}: {error}",
                    parent.display()
                ),
                Span::default(),
            )
        })?;
    }
    fs::write(path, key.to_persisted_text()).map_err(|error| {
        Diagnostic::new(
            "PK020",
            format!(
                "failed to write package check cache artifact `{}`: {error}",
                path.display()
            ),
            Span::default(),
        )
        .with_context(check_cache_artifact_file_context(path))
    })
}

pub fn read_package_check_artifact(path: &Path) -> Result<PackageCheckCacheKey, Vec<Diagnostic>> {
    let text = fs::read_to_string(path).map_err(|error| {
        vec![
            Diagnostic::new(
                "PK020",
                format!(
                    "failed to read package check cache artifact `{}`: {error}",
                    path.display()
                ),
                Span::default(),
            )
            .with_context(check_cache_artifact_file_context(path)),
        ]
    })?;
    PackageCheckCacheKey::from_persisted_text(&text).map_err(|mut diagnostics| {
        add_check_cache_artifact_file_context(&mut diagnostics, path);
        diagnostics
    })
}

pub fn validate_package_check_artifact(
    path: &Path,
    expected: &PackageCheckCacheKey,
) -> Result<(), Vec<Diagnostic>> {
    if !path.is_file() {
        let mut diagnostic = Diagnostic::new(
            "PK020",
            format!("missing package check cache artifact `{}`", path.display()),
            Span::default(),
        )
        .with_context(check_cache_artifact_file_context(path))
        .with_suggestion(REGENERATE_CHECK_CACHE_SUGGESTION);
        add_check_cache_regeneration_context(&mut diagnostic);
        return Err(vec![diagnostic]);
    }

    let actual = read_package_check_artifact(path)?;
    if actual.stable_hash() == expected.stable_hash() {
        Ok(())
    } else {
        let details = cache_key_difference_details(expected, &actual);
        let reason = if details.is_empty() {
            "cache inputs changed".to_string()
        } else {
            details.join("; ")
        };
        let expected_hash = expected.stable_hash();
        let actual_hash = actual.stable_hash();
        let mut diagnostic = Diagnostic::new(
            "PK021",
            format!(
                "stale package check cache artifact `{}`: {reason}; expected `{expected_hash}` but found `{actual_hash}`",
                path.display()
            ),
            Span::default(),
        )
        .with_context(check_cache_artifact_file_context(path))
        .with_context(artifact_hash_context(
            "expected",
            "artifact",
            None,
            &expected_hash,
        ))
        .with_context(artifact_hash_context(
            "actual",
            "artifact",
            None,
            &actual_hash,
        ))
        .with_suggestion(REGENERATE_CHECK_CACHE_SUGGESTION);
        add_check_cache_input_context(&mut diagnostic, expected, &actual);
        add_check_cache_regeneration_context(&mut diagnostic);
        Err(vec![diagnostic])
    }
}

fn cache_key_difference_details(
    expected: &PackageCheckCacheKey,
    actual: &PackageCheckCacheKey,
) -> Vec<String> {
    let mut details = Vec::new();
    if expected.source_hash != actual.source_hash {
        details.push("entry package source changed".to_string());
    }

    let expected_dependencies = dependency_hashes_by_path(expected);
    let actual_dependencies = dependency_hashes_by_path(actual);

    for (package_path, expected_hash) in &expected_dependencies {
        match actual_dependencies.get(package_path) {
            Some(actual_hash) if actual_hash == expected_hash => {}
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

fn dependency_hashes_by_path(key: &PackageCheckCacheKey) -> BTreeMap<&str, &str> {
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

fn cache_artifact_diagnostic(message: impl Into<String>) -> Diagnostic {
    let mut diagnostic = Diagnostic::new("PK020", message, Span::default())
        .with_suggestion(REGENERATE_CHECK_CACHE_SUGGESTION);
    add_check_cache_regeneration_context(&mut diagnostic);
    diagnostic
}

fn check_cache_artifact_file_context(path: &Path) -> crate::diagnostic::DiagnosticContext {
    artifact_file_context("check-cache", "checkCache", path)
}

fn add_check_cache_artifact_file_context(diagnostics: &mut [Diagnostic], path: &Path) {
    for diagnostic in diagnostics {
        diagnostic.add_context(check_cache_artifact_file_context(path));
    }
}

fn add_check_cache_regeneration_context(diagnostic: &mut Diagnostic) {
    for (role, command) in REGENERATE_CHECK_CACHE_COMMANDS {
        diagnostic.add_context(regeneration_command_context(role, command));
    }
}

fn add_check_cache_input_context(
    diagnostic: &mut Diagnostic,
    expected: &PackageCheckCacheKey,
    actual: &PackageCheckCacheKey,
) {
    if expected.source_hash != actual.source_hash {
        diagnostic.add_context(artifact_hash_context(
            "expected",
            "source",
            None,
            &expected.source_hash,
        ));
        diagnostic.add_context(artifact_hash_context(
            "actual",
            "source",
            None,
            &actual.source_hash,
        ));
    }

    let expected_dependencies = dependency_hashes_by_path(expected);
    let actual_dependencies = dependency_hashes_by_path(actual);

    for (package_path, expected_hash) in &expected_dependencies {
        match actual_dependencies.get(package_path) {
            Some(actual_hash) if actual_hash == expected_hash => {}
            Some(actual_hash) => {
                diagnostic.add_context(artifact_hash_context(
                    "expected",
                    "dependencyInterface",
                    Some(*package_path),
                    *expected_hash,
                ));
                diagnostic.add_context(artifact_hash_context(
                    "actual",
                    "dependencyInterface",
                    Some(*package_path),
                    *actual_hash,
                ));
            }
            None => diagnostic.add_context(artifact_hash_context(
                "expected",
                "dependencyInterface",
                Some(*package_path),
                *expected_hash,
            )),
        }
    }

    for (package_path, actual_hash) in actual_dependencies {
        if !expected_dependencies.contains_key(package_path) {
            diagnostic.add_context(artifact_hash_context(
                "actual",
                "dependencyInterface",
                Some(package_path),
                actual_hash,
            ));
        }
    }
}
