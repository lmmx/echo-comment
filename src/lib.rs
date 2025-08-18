use std::fs;

pub mod cli;
pub mod color;
pub mod config;
pub mod error;
pub mod processor;
pub mod runner;

pub use config::Config;
pub use error::{EchoCommentError, Result};
pub use processor::{Mode, process_script_content, process_script_content_with_config};
pub use runner::ScriptRunner;

macro_rules! debug {
    ($($arg:tt)*) => {
        if std::env::var("ECHO_COMMENT_DEBUG").is_ok() {
            eprintln!($($arg)*);
        }
    };
}

pub(crate) use debug;

/// Main entry point for processing and running a script
pub fn run_script(script_path: &str, script_args: &[String], mode: Mode) -> Result<()> {
    debug!("Processing script: {} in mode: {:?}", script_path, mode);

    // Read the input script
    let content = fs::read_to_string(script_path).map_err(|e| EchoCommentError::FileRead {
        path: script_path.to_string(),
        source: e,
    })?;

    // Process the content
    let processed_content = process_script_content(&content, mode)?;

    // Run the processed script
    let runner = ScriptRunner::new();
    runner.run_script(&processed_content, script_args)?;

    Ok(())
}
