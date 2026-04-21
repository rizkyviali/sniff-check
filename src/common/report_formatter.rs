// Common report formatting utilities

use serde::{Deserialize, Serialize};

/// Common severity levels used across different analysis types
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}
