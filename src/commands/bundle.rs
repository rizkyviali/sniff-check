use anyhow::{anyhow, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleReport {
    pub chunks: Vec<BundleChunk>,
    pub summary: BundleSummary,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleChunk {
    pub name: String,
    pub size_bytes: u64,
    pub size_compressed: Option<u64>,
    pub chunk_type: ChunkType,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkType {
    Main,
    Page,
    Component,
    Vendor,
    Runtime,
    Static,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Framework {
    NextJs,
    React,
    Vue,
    Angular,
    Svelte,
    Vite,
    Webpack,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FrameworkLimits {
    pub max_total_size_mb: f64,
    pub max_main_chunk_mb: f64,
    pub max_vendor_chunk_mb: f64,
    pub performance_budget_mb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleSummary {
    pub total_size: u64,
    pub total_compressed: u64,
    pub chunk_count: usize,
    pub largest_chunk: Option<String>,
    pub compression_ratio: f64,
    pub warnings: Vec<String>,
}

pub async fn run(_json: bool, quiet: bool) -> Result<()> {
    if !quiet {
        println!("{}", "🔍 Analyzing bundle size...".bold().blue());
    }
    
    let report = analyze_bundle(quiet).await?;
    
    if _json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report, quiet);
    }
    
    // Exit with error if bundles are too large
    if report.summary.total_size > 2_000_000 || has_oversized_chunks(&report) {
        std::process::exit(1);
    }
    
    Ok(())
}

async fn analyze_bundle(quiet: bool) -> Result<BundleReport> {
    // Check if this is a Next.js project
    let current_dir = std::env::current_dir()?;
    
    if !quiet {
        println!("🔍 Searching for build output directories...");
        println!("📂 Scanning for build files...");
    }
    
    // Look for Next.js build output
    let next_build_dir = current_dir.join(".next");
    if next_build_dir.exists() {
        if !quiet {
            println!("📁 Found Next.js build output in .next/");
        }
        analyze_nextjs_bundle(&next_build_dir, quiet).await
    } else {
        // Look for other common build outputs
        let potential_dirs = vec!["dist", "build", "out"];
        
        for dir_name in potential_dirs {
            let build_dir = current_dir.join(dir_name);
            if build_dir.exists() {
                if !quiet {
                    println!("📁 Found build output in {}/", dir_name);
                }
                return analyze_generic_bundle(&build_dir, quiet).await;
            }
        }
        
        Err(anyhow!("No build output found. Please run 'npm run build' or equivalent first."))
    }
}

async fn analyze_nextjs_bundle(next_dir: &Path, quiet: bool) -> Result<BundleReport> {
    let mut chunks = Vec::new();
    let mut total_size = 0u64;
    let mut total_compressed = 0u64;
    
    if !quiet {
        println!("📊 Analyzing Next.js bundle structure...");
        println!("⚙️ Processing bundle chunks...");
    }
    
    // Analyze static chunks
    let static_dir = next_dir.join("static");
    if static_dir.exists() {
        if !quiet {
            println!("📁 Analyzing static chunks...");
        }
        chunks.extend(analyze_static_chunks(&static_dir)?);
    }
    
    // Analyze pages
    let pages_dir = next_dir.join("server").join("pages");
    if pages_dir.exists() {
        if !quiet {
            println!("📄 Analyzing page chunks...");
        }
        chunks.extend(analyze_pages_chunks(&pages_dir)?);
    }
    
    // Calculate totals
    for chunk in &chunks {
        total_size += chunk.size_bytes;
        if let Some(compressed) = chunk.size_compressed {
            total_compressed += compressed;
        }
    }
    
    if chunks.is_empty() {
        return Err(anyhow!("No bundle chunks found in .next directory. Please run 'npm run build' first."));
    }
    
    if !quiet {
        println!("✅ Bundle analysis completed");
    }
    
    let compression_ratio = if total_size > 0 {
        (total_compressed as f64) / (total_size as f64)
    } else {
        1.0
    };
    
    let largest_chunk = chunks
        .iter()
        .max_by_key(|c| c.size_bytes)
        .map(|c| c.name.clone());
    
    let warnings = generate_warnings(&chunks, next_dir);
    let recommendations = generate_recommendations(&chunks, next_dir);
    
    let chunk_count = chunks.len();
    
    Ok(BundleReport {
        chunks,
        summary: BundleSummary {
            total_size,
            total_compressed,
            chunk_count,
            largest_chunk,
            compression_ratio,
            warnings,
        },
        recommendations,
    })
}

async fn analyze_generic_bundle(build_dir: &Path, quiet: bool) -> Result<BundleReport> {
    let mut chunks = Vec::new();
    let mut total_size = 0u64;
    
    // Walk through build directory
    for entry in WalkDir::new(build_dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(extension) = entry.path().extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if matches!(ext.as_str(), "js" | "css" | "html" | "json") {
                    let size = entry.metadata()?.len();
                    total_size += size;
                    
                    let chunk_type = determine_chunk_type_from_path(entry.path());
                    let name = entry.file_name().to_string_lossy().to_string();
                    
                    chunks.push(BundleChunk {
                        name,
                        size_bytes: size,
                        size_compressed: None,
                        chunk_type,
                        path: entry.path().to_string_lossy().to_string(),
                    });
                }
            }
        }
    }
    
    if chunks.is_empty() {
        return Err(anyhow!("No bundle files found in build directory."));
    }
    
    let largest_chunk = chunks
        .iter()
        .max_by_key(|c| c.size_bytes)
        .map(|c| c.name.clone());
    
    let warnings = generate_warnings(&chunks, build_dir);
    let recommendations = generate_recommendations(&chunks, build_dir);
    
    let chunk_count = chunks.len();
    
    Ok(BundleReport {
        chunks,
        summary: BundleSummary {
            total_size,
            total_compressed: 0,
            chunk_count,
            largest_chunk,
            compression_ratio: 1.0,
            warnings,
        },
        recommendations,
    })
}

fn analyze_static_chunks(static_dir: &Path) -> Result<Vec<BundleChunk>> {
    let mut chunks = Vec::new();
    
    for entry in WalkDir::new(static_dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(extension) = entry.path().extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if matches!(ext.as_str(), "js" | "css") {
                    let size = entry.metadata()?.len();
                    let name = entry.file_name().to_string_lossy().to_string();
                    let chunk_type = determine_chunk_type(&name);
                    
                    chunks.push(BundleChunk {
                        name: name.clone(),
                        size_bytes: size,
                        size_compressed: estimate_compressed_size(size),
                        chunk_type,
                        path: entry.path().to_string_lossy().to_string(),
                    });
                }
            }
        }
    }
    
    Ok(chunks)
}

fn analyze_pages_chunks(pages_dir: &Path) -> Result<Vec<BundleChunk>> {
    let mut chunks = Vec::new();
    
    for entry in WalkDir::new(pages_dir) {
        let entry = entry?;
        if entry.file_type().is_file() && entry.path().extension().map_or(false, |e| e == "js") {
            let size = entry.metadata()?.len();
            let name = entry.file_name().to_string_lossy().to_string();
            
            chunks.push(BundleChunk {
                name: name.clone(),
                size_bytes: size,
                size_compressed: estimate_compressed_size(size),
                chunk_type: ChunkType::Page,
                path: entry.path().to_string_lossy().to_string(),
            });
        }
    }
    
    Ok(chunks)
}

fn determine_chunk_type(filename: &str) -> ChunkType {
    let filename_lower = filename.to_lowercase();
    
    if filename_lower.contains("main") {
        ChunkType::Main
    } else if filename_lower.contains("vendor") || filename_lower.contains("node_modules") {
        ChunkType::Vendor
    } else if filename_lower.contains("runtime") {
        ChunkType::Runtime
    } else if filename_lower.contains("page") {
        ChunkType::Page
    } else if filename_lower.ends_with(".css") {
        ChunkType::Static
    } else {
        ChunkType::Component
    }
}

fn determine_chunk_type_from_path(path: &Path) -> ChunkType {
    let path_str = path.to_string_lossy().to_lowercase();
    
    if path_str.contains("vendor") || path_str.contains("node_modules") {
        ChunkType::Vendor
    } else if path_str.contains("page") {
        ChunkType::Page
    } else if path_str.ends_with(".css") {
        ChunkType::Static
    } else {
        ChunkType::Component
    }
}

fn estimate_compressed_size(original_size: u64) -> Option<u64> {
    // Rough estimation: gzip typically achieves 60-80% compression for JS/CSS
    Some((original_size as f64 * 0.35) as u64)
}

fn generate_warnings(chunks: &[BundleChunk], build_dir: &Path) -> Vec<String> {
    let mut warnings = Vec::new();
    
    // Check for oversized chunks
    for chunk in chunks {
        if chunk.size_bytes > 500_000 {
            warnings.push(format!("Large chunk detected: {} ({} KB)", 
                chunk.name, chunk.size_bytes / 1024));
        }
    }
    
    // Check total bundle size with framework-specific limits
    let total_size: u64 = chunks.iter().map(|c| c.size_bytes).sum();
    let framework = detect_framework(build_dir);
    let limits = get_framework_limits(&framework);
    let total_size_mb = total_size as f64 / 1_000_000.0;
    
    if total_size_mb > limits.max_total_size_mb {
        let framework_name = match framework {
            Framework::NextJs => "Next.js",
            Framework::React => "React",
            Framework::Vue => "Vue",
            Framework::Angular => "Angular",
            Framework::Svelte => "Svelte",
            Framework::Vite => "Vite",
            Framework::Webpack => "Webpack",
            Framework::Unknown => "JavaScript",
        };
        
        warnings.push(format!("Total bundle size ({:.1} MB) exceeds recommended {} app limit ({:.1} MB)", 
            total_size_mb, framework_name, limits.max_total_size_mb));
        
        // Add performance budget warning if significantly over
        if total_size_mb > limits.performance_budget_mb {
            warnings.push(format!("Bundle size significantly exceeds performance budget ({:.1} MB) - consider aggressive optimization", 
                limits.performance_budget_mb));
        }
    }
    
    warnings
}

fn generate_recommendations(chunks: &[BundleChunk], build_dir: &Path) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    // Analyze chunk distribution
    let mut chunk_types: HashMap<String, u64> = HashMap::new();
    for chunk in chunks {
        let type_name = format!("{:?}", chunk.chunk_type);
        *chunk_types.entry(type_name).or_insert(0) += chunk.size_bytes;
    }
    
    // Check for vendor chunk size
    if let Some(vendor_size) = chunk_types.get("Vendor") {
        if *vendor_size > 800_000 {
            recommendations.push("Consider code splitting to reduce vendor bundle size".to_string());
            recommendations.push("Use dynamic imports for heavy libraries".to_string());
        }
    }
    
    // Check for main chunk size
    if let Some(main_size) = chunk_types.get("Main") {
        if *main_size > 300_000 {
            recommendations.push("Main bundle is large - consider lazy loading routes".to_string());
        }
    }
    
    // General recommendations
    recommendations.push("Enable gzip/brotli compression on your server".to_string());
    recommendations.push("Consider using a CDN for static assets".to_string());
    
    if chunks.len() > 20 {
        recommendations.push("High number of chunks - consider optimizing chunk splitting strategy".to_string());
    }
    
    // Add framework-specific recommendations
    let framework = detect_framework(build_dir);
    let limits = get_framework_limits(&framework);
    let framework_recommendations = generate_framework_recommendations(&framework, chunks, &limits);
    recommendations.extend(framework_recommendations);
    
    recommendations
}

fn has_oversized_chunks(report: &BundleReport) -> bool {
    report.chunks.iter().any(|chunk| chunk.size_bytes > 500_000)
}

fn print_report(report: &BundleReport, quiet: bool) {
    if !quiet {
        println!();
        println!("{}", "📊 Bundle Analysis Report".bold().blue());
        println!("{}", "========================".blue());
        println!();
    }
    
    if report.chunks.is_empty() {
        println!("{}", "⚠️ No bundle chunks found.".yellow());
        return;
    }
    
    // Sort chunks by size (largest first)
    let mut sorted_chunks = report.chunks.clone();
    sorted_chunks.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    
    // Group chunks by type
    let mut chunks_by_type: HashMap<String, Vec<&BundleChunk>> = HashMap::new();
    for chunk in &sorted_chunks {
        let type_name = format!("{:?}", chunk.chunk_type);
        chunks_by_type.entry(type_name).or_default().push(chunk);
    }
    
    // Print largest chunks first
    println!("{}", "📦 LARGEST CHUNKS".bold().white());
    println!("{}", "─────────────────".white());
    
    for (i, chunk) in sorted_chunks.iter().take(10).enumerate() {
        let size_kb = chunk.size_bytes / 1024;
        let size_color = if chunk.size_bytes > 500_000 {
            size_kb.to_string().red()
        } else if chunk.size_bytes > 200_000 {
            size_kb.to_string().yellow()
        } else {
            size_kb.to_string().green()
        };
        
        println!("  {}. {} - {} KB", (i + 1), chunk.name.cyan(), size_color);
        
        if let Some(compressed) = chunk.size_compressed {
            let compressed_kb = compressed / 1024;
            println!("     {} Compressed: {} KB", "💾".dimmed(), compressed_kb.to_string().dimmed());
        }
    }
    
    println!();
    
    // Print warnings
    if !report.summary.warnings.is_empty() {
        println!("{}", "⚠️  WARNINGS".bold().yellow());
        println!("{}", "───────────".yellow());
        for warning in &report.summary.warnings {
            println!("  • {}", warning.yellow());
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

fn print_summary(summary: &BundleSummary) {
    println!("{}", "📈 SUMMARY".bold().white());
    println!("{}", "─────────".white());
    
    let total_mb = summary.total_size as f64 / 1_000_000.0;
    let total_color = if summary.total_size > 2_000_000 {
        format!("{:.2} MB", total_mb).red()
    } else if summary.total_size > 1_000_000 {
        format!("{:.2} MB", total_mb).yellow()
    } else {
        format!("{:.2} MB", total_mb).green()
    };
    
    println!("  Total bundle size: {}", total_color);
    
    if summary.total_compressed > 0 {
        let compressed_mb = summary.total_compressed as f64 / 1_000_000.0;
        println!("  Compressed size: {:.2} MB", compressed_mb);
        println!("  Compression ratio: {:.1}%", (1.0 - summary.compression_ratio) * 100.0);
    }
    
    println!("  Number of chunks: {}", summary.chunk_count);
    
    if let Some(ref largest) = summary.largest_chunk {
        println!("  Largest chunk: {}", largest.cyan());
    }
    
    println!();
    
    // Performance tips
    if summary.total_size > 1_000_000 {
        println!("{}", "🚀 PERFORMANCE IMPACT".bold().red());
        println!("{}", "────────────────────".red());
        println!("  {} Large bundle size may impact loading performance", "⚠️".red());
        println!("  {} Consider implementing code splitting and lazy loading", "💡".yellow());
        println!();
    }
    
    println!("{}", "💡 TIP: Use tools like webpack-bundle-analyzer for detailed analysis".dimmed());
}

/// Detect the framework being used based on build output and package.json
fn detect_framework(build_dir: &Path) -> Framework {
    let project_root = build_dir.parent().unwrap_or(build_dir);
    
    // Check package.json for framework dependencies
    if let Ok(package_json) = fs::read_to_string(project_root.join("package.json")) {
        if package_json.contains("\"next\"") || package_json.contains("\"@next/") {
            return Framework::NextJs;
        }
        if package_json.contains("\"@angular/") {
            return Framework::Angular;
        }
        if package_json.contains("\"vue\"") && package_json.contains("\"@vue/") {
            return Framework::Vue;
        }
        if package_json.contains("\"svelte\"") || package_json.contains("\"@sveltejs/") {
            return Framework::Svelte;
        }
        if package_json.contains("\"vite\"") || package_json.contains("\"@vitejs/") {
            return Framework::Vite;
        }
        if package_json.contains("\"react\"") {
            return Framework::React;
        }
        if package_json.contains("\"webpack\"") {
            return Framework::Webpack;
        }
    }
    
    // Check build directory structure for framework-specific patterns
    if build_dir.join("_next").exists() || build_dir.join("server").exists() {
        return Framework::NextJs;
    }
    if build_dir.join("dist").join("index.html").exists() && 
       project_root.join("angular.json").exists() {
        return Framework::Angular;
    }
    if build_dir.join("assets").exists() && 
       (project_root.join("vite.config.js").exists() || project_root.join("vite.config.ts").exists()) {
        return Framework::Vite;
    }
    
    Framework::Unknown
}

/// Get framework-specific bundle size limits
fn get_framework_limits(framework: &Framework) -> FrameworkLimits {
    match framework {
        Framework::NextJs => FrameworkLimits {
            max_total_size_mb: 3.0,      // Next.js can handle larger bundles with SSR
            max_main_chunk_mb: 1.0,
            max_vendor_chunk_mb: 1.5,
            performance_budget_mb: 2.5,
        },
        Framework::React => FrameworkLimits {
            max_total_size_mb: 2.0,      // Standard SPA limits
            max_main_chunk_mb: 0.8,
            max_vendor_chunk_mb: 1.2,
            performance_budget_mb: 1.5,
        },
        Framework::Vue => FrameworkLimits {
            max_total_size_mb: 2.0,      // Similar to React
            max_main_chunk_mb: 0.8,
            max_vendor_chunk_mb: 1.2,
            performance_budget_mb: 1.5,
        },
        Framework::Angular => FrameworkLimits {
            max_total_size_mb: 4.0,      // Angular bundles tend to be larger
            max_main_chunk_mb: 1.5,
            max_vendor_chunk_mb: 2.0,
            performance_budget_mb: 3.0,
        },
        Framework::Svelte => FrameworkLimits {
            max_total_size_mb: 1.0,      // Svelte produces very small bundles
            max_main_chunk_mb: 0.4,
            max_vendor_chunk_mb: 0.6,
            performance_budget_mb: 0.8,
        },
        Framework::Vite => FrameworkLimits {
            max_total_size_mb: 2.0,      // Vite with modern bundling
            max_main_chunk_mb: 0.8,
            max_vendor_chunk_mb: 1.2,
            performance_budget_mb: 1.5,
        },
        Framework::Webpack => FrameworkLimits {
            max_total_size_mb: 2.5,      // Generic webpack setup
            max_main_chunk_mb: 1.0,
            max_vendor_chunk_mb: 1.5,
            performance_budget_mb: 2.0,
        },
        Framework::Unknown => FrameworkLimits {
            max_total_size_mb: 2.0,      // Conservative defaults
            max_main_chunk_mb: 0.8,
            max_vendor_chunk_mb: 1.2,
            performance_budget_mb: 1.5,
        },
    }
}

/// Generate framework-specific recommendations
fn generate_framework_recommendations(framework: &Framework, chunks: &[BundleChunk], limits: &FrameworkLimits) -> Vec<String> {
    let mut recommendations = Vec::new();
    let total_size_mb = chunks.iter().map(|c| c.size_bytes).sum::<u64>() as f64 / 1_000_000.0;
    
    // Framework-specific optimization tips
    match framework {
        Framework::NextJs => {
            recommendations.push("Use Next.js Image optimization for assets".to_string());
            recommendations.push("Enable compression in next.config.js".to_string());
            recommendations.push("Consider using Next.js dynamic imports for code splitting".to_string());
            if total_size_mb > limits.performance_budget_mb {
                recommendations.push("Use Next.js Bundle Analyzer: npm install @next/bundle-analyzer".to_string());
            }
        },
        Framework::React => {
            recommendations.push("Use React.lazy() for component-level code splitting".to_string());
            recommendations.push("Consider using React.memo() for expensive components".to_string());
            if total_size_mb > limits.performance_budget_mb {
                recommendations.push("Use webpack-bundle-analyzer to identify large dependencies".to_string());
            }
        },
        Framework::Vue => {
            recommendations.push("Use Vue's async components for code splitting".to_string());
            recommendations.push("Consider tree-shaking with ES modules".to_string());
            if total_size_mb > limits.performance_budget_mb {
                recommendations.push("Use Vue CLI Bundle Analyzer plugin".to_string());
            }
        },
        Framework::Angular => {
            recommendations.push("Use Angular's lazy loading for feature modules".to_string());
            recommendations.push("Enable Angular CLI's build optimizer".to_string());
            recommendations.push("Use OnPush change detection strategy".to_string());
            if total_size_mb > limits.performance_budget_mb {
                recommendations.push("Use Angular CLI bundle analyzer: ng build --stats-json".to_string());
            }
        },
        Framework::Svelte => {
            recommendations.push("Leverage Svelte's compile-time optimizations".to_string());
            recommendations.push("Use SvelteKit for automatic code splitting".to_string());
            if total_size_mb > limits.performance_budget_mb {
                recommendations.push("Check for unnecessary dependencies - Svelte apps should be very small".to_string());
            }
        },
        Framework::Vite => {
            recommendations.push("Use Vite's dynamic imports for code splitting".to_string());
            recommendations.push("Enable Vite's build optimizations".to_string());
            if total_size_mb > limits.performance_budget_mb {
                recommendations.push("Use vite-bundle-analyzer plugin".to_string());
            }
        },
        Framework::Webpack => {
            recommendations.push("Use webpack's SplitChunksPlugin for optimization".to_string());
            recommendations.push("Enable webpack's TerserPlugin for minification".to_string());
            if total_size_mb > limits.performance_budget_mb {
                recommendations.push("Use webpack-bundle-analyzer plugin".to_string());
            }
        },
        Framework::Unknown => {
            recommendations.push("Consider implementing code splitting".to_string());
            recommendations.push("Use tree-shaking to eliminate dead code".to_string());
        },
    }
    
    recommendations
}