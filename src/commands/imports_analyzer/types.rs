use serde::{Deserialize, Serialize};

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

#[derive(Debug)]
pub struct ParsedImport {
    pub import_type: ImportType,
    pub default_import: Option<String>,
    pub named_imports: Vec<String>,
    pub namespace_import: Option<String>,
}

pub struct FileAnalysis {
    pub total_imports: usize,
    pub unused_imports: Vec<UnusedImport>,
    pub broken_imports: Vec<BrokenImport>,
}