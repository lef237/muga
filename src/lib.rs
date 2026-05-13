pub mod ast;
pub mod bytecode;
pub mod cache;
pub mod diagnostic;
pub mod hir;
pub mod identity;
pub mod interface;
pub mod known_enum;
pub mod lexer;
pub mod package;
pub mod package_signature;
pub mod parser;
pub mod prelude;
pub mod resolver;
pub mod runtime;
pub mod span;
pub mod symbol;
pub mod token;
pub mod typed_hir;
pub mod types;
pub mod typing;

use ast::Program;
use bytecode::Program as BytecodeProgram;
use diagnostic::Diagnostic;
use hir::Program as HirProgram;
use interface::PackageInterfaceGraph;
use runtime::RunOutcome;
use std::path::{Path, PathBuf};
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
    pub type_output: typing::TypeCheckOutput,
    pub typed_program: TypedHirProgram,
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

pub fn check_path(path: &Path) -> Result<Program, Vec<Diagnostic>> {
    let program = package::load_program_from_entry(path)?;
    let mut diagnostics = resolver::resolve(&program);
    diagnostics.extend(typing::typecheck(&program));

    if diagnostics.is_empty() {
        Ok(program)
    } else {
        Err(diagnostics)
    }
}

pub fn compile_source(source: &str) -> Result<HirProgram, Vec<Diagnostic>> {
    let program = check_source(source)?;
    Ok(hir::lower(&program))
}

pub fn compile_path(path: &Path) -> Result<HirProgram, Vec<Diagnostic>> {
    let program = check_path(path)?;
    Ok(hir::lower(&program))
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
    let loaded = package::load_from_entry(path)?;
    compile_loaded_typed_program(loaded, None)
}

pub fn check_package_aware_path(path: &Path) -> Result<PackageAwareCheck, Vec<Diagnostic>> {
    let packages = package::load_package_graph_from_entry(path)?;
    let diagnostics = package::validate_loaded_package_graph(&packages);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let signatures = package_signature::PackageSignatureEnvironment::from_loaded_graph(&packages)?;
    let module_checks = typecheck_loaded_package_modules(&packages, &signatures)?;
    let typed_program = compile_typed_path(path)?;
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
    let typed_program =
        compile_typed_path_against_loaded_interfaces(path, interfaces, interface_symbols)?;
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
            let type_output =
                typing::typecheck_package_module(&file.program, signatures, module_id);
            let has_diagnostics = !type_output.diagnostics.is_empty();
            diagnostics.extend(type_output.diagnostics.clone());
            if has_diagnostics {
                continue;
            }
            let typed_program =
                typed_hir::lower(&file.program, &type_output, packages.package_graph.clone());
            module_checks.push(PackageModuleCheck {
                package: package_id,
                module: module_id,
                module_path: file.module_path.clone(),
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

pub fn compile_typed_path_against_interfaces(
    path: &Path,
    interfaces: &PackageInterfaceGraph,
) -> Result<TypedHirProgram, Vec<Diagnostic>> {
    let loaded = package::load_from_entry(path)?;
    compile_loaded_typed_program(loaded, Some(interfaces))
}

pub fn compile_typed_path_against_loaded_interfaces(
    path: &Path,
    interfaces: &PackageInterfaceGraph,
    interface_symbols: &symbol::SymbolTable,
) -> Result<TypedHirProgram, Vec<Diagnostic>> {
    let loaded = package::load_from_entry_against_interfaces(path, interfaces, interface_symbols)?;
    compile_loaded_typed_program_with_interface_symbols(loaded, interfaces, interface_symbols)
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
    let program = compile_typed_path(path)?;
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

    let mut written = Vec::new();
    let mut diagnostics = Vec::new();
    for package_path in requested_packages {
        match interfaces.write_persisted_artifact(artifact_root, &package_path, &program.symbols) {
            Ok(path) => written.push(path),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    if diagnostics.is_empty() {
        Ok(written)
    } else {
        Err(diagnostics)
    }
}

pub fn write_package_artifacts(
    path: &Path,
    artifact_root: &Path,
) -> Result<Vec<PathBuf>, Vec<Diagnostic>> {
    let mut paths = write_package_interface_artifacts(path, artifact_root, &[])?;
    paths.push(write_package_check_cache_artifact_for_root(
        path,
        artifact_root,
    )?);
    Ok(paths)
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

fn compile_loaded_typed_program(
    loaded: package::LoadedProgram,
    interfaces: Option<&PackageInterfaceGraph>,
) -> Result<TypedHirProgram, Vec<Diagnostic>> {
    compile_loaded_typed_program_inner(loaded, interfaces, None)
}

fn compile_loaded_typed_program_with_interface_symbols(
    loaded: package::LoadedProgram,
    interfaces: &PackageInterfaceGraph,
    interface_symbols: &symbol::SymbolTable,
) -> Result<TypedHirProgram, Vec<Diagnostic>> {
    compile_loaded_typed_program_inner(loaded, Some(interfaces), Some(interface_symbols))
}

fn compile_loaded_typed_program_inner(
    loaded: package::LoadedProgram,
    interfaces: Option<&PackageInterfaceGraph>,
    interface_symbols: Option<&symbol::SymbolTable>,
) -> Result<TypedHirProgram, Vec<Diagnostic>> {
    let resolve_output = resolver::resolve_program(&loaded.program);
    let type_output = typing::typecheck_program(&loaded.program);
    let mut diagnostics = resolve_output.diagnostics;
    diagnostics.extend(type_output.diagnostics.clone());
    if diagnostics.is_empty() {
        let program = typed_hir::lower(&loaded.program, &type_output, loaded.package_graph);
        let generated_interfaces;
        let normalized_interfaces;
        let interfaces = if let Some(interfaces) = interfaces {
            if let Some(interface_symbols) = interface_symbols {
                let mut symbols = program.symbols.clone();
                normalized_interfaces =
                    interfaces.reintern_symbols(interface_symbols, &mut symbols);
                &normalized_interfaces
            } else {
                interfaces
            }
        } else {
            generated_interfaces = program.package_interfaces();
            &generated_interfaces
        };
        diagnostics.extend(program.validate_package_references_against_interfaces(interfaces));
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
    let program = compile_source(source)?;
    Ok(bytecode::compile(program))
}

pub fn compile_bytecode_path(path: &Path) -> Result<BytecodeProgram, Vec<Diagnostic>> {
    let program = compile_path(path)?;
    Ok(bytecode::compile(program))
}

pub fn run_source(source: &str) -> Result<RunOutcome, Vec<Diagnostic>> {
    let program = compile_bytecode_source(source)?;
    runtime::run(&program)
}

pub fn run_path(path: &Path) -> Result<RunOutcome, Vec<Diagnostic>> {
    let program = compile_bytecode_path(path)?;
    runtime::run(&program)
}
