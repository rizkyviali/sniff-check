use anyhow::Result;
use colored::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvReport {
    pub env_files: Vec<EnvFileInfo>,
    pub variables: Vec<EnvVariable>,
    pub summary: EnvSummary,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvFileInfo {
    pub path: String,
    pub exists: bool,
    pub variables_count: usize,
    pub issues: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvVariable {
    pub name: String,
    pub status: VarStatus,
    pub source: Option<String>,
    pub issue_type: Option<IssueType>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VarStatus {
    Present,
    Missing,
    Empty,
    Invalid,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IssueType {
    MissingRequired,
    EmptyValue,
    InvalidFormat,
    SensitiveDataExposed,
    DatabaseConnectionInvalid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvSummary {
    pub total_required: usize,
    pub present: usize,
    pub missing: usize,
    pub empty: usize,
    pub invalid: usize,
    pub security_issues: usize,
}

pub async fn run(json: bool, quiet: bool) -> Result<()> {
    if !quiet {
        println!("{}", "🔍 Validating environment variables...".bold().blue());
    }
    
    let report = analyze_environment().await?;
    
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report, quiet);
    }
    
    // Exit with error if critical environment issues found
    if report.summary.missing > 0 || report.summary.security_issues > 0 {
        std::process::exit(1);
    }
    
    Ok(())
}

async fn analyze_environment() -> Result<EnvReport> {
    let current_dir = env::current_dir()?;
    
    // Analyze environment files
    let env_files = analyze_env_files(&current_dir)?;
    
    // Get required variables (from common patterns and package.json)
    let required_vars = get_required_variables(&current_dir)?;
    
    // Check each required variable
    let mut variables = Vec::new();
    let mut present = 0;
    let mut missing = 0;
    let mut empty = 0;
    let mut invalid = 0;
    let mut security_issues = 0;
    
    for var_name in &required_vars {
        let var_info = check_environment_variable(var_name);
        
        match var_info.status {
            VarStatus::Present => present += 1,
            VarStatus::Missing => missing += 1,
            VarStatus::Empty => empty += 1,
            VarStatus::Invalid => invalid += 1,
        }
        
        if matches!(var_info.issue_type, Some(IssueType::SensitiveDataExposed)) {
            security_issues += 1;
        }
        
        variables.push(var_info);
    }
    
    let recommendations = generate_env_recommendations(&variables, &env_files);
    
    Ok(EnvReport {
        env_files,
        variables,
        summary: EnvSummary {
            total_required: required_vars.len(),
            present,
            missing,
            empty,
            invalid,
            security_issues,
        },
        recommendations,
    })
}

fn analyze_env_files(dir: &Path) -> Result<Vec<EnvFileInfo>> {
    let env_file_names = vec![
        ".env",
        ".env.local",
        ".env.development",
        ".env.production", 
        ".env.staging",
        ".env.test",
    ];
    
    let mut env_files = Vec::new();
    
    for file_name in env_file_names {
        let file_path = dir.join(file_name);
        let exists = file_path.exists();
        
        let (variables_count, issues) = if exists {
            analyze_env_file(&file_path)?
        } else {
            (0, Vec::new())
        };
        
        env_files.push(EnvFileInfo {
            path: file_name.to_string(),
            exists,
            variables_count,
            issues,
        });
    }
    
    Ok(env_files)
}

fn analyze_env_file(file_path: &Path) -> Result<(usize, Vec<String>)> {
    let content = fs::read_to_string(file_path)?;
    let mut variables_count = 0;
    let mut issues = Vec::new();
    
    let var_regex = Regex::new(r"^([A-Z_][A-Z0-9_]*)=(.*)$")?;
    let sensitive_patterns = get_sensitive_patterns();
    
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        if let Some(captures) = var_regex.captures(line) {
            variables_count += 1;
            let var_name = captures.get(1).unwrap().as_str();
            let value = captures.get(2).unwrap().as_str();
            
            // Check for sensitive data patterns
            for pattern in &sensitive_patterns {
                if pattern.is_match(value) {
                    issues.push(format!(
                        "Line {}: Potential sensitive data in {} (consider using environment-specific files)",
                        line_num + 1, var_name
                    ));
                }
            }
            
            // Check for empty values
            if value.is_empty() {
                issues.push(format!("Line {}: Empty value for {}", line_num + 1, var_name));
            }
            
            // Check for unquoted values with spaces
            if value.contains(' ') && !value.starts_with('"') && !value.starts_with('\'') {
                issues.push(format!(
                    "Line {}: Value for {} contains spaces but is not quoted", 
                    line_num + 1, var_name
                ));
            }
        } else if !line.starts_with('#') {
            issues.push(format!("Line {}: Invalid format - should be KEY=value", line_num + 1));
        }
    }
    
    Ok((variables_count, issues))
}

fn get_sensitive_patterns() -> Vec<Regex> {
    vec![
        Regex::new(r"sk_live_").unwrap(),     // Stripe live keys
        Regex::new(r"pk_live_").unwrap(),     // Stripe live public keys  
        Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(), // AWS Access Keys
        Regex::new(r"[0-9a-f]{40}").unwrap(), // GitHub tokens
        Regex::new(r"xoxb-[0-9]{11}-[0-9]{11}-[0-9a-zA-Z]{24}").unwrap(), // Slack tokens
    ]
}

fn get_required_variables(dir: &Path) -> Result<HashSet<String>> {
    let mut required_vars = HashSet::new();
    
    // Common Next.js/TypeScript environment variables
    let common_vars = vec![
        "NODE_ENV",
        "NEXT_PUBLIC_APP_URL",
        "DATABASE_URL",
        "NEXTAUTH_URL",
        "NEXTAUTH_SECRET",
    ];
    
    for var in common_vars {
        required_vars.insert(var.to_string());
    }
    
    // Check package.json for additional hints
    let package_json_path = dir.join("package.json");
    if package_json_path.exists() {
        if let Ok(content) = fs::read_to_string(&package_json_path) {
            // Look for common patterns that suggest required env vars
            if content.contains("prisma") {
                required_vars.insert("DATABASE_URL".to_string());
            }
            if content.contains("stripe") {
                required_vars.insert("STRIPE_SECRET_KEY".to_string());
                required_vars.insert("NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY".to_string());
            }
            if content.contains("supabase") {
                required_vars.insert("NEXT_PUBLIC_SUPABASE_URL".to_string());
                required_vars.insert("NEXT_PUBLIC_SUPABASE_ANON_KEY".to_string());
            }
            if content.contains("vercel") || content.contains("next") {
                required_vars.insert("VERCEL_URL".to_string());
            }
        }
    }
    
    // Check for TypeScript config files that might reference env vars
    let ts_config_path = dir.join("tsconfig.json");
    if ts_config_path.exists() {
        // Additional TypeScript-specific variables
        required_vars.insert("NODE_ENV".to_string());
    }
    
    Ok(required_vars)
}

fn check_environment_variable(var_name: &str) -> EnvVariable {
    match env::var(var_name) {
        Ok(value) => {
            if value.is_empty() {
                EnvVariable {
                    name: var_name.to_string(),
                    status: VarStatus::Empty,
                    source: Some("environment".to_string()),
                    issue_type: Some(IssueType::EmptyValue),
                    suggestion: Some("Set a non-empty value for this variable".to_string()),
                }
            } else if is_invalid_format(var_name, &value) {
                EnvVariable {
                    name: var_name.to_string(),
                    status: VarStatus::Invalid,
                    source: Some("environment".to_string()),
                    issue_type: Some(IssueType::InvalidFormat),
                    suggestion: Some(get_format_suggestion(var_name)),
                }
            } else if is_sensitive_exposed(var_name, &value) {
                EnvVariable {
                    name: var_name.to_string(),
                    status: VarStatus::Present,
                    source: Some("environment".to_string()),
                    issue_type: Some(IssueType::SensitiveDataExposed),
                    suggestion: Some("Move sensitive data to environment-specific files".to_string()),
                }
            } else {
                EnvVariable {
                    name: var_name.to_string(),
                    status: VarStatus::Present,
                    source: Some("environment".to_string()),
                    issue_type: None,
                    suggestion: None,
                }
            }
        }
        Err(_) => EnvVariable {
            name: var_name.to_string(),
            status: VarStatus::Missing,
            source: None,
            issue_type: Some(IssueType::MissingRequired),
            suggestion: Some(format!("Add {} to your .env file", var_name)),
        },
    }
}

fn is_invalid_format(var_name: &str, value: &str) -> bool {
    match var_name {
        "DATABASE_URL" => !is_valid_database_url(value),
        "NEXTAUTH_URL" | "NEXT_PUBLIC_APP_URL" | "VERCEL_URL" => !is_valid_url(value),
        "NODE_ENV" => !matches!(value, "development" | "production" | "test"),
        _ => false,
    }
}

fn is_valid_database_url(url: &str) -> bool {
    url.starts_with("postgresql://") 
        || url.starts_with("mysql://")
        || url.starts_with("sqlite:")
        || url.starts_with("mongodb://")
}

fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

fn is_sensitive_exposed(_var_name: &str, value: &str) -> bool {
    // Check if sensitive data is in a potentially exposed location
    let sensitive_patterns = get_sensitive_patterns();
    sensitive_patterns.iter().any(|pattern| pattern.is_match(value))
}

fn get_format_suggestion(var_name: &str) -> String {
    match var_name {
        "DATABASE_URL" => "Should be a valid database connection string (e.g., postgresql://user:pass@host:port/db)".to_string(),
        "NEXTAUTH_URL" | "NEXT_PUBLIC_APP_URL" => "Should be a valid HTTP/HTTPS URL".to_string(),
        "NODE_ENV" => "Should be 'development', 'production', or 'test'".to_string(),
        _ => "Check the expected format for this variable".to_string(),
    }
}

fn generate_env_recommendations(variables: &[EnvVariable], env_files: &[EnvFileInfo]) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    // Check if .env.local exists for local development
    let has_env_local = env_files.iter().any(|f| f.path == ".env.local" && f.exists);
    if !has_env_local {
        recommendations.push("Create .env.local for local development variables".to_string());
    }
    
    // Check if .env.example exists
    let current_dir = env::current_dir().unwrap_or_default();
    if !current_dir.join(".env.example").exists() {
        recommendations.push("Create .env.example with required variables (without values)".to_string());
    }
    
    // Count missing variables
    let missing_count = variables.iter().filter(|v| matches!(v.status, VarStatus::Missing)).count();
    if missing_count > 0 {
        recommendations.push(format!("Set {} missing environment variables", missing_count));
    }
    
    // Security recommendations
    let security_issues = variables.iter().filter(|v| matches!(v.issue_type, Some(IssueType::SensitiveDataExposed))).count();
    if security_issues > 0 {
        recommendations.push("Review sensitive data exposure in environment files".to_string());
        recommendations.push("Consider using secret management services for production".to_string());
    }
    
    // General recommendations
    recommendations.push("Add .env* files to .gitignore to prevent committing secrets".to_string());
    recommendations.push("Use different .env files for different environments".to_string());
    
    recommendations
}

fn print_report(report: &EnvReport, quiet: bool) {
    if !quiet {
        println!();
        println!("{}", "📊 Environment Variables Report".bold().blue());
        println!("{}", "=============================".blue());
        println!();
    }
    
    // Print environment files status
    println!("{}", "📁 ENVIRONMENT FILES".bold().white());
    println!("{}", "───────────────────".white());
    
    for file in &report.env_files {
        let status = if file.exists {
            format!("✅ {} ({} variables)", file.path, file.variables_count).green()
        } else {
            format!("❌ {} (not found)", file.path).red()
        };
        
        println!("  {}", status);
        
        for issue in &file.issues {
            println!("    ⚠️  {}", issue.yellow());
        }
    }
    
    println!();
    
    // Print variable status
    let missing_vars: Vec<_> = report.variables.iter().filter(|v| matches!(v.status, VarStatus::Missing)).collect();
    let empty_vars: Vec<_> = report.variables.iter().filter(|v| matches!(v.status, VarStatus::Empty)).collect();
    let invalid_vars: Vec<_> = report.variables.iter().filter(|v| matches!(v.status, VarStatus::Invalid)).collect();
    let security_vars: Vec<_> = report.variables.iter().filter(|v| matches!(v.issue_type, Some(IssueType::SensitiveDataExposed))).collect();
    
    if !missing_vars.is_empty() {
        println!("{}", "🚫 MISSING VARIABLES".bold().red());
        println!("{}", "───────────────────".red());
        for var in missing_vars {
            println!("  {} {}", "❌".red(), var.name.red());
            if let Some(suggestion) = &var.suggestion {
                println!("     💡 {}", suggestion.dimmed());
            }
        }
        println!();
    }
    
    if !empty_vars.is_empty() {
        println!("{}", "⚠️  EMPTY VARIABLES".bold().yellow());
        println!("{}", "──────────────────".yellow());
        for var in empty_vars {
            println!("  {} {}", "⚠️".yellow(), var.name.yellow());
            if let Some(suggestion) = &var.suggestion {
                println!("     💡 {}", suggestion.dimmed());
            }
        }
        println!();
    }
    
    if !invalid_vars.is_empty() {
        println!("{}", "❌ INVALID FORMAT".bold().red());
        println!("{}", "─────────────────".red());
        for var in invalid_vars {
            println!("  {} {}", "❌".red(), var.name.red());
            if let Some(suggestion) = &var.suggestion {
                println!("     💡 {}", suggestion.dimmed());
            }
        }
        println!();
    }
    
    if !security_vars.is_empty() {
        println!("{}", "🔒 SECURITY ISSUES".bold().red());
        println!("{}", "─────────────────".red());
        for var in security_vars {
            println!("  {} {} - Sensitive data detected", "🔒".red(), var.name.red());
            if let Some(suggestion) = &var.suggestion {
                println!("     💡 {}", suggestion.dimmed());
            }
        }
        println!();
    }
    
    // Print recommendations
    if !report.recommendations.is_empty() {
        println!("{}", "💡 RECOMMENDATIONS".bold().green());
        println!("{}", "──────────────────".green());
        for rec in &report.recommendations {
            println!("  • {}", rec.green());
        }
        println!();
    }
    
    // Print summary
    print_summary(&report.summary);
}

fn print_summary(summary: &EnvSummary) {
    println!("{}", "📈 SUMMARY".bold().white());
    println!("{}", "─────────".white());
    
    println!("  Total required: {}", summary.total_required);
    println!("  {} {}", "Present:".green(), summary.present.to_string().green());
    
    if summary.missing > 0 {
        println!("  {} {}", "Missing:".red(), summary.missing.to_string().red());
    }
    if summary.empty > 0 {
        println!("  {} {}", "Empty:".yellow(), summary.empty.to_string().yellow());
    }
    if summary.invalid > 0 {
        println!("  {} {}", "Invalid:".red(), summary.invalid.to_string().red());
    }
    if summary.security_issues > 0 {
        println!("  {} {}", "Security issues:".red(), summary.security_issues.to_string().red());
    }
    
    println!();
    
    let health_score = if summary.total_required > 0 {
        (summary.present as f64 / summary.total_required as f64) * 100.0
    } else {
        100.0
    };
    
    let health_color = if health_score >= 90.0 {
        format!("{:.1}%", health_score).green()
    } else if health_score >= 70.0 {
        format!("{:.1}%", health_score).yellow()
    } else {
        format!("{:.1}%", health_score).red()
    };
    
    println!("  Environment Health: {}", health_color);
    
    if summary.missing > 0 || summary.security_issues > 0 {
        println!();
        println!("{}", "🚨 CRITICAL: Fix missing variables and security issues before deployment".red().bold());
    }
    
    println!();
    println!("{}", "💡 TIP: Use .env.example to document required variables for your team".dimmed());
}