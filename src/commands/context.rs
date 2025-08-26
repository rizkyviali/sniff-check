use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use crate::utils::FileUtils;
use crate::common::OptimizedFileWalker;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
    
    let report = analyze_project_context(quiet).await?;
    
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report, quiet);
    }
    
    Ok(())
}

async fn analyze_project_context(quiet: bool) -> Result<ContextReport> {
    let current_dir = std::env::current_dir()?;
    
    // Analyze project info
    if !quiet {
        println!("📁 Analyzing project information...");
    }
    let project_info = analyze_project_info(&current_dir).await?;
    
    // Analyze project structure
    if !quiet {
        println!("🏢 Analyzing project structure...");
    }
    let structure = analyze_project_structure(&current_dir).await?;
    
    // Analyze dependencies
    if !quiet {
        println!("📦 Analyzing dependencies...");
    }
    let dependencies = analyze_dependencies(&current_dir).await?;
    
    // Generate architecture insights
    if !quiet {
        println!("🎨 Generating architecture insights...");
    }
    let architecture = generate_architecture_insights(&structure, &dependencies).await?;
    
    // Analyze file relationships
    if !quiet {
        println!("🔗 Analyzing file relationships...");
    }
    let relationships = analyze_file_relationships(&current_dir).await?;
    
    if !quiet {
        println!("✅ Context analysis completed");
    }
    
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
    
    let files = FileUtils::find_files_with_progress(project_dir, &extensions, true)?;
    
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
    let files = FileUtils::find_files_with_progress(project_dir, &extensions, true)?;
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

async fn analyze_components(project_dir: &Path) -> Result<Vec<ComponentInfo>> {
    let mut components = Vec::new();
    let extensions = vec!["tsx", "jsx"];
    let files = FileUtils::find_files_with_progress(project_dir, &extensions, true)?;
    
    for file in files {
        if let Ok(content) = fs::read_to_string(&file) {
            if is_component_file(&content) {
                let component_info = analyze_component_file(&file, &content)?;
                components.push(component_info);
            }
        }
    }
    
    Ok(components)
}

fn is_component_file(content: &str) -> bool {
    // Check for React component patterns
    content.contains("export default") && (
        content.contains("function ") ||
        content.contains("const ") ||
        content.contains("export function")
    ) && (
        content.contains("return (") ||
        content.contains("return <") ||
        content.contains("jsx") ||
        content.contains("tsx")
    )
}

fn analyze_component_file(path: &Path, content: &str) -> Result<ComponentInfo> {
    let name = path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    let relative_path = FileUtils::get_relative_path(path);
    
    let component_type = if relative_path.contains("/pages/") {
        ComponentType::Page
    } else if name.to_lowercase().contains("layout") {
        ComponentType::Layout
    } else if relative_path.contains("/hooks/") || name.starts_with("use") {
        ComponentType::Hook
    } else if name.to_lowercase().contains("context") {
        ComponentType::Context
    } else if relative_path.contains("/components/ui/") {
        ComponentType::UI
    } else {
        ComponentType::Feature
    };
    
    // Count props interfaces/types
    let props_count = count_props_definitions(content);
    
    // Extract hooks used
    let hooks_used = extract_hooks_used(content);
    
    // Find child components (simplified)
    let children_components = extract_child_components(content);
    
    Ok(ComponentInfo {
        name,
        path: relative_path,
        component_type,
        props_count,
        hooks_used,
        children_components,
    })
}

fn count_props_definitions(content: &str) -> usize {
    content.lines().filter(|line| {
        let line = line.trim();
        line.contains("Props") && (line.contains("interface") || line.contains("type"))
    }).count()
}

fn extract_hooks_used(content: &str) -> Vec<String> {
    let mut hooks = Vec::new();
    let hook_patterns = ["useState", "useEffect", "useContext", "useReducer", "useMemo", "useCallback", "useRef"];
    
    for pattern in hook_patterns {
        if content.contains(pattern) {
            hooks.push(pattern.to_string());
        }
    }
    
    // Extract custom hooks (starting with 'use')
    for line in content.lines() {
        if let Some(start) = line.find("use") {
            let rest = &line[start..];
            if let Some(end) = rest.find('(') {
                let hook_name = &rest[..end];
                if hook_name.len() > 3 && hook_name.chars().nth(3).unwrap_or(' ').is_uppercase() {
                    hooks.push(hook_name.to_string());
                }
            }
        }
    }
    
    hooks.sort();
    hooks.dedup();
    hooks
}

fn extract_child_components(content: &str) -> Vec<String> {
    let mut components = Vec::new();
    
    for line in content.lines() {
        // Look for JSX component usage like <ComponentName
        if line.contains('<') {
            let parts: Vec<&str> = line.split('<').collect();
            for part in parts.iter().skip(1) {
                if let Some(space_pos) = part.find(|c: char| c.is_whitespace() || c == '>' || c == '/') {
                    let component_name = &part[..space_pos];
                    if component_name.len() > 0 && component_name.chars().next().unwrap().is_uppercase() {
                        components.push(component_name.to_string());
                    }
                }
            }
        }
    }
    
    components.sort();
    components.dedup();
    components.truncate(10); // Limit to top 10
    components
}

async fn analyze_pages(project_dir: &Path) -> Result<Vec<PageInfo>> {
    let mut pages = Vec::new();
    
    // Common page directories for different frameworks
    let page_dirs = [
        "pages",           // Next.js pages directory
        "app",             // Next.js app directory
        "src/pages",       // Next.js with src
        "src/app",         // Next.js app with src
        "src/routes",      // SvelteKit routes
        "routes",          // SvelteKit routes
    ];
    
    for dir_name in &page_dirs {
        let dir_path = project_dir.join(dir_name);
        if dir_path.exists() && dir_path.is_dir() {
            let walker = OptimizedFileWalker::new()
                .parallel_threshold(10);
            
            let files = walker.walk(&dir_path)
                .into_iter()
                .filter(|path| {
                    if let Some(ext) = path.extension() {
                        matches!(ext.to_str(), Some("ts") | Some("tsx") | Some("js") | Some("jsx"))
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>();
            
            for file_path in files {
                if let Ok(content) = std::fs::read_to_string(&file_path) {
                    let relative_path = file_path.strip_prefix(project_dir)
                        .unwrap_or(&file_path)
                        .to_string_lossy()
                        .to_string();
                    
                    let _line_count = content.lines().count();
                    let _is_dynamic = relative_path.contains('[') && relative_path.contains(']');
                    let has_getstaticprops = content.contains("getStaticProps");
                    let has_getserversideprops = content.contains("getServerSideProps");
                    
                    let name = file_path.file_stem()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let route = relative_path.replace(".tsx", "").replace(".ts", "").replace(".jsx", "").replace(".js", "");
                    
                    pages.push(PageInfo {
                        name,
                        path: relative_path,
                        route,
                        has_ssr: has_getserversideprops,
                        has_ssg: has_getstaticprops,
                        api_calls: Vec::new(), // Could be enhanced to detect API calls
                    });
                }
            }
        }
    }
    
    Ok(pages)
}

async fn analyze_api_routes(project_dir: &Path) -> Result<Vec<ApiRouteInfo>> {
    let mut api_routes = Vec::new();
    
    // Common API directories
    let api_dirs = [
        "pages/api",
        "src/pages/api", 
        "app/api",
        "src/app/api",
        "api",
        "src/api",
    ];
    
    for dir_name in &api_dirs {
        let dir_path = project_dir.join(dir_name);
        if dir_path.exists() && dir_path.is_dir() {
            let walker = OptimizedFileWalker::new()
                .parallel_threshold(10);
            
            let files = walker.walk(&dir_path)
                .into_iter()
                .filter(|path| {
                    if let Some(ext) = path.extension() {
                        matches!(ext.to_str(), Some("ts") | Some("tsx") | Some("js") | Some("jsx"))
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>();
            
            for file_path in files {
                if let Ok(content) = std::fs::read_to_string(&file_path) {
                    let relative_path = file_path.strip_prefix(project_dir)
                        .unwrap_or(&file_path)
                        .to_string_lossy()
                        .to_string();
                    
                    let _line_count = content.lines().count();
                    
                    // Detect HTTP methods
                    let mut methods = Vec::new();
                    if content.contains("req.method === 'GET'") || content.contains("method: 'GET'") {
                        methods.push("GET".to_string());
                    }
                    if content.contains("req.method === 'POST'") || content.contains("method: 'POST'") {
                        methods.push("POST".to_string());
                    }
                    if content.contains("req.method === 'PUT'") || content.contains("method: 'PUT'") {
                        methods.push("PUT".to_string());
                    }
                    if content.contains("req.method === 'DELETE'") || content.contains("method: 'DELETE'") {
                        methods.push("DELETE".to_string());
                    }
                    
                    let has_middleware = content.contains("middleware") || content.contains("cors") || content.contains("auth");
                    let has_validation = content.contains("validate") || content.contains("schema") || content.contains("joi") || content.contains("yup");
                    
                    api_routes.push(ApiRouteInfo {
                        path: relative_path,
                        methods: if methods.is_empty() { vec!["GET".to_string()] } else { methods },
                        middleware: if has_middleware { vec!["middleware".to_string()] } else { Vec::new() },
                        database_operations: if has_validation { vec!["validation".to_string()] } else { Vec::new() },
                    });
                }
            }
        }
    }
    
    Ok(api_routes)
}

async fn analyze_utilities(project_dir: &Path) -> Result<Vec<UtilityInfo>> {
    let mut utilities = Vec::new();
    
    // Common utility directories
    let util_dirs = [
        "utils",
        "src/utils",
        "lib", 
        "src/lib",
        "helpers",
        "src/helpers",
        "common",
        "src/common",
    ];
    
    for dir_name in &util_dirs {
        let dir_path = project_dir.join(dir_name);
        if dir_path.exists() && dir_path.is_dir() {
            let walker = OptimizedFileWalker::new()
                .parallel_threshold(10);
            
            let files = walker.walk(&dir_path)
                .into_iter()
                .filter(|path| {
                    if let Some(ext) = path.extension() {
                        matches!(ext.to_str(), Some("ts") | Some("tsx") | Some("js") | Some("jsx"))
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>();
            
            for file_path in files {
                if let Ok(content) = std::fs::read_to_string(&file_path) {
                    let relative_path = file_path.strip_prefix(project_dir)
                        .unwrap_or(&file_path)
                        .to_string_lossy()
                        .to_string();
                    
                    let _line_count = content.lines().count();
                    
                    // Determine utility type based on content patterns
                    let utility_type = if content.contains("fetch") || content.contains("axios") || content.contains("http") {
                        "api".to_string()
                    } else if content.contains("format") || content.contains("parse") || content.contains("Date") {
                        "formatting".to_string()
                    } else if content.contains("validate") || content.contains("regex") || content.contains("test") {
                        "validation".to_string()
                    } else if content.contains("typeof") || content.contains("Array.isArray") {
                        "type-checking".to_string()
                    } else if content.contains("localStorage") || content.contains("sessionStorage") {
                        "storage".to_string()
                    } else {
                        "general".to_string()
                    };
                    
                    // Count exports
                    let export_count = content.matches("export ").count() + 
                                     content.matches("export{").count() +
                                     content.matches("export {").count();
                    
                    let _has_tests = project_dir.join(format!("{}.test.ts", file_path.file_stem().unwrap().to_string_lossy())).exists() ||
                                  project_dir.join(format!("{}.spec.ts", file_path.file_stem().unwrap().to_string_lossy())).exists();
                    
                    let purpose = match utility_type.as_str() {
                        "api" => UtilityPurpose::DataFetching,
                        "formatting" => UtilityPurpose::Formatting,
                        "validation" => UtilityPurpose::Validation,
                        "type-checking" => UtilityPurpose::Types,
                        "storage" => UtilityPurpose::Helpers,
                        _ => UtilityPurpose::Other,
                    };
                    
                    utilities.push(UtilityInfo {
                        path: relative_path,
                        functions: vec![format!("exports: {}", export_count)],
                        purpose,
                        complexity: _line_count.min(100), // Cap complexity at 100
                    });
                }
            }
        }
    }
    
    Ok(utilities)
}

async fn analyze_dependencies(project_dir: &Path) -> Result<DependencyAnalysis> {
    let package_json = analyze_package_json(project_dir).await?;
    let (imports, exports) = analyze_imports_exports(project_dir).await?;
    let external_dependencies = extract_external_dependencies(&imports);
    
    Ok(DependencyAnalysis {
        package_json,
        imports,
        exports,
        external_dependencies,
    })
}

async fn analyze_package_json(project_dir: &Path) -> Result<Option<PackageJsonInfo>> {
    let package_json_path = project_dir.join("package.json");
    
    if !package_json_path.exists() {
        return Ok(None);
    }
    
    let content = fs::read_to_string(&package_json_path)?;
    let package_info: serde_json::Value = serde_json::from_str(&content)?;
    
    let dependencies = extract_dependencies(&package_info, "dependencies");
    let dev_dependencies = extract_dependencies(&package_info, "devDependencies");
    let scripts = extract_scripts(&package_info);
    
    let main_dependencies = identify_main_dependencies(&dependencies);
    
    Ok(Some(PackageJsonInfo {
        dependencies,
        dev_dependencies,
        scripts,
        main_dependencies,
    }))
}

fn extract_dependencies(package_info: &serde_json::Value, key: &str) -> HashMap<String, String> {
    package_info
        .get(key)
        .and_then(|deps| deps.as_object())
        .map(|deps| {
            deps.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn extract_scripts(package_info: &serde_json::Value) -> HashMap<String, String> {
    package_info
        .get("scripts")
        .and_then(|scripts| scripts.as_object())
        .map(|scripts| {
            scripts.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn identify_main_dependencies(dependencies: &HashMap<String, String>) -> Vec<String> {
    let main_frameworks = ["react", "next", "vue", "angular", "svelte"];
    let main_tools = ["typescript", "webpack", "vite", "tailwindcss", "prisma"];
    
    let mut main_deps = Vec::new();
    
    for dep in dependencies.keys() {
        if main_frameworks.iter().any(|&framework| dep.contains(framework)) ||
           main_tools.iter().any(|&tool| dep.contains(tool)) {
            main_deps.push(dep.clone());
        }
    }
    
    main_deps.sort();
    main_deps
}

async fn analyze_imports_exports(project_dir: &Path) -> Result<(HashMap<String, Vec<ImportInfo>>, HashMap<String, Vec<ExportInfo>>)> {
    let mut imports: HashMap<String, Vec<ImportInfo>> = HashMap::new();
    let mut exports: HashMap<String, Vec<ExportInfo>> = HashMap::new();
    
    let extensions = vec!["ts", "tsx", "js", "jsx"];
    let files = FileUtils::find_files_with_progress(project_dir, &extensions, true)?;
    
    for file in files.iter().take(50) { // Limit to first 50 files for performance
        if let Ok(content) = fs::read_to_string(file) {
            let relative_path = FileUtils::get_relative_path(file);
            
            let file_imports = extract_imports_from_content(&content);
            if !file_imports.is_empty() {
                imports.insert(relative_path.clone(), file_imports);
            }
            
            let file_exports = extract_exports_from_content(&content);
            if !file_exports.is_empty() {
                exports.insert(relative_path, file_exports);
            }
        }
    }
    
    Ok((imports, exports))
}

fn extract_imports_from_content(content: &str) -> Vec<ImportInfo> {
    let mut imports = Vec::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("import ") && line.contains("from ") {
            if let Some(from_pos) = line.rfind("from ") {
                let import_part = &line[6..from_pos].trim();
                let from_part = &line[from_pos + 5..].trim().trim_matches('"').trim_matches('\'').trim_matches(';');
                
                let import_type = if import_part.starts_with('{') {
                    ImportType::Named
                } else if import_part.contains('*') {
                    ImportType::Namespace
                } else {
                    ImportType::Default
                };
                
                let import_names = if import_type == ImportType::Named {
                    import_part.trim_matches('{').trim_matches('}').split(',').map(|s| s.trim().to_string()).collect()
                } else {
                    vec![import_part.to_string()]
                };
                
                imports.push(ImportInfo {
                    from: from_part.to_string(),
                    imports: import_names,
                    import_type,
                });
            }
        }
    }
    
    imports
}

fn extract_exports_from_content(content: &str) -> Vec<ExportInfo> {
    let mut exports = Vec::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("export ") {
            if line.contains("export default") {
                exports.push(ExportInfo {
                    name: "default".to_string(),
                    export_type: ExportType::Default,
                    used_by: Vec::new(),
                });
            } else if line.contains("export {") {
                // Named exports
                if let Some(start) = line.find('{') {
                    if let Some(end) = line.find('}') {
                        let names = &line[start+1..end];
                        for name in names.split(',') {
                            exports.push(ExportInfo {
                                name: name.trim().to_string(),
                                export_type: ExportType::Named,
                                used_by: Vec::new(),
                            });
                        }
                    }
                }
            }
        }
    }
    
    exports
}

fn extract_external_dependencies(imports: &HashMap<String, Vec<ImportInfo>>) -> Vec<String> {
    let mut external_deps = HashSet::new();
    
    for import_list in imports.values() {
        for import in import_list {
            let from = &import.from;
            // External dependencies typically don't start with './' or '../'
            if !from.starts_with('.') && !from.starts_with('/') {
                // Extract the package name (first part before '/')
                let package_name = from.split('/').next().unwrap_or(from);
                external_deps.insert(package_name.to_string());
            }
        }
    }
    
    let mut deps: Vec<String> = external_deps.into_iter().collect();
    deps.sort();
    deps
}

async fn generate_architecture_insights(structure: &ProjectStructure, dependencies: &DependencyAnalysis) -> Result<ArchitectureInsights> {
    let patterns = detect_architecture_patterns(structure, dependencies);
    let organization_score = calculate_organization_score(structure);
    let complexity_level = determine_complexity_level(structure, dependencies);
    let recommendations = generate_recommendations(structure, dependencies, organization_score);
    
    Ok(ArchitectureInsights {
        patterns,
        organization_score,
        complexity_level,
        recommendations,
    })
}

fn detect_architecture_patterns(structure: &ProjectStructure, dependencies: &DependencyAnalysis) -> Vec<ArchitecturePattern> {
    let mut patterns = Vec::new();
    
    // Check for layered architecture
    let has_layers = structure.directories.iter().any(|d| {
        matches!(d.purpose, DirectoryPurpose::Services | DirectoryPurpose::Utils | DirectoryPurpose::Api)
    });
    if has_layers {
        patterns.push(ArchitecturePattern::LayeredArchitecture);
    }
    
    // Check for component composition
    if !structure.components.is_empty() {
        let has_composition = structure.components.iter().any(|c| !c.children_components.is_empty());
        if has_composition {
            patterns.push(ArchitecturePattern::ComponentComposition);
        }
    }
    
    // Check for custom hooks usage
    let has_custom_hooks = structure.components.iter().any(|c| {
        c.hooks_used.iter().any(|h| h.starts_with("use") && h.len() > 3)
    });
    if has_custom_hooks {
        patterns.push(ArchitecturePattern::CustomHooks);
    }
    
    // Check for context providers
    let has_context = structure.components.iter().any(|c| {
        matches!(c.component_type, ComponentType::Context) || 
        c.hooks_used.contains(&"useContext".to_string())
    });
    if has_context {
        patterns.push(ArchitecturePattern::ContextProviders);
    }
    
    // Check for service layer
    let has_services = structure.directories.iter().any(|d| {
        matches!(d.purpose, DirectoryPurpose::Services)
    });
    if has_services {
        patterns.push(ArchitecturePattern::ServiceLayer);
    }
    
    // Check for utility-first approach
    let has_utils = structure.directories.iter().any(|d| {
        matches!(d.purpose, DirectoryPurpose::Utils) && d.file_count > 3
    });
    if has_utils {
        patterns.push(ArchitecturePattern::UtilityFirst);
    }
    
    // Check for config-driven approach
    if let Some(package_info) = &dependencies.package_json {
        if package_info.scripts.len() > 5 {
            patterns.push(ArchitecturePattern::ConfigDriven);
        }
    }
    
    patterns
}

fn calculate_organization_score(structure: &ProjectStructure) -> f64 {
    let mut score = 0.0;
    let mut max_score = 0.0;
    
    // Directory structure score (30%)
    max_score += 30.0;
    let essential_dirs = [DirectoryPurpose::Components, DirectoryPurpose::Pages, DirectoryPurpose::Utils];
    let present_dirs = essential_dirs.iter().filter(|&purpose| {
        structure.directories.iter().any(|d| std::mem::discriminant(&d.purpose) == std::mem::discriminant(purpose))
    }).count();
    score += (present_dirs as f64 / essential_dirs.len() as f64) * 30.0;
    
    // Component organization score (25%)
    max_score += 25.0;
    if !structure.components.is_empty() {
        let components_with_types = structure.components.iter().filter(|c| {
            !matches!(c.component_type, ComponentType::Feature) // Feature is generic
        }).count();
        score += (components_with_types as f64 / structure.components.len() as f64) * 25.0;
    }
    
    // File distribution score (20%)
    max_score += 20.0;
    let total_files: usize = structure.directories.iter().map(|d| d.file_count).sum();
    if total_files > 0 {
        let largest_dir_files = structure.directories.iter().map(|d| d.file_count).max().unwrap_or(0);
        let balance_score = 1.0 - (largest_dir_files as f64 / total_files as f64);
        score += balance_score * 20.0;
    }
    
    // Naming consistency score (15%)
    max_score += 15.0;
    let consistent_naming = structure.directories.iter().filter(|d| {
        !matches!(d.purpose, DirectoryPurpose::Other)
    }).count();
    if !structure.directories.is_empty() {
        score += (consistent_naming as f64 / structure.directories.len() as f64) * 15.0;
    }
    
    // Complexity management score (10%)
    max_score += 10.0;
    let reasonable_complexity = structure.directories.iter().filter(|d| {
        d.file_count < 20 // Reasonable number of files per directory
    }).count();
    if !structure.directories.is_empty() {
        score += (reasonable_complexity as f64 / structure.directories.len() as f64) * 10.0;
    }
    
    if max_score > 0.0 {
        (score / max_score) * 100.0
    } else {
        0.0
    }
}

fn determine_complexity_level(structure: &ProjectStructure, dependencies: &DependencyAnalysis) -> ComplexityLevel {
    let total_files: usize = structure.directories.iter().map(|d| d.file_count).sum();
    let total_components = structure.components.len();
    let external_deps = dependencies.external_dependencies.len();
    
    let complexity_score = total_files + (total_components * 2) + external_deps;
    
    match complexity_score {
        0..=20 => ComplexityLevel::Simple,
        21..=50 => ComplexityLevel::Moderate,
        51..=100 => ComplexityLevel::Complex,
        _ => ComplexityLevel::VeryComplex,
    }
}

fn generate_recommendations(structure: &ProjectStructure, dependencies: &DependencyAnalysis, org_score: f64) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    // Organization recommendations
    if org_score < 60.0 {
        recommendations.push("🏗️ Consider reorganizing project structure for better maintainability".to_string());
    }
    
    // Directory recommendations
    let has_components = structure.directories.iter().any(|d| matches!(d.purpose, DirectoryPurpose::Components));
    if !has_components && !structure.components.is_empty() {
        recommendations.push("📁 Create a dedicated 'components' directory to organize React components".to_string());
    }
    
    let has_utils = structure.directories.iter().any(|d| matches!(d.purpose, DirectoryPurpose::Utils));
    if !has_utils {
        recommendations.push("🛠️ Consider creating a 'utils' directory for shared utility functions".to_string());
    }
    
    // Component recommendations
    let large_components = structure.components.iter().filter(|c| c.children_components.len() > 8).count();
    if large_components > 0 {
        recommendations.push("🧩 Some components have many children - consider breaking them into smaller pieces".to_string());
    }
    
    let components_without_types = structure.components.iter().filter(|c| c.props_count == 0).count();
    if components_without_types > structure.components.len() / 2 {
        recommendations.push("📝 Consider adding TypeScript prop interfaces for better type safety".to_string());
    }
    
    // Dependency recommendations
    if dependencies.external_dependencies.len() > 30 {
        recommendations.push("📦 High number of external dependencies - consider auditing for unused packages".to_string());
    }
    
    if let Some(package_info) = &dependencies.package_json {
        if package_info.dev_dependencies.len() > package_info.dependencies.len() * 2 {
            recommendations.push("🔧 Many dev dependencies - ensure they're all necessary for development".to_string());
        }
    }
    
    // File size recommendations
    let large_dirs = structure.directories.iter().filter(|d| d.file_count > 15).count();
    if large_dirs > 0 {
        recommendations.push("📂 Some directories contain many files - consider creating subdirectories".to_string());
    }
    
    recommendations.truncate(6); // Limit to top 6 recommendations
    recommendations
}

async fn analyze_file_relationships(project_dir: &Path) -> Result<FileRelationships> {
    let mut import_graph: HashMap<String, Vec<String>> = HashMap::new();
    let mut component_hierarchy: HashMap<String, Vec<String>> = HashMap::new();
    let mut import_counts: HashMap<String, usize> = HashMap::new();
    
    // Find all TypeScript/JavaScript files
    let walker = OptimizedFileWalker::new()
        .parallel_threshold(20);
    let files = walker.walk(project_dir)
        .into_iter()
        .filter(|path| {
            if let Some(ext) = path.extension() {
                matches!(ext.to_str(), Some("ts") | Some("tsx") | Some("js") | Some("jsx"))
            } else {
                false
            }
        })
        .collect::<Vec<_>>();
    
    // Parse imports from each file
    for file_path in &files {
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let relative_path = file_path.strip_prefix(project_dir)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();
            
            let imports = extract_imports(&content);
            import_graph.insert(relative_path.clone(), imports.clone());
            
            // Count imports for popularity analysis
            for import in imports {
                *import_counts.entry(import).or_insert(0) += 1;
            }
            
            // Analyze component hierarchy for React components
            if file_path.extension().map_or(false, |ext| ext == "tsx" || ext == "jsx") {
                let children = extract_jsx_children(&content);
                if !children.is_empty() {
                    component_hierarchy.insert(relative_path, children);
                }
            }
        }
    }
    
    // Find most imported files (top 10)
    let mut most_imported: Vec<(String, usize)> = import_counts.into_iter().collect();
    most_imported.sort_by(|a, b| b.1.cmp(&a.1));
    most_imported.truncate(10);
    // Keep as Vec<(String, usize)> for the struct
    // Don't convert to strings here
    
    // Basic circular dependency detection (simplified)
    let circular_dependencies = detect_circular_dependencies(&import_graph)
        .into_iter()
        .map(|file| vec![file]) // Convert single files to cycles
        .collect();
    
    Ok(FileRelationships {
        import_graph,
        component_hierarchy,
        most_imported: most_imported.into_iter().take(10).collect(),
        circular_dependencies,
    })
}

fn extract_imports(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    
    for line in content.lines() {
        let line = line.trim();
        
        // Match import statements
        if line.starts_with("import ") && line.contains(" from ") {
            if let Some(from_pos) = line.rfind(" from ") {
                let import_path = &line[from_pos + 6..];
                let import_path = import_path.trim()
                    .trim_matches('\'')
                    .trim_matches('"')
                    .trim_matches(';');
                
                // Only include relative imports
                if import_path.starts_with('.') {
                    imports.push(import_path.to_string());
                }
            }
        }
        
        // Match require statements
        if line.contains("require(") {
            if let Some(start) = line.find("require(") {
                let after_require = &line[start + 8..];
                if let Some(end) = after_require.find(')') {
                    let import_path = &after_require[..end]
                        .trim()
                        .trim_matches('\'')
                        .trim_matches('"');
                    
                    if import_path.starts_with('.') {
                        imports.push(import_path.to_string());
                    }
                }
            }
        }
    }
    
    imports
}

fn extract_jsx_children(content: &str) -> Vec<String> {
    let mut children = Vec::new();
    
    // Simple regex-like pattern matching for JSX components
    for line in content.lines() {
        // Look for JSX component usage like <ComponentName>
        if line.contains('<') && line.contains('>') {
            let line = line.trim();
            let mut chars = line.chars().peekable();
            
            while let Some(ch) = chars.next() {
                if ch == '<' {
                    if chars.peek() == Some(&'/') {
                        continue; // Skip closing tags
                    }
                    
                    let mut component_name = String::new();
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_alphanumeric() || next_ch == '_' {
                            component_name.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    
                    // Only include components that start with uppercase (React convention)
                    if !component_name.is_empty() && component_name.chars().next().unwrap().is_uppercase() {
                        children.push(component_name);
                    }
                }
            }
        }
    }
    
    children.sort();
    children.dedup();
    children
}

fn detect_circular_dependencies(import_graph: &HashMap<String, Vec<String>>) -> Vec<String> {
    let mut circular_deps = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut rec_stack = std::collections::HashSet::new();
    
    for file in import_graph.keys() {
        if !visited.contains(file) {
            if has_cycle(file, import_graph, &mut visited, &mut rec_stack, &mut Vec::new()) {
                // Found a cycle involving this file
                circular_deps.push(file.clone());
            }
        }
    }
    
    circular_deps
}

fn has_cycle(
    file: &str,
    graph: &HashMap<String, Vec<String>>,
    visited: &mut std::collections::HashSet<String>,
    rec_stack: &mut std::collections::HashSet<String>,
    _path: &mut Vec<String>,
) -> bool {
    visited.insert(file.to_string());
    rec_stack.insert(file.to_string());
    
    if let Some(imports) = graph.get(file) {
        for import in imports {
            // Resolve relative import to absolute path (simplified)
            let resolved_import = resolve_import_path(file, import);
            
            if !visited.contains(&resolved_import) {
                if has_cycle(&resolved_import, graph, visited, rec_stack, _path) {
                    return true;
                }
            } else if rec_stack.contains(&resolved_import) {
                return true; // Found cycle
            }
        }
    }
    
    rec_stack.remove(file);
    false
}

fn resolve_import_path(current_file: &str, import_path: &str) -> String {
    // Simplified import resolution
    if import_path.starts_with("./") {
        // Same directory
        let dir = std::path::Path::new(current_file).parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        format!("{}/{}", dir, &import_path[2..])
    } else if import_path.starts_with("../") {
        // Parent directory - simplified resolution
        let mut parts: Vec<&str> = current_file.split('/').collect();
        parts.pop(); // Remove filename
        
        let mut import_parts: Vec<&str> = import_path.split('/').collect();
        while import_parts.first() == Some(&"..") {
            import_parts.remove(0);
            if !parts.is_empty() {
                parts.pop();
            }
        }
        
        parts.extend(import_parts);
        parts.join("/")
    } else {
        import_path.to_string()
    }
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
    print_dependencies_summary(&report.dependencies);
    print_architecture_insights(&report.architecture);
    print_component_analysis(&report.structure);
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
        
        // Sort directories by file count (most important first)
        let mut sorted_dirs = structure.directories.clone();
        sorted_dirs.sort_by(|a, b| b.file_count.cmp(&a.file_count));
        
        for dir in sorted_dirs.iter().take(8) { // Show top 8 directories
            let purpose_emoji = match dir.purpose {
                DirectoryPurpose::Components => "🧩",
                DirectoryPurpose::Pages => "📄",
                DirectoryPurpose::Api => "🔌",
                DirectoryPurpose::Utils => "🛠️",
                DirectoryPurpose::Services => "⚙️",
                DirectoryPurpose::Styles => "🎨",
                DirectoryPurpose::Public => "🌐",
                DirectoryPurpose::Config => "⚙️",
                DirectoryPurpose::Tests => "🧪",
                _ => "📂",
            };
            
            let purpose_str = match dir.purpose {
                DirectoryPurpose::Components => "Components",
                DirectoryPurpose::Pages => "Pages",
                DirectoryPurpose::Api => "API",
                DirectoryPurpose::Utils => "Utils",
                DirectoryPurpose::Services => "Services",
                DirectoryPurpose::Styles => "Styles",
                DirectoryPurpose::Public => "Public",
                DirectoryPurpose::Config => "Config",
                DirectoryPurpose::Tests => "Tests",
                DirectoryPurpose::Build => "Build",
                DirectoryPurpose::Other => "Other",
            };
            
            println!("    {} {} ({} files, {} lines)", 
                purpose_emoji, 
                dir.path.bold(), 
                dir.file_count, 
                dir.line_count
            );
            println!("      {} | File types: {}", 
                purpose_str.dimmed(),
                dir.main_file_types.join(", ").dimmed()
            );
        }
    }
    
    println!();
}

fn print_dependencies_summary(dependencies: &DependencyAnalysis) {
    println!("{}", "📦 DEPENDENCIES OVERVIEW".bold().magenta());
    println!("{}", "────────────────────────".magenta());
    
    if let Some(package_info) = &dependencies.package_json {
        println!("  Production Dependencies: {}", package_info.dependencies.len());
        println!("  Development Dependencies: {}", package_info.dev_dependencies.len());
        
        if !package_info.main_dependencies.is_empty() {
            println!("  Key Frameworks/Tools:");
            for dep in &package_info.main_dependencies {
                if let Some(version) = package_info.dependencies.get(dep) {
                    println!("    📚 {} ({})", dep.bold(), version.dimmed());
                }
            }
        }
        
        if !dependencies.external_dependencies.is_empty() {
            let external_count = dependencies.external_dependencies.len();
            println!("  External packages used in code: {}", external_count);
            if external_count > 10 {
                println!("    Top imports: {}", 
                    dependencies.external_dependencies.iter().take(8).cloned().collect::<Vec<_>>().join(", ").dimmed()
                );
            }
        }
    } else {
        println!("  {} No package.json found", "⚠️".yellow());
    }
    
    println!();
}

fn print_component_analysis(structure: &ProjectStructure) {
    if !structure.components.is_empty() {
        println!("{}", "⚛️  COMPONENT ANALYSIS".bold().green());
        println!("{}", "────────────────────────".green());
        
        println!("  Total Components: {}", structure.components.len());
        
        // Group by component type
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for component in &structure.components {
            let type_name = match component.component_type {
                ComponentType::Page => "Pages",
                ComponentType::Layout => "Layouts", 
                ComponentType::Feature => "Features",
                ComponentType::UI => "UI Components",
                ComponentType::Hook => "Custom Hooks",
                ComponentType::Context => "Context Providers",
            }.to_string();
            *type_counts.entry(type_name).or_insert(0) += 1;
        }
        
        for (component_type, count) in type_counts {
            println!("    {}: {}", component_type, count);
        }
        
        // Show hooks usage summary
        let mut all_hooks: HashMap<String, usize> = HashMap::new();
        for component in &structure.components {
            for hook in &component.hooks_used {
                *all_hooks.entry(hook.clone()).or_insert(0) += 1;
            }
        }
        
        if !all_hooks.is_empty() {
            println!("  Most Used Hooks:");
            let mut hook_vec: Vec<(String, usize)> = all_hooks.into_iter().collect();
            hook_vec.sort_by(|a, b| b.1.cmp(&a.1));
            
            for (hook, count) in hook_vec.iter().take(5) {
                println!("    🎣 {} (used in {} components)", hook.bold(), count);
            }
        }
        
        // Show complex components
        let complex_components: Vec<&ComponentInfo> = structure.components.iter()
            .filter(|c| c.children_components.len() > 5)
            .collect();
        
        if !complex_components.is_empty() {
            println!("  Complex Components (>5 children):");
            for component in complex_components.iter().take(3) {
                println!("    🏗️  {} ({} children)", component.name.bold(), component.children_components.len());
            }
        }
        
        println!();
    }
}

fn print_architecture_insights(insights: &ArchitectureInsights) {
    println!("{}", "🏛️  ARCHITECTURE INSIGHTS".bold().yellow());
    println!("{}", "─────────────────────────".yellow());
    
    // Color-coded organization score
    let score_color = if insights.organization_score >= 80.0 {
        insights.organization_score.to_string().green()
    } else if insights.organization_score >= 60.0 {
        insights.organization_score.to_string().yellow()
    } else {
        insights.organization_score.to_string().red()
    };
    
    println!("  Organization Score: {:.1}%", score_color);
    
    let complexity_display = match insights.complexity_level {
        ComplexityLevel::Simple => "Simple 🟢".green(),
        ComplexityLevel::Moderate => "Moderate 🟡".yellow(),
        ComplexityLevel::Complex => "Complex 🟠".yellow(),
        ComplexityLevel::VeryComplex => "Very Complex 🔴".red(),
    };
    println!("  Complexity Level: {}", complexity_display);
    
    if !insights.patterns.is_empty() {
        println!("  Detected Patterns:");
        for pattern in &insights.patterns {
            let pattern_emoji = match pattern {
                ArchitecturePattern::LayeredArchitecture => "🏗️",
                ArchitecturePattern::ComponentComposition => "🧩",
                ArchitecturePattern::CustomHooks => "🎣",
                ArchitecturePattern::ContextProviders => "🌐",
                ArchitecturePattern::ServiceLayer => "⚙️",
                ArchitecturePattern::UtilityFirst => "🛠️",
                ArchitecturePattern::ConfigDriven => "⚙️",
            };
            
            let pattern_name = match pattern {
                ArchitecturePattern::LayeredArchitecture => "Layered Architecture",
                ArchitecturePattern::ComponentComposition => "Component Composition",
                ArchitecturePattern::CustomHooks => "Custom Hooks",
                ArchitecturePattern::ContextProviders => "Context Providers",
                ArchitecturePattern::ServiceLayer => "Service Layer",
                ArchitecturePattern::UtilityFirst => "Utility First",
                ArchitecturePattern::ConfigDriven => "Config Driven",
            };
            
            println!("    {} {}", pattern_emoji, pattern_name);
        }
    }
    
    if !insights.recommendations.is_empty() {
        println!();
        println!("{}", "💡 RECOMMENDATIONS".bold().blue());
        println!("{}", "──────────────────".blue());
        for (i, rec) in insights.recommendations.iter().enumerate() {
            println!("  {}. {}", i + 1, rec);
        }
    }
    
    println!();
}