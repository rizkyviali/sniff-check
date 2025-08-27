use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::commands::imports_analyzer::validation::import_exists;

#[derive(Debug, Deserialize)]
struct TsConfigCompilerOptions {
    #[serde(rename = "baseUrl")]
    base_url: Option<String>,
    paths: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Deserialize)]
struct TsConfig {
    #[serde(rename = "compilerOptions")]
    compiler_options: Option<TsConfigCompilerOptions>,
}

pub struct PathAliasResolver {
    #[allow(dead_code)]
    base_url: PathBuf,
    path_mappings: HashMap<String, Vec<PathBuf>>,
}

impl PathAliasResolver {
    pub fn from_project_root(project_root: &Path) -> Option<Self> {
        let tsconfig_path = project_root.join("tsconfig.json");
        if !tsconfig_path.exists() {
            return None;
        }
        
        let content = fs::read_to_string(&tsconfig_path).ok()?;
        let tsconfig: TsConfig = serde_json::from_str(&content).ok()?;
        
        let compiler_options = tsconfig.compiler_options?;
        let base_url = compiler_options.base_url
            .map(|base| project_root.join(base))
            .unwrap_or_else(|| project_root.to_path_buf());
        
        let mut path_mappings = HashMap::new();
        if let Some(paths) = compiler_options.paths {
            for (pattern, targets) in paths {
                let resolved_targets: Vec<PathBuf> = targets
                    .into_iter()
                    .map(|target| base_url.join(target.trim_end_matches("/*")))
                    .collect();
                path_mappings.insert(pattern, resolved_targets);
            }
        }
        
        Some(Self {
            base_url,
            path_mappings,
        })
    }
    
    pub fn resolve_alias_path(&self, import_path: &str) -> Option<PathBuf> {
        for (pattern, targets) in &self.path_mappings {
            if let Some(resolved) = self.try_resolve_pattern(pattern, import_path, targets) {
                return Some(resolved);
            }
        }
        None
    }
    
    fn try_resolve_pattern(&self, pattern: &str, import_path: &str, targets: &[PathBuf]) -> Option<PathBuf> {
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            if import_path.starts_with(prefix) {
                let suffix = &import_path[prefix.len()..];
                if let Some(target) = targets.first() {
                    let resolved = target.join(suffix.trim_start_matches('/'));
                    return Some(resolved);
                }
            }
        } else if pattern == import_path {
            if let Some(target) = targets.first() {
                return Some(target.clone());
            }
        }
        None
    }
    
    #[allow(dead_code)]
    fn resolve_alias(&self, import_path: &str) -> Option<PathBuf> {
        for (pattern, targets) in &self.path_mappings {
            if let Some(resolved) = self.try_match_pattern(pattern, import_path, targets) {
                return Some(resolved);
            }
        }
        None
    }
    
    #[allow(dead_code)]
    fn try_match_pattern(&self, pattern: &str, import_path: &str, targets: &[PathBuf]) -> Option<PathBuf> {
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            if import_path.starts_with(prefix) {
                let suffix = &import_path[prefix.len()..];
                for target in targets {
                    let resolved = target.join(suffix.trim_start_matches('/'));
                    if import_exists(&resolved) {
                        return Some(resolved);
                    }
                }
            }
        } else if pattern == import_path {
            for target in targets {
                if import_exists(target) {
                    return Some(target.clone());
                }
            }
        }
        None
    }
}