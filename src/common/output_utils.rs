/// Utility functions for standardized command output
use colored::*;
use serde::Serialize;

/// Handle standard command output with JSON and quiet mode support
pub fn handle_command_output<T>(
    data: &T,
    json: bool,
    quiet: bool,
    print_fn: impl Fn(&T, bool),
) -> anyhow::Result<()>
where
    T: Serialize,
{
    if json {
        println!("{}", serde_json::to_string_pretty(data)?);
    } else {
        print_fn(data, quiet);
    }
    Ok(())
}

/// Print a status message only if not in quiet mode
pub fn print_status(message: &str, quiet: bool) {
    if !quiet {
        println!("{}", message.bold().blue());
    }
}

/// Print a success message with standard formatting
pub fn print_success(message: &str, quiet: bool) {
    if !quiet {
        println!("{}", message.bold().green());
    }
}

/// Print a warning message with standard formatting
pub fn print_warning(message: &str, quiet: bool) {
    if !quiet {
        println!("{}", message.bold().yellow());
    }
}

/// Print an error message with standard formatting
pub fn print_error(message: &str, quiet: bool) {
    if !quiet {
        eprintln!("{}", message.bold().red());
    }
}

/// Common pattern for command initialization
pub fn init_command(command_name: &str, quiet: bool) {
    print_status(&format!("🔍 Running {} analysis...", command_name), quiet);
}

/// Common pattern for command completion
pub fn complete_command(command_name: &str, success: bool, quiet: bool) {
    if success {
        print_success(&format!("✅ {} analysis completed", command_name), quiet);
    } else {
        print_warning(&format!("⚠️ {} analysis completed with issues", command_name), quiet);
    }
}