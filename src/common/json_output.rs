/// Unified JSON output formatting utilities
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Standard JSON response wrapper for all commands
#[derive(Serialize, Deserialize)]
pub struct StandardResponse<T> {
    /// The command that generated this response
    pub command: String,
    /// Timestamp when the analysis was performed
    pub timestamp: DateTime<Utc>,
    /// Version of the tool that generated this response
    pub version: String,
    /// The actual command-specific data
    pub data: T,
    /// Summary information for quick overview
    pub summary: ResponseSummary,
    /// Any warnings or metadata
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Common summary information across all responses
#[derive(Serialize, Deserialize)]
pub struct ResponseSummary {
    /// Total number of items analyzed
    pub total_items: usize,
    /// Number of issues found
    pub issues_found: usize,
    /// Overall status of the analysis
    pub status: AnalysisStatus,
    /// Duration of the analysis in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// Standard analysis status across all commands
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisStatus {
    Success,
    Warning,
    Error,
    Failed,
}

impl<T> StandardResponse<T> 
where 
    T: Serialize 
{
    /// Create a new standard response
    pub fn new(command: &str, data: T, summary: ResponseSummary) -> Self {
        Self {
            command: command.to_string(),
            timestamp: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            data,
            summary,
            warnings: Vec::new(),
            metadata: None,
        }
    }
    
    /// Add a warning to the response
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
    
    /// Add warnings to the response
    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings.extend(warnings);
        self
    }
    
    /// Add metadata to the response
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        if self.metadata.is_none() {
            self.metadata = Some(std::collections::HashMap::new());
        }
        self.metadata.as_mut().unwrap().insert(key, value);
        self
    }
    
    /// Convert to pretty JSON string
    pub fn to_json_pretty(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
    
    /// Convert to compact JSON string
    pub fn to_json_compact(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

impl AnalysisStatus {
    /// Determine status based on issue count and thresholds
    pub fn from_issues(issues_found: usize, warning_threshold: usize, error_threshold: usize) -> Self {
        if issues_found == 0 {
            Self::Success
        } else if issues_found <= warning_threshold {
            Self::Warning
        } else if issues_found <= error_threshold {
            Self::Error
        } else {
            Self::Failed
        }
    }
    
    /// Simple status based on whether issues were found
    pub fn from_has_issues(has_issues: bool) -> Self {
        if has_issues {
            Self::Warning
        } else {
            Self::Success
        }
    }
}

/// Helper function to create a standard JSON output
pub fn create_standard_json_output<T>(
    command: &str,
    data: T,
    total_items: usize,
    issues_found: usize,
    duration_ms: Option<u64>,
) -> StandardResponse<T>
where
    T: Serialize,
{
    let status = AnalysisStatus::from_has_issues(issues_found > 0);
    let summary = ResponseSummary {
        total_items,
        issues_found,
        status,
        duration_ms,
    };
    
    StandardResponse::new(command, data, summary)
}

/// Helper function to output either JSON or call a custom print function
pub fn output_result<T>(
    response: &StandardResponse<T>,
    json: bool,
    quiet: bool,
    print_fn: impl Fn(&T, bool),
) -> anyhow::Result<()>
where
    T: Serialize,
{
    if json {
        println!("{}", response.to_json_pretty()?);
    } else {
        print_fn(&response.data, quiet);
    }
    Ok(())
}