use crate::{
    Config, EchoCommentError, Mode, Result, color::resolve_color,
    process_script_content_with_config, runner::ScriptRunner,
};
use clap::Parser;
use std::fs;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A bidirectional bash interpreter that converts comments ↔ echo statements"
)]
#[command(arg_required_else_help = true)]
pub struct Args {
    /// Script file to process
    pub script: String,

    /// Arguments to pass to the script
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub script_args: Vec<String>,

    /// Shell to use (default: bash)
    #[arg(long = "shell", short = 's')]
    pub shell: Option<String>,

    /// Shell flags (e.g., "-euo pipefail")
    #[arg(long = "shell-flags")]
    pub shell_flags: Option<String>,

    /// Color name or ANSI code for comments (e.g., "red", "green", "bold-blue")
    #[arg(long = "color", short = 'c')]
    pub color: Option<String>,

    /// Enable verbose debug output
    #[arg(long = "verbose", short = 'v')]
    pub verbose: bool,
}

impl Args {
    /// Convert CLI args to Config
    pub fn to_config(&self) -> Config {
        let mut config = Config::from_env(); // Still respect env vars as fallback

        // CLI args override env vars
        if let Some(shell) = &self.shell {
            config.shell = shell.clone();
        }

        if let Some(flags) = &self.shell_flags {
            config.shell_flags = flags.split_whitespace().map(|s| s.to_string()).collect();
        }

        if let Some(color) = &self.color {
            config.comment_color = Some(resolve_color(color));
        }

        config
    }
}

/// Run the CLI with the specified mode and binary name for usage messages
pub fn run_cli(mode: Mode, binary_name: &str, description: &str) {
    // Parse arguments with clap - much simpler approach
    let args = Args::parse();

    // Run the script processing
    if let Err(e) = run_script_with_args(&args, mode) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_script_with_args(args: &Args, mode: Mode) -> Result<()> {
    if args.verbose {
        eprintln!("Processing script: {} in mode: {:?}", args.script, mode);
    }

    // Read the input script
    let content = fs::read_to_string(&args.script).map_err(|e| EchoCommentError::FileRead {
        path: args.script.clone(),
        source: e,
    })?;

    // Convert CLI args to config
    let config = args.to_config();
    if args.verbose {
        eprintln!("Using config: {:?}", config);
    }

    // Process the content with config
    let processed_content = process_script_content_with_config(&content, mode, &config)?;

    // Run the processed script
    let runner = ScriptRunner::new();
    runner.run_script(&processed_content, &args.script_args)?;

    Ok(())
}
