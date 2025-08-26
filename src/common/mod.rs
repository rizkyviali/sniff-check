// Common utilities shared across commands

pub mod file_scanner;
pub mod regex_patterns;
pub mod report_formatter;
pub mod progress;
pub mod error_handler;
pub mod cli_args;
pub mod output_utils;
pub mod json_output;
pub mod performance;

pub use file_scanner::{FileScanner};
pub use regex_patterns::{get_common_patterns, is_in_string_literal_or_comment, is_keyword_or_builtin};
pub use report_formatter::Severity;
pub use error_handler::{ExitCode, check_failure_threshold};
pub use output_utils::{init_command, complete_command};
pub use json_output::{create_standard_json_output, output_result};
pub use performance::{OptimizedFileWalker, count_lines_optimized, PerformanceMonitor};
// progress module exports removed as unused