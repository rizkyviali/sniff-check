// Utility functions for file operations, formatting, and common tasks

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use colored::*;
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use crate::config::Config;

/// File utilities
pub struct FileUtils;

impl FileUtils {
    /// Count lines in a file, excluding empty lines and comments
    pub fn count_meaningful_lines(content: &str) -> usize {
        content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("//") && !line.starts_with("/*"))
            .count()
    }
    
    /// Count total lines in a file
    pub fn count_total_lines(content: &str) -> usize {
        content.lines().count()
    }
    
    /// Check if a file is a TypeScript/JavaScript file
    pub fn is_js_ts_file(path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            matches!(extension.to_str(), Some("ts") | Some("tsx") | Some("js") | Some("jsx"))
        } else {
            false
        }
    }
    
    /// Get file size in bytes
    pub fn get_file_size(path: &Path) -> Result<u64, std::io::Error> {
        let metadata = fs::metadata(path)?;
        Ok(metadata.len())
    }
    
    /// Format file size in human readable format
    pub fn format_file_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
    
    /// Find all TypeScript/JavaScript files in a directory
    pub fn find_js_ts_files(dir: &Path) -> Vec<std::path::PathBuf> {
        Self::find_files_with_extensions(dir, &["ts", "tsx", "js", "jsx"])
    }
    
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
            Some(pb)
        } else {
            None
        };
        
        let files = Self::find_files_with_extensions(dir, extensions);
        
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

/// Formatting utilities
pub struct FormatUtils;

impl FormatUtils {
    /// Format duration in milliseconds to human readable
    pub fn format_duration(ms: u64) -> String {
        if ms < 1000 {
            format!("{}ms", ms)
        } else if ms < 60_000 {
            format!("{:.1}s", ms as f64 / 1000.0)
        } else {
            let minutes = ms / 60_000;
            let seconds = (ms % 60_000) / 1000;
            format!("{}m {}s", minutes, seconds)
        }
    }
    
    /// Format percentage with color coding
    pub fn format_percentage(value: f64, good_threshold: f64, poor_threshold: f64) -> colored::ColoredString {
        let formatted = format!("{:.1}%", value);
        
        if value >= good_threshold {
            formatted.green()
        } else if value >= poor_threshold {
            formatted.yellow()
        } else {
            formatted.red()
        }
    }
    
    /// Create a progress bar string
    pub fn create_progress_bar(current: usize, total: usize, width: usize) -> String {
        if total == 0 {
            return "█".repeat(width);
        }
        
        let filled = (current * width) / total;
        let empty = width - filled;
        
        format!("{}{}",
            "█".repeat(filled),
            "░".repeat(empty)
        )
    }
    
    /// Format number with thousands separator
    pub fn format_number(num: usize) -> String {
        let num_str = num.to_string();
        let chars: Vec<char> = num_str.chars().collect();
        let mut result = String::new();
        
        for (i, &ch) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i) % 3 == 0 {
                result.push(',');
            }
            result.push(ch);
        }
        
        result
    }
    
    /// Truncate string with ellipsis
    pub fn truncate_string(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }
}

/// Validation utilities
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validate file path exists
    pub fn validate_path(path: &str) -> Result<(), String> {
        if Path::new(path).exists() {
            Ok(())
        } else {
            Err(format!("Path does not exist: {}", path))
        }
    }
    
    /// Validate threshold value
    pub fn validate_threshold(threshold: usize) -> Result<(), String> {
        if threshold == 0 {
            Err("Threshold must be greater than 0".to_string())
        } else if threshold > 10000 {
            Err("Threshold too large (max: 10000)".to_string())
        } else {
            Ok(())
        }
    }
    
    /// Validate percentage value
    pub fn validate_percentage(value: f64) -> Result<(), String> {
        if !(0.0..=100.0).contains(&value) {
            Err("Percentage must be between 0 and 100".to_string())
        } else {
            Ok(())
        }
    }
    
    /// Validate URL format
    pub fn validate_url(url: &str) -> Result<(), String> {
        if url.starts_with("http://") || url.starts_with("https://") {
            Ok(())
        } else {
            Err("URL must start with http:// or https://".to_string())
        }
    }
}

/// Statistics utilities
pub struct StatsUtils;

impl StatsUtils {
    /// Calculate average
    pub fn average(values: &[f64]) -> f64 {
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        }
    }
    
    /// Calculate median
    pub fn median(values: &mut [f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = values.len();
        
        if len % 2 == 0 {
            (values[len / 2 - 1] + values[len / 2]) / 2.0
        } else {
            values[len / 2]
        }
    }
    
    /// Calculate percentile
    pub fn percentile(values: &mut [f64], p: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let index = (p / 100.0 * (values.len() - 1) as f64).round() as usize;
        values[index.min(values.len() - 1)]
    }
    
    /// Calculate standard deviation
    pub fn std_deviation(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        
        let mean = Self::average(values);
        let variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (values.len() - 1) as f64;
        
        variance.sqrt()
    }
}

/// Terminal utilities
pub struct TerminalUtils;

impl TerminalUtils {
    /// Get terminal width
    pub fn get_terminal_width() -> usize {
        crossterm::terminal::size()
            .map(|(width, _)| width as usize)
            .unwrap_or(80)
    }
    
    /// Create separator line
    pub fn create_separator(ch: char, width: Option<usize>) -> String {
        let width = width.unwrap_or_else(Self::get_terminal_width);
        ch.to_string().repeat(width)
    }
    
    /// Center text in terminal
    pub fn center_text(text: &str, width: Option<usize>) -> String {
        let width = width.unwrap_or_else(Self::get_terminal_width);
        let text_len = text.len();
        
        if text_len >= width {
            text.to_string()
        } else {
            let padding = (width - text_len) / 2;
            format!("{}{}", " ".repeat(padding), text)
        }
    }
    
    /// Create bordered text
    pub fn create_border(text: &str, border_char: char) -> String {
        let width = text.len() + 4; // 2 spaces + 2 border chars
        let top_bottom = border_char.to_string().repeat(width);
        
        format!(
            "{}\n{} {} {}\n{}",
            top_bottom,
            border_char,
            text,
            border_char,
            top_bottom
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_count_meaningful_lines() {
        let content = r#"
// This is a comment
export function test() {
    console.log("hello");
    
    // Another comment
    return true;
}
        "#;
        
        let meaningful = FileUtils::count_meaningful_lines(content);
        let total = FileUtils::count_total_lines(content);
        
        assert!(meaningful < total);
        assert!(meaningful > 0);
    }
    
    #[test]
    fn test_is_js_ts_file() {
        assert!(FileUtils::is_js_ts_file(&PathBuf::from("test.ts")));
        assert!(FileUtils::is_js_ts_file(&PathBuf::from("test.tsx")));
        assert!(FileUtils::is_js_ts_file(&PathBuf::from("test.js")));
        assert!(FileUtils::is_js_ts_file(&PathBuf::from("test.jsx")));
        assert!(!FileUtils::is_js_ts_file(&PathBuf::from("test.py")));
    }
    
    #[test]
    fn test_format_file_size() {
        assert_eq!(FileUtils::format_file_size(512), "512 B");
        assert_eq!(FileUtils::format_file_size(1024), "1.0 KB");
        assert_eq!(FileUtils::format_file_size(1048576), "1.0 MB");
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(FormatUtils::format_duration(500), "500ms");
        assert_eq!(FormatUtils::format_duration(1500), "1.5s");
        assert_eq!(FormatUtils::format_duration(65000), "1m 5s");
    }
    
    #[test]
    fn test_format_number() {
        assert_eq!(FormatUtils::format_number(1000), "1,000");
        assert_eq!(FormatUtils::format_number(1234567), "1,234,567");
        assert_eq!(FormatUtils::format_number(42), "42");
    }
    
    #[test]
    fn test_validate_threshold() {
        assert!(ValidationUtils::validate_threshold(100).is_ok());
        assert!(ValidationUtils::validate_threshold(0).is_err());
        assert!(ValidationUtils::validate_threshold(20000).is_err());
    }
    
    #[test]
    fn test_stats_utils() {
        let mut values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        assert_eq!(StatsUtils::average(&values), 3.0);
        assert_eq!(StatsUtils::median(&mut values), 3.0);
        assert!(StatsUtils::std_deviation(&values) > 0.0);
    }
}