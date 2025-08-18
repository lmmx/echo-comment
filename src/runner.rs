use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

use crate::error::{EchoCommentError, Result};

/// Handles the execution of processed scripts
pub struct ScriptRunner {
    // Future: This could hold configuration like shell, flags, etc.
}

impl ScriptRunner {
    pub fn new() -> Self {
        Self {}
    }

    /// Execute a processed script with the given arguments
    pub fn run_script(&self, script_content: &str, script_args: &[String]) -> Result<()> {
        // Create a temporary file for the processed script
        let mut temp_file =
            NamedTempFile::new().map_err(|e| EchoCommentError::TempFileCreation { source: e })?;

        // Write the processed content
        temp_file
            .write_all(script_content.as_bytes())
            .map_err(|e| EchoCommentError::FileWrite { source: e })?;

        // Flush to ensure content is written
        temp_file
            .flush()
            .map_err(|e| EchoCommentError::FileWrite { source: e })?;

        // Get the temp file path
        let temp_path = temp_file.into_temp_path();

        // Make the temp file executable (Unix only)
        #[cfg(unix)]
        self.make_executable(&temp_path)?;

        // Execute the processed script with the provided arguments
        let status = Command::new(&temp_path)
            .args(script_args)
            .status()
            .map_err(|e| EchoCommentError::ScriptExecution {
                message: "Failed to execute processed script".to_string(),
                source: e,
            })?;

        // Clean up the temp file
        temp_path.close().map_err(|e| EchoCommentError::FileWrite {
            source: std::io::Error::other(e),
        })?;

        // Exit with the same code as the script
        std::process::exit(status.code().unwrap_or(1));
    }

    #[cfg(unix)]
    fn make_executable(&self, path: &std::path::Path) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = fs::metadata(path)
            .map_err(|e| EchoCommentError::PermissionSet { source: e })?
            .permissions();

        perms.set_mode(0o755);

        fs::set_permissions(path, perms)
            .map_err(|e| EchoCommentError::PermissionSet { source: e })?;

        Ok(())
    }
}

impl Default for ScriptRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_runner_creation() {
        let _runner = ScriptRunner::new();
    }
}
