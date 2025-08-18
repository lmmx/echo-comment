/// Convert color names to ANSI escape codes
pub fn resolve_color(color_input: &str) -> String {
    match color_input.to_lowercase().as_str() {
        // Basic colors
        "black" => "\x1b[0;30m".to_string(),
        "red" => "\x1b[0;31m".to_string(),
        "green" => "\x1b[0;32m".to_string(),
        "yellow" => "\x1b[0;33m".to_string(),
        "blue" => "\x1b[0;34m".to_string(),
        "magenta" | "purple" => "\x1b[0;35m".to_string(),
        "cyan" => "\x1b[0;36m".to_string(),
        "white" => "\x1b[0;37m".to_string(),

        // Bright colors
        "bright-black" | "gray" | "grey" => "\x1b[0;90m".to_string(),
        "bright-red" => "\x1b[0;91m".to_string(),
        "bright-green" => "\x1b[0;92m".to_string(),
        "bright-yellow" => "\x1b[0;93m".to_string(),
        "bright-blue" => "\x1b[0;94m".to_string(),
        "bright-magenta" | "bright-purple" => "\x1b[0;95m".to_string(),
        "bright-cyan" => "\x1b[0;96m".to_string(),
        "bright-white" => "\x1b[0;97m".to_string(),

        // Bold colors
        "bold-black" => "\x1b[1;30m".to_string(),
        "bold-red" => "\x1b[1;31m".to_string(),
        "bold-green" => "\x1b[1;32m".to_string(),
        "bold-yellow" => "\x1b[1;33m".to_string(),
        "bold-blue" => "\x1b[1;34m".to_string(),
        "bold-magenta" | "bold-purple" => "\x1b[1;35m".to_string(),
        "bold-cyan" => "\x1b[1;36m".to_string(),
        "bold-white" => "\x1b[1;37m".to_string(),

        // Common aliases
        "orange" => "\x1b[0;33m".to_string(), // Yellow is close enough
        "pink" => "\x1b[1;35m".to_string(),   // Bold magenta

        // If it looks like an ANSI code already, pass it through
        input
            if input.starts_with("\x1b[")
                || input.starts_with("\\033[")
                || input.starts_with("\\x1b[") =>
        {
            // Handle different escape sequence formats
            input.replace("\\033", "\x1b").replace("\\x1b", "\x1b")
        }

        // Unknown color name - return as-is (might be a custom ANSI code)
        _ => color_input.to_string(),
    }
}

/// Get a list of supported color names for help text
pub fn supported_colors() -> Vec<&'static str> {
    vec![
        "black",
        "red",
        "green",
        "yellow",
        "blue",
        "magenta",
        "cyan",
        "white",
        "bright-black",
        "bright-red",
        "bright-green",
        "bright-yellow",
        "bright-blue",
        "bright-magenta",
        "bright-cyan",
        "bright-white",
        "bold-red",
        "bold-green",
        "bold-yellow",
        "bold-blue",
        "bold-magenta",
        "bold-cyan",
        "bold-white",
        "gray",
        "grey",
        "purple",
        "orange",
        "pink",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_colors() {
        assert_eq!(resolve_color("red"), "\x1b[0;31m");
        assert_eq!(resolve_color("RED"), "\x1b[0;31m");
        assert_eq!(resolve_color("Red"), "\x1b[0;31m");
        assert_eq!(resolve_color("green"), "\x1b[0;32m");
        assert_eq!(resolve_color("blue"), "\x1b[0;34m");
    }

    #[test]
    fn test_bright_colors() {
        assert_eq!(resolve_color("bright-red"), "\x1b[0;91m");
        assert_eq!(resolve_color("gray"), "\x1b[0;90m");
        assert_eq!(resolve_color("grey"), "\x1b[0;90m");
    }

    #[test]
    fn test_bold_colors() {
        assert_eq!(resolve_color("bold-red"), "\x1b[1;31m");
        assert_eq!(resolve_color("bold-green"), "\x1b[1;32m");
    }

    #[test]
    fn test_aliases() {
        assert_eq!(resolve_color("purple"), "\x1b[0;35m");
        assert_eq!(resolve_color("orange"), "\x1b[0;33m");
        assert_eq!(resolve_color("pink"), "\x1b[1;35m");
    }

    #[test]
    fn test_ansi_passthrough() {
        assert_eq!(resolve_color("\x1b[0;32m"), "\x1b[0;32m");
        assert_eq!(resolve_color("\\033[1;31m"), "\x1b[1;31m");
        assert_eq!(resolve_color("\\x1b[0;34m"), "\x1b[0;34m");
    }

    #[test]
    fn test_unknown_color() {
        assert_eq!(resolve_color("unknown"), "unknown");
        assert_eq!(resolve_color("custom-code"), "custom-code");
    }
}
