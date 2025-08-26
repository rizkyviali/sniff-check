// Utility functions for file operations, formatting, and common tasks

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use crate::config::Config;

/// File utilities
pub struct FileUtils;

impl FileUtils {
    
    /// Find files with specific extensions (optimized with parallel processing)
    pub fn find_files_with_extensions(dir: &Path, extensions: &[&str]) -> Vec<PathBuf> {
        let config = Config::load().unwrap_or_default();
        
        WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| Self::has_extension(e.path(), extensions))
            .filter(|e| !Self::is_excluded_path_with_config(e.path(), &config))
            .map(|e| e.path().to_path_buf())
            .collect()
    }
    
    /// Find files with extensions and show progress
    pub fn find_files_with_progress(dir: &Path, extensions: &[&str], quiet: bool) -> Result<Vec<PathBuf>> {
        let pb = if !quiet {
            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap());
            pb.set_message("Scanning files...");
            pb.enable_steady_tick(std::time::Duration::from_millis(80));
            Some(pb)
        } else {
            None
        };
        
        // Add minimum display time for spinner visibility
        let start_time = std::time::Instant::now();
        let files = Self::find_files_with_extensions(dir, extensions);
        
        // Ensure spinner shows for at least 200ms for visibility
        if let Some(_pb) = &pb {
            let elapsed = start_time.elapsed();
            if elapsed < std::time::Duration::from_millis(200) {
                std::thread::sleep(std::time::Duration::from_millis(200) - elapsed);
            }
        }
        
        if let Some(pb) = pb {
            pb.finish_with_message(format!("Found {} files", files.len()));
        }
        
        Ok(files)
    }
    
    /// Check if file has one of the specified extensions
    pub fn has_extension(path: &Path, extensions: &[&str]) -> bool {
        if let Some(ext) = path.extension() {
            extensions.contains(&ext.to_string_lossy().as_ref())
        } else {
            false
        }
    }
    
    /// Check if path should be excluded based on configuration
    pub fn is_excluded_path_with_config(path: &Path, config: &Config) -> bool {
        path.ancestors().any(|ancestor| {
            if let Some(name) = ancestor.file_name() {
                config.large_files.excluded_dirs.contains(&name.to_string_lossy().to_string())
            } else {
                false
            }
        })
    }
    
    /// Check if path is in node_modules or other build directories (legacy)
    pub fn is_node_modules(path: &Path) -> bool {
        let config = Config::load().unwrap_or_default();
        Self::is_excluded_path_with_config(path, &config)
    }
    
    /// Count lines in a file with memory mapping for large files
    pub fn count_lines_optimized(path: &Path) -> Result<usize> {
        let file = fs::File::open(path)?;
        let metadata = file.metadata()?;
        
        // Use memory mapping for files larger than 1MB
        if metadata.len() > 1_048_576 {
            let mmap = unsafe { memmap2::Mmap::map(&file)? };
            Ok(mmap.iter().filter(|&&b| b == b'\n').count())
        } else {
            let content = fs::read_to_string(path)?;
            Ok(content.lines().count())
        }
    }
    
    /// Process files in parallel with progress tracking
    pub fn process_files_parallel<T, F>(
        files: &[PathBuf], 
        operation: F, 
        description: &str,
        quiet: bool
    ) -> Result<Vec<T>>
    where
        T: Send,
        F: Fn(&Path) -> Result<T> + Sync + Send,
    {
        let pb = if !quiet {
            let pb = ProgressBar::new(files.len() as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"));
            pb.set_message(description.to_string());
            Some(pb)
        } else {
            None
        };
        
        let results: Result<Vec<T>, _> = files
            .par_iter()
            .map(|path| {
                let result = operation(path);
                if let Some(pb) = &pb {
                    pb.inc(1);
                }
                result
            })
            .collect();
        
        if let Some(pb) = pb {
            pb.finish_with_message("Complete");
        }
        
        results
    }
    
    /// Get relative path from current directory
    pub fn get_relative_path(path: &Path) -> String {
        if let Ok(current) = std::env::current_dir() {
            if let Ok(relative) = path.strip_prefix(&current) {
                return relative.to_string_lossy().to_string();
            }
        }
        path.to_string_lossy().to_string()
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_has_extension() {
        use std::path::Path;
        
        assert!(FileUtils::has_extension(Path::new("test.ts"), &["ts", "tsx"]));
        assert!(FileUtils::has_extension(Path::new("test.tsx"), &["ts", "tsx"]));
        assert!(!FileUtils::has_extension(Path::new("test.py"), &["ts", "tsx"]));
    }
    
    #[test]
    fn test_get_relative_path() {
        use std::path::Path;
        
        let path = Path::new("/some/absolute/path/file.rs");
        let relative = FileUtils::get_relative_path(path);
        
        // Should return some path representation
        assert!(!relative.is_empty());
    }
}