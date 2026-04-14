use anyhow::{anyhow, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub audit_results: Vec<AuditResult>,
    pub summary: PerformanceSummary,
    pub recommendations: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditResult {
    pub name: String,
    pub score: f64,
    pub status: PerformanceStatus,
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub description: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PerformanceStatus {
    Excellent,  // 90-100
    Good,       // 75-89
    NeedsWork,  // 50-74
    Poor,       // 0-49
    NotMeasured,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub overall_score: f64,
    pub performance_score: f64,
    pub accessibility_score: f64,
    pub best_practices_score: f64,
    pub seo_score: f64,
    pub total_audits: usize,
    pub passed_audits: usize,
}

pub async fn run(json: bool, quiet: bool) -> Result<()> {
    if !check_lighthouse_available() {
        println!("{}", "📦 sniff perf requires Lighthouse to run.".bold());
        println!();
        println!("  Install it with:");
        println!("    {}", "npm install -g lighthouse".bright_white());
        println!();
        println!("  Then make sure your dev server is running and re-run:");
        println!("    {}", "sniff perf".bright_white());
        return Ok(());
    }

    if !quiet {
        println!("{}", "🚀 Running Lighthouse performance audit...".bold().blue());
        println!("{}", "Please ensure your development server is running".dimmed());
    }

    let start_time = Instant::now();
    let (audit_results, recommendations) = run_lighthouse_audit().await?;
    let duration = start_time.elapsed().as_millis() as u64;

    let summary = calculate_performance_summary(&audit_results);

    let report = PerformanceReport {
        audit_results,
        summary,
        recommendations,
        duration_ms: duration,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_performance_report(&report, quiet);
    }

    if report.summary.overall_score < 50.0 {
        std::process::exit(1);
    }

    Ok(())
}

fn check_lighthouse_available() -> bool {
    Command::new("lighthouse")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

async fn run_lighthouse_audit() -> Result<(Vec<AuditResult>, Vec<String>)> {
    let detected_urls = detect_running_servers().await;

    let fallback_urls = vec![
        "http://localhost:3000".to_string(),
        "http://localhost:3001".to_string(),
        "http://localhost:8000".to_string(),
        "http://localhost:8080".to_string(),
    ];

    let urls = if !detected_urls.is_empty() { detected_urls } else { fallback_urls };

    let mut lighthouse_output = None;

    for url in &urls {
        if let Ok(output) = Command::new("lighthouse")
            .arg(url)
            .arg("--output=json")
            .arg("--only-categories=performance,accessibility,best-practices,seo")
            .arg("--chrome-flags=--headless")
            .arg("--quiet")
            .output()
        {
            if output.status.success() {
                lighthouse_output = Some(String::from_utf8_lossy(&output.stdout).to_string());
                break;
            }
        }
    }

    let output = lighthouse_output.ok_or_else(|| {
        anyhow!(
            "Lighthouse could not reach any running server.\nTried: {}\n\nStart your dev server first (e.g. npm run dev).",
            urls.join(", ")
        )
    })?;

    let lighthouse_data: serde_json::Value = serde_json::from_str(&output)?;

    let mut audit_results = Vec::new();
    let mut recommendations = Vec::new();

    if let Some(categories) = lighthouse_data["categories"].as_object() {
        for (category_name, category) in categories {
            if let Some(score) = category["score"].as_f64() {
                let score_percent = score * 100.0;
                let status = match score_percent {
                    s if s >= 90.0 => PerformanceStatus::Excellent,
                    s if s >= 75.0 => PerformanceStatus::Good,
                    s if s >= 50.0 => PerformanceStatus::NeedsWork,
                    _ => PerformanceStatus::Poor,
                };

                audit_results.push(AuditResult {
                    name: category_name.replace('-', " ").to_title_case(),
                    score: score_percent,
                    status,
                    value: Some(score_percent),
                    unit: Some("%".to_string()),
                    description: format!("{} score from Lighthouse audit", category_name),
                    recommendation: get_category_recommendation(category_name, score_percent),
                });
            }
        }
    }

    recommendations.extend(generate_lighthouse_recommendations(&lighthouse_data));

    Ok((audit_results, recommendations))
}

fn get_category_recommendation(category: &str, score: f64) -> Option<String> {
    match category {
        "performance" if score < 75.0 => Some("Focus on Core Web Vitals: LCP, FID, and CLS metrics".to_string()),
        "accessibility" if score < 75.0 => Some("Add proper ARIA labels, alt text, and keyboard navigation".to_string()),
        "best-practices" if score < 75.0 => Some("Follow security best practices and avoid deprecated APIs".to_string()),
        "seo" if score < 75.0 => Some("Add meta tags, structured data, and improve page titles".to_string()),
        _ => None,
    }
}

fn generate_lighthouse_recommendations(data: &serde_json::Value) -> Vec<String> {
    let mut recommendations = Vec::new();

    if let Some(audits) = data["audits"].as_object() {
        for (audit_id, audit) in audits {
            if let Some(score) = audit["score"].as_f64() {
                if score < 0.9 {
                    match audit_id.as_str() {
                        "first-contentful-paint" => recommendations.push("Optimize First Contentful Paint by reducing server response times".to_string()),
                        "largest-contentful-paint" => recommendations.push("Improve Largest Contentful Paint by optimizing images and preloading key resources".to_string()),
                        "cumulative-layout-shift" => recommendations.push("Reduce Cumulative Layout Shift by setting dimensions on images and embeds".to_string()),
                        "unused-javascript" => recommendations.push("Remove unused JavaScript to reduce bundle size".to_string()),
                        "render-blocking-resources" => recommendations.push("Eliminate render-blocking resources by inlining critical CSS".to_string()),
                        _ => {}
                    }
                }
            }
        }
    }

    recommendations
}

fn calculate_performance_summary(audit_results: &[AuditResult]) -> PerformanceSummary {
    let total_audits = audit_results.len();
    let passed_audits = audit_results.iter().filter(|r| r.score >= 75.0).count();

    let overall_score = if total_audits > 0 {
        audit_results.iter().map(|r| r.score).sum::<f64>() / total_audits as f64
    } else {
        0.0
    };

    let performance_score = audit_results.iter()
        .filter(|r| r.name.to_lowercase().contains("performance") || r.name.to_lowercase().contains("bundle"))
        .map(|r| r.score)
        .fold(0.0_f64, |acc, score| if acc == 0.0 { score } else { (acc + score) / 2.0 });

    let accessibility_score = audit_results.iter()
        .find(|r| r.name.to_lowercase().contains("accessibility"))
        .map(|r| r.score)
        .unwrap_or(0.0);

    let best_practices_score = audit_results.iter()
        .find(|r| r.name.to_lowercase().contains("best"))
        .map(|r| r.score)
        .unwrap_or(0.0);

    let seo_score = audit_results.iter()
        .find(|r| r.name.to_lowercase().contains("seo"))
        .map(|r| r.score)
        .unwrap_or(0.0);

    PerformanceSummary {
        overall_score,
        performance_score,
        accessibility_score,
        best_practices_score,
        seo_score,
        total_audits,
        passed_audits,
    }
}

fn print_performance_report(report: &PerformanceReport, quiet: bool) {
    if !quiet {
        println!();
        println!("{}", "🚀 Performance Audit Report".bold().blue());
        println!("{}", "==========================".blue());
        println!();
    }

    let mut categories: HashMap<String, Vec<&AuditResult>> = HashMap::new();
    for result in &report.audit_results {
        let category = if result.name.to_lowercase().contains("performance") {
            "Performance"
        } else if result.name.to_lowercase().contains("accessibility") {
            "Accessibility"
        } else if result.name.to_lowercase().contains("seo") {
            "SEO"
        } else if result.name.to_lowercase().contains("best") {
            "Best Practices"
        } else {
            "General"
        };
        categories.entry(category.to_string()).or_default().push(result);
    }

    for (category, results) in &categories {
        println!("{}", format!("📊 {}", category.to_uppercase()).bold().white());
        println!("{}", "─".repeat(category.len() + 4).white());

        for result in results {
            let (icon, color) = match result.status {
                PerformanceStatus::Excellent => ("🟢", "green"),
                PerformanceStatus::Good => ("🟡", "yellow"),
                PerformanceStatus::NeedsWork => ("🟠", "yellow"),
                PerformanceStatus::Poor => ("🔴", "red"),
                PerformanceStatus::NotMeasured => ("⚪", "white"),
            };

            let score_text = format!("{:.1}", result.score);
            let colored_score = match color {
                "green" => score_text.green(),
                "yellow" => score_text.yellow(),
                "red" => score_text.red(),
                _ => score_text.white(),
            };
            let unit_suffix = result.unit.as_deref().map(|u| format!(" {}", u)).unwrap_or_default();

            println!("  {} {} ({}{})", icon, result.name.bold(), colored_score, unit_suffix);

            if !result.description.is_empty() {
                println!("     {}", result.description.dimmed());
            }

            if let Some(recommendation) = &result.recommendation {
                println!("     💡 {}", recommendation.yellow());
            }
        }
        println!();
    }

    if !report.recommendations.is_empty() {
        println!("{}", "💡 RECOMMENDATIONS".bold().green());
        println!("{}", "──────────────────".green());
        for rec in &report.recommendations {
            println!("  • {}", rec.green());
        }
        println!();
    }

    print_performance_summary(&report.summary, report.duration_ms);
}

fn print_performance_summary(summary: &PerformanceSummary, duration_ms: u64) {
    println!("{}", "📈 PERFORMANCE SUMMARY".bold().white());
    println!("{}", "─────────────────────".white());

    let colored_score = match summary.overall_score {
        s if s >= 90.0 => format!("{:.1}%", s).green(),
        s if s >= 75.0 => format!("{:.1}%", s).yellow(),
        s if s >= 50.0 => format!("{:.1}%", s).yellow(),
        s => format!("{:.1}%", s).red(),
    };

    println!("  Overall Score: {}", colored_score);

    if summary.performance_score > 0.0 {
        println!("  Performance: {:.1}%", summary.performance_score);
    }
    if summary.accessibility_score > 0.0 {
        println!("  Accessibility: {:.1}%", summary.accessibility_score);
    }
    if summary.best_practices_score > 0.0 {
        println!("  Best Practices: {:.1}%", summary.best_practices_score);
    }
    if summary.seo_score > 0.0 {
        println!("  SEO: {:.1}%", summary.seo_score);
    }

    println!("  Audits passed: {}/{}", summary.passed_audits, summary.total_audits);
    println!("  Audit time: {}ms", duration_ms);
    println!();

    let (status_icon, status_text, status_color) = match summary.overall_score {
        s if s >= 90.0 => ("🎉", "EXCELLENT PERFORMANCE", "green"),
        s if s >= 75.0 => ("✅", "GOOD PERFORMANCE", "green"),
        s if s >= 50.0 => ("⚠️", "NEEDS IMPROVEMENT", "yellow"),
        _ => ("🚨", "POOR PERFORMANCE", "red"),
    };

    let colored_status = match status_color {
        "green" => format!("{} {}", status_icon, status_text).green().bold(),
        "yellow" => format!("{} {}", status_icon, status_text).yellow().bold(),
        "red" => format!("{} {}", status_icon, status_text).red().bold(),
        _ => format!("{} {}", status_icon, status_text).white().bold(),
    };

    println!("  Status: {}", colored_status);

    if summary.overall_score < 75.0 {
        println!();
        println!("{}", "🎯 FOCUS AREAS".bold().cyan());
        println!("{}", "─────────────".cyan());
        if summary.performance_score > 0.0 && summary.performance_score < 75.0 {
            println!("  • Optimize Core Web Vitals (LCP, FID, CLS)");
        }
        if summary.accessibility_score > 0.0 && summary.accessibility_score < 75.0 {
            println!("  • Improve accessibility compliance");
        }
        if summary.best_practices_score > 0.0 && summary.best_practices_score < 75.0 {
            println!("  • Follow web development best practices");
        }
        if summary.seo_score > 0.0 && summary.seo_score < 75.0 {
            println!("  • Enhance SEO optimization");
        }
    }

    println!();
    println!("{}", "💡 TIP: Run performance audits regularly during development".dimmed());
}

trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for str {
    fn to_title_case(&self) -> String {
        self.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
}

async fn detect_running_servers() -> Vec<String> {
    let mut detected_servers = Vec::new();

    let ports_to_check = vec![
        3000, 3001, 3002, 3003,
        4200, 4201,
        8000, 8001, 8080, 8081,
        5000, 5001, 5173, 5174,
        9000, 9001,
        1234,
    ];

    for port in ports_to_check {
        if is_port_responsive(port).await {
            let url = format!("http://localhost:{}", port);
            if is_http_server_responsive(&url).await {
                detected_servers.push(url);
            }
        }
    }

    if let Ok(framework_servers) = detect_framework_servers().await {
        detected_servers.extend(framework_servers);
    }

    detected_servers
}

async fn is_port_responsive(port: u16) -> bool {
    use std::net::{TcpStream, SocketAddr};
    use std::time::Duration;

    let addr = format!("127.0.0.1:{}", port);
    if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
        TcpStream::connect_timeout(&socket_addr, Duration::from_millis(100)).is_ok()
    } else {
        false
    }
}

async fn is_http_server_responsive(url: &str) -> bool {
    if let Ok(output) = tokio::process::Command::new("curl")
        .arg("-s")
        .arg("-I")
        .arg("--connect-timeout")
        .arg("1")
        .arg("--max-time")
        .arg("2")
        .arg(url)
        .output()
        .await
    {
        let response = String::from_utf8_lossy(&output.stdout);
        response.starts_with("HTTP/") && (response.contains("200") || response.contains("404"))
    } else {
        false
    }
}

async fn detect_framework_servers() -> Result<Vec<String>> {
    let mut servers = Vec::new();

    if let Ok(output) = tokio::process::Command::new("pgrep")
        .arg("-f").arg("next dev")
        .output().await
    {
        if output.status.success() && !output.stdout.is_empty() && is_port_responsive(3000).await {
            servers.push("http://localhost:3000".to_string());
        }
    }

    if let Ok(output) = tokio::process::Command::new("pgrep")
        .arg("-f").arg("vite")
        .output().await
    {
        if output.status.success() && !output.stdout.is_empty() && is_port_responsive(5173).await {
            servers.push("http://localhost:5173".to_string());
        }
    }

    if let Ok(output) = tokio::process::Command::new("pgrep")
        .arg("-f").arg("ng serve")
        .output().await
    {
        if output.status.success() && !output.stdout.is_empty() && is_port_responsive(4200).await {
            servers.push("http://localhost:4200".to_string());
        }
    }

    Ok(servers)
}
