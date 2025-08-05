use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub large_files: LargeFilesConfig,
    pub typescript: TypeScriptConfig,
    pub imports: ImportsConfig,
    pub bundle: BundleConfig,
    pub performance: PerformanceConfig,
    pub memory: MemoryConfig,
    pub environment: EnvironmentConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LargeFilesConfig {
    pub threshold: usize,
    pub excluded_dirs: Vec<String>,
    pub excluded_files: Vec<String>,
    pub severity_levels: SeverityLevels,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeverityLevels {
    pub warning: usize,
    pub error: usize,
    pub critical: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TypeScriptConfig {
    pub strict_any_check: bool,
    pub allow_ts_ignore: bool,
    pub require_return_types: bool,
    pub min_type_coverage: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportsConfig {
    pub auto_fix: bool,
    pub excluded_patterns: Vec<String>,
    pub check_dev_dependencies: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BundleConfig {
    pub max_bundle_size_mb: f64,
    pub max_chunk_size_mb: f64,
    pub build_dirs: Vec<String>,
    pub warn_on_large_chunks: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerformanceConfig {
    pub lighthouse_enabled: bool,
    pub min_performance_score: f64,
    pub min_accessibility_score: f64,
    pub server_urls: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryConfig {
    pub check_patterns: bool,
    pub check_processes: bool,
    pub max_process_memory_mb: f64,
    pub pattern_severity_threshold: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvironmentConfig {
    pub required_vars: Vec<String>,
    pub check_security: bool,
    pub allow_empty_values: bool,
    pub env_files: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            large_files: LargeFilesConfig {
                threshold: 100,
                excluded_dirs: vec![
                    "node_modules".to_string(),
                    ".next".to_string(),
                    "dist".to_string(),
                    ".git".to_string(),
                    "target".to_string(),
                    "build".to_string(),
                ],
                excluded_files: vec![
                    "*.min.js".to_string(),
                    "*.bundle.js".to_string(),
                    "package-lock.json".to_string(),
                    "yarn.lock".to_string(),
                ],
                severity_levels: SeverityLevels {
                    warning: 100,
                    error: 200,
                    critical: 400,
                },
            },
            typescript: TypeScriptConfig {
                strict_any_check: true,
                allow_ts_ignore: false,
                require_return_types: true,
                min_type_coverage: 80.0,
            },
            imports: ImportsConfig {
                auto_fix: false,
                excluded_patterns: vec![
                    "react".to_string(),
                    "@types/*".to_string(),
                ],
                check_dev_dependencies: true,
            },
            bundle: BundleConfig {
                max_bundle_size_mb: 2.0,
                max_chunk_size_mb: 0.5,
                build_dirs: vec![
                    ".next".to_string(),
                    "dist".to_string(),
                    "build".to_string(),
                    "out".to_string(),
                ],
                warn_on_large_chunks: true,
            },
            performance: PerformanceConfig {
                lighthouse_enabled: true,
                min_performance_score: 75.0,
                min_accessibility_score: 90.0,
                server_urls: vec![
                    "http://localhost:3000".to_string(),
                    "http://localhost:3001".to_string(),
                    "http://localhost:8000".to_string(),
                    "http://localhost:8080".to_string(),
                ],
            },
            memory: MemoryConfig {
                check_patterns: true,
                check_processes: true,
                max_process_memory_mb: 1000.0,
                pattern_severity_threshold: "high".to_string(),
            },
            environment: EnvironmentConfig {
                required_vars: vec![
                    "NODE_ENV".to_string(),
                    "NEXT_PUBLIC_APP_URL".to_string(),
                ],
                check_security: true,
                allow_empty_values: false,
                env_files: vec![
                    ".env".to_string(),
                    ".env.local".to_string(),
                    ".env.development".to_string(),
                    ".env.production".to_string(),
                ],
            },
        }
    }
}

impl Config {
    /// Load configuration from file or create default
    pub fn load() -> Result<Self> {
        let config_paths = vec![
            "sniff.toml",
            "sniff-check.toml",
            ".sniff.toml",
            ".sniffrc.toml",
        ];
        
        for path in config_paths {
            if Path::new(path).exists() {
                return Self::load_from_file(path);
            }
        }
        
        // If no config file found, return default
        Ok(Config::default())
    }
    
    /// Load configuration from specific file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    /// Create default configuration file
    pub fn create_default_config() -> Result<()> {
        let config = Config::default();
        config.save_to_file("sniff.toml")?;
        Ok(())
    }
    
    /// Get configuration file path if it exists
    pub fn get_config_path() -> Option<PathBuf> {
        let config_paths = vec![
            "sniff.toml",
            "sniff-check.toml", 
            ".sniff.toml",
            ".sniffrc.toml",
        ];
        
        for path in config_paths {
            if Path::new(path).exists() {
                return Some(PathBuf::from(path));
            }
        }
        
        None
    }
    
    /// Check if a directory should be excluded from scanning
    pub fn is_dir_excluded(&self, dir_name: &str) -> bool {
        self.large_files.excluded_dirs.iter().any(|excluded| {
            if excluded.contains('*') {
                // Simple glob matching
                let pattern = excluded.replace('*', ".*");
                regex::Regex::new(&pattern).unwrap().is_match(dir_name)
            } else {
                dir_name == excluded
            }
        })
    }
    
    /// Check if a file should be excluded from scanning
    pub fn is_file_excluded(&self, file_name: &str) -> bool {
        self.large_files.excluded_files.iter().any(|excluded| {
            if excluded.contains('*') {
                // Simple glob matching
                let pattern = excluded.replace('*', ".*");
                regex::Regex::new(&pattern).unwrap().is_match(file_name)
            } else {
                file_name == excluded
            }
        })
    }
    
    /// Get required environment variables
    pub fn get_required_env_vars(&self) -> &[String] {
        &self.environment.required_vars
    }
    
    /// Get severity level for line count
    pub fn get_severity_for_lines(&self, line_count: usize) -> SeverityLevel {
        if line_count >= self.large_files.severity_levels.critical {
            SeverityLevel::Critical
        } else if line_count >= self.large_files.severity_levels.error {
            SeverityLevel::Error
        } else if line_count >= self.large_files.severity_levels.warning {
            SeverityLevel::Warning
        } else {
            SeverityLevel::Info
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SeverityLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Configuration utilities
pub struct ConfigUtils;

impl ConfigUtils {
    /// Initialize configuration in current directory
    pub fn init() -> Result<()> {
        if Config::get_config_path().is_some() {
            println!("Configuration file already exists.");
            return Ok(());
        }
        
        Config::create_default_config()?;
        println!("Created default configuration file: sniff.toml");
        println!("Edit this file to customize sniff-check behavior for your project.");
        
        Ok(())
    }
    
    /// Show current configuration
    pub fn show() -> Result<()> {
        let config = Config::load()?;
        
        if let Some(path) = Config::get_config_path() {
            println!("Configuration loaded from: {}", path.display());
        } else {
            println!("Using default configuration (no config file found)");
        }
        
        println!("\nCurrent configuration:");
        println!("{}", toml::to_string_pretty(&config)?);
        
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate() -> Result<()> {
        let config = Config::load()?;
        
        // Validate thresholds
        if config.large_files.threshold == 0 {
            return Err(anyhow::anyhow!("Large files threshold cannot be 0"));
        }
        
        if config.typescript.min_type_coverage < 0.0 || config.typescript.min_type_coverage > 100.0 {
            return Err(anyhow::anyhow!("TypeScript coverage must be between 0 and 100"));
        }
        
        if config.bundle.max_bundle_size_mb <= 0.0 {
            return Err(anyhow::anyhow!("Bundle size limit must be positive"));
        }
        
        // Validate severity levels
        let levels = &config.large_files.severity_levels;
        if levels.warning >= levels.error || levels.error >= levels.critical {
            return Err(anyhow::anyhow!("Severity levels must be in ascending order: warning < error < critical"));
        }
        
        println!("✅ Configuration is valid");
        Ok(())
    }
    
    /// Get configuration for specific command
    pub fn get_command_config(command: &str) -> Result<String> {
        let config = Config::load()?;
        
        let section = match command {
            "large" => toml::to_string_pretty(&config.large_files)?,
            "types" => toml::to_string_pretty(&config.typescript)?,
            "imports" => toml::to_string_pretty(&config.imports)?,
            "bundle" => toml::to_string_pretty(&config.bundle)?,
            "perf" => toml::to_string_pretty(&config.performance)?,
            "memory" => toml::to_string_pretty(&config.memory)?,
            "env" => toml::to_string_pretty(&config.environment)?,
            _ => return Err(anyhow::anyhow!("Unknown command: {}", command)),
        };
        
        Ok(section)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.large_files.threshold, 100);
        assert_eq!(config.typescript.strict_any_check, true);
        assert_eq!(config.bundle.max_bundle_size_mb, 2.0);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.large_files.threshold, deserialized.large_files.threshold);
    }
    
    #[test]
    fn test_file_exclusion() {
        let config = Config::default();
        
        assert!(config.is_file_excluded("test.min.js"));
        assert!(config.is_file_excluded("bundle.min.js"));
        assert!(!config.is_file_excluded("test.js"));
    }
    
    #[test]
    fn test_dir_exclusion() {
        let config = Config::default();
        
        assert!(config.is_dir_excluded("node_modules"));
        assert!(config.is_dir_excluded(".next"));
        assert!(!config.is_dir_excluded("src"));
    }
    
    #[test]
    fn test_severity_levels() {
        let config = Config::default();
        
        assert_eq!(config.get_severity_for_lines(50), SeverityLevel::Info);
        assert_eq!(config.get_severity_for_lines(150), SeverityLevel::Warning);
        assert_eq!(config.get_severity_for_lines(300), SeverityLevel::Error);
        assert_eq!(config.get_severity_for_lines(500), SeverityLevel::Critical);
    }
}