use colored::*;
use std::collections::{HashMap, HashSet};

use super::types::{ImportsReport, ImportsSummary, UnusedImport, BrokenImport, BrokenImportType};

pub fn calculate_savings(unused_imports: &[UnusedImport]) -> String {
    let total_lines = unused_imports.len();
    if total_lines == 0 {
        "0 lines".to_string()
    } else {
        format!("~{} lines of code", total_lines)
    }
}

pub fn print_report(report: &ImportsReport, quiet: bool) {
    if !quiet {
        println!();
        println!("{}", "📊 Imports Analysis Report".bold().blue());
        println!("{}", "==========================".blue());
        println!();
    }
    
    let has_issues = !report.unused_imports.is_empty() || !report.broken_imports.is_empty();
    
    if !has_issues {
        println!("{}", "✅ No import issues found! Your imports are clean.".green());
        return;
    }
    
    // Group unused imports by file
    let mut unused_by_file: HashMap<String, Vec<&UnusedImport>> = HashMap::new();
    for import in &report.unused_imports {
        unused_by_file.entry(import.file.clone()).or_default().push(import);
    }
    
    // Group broken imports by file
    let mut broken_by_file: HashMap<String, Vec<&BrokenImport>> = HashMap::new();
    for import in &report.broken_imports {
        broken_by_file.entry(import.file.clone()).or_default().push(import);
    }
    
    // Get all unique files
    let mut all_files: HashSet<String> = HashSet::new();
    all_files.extend(unused_by_file.keys().cloned());
    all_files.extend(broken_by_file.keys().cloned());
    
    // Print issues by file
    for file in all_files {
        println!("{}", file.cyan().bold());
        
        // Print unused imports for this file
        if let Some(unused_imports) = unused_by_file.get(&file) {
            for import in unused_imports {
                println!("  Line {}: {}", import.line.to_string().yellow(), import.import_statement.dimmed());
                println!("    {} Unused: {}", "🚫".red(), import.unused_items.join(", ").red());
                println!();
            }
        }
        
        // Print broken imports for this file
        if let Some(broken_imports) = broken_by_file.get(&file) {
            for import in broken_imports {
                println!("  Line {}: {}", import.line.to_string().yellow(), import.import_statement.dimmed());
                let error_msg = match import.error_type {
                    BrokenImportType::FileNotFound => format!("File not found: {}", import.import_path),
                    BrokenImportType::ModuleNotInstalled => format!("Module not installed: {}", import.import_path),
                    BrokenImportType::InvalidPath => format!("Invalid path: {}", import.import_path),
                };
                println!("    {} {}", "💥".red(), error_msg.red());
                if let Some(ref suggestion) = import.suggestion {
                    println!("    {} {}", "💡".yellow(), suggestion.green());
                }
                println!();
            }
        }
    }
    
    // Print summary
    print_summary(&report.summary);
}

fn print_summary(summary: &ImportsSummary) {
    println!("{}", "📈 SUMMARY".bold().white());
    println!("{}", "─────────".white());
    println!("  Files scanned: {}", summary.files_scanned);
    println!("  Total imports: {}", summary.total_imports);
    println!("  {} {}", "Unused imports:".red(), summary.unused_imports.to_string().red());
    println!("  {} {}", "Broken imports:".red(), summary.broken_imports.to_string().red());
    println!("  Potential savings: {}", summary.potential_savings.green());
    
    println!();
    
    if summary.unused_imports > 0 {
        println!("{}", "💡 TIP: Remove unused imports to reduce bundle size and improve build performance".dimmed());
        println!("{}", "🔧 Consider using an IDE extension or linter to automatically remove unused imports".dimmed());
    }
    
    if summary.broken_imports > 0 {
        println!("{}", "🔧 Fix broken imports to resolve compilation errors".yellow());
        println!("{}", "💡 Check if files were moved/renamed, or if packages need to be installed".dimmed());
    }
}