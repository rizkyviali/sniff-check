use anyhow::Result;
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
    if !quiet {
        println!("{}", "🚀 Running performance audit...".bold().blue());
        println!("{}", "Please ensure your development server is running".dimmed());
    }
    
    let start_time = Instant::now();
    let report = perform_audit().await?;
    let duration = start_time.elapsed().as_millis() as u64;
    
    let final_report = PerformanceReport {
        audit_results: report.0,
        summary: report.1,
        recommendations: report.2,
        duration_ms: duration,
    };
    
    if json {
        println!("{}", serde_json::to_string_pretty(&final_report)?);
    } else {
        print_performance_report(&final_report, quiet);
    }
    
    // Exit with error if performance is poor
    if final_report.summary.overall_score < 50.0 {
        std::process::exit(1);
    }
    
    Ok(())
}

async fn perform_audit() -> Result<(Vec<AuditResult>, PerformanceSummary, Vec<String>)> {
    let mut audit_results = Vec::new();
    let mut recommendations = Vec::new();
    
    // Check if Lighthouse is available
    let lighthouse_available = check_lighthouse_available();
    
    if lighthouse_available {
        // Run Lighthouse audit
        match run_lighthouse_audit().await {
            Ok((results, recs)) => {
                audit_results.extend(results);
                recommendations.extend(recs);
            }
            Err(_) => {
                // Fallback to basic performance checks
                let (results, recs) = run_basic_performance_checks().await?;
                audit_results.extend(results);
                recommendations.extend(recs);
            }
        }
    } else {
        // Run basic performance checks
        let (results, recs) = run_basic_performance_checks().await?;
        audit_results.extend(results);
        recommendations.extend(recs);
        
        recommendations.insert(0, "Install Lighthouse CLI for comprehensive performance auditing: npm install -g lighthouse".to_string());
    }
    
    let summary = calculate_performance_summary(&audit_results);
    
    Ok((audit_results, summary, recommendations))
}

fn check_lighthouse_available() -> bool {
    Command::new("lighthouse")
        .arg("--version")
        .output()
        .is_ok()
}

async fn run_lighthouse_audit() -> Result<(Vec<AuditResult>, Vec<String>)> {
    let mut audit_results = Vec::new();
    let mut recommendations = Vec::new();
    
    // Try common development server URLs
    let urls = vec![
        "http://localhost:3000",
        "http://localhost:3001", 
        "http://localhost:8000",
        "http://localhost:8080",
    ];
    
    let mut lighthouse_output = None;
    
    for url in urls {
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
    
    if let Some(output) = lighthouse_output {
        let lighthouse_data: serde_json::Value = serde_json::from_str(&output)?;
        
        // Parse Lighthouse results
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
        
        // Add specific performance recommendations
        recommendations.extend(generate_lighthouse_recommendations(&lighthouse_data));
    }
    
    Ok((audit_results, recommendations))
}

async fn run_basic_performance_checks() -> Result<(Vec<AuditResult>, Vec<String>)> {
    let mut audit_results = Vec::new();
    let mut recommendations = Vec::new();
    
    // Check build output size (if exists)
    if let Ok(build_size) = check_build_size().await {
        let status = if build_size < 1.0 {
            PerformanceStatus::Excellent
        } else if build_size < 2.0 {
            PerformanceStatus::Good
        } else if build_size < 5.0 {
            PerformanceStatus::NeedsWork
        } else {
            PerformanceStatus::Poor
        };
        
        audit_results.push(AuditResult {
            name: "Bundle Size".to_string(),
            score: calculate_bundle_score(build_size),
            status,
            value: Some(build_size),
            unit: Some("MB".to_string()),
            description: "Total size of JavaScript bundles".to_string(),
            recommendation: if build_size > 2.0 {
                Some("Consider code splitting and tree shaking to reduce bundle size".to_string())
            } else {
                None
            },
        });
    }
    
    // Check for performance-related files and patterns
    let performance_checks = check_performance_patterns().await?;
    audit_results.extend(performance_checks.0);
    recommendations.extend(performance_checks.1);
    
    // Add general recommendations
    recommendations.extend(vec![
        "Use Next.js Image component for optimized images".to_string(),
        "Implement proper caching strategies".to_string(),
        "Consider using a CDN for static assets".to_string(),
        "Minimize third-party scripts and dependencies".to_string(),
    ]);
    
    Ok((audit_results, recommendations))
}

async fn check_build_size() -> Result<f64> {
    use std::fs;
    use walkdir::WalkDir;
    
    let build_dirs = vec![".next", "dist", "build", "out"];
    let mut total_size = 0u64;
    
    for dir in build_dirs {
        if std::path::Path::new(dir).exists() {
            for entry in WalkDir::new(dir) {
                if let Ok(entry) = entry {
                    if entry.file_type().is_file() {
                        if let Ok(metadata) = fs::metadata(entry.path()) {
                            total_size += metadata.len();
                        }
                    }
                }
            }
            break; // Only check the first existing build directory
        }
    }
    
    Ok(total_size as f64 / 1_048_576.0) // Convert to MB
}

fn calculate_bundle_score(size_mb: f64) -> f64 {
    match size_mb {
        s if s < 0.5 => 100.0,
        s if s < 1.0 => 90.0,
        s if s < 2.0 => 75.0,
        s if s < 3.0 => 60.0,
        s if s < 5.0 => 40.0,
        _ => 20.0,
    }
}

async fn check_performance_patterns() -> Result<(Vec<AuditResult>, Vec<String>)> {
    use walkdir::WalkDir;
    use std::fs;
    
    let mut audit_results = Vec::new();
    let mut recommendations = Vec::new();
    let mut has_lazy_loading = false;
    let mut has_image_optimization = false;
    let mut has_code_splitting = false;
    
    // Scan for performance patterns
    for entry in WalkDir::new(".").max_depth(3) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if matches!(extension.to_str(), Some("ts") | Some("tsx") | Some("js") | Some("jsx")) {
                        if let Ok(content) = fs::read_to_string(path) {
                            // Check for lazy loading patterns
                            if content.contains("React.lazy") || content.contains("dynamic(") {
                                has_lazy_loading = true;
                            }
                            
                            // Check for Next.js Image component
                            if content.contains("next/image") {
                                has_image_optimization = true;
                            }
                            
                            // Check for code splitting
                            if content.contains("import(") {
                                has_code_splitting = true;
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Add audit results for patterns
    audit_results.push(AuditResult {
        name: "Lazy Loading".to_string(),
        score: if has_lazy_loading { 100.0 } else { 0.0 },
        status: if has_lazy_loading { PerformanceStatus::Excellent } else { PerformanceStatus::Poor },
        value: Some(if has_lazy_loading { 1.0 } else { 0.0 }),
        unit: Some("implemented".to_string()),
        description: "Components are loaded lazily to improve performance".to_string(),
        recommendation: if !has_lazy_loading {
            Some("Implement lazy loading for components using React.lazy() or Next.js dynamic()".to_string())
        } else {
            None
        },
    });
    
    audit_results.push(AuditResult {
        name: "Image Optimization".to_string(),
        score: if has_image_optimization { 100.0 } else { 0.0 },
        status: if has_image_optimization { PerformanceStatus::Excellent } else { PerformanceStatus::Poor },
        value: Some(if has_image_optimization { 1.0 } else { 0.0 }),
        unit: Some("implemented".to_string()),
        description: "Images are optimized using Next.js Image component".to_string(),  
        recommendation: if !has_image_optimization {
            Some("Use Next.js Image component for automatic image optimization".to_string())
        } else {
            None
        },
    });
    
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
    
    // Parse audit recommendations from Lighthouse
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
    
    // Calculate category-specific scores
    let performance_score = audit_results.iter()
        .filter(|r| r.name.to_lowercase().contains("performance") || r.name.to_lowercase().contains("bundle"))
        .map(|r| r.score)
        .fold(0.0, |acc, score| if acc == 0.0 { score } else { (acc + score) / 2.0 });
    
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
    
    // Print audit results by category
    let mut categories: HashMap<String, Vec<&AuditResult>> = HashMap::new();
    for result in &report.audit_results {
        let category = if result.name.to_lowercase().contains("bundle") || result.name.to_lowercase().contains("performance") {
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
    
    for (category, results) in categories {
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
            
            println!("  {} {} ({}{})", 
                icon, 
                result.name.bold(),
                colored_score,
                result.unit.as_deref().unwrap_or("")
            );
            
            if !result.description.is_empty() {
                println!("     {}", result.description.dimmed());
            }
            
            if let Some(recommendation) = &result.recommendation {
                println!("     💡 {}", recommendation.yellow());
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
    print_performance_summary(&report.summary, report.duration_ms);
}

fn print_performance_summary(summary: &PerformanceSummary, duration_ms: u64) {
    println!("{}", "📈 PERFORMANCE SUMMARY".bold().white());
    println!("{}", "─────────────────────".white());
    
    let overall_color = match summary.overall_score {
        s if s >= 90.0 => "green",
        s if s >= 75.0 => "yellow", 
        s if s >= 50.0 => "yellow",
        _ => "red",
    };
    
    let colored_score = match overall_color {
        "green" => format!("{:.1}%", summary.overall_score).green(),
        "yellow" => format!("{:.1}%", summary.overall_score).yellow(),
        "red" => format!("{:.1}%", summary.overall_score).red(),
        _ => format!("{:.1}%", summary.overall_score).white(),
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
    
    // Overall assessment
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
        if summary.performance_score < 75.0 {
            println!("  • Optimize Core Web Vitals (LCP, FID, CLS)");
        }
        if summary.accessibility_score < 75.0 {
            println!("  • Improve accessibility compliance");
        }
        if summary.best_practices_score < 75.0 {
            println!("  • Follow web development best practices");
        }
        if summary.seo_score < 75.0 {
            println!("  • Enhance SEO optimization");
        }
    }
    
    println!();
    println!("{}", "💡 TIP: Run performance audits regularly during development".dimmed());
}

// Helper trait for title case conversion
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