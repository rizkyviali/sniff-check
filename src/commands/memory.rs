use anyhow::Result;
use colored::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::time::Instant;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryReport {
    pub patterns: Vec<MemoryPattern>,
    pub node_processes: Vec<NodeProcess>,
    pub summary: MemorySummary,
    pub recommendations: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryPattern {
    pub file_path: String,
    pub line_number: usize,
    pub pattern_type: PatternType,
    pub code_snippet: String,
    pub severity: Severity,
    pub description: String,
    pub recommendation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PatternType {
    UnboundedArrayGrowth,
    UnremovedEventListener,
    CircularReference,
    LargeObjectRetention,
    UncontrolledLoop,
    TimerLeak,
    DomElementLeak,
    ClosureLeak,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeProcess {
    pub pid: u32,
    pub memory_usage_mb: f64,
    pub cpu_usage: f64,
    pub command: String,
    pub status: ProcessStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProcessStatus {
    Normal,
    HighMemory,
    MemoryLeak,
    Unresponsive,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemorySummary {
    pub total_patterns: usize,
    pub critical_issues: usize,
    pub high_issues: usize,
    pub medium_issues: usize,
    pub low_issues: usize,
    pub active_processes: usize,
    pub high_memory_processes: usize,
}

pub async fn run(json: bool, quiet: bool) -> Result<()> {
    if !quiet {
        println!("{}", "🔍 Analyzing memory usage and potential leaks...".bold().blue());
    }
    
    let start_time = Instant::now();
    let report = analyze_memory_issues().await?;
    let duration = start_time.elapsed().as_millis() as u64;
    
    let final_report = MemoryReport {
        patterns: report.0,
        node_processes: report.1,
        summary: report.2,
        recommendations: report.3,
        duration_ms: duration,
    };
    
    if json {
        println!("{}", serde_json::to_string_pretty(&final_report)?);
    } else {
        print_memory_report(&final_report, quiet);
    }
    
    // Exit with error if critical memory issues found
    if final_report.summary.critical_issues > 0 || final_report.summary.high_memory_processes > 2 {
        std::process::exit(1);
    }
    
    Ok(())
}

async fn analyze_memory_issues() -> Result<(Vec<MemoryPattern>, Vec<NodeProcess>, MemorySummary, Vec<String>)> {
    let mut patterns = Vec::new();
    let mut recommendations = Vec::new();
    
    // Scan code for memory leak patterns
    let code_patterns = scan_for_memory_patterns().await?;
    patterns.extend(code_patterns.0);
    recommendations.extend(code_patterns.1);
    
    // Check running Node.js processes
    let node_processes = check_node_processes().await?;
    
    // Generate recommendations based on findings
    recommendations.extend(generate_memory_recommendations(&patterns, &node_processes));
    
    let summary = calculate_memory_summary(&patterns, &node_processes);
    
    Ok((patterns, node_processes, summary, recommendations))
}

async fn scan_for_memory_patterns() -> Result<(Vec<MemoryPattern>, Vec<String>)> {
    let mut patterns = Vec::new();
    let mut recommendations = Vec::new();
    
    // Memory leak detection patterns
    let leak_patterns = get_memory_leak_patterns();
    
    // Scan TypeScript/JavaScript files
    for entry in WalkDir::new(".").max_depth(5) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if matches!(extension.to_str(), Some("ts") | Some("tsx") | Some("js") | Some("jsx")) {
                        if let Ok(content) = fs::read_to_string(path) {
                            let file_patterns = analyze_file_for_patterns(path.to_string_lossy().to_string(), &content, &leak_patterns)?;
                            patterns.extend(file_patterns);
                        }
                    }
                }
            }
        }
    }
    
    // Generate basic recommendations
    if !patterns.is_empty() {
        recommendations.push("Review identified memory leak patterns and implement proper cleanup".to_string());
        recommendations.push("Use proper cleanup in useEffect hooks and component unmounting".to_string());
        recommendations.push("Monitor memory usage during development and testing".to_string());
    }
    
    Ok((patterns, recommendations))
}

fn get_memory_leak_patterns() -> Vec<(PatternType, Regex, Severity, String, String)> {
    vec![
        (
            PatternType::UnremovedEventListener,
            Regex::new(r"addEventListener\([^)]+\)(?![\s\S]*removeEventListener)").unwrap(),
            Severity::High,
            "Event listener added without corresponding removal".to_string(),
            "Add removeEventListener in cleanup function or useEffect return".to_string(),
        ),
        (
            PatternType::TimerLeak,
            Regex::new(r"setInterval\([^)]+\)(?![\s\S]*clearInterval)").unwrap(),
            Severity::High,
            "setInterval without clearInterval cleanup".to_string(),
            "Store interval ID and call clearInterval in cleanup".to_string(),
        ),
        (
            PatternType::TimerLeak,
            Regex::new(r"setTimeout\([^)]+\)(?![\s\S]*clearTimeout)").unwrap(),
            Severity::Medium,
            "setTimeout without clearTimeout cleanup".to_string(),
            "Store timeout ID and call clearTimeout in cleanup if needed".to_string(),
        ),
        (
            PatternType::UnboundedArrayGrowth,
            Regex::new(r"\w+\.push\([^)]+\)").unwrap(),
            Severity::Medium,
            "Array push without bounds checking".to_string(),
            "Implement array size limits or periodic cleanup".to_string(),
        ),
        (
            PatternType::UncontrolledLoop,
            Regex::new(r"while\s*\(\s*true\s*\)").unwrap(),
            Severity::Critical,
            "Infinite loop detected".to_string(),
            "Add proper exit conditions to prevent infinite loops".to_string(),
        ),
        (
            PatternType::CircularReference,
            Regex::new(r"(\w+)\.(\w+)\s*=\s*\1").unwrap(),
            Severity::High,
            "Potential circular reference pattern".to_string(),
            "Use WeakMap, WeakSet, or break circular references manually".to_string(),
        ),
        (
            PatternType::LargeObjectRetention,
            Regex::new(r"new\s+Array\(\s*\d{4,}\s*\)").unwrap(),
            Severity::Medium,
            "Large array allocation".to_string(),
            "Consider lazy loading or chunking for large data sets".to_string(),
        ),
        (
            PatternType::ClosureLeak,
            Regex::new(r"function[^{]*\{[\s\S]*function[^{]*\{[\s\S]*\}[\s\S]*\}").unwrap(),
            Severity::Low,
            "Nested function closures may retain outer scope".to_string(),
            "Minimize closure scope and avoid unnecessary variable capture".to_string(),
        ),
    ]
}

fn analyze_file_for_patterns(file_path: String, content: &str, patterns: &[(PatternType, Regex, Severity, String, String)]) -> Result<Vec<MemoryPattern>> {
    let mut file_patterns = Vec::new();
    
    for (line_num, line) in content.lines().enumerate() {
        for (pattern_type, regex, severity, description, recommendation) in patterns {
            if regex.is_match(line) {
                // Skip if it's in a comment
                let trimmed_line = line.trim();
                if trimmed_line.starts_with("//") || trimmed_line.starts_with("/*") {
                    continue;
                }
                
                file_patterns.push(MemoryPattern {
                    file_path: file_path.clone(),
                    line_number: line_num + 1,
                    pattern_type: pattern_type.clone(),
                    code_snippet: line.trim().to_string(),
                    severity: severity.clone(),
                    description: description.clone(),
                    recommendation: recommendation.clone(),
                });
            }
        }
    }
    
    Ok(file_patterns)
}

async fn check_node_processes() -> Result<Vec<NodeProcess>> {
    let mut processes = Vec::new();
    
    // Try to get process information using ps command
    if let Ok(output) = Command::new("ps")
        .arg("aux")
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) { // Skip header
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 11 {
                    let command = fields[10..].join(" ");
                    if command.contains("node") || command.contains("npm") || command.contains("yarn") {
                        if let (Ok(pid), Ok(cpu), Ok(mem)) = (
                            fields[1].parse::<u32>(),
                            fields[2].parse::<f64>(),
                            fields[3].parse::<f64>()
                        ) {
                            // Convert memory percentage to MB (rough estimation)
                            let memory_mb = mem * 16.0; // Rough conversion assuming 16GB system
                            
                            let status = if memory_mb > 1000.0 {
                                ProcessStatus::MemoryLeak
                            } else if memory_mb > 500.0 {
                                ProcessStatus::HighMemory
                            } else {
                                ProcessStatus::Normal
                            };
                            
                            processes.push(NodeProcess {
                                pid,
                                memory_usage_mb: memory_mb,
                                cpu_usage: cpu,
                                command: command.chars().take(80).collect(), // Truncate long commands
                                status,
                            });
                        }
                    }
                }
            }
        }
    }
    
    Ok(processes)
}

fn generate_memory_recommendations(patterns: &[MemoryPattern], processes: &[NodeProcess]) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    // Pattern-based recommendations
    let critical_count = patterns.iter().filter(|p| matches!(p.severity, Severity::Critical)).count();
    let high_count = patterns.iter().filter(|p| matches!(p.severity, Severity::High)).count();
    
    if critical_count > 0 {
        recommendations.push(format!("🚨 {} critical memory issues require immediate attention", critical_count));
    }
    
    if high_count > 0 {
        recommendations.push(format!("⚠️ {} high-priority memory issues should be addressed", high_count));
    }
    
    // Process-based recommendations
    let high_memory_processes = processes.iter().filter(|p| matches!(p.status, ProcessStatus::HighMemory | ProcessStatus::MemoryLeak)).count();
    
    if high_memory_processes > 0 {
        recommendations.push(format!("Monitor {} high-memory Node.js processes", high_memory_processes));
    }
    
    // Pattern type specific recommendations
    let has_event_listeners = patterns.iter().any(|p| matches!(p.pattern_type, PatternType::UnremovedEventListener));
    let has_timers = patterns.iter().any(|p| matches!(p.pattern_type, PatternType::TimerLeak));
    
    if has_event_listeners {
        recommendations.push("Implement proper event listener cleanup in React useEffect dependencies".to_string());
    }
    
    if has_timers {
        recommendations.push("Use React useEffect cleanup functions for timers and intervals".to_string());
    }
    
    // General recommendations
    recommendations.extend(vec![
        "Use React DevTools Profiler to identify memory leaks during development".to_string(),
        "Implement memory monitoring in production environments".to_string(),
        "Consider using WeakMap and WeakSet for managing object references".to_string(),
        "Profile memory usage before and after major code changes".to_string(),
    ]);
    
    recommendations
}

fn calculate_memory_summary(patterns: &[MemoryPattern], processes: &[NodeProcess]) -> MemorySummary {
    let total_patterns = patterns.len();
    let critical_issues = patterns.iter().filter(|p| matches!(p.severity, Severity::Critical)).count();
    let high_issues = patterns.iter().filter(|p| matches!(p.severity, Severity::High)).count();
    let medium_issues = patterns.iter().filter(|p| matches!(p.severity, Severity::Medium)).count();
    let low_issues = patterns.iter().filter(|p| matches!(p.severity, Severity::Low)).count();
    
    let active_processes = processes.len();
    let high_memory_processes = processes.iter().filter(|p| matches!(p.status, ProcessStatus::HighMemory | ProcessStatus::MemoryLeak)).count();
    
    MemorySummary {
        total_patterns,
        critical_issues,
        high_issues,
        medium_issues,
        low_issues,
        active_processes,
        high_memory_processes,
    }
}

fn print_memory_report(report: &MemoryReport, quiet: bool) {
    if !quiet {
        println!();
        println!("{}", "🧠 Memory Leak Analysis Report".bold().blue());
        println!("{}", "=============================".blue());
        println!();
    }
    
    // Print memory leak patterns by severity
    let critical_patterns: Vec<_> = report.patterns.iter().filter(|p| matches!(p.severity, Severity::Critical)).collect();
    let high_patterns: Vec<_> = report.patterns.iter().filter(|p| matches!(p.severity, Severity::High)).collect();
    let medium_patterns: Vec<_> = report.patterns.iter().filter(|p| matches!(p.severity, Severity::Medium)).collect();
    let low_patterns: Vec<_> = report.patterns.iter().filter(|p| matches!(p.severity, Severity::Low)).collect();
    
    if !critical_patterns.is_empty() {
        println!("{}", "🚨 CRITICAL MEMORY ISSUES".bold().red());
        println!("{}", "───────────────────────────".red());
        for pattern in critical_patterns {
            print_memory_pattern(pattern);
        }
        println!();
    }
    
    if !high_patterns.is_empty() {
        println!("{}", "⚠️  HIGH PRIORITY ISSUES".bold().yellow());
        println!("{}", "───────────────────────".yellow());
        for pattern in high_patterns {
            print_memory_pattern(pattern);
        }
        println!();
    }
    
    if !medium_patterns.is_empty() {
        println!("{}", "📋 MEDIUM PRIORITY ISSUES".bold().white());
        println!("{}", "────────────────────────".white());
        for pattern in medium_patterns {
            print_memory_pattern(pattern);
        }
        println!();
    }
    
    if !low_patterns.is_empty() && !quiet {
        println!("{}", "ℹ️  LOW PRIORITY ISSUES".bold().cyan());
        println!("{}", "──────────────────────".cyan());
        for pattern in low_patterns {
            print_memory_pattern(pattern);
        }
        println!();
    }
    
    // Print Node.js processes
    if !report.node_processes.is_empty() {
        println!("{}", "🔄 NODE.JS PROCESSES".bold().white());
        println!("{}", "────────────────────".white());
        
        for process in &report.node_processes {
            let (status_icon, status_color) = match process.status {
                ProcessStatus::Normal => ("✅", "green"),
                ProcessStatus::HighMemory => ("⚠️", "yellow"),
                ProcessStatus::MemoryLeak => ("🚨", "red"),
                ProcessStatus::Unresponsive => ("💀", "red"),
            };
            
            let memory_text = format!("{:.1}MB", process.memory_usage_mb);
            let colored_memory = match status_color {
                "green" => memory_text.green(),
                "yellow" => memory_text.yellow(),
                "red" => memory_text.red(),
                _ => memory_text.white(),
            };
            
            println!("  {} PID: {} | Memory: {} | CPU: {:.1}%", 
                status_icon, 
                process.pid, 
                colored_memory,
                process.cpu_usage
            );
            println!("     {}", process.command.dimmed());
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
    print_memory_summary(&report.summary, report.duration_ms);
}

fn print_memory_pattern(pattern: &MemoryPattern) {
    let severity_icon = match pattern.severity {
        Severity::Critical => "🚨".red(),
        Severity::High => "⚠️".yellow(),
        Severity::Medium => "📋".white(),
        Severity::Low => "ℹ️".cyan(),
    };
    
    println!("  {} {}:{}", severity_icon, pattern.file_path, pattern.line_number);
    println!("     {}", pattern.code_snippet.dimmed());
    println!("     {}", pattern.description.yellow());
    println!("     💡 {}", pattern.recommendation.green());
    println!();
}

fn print_memory_summary(summary: &MemorySummary, duration_ms: u64) {
    println!("{}", "📊 MEMORY ANALYSIS SUMMARY".bold().white());
    println!("{}", "─────────────────────────".white());
    
    println!("  Total patterns found: {}", summary.total_patterns);
    
    if summary.critical_issues > 0 {
        println!("  {} {}", "Critical issues:".red(), summary.critical_issues.to_string().red());
    }
    if summary.high_issues > 0 {
        println!("  {} {}", "High priority:".yellow(), summary.high_issues.to_string().yellow());
    }
    if summary.medium_issues > 0 {
        println!("  {} {}", "Medium priority:".white(), summary.medium_issues.to_string().white());
    }
    if summary.low_issues > 0 {
        println!("  {} {}", "Low priority:".cyan(), summary.low_issues.to_string().cyan());
    }
    
    println!("  Active Node.js processes: {}", summary.active_processes);
    if summary.high_memory_processes > 0 {
        println!("  {} {}", "High memory processes:".red(), summary.high_memory_processes.to_string().red());
    }
    
    println!("  Analysis time: {}ms", duration_ms);
    println!();
    
    // Overall assessment
    let overall_status = if summary.critical_issues > 0 {
        ("🚨", "CRITICAL MEMORY ISSUES DETECTED", "red")
    } else if summary.high_issues > 3 || summary.high_memory_processes > 2 {
        ("⚠️", "MEMORY ISSUES NEED ATTENTION", "yellow")
    } else if summary.total_patterns > 0 {
        ("📋", "MINOR MEMORY CONCERNS", "white")
    } else {
        ("✅", "NO MAJOR MEMORY ISSUES", "green")
    };
    
    let colored_status = match overall_status.2 {
        "green" => format!("{} {}", overall_status.0, overall_status.1).green().bold(),
        "yellow" => format!("{} {}", overall_status.0, overall_status.1).yellow().bold(),
        "red" => format!("{} {}", overall_status.0, overall_status.1).red().bold(),
        _ => format!("{} {}", overall_status.0, overall_status.1).white().bold(),
    };
    
    println!("  Status: {}", colored_status);
    
    if summary.critical_issues > 0 || summary.high_memory_processes > 2 {
        println!();
        println!("{}", "🎯 ACTION REQUIRED".bold().red());
        println!("{}", "─────────────────".red());
        if summary.critical_issues > 0 {
            println!("  • Fix critical memory leak patterns immediately");
        }
        if summary.high_memory_processes > 2 {
            println!("  • Investigate high-memory Node.js processes");
        }
        println!("  • Monitor memory usage during development");
        println!("  • Set up memory alerts in production");
    }
    
    println!();
    println!("{}", "💡 TIP: Use 'node --max-old-space-size=4096' to increase Node.js memory limit if needed".dimmed());
}