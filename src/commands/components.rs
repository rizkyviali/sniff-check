use anyhow::Result;
use colored::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use crate::config::Config;
use crate::common::{ExitCode, check_failure_threshold, init_command, complete_command, create_standard_json_output, output_result};

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentReport {
    pub components: Vec<ComponentAnalysis>,
    pub summary: ComponentSummary,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentAnalysis {
    pub file_path: String,
    pub component_name: String,
    pub component_type: ComponentType,
    pub framework: Framework,
    pub line_count: usize,
    pub complexity_score: u32,
    pub issues: Vec<ComponentIssue>,
    pub refactor_suggestions: Vec<String>,
    pub extractable_parts: Vec<ExtractablePart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    FunctionalComponent,
    ClassComponent,
    VueComponent,
    AngularComponent,
    SvelteComponent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Framework {
    React,
    Vue,
    Angular,
    Svelte,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentIssue {
    pub issue_type: IssueType,
    pub line_number: usize,
    pub description: String,
    pub severity: IssueSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueType {
    TooManyLines,
    TooManyHooks,
    TooManyProps,
    ComplexLogic,
    MultipleConcerns,
    DeepNesting,
    DuplicatedCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractablePart {
    pub name: String,
    pub part_type: ExtractableType,
    pub start_line: usize,
    pub end_line: usize,
    pub suggested_filename: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtractableType {
    CustomHook,
    UtilityFunction,
    SubComponent,
    Constants,
    TypeDefinitions,
    BusinessLogic,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentSummary {
    pub total_components: usize,
    pub large_components: usize,
    pub complex_components: usize,
    pub components_needing_refactor: usize,
    pub potential_extractions: usize,
}

pub async fn run(threshold: usize, json: bool, quiet: bool) -> Result<()> {
    let start_time = std::time::Instant::now();
    init_command("component analysis", quiet);
    
    let config = Config::load().unwrap_or_default();
    let effective_threshold = if threshold == 100 {
        config.large_files.severity_levels.warning  // Use warning threshold from large files config
    } else {
        threshold
    };
    
    let report = analyze_components(effective_threshold, quiet)?;
    let duration_ms = start_time.elapsed().as_millis() as u64;
    
    let response = create_standard_json_output(
        "components",
        &report,
        report.summary.total_components,
        report.summary.components_needing_refactor,
        Some(duration_ms),
    );
    
    output_result(&response, json, quiet, |report, quiet| print_component_report(report, &config, quiet))?;
    
    complete_command("component analysis", report.summary.components_needing_refactor == 0, quiet);
    check_failure_threshold(report.summary.components_needing_refactor > 0, ExitCode::ThresholdExceeded);
    
    Ok(())
}

fn analyze_components(threshold: usize, quiet: bool) -> Result<ComponentReport> {
    let current_dir = std::env::current_dir()?;
    let mut components = Vec::new();
    
    if !quiet {
        println!("🔍 Scanning for React, Vue, Angular, and Svelte components...");
    }
    
    // Find component files
    let component_files = find_component_files(&current_dir);
    
    if !quiet {
        println!("📊 Analyzing {} components for size and complexity...", component_files.len());
    }
    
    for file_path in component_files {
        if let Ok(content) = fs::read_to_string(&file_path) {
            let line_count = content.lines().count();
            
            if line_count >= threshold {
                if let Some(analysis) = analyze_single_component(&file_path, &content, line_count) {
                    components.push(analysis);
                }
            }
        }
    }
    
    let summary = create_component_summary(&components);
    let recommendations = generate_global_recommendations(&components);
    
    Ok(ComponentReport {
        components,
        summary,
        recommendations,
    })
}

fn find_component_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut component_files = Vec::new();
    
    for entry in WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // Skip common directories
        if let Some(parent) = path.parent() {
            let parent_name = parent.file_name().unwrap_or_default().to_string_lossy();
            if parent_name.contains("node_modules") || 
               parent_name.contains(".git") ||
               parent_name.contains("dist") ||
               parent_name.contains("build") {
                continue;
            }
        }
        
        if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
            match extension {
                "tsx" | "jsx" | "vue" | "svelte" => {
                    component_files.push(path.to_path_buf());
                },
                "ts" | "js" => {
                    // Check if it's likely a component file based on naming conventions
                    if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                        if file_name.chars().next().unwrap_or('a').is_uppercase() || 
                           file_name.contains("Component") ||
                           file_name.contains("component") {
                            component_files.push(path.to_path_buf());
                        }
                    }
                },
                _ => {}
            }
        }
    }
    
    component_files
}

fn analyze_single_component(file_path: &Path, content: &str, line_count: usize) -> Option<ComponentAnalysis> {
    let framework = detect_framework_from_content(content);
    let component_type = detect_component_type(content, &framework);
    let component_name = extract_component_name(file_path, content, &framework);
    
    let complexity_score = calculate_complexity_score(content, &framework);
    let issues = detect_component_issues(content, line_count, &framework);
    let refactor_suggestions = generate_refactor_suggestions(&issues, &framework, line_count);
    let extractable_parts = find_extractable_parts(content, &framework);
    
    Some(ComponentAnalysis {
        file_path: file_path.to_string_lossy().to_string(),
        component_name,
        component_type,
        framework,
        line_count,
        complexity_score,
        issues,
        refactor_suggestions,
        extractable_parts,
    })
}

fn detect_framework_from_content(content: &str) -> Framework {
    if content.contains("import React") || content.contains("from 'react'") || content.contains("from \"react\"") {
        Framework::React
    } else if content.contains("<template>") && content.contains("<script") {
        Framework::Vue
    } else if content.contains("@Component") || content.contains("@angular/") {
        Framework::Angular
    } else if content.contains("<script>") && (content.contains("export default") || content.contains("let ")) {
        Framework::Svelte
    } else {
        Framework::Unknown
    }
}

fn detect_component_type(content: &str, framework: &Framework) -> ComponentType {
    match framework {
        Framework::React => {
            if content.contains("class ") && content.contains("extends") && content.contains("Component") {
                ComponentType::ClassComponent
            } else {
                ComponentType::FunctionalComponent
            }
        },
        Framework::Vue => ComponentType::VueComponent,
        Framework::Angular => ComponentType::AngularComponent,
        Framework::Svelte => ComponentType::SvelteComponent,
        Framework::Unknown => ComponentType::FunctionalComponent,
    }
}

fn extract_component_name(file_path: &Path, content: &str, framework: &Framework) -> String {
    // Try to extract from file name first
    if let Some(file_name) = file_path.file_stem().and_then(|s| s.to_str()) {
        if file_name != "index" {
            return file_name.to_string();
        }
    }
    
    // Try to extract from content based on framework
    match framework {
        Framework::React => {
            // Look for function component or class component names
            let patterns = [
                r"function\s+([A-Z][a-zA-Z0-9]*)",
                r"const\s+([A-Z][a-zA-Z0-9]*)\s*=",
                r"class\s+([A-Z][a-zA-Z0-9]*)",
            ];
            
            for pattern in &patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if let Some(captures) = re.captures(content) {
                        if let Some(name) = captures.get(1) {
                            return name.as_str().to_string();
                        }
                    }
                }
            }
        },
        Framework::Vue => {
            // Look for name property in Vue component
            if let Ok(re) = Regex::new(r#"name:\s*['"]([^'"]+)['"]"#) {
                if let Some(captures) = re.captures(content) {
                    if let Some(name) = captures.get(1) {
                        return name.as_str().to_string();
                    }
                }
            }
        },
        _ => {}
    }
    
    // Fallback to file name
    file_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Component")
        .to_string()
}

fn calculate_complexity_score(content: &str, framework: &Framework) -> u32 {
    let mut score = 0u32;
    
    // Count various complexity indicators
    let hooks_count = count_react_hooks(content);
    let props_count = count_props(content, framework);
    let state_vars_count = count_state_variables(content, framework);
    let conditional_count = count_conditionals(content);
    let loop_count = count_loops(content);
    let function_count = count_internal_functions(content);
    
    score += hooks_count * 2;
    score += props_count;
    score += state_vars_count * 2;
    score += conditional_count;
    score += loop_count * 2;
    score += function_count;
    
    score
}

fn count_react_hooks(content: &str) -> u32 {
    let hook_patterns = [
        r"useState\s*\(",
        r"useEffect\s*\(",
        r"useContext\s*\(",
        r"useReducer\s*\(",
        r"useCallback\s*\(",
        r"useMemo\s*\(",
        r"useRef\s*\(",
        r"use[A-Z][a-zA-Z]*\s*\(",  // Custom hooks
    ];
    
    let mut count = 0;
    for pattern in &hook_patterns {
        if let Ok(re) = Regex::new(pattern) {
            count += re.find_iter(content).count() as u32;
        }
    }
    count
}

fn count_props(content: &str, framework: &Framework) -> u32 {
    match framework {
        Framework::React => {
            // Count destructured props and props usage
            let patterns = [
                r"\{\s*([^}]+)\s*\}\s*=\s*props",  // { prop1, prop2 } = props
                r"props\.([a-zA-Z_][a-zA-Z0-9_]*)", // props.something
            ];
            
            let mut props = std::collections::HashSet::new();
            for pattern in &patterns {
                if let Ok(re) = Regex::new(pattern) {
                    for cap in re.captures_iter(content) {
                        if let Some(match_str) = cap.get(1) {
                            // Split by comma for destructured props
                            for prop in match_str.as_str().split(',') {
                                props.insert(prop.trim().to_string());
                            }
                        }
                    }
                }
            }
            props.len() as u32
        },
        _ => 0, // TODO: Implement for other frameworks
    }
}

fn count_state_variables(content: &str, framework: &Framework) -> u32 {
    match framework {
        Framework::React => {
            if let Ok(re) = Regex::new(r"useState\s*\(") {
                re.find_iter(content).count() as u32
            } else {
                0
            }
        },
        _ => 0, // TODO: Implement for other frameworks
    }
}

fn count_conditionals(content: &str) -> u32 {
    let patterns = [
        r"if\s*\(",
        r"\?\s*[^:]+\s*:",  // Ternary operators
        r"&&\s*[^&]",       // Logical AND in JSX
    ];
    
    let mut count = 0;
    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            count += re.find_iter(content).count() as u32;
        }
    }
    count
}

fn count_loops(content: &str) -> u32 {
    let patterns = [
        r"for\s*\(",
        r"while\s*\(",
        r"\.map\s*\(",
        r"\.forEach\s*\(",
        r"\.filter\s*\(",
        r"\.reduce\s*\(",
    ];
    
    let mut count = 0;
    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            count += re.find_iter(content).count() as u32;
        }
    }
    count
}

fn count_internal_functions(content: &str) -> u32 {
    let patterns = [
        r"const\s+[a-zA-Z_][a-zA-Z0-9_]*\s*=\s*\(",  // const func = (
        r"function\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\(",    // function func(
        r"const\s+[a-zA-Z_][a-zA-Z0-9_]*\s*=\s*async", // const func = async
    ];
    
    let mut count = 0;
    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            count += re.find_iter(content).count() as u32;
        }
    }
    count
}

fn detect_component_issues(content: &str, line_count: usize, framework: &Framework) -> Vec<ComponentIssue> {
    let mut issues = Vec::new();
    
    // Check line count
    if line_count > 200 {
        issues.push(ComponentIssue {
            issue_type: IssueType::TooManyLines,
            line_number: 1,
            description: format!("Component has {} lines (>200 is critical)", line_count),
            severity: IssueSeverity::Critical,
        });
    } else if line_count > 100 {
        issues.push(ComponentIssue {
            issue_type: IssueType::TooManyLines,
            line_number: 1,
            description: format!("Component has {} lines (>100 needs refactoring)", line_count),
            severity: IssueSeverity::Error,
        });
    }
    
    // Check React-specific issues
    if matches!(framework, Framework::React) {
        let hooks_count = count_react_hooks(content);
        if hooks_count > 10 {
            issues.push(ComponentIssue {
                issue_type: IssueType::TooManyHooks,
                line_number: 1,
                description: format!("Component uses {} hooks (>10 is too many)", hooks_count),
                severity: IssueSeverity::Error,
            });
        }
        
        let props_count = count_props(content, framework);
        if props_count > 8 {
            issues.push(ComponentIssue {
                issue_type: IssueType::TooManyProps,
                line_number: 1,
                description: format!("Component has {} props (>8 suggests multiple concerns)", props_count),
                severity: IssueSeverity::Warning,
            });
        }
    }
    
    // Check for deep nesting
    let max_indent = find_max_indentation(content);
    if max_indent > 6 {
        issues.push(ComponentIssue {
            issue_type: IssueType::DeepNesting,
            line_number: 1,
            description: format!("Deep nesting detected ({} levels)", max_indent),
            severity: IssueSeverity::Warning,
        });
    }
    
    issues
}

fn find_max_indentation(content: &str) -> usize {
    content
        .lines()
        .map(|line| {
            let spaces = line.chars().take_while(|c| c.is_whitespace()).count();
            spaces / 2 // Assume 2-space indentation
        })
        .max()
        .unwrap_or(0)
}

fn generate_refactor_suggestions(issues: &[ComponentIssue], framework: &Framework, line_count: usize) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    for issue in issues {
        match issue.issue_type {
            IssueType::TooManyLines => {
                if line_count > 200 {
                    suggestions.push("🚨 CRITICAL: Split this component into 3-4 smaller components".to_string());
                    suggestions.push("📦 Extract reusable UI components".to_string());
                    suggestions.push("🔧 Move business logic to custom hooks or utilities".to_string());
                } else {
                    suggestions.push("⚠️ Consider splitting into 2-3 smaller components".to_string());
                    suggestions.push("🎯 Extract complex logic into separate functions".to_string());
                }
            },
            IssueType::TooManyHooks => {
                suggestions.push("🪝 Extract related hooks into custom hooks".to_string());
                suggestions.push("📋 Group useState calls into useReducer if managing related state".to_string());
            },
            IssueType::TooManyProps => {
                suggestions.push("📦 Group related props into objects".to_string());
                suggestions.push("🎯 Consider if this component has too many responsibilities".to_string());
            },
            IssueType::DeepNesting => {
                suggestions.push("📏 Extract nested logic into separate components".to_string());
                suggestions.push("🔄 Use early returns to reduce nesting".to_string());
            },
            _ => {}
        }
    }
    
    // Framework-specific suggestions
    match framework {
        Framework::React => {
            suggestions.push("⚛️ Consider using React.memo for performance optimization".to_string());
            suggestions.push("🪝 Extract business logic into custom hooks".to_string());
        },
        Framework::Vue => {
            suggestions.push("🎯 Use Vue composition API for better logic organization".to_string());
        },
        _ => {}
    }
    
    suggestions
}

fn find_extractable_parts(content: &str, framework: &Framework) -> Vec<ExtractablePart> {
    let mut parts = Vec::new();
    
    match framework {
        Framework::React => {
            // Find potential custom hooks
            if let Ok(re) = Regex::new(r"(useState|useEffect|useCallback|useMemo)\s*\([^)]*\)") {
                for (i, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        // Look for groups of hooks that could be extracted
                        parts.push(ExtractablePart {
                            name: "CustomHook".to_string(),
                            part_type: ExtractableType::CustomHook,
                            start_line: i + 1,
                            end_line: i + 5, // Approximate
                            suggested_filename: "useCustomHook.ts".to_string(),
                            description: "Extract related hooks into a custom hook".to_string(),
                        });
                    }
                }
            }
            
            // Find utility functions
            if let Ok(re) = Regex::new(r"const\s+([a-z][a-zA-Z0-9]*)\s*=\s*\([^)]*\)\s*=>\s*\{") {
                for (i, line) in content.lines().enumerate() {
                    if let Some(cap) = re.captures(line) {
                        if let Some(func_name) = cap.get(1) {
                            parts.push(ExtractablePart {
                                name: func_name.as_str().to_string(),
                                part_type: ExtractableType::UtilityFunction,
                                start_line: i + 1,
                                end_line: i + 10, // Approximate
                                suggested_filename: format!("{}.utils.ts", func_name.as_str()),
                                description: format!("Extract {} utility function", func_name.as_str()),
                            });
                        }
                    }
                }
            }
        },
        _ => {}
    }
    
    parts
}

fn create_component_summary(components: &[ComponentAnalysis]) -> ComponentSummary {
    let total_components = components.len();
    let large_components = components.iter().filter(|c| c.line_count > 100).count();
    let complex_components = components.iter().filter(|c| c.complexity_score > 20).count();
    let components_needing_refactor = components.iter().filter(|c| {
        c.issues.iter().any(|issue| matches!(issue.severity, IssueSeverity::Error | IssueSeverity::Critical))
    }).count();
    let potential_extractions = components.iter().map(|c| c.extractable_parts.len()).sum();
    
    ComponentSummary {
        total_components,
        large_components,
        complex_components,
        components_needing_refactor,
        potential_extractions,
    }
}

fn generate_global_recommendations(components: &[ComponentAnalysis]) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    let avg_lines = if !components.is_empty() {
        components.iter().map(|c| c.line_count).sum::<usize>() / components.len()
    } else {
        0
    };
    
    if avg_lines > 150 {
        recommendations.push("📊 Your components average over 150 lines - consider adopting a component splitting strategy".to_string());
    }
    
    let react_components = components.iter().filter(|c| matches!(c.framework, Framework::React)).count();
    if react_components > 0 {
        recommendations.push("⚛️ For React components: Extract custom hooks for reusable logic".to_string());
        recommendations.push("🎯 Use component composition over large monolithic components".to_string());
    }
    
    let vue_components = components.iter().filter(|c| matches!(c.framework, Framework::Vue)).count();
    if vue_components > 0 {
        recommendations.push("🎯 For Vue components: Use composition API for better code organization".to_string());
    }
    
    recommendations.push("📦 Consider creating a design system for reusable UI components".to_string());
    recommendations.push("🔧 Use linting rules to enforce component size limits".to_string());
    
    recommendations
}

fn print_component_report(report: &ComponentReport, config: &Config, quiet: bool) {
    if !quiet {
        println!();
        println!("{}", "🧩 Component Analysis Report".bold().blue());
        println!("{}", "==========================".blue());
        println!();
    }
    
    if report.components.is_empty() {
        println!("{}", "✅ No large components found! Your components are well-sized.".green());
        return;
    }
    
    // Print components that need attention
    let mut critical_components = Vec::new();
    let mut error_components = Vec::new();
    let mut warning_components = Vec::new();
    
    for component in &report.components {
        let has_critical = component.issues.iter().any(|i| matches!(i.severity, IssueSeverity::Critical));
        let has_error = component.issues.iter().any(|i| matches!(i.severity, IssueSeverity::Error));
        
        if has_critical {
            critical_components.push(component);
        } else if has_error {
            error_components.push(component);
        } else {
            warning_components.push(component);
        }
    }
    
    // Print critical components first
    for component in critical_components {
        print_component_analysis(component, "critical");
    }
    
    // Print error components
    for component in error_components {
        print_component_analysis(component, "error");
    }
    
    // Print warning components  
    for component in warning_components {
        print_component_analysis(component, "warning");
    }
    
    // Print summary
    print_component_summary(&report.summary, config);
    
    // Print recommendations
    if !report.recommendations.is_empty() {
        println!("{}", "💡 RECOMMENDATIONS".bold().yellow());
        println!("{}", "─────────────────".yellow());
        for recommendation in &report.recommendations {
            println!("  {}", recommendation);
        }
        println!();
    }
}

fn print_component_analysis(component: &ComponentAnalysis, severity: &str) {
    let (emoji, color) = match severity {
        "critical" => ("🚨", "red"),
        "error" => ("⚠️", "yellow"),  
        "warning" => ("💡", "cyan"),
        _ => ("📄", "white"),
    };
    
    println!("{} {}: {} ({} lines, complexity: {})", 
        emoji,
        match color {
            "red" => severity.red(),
            "yellow" => severity.yellow(), 
            "cyan" => severity.cyan(),
            _ => severity.white(),
        },
        component.component_name.bold(),
        component.line_count,
        component.complexity_score
    );
    
    println!("   📁 {}", component.file_path.dimmed());
    println!("   🏗️  {} {:?} component", 
        format!("{:?}", component.framework).dimmed(),
        component.component_type
    );
    
    // Print issues
    for issue in &component.issues {
        let issue_icon = match issue.severity {
            IssueSeverity::Critical => "🚨",
            IssueSeverity::Error => "❌",
            IssueSeverity::Warning => "⚠️",
        };
        println!("   {} {}", issue_icon, issue.description);
    }
    
    // Print refactor suggestions
    if !component.refactor_suggestions.is_empty() {
        println!("   💡 Refactor suggestions:");
        for suggestion in &component.refactor_suggestions {
            println!("     • {}", suggestion);
        }
    }
    
    // Print extractable parts
    if !component.extractable_parts.is_empty() {
        println!("   📦 Extractable parts:");
        for part in &component.extractable_parts {
            println!("     • {} → {}", part.description, part.suggested_filename.bold());
        }
    }
    
    println!();
}

fn print_component_summary(summary: &ComponentSummary, config: &Config) {
    println!("{}", "📈 SUMMARY".bold().white());
    println!("{}", "─────────".white());
    println!("  Components analyzed: {}", summary.total_components);
    println!("  Large components (>100 lines): {}", 
        if summary.large_components > 0 { 
            summary.large_components.to_string().yellow() 
        } else { 
            summary.large_components.to_string().green() 
        }
    );
    println!("  Complex components (high complexity): {}", 
        if summary.complex_components > 0 { 
            summary.complex_components.to_string().yellow() 
        } else { 
            summary.complex_components.to_string().green() 
        }
    );
    println!("  Components needing refactor: {}", 
        if summary.components_needing_refactor > 0 { 
            summary.components_needing_refactor.to_string().red() 
        } else { 
            summary.components_needing_refactor.to_string().green() 
        }
    );
    println!("  Potential extractions found: {}", summary.potential_extractions);
    
    println!();
    let threshold = config.large_files.severity_levels.warning;
    println!("{}", format!("💡 TIP: Keep components under {} lines for better maintainability", threshold).dimmed());
}