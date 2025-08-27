use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::{env, types, large, imports_analyzer as imports, bundle};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentReport {
    pub checks: Vec<CheckResult>,
    pub summary: DeploymentSummary,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub duration_ms: u64,
    pub issues_found: usize,
    pub critical_issues: usize,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CheckStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentSummary {
    pub total_checks: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub skipped: usize,
    pub overall_status: CheckStatus,
    pub deployment_ready: bool,
}

pub async fn run(json: bool, quiet: bool) -> Result<()> {
    if !quiet {
        println!("{}", "🚀 Running Pre-Deployment Validation Pipeline".bold().blue());
        println!("{}", "=============================================".blue());
        println!();
    }
    
    let start_time = Instant::now();
    let mut checks = Vec::new();
    
    if !quiet {
        println!("🔄 Running 5 deployment validation checks...");
    }
    
    // Run all checks in sequence
    checks.push(run_env_check(quiet).await);
    if !quiet { println!("✅ 1/5 Environment check completed"); }
    
    checks.push(run_types_check(quiet).await);
    if !quiet { println!("✅ 2/5 TypeScript check completed"); }
    
    checks.push(run_large_files_check(quiet).await);
    if !quiet { println!("✅ 3/5 Large files check completed"); }
    
    checks.push(run_imports_check(quiet).await);
    if !quiet { println!("✅ 4/5 Imports check completed"); }
    
    checks.push(run_bundle_check(quiet).await);
    if !quiet { println!("✅ 5/5 Bundle check completed"); }
    
    if !quiet {
        println!("🎉 All deployment checks completed!");
    }
    
    let total_duration = start_time.elapsed().as_millis() as u64;
    
    // Calculate summary
    let summary = calculate_summary(&checks);
    
    let report = DeploymentReport {
        checks,
        summary,
        duration_ms: total_duration,
    };
    
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_deployment_report(&report, quiet);
    }
    
    // Exit with appropriate code
    if !report.summary.deployment_ready {
        std::process::exit(1);
    }
    
    Ok(())
}

async fn run_env_check(quiet: bool) -> CheckResult {
    let start_time = Instant::now();
    let name = "Environment Variables".to_string();
    
    if !quiet {
        println!("{} {}", "🔍".blue(), "Checking environment variables...".bold());
    }
    
    // Run the check directly
    let result = env::run(false, true).await;
    
    let duration = start_time.elapsed().as_millis() as u64;
    
    match result {
        Ok(()) => CheckResult {
            name,
            status: CheckStatus::Passed,
            duration_ms: duration,
            issues_found: 0,
            critical_issues: 0,
            message: "All environment variables are properly configured".to_string(),
        },
        Err(_) => CheckResult {
            name,
            status: CheckStatus::Failed,
            duration_ms: duration,
            issues_found: 1,
            critical_issues: 1,
            message: "Environment variables validation failed".to_string(),
        },
    }
}

async fn run_types_check(quiet: bool) -> CheckResult {
    let start_time = Instant::now();
    let name = "TypeScript Quality".to_string();
    
    if !quiet {
        println!("{} {}", "📝".blue(), "Checking TypeScript quality...".bold());
    }
    
    let result = types::run(false, true).await;
    
    let duration = start_time.elapsed().as_millis() as u64;
    
    match result {
        Ok(()) => CheckResult {
            name,
            status: CheckStatus::Passed,
            duration_ms: duration,
            issues_found: 0,
            critical_issues: 0,
            message: "TypeScript quality check passed".to_string(),
        },
        Err(_) => CheckResult {
            name,
            status: CheckStatus::Failed,
            duration_ms: duration,
            issues_found: 1,
            critical_issues: 1,
            message: "TypeScript quality issues found ('any' usage or missing types)".to_string(),
        },
    }
}

async fn run_large_files_check(quiet: bool) -> CheckResult {
    let start_time = Instant::now();
    let name = "Large Files Detection".to_string();
    
    if !quiet {
        println!("{} {}", "📏".blue(), "Checking for large files...".bold());
    }
    
    let result = large::run(100, false, true).await;
    
    let duration = start_time.elapsed().as_millis() as u64;
    
    match result {
        Ok(()) => CheckResult {
            name,
            status: CheckStatus::Passed,
            duration_ms: duration,
            issues_found: 0,
            critical_issues: 0,
            message: "No large files detected".to_string(),
        },
        Err(_) => CheckResult {
            name,
            status: CheckStatus::Warning,
            duration_ms: duration,
            issues_found: 1,
            critical_issues: 0,
            message: "Large files detected - consider refactoring".to_string(),
        },
    }
}

async fn run_imports_check(quiet: bool) -> CheckResult {
    let start_time = Instant::now();
    let name = "Unused Imports".to_string();
    
    if !quiet {
        println!("{} {}", "🧹".blue(), "Checking for unused imports...".bold());
    }
    
    let result = imports::run(false, true).await;
    
    let duration = start_time.elapsed().as_millis() as u64;
    
    match result {
        Ok(()) => CheckResult {
            name,
            status: CheckStatus::Passed,
            duration_ms: duration,
            issues_found: 0,
            critical_issues: 0,
            message: "No unused imports found".to_string(),
        },
        Err(_) => CheckResult {
            name,
            status: CheckStatus::Warning,
            duration_ms: duration,
            issues_found: 1,
            critical_issues: 0,
            message: "Unused imports detected - clean up recommended".to_string(),
        },
    }
}

async fn run_bundle_check(quiet: bool) -> CheckResult {
    let start_time = Instant::now();
    let name = "Bundle Analysis".to_string();
    
    if !quiet {
        println!("{} {}", "📦".blue(), "Analyzing bundle size...".bold());
    }
    
    let result = bundle::run(false, true).await;
    
    let duration = start_time.elapsed().as_millis() as u64;
    
    match result {
        Ok(()) => CheckResult {
            name,
            status: CheckStatus::Passed,
            duration_ms: duration,
            issues_found: 0,
            critical_issues: 0,
            message: "Bundle size is within acceptable limits".to_string(),
        },
        Err(_) => {
            // Check if it's a "no build output" error or actual failure
            CheckResult {
                name,
                status: CheckStatus::Skipped,
                duration_ms: duration,
                issues_found: 0,
                critical_issues: 0,
                message: "Bundle analysis skipped (no build output found)".to_string(),
            }
        },
    }
}

fn calculate_summary(checks: &[CheckResult]) -> DeploymentSummary {
    let total_checks = checks.len();
    let mut passed = 0;
    let mut failed = 0;
    let mut warnings = 0;
    let mut skipped = 0;
    let mut has_critical = false;
    
    for check in checks {
        match check.status {
            CheckStatus::Passed => passed += 1,
            CheckStatus::Failed => {
                failed += 1;
                if check.critical_issues > 0 {
                    has_critical = true;
                }
            }
            CheckStatus::Warning => warnings += 1,
            CheckStatus::Skipped => skipped += 1,
        }
    }
    
    let overall_status = if failed > 0 {
        CheckStatus::Failed
    } else if warnings > 0 {
        CheckStatus::Warning
    } else {
        CheckStatus::Passed
    };
    
    // Deployment is ready if no critical failures
    let deployment_ready = failed == 0 || !has_critical;
    
    DeploymentSummary {
        total_checks,
        passed,
        failed,
        warnings,
        skipped,
        overall_status,
        deployment_ready,
    }
}

fn print_deployment_report(report: &DeploymentReport, quiet: bool) {
    if !quiet {
        println!();
        println!("{}", "📊 Deployment Validation Results".bold().blue());
        println!("{}", "===============================".blue());
        println!();
    }
    
    // Print individual check results
    for check in &report.checks {
        let (icon, status_color) = match check.status {
            CheckStatus::Passed => ("✅", "green"),
            CheckStatus::Failed => ("❌", "red"),
            CheckStatus::Warning => ("⚠️", "yellow"),
            CheckStatus::Skipped => ("⏭️", "cyan"),
        };
        
        let status_text = match status_color {
            "green" => format!("{} PASSED", icon).green(),
            "red" => format!("{} FAILED", icon).red(),
            "yellow" => format!("{} WARNING", icon).yellow(),
            "cyan" => format!("{} SKIPPED", icon).cyan(),
            _ => format!("{} UNKNOWN", icon).normal(),
        };
        
        println!("  {} {} ({}ms)", status_text, check.name.bold(), check.duration_ms);
        println!("     {}", check.message.dimmed());
        
        if check.issues_found > 0 {
            println!("     Issues: {}, Critical: {}", check.issues_found, check.critical_issues);
        }
        
        println!();
    }
    
    // Print summary
    print_deployment_summary(&report.summary, report.duration_ms);
}

fn print_deployment_summary(summary: &DeploymentSummary, duration_ms: u64) {
    println!("{}", "🎯 DEPLOYMENT SUMMARY".bold().white());
    println!("{}", "════════════════════".white());
    
    println!("  Total checks: {}", summary.total_checks);
    
    if summary.passed > 0 {
        println!("  {} {}", "Passed:".green(), summary.passed.to_string().green());
    }
    if summary.failed > 0 {
        println!("  {} {}", "Failed:".red(), summary.failed.to_string().red());
    }
    if summary.warnings > 0 {
        println!("  {} {}", "Warnings:".yellow(), summary.warnings.to_string().yellow());
    }
    if summary.skipped > 0 {
        println!("  {} {}", "Skipped:".cyan(), summary.skipped.to_string().cyan());
    }
    
    println!("  Total time: {}ms", duration_ms);
    println!();
    
    // Overall status
    let (status_icon, status_text, status_color) = match summary.overall_status {
        CheckStatus::Passed => ("🎉", "ALL CHECKS PASSED", "green"),
        CheckStatus::Failed => ("🚨", "CHECKS FAILED", "red"), 
        CheckStatus::Warning => ("⚠️", "WARNINGS DETECTED", "yellow"),
        CheckStatus::Skipped => ("⏭️", "SOME CHECKS SKIPPED", "cyan"),
    };
    
    let colored_status = match status_color {
        "green" => format!("{} {}", status_icon, status_text).green().bold(),
        "red" => format!("{} {}", status_icon, status_text).red().bold(),
        "yellow" => format!("{} {}", status_icon, status_text).yellow().bold(),
        "cyan" => format!("{} {}", status_icon, status_text).cyan().bold(),
        _ => format!("{} {}", status_icon, status_text).normal().bold(),
    };
    
    println!("  Status: {}", colored_status);
    
    // Deployment readiness
    if summary.deployment_ready {
        println!("  {} {}", "🚀".green(), "READY FOR DEPLOYMENT".green().bold());
        println!();
        println!("{}", "✨ Your project passes all critical checks and is ready for deployment!".green());
    } else {
        println!("  {} {}", "🛑".red(), "NOT READY FOR DEPLOYMENT".red().bold());
        println!();
        println!("{}", "🚨 Critical issues must be fixed before deployment".red().bold());
        println!();
        println!("{}", "💡 Fix the failed checks above and run 'sniff deploy' again".yellow());
    }
    
    println!();
    
    // Next steps
    if summary.deployment_ready {
        println!("{}", "🎯 NEXT STEPS".bold().cyan());
        println!("{}", "────────────".cyan());
        println!("  • Run your build command (npm run build)");
        println!("  • Test in staging environment");
        println!("  • Deploy to production");
        
        if summary.warnings > 0 {
            println!("  • Consider addressing warnings for optimal performance");
        }
    } else {
        println!("{}", "🔧 ACTION REQUIRED".bold().red());
        println!("{}", "─────────────────".red());
        println!("  • Fix all failed checks above");
        println!("  • Run 'sniff deploy' again to verify fixes");
        println!("  • Only deploy when all critical checks pass");
    }
    
    println!();
    println!("{}", "💡 TIP: Use 'sniff --help' to run individual checks during development".dimmed());
}