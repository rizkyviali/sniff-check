use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use super::types::{BrokenImport, BrokenImportType};
use super::resolver::PathAliasResolver;

pub fn import_exists(base_path: &PathBuf) -> bool {
    // Try the exact path first
    if base_path.exists() {
        return true;
    }
    
    // Try common JavaScript/TypeScript file extensions
    let extensions = [".js", ".ts", ".jsx", ".tsx", ".json", ".mjs", ".cjs"];
    
    for ext in extensions {
        let with_ext = base_path.with_extension(&ext[1..]);
        if with_ext.exists() {
            return true;
        }
    }
    
    // Try index files in the directory
    if base_path.is_dir() || !base_path.exists() {
        for ext in extensions {
            let index_file = base_path.join(format!("index{}", ext));
            if index_file.exists() {
                return true;
            }
        }
    }
    
    false
}

pub fn check_import_validity(
    current_file: &Path,
    project_root: &Path,
    import_path: &str,
    line_num: usize,
    import_statement: &str,
    path_resolver: &Option<PathAliasResolver>,
) -> Result<Option<BrokenImport>> {
    // First try to resolve TypeScript path aliases
    if !import_path.starts_with('.') {
        if let Some(resolver) = path_resolver {
            if let Some(resolved_path) = resolver.resolve_alias_path(import_path) {
                // Path alias pattern matched, check if file exists
                if import_exists(&resolved_path) {
                    return Ok(None); // Import is valid
                } else {
                    // Path alias resolved but file doesn't exist
                    return Ok(Some(BrokenImport {
                        file: current_file.to_string_lossy().to_string(),
                        line: line_num,
                        import_statement: import_statement.to_string(),
                        import_path: import_path.to_string(),
                        error_type: BrokenImportType::FileNotFound,
                        suggestion: Some(format!("Path alias '{}' resolves to '{}' but file not found", import_path, resolved_path.display())),
                    }));
                }
            }
        }
        
        // If not a path alias, check node_modules
        return check_node_modules_import(current_file, project_root, import_path, line_num, import_statement);
    }
    
    // Handle relative imports
    let current_dir = current_file.parent().unwrap();
    let resolved_path = resolve_import_path(current_dir, import_path);
    
    // Check if the resolved path exists (try common extensions)
    if !import_exists(&resolved_path) {
        // Try to find a suggestion
        let suggestion = find_similar_file(current_dir, import_path);
        
        return Ok(Some(BrokenImport {
            file: current_file.to_string_lossy().to_string(),
            line: line_num,
            import_statement: import_statement.to_string(),
            import_path: import_path.to_string(),
            error_type: BrokenImportType::FileNotFound,
            suggestion,
        }));
    }
    
    Ok(None)
}

fn check_node_modules_import(
    current_file: &Path,
    project_root: &Path,
    import_path: &str,
    line_num: usize,
    import_statement: &str,
) -> Result<Option<BrokenImport>> {
    // Extract package name (handle scoped packages like @types/node)
    let package_name = if import_path.starts_with('@') {
        // Scoped package: @scope/package or @scope/package/subpath
        let parts: Vec<&str> = import_path.splitn(3, '/').collect();
        if parts.len() >= 2 {
            format!("{}/{}", parts[0], parts[1])
        } else {
            import_path.to_string()
        }
    } else {
        // Regular package: package or package/subpath
        import_path.split('/').next().unwrap_or(import_path).to_string()
    };
    
    // Check if package exists in node_modules
    let node_modules_path = project_root.join("node_modules").join(&package_name);
    if !node_modules_path.exists() {
        return Ok(Some(BrokenImport {
            file: current_file.to_string_lossy().to_string(),
            line: line_num,
            import_statement: import_statement.to_string(),
            import_path: import_path.to_string(),
            error_type: BrokenImportType::ModuleNotInstalled,
            suggestion: Some(format!("Run: npm install {}", package_name)),
        }));
    }
    
    Ok(None)
}

fn resolve_import_path(current_dir: &Path, import_path: &str) -> PathBuf {
    let mut resolved = current_dir.to_path_buf();
    
    // Split path and navigate
    for part in import_path.split('/') {
        match part {
            "." => {} // Current directory, do nothing
            ".." => { resolved.pop(); }
            _ => resolved.push(part),
        }
    }
    
    resolved
}

fn find_similar_file(current_dir: &Path, import_path: &str) -> Option<String> {
    // Extract the filename from the import path
    let filename = import_path.split('/').last()?;
    
    // Look for similar files in the current directory and parent directories
    let search_dirs = [current_dir, current_dir.parent()?];
    
    for search_dir in search_dirs {
        if let Ok(entries) = fs::read_dir(search_dir) {
            for entry in entries.flatten() {
                if let Some(entry_name) = entry.file_name().to_str() {
                    // Remove extension for comparison
                    let entry_path = entry.path();
                    if let Some(entry_stem) = entry_path.file_stem().and_then(|s| s.to_str()) {
                        if entry_stem.to_lowercase().contains(&filename.to_lowercase()) {
                            // Build relative path suggestion
                            let relative = if search_dir == current_dir {
                                format!("./{}", entry_name)
                            } else {
                                format!("../{}", entry_name)
                            };
                            return Some(relative);
                        }
                    }
                }
            }
        }
    }
    
    None
}