use anyhow::Result;
use colored::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct LargeFileReport {
    pub files: Vec<LargeFile>,
    pub summary: Summary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LargeFile {
    pub path: String,
    pub lines: usize,
    pub file_type: FileType,
    pub severity: Severity,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileType {
    Component,
    Service,
    Api,
    Page,
    Util,
    Config,
    Test,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Severity {
    Warning,  // 100-200 lines
    Error,    // 200-400 lines
    Critical, // 400+ lines
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub total_files_scanned: usize,
    pub large_files_found: usize,
    pub warnings: usize,
    pub errors: usize,
    pub critical: usize,
}

pub async fn run(threshold: usize, json: bool, quiet: bool) -> Result<()> {
    if !quiet {
        println!("{}", "🔍 Scanning for large files...".bold().blue());
    }
    
    let report = scan_large_files(threshold)?;
    
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report, quiet);
    }
    
    if report.summary.large_files_found > 0 {
        std::process::exit(1);
    }
    
    Ok(())
}

fn scan_large_files(threshold: usize) -> Result<LargeFileReport> {
    let current_dir = std::env::current_dir()?;
    let extensions = vec!["ts", "tsx", "js", "jsx"];
    
    let files: Vec<PathBuf> = WalkDir::new(&current_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            if let Some(ext) = e.path().extension() {
                extensions.contains(&ext.to_string_lossy().as_ref())
            } else {
                false
            }
        })
        .filter(|e| !is_excluded_path(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect();
    
    let total_files = files.len();
    
    let large_files: Vec<LargeFile> = files
        .par_iter()
        .filter_map(|path| {
            let line_count = count_lines(path).ok()?;
            if line_count >= threshold {
                Some(create_large_file_info(path, line_count))
            } else {
                None
            }
        })
        .collect();
    
    let summary = create_summary(total_files, &large_files);
    
    Ok(LargeFileReport {
        files: large_files,
        summary,
    })
}

fn is_excluded_path(path: &Path) -> bool {
    let excluded_dirs = vec![
        "node_modules", ".next", "dist", "build", ".git", 
        "coverage", "target", ".vscode", ".idea"
    ];
    
    path.ancestors().any(|ancestor| {
        if let Some(name) = ancestor.file_name() {
            excluded_dirs.contains(&name.to_string_lossy().as_ref())
        } else {
            false
        }
    })
}

fn count_lines(path: &Path) -> Result<usize> {
    let content = fs::read_to_string(path)?;
    Ok(content.lines().count())
}

fn create_large_file_info(path: &Path, lines: usize) -> LargeFile {
    let file_type = determine_file_type(path);
    let severity = determine_severity(lines);
    let suggestions = generate_suggestions(&file_type, lines);
    
    LargeFile {
        path: path.to_string_lossy().to_string(),
        lines,
        file_type,
        severity,
        suggestions,
    }
}

fn determine_file_type(path: &Path) -> FileType {
    let path_str = path.to_string_lossy().to_lowercase();
    
    if path_str.contains("/pages/") || path_str.contains("/app/") {
        FileType::Page
    } else if path_str.contains("/api/") {
        FileType::Api
    } else if path_str.contains("/components/") {
        FileType::Component
    } else if path_str.contains("/services/") || path_str.contains("/lib/") {
        FileType::Service
    } else if path_str.contains("/utils/") || path_str.contains("/helpers/") {
        FileType::Util
    } else if path_str.contains("config") {
        FileType::Config
    } else if path_str.contains("test") || path_str.contains("spec") {
        FileType::Test
    } else {
        FileType::Other
    }
}

fn determine_severity(lines: usize) -> Severity {
    match lines {
        100..=199 => Severity::Warning,
        200..=399 => Severity::Error,
        _ => Severity::Critical,
    }
}

fn generate_suggestions(file_type: &FileType, lines: usize) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    match file_type {
        FileType::Component => {
            suggestions.push("Break into smaller sub-components".to_string());
            suggestions.push("Extract custom hooks for logic".to_string());
            suggestions.push("Move utility functions to separate files".to_string());
        },
        FileType::Service => {
            suggestions.push("Split into multiple service classes".to_string());
            suggestions.push("Extract interfaces and types".to_string());
            suggestions.push("Use dependency injection".to_string());
        },
        FileType::Api => {
            suggestions.push("Split into multiple route handlers".to_string());
            suggestions.push("Extract validation logic".to_string());
            suggestions.push("Move business logic to services".to_string());
        },
        FileType::Page => {
            suggestions.push("Extract page components".to_string());
            suggestions.push("Move data fetching to separate hooks".to_string());
            suggestions.push("Split into layout and content components".to_string());
        },
        FileType::Util => {
            suggestions.push("Split utility functions by domain".to_string());
            suggestions.push("Create separate files for each utility group".to_string());
        },
        _ => {
            suggestions.push("Consider breaking into smaller modules".to_string());
            suggestions.push("Extract reusable logic".to_string());
        }
    }
    
    if lines > 300 {
        suggestions.push("⚠️ CRITICAL: This file is extremely large and should be refactored immediately".to_string());
    }
    
    suggestions
}

fn create_summary(total_files: usize, large_files: &[LargeFile]) -> Summary {
    let mut warnings = 0;
    let mut errors = 0;
    let mut critical = 0;
    
    for file in large_files {
        match file.severity {
            Severity::Warning => warnings += 1,
            Severity::Error => errors += 1,
            Severity::Critical => critical += 1,
        }
    }
    
    Summary {
        total_files_scanned: total_files,
        large_files_found: large_files.len(),
        warnings,
        errors,
        critical,
    }
}

fn print_report(report: &LargeFileReport, quiet: bool) {
    if !quiet {
        println!();
        println!("{}", "📊 Large Files Report".bold().blue());
        println!("{}", "====================".blue());
        println!();
    }
    
    if report.summary.large_files_found == 0 {
        println!("{}", "✅ No large files found! Your code is clean.".green());
        return;
    }
    
    // Group files by severity
    let mut files_by_severity: HashMap<String, Vec<&LargeFile>> = HashMap::new();
    
    for file in &report.files {
        let severity_key = match file.severity {
            Severity::Critical => "Critical (400+ lines)",
            Severity::Error => "Error (200-399 lines)",
            Severity::Warning => "Warning (100-199 lines)",
        };
        files_by_severity.entry(severity_key.to_string()).or_default().push(file);
    }
    
    // Print critical files first
    if let Some(critical_files) = files_by_severity.get("Critical (400+ lines)") {
        println!("{}", "🚨 CRITICAL FILES".bold().red());
        for file in critical_files {
            print_file_info(file, "red");
        }
        println!();
    }
    
    // Print error files
    if let Some(error_files) = files_by_severity.get("Error (200-399 lines)") {
        println!("{}", "⚠️  ERROR FILES".bold().yellow());
        for file in error_files {
            print_file_info(file, "yellow");
        }
        println!();
    }
    
    // Print warning files
    if let Some(warning_files) = files_by_severity.get("Warning (100-199 lines)") {
        println!("{}", "⚡ WARNING FILES".bold().cyan());
        for file in warning_files {
            print_file_info(file, "cyan");
        }
        println!();
    }
    
    // Print summary
    print_summary(&report.summary);
}

fn print_file_info(file: &LargeFile, color: &str) {
    let path_colored = match color {
        "red" => file.path.red(),
        "yellow" => file.path.yellow(),
        "cyan" => file.path.cyan(),
        _ => file.path.normal(),
    };
    
    println!("  {} ({} lines) - {:?}", path_colored, file.lines, file.file_type);
    
    for suggestion in &file.suggestions {
        println!("    • {}", suggestion.dimmed());
    }
    println!();
}

fn print_summary(summary: &Summary) {
    println!("{}", "📈 SUMMARY".bold().white());
    println!("{}", "─────────".white());
    println!("  Files scanned: {}", summary.total_files_scanned);
    println!("  Large files found: {}", summary.large_files_found);
    
    if summary.critical > 0 {
        println!("  {} {}", "Critical:".red(), summary.critical.to_string().red());
    }
    if summary.errors > 0 {
        println!("  {} {}", "Errors:".yellow(), summary.errors.to_string().yellow());
    }
    if summary.warnings > 0 {
        println!("  {} {}", "Warnings:".cyan(), summary.warnings.to_string().cyan());
    }
    
    println!();
    println!("{}", "💡 TIP: Files over 100 lines are considered 'smelly code' and should be refactored".dimmed());
}