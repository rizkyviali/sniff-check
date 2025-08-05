use anyhow::Result;
use colored::*;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use crate::utils::FileUtils;
use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeScriptReport {
    pub issues: Vec<TypeIssue>,
    pub summary: TypeSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeIssue {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub issue_type: IssueType,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IssueType {
    AnyUsage,
    MissingReturnType,
    UntypedParameter,
    TSIgnore,
    TSExpectError,
    ImplicitAny,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeSummary {
    pub files_scanned: usize,
    pub total_issues: usize,
    pub any_usage_count: usize,
    pub missing_return_types: usize,
    pub untyped_parameters: usize,
    pub ts_ignore_count: usize,
    pub type_coverage_score: f64,
}

pub async fn run(json: bool, quiet: bool) -> Result<()> {
    if !quiet {
        println!("{}", "🔍 Checking TypeScript type coverage...".bold().blue());
    }
    
    let report = analyze_typescript_files()?;
    
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report, quiet);
    }
    
    // Exit with error code if there are critical issues
    if report.summary.any_usage_count > 0 || report.summary.ts_ignore_count > 5 {
        std::process::exit(1);
    }
    
    Ok(())
}

// Global regex patterns compiled once
static REGEX_PATTERNS: OnceLock<RegexPatterns> = OnceLock::new();

struct RegexPatterns {
    any_regex: Regex,
    function_regex: Regex,
    ts_ignore_regex: Regex,
    ts_expect_error_regex: Regex,
}

impl RegexPatterns {
    fn new() -> Result<Self> {
        Ok(Self {
            any_regex: Regex::new(r"\b:\s*any\b")?,
            function_regex: Regex::new(r"(?:function\s+\w+|const\s+\w+\s*=\s*(?:async\s+)?\([^)]*\)\s*=>|(?:async\s+)?function\s*\([^)]*\))\s*\{")?,
            ts_ignore_regex: Regex::new(r"@ts-ignore")?,
            ts_expect_error_regex: Regex::new(r"@ts-expect-error")?,
        })
    }
}

fn get_regex_patterns() -> &'static RegexPatterns {
    REGEX_PATTERNS.get_or_init(|| RegexPatterns::new().expect("Failed to compile regex patterns"))
}

fn analyze_typescript_files() -> Result<TypeScriptReport> {
    let current_dir = std::env::current_dir()?;
    let extensions = vec!["ts", "tsx"];
    
    let files = FileUtils::find_files_with_extensions(&current_dir, &extensions);
    let files_count = files.len();
    
    let all_issues: Vec<Vec<TypeIssue>> = FileUtils::process_files_parallel(
        &files,
        |path| analyze_file_optimized(path),
        "Analyzing TypeScript files",
        false
    )?;
    
    let issues: Vec<TypeIssue> = all_issues.into_iter().flatten().collect();
    let summary = create_summary(files_count, &issues);
    
    Ok(TypeScriptReport { issues, summary })
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

fn analyze_file_optimized(path: &Path) -> Result<Vec<TypeIssue>> {
    let content = fs::read_to_string(path)?;
    let mut issues = Vec::new();
    let patterns = get_regex_patterns();
    let file_path = FileUtils::get_relative_path(path);
    
    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;
        
        // Check for 'any' usage
        for mat in patterns.any_regex.find_iter(line) {
            issues.push(TypeIssue {
                file: file_path.clone(),
                line: line_num,
                column: mat.start(),
                issue_type: IssueType::AnyUsage,
                message: "Usage of 'any' type detected".to_string(),
                suggestion: Some("Consider using a more specific type".to_string()),
            });
        }
        
        // Check for @ts-ignore
        if patterns.ts_ignore_regex.is_match(line) {
            issues.push(TypeIssue {
                file: file_path.clone(),
                line: line_num,
                column: 0,
                issue_type: IssueType::TSIgnore,
                message: "@ts-ignore comment found".to_string(),
                suggestion: Some("Consider fixing the underlying type issue instead".to_string()),
            });
        }
        
        // Check for @ts-expect-error
        if patterns.ts_expect_error_regex.is_match(line) {
            issues.push(TypeIssue {
                file: file_path.clone(),
                line: line_num,
                column: 0,
                issue_type: IssueType::TSExpectError,
                message: "@ts-expect-error comment found".to_string(),
                suggestion: Some("Verify if this error suppression is still needed".to_string()),
            });
        }
        
        // Check for functions without return types (simplified check)
        if patterns.function_regex.is_match(line) && !line.contains("):") && !line.trim_end().ends_with("=> {") {
            if !line.contains("constructor") && !line.contains("() {") {
                issues.push(TypeIssue {
                    file: file_path.clone(),
                    line: line_num,
                    column: 0,
                    issue_type: IssueType::MissingReturnType,
                    message: "Function missing explicit return type".to_string(),
                    suggestion: Some("Add explicit return type annotation".to_string()),
                });
            }
        }
    }
    
    Ok(issues)
}

fn create_summary(files_scanned: usize, issues: &[TypeIssue]) -> TypeSummary {
    let mut any_usage_count = 0;
    let mut missing_return_types = 0;
    let mut untyped_parameters = 0;
    let mut ts_ignore_count = 0;
    
    for issue in issues {
        match issue.issue_type {
            IssueType::AnyUsage => any_usage_count += 1,
            IssueType::MissingReturnType => missing_return_types += 1,
            IssueType::UntypedParameter => untyped_parameters += 1,
            IssueType::TSIgnore | IssueType::TSExpectError => ts_ignore_count += 1,
            _ => {}
        }
    }
    
    // Calculate type coverage score (simplified)
    let total_potential_issues = files_scanned * 10; // Rough estimate
    let type_coverage_score = if total_potential_issues > 0 {
        ((total_potential_issues - issues.len()) as f64 / total_potential_issues as f64) * 100.0
    } else {
        100.0
    };
    
    TypeSummary {
        files_scanned,
        total_issues: issues.len(),
        any_usage_count,
        missing_return_types,
        untyped_parameters,
        ts_ignore_count,
        type_coverage_score: type_coverage_score.max(0.0).min(100.0),
    }
}

fn print_report(report: &TypeScriptReport, quiet: bool) {
    if !quiet {
        println!();
        println!("{}", "📊 TypeScript Quality Report".bold().blue());
        println!("{}", "===========================".blue());
        println!();
    }
    
    if report.summary.total_issues == 0 {
        println!("{}", "✅ Excellent TypeScript quality! No issues found.".green());
        return;
    }
    
    // Group issues by type
    let mut issues_by_type: HashMap<String, Vec<&TypeIssue>> = HashMap::new();
    
    for issue in &report.issues {
        let type_key = match issue.issue_type {
            IssueType::AnyUsage => "🚫 'any' Type Usage",
            IssueType::MissingReturnType => "📝 Missing Return Types",
            IssueType::UntypedParameter => "❓ Untyped Parameters",
            IssueType::TSIgnore => "⚠️ @ts-ignore Comments",
            IssueType::TSExpectError => "⚠️ @ts-expect-error Comments",
            IssueType::ImplicitAny => "🔄 Implicit Any",
        };
        
        issues_by_type.entry(type_key.to_string()).or_default().push(issue);
    }
    
    // Print critical issues first (any usage)
    if let Some(any_issues) = issues_by_type.get("🚫 'any' Type Usage") {
        println!("{}", "🚫 'ANY' TYPE USAGE (CRITICAL)".bold().red());
        println!("{}", "─────────────────────────────".red());
        for issue in any_issues.iter().take(10) { // Show first 10
            print_issue(issue, "red");
        }
        if any_issues.len() > 10 {
            println!("  {} {} more 'any' usages...", "...and".dimmed(), (any_issues.len() - 10).to_string().red());
        }
        println!();
    }
    
    // Print other issues
    for (type_name, issues) in &issues_by_type {
        if type_name.contains("'any'") {
            continue; // Already printed
        }
        
        let color = if type_name.contains("@ts-") { "yellow" } else { "cyan" };
        
        println!("{}", type_name.bold());
        println!("{}", "─".repeat(type_name.len()));
        
        for issue in issues.iter().take(5) { // Show first 5 of each type
            print_issue(issue, color);
        }
        
        if issues.len() > 5 {
            println!("  {} {} more issues...", "...and".dimmed(), (issues.len() - 5).to_string());
        }
        println!();
    }
    
    // Print summary
    print_summary(&report.summary);
}

fn print_issue(issue: &TypeIssue, color: &str) {
    let file_colored = match color {
        "red" => issue.file.red(),
        "yellow" => issue.file.yellow(),
        "cyan" => issue.file.cyan(),
        _ => issue.file.normal(),
    };
    
    println!("  {}:{} - {}", file_colored, issue.line, issue.message);
    
    if let Some(suggestion) = &issue.suggestion {
        println!("    💡 {}", suggestion.dimmed());
    }
}

fn print_summary(summary: &TypeSummary) {
    println!("{}", "📈 SUMMARY".bold().white());
    println!("{}", "─────────".white());
    println!("  Files scanned: {}", summary.files_scanned);
    println!("  Total issues: {}", summary.total_issues);
    
    if summary.any_usage_count > 0 {
        println!("  {} {}", "'any' usage:".red(), summary.any_usage_count.to_string().red());
    }
    if summary.missing_return_types > 0 {
        println!("  {} {}", "Missing return types:".yellow(), summary.missing_return_types.to_string().yellow());
    }
    if summary.ts_ignore_count > 0 {
        println!("  {} {}", "TS suppressions:".cyan(), summary.ts_ignore_count.to_string().cyan());
    }
    
    println!();
    
    // Type coverage score with color
    let coverage_str = format!("{:.1}%", summary.type_coverage_score);
    let coverage_colored = if summary.type_coverage_score >= 90.0 {
        coverage_str.green()
    } else if summary.type_coverage_score >= 70.0 {
        coverage_str.yellow()
    } else {
        coverage_str.red()
    };
    
    println!("  Type Coverage Score: {}", coverage_colored);
    println!();
    
    if summary.any_usage_count > 0 {
        println!("{}", "🚫 CRITICAL: Usage of 'any' type is strictly forbidden!".red().bold());
        println!("{}", "   All 'any' types must be replaced with specific types.".dimmed());
    }
    
    println!("{}", "💡 TIP: Enable strict mode in tsconfig.json for better type safety".dimmed());
}