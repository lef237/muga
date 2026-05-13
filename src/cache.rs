use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    diagnostic::Diagnostic,
    interface::{PackageInterfaceGraph, stable_hash_hex},
    package,
    span::Span,
    symbol::SymbolTable,
};

const PERSISTED_CHECK_HEADER: &str = "muga-package-check-v1";

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
            return Err(vec![
                Diagnostic::new(
                    "PK020",
                    format!(
                        "package check cache artifact hash mismatch: expected `{expected_hash}` but found `{actual_hash}`"
                    ),
                    Span::default(),
                )
                .with_suggestion("regenerate the package check cache artifact"),
            ]);
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
    })
}

pub fn read_package_check_artifact(path: &Path) -> Result<PackageCheckCacheKey, Vec<Diagnostic>> {
    let text = fs::read_to_string(path).map_err(|error| {
        vec![Diagnostic::new(
            "PK020",
            format!(
                "failed to read package check cache artifact `{}`: {error}",
                path.display()
            ),
            Span::default(),
        )]
    })?;
    PackageCheckCacheKey::from_persisted_text(&text)
}

pub fn validate_package_check_artifact(
    path: &Path,
    expected: &PackageCheckCacheKey,
) -> Result<(), Vec<Diagnostic>> {
    if !path.is_file() {
        return Err(vec![
            Diagnostic::new(
                "PK020",
                format!("missing package check cache artifact `{}`", path.display()),
                Span::default(),
            )
            .with_suggestion("regenerate the package check cache artifact"),
        ]);
    }

    let actual = read_package_check_artifact(path)?;
    if actual.stable_hash() == expected.stable_hash() {
        Ok(())
    } else {
        Err(vec![
            Diagnostic::new(
                "PK021",
                format!(
                    "stale package check cache artifact `{}`: expected `{}` but found `{}`",
                    path.display(),
                    expected.stable_hash(),
                    actual.stable_hash()
                ),
                Span::default(),
            )
            .with_suggestion("regenerate the package check cache artifact"),
        ])
    }
}

fn cache_artifact_diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("PK020", message, Span::default())
        .with_suggestion("regenerate the package check cache artifact")
}
