// Common report formatting utilities

use colored::*;
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

impl Severity {
    /// Get the colored string representation of the severity
    pub fn to_colored_string(&self) -> ColoredString {
        match self {
            Severity::Info => "INFO".blue(),
            Severity::Low => "LOW".cyan(),
            Severity::Medium => "MEDIUM".yellow(),
            Severity::High => "HIGH".yellow(),
            Severity::Critical => "CRITICAL".red(),
        }
    }

    /// Get the emoji icon for the severity
    pub fn to_icon(&self) -> &'static str {
        match self {
            Severity::Info => "ℹ️",
            Severity::Low => "⚡",
            Severity::Medium => "⚠️",
            Severity::High => "🔴",
            Severity::Critical => "🚨",
        }
    }
}

/// Common status used across different checks
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Status {
    Passed,
    Failed,
    Warning,
    Skipped,
}

impl Status {
    /// Get the colored string representation of the status
    pub fn to_colored_string(&self) -> ColoredString {
        match self {
            Status::Passed => "PASSED".green(),
            Status::Failed => "FAILED".red(),
            Status::Warning => "WARNING".yellow(),
            Status::Skipped => "SKIPPED".cyan(),
        }
    }

    /// Get the icon for the status
    pub fn to_icon(&self) -> &'static str {
        match self {
            Status::Passed => "✅",
            Status::Failed => "❌",
            Status::Warning => "⚠️",
            Status::Skipped => "⏭️",
        }
    }
}

/// Trait for objects that can generate summary information
pub trait Summarizable {
    type Summary;
    
    fn generate_summary(&self) -> Self::Summary;
}

/// Trait for objects that can be formatted as reports
pub trait ReportFormatter {
    fn print_report(&self, quiet: bool);
    fn to_json(&self) -> Result<String, serde_json::Error>;
}

/// Common summary statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct CommonSummary {
    pub total_items: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub skipped: usize,
    pub critical_issues: usize,
}

impl CommonSummary {
    pub fn new() -> Self {
        Self {
            total_items: 0,
            passed: 0,
            failed: 0,
            warnings: 0,
            skipped: 0,
            critical_issues: 0,
        }
    }

    pub fn add_result(&mut self, status: &Status, severity: Option<&Severity>) {
        self.total_items += 1;
        
        match status {
            Status::Passed => self.passed += 1,
            Status::Failed => self.failed += 1,
            Status::Warning => self.warnings += 1,
            Status::Skipped => self.skipped += 1,
        }

        if let Some(Severity::Critical) = severity {
            self.critical_issues += 1;
        }
    }

    pub fn is_successful(&self) -> bool {
        self.failed == 0 && self.critical_issues == 0
    }

    pub fn get_overall_status(&self) -> Status {
        if self.failed > 0 || self.critical_issues > 0 {
            Status::Failed
        } else if self.warnings > 0 {
            Status::Warning
        } else {
            Status::Passed
        }
    }
}

/// Utility functions for consistent formatting across reports
pub struct ReportUtils;

impl ReportUtils {
    /// Print a standard header for reports
    pub fn print_header(title: &str, icon: &str) {
        println!();
        println!("{}", format!("{} {}", icon, title).bold().blue());
        println!("{}", "=".repeat(title.len() + 4).blue());
        println!();
    }

    /// Print a section header
    pub fn print_section(title: &str, icon: &str) {
        println!("{}", format!("{} {}", icon, title).bold().white());
        println!("{}", "─".repeat(title.len() + 4).white());
    }

    /// Print a summary section
    pub fn print_summary(summary: &CommonSummary, duration_ms: Option<u64>) {
        println!("{}", "📈 SUMMARY".bold().white());
        println!("{}", "─────────".white());
        
        println!("  Total items: {}", summary.total_items);
        
        if summary.passed > 0 {
            println!("  {} {}", "Passed:".green(), summary.passed.to_string().green());
        }
        if summary.failed > 0 {
            println!("  {} {}", "Failed:".red(), summary.failed.to_string().red());
        }
        if summary.warnings > 0 {
            println!("  {} {}", "Warnings:".yellow(), summary.warnings.to_string().yellow());
        }
        if summary.skipped > 0 {
            println!("  {} {}", "Skipped:".cyan(), summary.skipped.to_string().cyan());
        }
        if summary.critical_issues > 0 {
            println!("  {} {}", "Critical:".red(), summary.critical_issues.to_string().red());
        }

        if let Some(duration) = duration_ms {
            println!("  Analysis time: {}ms", duration);
        }

        println!();
        
        let status = summary.get_overall_status();
        let status_message = match status {
            Status::Passed => ("✅", "ALL CHECKS PASSED", "green"),
            Status::Failed => ("🚨", "ISSUES FOUND", "red"),
            Status::Warning => ("⚠️", "WARNINGS DETECTED", "yellow"),
            Status::Skipped => ("⏭️", "SOME CHECKS SKIPPED", "cyan"),
        };

        let colored_status = match status_message.2 {
            "green" => format!("{} {}", status_message.0, status_message.1).green().bold(),
            "red" => format!("{} {}", status_message.0, status_message.1).red().bold(),
            "yellow" => format!("{} {}", status_message.0, status_message.1).yellow().bold(),
            "cyan" => format!("{} {}", status_message.0, status_message.1).cyan().bold(),
            _ => format!("{} {}", status_message.0, status_message.1).white().bold(),
        };

        println!("  Status: {}", colored_status);
        println!();
    }

    /// Print recommendations section
    pub fn print_recommendations(recommendations: &[String]) {
        if !recommendations.is_empty() {
            println!("{}", "💡 RECOMMENDATIONS".bold().green());
            println!("{}", "──────────────────".green());
            for (i, rec) in recommendations.iter().enumerate() {
                println!("  {}. {}", i + 1, rec.green());
            }
            println!();
        }
    }

    /// Format file size in human readable format
    pub fn format_file_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    /// Format percentage with color coding
    pub fn format_percentage(value: f64, good_threshold: f64, poor_threshold: f64) -> ColoredString {
        let formatted = format!("{:.1}%", value);
        
        if value >= good_threshold {
            formatted.green()
        } else if value >= poor_threshold {
            formatted.yellow()
        } else {
            formatted.red()
        }
    }

    /// Format number with thousands separator
    pub fn format_number(num: usize) -> String {
        let num_str = num.to_string();
        let chars: Vec<char> = num_str.chars().collect();
        let mut result = String::new();
        
        for (i, &ch) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i) % 3 == 0 {
                result.push(',');
            }
            result.push(ch);
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_formatting() {
        assert_eq!(Severity::Critical.to_icon(), "🚨");
        assert_eq!(Severity::Info.to_icon(), "ℹ️");
    }

    #[test]
    fn test_status_formatting() {
        assert_eq!(Status::Passed.to_icon(), "✅");
        assert_eq!(Status::Failed.to_icon(), "❌");
    }

    #[test]
    fn test_common_summary() {
        let mut summary = CommonSummary::new();
        
        summary.add_result(&Status::Passed, Some(&Severity::Info));
        summary.add_result(&Status::Failed, Some(&Severity::Critical));
        
        assert_eq!(summary.total_items, 2);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.critical_issues, 1);
        assert!(!summary.is_successful());
    }

    #[test]
    fn test_file_size_formatting() {
        assert_eq!(ReportUtils::format_file_size(512), "512 B");
        assert_eq!(ReportUtils::format_file_size(1024), "1.0 KB");
        assert_eq!(ReportUtils::format_file_size(1048576), "1.0 MB");
    }
}