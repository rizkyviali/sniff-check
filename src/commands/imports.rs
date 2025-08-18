use anyhow::Result;
use colored::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use crate::common::{FileScanner, get_common_patterns, is_keyword_or_builtin, ExitCode, check_failure_threshold};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportsReport {
    pub unused_imports: Vec<UnusedImport>,
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
    pub potential_savings: String,
}

pub async fn run(json: bool, quiet: bool) -> Result<()> {
    if !quiet {
        println!("{}", "🔍 Scanning for unused imports...".bold().blue());
    }
    
    let report = analyze_imports()?;
    
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report, quiet);
    }
    
    // Use common error handling for unused imports
    check_failure_threshold(report.summary.unused_imports > 0, ExitCode::ValidationFailed);
    
    Ok(())
}

fn analyze_imports() -> Result<ImportsReport> {
    let current_dir = std::env::current_dir()?;
    let scanner = FileScanner::with_defaults();
    let files = scanner.find_js_ts_files(&current_dir);
    let files_count = files.len();
    
    let file_analyses: Vec<FileAnalysis> = files
        .par_iter()
        .map(|path| analyze_file_imports(path))
        .collect::<Result<Vec<_>, _>>()?;
    
    let mut unused_imports = Vec::new();
    let mut total_imports = 0;
    
    for analysis in file_analyses {
        total_imports += analysis.total_imports;
        unused_imports.extend(analysis.unused_imports);
    }
    
    let summary = ImportsSummary {
        files_scanned: files_count,
        total_imports,
        unused_imports: unused_imports.len(),
        potential_savings: calculate_savings(&unused_imports),
    };
    
    Ok(ImportsReport {
        unused_imports,
        summary,
    })
}


struct FileAnalysis {
    total_imports: usize,
    unused_imports: Vec<UnusedImport>,
}

fn analyze_file_imports(path: &Path) -> Result<FileAnalysis> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    let mut imports = Vec::new();
    let mut used_identifiers = HashSet::new();
    
    // Extract import statements and used identifiers
    let patterns = get_common_patterns();
    let usage_regex = regex::Regex::new(r"\b([A-Z][a-zA-Z0-9_]*|[a-z][a-zA-Z0-9_]*)\b")?;
    
    // First pass: collect imports
    for (line_num, line) in lines.iter().enumerate() {
        if let Some(captures) = patterns.import_statement.captures(line.trim()) {
            let import_spec = captures.get(1).unwrap().as_str();
            let import_path = captures.get(2).unwrap().as_str();
            
            let parsed_import = parse_import_statement(import_spec, import_path);
            imports.push((line_num + 1, line.trim().to_string(), parsed_import));
        }
    }
    
    // Second pass: collect used identifiers (skip import lines)
    for (_line_num, line) in lines.iter().enumerate() {
        // Skip import lines
        if patterns.import_statement.is_match(line.trim()) {
            continue;
        }
        
        // Extract identifiers from this line
        for cap in usage_regex.find_iter(line) {
            let identifier = cap.as_str();
            // Skip common keywords and built-ins
            if !is_keyword_or_builtin(identifier) {
                used_identifiers.insert(identifier.to_string());
            }
        }
    }
    
    // Check which imports are unused
    let mut unused_imports = Vec::new();
    let total_imports = imports.len();
    
    for (line_num, import_statement, parsed_import) in imports {
        let unused_items = find_unused_items(&parsed_import, &used_identifiers);
        
        if !unused_items.is_empty() {
            unused_imports.push(UnusedImport {
                file: path.to_string_lossy().to_string(),
                line: line_num,
                import_statement,
                unused_items,
                import_type: parsed_import.import_type,
            });
        }
    }
    
    Ok(FileAnalysis {
        total_imports,
        unused_imports,
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
    
    // Check for different import patterns
    if spec.starts_with('{') && spec.ends_with('}') {
        // Named imports: { foo, bar }
        let named_part = &spec[1..spec.len()-1];
        let named_imports: Vec<String> = named_part
            .split(',')
            .map(|s| s.trim().split_whitespace().next().unwrap_or("").to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        ParsedImport {
            import_type: ImportType::NamedImport,
            default_import: None,
            named_imports,
            namespace_import: None,
        }
    } else if spec.contains(" as ") && spec.starts_with('*') {
        // Namespace import: * as foo
        let parts: Vec<&str> = spec.split(" as ").collect();
        let namespace_name = parts.get(1).unwrap_or(&"").trim().to_string();
        
        ParsedImport {
            import_type: ImportType::NamespaceImport,
            default_import: None,
            named_imports: Vec::new(),
            namespace_import: Some(namespace_name),
        }
    } else if spec.contains(',') {
        // Mixed import: foo, { bar, baz }
        let parts: Vec<&str> = spec.split(',').collect();
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
            default_import: Some(spec.to_string()),
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
        println!("{}", "📊 Unused Imports Report".bold().blue());
        println!("{}", "=======================".blue());
        println!();
    }
    
    if report.unused_imports.is_empty() {
        println!("{}", "✅ No unused imports found! Your imports are clean.".green());
        return;
    }
    
    // Group by file
    let mut imports_by_file: HashMap<String, Vec<&UnusedImport>> = HashMap::new();
    
    for import in &report.unused_imports {
        imports_by_file.entry(import.file.clone()).or_default().push(import);
    }
    
    // Print unused imports by file
    for (file, imports) in imports_by_file {
        println!("{}", file.cyan().bold());
        
        for import in imports {
            println!("  Line {}: {}", import.line.to_string().yellow(), import.import_statement.dimmed());
            println!("    {} Unused: {}", "🚫".red(), import.unused_items.join(", ").red());
            println!();
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
    println!("  Potential savings: {}", summary.potential_savings.green());
    
    println!();
    println!("{}", "💡 TIP: Remove unused imports to reduce bundle size and improve build performance".dimmed());
    
    if summary.unused_imports > 0 {
        println!("{}", "🔧 Consider using an IDE extension or linter to automatically remove unused imports".dimmed());
    }
}