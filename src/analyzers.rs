// Analyzer utilities for different file types and patterns

use regex::Regex;
use std::collections::HashMap;

/// File type analyzer
pub struct FileAnalyzer;

impl FileAnalyzer {
    /// Determine file type based on path and content
    pub fn analyze_file_type(path: &str, content: &str) -> FileType {
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // Check for specific patterns in filename
        if filename.contains(".component.") || filename.contains(".comp.") {
            return FileType::Component;
        }
        
        if filename.contains(".service.") || filename.contains(".svc.") {
            return FileType::Service;
        }
        
        if filename.contains(".util.") || filename.contains(".helper.") || path.contains("/utils/") || path.contains("/helpers/") {
            return FileType::Util;
        }
        
        if path.contains("/api/") || filename.contains(".api.") {
            return FileType::Api;
        }
        
        if path.contains("/pages/") || path.contains("/page.") {
            return FileType::Page;
        }
        
        // Analyze content patterns
        if content.contains("export default function") || content.contains("export const") && content.contains("FC<") {
            return FileType::Component;
        }
        
        if content.contains("class") && content.contains("Service") {
            return FileType::Service;
        }
        
        if content.contains("export") && (content.contains("function") || content.contains("const")) {
            return FileType::Util;
        }
        
        FileType::Unknown
    }
    
    /// Get refactoring suggestions based on file type and size
    pub fn get_refactoring_suggestions(file_type: &FileType, line_count: usize) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        match file_type {
            FileType::Component => {
                suggestions.push("Break into smaller sub-components".to_string());
                suggestions.push("Extract custom hooks for logic".to_string());
                suggestions.push("Move utility functions to separate files".to_string());
                if line_count > 300 {
                    suggestions.push("Consider using compound component pattern".to_string());
                }
            }
            FileType::Service => {
                suggestions.push("Split into multiple service classes".to_string());
                suggestions.push("Extract interfaces and types".to_string());
                suggestions.push("Use dependency injection".to_string());
                if line_count > 400 {
                    suggestions.push("Apply single responsibility principle".to_string());
                }
            }
            FileType::Api => {
                suggestions.push("Split endpoints into separate files".to_string());
                suggestions.push("Extract validation logic".to_string());
                suggestions.push("Move business logic to services".to_string());
            }
            FileType::Page => {
                suggestions.push("Extract components from page logic".to_string());
                suggestions.push("Move data fetching to separate hooks".to_string());
                suggestions.push("Split into layout and content components".to_string());
            }
            FileType::Util => {
                suggestions.push("Group related functions into modules".to_string());
                suggestions.push("Extract constants to separate file".to_string());
                suggestions.push("Split by functionality domain".to_string());
            }
            FileType::Unknown => {
                suggestions.push("Consider breaking into smaller modules".to_string());
                suggestions.push("Extract reusable functions".to_string());
                suggestions.push("Organize by single responsibility".to_string());
            }
        }
        
        suggestions
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Component,
    Service,
    Api,
    Page,
    Util,
    Unknown,
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Component => write!(f, "Component"),
            FileType::Service => write!(f, "Service"),
            FileType::Api => write!(f, "API"),
            FileType::Page => write!(f, "Page"),
            FileType::Util => write!(f, "Util"),
            FileType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Pattern analyzer for code quality
pub struct PatternAnalyzer;

impl PatternAnalyzer {
    /// Analyze TypeScript patterns in content
    pub fn analyze_typescript_patterns(content: &str) -> TypeScriptAnalysis {
        let any_pattern = Regex::new(r"\b(any)\b").unwrap();
        let ts_ignore_pattern = Regex::new(r"@ts-ignore|@ts-expect-error").unwrap();
        let function_pattern = Regex::new(r"function\s+\w+\s*\([^)]*\)\s*\{").unwrap();
        let arrow_function_pattern = Regex::new(r"const\s+\w+\s*=\s*\([^)]*\)\s*=>\s*\{").unwrap();
        let return_type_pattern = Regex::new(r":\s*\w+\s*=>|\):\s*\w+\s*\{").unwrap();
        
        let any_count = any_pattern.find_iter(content).count();
        let ts_suppressions = ts_ignore_pattern.find_iter(content).count();
        let total_functions = function_pattern.find_iter(content).count() + arrow_function_pattern.find_iter(content).count();
        let typed_functions = return_type_pattern.find_iter(content).count();
        
        let type_coverage = if total_functions > 0 {
            (typed_functions as f64 / total_functions as f64) * 100.0
        } else {
            100.0
        };
        
        TypeScriptAnalysis {
            any_usage_count: any_count,
            ts_suppressions,
            total_functions,
            typed_functions,
            type_coverage_percentage: type_coverage,
        }
    }
    
    /// Analyze import patterns
    pub fn analyze_import_patterns(content: &str) -> ImportAnalysis {
        let import_pattern = Regex::new(r#"import\s+.*?\s+from\s+['"]([^'"]+)['"]"#).unwrap();
        let named_import_pattern = Regex::new(r"import\s*\{\s*([^}]+)\s*\}").unwrap();
        let default_import_pattern = Regex::new(r"import\s+(\w+)\s+from").unwrap();
        
        let mut imports = Vec::new();
        let mut named_imports = HashMap::new();
        let mut default_imports = Vec::new();
        
        // Extract all imports
        for cap in import_pattern.captures_iter(content) {
            imports.push(cap[1].to_string());
        }
        
        // Extract named imports
        for cap in named_import_pattern.captures_iter(content) {
            let names: Vec<String> = cap[1]
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            for name in names {
                *named_imports.entry(name).or_insert(0) += 1;
            }
        }
        
        // Extract default imports  
        for cap in default_import_pattern.captures_iter(content) {
            default_imports.push(cap[1].to_string());
        }
        
        ImportAnalysis {
            total_imports: imports.len(),
            named_imports,
            default_imports,
            import_paths: imports,
        }
    }
}

#[derive(Debug)]
pub struct TypeScriptAnalysis {
    pub any_usage_count: usize,
    pub ts_suppressions: usize,
    pub total_functions: usize,
    pub typed_functions: usize,
    pub type_coverage_percentage: f64,
}

#[derive(Debug)]
pub struct ImportAnalysis {
    pub total_imports: usize,
    pub named_imports: HashMap<String, usize>,
    pub default_imports: Vec<String>,
    pub import_paths: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_file_type_detection() {
        let component_content = "export default function MyComponent() { return <div />; }";
        let file_type = FileAnalyzer::analyze_file_type("src/components/MyComponent.tsx", component_content);
        assert_eq!(file_type, FileType::Component);
        
        let service_content = "export class UserService { getUser() {} }";
        let file_type = FileAnalyzer::analyze_file_type("src/services/user.service.ts", service_content);
        assert_eq!(file_type, FileType::Service);
    }
    
    #[test]
    fn test_typescript_analysis() {
        let content = r#"
            function test(param: any): void {}
            const arrow = (x: number): string => "test";
            // @ts-ignore
            let bad: any = {};
        "#;
        
        let analysis = PatternAnalyzer::analyze_typescript_patterns(content);
        assert_eq!(analysis.any_usage_count, 2);
        assert_eq!(analysis.ts_suppressions, 1);
    }
    
    #[test]
    fn test_import_analysis() {
        let content = r#"
            import React from 'react';
            import { useState, useEffect } from 'react';
            import utils from './utils';
        "#;
        
        let analysis = PatternAnalyzer::analyze_import_patterns(content);
        assert_eq!(analysis.total_imports, 3);
        assert_eq!(analysis.default_imports.len(), 2);
    }
}