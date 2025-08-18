// Unified file scanning and filtering utilities

use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::config::Config;

/// Common file scanner with unified exclusion and filtering logic
pub struct FileScanner {
    config: Config,
}

impl FileScanner {
    /// Create a new file scanner with the given configuration
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// Create a file scanner with default configuration
    pub fn with_defaults() -> Self {
        Self {
            config: Config::load().unwrap_or_default(),
        }
    }

    /// Find files with specific extensions, applying all exclusion rules
    pub fn find_files_with_extensions(&self, dir: &Path, extensions: &[&str]) -> Vec<PathBuf> {
        WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| self.has_extension(e.path(), extensions))
            .filter(|e| !self.is_excluded_path(e.path()))
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    /// Find TypeScript/JavaScript files specifically
    pub fn find_js_ts_files(&self, dir: &Path) -> Vec<PathBuf> {
        self.find_files_with_extensions(dir, &["ts", "tsx", "js", "jsx"])
    }

    /// Check if a path should be excluded based on configuration
    pub fn is_excluded_path(&self, path: &Path) -> bool {
        // Check if any ancestor directory is excluded
        path.ancestors().any(|ancestor| {
            if let Some(name) = ancestor.file_name() {
                let name_str = name.to_string_lossy();
                
                // Check against configured excluded directories
                self.config.large_files.excluded_dirs.iter().any(|excluded| {
                    if excluded.contains('*') {
                        // Simple glob matching
                        let pattern = excluded.replace('*', ".*");
                        regex::Regex::new(&pattern)
                            .map(|re| re.is_match(&name_str))
                            .unwrap_or(false)
                    } else {
                        name_str == excluded.as_str()
                    }
                })
            } else {
                false
            }
        }) || self.is_excluded_file(path)
    }

    /// Check if a file should be excluded based on filename patterns
    pub fn is_excluded_file(&self, path: &Path) -> bool {
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            self.config.large_files.excluded_files.iter().any(|pattern| {
                if pattern.contains('*') {
                    let regex_pattern = pattern.replace('*', ".*");
                    regex::Regex::new(&regex_pattern)
                        .map(|re| re.is_match(file_name))
                        .unwrap_or(false)
                } else {
                    file_name == pattern
                }
            })
        } else {
            false
        }
    }

    /// Check if file has one of the specified extensions
    pub fn has_extension(&self, path: &Path, extensions: &[&str]) -> bool {
        if let Some(ext) = path.extension() {
            extensions.contains(&ext.to_string_lossy().as_ref())
        } else {
            false
        }
    }

    /// Check if a path is a TypeScript/JavaScript file
    pub fn is_js_ts_file(&self, path: &Path) -> bool {
        self.has_extension(path, &["ts", "tsx", "js", "jsx"])
    }
}

/// Legacy compatibility functions that delegate to FileScanner
pub fn is_excluded_path(path: &Path) -> bool {
    FileScanner::with_defaults().is_excluded_path(path)
}

pub fn find_js_ts_files(dir: &Path) -> Vec<PathBuf> {
    FileScanner::with_defaults().find_js_ts_files(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extension_checking() {
        let scanner = FileScanner::with_defaults();
        
        assert!(scanner.has_extension(&PathBuf::from("test.ts"), &["ts", "js"]));
        assert!(!scanner.has_extension(&PathBuf::from("test.py"), &["ts", "js"]));
    }

    #[test]
    fn test_js_ts_file_detection() {
        let scanner = FileScanner::with_defaults();
        
        assert!(scanner.is_js_ts_file(&PathBuf::from("component.tsx")));
        assert!(scanner.is_js_ts_file(&PathBuf::from("utils.js")));
        assert!(!scanner.is_js_ts_file(&PathBuf::from("styles.css")));
    }

    #[test]
    fn test_exclusion_patterns() {
        let scanner = FileScanner::with_defaults();
        
        // Should exclude node_modules by default
        assert!(scanner.is_excluded_path(&PathBuf::from("./node_modules/package/file.js")));
        assert!(scanner.is_excluded_path(&PathBuf::from("./project/node_modules/test.ts")));
        
        // Should not exclude regular source files
        assert!(!scanner.is_excluded_path(&PathBuf::from("./src/components/Test.tsx")));
    }
}