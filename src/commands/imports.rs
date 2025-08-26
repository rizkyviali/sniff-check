use anyhow::Result;
use colored::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use crate::common::{FileScanner, get_common_patterns, is_keyword_or_builtin, ExitCode, check_failure_threshold, progress::FileProgressTracker};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportsReport {
    pub unused_imports: Vec<UnusedImport>,
    pub broken_imports: Vec<BrokenImport>,
    pub summary: ImportsSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnusedImport {
    pub file: String,
    pub line: usize,
    pub import_statement: String,
    pub unused_items: Vec<String>,
    pub import_type: ImportType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrokenImport {
    pub file: String,
    pub line: usize,
    pub import_statement: String,
    pub import_path: String,
    pub error_type: BrokenImportType,
    pub suggestion: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BrokenImportType {
    FileNotFound,
    ModuleNotInstalled,
    InvalidPath,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ImportType {
    DefaultImport,
    NamedImport,
    NamespaceImport,
    SideEffectImport,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportsSummary {
    pub files_scanned: usize,
    pub total_imports: usize,
    pub unused_imports: usize,
    pub broken_imports: usize,
    pub potential_savings: String,
}

pub async fn run(json: bool, quiet: bool) -> Result<()> {
    if !quiet {
        println!("{}", "🔍 Scanning for unused and broken imports...".bold().blue());
    }
    
    let report = analyze_imports(quiet)?;
    
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report, quiet);
    }
    
    // Use common error handling for imports issues
    check_failure_threshold(
        report.summary.unused_imports > 0 || report.summary.broken_imports > 0, 
        ExitCode::ValidationFailed
    );
    
    Ok(())
}

fn analyze_imports(quiet: bool) -> Result<ImportsReport> {
    let current_dir = std::env::current_dir()?;
    let scanner = FileScanner::with_defaults();
    let files = scanner.find_js_ts_files(&current_dir);
    
    let files_count = files.len();
    
    // Show progress for larger projects (>50 files)
    let progress = if files_count > 50 {
        FileProgressTracker::new(
            "Analyzing imports", 
            Some(files_count), 
            quiet
        )
    } else {
        FileProgressTracker::new("Analyzing imports", None, true) // No progress for small projects
    };
    
    let file_analyses: Vec<FileAnalysis> = if files_count > 50 {
        // Sequential processing with progress for large projects
        let mut analyses = Vec::with_capacity(files_count);
        for (i, path) in files.iter().enumerate() {
            progress.set_position(i as u64);
            analyses.push(analyze_file_imports(path, &current_dir)?);
        }
        progress.finish_with_message(&format!("Analyzed {} files", files_count));
        analyses
    } else {
        // Parallel processing for smaller projects (no progress needed)
        files
            .par_iter()
            .map(|path| analyze_file_imports(path, &current_dir))
            .collect::<Result<Vec<_>, _>>()?
    };
    
    let mut unused_imports = Vec::new();
    let mut broken_imports = Vec::new();
    let mut total_imports = 0;
    
    for analysis in file_analyses {
        total_imports += analysis.total_imports;
        unused_imports.extend(analysis.unused_imports);
        broken_imports.extend(analysis.broken_imports);
    }
    
    let summary = ImportsSummary {
        files_scanned: files_count,
        total_imports,
        unused_imports: unused_imports.len(),
        broken_imports: broken_imports.len(),
        potential_savings: calculate_savings(&unused_imports),
    };
    
    Ok(ImportsReport {
        unused_imports,
        broken_imports,
        summary,
    })
}


struct FileAnalysis {
    total_imports: usize,
    unused_imports: Vec<UnusedImport>,
    broken_imports: Vec<BrokenImport>,
}

fn analyze_file_imports(path: &Path, project_root: &Path) -> Result<FileAnalysis> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    let mut imports = Vec::new();
    let mut used_identifiers = HashSet::new();
    
    // Extract import statements and used identifiers
    let patterns = get_common_patterns();
    
    // Comprehensive usage detection patterns
    let general_usage = regex::Regex::new(r"\b([A-Z][a-zA-Z0-9_]*|[a-z][a-zA-Z0-9_]*)\b")?;
    let react_hook_usage = regex::Regex::new(r"const\s*\[([^,\]]+),\s*([^\]]+)\]\s*=\s*(use[A-Z]\w*)")?;
    let type_annotation = regex::Regex::new(r":\s*([A-Z][a-zA-Z0-9_<>,\s\[\]]*)")?;
    let generic_usage = regex::Regex::new(r"<([A-Z][a-zA-Z0-9_<>,\s\[\]]*?)>")?;
    let jsx_usage = regex::Regex::new(r"</?([A-Z][a-zA-Z0-9_.]*)")?;
    let interface_extends = regex::Regex::new(r"(?:extends|implements)\s+([A-Z][a-zA-Z0-9_<>,\s]*)")?;
    let function_param_type = regex::Regex::new(r"\(\s*[^:)]*:\s*([A-Z][a-zA-Z0-9_<>,\s\[\]]*)")?;
    
    // First pass: collect imports
    for (line_num, line) in lines.iter().enumerate() {
        if let Some(captures) = patterns.import_statement.captures(line.trim()) {
            let import_spec = captures.get(1).unwrap().as_str();
            let import_path = captures.get(2).unwrap().as_str();
            
            let parsed_import = parse_import_statement(import_spec, import_path);
            imports.push((line_num + 1, line.trim().to_string(), parsed_import, import_path.to_string()));
        }
    }
    
    // Second pass: collect used identifiers with comprehensive detection
    for (_line_num, line) in lines.iter().enumerate() {
        // Skip import lines
        if patterns.import_statement.is_match(line.trim()) {
            continue;
        }
        
        let line_content = line.trim();
        
        // 1. General identifier usage
        for cap in general_usage.find_iter(line_content) {
            let identifier = cap.as_str();
            if !is_keyword_or_builtin(identifier) {
                used_identifiers.insert(identifier.to_string());
            }
        }
        
        // 2. React hook usage patterns: const [state, setState] = useState()
        if let Some(captures) = react_hook_usage.captures(line_content) {
            if let Some(hook_name) = captures.get(3) {
                used_identifiers.insert(hook_name.as_str().to_string());
            }
        }
        
        // 3. TypeScript type annotations: variable: Type
        for captures in type_annotation.captures_iter(line_content) {
            if let Some(type_match) = captures.get(1) {
                extract_type_identifiers(type_match.as_str(), &mut used_identifiers);
            }
        }
        
        // 4. Generic type usage: Array<Type>, Promise<Result>
        for captures in generic_usage.captures_iter(line_content) {
            if let Some(generic_match) = captures.get(1) {
                extract_type_identifiers(generic_match.as_str(), &mut used_identifiers);
            }
        }
        
        // 5. JSX component usage: <Component>
        for captures in jsx_usage.captures_iter(line_content) {
            if let Some(component) = captures.get(1) {
                used_identifiers.insert(component.as_str().to_string());
            }
        }
        
        // 6. Interface extends/implements
        for captures in interface_extends.captures_iter(line_content) {
            if let Some(interface_match) = captures.get(1) {
                extract_type_identifiers(interface_match.as_str(), &mut used_identifiers);
            }
        }
        
        // 7. Function parameter types: (param: Type)
        for captures in function_param_type.captures_iter(line_content) {
            if let Some(param_type) = captures.get(1) {
                extract_type_identifiers(param_type.as_str(), &mut used_identifiers);
            }
        }
    }
    
    // Check which imports are unused and broken
    let mut unused_imports = Vec::new();
    let mut broken_imports = Vec::new();
    let total_imports = imports.len();
    
    for (line_num, import_statement, parsed_import, import_path) in imports {
        // Check for unused imports
        let unused_items = find_unused_items(&parsed_import, &used_identifiers);
        if !unused_items.is_empty() {
            unused_imports.push(UnusedImport {
                file: path.to_string_lossy().to_string(),
                line: line_num,
                import_statement: import_statement.clone(),
                unused_items,
                import_type: parsed_import.import_type,
            });
        }
        
        // Check for broken imports
        if let Some(broken_import) = check_import_validity(path, project_root, &import_path, line_num, &import_statement)? {
            broken_imports.push(broken_import);
        }
    }
    
    Ok(FileAnalysis {
        total_imports,
        unused_imports,
        broken_imports,
    })
}

#[derive(Debug)]
struct ParsedImport {
    import_type: ImportType,
    default_import: Option<String>,
    named_imports: Vec<String>,
    namespace_import: Option<String>,
    // module_path field removed as unused
}

fn parse_import_statement(import_spec: &str, _module_path: &str) -> ParsedImport {
    let spec = import_spec.trim();
    
    // Handle TypeScript 'type' imports by removing the 'type' keyword
    let cleaned_spec = if spec.starts_with("type ") {
        spec[5..].trim()
    } else {
        spec
    };
    
    // Check for different import patterns
    if cleaned_spec.starts_with('{') && cleaned_spec.ends_with('}') {
        // Named imports: { foo, bar } or { type Foo, Bar }
        let named_part = &cleaned_spec[1..cleaned_spec.len()-1];
        let named_imports: Vec<String> = named_part
            .split(',')
            .map(|s| {
                let trimmed = s.trim();
                // Handle inline type imports like "type NextRequest"
                if trimmed.starts_with("type ") {
                    // Extract the actual type name after "type "
                    trimmed[5..].trim().to_string()
                } else {
                    // Handle regular imports and "as" aliases
                    trimmed.split_whitespace().next().unwrap_or("").to_string()
                }
            })
            .filter(|s| !s.is_empty())
            .collect();
        
        ParsedImport {
            import_type: ImportType::NamedImport,
            default_import: None,
            named_imports,
            namespace_import: None,
        }
    } else if cleaned_spec.contains(" as ") && cleaned_spec.starts_with('*') {
        // Namespace import: * as foo
        let parts: Vec<&str> = cleaned_spec.split(" as ").collect();
        let namespace_name = parts.get(1).unwrap_or(&"").trim().to_string();
        
        ParsedImport {
            import_type: ImportType::NamespaceImport,
            default_import: None,
            named_imports: Vec::new(),
            namespace_import: Some(namespace_name),
        }
    } else if cleaned_spec.contains(',') {
        // Mixed import: foo, { bar, baz }
        let parts: Vec<&str> = cleaned_spec.split(',').collect();
        let default_import = Some(parts[0].trim().to_string());
        
        let mut named_imports = Vec::new();
        for part in parts.iter().skip(1) {
            let part = part.trim();
            if part.starts_with('{') && part.ends_with('}') {
                let named_part = &part[1..part.len()-1];
                named_imports.extend(
                    named_part
                        .split(',')
                        .map(|s| s.trim().split_whitespace().next().unwrap_or("").to_string())
                        .filter(|s| !s.is_empty())
                );
            }
        }
        
        ParsedImport {
            import_type: ImportType::DefaultImport,
            default_import,
            named_imports,
            namespace_import: None,
        }
    } else {
        // Default import: foo
        ParsedImport {
            import_type: ImportType::DefaultImport,
            default_import: Some(cleaned_spec.to_string()),
            named_imports: Vec::new(),
            namespace_import: None,
        }
    }
}

fn find_unused_items(parsed_import: &ParsedImport, used_identifiers: &HashSet<String>) -> Vec<String> {
    let mut unused = Vec::new();
    
    // Check default import
    if let Some(ref default) = parsed_import.default_import {
        if !used_identifiers.contains(default) {
            unused.push(default.clone());
        }
    }
    
    // Check named imports
    for named in &parsed_import.named_imports {
        if !used_identifiers.contains(named) {
            unused.push(named.clone());
        }
    }
    
    // Check namespace import
    if let Some(ref namespace) = parsed_import.namespace_import {
        if !used_identifiers.contains(namespace) {
            unused.push(namespace.clone());
        }
    }
    
    unused
}

/// Extract type identifiers from a type string like "Product & { creator: Creator }" or "Array<User>"
fn extract_type_identifiers(type_str: &str, used_identifiers: &mut HashSet<String>) {
    // Clean up the type string and extract identifiers
    let type_identifier_regex = regex::Regex::new(r"\b([A-Z][a-zA-Z0-9_]*)\b").unwrap();
    
    for cap in type_identifier_regex.find_iter(type_str) {
        let identifier = cap.as_str();
        // Skip built-in TypeScript types
        if !is_typescript_builtin_type(identifier) {
            used_identifiers.insert(identifier.to_string());
        }
    }
}

/// Check if an identifier is a built-in TypeScript type
fn is_typescript_builtin_type(identifier: &str) -> bool {
    matches!(identifier, 
        "Array" | "Promise" | "Record" | "Partial" | "Required" | "Pick" | "Omit" | 
        "Exclude" | "Extract" | "NonNullable" | "Parameters" | "ConstructorParameters" |
        "ReturnType" | "InstanceType" | "ThisParameterType" | "OmitThisParameter" |
        "ThisType" | "Uppercase" | "Lowercase" | "Capitalize" | "Uncapitalize" |
        "String" | "Number" | "Boolean" | "Object" | "Function" | "Date" | "RegExp" |
        "Error" | "Map" | "Set" | "WeakMap" | "WeakSet" | "ArrayBuffer" | "DataView" |
        "Int8Array" | "Uint8Array" | "Uint8ClampedArray" | "Int16Array" | "Uint16Array" |
        "Int32Array" | "Uint32Array" | "Float32Array" | "Float64Array" | "BigInt64Array" |
        "BigUint64Array"
    )
}

fn check_import_validity(
    current_file: &Path,
    project_root: &Path,
    import_path: &str,
    line_num: usize,
    import_statement: &str,
) -> Result<Option<BrokenImport>> {
    // Skip node_modules imports for now (we'd need to check package.json)
    if !import_path.starts_with('.') {
        return check_node_modules_import(current_file, project_root, import_path, line_num, import_statement);
    }
    
    // Handle relative imports
    let current_dir = current_file.parent().unwrap();
    let resolved_path = resolve_import_path(current_dir, import_path);
    
    // Check if the resolved path exists (try common extensions)
    if !import_exists(&resolved_path) {
        // Try to find a suggestion
        let suggestion = find_similar_file(current_dir, import_path);
        
        return Ok(Some(BrokenImport {
            file: current_file.to_string_lossy().to_string(),
            line: line_num,
            import_statement: import_statement.to_string(),
            import_path: import_path.to_string(),
            error_type: BrokenImportType::FileNotFound,
            suggestion,
        }));
    }
    
    Ok(None)
}

fn check_node_modules_import(
    _current_file: &Path,
    project_root: &Path,
    import_path: &str,
    line_num: usize,
    import_statement: &str,
) -> Result<Option<BrokenImport>> {
    // Extract package name (handle scoped packages like @types/node)
    let package_name = if import_path.starts_with('@') {
        // Scoped package: @scope/package or @scope/package/subpath
        let parts: Vec<&str> = import_path.splitn(3, '/').collect();
        if parts.len() >= 2 {
            format!("{}/{}", parts[0], parts[1])
        } else {
            import_path.to_string()
        }
    } else {
        // Regular package: package or package/subpath
        import_path.split('/').next().unwrap_or(import_path).to_string()
    };
    
    // Check if package exists in node_modules
    let node_modules_path = project_root.join("node_modules").join(&package_name);
    if !node_modules_path.exists() {
        return Ok(Some(BrokenImport {
            file: _current_file.to_string_lossy().to_string(),
            line: line_num,
            import_statement: import_statement.to_string(),
            import_path: import_path.to_string(),
            error_type: BrokenImportType::ModuleNotInstalled,
            suggestion: Some(format!("Run: npm install {}", package_name)),
        }));
    }
    
    Ok(None)
}

fn resolve_import_path(current_dir: &Path, import_path: &str) -> PathBuf {
    let mut resolved = current_dir.to_path_buf();
    
    // Split path and navigate
    for part in import_path.split('/') {
        match part {
            "." => {} // Current directory, do nothing
            ".." => { resolved.pop(); }
            _ => resolved.push(part),
        }
    }
    
    resolved
}

fn import_exists(base_path: &PathBuf) -> bool {
    // Try the exact path first
    if base_path.exists() {
        return true;
    }
    
    // Try common JavaScript/TypeScript file extensions
    let extensions = [".js", ".ts", ".jsx", ".tsx", ".json", ".mjs", ".cjs"];
    
    for ext in extensions {
        let with_ext = base_path.with_extension(&ext[1..]);
        if with_ext.exists() {
            return true;
        }
    }
    
    // Try index files in the directory
    if base_path.is_dir() || !base_path.exists() {
        for ext in extensions {
            let index_file = base_path.join(format!("index{}", ext));
            if index_file.exists() {
                return true;
            }
        }
    }
    
    false
}

fn find_similar_file(current_dir: &Path, import_path: &str) -> Option<String> {
    // Extract the filename from the import path
    let filename = import_path.split('/').last()?;
    
    // Look for similar files in the current directory and parent directories
    let search_dirs = [current_dir, current_dir.parent()?];
    
    for search_dir in search_dirs {
        if let Ok(entries) = fs::read_dir(search_dir) {
            for entry in entries.flatten() {
                if let Some(entry_name) = entry.file_name().to_str() {
                    // Remove extension for comparison
                    let entry_path = entry.path();
                    if let Some(entry_stem) = entry_path.file_stem().and_then(|s| s.to_str()) {
                        if entry_stem.to_lowercase().contains(&filename.to_lowercase()) {
                            // Build relative path suggestion
                            let relative = if search_dir == current_dir {
                                format!("./{}", entry_name)
                            } else {
                                format!("../{}", entry_name)
                            };
                            return Some(relative);
                        }
                    }
                }
            }
        }
    }
    
    None
}


fn calculate_savings(unused_imports: &[UnusedImport]) -> String {
    let total_lines = unused_imports.len();
    if total_lines == 0 {
        "0 lines".to_string()
    } else {
        format!("~{} lines of code", total_lines)
    }
}

fn print_report(report: &ImportsReport, quiet: bool) {
    if !quiet {
        println!();
        println!("{}", "📊 Imports Analysis Report".bold().blue());
        println!("{}", "==========================".blue());
        println!();
    }
    
    let has_issues = !report.unused_imports.is_empty() || !report.broken_imports.is_empty();
    
    if !has_issues {
        println!("{}", "✅ No import issues found! Your imports are clean.".green());
        return;
    }
    
    // Group unused imports by file
    let mut unused_by_file: HashMap<String, Vec<&UnusedImport>> = HashMap::new();
    for import in &report.unused_imports {
        unused_by_file.entry(import.file.clone()).or_default().push(import);
    }
    
    // Group broken imports by file
    let mut broken_by_file: HashMap<String, Vec<&BrokenImport>> = HashMap::new();
    for import in &report.broken_imports {
        broken_by_file.entry(import.file.clone()).or_default().push(import);
    }
    
    // Get all unique files
    let mut all_files: HashSet<String> = HashSet::new();
    all_files.extend(unused_by_file.keys().cloned());
    all_files.extend(broken_by_file.keys().cloned());
    
    // Print issues by file
    for file in all_files {
        println!("{}", file.cyan().bold());
        
        // Print unused imports for this file
        if let Some(unused_imports) = unused_by_file.get(&file) {
            for import in unused_imports {
                println!("  Line {}: {}", import.line.to_string().yellow(), import.import_statement.dimmed());
                println!("    {} Unused: {}", "🚫".red(), import.unused_items.join(", ").red());
                println!();
            }
        }
        
        // Print broken imports for this file
        if let Some(broken_imports) = broken_by_file.get(&file) {
            for import in broken_imports {
                println!("  Line {}: {}", import.line.to_string().yellow(), import.import_statement.dimmed());
                let error_msg = match import.error_type {
                    BrokenImportType::FileNotFound => format!("File not found: {}", import.import_path),
                    BrokenImportType::ModuleNotInstalled => format!("Module not installed: {}", import.import_path),
                    BrokenImportType::InvalidPath => format!("Invalid path: {}", import.import_path),
                };
                println!("    {} {}", "💥".red(), error_msg.red());
                if let Some(ref suggestion) = import.suggestion {
                    println!("    {} {}", "💡".yellow(), suggestion.green());
                }
                println!();
            }
        }
    }
    
    // Print summary
    print_summary(&report.summary);
}

fn print_summary(summary: &ImportsSummary) {
    println!("{}", "📈 SUMMARY".bold().white());
    println!("{}", "─────────".white());
    println!("  Files scanned: {}", summary.files_scanned);
    println!("  Total imports: {}", summary.total_imports);
    println!("  {} {}", "Unused imports:".red(), summary.unused_imports.to_string().red());
    println!("  {} {}", "Broken imports:".red(), summary.broken_imports.to_string().red());
    println!("  Potential savings: {}", summary.potential_savings.green());
    
    println!();
    
    if summary.unused_imports > 0 {
        println!("{}", "💡 TIP: Remove unused imports to reduce bundle size and improve build performance".dimmed());
        println!("{}", "🔧 Consider using an IDE extension or linter to automatically remove unused imports".dimmed());
    }
    
    if summary.broken_imports > 0 {
        println!("{}", "🔧 Fix broken imports to resolve compilation errors".yellow());
        println!("{}", "💡 Check if files were moved/renamed, or if packages need to be installed".dimmed());
    }
}