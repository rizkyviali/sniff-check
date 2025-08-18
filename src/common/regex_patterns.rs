// Shared regex patterns used across multiple commands

use regex::Regex;
use std::sync::OnceLock;

/// Container for commonly used regex patterns
pub struct CommonPatterns {
    pub any_type: Regex,
    pub function_def: Regex,
    pub ts_ignore: Regex,
    pub ts_expect_error: Regex,
    pub import_statement: Regex,
    pub named_import: Regex,
    pub default_import: Regex,
    pub event_listener: Regex,
    pub timer_function: Regex,
    pub array_push: Regex,
    pub infinite_loop: Regex,
    pub closure_pattern: Regex,
}

static COMMON_PATTERNS: OnceLock<CommonPatterns> = OnceLock::new();

impl CommonPatterns {
    fn new() -> Result<Self, regex::Error> {
        Ok(Self {
            any_type: Regex::new(r"\b:\s*any\b")?,
            function_def: Regex::new(r"(?:function\s+\w+|const\s+\w+\s*=\s*(?:async\s+)?\([^)]*\)\s*=>|(?:async\s+)?function\s*\([^)]*\))\s*\{")?,
            ts_ignore: Regex::new(r"@ts-ignore")?,
            ts_expect_error: Regex::new(r"@ts-expect-error")?,
            import_statement: Regex::new(r#"^import\s+(.+?)\s+from\s+['"](.+?)['"];?\s*$"#)?,
            named_import: Regex::new(r"import\s*\{\s*([^}]+)\s*\}")?,
            default_import: Regex::new(r"import\s+(\w+)\s+from")?,
            event_listener: Regex::new(r"addEventListener\([^)]+\)")?,
            timer_function: Regex::new(r"set(?:Interval|Timeout)\([^)]+\)")?,
            array_push: Regex::new(r"\w+\.push\([^)]+\)")?,
            infinite_loop: Regex::new(r"while\s*\(\s*true\s*\)")?,
            closure_pattern: Regex::new(r"function[^{]*\{[\s\S]*function[^{]*\{[\s\S]*\}[\s\S]*\}")?,
        })
    }
}

/// Get the global instance of common regex patterns
pub fn get_common_patterns() -> &'static CommonPatterns {
    COMMON_PATTERNS.get_or_init(|| {
        CommonPatterns::new().expect("Failed to compile common regex patterns")
    })
}

/// Check if a string contains keywords or built-in identifiers that should be ignored
pub fn is_keyword_or_builtin(identifier: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        // JavaScript/TypeScript keywords
        "const", "let", "var", "function", "class", "interface", "type", "enum",
        "if", "else", "for", "while", "do", "switch", "case", "default",
        "return", "break", "continue", "throw", "try", "catch", "finally",
        "import", "export", "from", "as", "async", "await", "yield",
        "true", "false", "null", "undefined", "this", "super",
        
        // Common globals
        "console", "window", "document", "process", "require", "module",
        
        // React/common hooks
        "React", "Component", "useState", "useEffect", "useContext",
        
        // TypeScript types
        "string", "number", "boolean", "object", "any", "void", "never",
    ];
    
    KEYWORDS.contains(&identifier) || identifier.len() <= 2
}

/// Check if a line appears to be within a string literal or comment
pub fn is_in_string_literal_or_comment(line: &str) -> bool {
    let trimmed = line.trim();
    
    // Check for comments
    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
        return true;
    }
    
    // Check for string literals (simplified check)
    (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
    (trimmed.starts_with('\'') && trimmed.ends_with('\'')) ||
    (trimmed.starts_with('`') && trimmed.ends_with('`')) ||
    trimmed.contains("console.log") ||
    trimmed.contains("console.error") ||
    trimmed.contains("console.warn")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_patterns_compilation() {
        let patterns = get_common_patterns();
        assert!(patterns.any_type.is_match("param: any"));
        assert!(patterns.import_statement.is_match("import React from 'react';"));
    }

    #[test]
    fn test_keyword_detection() {
        assert!(is_keyword_or_builtin("const"));
        assert!(is_keyword_or_builtin("React"));
        assert!(!is_keyword_or_builtin("MyComponent"));
    }

    #[test]
    fn test_string_literal_detection() {
        assert!(is_in_string_literal_or_comment("// This is a comment"));
        assert!(is_in_string_literal_or_comment("console.log('test')"));
        assert!(!is_in_string_literal_or_comment("const x = 5;"));
    }
}