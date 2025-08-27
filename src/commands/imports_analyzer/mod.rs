mod types;
mod resolver;
mod validation;
mod parser;
mod reporter;

use anyhow::Result;
use colored::*;
use rayon::prelude::*;
use std::fs;
use std::path::Path;

use crate::common::{
    FileScanner, get_common_patterns, ExitCode, check_failure_threshold, 
    progress::FileProgressTracker
};

use types::{ImportsReport, ImportsSummary, UnusedImport, FileAnalysis};
use resolver::PathAliasResolver;
use validation::check_import_validity;
use parser::{parse_import_statement, find_unused_items, collect_used_identifiers};
use reporter::{print_report, calculate_savings};

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
    
    // Create path alias resolver
    let path_resolver = PathAliasResolver::from_project_root(&current_dir);
    
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
            analyses.push(analyze_file_imports(path, &current_dir, &path_resolver)?);
        }
        progress.finish_with_message(&format!("Analyzed {} files", files_count));
        analyses
    } else {
        // Parallel processing for smaller projects (no progress needed)
        files
            .par_iter()
            .map(|path| analyze_file_imports(path, &current_dir, &path_resolver))
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

fn analyze_file_imports(
    path: &Path, 
    project_root: &Path, 
    path_resolver: &Option<PathAliasResolver>
) -> Result<FileAnalysis> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    let mut imports = Vec::new();
    
    // Extract import statements
    let patterns = get_common_patterns();
    
    // First pass: collect imports
    for (line_num, line) in lines.iter().enumerate() {
        if let Some(captures) = patterns.import_statement.captures(line.trim()) {
            let import_spec = captures.get(1).unwrap().as_str();
            let import_path = captures.get(2).unwrap().as_str();
            
            let parsed_import = parse_import_statement(import_spec, import_path);
            imports.push((line_num + 1, line.trim().to_string(), parsed_import, import_path.to_string()));
        }
    }
    
    // Second pass: collect used identifiers
    let used_identifiers = collect_used_identifiers(&lines)?;
    
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
        if let Some(broken_import) = check_import_validity(
            path, 
            project_root, 
            &import_path, 
            line_num, 
            &import_statement, 
            path_resolver
        )? {
            broken_imports.push(broken_import);
        }
    }
    
    Ok(FileAnalysis {
        total_imports,
        unused_imports,
        broken_imports,
    })
}