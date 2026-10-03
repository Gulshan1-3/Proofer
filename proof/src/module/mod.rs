//! Module and Import System (Stage 11).
//!
//! Provides:
//! - Multi-file module resolution (`import path.to.module`).
//! - Directed acyclic dependency ordering for multi-file projects.
//! - Cycle detection in imports.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use crate::ast::FileAst;
use crate::parser::Parser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleId(pub String);

#[derive(Debug, Clone)]
pub struct Module {
    pub id: ModuleId,
    pub path: PathBuf,
    pub source: String,
    pub imports: Vec<ModuleId>,
    pub ast: FileAst,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleError {
    FileNotFound(PathBuf),
    CyclicDependency(Vec<ModuleId>),
    ParseError(String),
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleError::FileNotFound(p) => write!(f, "Module file not found: {:?}", p),
            ModuleError::CyclicDependency(cycle) => {
                let names: Vec<_> = cycle.iter().map(|m| m.0.as_str()).collect();
                write!(f, "Cyclic dependency detected: {}", names.join(" -> "))
            }
            ModuleError::ParseError(msg) => write!(f, "Failed to parse module: {}", msg),
        }
    }
}

impl std::error::Error for ModuleError {}

/// In-memory and filesystem module loader.
#[derive(Debug, Default)]
pub struct ModuleLoader {
    base_dirs: Vec<PathBuf>,
    in_memory_files: HashMap<String, String>,
}

impl ModuleLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_base_dir(&mut self, dir: impl Into<PathBuf>) {
        self.base_dirs.push(dir.into());
    }

    pub fn add_virtual_file(&mut self, module_name: &str, content: &str) {
        self.in_memory_files.insert(module_name.to_string(), content.to_string());
    }

    /// Load source for a given module name.
    pub fn load_source(&self, module_name: &str) -> Result<String, ModuleError> {
        if let Some(src) = self.in_memory_files.get(module_name) {
            return Ok(src.clone());
        }

        let rel_path = PathBuf::from(module_name.replace('.', "/")).with_extension("proof");
        for base in &self.base_dirs {
            let full_path = base.join(&rel_path);
            if full_path.exists() {
                return std::fs::read_to_string(&full_path)
                    .map_err(|_| ModuleError::FileNotFound(full_path));
            }
        }

        Err(ModuleError::FileNotFound(rel_path))
    }

    /// Parse a module and extract its syntactic imports.
    pub fn parse_module(&self, module_name: &str) -> Result<Module, ModuleError> {
        let source = self.load_source(module_name)?;
        let mut parser = Parser::new(&source);
        let ast = parser.parse_file();

        // Extract imports (syntax: lines starting with `import <name>`)
        let mut imports = Vec::new();
        for line in source.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("import ") {
                let imp_name = rest.trim().trim_end_matches(';').trim();
                if !imp_name.is_empty() {
                    imports.push(ModuleId(imp_name.to_string()));
                }
            }
        }

        Ok(Module {
            id: ModuleId(module_name.to_string()),
            path: PathBuf::from(module_name.replace('.', "/")),
            source,
            imports,
            ast,
        })
    }

    /// Recursively resolve dependencies of root module in topological order.
    pub fn resolve_project(&self, root_module: &str) -> Result<Vec<Module>, ModuleError> {
        let mut modules: HashMap<String, Module> = HashMap::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut on_stack: HashSet<String> = HashSet::new();
        let mut order: Vec<ModuleId> = Vec::new();

        fn dfs(
            loader: &ModuleLoader,
            mod_name: &str,
            visited: &mut HashSet<String>,
            on_stack: &mut HashSet<String>,
            order: &mut Vec<ModuleId>,
            modules: &mut HashMap<String, Module>,
        ) -> Result<(), ModuleError> {
            if on_stack.contains(mod_name) {
                return Err(ModuleError::CyclicDependency(vec![ModuleId(mod_name.to_string())]));
            }
            if visited.contains(mod_name) {
                return Ok(());
            }

            visited.insert(mod_name.to_string());
            on_stack.insert(mod_name.to_string());

            let module = loader.parse_module(mod_name)?;
            for dep in &module.imports {
                dfs(loader, &dep.0, visited, on_stack, order, modules)?;
            }

            on_stack.remove(mod_name);
            order.push(ModuleId(mod_name.to_string()));
            modules.insert(mod_name.to_string(), module);
            Ok(())
        }

        dfs(self, root_module, &mut visited, &mut on_stack, &mut order, &mut modules)?;

        let mut sorted = Vec::new();
        for id in order {
            if let Some(m) = modules.remove(&id.0) {
                sorted.push(m);
            }
        }

        Ok(sorted)
    }
}
