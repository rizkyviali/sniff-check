use anyhow::{anyhow, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
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
    
    let report = analyze_bundle().await?;
    
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

async fn analyze_bundle() -> Result<BundleReport> {
    // Check if this is a Next.js project
    let current_dir = std::env::current_dir()?;
    
    // Look for Next.js build output
    let next_build_dir = current_dir.join(".next");
    if next_build_dir.exists() {
        analyze_nextjs_bundle(&next_build_dir).await
    } else {
        // Look for other common build outputs
        let potential_dirs = vec!["dist", "build", "out"];
        
        for dir_name in potential_dirs {
            let build_dir = current_dir.join(dir_name);
            if build_dir.exists() {
                return analyze_generic_bundle(&build_dir).await;
            }
        }
        
        Err(anyhow!("No build output found. Please run 'npm run build' or equivalent first."))
    }
}

async fn analyze_nextjs_bundle(next_dir: &Path) -> Result<BundleReport> {
    let mut chunks = Vec::new();
    let mut total_size = 0u64;
    let mut total_compressed = 0u64;
    
    // Analyze static chunks
    let static_dir = next_dir.join("static");
    if static_dir.exists() {
        chunks.extend(analyze_static_chunks(&static_dir)?);
    }
    
    // Analyze pages
    let pages_dir = next_dir.join("server").join("pages");
    if pages_dir.exists() {
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
    
    let compression_ratio = if total_size > 0 {
        (total_compressed as f64) / (total_size as f64)
    } else {
        1.0
    };
    
    let largest_chunk = chunks
        .iter()
        .max_by_key(|c| c.size_bytes)
        .map(|c| c.name.clone());
    
    let warnings = generate_warnings(&chunks);
    let recommendations = generate_recommendations(&chunks);
    
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

async fn analyze_generic_bundle(build_dir: &Path) -> Result<BundleReport> {
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
    
    let warnings = generate_warnings(&chunks);
    let recommendations = generate_recommendations(&chunks);
    
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

fn generate_warnings(chunks: &[BundleChunk]) -> Vec<String> {
    let mut warnings = Vec::new();
    
    // Check for oversized chunks
    for chunk in chunks {
        if chunk.size_bytes > 500_000 {
            warnings.push(format!("Large chunk detected: {} ({} KB)", 
                chunk.name, chunk.size_bytes / 1024));
        }
    }
    
    // Check total bundle size
    let total_size: u64 = chunks.iter().map(|c| c.size_bytes).sum();
    if total_size > 2_000_000 {
        warnings.push(format!("Total bundle size exceeds 2MB ({} MB)", 
            total_size / 1_000_000));
    }
    
    warnings
}

fn generate_recommendations(chunks: &[BundleChunk]) -> Vec<String> {
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