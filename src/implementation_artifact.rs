use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    diagnostic::Diagnostic,
    interface::{PackageInterfaceGraph, stable_hash_hex},
    package::{LoadedPackage, PackageSourceFile},
    span::Span,
    symbol::SymbolTable,
};

const PERSISTED_IMPLEMENTATION_HEADER: &str = "muga-package-implementation-v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageImplementationArtifact {
    pub package_path: String,
    pub interface_hash: String,
    pub dependency_interfaces: Vec<PackageImplementationDependencyHash>,
    pub files: Vec<PackageSourceFile>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageImplementationDependencyHash {
    pub package_path: String,
    pub interface_hash: String,
}

impl PackageImplementationArtifact {
    pub fn from_loaded_package(
        package: &LoadedPackage,
        interfaces: &PackageInterfaceGraph,
        symbols: &SymbolTable,
    ) -> Result<Self, Diagnostic> {
        let Some(interface) = interfaces.package_by_path(&package.path) else {
            return Err(implementation_artifact_diagnostic(format!(
                "compiled package interfaces do not contain `{}`",
                package.path
            )));
        };
        let Some(interface_hash) = interfaces.stable_hash_for_package(&package.path, symbols)
        else {
            return Err(implementation_artifact_diagnostic(format!(
                "missing package interface hash for `{}`",
                package.path
            )));
        };

        let mut dependency_interfaces = Vec::with_capacity(interface.dependencies.len());
        for dependency in &interface.dependencies {
            let Some(interface_hash) = interfaces.stable_hash_for_package(dependency, symbols)
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

        let mut files = package
            .files
            .iter()
            .map(|file| PackageSourceFile {
                module_path: file.module_path.clone(),
                source: file.source.clone(),
            })
            .collect::<Vec<_>>();
        files.sort_by(|left, right| left.module_path.cmp(&right.module_path));

        Ok(Self {
            package_path: package.path.clone(),
            interface_hash,
            dependency_interfaces,
            files,
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

        Self::from_body_lines(&lines[2..])
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

    fn from_body_lines(lines: &[&str]) -> Result<Self, Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let mut index = 0usize;

        let package_path = match next_prefixed_field(lines, &mut index, "package") {
            Ok(value) => value,
            Err(diagnostic) => return Err(vec![diagnostic]),
        };
        let interface_hash = match next_prefixed_field(lines, &mut index, "interface") {
            Ok(value) => value,
            Err(diagnostic) => return Err(vec![diagnostic]),
        };
        let dep_count = match next_count(lines, &mut index, "deps") {
            Ok(value) => value,
            Err(diagnostic) => return Err(vec![diagnostic]),
        };

        let mut dependency_interfaces = Vec::with_capacity(dep_count);
        for _ in 0..dep_count {
            let Some(line) = lines.get(index) else {
                diagnostics.push(implementation_artifact_diagnostic(
                    "package implementation artifact is missing dependency lines",
                ));
                break;
            };
            index += 1;
            let parts = line.split('\t').collect::<Vec<_>>();
            if parts.len() != 3 || parts[0] != "dep" {
                diagnostics.push(implementation_artifact_diagnostic(
                    "invalid package implementation dependency line",
                ));
                continue;
            }
            let package_path = match unescape_field(parts[1]) {
                Ok(value) => value,
                Err(diagnostic) => {
                    diagnostics.push(diagnostic);
                    continue;
                }
            };
            dependency_interfaces.push(PackageImplementationDependencyHash {
                package_path,
                interface_hash: parts[2].to_string(),
            });
        }

        let file_count = match next_count(lines, &mut index, "files") {
            Ok(value) => value,
            Err(diagnostic) => {
                diagnostics.push(diagnostic);
                0
            }
        };
        let mut files = Vec::with_capacity(file_count);
        for _ in 0..file_count {
            let Some(line) = lines.get(index) else {
                diagnostics.push(implementation_artifact_diagnostic(
                    "package implementation artifact is missing source file lines",
                ));
                break;
            };
            index += 1;
            let parts = line.split('\t').collect::<Vec<_>>();
            if parts.len() != 3 || parts[0] != "file" {
                diagnostics.push(implementation_artifact_diagnostic(
                    "invalid package implementation source file line",
                ));
                continue;
            }
            let module_path = match unescape_field(parts[1]) {
                Ok(value) => value,
                Err(diagnostic) => {
                    diagnostics.push(diagnostic);
                    continue;
                }
            };
            let source = match unescape_field(parts[2]) {
                Ok(value) => value,
                Err(diagnostic) => {
                    diagnostics.push(diagnostic);
                    continue;
                }
            };
            files.push(PackageSourceFile {
                module_path,
                source,
            });
        }

        if index != lines.len() {
            diagnostics.push(implementation_artifact_diagnostic(
                "package implementation artifact contains trailing data",
            ));
        }

        if diagnostics.is_empty() {
            Ok(Self {
                package_path,
                interface_hash,
                dependency_interfaces,
                files,
            })
        } else {
            Err(diagnostics)
        }
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
        out.push_str(&format!("files\t{}\n", self.files.len()));
        for file in &self.files {
            out.push_str(&format!(
                "file\t{}\t{}\n",
                escape_field(&file.module_path),
                escape_field(&file.source)
            ));
        }
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
    let mut artifacts = Vec::new();
    let mut diagnostics = Vec::new();

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
                artifacts.push(artifact);
            }
            Err(mut errors) => diagnostics.append(&mut errors),
        }
    }

    if diagnostics.is_empty() {
        Ok(artifacts)
    } else {
        Err(diagnostics)
    }
}

pub fn source_map_from_artifacts(
    artifacts: Vec<PackageImplementationArtifact>,
) -> HashMap<String, Vec<PackageSourceFile>> {
    artifacts
        .into_iter()
        .map(|artifact| (artifact.package_path, artifact.files))
        .collect()
}

pub fn persisted_file_path(root: &Path, package_path: &str) -> PathBuf {
    root.join(format!("{}.mgb", package_path.replace("::", "__")))
}

fn next_prefixed_field(
    lines: &[&str],
    index: &mut usize,
    prefix: &str,
) -> Result<String, Diagnostic> {
    let Some(line) = lines.get(*index) else {
        return Err(implementation_artifact_diagnostic(format!(
            "package implementation artifact is missing `{prefix}`"
        )));
    };
    *index += 1;
    let parts = line.split('\t').collect::<Vec<_>>();
    if parts.len() != 2 || parts[0] != prefix {
        return Err(implementation_artifact_diagnostic(format!(
            "invalid package implementation `{prefix}` line"
        )));
    }
    unescape_field(parts[1])
}

fn next_count(lines: &[&str], index: &mut usize, prefix: &str) -> Result<usize, Diagnostic> {
    let Some(line) = lines.get(*index) else {
        return Err(implementation_artifact_diagnostic(format!(
            "package implementation artifact is missing `{prefix}` count"
        )));
    };
    *index += 1;
    let parts = line.split('\t').collect::<Vec<_>>();
    if parts.len() != 2 || parts[0] != prefix {
        return Err(implementation_artifact_diagnostic(format!(
            "invalid package implementation `{prefix}` count line"
        )));
    }
    parts[1].parse::<usize>().map_err(|_| {
        implementation_artifact_diagnostic(format!(
            "invalid package implementation `{prefix}` count"
        ))
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

fn stale_implementation_artifact_diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("PK023", message, Span::default())
        .with_suggestion("regenerate the package implementation artifact")
}
