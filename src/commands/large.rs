use anyhow::Result;
use colored::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::utils::FileUtils;
use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct LargeFileReport {
    pub files: Vec<LargeFile>,
    pub summary: Summary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LargeFile {
    pub path: String,
    pub lines: usize,
    pub size_bytes: u64,
    pub size_kb: f64,
    pub file_type: FileType,
    pub severity: Severity,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileType {
    ApiRoute,
    ServerComponent,
    ClientComponent,
    CustomHook,
    TypeDefinition,
    Middleware,
    Layout,
    Page,
    Component,
    Service,
    Util,
    Config,
    Test,
    Other,
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            FileType::ApiRoute => "API Route",
            FileType::ServerComponent => "Server Component",
            FileType::ClientComponent => "Client Component", 
            FileType::CustomHook => "Custom Hook",
            FileType::TypeDefinition => "Type Definition",
            FileType::Middleware => "Middleware",
            FileType::Layout => "Layout",
            FileType::Page => "Page",
            FileType::Component => "Component",
            FileType::Service => "Service",
            FileType::Util => "Utility",
            FileType::Config => "Configuration",
            FileType::Test => "Test",
            FileType::Other => "Other",
        };
        write!(f, "{}", display)
    }
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
    
    let report = scan_large_files(threshold, quiet)?;
    
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

fn scan_large_files(threshold: usize, quiet: bool) -> Result<LargeFileReport> {
    let current_dir = std::env::current_dir()?;
    let config = Config::load().unwrap_or_default();
    let extensions = vec!["ts", "tsx", "js", "jsx"];
    
    let files = FileUtils::find_files_with_progress(&current_dir, &extensions, quiet)?;
    let total_files = files.len();
    
    let large_files: Vec<LargeFile> = FileUtils::process_files_parallel(
        &files,
        |path| {
            let line_count = FileUtils::count_lines_optimized(path)?;
            if line_count >= threshold {
                let size_bytes = fs::metadata(path)?.len();
                Ok(Some(create_large_file_info(path, line_count, size_bytes, &config)))
            } else {
                Ok(None)
            }
        },
        "Analyzing file sizes",
        quiet
    )?
    .into_iter()
    .filter_map(|opt| opt)
    .collect();
    
    let summary = create_summary(total_files, &large_files);
    
    Ok(LargeFileReport {
        files: large_files,
        summary,
    })
}

fn create_large_file_info(path: &Path, lines: usize, size_bytes: u64, config: &Config) -> LargeFile {
    let file_type = determine_file_type(path);
    let severity = determine_severity_with_config(lines, config);
    let suggestions = generate_suggestions(&file_type, lines);
    
    let size_kb = size_bytes as f64 / 1024.0;
    
    LargeFile {
        path: FileUtils::get_relative_path(path),
        lines,
        size_bytes,
        size_kb,
        file_type,
        severity,
        suggestions,
    }
}

fn determine_file_type(path: &Path) -> FileType {
    let path_str = path.to_string_lossy();
    let path_lower = path_str.to_lowercase();
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    
    // Check file content for client directive (simplified check)
    let has_use_client = if let Ok(content) = fs::read_to_string(path) {
        content.lines().take(10).any(|line| line.trim().starts_with("'use client'") || line.trim().starts_with("\"use client\""))
    } else {
        false
    };
    
    // Specific Next.js patterns
    if file_name == "middleware.ts" || file_name == "middleware.js" {
        FileType::Middleware
    } else if file_name == "layout.tsx" || file_name == "layout.js" {
        FileType::Layout
    } else if file_name == "page.tsx" || file_name == "page.js" {
        FileType::Page
    } else if path_lower.contains("/api/") && (path_lower.ends_with("/route.ts") || path_lower.ends_with("/route.js")) {
        FileType::ApiRoute
    } else if path_lower.contains("/api/") {
        FileType::ApiRoute
    } else if path_str.ends_with(".d.ts") || (path_lower.contains("/types/") && (path_lower.ends_with(".ts") || path_lower.ends_with(".tsx"))) {
        FileType::TypeDefinition
    } else if file_name.starts_with("use") && file_name.len() > 3 {
        let char_after_use = file_name.chars().nth(3).unwrap_or(' ');
        if char_after_use.is_uppercase() {
            FileType::CustomHook
        } else {
            FileType::Component
        }
    } else if has_use_client {
        FileType::ClientComponent
    } else if path_lower.contains("/components/") {
        // Check if it's likely a server component (React 18+ pattern)
        if path_lower.contains("/app/") && !has_use_client {
            FileType::ServerComponent
        } else {
            FileType::Component
        }
    } else if path_lower.contains("/pages/") {
        FileType::Page
    } else if path_lower.contains("/services/") || path_lower.contains("/lib/") {
        FileType::Service
    } else if path_lower.contains("/utils/") || path_lower.contains("/helpers/") {
        FileType::Util
    } else if path_lower.contains("config") {
        FileType::Config
    } else if path_lower.contains("test") || path_lower.contains("spec") {
        FileType::Test
    } else {
        FileType::Other
    }
}

fn determine_severity_with_config(lines: usize, config: &Config) -> Severity {
    let levels = &config.large_files.severity_levels;
    
    if lines >= levels.critical {
        Severity::Critical
    } else if lines >= levels.error {
        Severity::Error
    } else if lines >= levels.warning {
        Severity::Warning
    } else {
        Severity::Warning // Fallback, shouldn't happen if threshold is set correctly
    }
}

fn generate_suggestions(file_type: &FileType, lines: usize) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    match file_type {
        FileType::ServerComponent | FileType::ClientComponent | FileType::Component => {
            suggestions.push("🧩 Break into smaller components".to_string());
            suggestions.push("🎣 Extract custom hooks for logic".to_string());
            suggestions.push("📦 Move utility functions to separate files".to_string());
        },
        FileType::Service => {
            suggestions.push("🔧 Split into multiple service classes".to_string());
            suggestions.push("📝 Extract interfaces and types".to_string());
            suggestions.push("💉 Use dependency injection".to_string());
        },
        FileType::ApiRoute => {
            suggestions.push("🛣️ Split into multiple route handlers".to_string());
            suggestions.push("✅ Extract validation logic".to_string());
            suggestions.push("🏢 Move business logic to services".to_string());
        },
        FileType::Page => {
            suggestions.push("🏗️ Extract page components".to_string());
            suggestions.push("🎣 Move data fetching to separate hooks".to_string());
            suggestions.push("📱 Split into layout and content components".to_string());
        },
        FileType::Layout => {
            suggestions.push("🎨 Extract layout components".to_string());
            suggestions.push("🔧 Move layout logic to custom hooks".to_string());
            suggestions.push("📐 Split complex layouts into sections".to_string());
        },
        FileType::CustomHook => {
            suggestions.push("⚡ Split hook into smaller focused hooks".to_string());
            suggestions.push("🔄 Extract shared logic to utilities".to_string());
            suggestions.push("🎯 Consider hook composition patterns".to_string());
        },
        FileType::TypeDefinition => {
            suggestions.push("📋 Split types by domain or feature".to_string());
            suggestions.push("🏗️ Group related interfaces together".to_string());
            suggestions.push("📦 Consider type-only import/export".to_string());
        },
        FileType::Middleware => {
            suggestions.push("🔀 Split middleware by functionality".to_string());
            suggestions.push("🛡️ Extract validation to separate functions".to_string());
            suggestions.push("📊 Move logging logic to utilities".to_string());
        },
        FileType::Util => {
            suggestions.push("🔧 Split utility functions by domain".to_string());
            suggestions.push("📁 Create separate files for each utility group".to_string());
            suggestions.push("🎯 Group related functions together".to_string());
        },
        _ => {
            suggestions.push("📦 Consider breaking into smaller modules".to_string());
            suggestions.push("♻️ Extract reusable logic".to_string());
        }
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
        for file in critical_files {
            print_file_info_compact(file, "critical");
        }
    }
    
    // Print error files
    if let Some(error_files) = files_by_severity.get("Error (200-399 lines)") {
        for file in error_files {
            print_file_info_compact(file, "error");
        }
    }
    
    // Print warning files
    if let Some(warning_files) = files_by_severity.get("Warning (100-199 lines)") {
        for file in warning_files {
            print_file_info_compact(file, "warning");
        }
    }
    
    // Print summary
    print_summary(&report.summary);
}

fn print_file_info_compact(file: &LargeFile, severity: &str) {
    let (emoji, path_color) = match severity {
        "critical" => ("🚨 CRITICAL:", "red"),
        "error" => ("⚠️  ERROR:", "yellow"),
        "warning" => ("⚡ WARNING:", "cyan"),
        _ => ("📄", "white"),
    };
    
    let path_colored = match path_color {
        "red" => file.path.red(),
        "yellow" => file.path.yellow(),
        "cyan" => file.path.cyan(),
        _ => file.path.normal(),
    };
    
    // Format file size
    let size_display = if file.size_kb >= 1024.0 {
        format!("{:.1} MB", file.size_kb / 1024.0)
    } else {
        format!("{:.1} KB", file.size_kb)
    };
    
    println!("{} {}", emoji.bold(), path_colored.bold());
    println!("   📏 {} lines | 💾 {}", file.lines.to_string().bold(), size_display.bold());
    
    for suggestion in &file.suggestions {
        println!("   {}", suggestion);
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