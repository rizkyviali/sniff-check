/// Common error handling utilities

/// Standard error codes for different failure types
#[allow(dead_code)]
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    ValidationFailed = 2,
    ThresholdExceeded = 3,
    ConfigurationError = 4,
}

/// Check if issues exceed failure thresholds and exit appropriately
pub fn check_failure_threshold(has_critical_issues: bool, exit_code: ExitCode) {
    if has_critical_issues {
        std::process::exit(exit_code as i32);
    }
}

