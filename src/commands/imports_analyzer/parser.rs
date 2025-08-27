use anyhow::Result;
use std::collections::HashSet;

use super::types::{ImportType, ParsedImport};
use crate::common::get_common_patterns;

fn is_keyword_or_builtin(identifier: &str) -> bool {
    matches!(identifier,
        // JavaScript keywords
        "abstract" | "as" | "async" | "await" | "break" | "case" | "catch" | "class" |
        "const" | "continue" | "debugger" | "default" | "delete" | "do" | "else" |
        "enum" | "export" | "extends" | "false" | "finally" | "for" | "from" |
        "function" | "get" | "if" | "implements" | "import" | "in" | "instanceof" |
        "interface" | "let" | "new" | "null" | "of" | "package" | "private" |
        "protected" | "public" | "return" | "set" | "static" | "super" | "switch" |
        "this" | "throw" | "true" | "try" | "typeof" | "var" | "void" | "while" |
        "with" | "yield" |
        // TypeScript keywords
        "any" | "boolean" | "declare" | "infer" | "is" | "keyof" | "module" |
        "namespace" | "never" | "number" | "object" | "readonly" | "require" |
        "string" | "symbol" | "type" | "undefined" | "unique" | "unknown" |
        // Common globals
        "console" | "window" | "document" | "global" | "process" | "Buffer" |
        "setTimeout" | "setInterval" | "clearTimeout" | "clearInterval" |
        "JSON" | "Math" | "Date" | "Error" | "RegExp" | "Array" | "Object" |
        "String" | "Number" | "Boolean" | "Symbol" | "BigInt" | "Promise" |
        // Node.js globals (excluding duplicates)
        "__dirname" | "__filename" | "exports" |
        // Common test framework globals
        "describe" | "it" | "test" | "expect" | "beforeEach" | "afterEach" |
        "beforeAll" | "afterAll" | "jest" | "jasmine" | "mocha"
    )
}

pub fn parse_import_statement(import_spec: &str, _module_path: &str) -> ParsedImport {
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

pub fn find_unused_items(parsed_import: &ParsedImport, used_identifiers: &HashSet<String>) -> Vec<String> {
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

pub fn collect_used_identifiers(lines: &[&str]) -> Result<HashSet<String>> {
    let mut used_identifiers = HashSet::new();
    
    let patterns = get_common_patterns();
    
    // Comprehensive usage detection patterns
    let general_usage = regex::Regex::new(r"\b([A-Z][a-zA-Z0-9_]*|[a-z][a-zA-Z0-9_]*)\b")?;
    let react_hook_usage = regex::Regex::new(r"const\s*\[([^,\]]+),\s*([^\]]+)\]\s*=\s*(use[A-Z]\w*)")?;
    let type_annotation = regex::Regex::new(r":\s*([A-Z][a-zA-Z0-9_<>,\s\[\]]*)")?;
    let generic_usage = regex::Regex::new(r"<([A-Z][a-zA-Z0-9_<>,\s\[\]]*?)>")?;
    let jsx_usage = regex::Regex::new(r"</?([A-Z][a-zA-Z0-9_.]*)")?;
    let interface_extends = regex::Regex::new(r"(?:extends|implements)\s+([A-Z][a-zA-Z0-9_<>,\s]*)")?;
    let function_param_type = regex::Regex::new(r"\(\s*[^:)]*:\s*([A-Z][a-zA-Z0-9_<>,\s\[\]]*)")?;
    
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
    
    Ok(used_identifiers)
}