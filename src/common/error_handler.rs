/// Common error handling utilities
use anyhow::Result;

/// Standard error codes for different failure types
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    ValidationFailed = 2,
    ThresholdExceeded = 3,
    ConfigurationError = 4,
}

/// Handle command execution results with appropriate exit codes
pub fn handle_command_result(result: Result<()>, exit_code: ExitCode) {
    if result.is_err() {
        std::process::exit(exit_code as i32);
    }
}

/// Check if issues exceed failure thresholds and exit appropriately
pub fn check_failure_threshold(has_critical_issues: bool, exit_code: ExitCode) {
    if has_critical_issues {
        std::process::exit(exit_code as i32);
    }
}

/// Standardized error reporting
pub fn report_error(error: &anyhow::Error) {
    eprintln!("Error: {}", error);
}

/// Validation error helper - commonly used pattern
pub fn validation_failed(message: &str) -> anyhow::Error {
    anyhow::anyhow!("Validation failed: {}", message)
}

/// File operation error helper - commonly used pattern  
pub fn file_error(operation: &str, path: &str, source: std::io::Error) -> anyhow::Error {
    anyhow::anyhow!("Failed to {} file '{}': {}", operation, path, source)
}