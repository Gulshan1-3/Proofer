//! Proofer Package Manager & Project System (Stage 15).
//!
//! Provides:
//! - `proof.toml` project manifest specification.
//! - Project scaffolding (`proof new <path>`).
//! - Multi-file compilation & topological verification (`proof build`).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::parser::Parser;
use crate::hir::resolve::Resolver;
use crate::elab::Elaborator;
use crate::module::{ModuleLoader, ModuleError};

#[derive(Debug, Clone)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PackageManifest {
    pub package: PackageMetadata,
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug)]
pub enum PackageError {
    IoError(std::io::Error),
    ParseError(String),
    ModuleError(ModuleError),
    ManifestNotFound(PathBuf),
    VerificationFailed(String),
}

impl From<std::io::Error> for PackageError {
    fn from(e: std::io::Error) -> Self { PackageError::IoError(e) }
}
impl From<ModuleError> for PackageError {
    fn from(e: ModuleError) -> Self { PackageError::ModuleError(e) }
}

impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageError::IoError(e) => write!(f, "I/O error: {}", e),
            PackageError::ParseError(e) => write!(f, "Invalid proof.toml: {}", e),
            PackageError::ModuleError(e) => write!(f, "Module resolution error: {}", e),
            PackageError::ManifestNotFound(p) => write!(f, "No proof.toml found at {:?}", p),
            PackageError::VerificationFailed(msg) => write!(f, "Verification failed: {}", msg),
        }
    }
}

impl std::error::Error for PackageError {}

pub struct PackageManager;

impl PackageManager {
    /// Loads and parses a `proof.toml` file without external TOML dependencies.
    pub fn load_manifest(manifest_path: &Path) -> Result<PackageManifest, PackageError> {
        if !manifest_path.exists() {
            return Err(PackageError::ManifestNotFound(manifest_path.to_path_buf()));
        }
        let content = fs::read_to_string(manifest_path)?;
        Self::parse_manifest_str(&content)
    }

    pub fn parse_manifest_str(content: &str) -> Result<PackageManifest, PackageError> {
        let mut name = "unnamed".to_string();
        let mut version = "0.1.0".to_string();
        let mut edition = "2026".to_string();
        let mut authors = Vec::new();
        let mut description = None;
        let mut dependencies = HashMap::new();

        let mut current_section = "";

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                current_section = &line[1..line.len() - 1];
                continue;
            }

            if let Some(eq_idx) = line.find('=') {
                let key = line[..eq_idx].trim();
                let val_raw = line[eq_idx + 1..].trim();
                let val_str = val_raw.trim_matches('"');

                match current_section {
                    "package" => match key {
                        "name" => name = val_str.to_string(),
                        "version" => version = val_str.to_string(),
                        "edition" => edition = val_str.to_string(),
                        "description" => description = Some(val_str.to_string()),
                        "authors" => {
                            if val_raw.starts_with('[') && val_raw.ends_with(']') {
                                let inner = &val_raw[1..val_raw.len() - 1];
                                authors = inner.split(',')
                                    .map(|s| s.trim().trim_matches('"').to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                            }
                        }
                        _ => {}
                    },
                    "dependencies" => {
                        dependencies.insert(key.to_string(), val_str.to_string());
                    }
                    _ => {}
                }
            }
        }

        Ok(PackageManifest {
            package: PackageMetadata {
                name,
                version,
                edition,
                authors,
                description,
            },
            dependencies,
        })
    }

    /// Scaffolds a new Proofer project directory with `proof.toml` and sample proof.
    pub fn create_project(target_dir: &Path, name: &str) -> Result<(), PackageError> {
        fs::create_dir_all(target_dir)?;
        let src_dir = target_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        // 1. proof.toml
        let manifest_content = format!(
r#"[package]
name = "{}"
version = "0.1.0"
edition = "2026"
authors = ["Mathematician"]
description = "Formal proofs verified with Proofer"

[dependencies]
# mathlib = "../mathlib"
"#,
            name
        );
        fs::write(target_dir.join("proof.toml"), manifest_content)?;

        // 2. src/main.proof
        let sample_proof = r#"theorem pythagorean_prelude:
    AB = AC -> angle_ABC = angle_ACB
proof
    suppose h1 : AB = AC
    derive h2 : angle_ABC = angle_ACB from h1 using IsoscelesBaseAngles
    therefore angle_ABC = angle_ACB from h2
end
"#;
        fs::write(src_dir.join("main.proof"), sample_proof)?;

        // 3. .gitignore
        fs::write(target_dir.join(".gitignore"), "target/\n.proofer/\n")?;

        Ok(())
    }

    /// Builds and formally verifies all modules in a project.
    pub fn build_project(project_dir: &Path) -> Result<BuildSummary, PackageError> {
        let manifest_path = project_dir.join("proof.toml");
        let manifest = Self::load_manifest(&manifest_path)?;

        let mut loader = ModuleLoader::new();
        loader.add_base_dir(project_dir);
        let src_dir = project_dir.join("src");
        if src_dir.exists() {
            loader.add_base_dir(&src_dir);
        }

        // Collect all .proof files in project directory and subdirectories
        let mut proof_files = Vec::new();
        Self::collect_proof_files(project_dir, &mut proof_files)?;

        let mut total_theorems = 0;
        let mut verified_theorems = 0;
        let mut errors = Vec::new();

        for file_path in &proof_files {
            let content = fs::read_to_string(file_path)?;
            let mut parser = Parser::new(&content);
            let file_ast = parser.parse_file();

            if !parser.diagnostics().is_empty() {
                for d in parser.diagnostics() {
                    errors.push(format!("{}: syntax error: {}", file_path.display(), d.message));
                }
                continue;
            }

            let mut resolver = Resolver::new();
            let pkg = resolver.resolve_file(&file_ast);
            if !resolver.diagnostics().is_empty() {
                for d in resolver.diagnostics() {
                    errors.push(format!("{}: resolution error: {}", file_path.display(), d.message));
                }
                continue;
            }

            let mut elaborator = Elaborator::new();
            let results = elaborator.elaborate_package(&pkg);
            for res in results {
                total_theorems += 1;
                if res.proven.is_some() {
                    verified_theorems += 1;
                } else {
                    errors.push(format!("{}: theorem '{}' failed formal kernel verification", file_path.display(), res.name));
                }
            }
        }

        Ok(BuildSummary {
            package_name: manifest.package.name,
            version: manifest.package.version,
            files_count: proof_files.len(),
            theorems_count: total_theorems,
            verified_count: verified_theorems,
            errors,
        })
    }

    fn collect_proof_files(dir: &Path, list: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                // Defense-in-depth: Never traverse directory symlinks to avoid circular recursion loops
                if path.is_dir() && !path.is_symlink() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if dir_name != "target" && dir_name != "node_modules" && !dir_name.starts_with('.') {
                        Self::collect_proof_files(&path, list)?;
                    }
                } else if path.extension().and_then(|e| e.to_str()) == Some("proof") {
                    list.push(path);
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BuildSummary {
    pub package_name: String,
    pub version: String,
    pub files_count: usize,
    pub theorems_count: usize,
    pub verified_count: usize,
    pub errors: Vec<String>,
}
