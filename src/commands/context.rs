use anyhow::Result;
use colored::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::utils::FileUtils;
use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextReport {
    pub project_info: ProjectInfo,
    pub structure: ProjectStructure,
    pub dependencies: DependencyAnalysis,
    pub architecture: ArchitectureInsights,
    pub relationships: FileRelationships,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub framework: Framework,
    pub languages: Vec<Language>,
    pub total_files: usize,
    pub total_lines: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub directories: Vec<DirectoryInfo>,
    pub components: Vec<ComponentInfo>,
    pub pages: Vec<PageInfo>,
    pub api_routes: Vec<ApiRouteInfo>,
    pub utilities: Vec<UtilityInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub package_json: Option<PackageJsonInfo>,
    pub imports: HashMap<String, Vec<ImportInfo>>,
    pub exports: HashMap<String, Vec<ExportInfo>>,
    pub external_dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchitectureInsights {
    pub patterns: Vec<ArchitecturePattern>,
    pub organization_score: f64,
    pub complexity_level: ComplexityLevel,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileRelationships {
    pub import_graph: HashMap<String, Vec<String>>,
    pub component_hierarchy: HashMap<String, Vec<String>>,
    pub most_imported: Vec<(String, usize)>,
    pub circular_dependencies: Vec<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryInfo {
    pub path: String,
    pub purpose: DirectoryPurpose,
    pub file_count: usize,
    pub line_count: usize,
    pub main_file_types: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub name: String,
    pub path: String,
    pub component_type: ComponentType,
    pub props_count: usize,
    pub hooks_used: Vec<String>,
    pub children_components: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageInfo {
    pub name: String,
    pub path: String,
    pub route: String,
    pub has_ssr: bool,
    pub has_ssg: bool,
    pub api_calls: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiRouteInfo {
    pub path: String,
    pub methods: Vec<String>,
    pub middleware: Vec<String>,
    pub database_operations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UtilityInfo {
    pub path: String,
    pub functions: Vec<String>,
    pub purpose: UtilityPurpose,
    pub complexity: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportInfo {
    pub from: String,
    pub imports: Vec<String>,
    pub import_type: ImportType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportInfo {
    pub name: String,
    pub export_type: ExportType,
    pub used_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageJsonInfo {
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub scripts: HashMap<String, String>,
    pub main_dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Framework {
    NextJs,
    React,
    Vue,
    Angular,
    Svelte,
    Vanilla,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub enum Language {
    TypeScript,
    JavaScript,
    CSS,
    SCSS,
    JSON,
    Markdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DirectoryPurpose {
    Components,
    Pages,
    Api,
    Utils,
    Services,
    Styles,
    Public,
    Config,
    Tests,
    Build,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ComponentType {
    Page,
    Layout,
    Feature,
    UI,
    Hook,
    Context,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UtilityPurpose {
    DataFetching,
    Validation,
    Formatting,
    Constants,
    Types,
    Helpers,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ImportType {
    Default,
    Named,
    Namespace,
    Dynamic,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExportType {
    Default,
    Named,
    Namespace,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ArchitecturePattern {
    LayeredArchitecture,
    ComponentComposition,
    CustomHooks,
    ContextProviders,
    ServiceLayer,
    UtilityFirst,
    ConfigDriven,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

pub async fn run(json: bool, quiet: bool) -> Result<()> {
    if !quiet {
        println!("{}", "🔍 Analyzing project structure and context...".bold().blue());
    }
    
    let report = analyze_project_context().await?;
    
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report, quiet);
    }
    
    Ok(())
}

async fn analyze_project_context() -> Result<ContextReport> {
    let current_dir = std::env::current_dir()?;
    
    // Analyze project info
    let project_info = analyze_project_info(&current_dir).await?;
    
    // Analyze project structure
    let structure = analyze_project_structure(&current_dir).await?;
    
    // Analyze dependencies
    let dependencies = analyze_dependencies(&current_dir).await?;
    
    // Generate architecture insights
    let architecture = generate_architecture_insights(&structure, &dependencies).await?;
    
    // Analyze file relationships
    let relationships = analyze_file_relationships(&current_dir).await?;
    
    Ok(ContextReport {
        project_info,
        structure,
        dependencies,
        architecture,
        relationships,
    })
}

async fn analyze_project_info(project_dir: &Path) -> Result<ProjectInfo> {
    let package_json_path = project_dir.join("package.json");
    let mut name = "Unknown".to_string();
    let mut version = None;
    let mut description = None;
    
    if package_json_path.exists() {
        let content = fs::read_to_string(&package_json_path)?;
        if let Ok(package_info) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(pkg_name) = package_info.get("name").and_then(|v| v.as_str()) {
                name = pkg_name.to_string();
            }
            if let Some(pkg_version) = package_info.get("version").and_then(|v| v.as_str()) {
                version = Some(pkg_version.to_string());
            }
            if let Some(pkg_desc) = package_info.get("description").and_then(|v| v.as_str()) {
                description = Some(pkg_desc.to_string());
            }
        }
    }
    
    let framework = detect_framework(project_dir).await?;
    let languages = detect_languages(project_dir).await?;
    let (total_files, total_lines) = count_files_and_lines(project_dir).await?;
    
    Ok(ProjectInfo {
        name,
        version,
        description,
        framework,
        languages,
        total_files,
        total_lines,
    })
}

async fn detect_framework(project_dir: &Path) -> Result<Framework> {
    let package_json_path = project_dir.join("package.json");
    
    if package_json_path.exists() {
        let content = fs::read_to_string(&package_json_path)?;
        if let Ok(package_info) = serde_json::from_str::<serde_json::Value>(&content) {
            let deps = package_info.get("dependencies").unwrap_or(&serde_json::Value::Null);
            let dev_deps = package_info.get("devDependencies").unwrap_or(&serde_json::Value::Null);
            
            if deps.get("next").is_some() || dev_deps.get("next").is_some() {
                return Ok(Framework::NextJs);
            }
            if deps.get("react").is_some() || dev_deps.get("react").is_some() {
                return Ok(Framework::React);
            }
            if deps.get("vue").is_some() || dev_deps.get("vue").is_some() {
                return Ok(Framework::Vue);
            }
            if deps.get("@angular/core").is_some() || dev_deps.get("@angular/core").is_some() {
                return Ok(Framework::Angular);
            }
            if deps.get("svelte").is_some() || dev_deps.get("svelte").is_some() {
                return Ok(Framework::Svelte);
            }
        }
    }
    
    // Check for Next.js specific files
    if project_dir.join("next.config.js").exists() || project_dir.join("next.config.ts").exists() {
        return Ok(Framework::NextJs);
    }
    
    // Check for framework-specific directories
    if project_dir.join("pages").exists() || project_dir.join("app").exists() {
        return Ok(Framework::NextJs);
    }
    
    Ok(Framework::Unknown)
}

async fn detect_languages(project_dir: &Path) -> Result<Vec<Language>> {
    let mut languages = HashSet::new();
    let extensions = ["ts", "tsx", "js", "jsx", "css", "scss", "json", "md"];
    
    let files = FileUtils::find_files_with_extensions(project_dir, &extensions);
    
    for file in files {
        if let Some(ext) = file.extension() {
            let ext_str = ext.to_string_lossy();
            match ext_str.as_ref() {
                "ts" | "tsx" => { languages.insert(Language::TypeScript); }
                "js" | "jsx" => { languages.insert(Language::JavaScript); }
                "css" => { languages.insert(Language::CSS); }
                "scss" => { languages.insert(Language::SCSS); }
                "json" => { languages.insert(Language::JSON); }
                "md" => { languages.insert(Language::Markdown); }
                _ => {}
            }
        }
    }
    
    Ok(languages.into_iter().collect())
}

async fn analyze_project_structure(project_dir: &Path) -> Result<ProjectStructure> {
    let directories = analyze_directories(project_dir).await?;
    let components = analyze_components(project_dir).await?;
    let pages = analyze_pages(project_dir).await?;
    let api_routes = analyze_api_routes(project_dir).await?;
    let utilities = analyze_utilities(project_dir).await?;
    
    Ok(ProjectStructure {
        directories,
        components,
        pages,
        api_routes,
        utilities,
    })
}

async fn analyze_directories(project_dir: &Path) -> Result<Vec<DirectoryInfo>> {
    let mut directories = Vec::new();
    
    for entry in WalkDir::new(project_dir).max_depth(3) {
        let entry = entry?;
        if entry.file_type().is_dir() && !FileUtils::is_node_modules(entry.path()) {
            let path = entry.path();
            let relative_path = path.strip_prefix(project_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
            
            if relative_path.is_empty() || relative_path == "." {
                continue;
            }
            
            let purpose = determine_directory_purpose(&relative_path);
            let (file_count, line_count, main_file_types) = count_directory_contents(path).await?;
            
            directories.push(DirectoryInfo {
                path: relative_path,
                purpose,
                file_count,
                line_count,
                main_file_types,
            });
        }
    }
    
    Ok(directories)
}

async fn count_files_and_lines(project_dir: &Path) -> Result<(usize, usize)> {
    let extensions = vec!["ts", "tsx", "js", "jsx"];
    let files = FileUtils::find_files_with_extensions(project_dir, &extensions);
    let total_files = files.len();
    
    let total_lines: usize = FileUtils::process_files_parallel(
        &files,
        |path| FileUtils::count_lines_optimized(path),
        "Counting lines",
        true // quiet mode for sub-operation
    )?
    .into_iter()
    .sum();
    
    Ok((total_files, total_lines))
}

async fn count_directory_contents(dir_path: &Path) -> Result<(usize, usize, Vec<String>)> {
    let mut file_count = 0;
    let mut line_count = 0;
    let mut extensions: HashMap<String, usize> = HashMap::new();
    
    for entry in WalkDir::new(dir_path).max_depth(1) {
        let entry = entry?;
        if entry.file_type().is_file() {
            file_count += 1;
            
            if let Some(ext) = entry.path().extension() {
                let ext_str = ext.to_string_lossy().to_string();
                *extensions.entry(ext_str).or_insert(0) += 1;
                
                // Count lines for code files
                if ["ts", "tsx", "js", "jsx", "css", "scss"].contains(&ext.to_string_lossy().as_ref()) {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        line_count += content.lines().count();
                    }
                }
            }
        }
    }
    
    let mut main_file_types: Vec<(String, usize)> = extensions.into_iter().collect();
    main_file_types.sort_by(|a, b| b.1.cmp(&a.1));
    let main_file_types: Vec<String> = main_file_types
        .into_iter()
        .take(3)
        .map(|(ext, _)| ext)
        .collect();
    
    Ok((file_count, line_count, main_file_types))
}

fn determine_directory_purpose(path: &str) -> DirectoryPurpose {
    let path_lower = path.to_lowercase();
    
    if path_lower.contains("component") {
        DirectoryPurpose::Components
    } else if path_lower.contains("page") || path_lower == "app" {
        DirectoryPurpose::Pages
    } else if path_lower.contains("api") {
        DirectoryPurpose::Api
    } else if path_lower.contains("util") || path_lower.contains("helper") {
        DirectoryPurpose::Utils
    } else if path_lower.contains("service") || path_lower.contains("lib") {
        DirectoryPurpose::Services
    } else if path_lower.contains("style") || path_lower.contains("css") {
        DirectoryPurpose::Styles
    } else if path_lower.contains("public") || path_lower.contains("static") {
        DirectoryPurpose::Public
    } else if path_lower.contains("config") {
        DirectoryPurpose::Config
    } else if path_lower.contains("test") || path_lower.contains("spec") {
        DirectoryPurpose::Tests
    } else if path_lower.contains("build") || path_lower.contains("dist") || path_lower.contains(".next") {
        DirectoryPurpose::Build
    } else {
        DirectoryPurpose::Other
    }
}

// Placeholder implementations for remaining analysis functions
async fn analyze_components(_project_dir: &Path) -> Result<Vec<ComponentInfo>> {
    Ok(Vec::new()) // TODO: Implement component analysis
}

async fn analyze_pages(_project_dir: &Path) -> Result<Vec<PageInfo>> {
    Ok(Vec::new()) // TODO: Implement page analysis
}

async fn analyze_api_routes(_project_dir: &Path) -> Result<Vec<ApiRouteInfo>> {
    Ok(Vec::new()) // TODO: Implement API route analysis
}

async fn analyze_utilities(_project_dir: &Path) -> Result<Vec<UtilityInfo>> {
    Ok(Vec::new()) // TODO: Implement utility analysis
}

async fn analyze_dependencies(_project_dir: &Path) -> Result<DependencyAnalysis> {
    Ok(DependencyAnalysis {
        package_json: None,
        imports: HashMap::new(),
        exports: HashMap::new(),
        external_dependencies: Vec::new(),
    }) // TODO: Implement dependency analysis
}

async fn generate_architecture_insights(_structure: &ProjectStructure, _dependencies: &DependencyAnalysis) -> Result<ArchitectureInsights> {
    Ok(ArchitectureInsights {
        patterns: Vec::new(),
        organization_score: 0.0,
        complexity_level: ComplexityLevel::Simple,
        recommendations: Vec::new(),
    }) // TODO: Implement architecture insights
}

async fn analyze_file_relationships(_project_dir: &Path) -> Result<FileRelationships> {
    Ok(FileRelationships {
        import_graph: HashMap::new(),
        component_hierarchy: HashMap::new(),
        most_imported: Vec::new(),
        circular_dependencies: Vec::new(),
    }) // TODO: Implement relationship analysis
}


fn print_report(report: &ContextReport, quiet: bool) {
    if !quiet {
        println!();
        println!("{}", "📊 Project Context Report".bold().blue());
        println!("{}", "========================".blue());
        println!();
    }
    
    print_project_info(&report.project_info);
    print_project_structure(&report.structure);
    print_architecture_insights(&report.architecture);
}

fn print_project_info(info: &ProjectInfo) {
    println!("{}", "🏗️  PROJECT OVERVIEW".bold().green());
    println!("{}", "─────────────────────".green());
    println!("  Name: {}", info.name.bold());
    
    if let Some(version) = &info.version {
        println!("  Version: {}", version);
    }
    
    if let Some(description) = &info.description {
        println!("  Description: {}", description.dimmed());
    }
    
    println!("  Framework: {:?}", info.framework);
    println!("  Languages: {:?}", info.languages);
    println!("  Total Files: {}", info.total_files);
    println!("  Total Lines: {}", info.total_lines);
    println!();
}

fn print_project_structure(structure: &ProjectStructure) {
    println!("{}", "📁 PROJECT STRUCTURE".bold().cyan());
    println!("{}", "──────────────────────".cyan());
    
    if !structure.directories.is_empty() {
        println!("  Key Directories:");
        for dir in &structure.directories {
            let purpose_str = format!("{:?}", dir.purpose);
            println!("    {} {} ({} files, {} lines)", 
                "•".cyan(), 
                dir.path.bold(), 
                dir.file_count, 
                dir.line_count
            );
            println!("      Purpose: {} | File types: {}", 
                purpose_str.dimmed(),
                dir.main_file_types.join(", ").dimmed()
            );
        }
    }
    
    println!();
}

fn print_architecture_insights(insights: &ArchitectureInsights) {
    println!("{}", "🏛️  ARCHITECTURE INSIGHTS".bold().yellow());
    println!("{}", "─────────────────────────".yellow());
    println!("  Organization Score: {:.1}%", insights.organization_score);
    println!("  Complexity Level: {:?}", insights.complexity_level);
    
    if !insights.patterns.is_empty() {
        println!("  Detected Patterns:");
        for pattern in &insights.patterns {
            println!("    • {:?}", pattern);
        }
    }
    
    if !insights.recommendations.is_empty() {
        println!("  Recommendations:");
        for rec in &insights.recommendations {
            println!("    💡 {}", rec.dimmed());
        }
    }
    
    println!();
}