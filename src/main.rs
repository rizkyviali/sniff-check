use clap::{Parser, Subcommand};
use std::process;

mod commands;
mod config;
mod utils;
mod common;

// Import specific command functions instead of using glob imports
use commands::{menu, large, types, imports, bundle, perf, memory, env, context, deploy};
use config::ConfigUtils;

#[derive(Parser)]
#[command(name = "sniff")]
#[command(about = "Opinionated TypeScript/Next.js Development Toolkit")]
#[command(version = "0.1.10")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    #[arg(long, help = "Use custom configuration file")]
    config: Option<String>,
    
    #[arg(long, help = "Output in JSON format")]
    json: bool,
    
    #[arg(long, help = "Quiet mode (minimal output)")]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Show interactive menu (default)")]
    Menu,
    #[command(about = "Find large files over threshold")]
    Large {
        #[arg(long, default_value_t = 100)]
        threshold: usize,
    },
    #[command(about = "Check TypeScript type coverage and quality")]
    Types,
    #[command(about = "Detect unused and broken imports")]
    Imports,
    #[command(about = "Analyze bundle size and optimization")]
    Bundle,
    #[command(about = "Run Lighthouse performance audits")]
    Perf,
    #[command(about = "Detect memory leaks")]
    Memory,
    #[command(about = "Validate environment variables")]
    Env,
    #[command(about = "Analyze project structure and provide context")]
    Context,
    #[command(about = "Run complete deployment validation")]
    Deploy,
    #[command(about = "Configuration management")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    #[command(about = "Initialize default configuration file")]
    Init,
    #[command(about = "Show current configuration")]
    Show,
    #[command(about = "Validate configuration file")]
    Validate,
    #[command(about = "Show configuration for specific command")]
    Get {
        #[arg(help = "Command name (large, types, imports, etc.)")]
        command: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    let result = match cli.command {
        Some(Commands::Menu) | None => menu::run().await,
        Some(Commands::Large { threshold }) => large::run(threshold, cli.json, cli.quiet).await,
        Some(Commands::Types) => types::run(cli.json, cli.quiet).await,
        Some(Commands::Imports) => imports::run(cli.json, cli.quiet).await,
        Some(Commands::Bundle) => bundle::run(cli.json, cli.quiet).await,
        Some(Commands::Perf) => perf::run(cli.json, cli.quiet).await,
        Some(Commands::Memory) => memory::run(cli.json, cli.quiet).await,
        Some(Commands::Env) => env::run(cli.json, cli.quiet).await,
        Some(Commands::Context) => context::run(cli.json, cli.quiet).await,
        Some(Commands::Deploy) => deploy::run(cli.json, cli.quiet).await,
        Some(Commands::Config { action }) => handle_config_command(action).await,
    };
    
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

async fn handle_config_command(action: ConfigAction) -> anyhow::Result<()> {
    match action {
        ConfigAction::Init => ConfigUtils::init(),
        ConfigAction::Show => ConfigUtils::show(),
        ConfigAction::Validate => ConfigUtils::validate(),
        ConfigAction::Get { command } => {
            let config = ConfigUtils::get_command_config(&command)?;
            println!("Configuration for '{}':", command);
            println!("{}", config);
            Ok(())
        }
    }
}
