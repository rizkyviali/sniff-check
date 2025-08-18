/// Common CLI argument patterns shared across commands
use clap::Args;

/// Standard output options available to all commands
#[derive(Args, Clone)]
pub struct OutputOptions {
    #[arg(long, help = "Output in JSON format")]
    pub json: bool,
    
    #[arg(long, help = "Quiet mode (minimal output)")]
    pub quiet: bool,
}

/// Common threshold-based command options
#[derive(Args, Clone)]
pub struct ThresholdOptions {
    #[arg(long, default_value_t = 100, help = "Threshold value for detection")]
    pub threshold: usize,
    
    #[arg(long, help = "Set custom warning threshold")]
    pub warning_threshold: Option<usize>,
    
    #[arg(long, help = "Set custom error threshold")]
    pub error_threshold: Option<usize>,
}

/// Common file filtering options
#[derive(Args, Clone)]
pub struct FileFilterOptions {
    #[arg(long, help = "Include only these file extensions (comma-separated)")]
    pub include: Option<String>,
    
    #[arg(long, help = "Exclude these file extensions (comma-separated)")]
    pub exclude: Option<String>,
    
    #[arg(long, help = "Include files matching this pattern")]
    pub pattern: Option<String>,
}

/// Common validation options
#[derive(Args, Clone)]
pub struct ValidationOptions {
    #[arg(long, help = "Fail fast - exit on first critical issue")]
    pub fail_fast: bool,
    
    #[arg(long, help = "Set maximum allowed warnings before failure")]
    pub max_warnings: Option<usize>,
    
    #[arg(long, help = "Ignore common false positives")]
    pub ignore_false_positives: bool,
}

impl OutputOptions {
    pub fn new(json: bool, quiet: bool) -> Self {
        Self { json, quiet }
    }
    
    /// Print output based on the format settings
    pub fn print_output<T>(&self, data: &T) -> anyhow::Result<()> 
    where 
        T: serde::Serialize + std::fmt::Display
    {
        if self.json {
            println!("{}", serde_json::to_string_pretty(data)?);
        } else if !self.quiet {
            println!("{}", data);
        }
        Ok(())
    }
    
    /// Print a message only if not in quiet mode
    pub fn print_if_not_quiet(&self, message: &str) {
        if !self.quiet {
            println!("{}", message);
        }
    }
}

impl Default for OutputOptions {
    fn default() -> Self {
        Self {
            json: false,
            quiet: false,
        }
    }
}

impl Default for ThresholdOptions {
    fn default() -> Self {
        Self {
            threshold: 100,
            warning_threshold: None,
            error_threshold: None,
        }
    }
}

impl Default for FileFilterOptions {
    fn default() -> Self {
        Self {
            include: None,
            exclude: None,
            pattern: None,
        }
    }
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self {
            fail_fast: false,
            max_warnings: None,
            ignore_false_positives: false,
        }
    }
}