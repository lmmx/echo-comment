use crate::color::resolve_color;
use std::env;

/// Configuration for echo-comment behavior
#[derive(Debug, Clone)]
pub struct Config {
    pub shell: String,
    pub shell_flags: Vec<String>,
    pub comment_color: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shell: "bash".to_string(),
            shell_flags: vec![],
            comment_color: None,
        }
    }
}

impl Config {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Config::default();

        // Shell configuration
        if let Ok(shell) = env::var("ECHO_COMMENT_SHELL") {
            config.shell = shell;
        }

        if let Ok(flags) = env::var("ECHO_COMMENT_SHELL_FLAGS") {
            config.shell_flags = flags.split_whitespace().map(|s| s.to_string()).collect();
        }

        // Color configuration
        if let Ok(color) = env::var("ECHO_COMMENT_COLOR") {
            config.comment_color = Some(resolve_color(&color));
        }

        config
    }

    /// Generate the shebang line based on shell and flags
    pub fn shebang(&self) -> String {
        if self.shell_flags.is_empty() {
            format!("#!/usr/bin/env {}", self.shell)
        } else {
            format!(
                "#!/usr/bin/env -S {} {}",
                self.shell,
                self.shell_flags.join(" ")
            )
        }
    }

    /// Wrap text with color codes if color is configured
    pub fn colorize(&self, text: &str) -> String {
        if let Some(color) = &self.comment_color {
            format!("{}{}\x1b[0m", color, text)
        } else {
            text.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.shell, "bash");
        assert!(config.shell_flags.is_empty());
        assert!(config.comment_color.is_none());
    }

    #[test]
    fn test_shebang_generation() {
        let config = Config::default();
        assert_eq!(config.shebang(), "#!/usr/bin/env bash");

        let config = Config {
            shell: "zsh".to_string(),
            shell_flags: vec!["-euo".to_string(), "pipefail".to_string()],
            comment_color: None,
        };
        assert_eq!(config.shebang(), "#!/usr/bin/env -S zsh -euo pipefail");
    }

    #[test]
    fn test_colorize() {
        let config = Config::default();
        assert_eq!(config.colorize("test"), "test");

        let config = Config {
            shell: "bash".to_string(),
            shell_flags: vec![],
            comment_color: Some("\x1b[0;32m".to_string()),
        };
        assert_eq!(config.colorize("test"), "\x1b[0;32mtest\x1b[0m");
    }

    #[test]
    fn test_config_construction() {
        // Test that we can construct configs directly for testing
        let config = Config {
            shell: "zsh".to_string(),
            shell_flags: vec!["-euo".to_string(), "pipefail".to_string()],
            comment_color: Some("red".to_string()),
        };

        assert_eq!(config.shell, "zsh");
        assert_eq!(config.shell_flags, vec!["-euo", "pipefail"]);
        assert_eq!(config.comment_color, Some("red".to_string()));
    }
}
