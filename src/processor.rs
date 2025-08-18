use crate::config::Config;
use crate::debug;
use crate::error::Result;

#[derive(Debug)]
pub enum Mode {
    CommentToEcho, // echo-comment: comments become echo statements
    EchoToComment, // comment-echo: echo statements become comments
}

#[derive(Debug, PartialEq)]
enum CommentType {
    Regular(String),     // # comment -> echo "comment"
    NoEcho(String),      // ## comment -> # comment (no echo)
    EscapedHash(String), // #\# comment -> echo "# comment"
}

/// Process script content by converting between comments and echo statements
pub fn process_script_content(content: &str, mode: Mode) -> Result<String> {
    let config = Config::from_env();
    process_script_content_with_config(content, mode, &config)
}

/// Process script content with explicit configuration
pub fn process_script_content_with_config(
    content: &str,
    mode: Mode,
    config: &Config,
) -> Result<String> {
    let mut result = String::new();

    // Use configured shebang
    result.push_str(&config.shebang());
    result.push('\n');

    // Process each line based on mode
    let mut is_first_line = true;
    for (i, line) in content.lines().enumerate() {
        debug!("Processing line {}: '{}'", i, line);
        if is_first_line && line.starts_with("#!") {
            // Skip original shebang - we already added our own
            debug!("DEBUG: Skipping first-line shebang: {}", line);
            is_first_line = false;
            continue;
        }
        is_first_line = false;

        let processed_line = match mode {
            Mode::CommentToEcho => process_comment_to_echo(line, config),
            Mode::EchoToComment => process_echo_to_comment(line),
        };

        debug!("'{}' -> '{}'", line, processed_line);

        result.push_str(&processed_line);
        result.push('\n');
    }

    debug!("Finished processing all lines");
    Ok(result)
}

fn process_comment_to_echo(line: &str, config: &Config) -> String {
    if let Some(comment) = extract_comment(line) {
        match comment {
            CommentType::Regular(content) => {
                // Convert regular comments to echo statements with optional color
                let indent = get_indent(line);
                let colored_content = config.colorize(&content);
                let echo_cmd = if config.comment_color.is_some() {
                    "echo -e" // Use -e for escape sequences when we have colors
                } else {
                    "echo"
                };
                format!(
                    "{}{} \"{}\"",
                    indent,
                    echo_cmd,
                    escape_for_echo(&colored_content)
                )
            }
            CommentType::NoEcho(content) => {
                // Keep ## comments as regular comments (remove one #)
                let indent = get_indent(line);
                if content.is_empty() {
                    format!("{}#", indent)
                } else {
                    format!("{}# {}", indent, content)
                }
            }
            CommentType::EscapedHash(content) => {
                // Convert #\# to echo "# content"
                let indent = get_indent(line);
                let echo_content = if content.is_empty() {
                    "#".to_string()
                } else {
                    format!("# {}", content)
                };
                let colored_content = config.colorize(&echo_content);
                let echo_cmd = if config.comment_color.is_some() {
                    "echo -e"
                } else {
                    "echo"
                };
                format!(
                    "{}{} \"{}\"",
                    indent,
                    echo_cmd,
                    escape_for_echo(&colored_content)
                )
            }
        }
    } else {
        // Keep other lines as-is
        line.to_string()
    }
}

fn process_echo_to_comment(line: &str) -> String {
    if let Some(echo_content) = extract_echo(line) {
        // Convert echo statements to comments
        let indent = get_indent(line);

        // Strip color codes when converting back to comments
        let clean_content = strip_color_codes(&echo_content);

        // Check if the echo content starts with "# " - if so, escape it
        if let Some(content) = clean_content.strip_prefix("# ") {
            // Remove "# " prefix
            if content.is_empty() {
                format!("{}#\\#", indent)
            } else {
                format!("{}#\\# {}", indent, content)
            }
        } else if clean_content == "#" {
            format!("{}#\\#", indent)
        } else if clean_content.is_empty() {
            format!("{}#", indent)
        } else {
            format!("{}# {}", indent, clean_content)
        }
    } else {
        // Keep other lines as-is
        line.to_string()
    }
}

fn extract_comment(line: &str) -> Option<CommentType> {
    let trimmed = line.trim_start();

    // Check for #\# (escaped hash)
    if let Some(rest) = trimmed.strip_prefix("#\\# ") {
        return Some(CommentType::EscapedHash(rest.trim_start().to_string()));
    } else if trimmed == "#\\#" {
        return Some(CommentType::EscapedHash(String::new()));
    }

    // Check for ## (no-echo comment)
    if let Some(rest) = trimmed.strip_prefix("## ") {
        return Some(CommentType::NoEcho(rest.trim_start().to_string()));
    } else if trimmed == "##" {
        return Some(CommentType::NoEcho(String::new()));
    }

    // Check for regular # comment
    if let Some(rest) = trimmed.strip_prefix("# ") {
        return Some(CommentType::Regular(rest.trim_start().to_string()));
    } else if trimmed == "#" {
        return Some(CommentType::Regular(String::new()));
    }

    None
}

fn extract_echo(line: &str) -> Option<String> {
    let trimmed = line.trim_start();

    // Match exactly "echo" or "echo -e" followed by end-of-string or whitespace
    if trimmed == "echo" || trimmed == "echo -e" {
        return Some(String::new());
    }

    // Match "echo " or "echo -e " (with space) for content after
    let content = if let Some(rest) = trimmed.strip_prefix("echo -e ") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("echo ") {
        rest
    } else {
        return None;
    };

    let content = content.trim();

    // Handle quoted strings
    if (content.starts_with('"') && content.ends_with('"'))
        || (content.starts_with('\'') && content.ends_with('\''))
    {
        let inner = &content[1..content.len() - 1];
        Some(unescape_from_echo(inner))
    } else if content.is_empty() {
        Some(String::new())
    } else {
        // Unquoted echo - take everything after "echo "
        Some(content.to_string())
    }
}

fn get_indent(line: &str) -> String {
    line.chars().take_while(|c| c.is_whitespace()).collect()
}

fn escape_for_echo(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "\\$")
        .replace('`', "\\`")
}

fn unescape_from_echo(text: &str) -> String {
    text.replace("\\\"", "\"")
        .replace("\\$", "$")
        .replace("\\`", "`")
        .replace("\\\\", "\\")
}

/// Strip ANSI color codes from text
fn strip_color_codes(text: &str) -> String {
    // Simple regex-free approach to strip ANSI escape sequences
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            // Skip the escape sequence
            chars.next(); // consume '['
            for ch in chars.by_ref() {
                if ch.is_ascii_alphabetic() {
                    break; // End of escape sequence
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_comment_types() {
        // Regular comments (will be echoed)
        assert_eq!(
            extract_comment("# hello world"),
            Some(CommentType::Regular("hello world".to_string()))
        );
        assert_eq!(
            extract_comment("    # indented comment"),
            Some(CommentType::Regular("indented comment".to_string()))
        );
        assert_eq!(
            extract_comment("#"),
            Some(CommentType::Regular(String::new()))
        );

        // No-echo comments (## -> #)
        assert_eq!(
            extract_comment("## private comment"),
            Some(CommentType::NoEcho("private comment".to_string()))
        );
        assert_eq!(
            extract_comment("  ## indented private"),
            Some(CommentType::NoEcho("indented private".to_string()))
        );
        assert_eq!(
            extract_comment("##"),
            Some(CommentType::NoEcho(String::new()))
        );

        // Escaped hash comments (#\# -> echo "#")
        assert_eq!(
            extract_comment("#\\# with hash"),
            Some(CommentType::EscapedHash("with hash".to_string()))
        );
        assert_eq!(
            extract_comment("  #\\# indented hash"),
            Some(CommentType::EscapedHash("indented hash".to_string()))
        );
        assert_eq!(
            extract_comment("#\\#"),
            Some(CommentType::EscapedHash(String::new()))
        );

        // Non-comments
        assert_eq!(extract_comment("echo hello"), None);
        assert_eq!(extract_comment("#!/bin/bash"), None);
        assert_eq!(extract_comment("#no space"), None);
        assert_eq!(extract_comment("not # a comment"), None);
    }

    #[test]
    fn test_extract_echo() {
        assert_eq!(
            extract_echo("echo \"hello world\""),
            Some("hello world".to_string())
        );
        assert_eq!(
            extract_echo("echo -e \"colored text\""),
            Some("colored text".to_string())
        );
        assert_eq!(extract_echo("    echo 'test'"), Some("test".to_string()));
        assert_eq!(extract_echo("echo"), Some(String::new()));
        assert_eq!(extract_echo("echo -e"), Some(String::new()));
        assert_eq!(extract_echo("# comment"), None);
        assert_eq!(extract_echo("ls -la"), None);

        // Edge cases
        assert_eq!(extract_echo("echo "), Some(String::new()));
        assert_eq!(extract_echo("echo -e "), Some(String::new()));
        assert_eq!(
            extract_echo("echo unquoted text"),
            Some("unquoted text".to_string())
        );
        assert_eq!(extract_echo("echo \"\""), Some(String::new()));
        assert_eq!(extract_echo("echo ''"), Some(String::new()));
        assert_eq!(
            extract_echo("echo \"with \\\"escaped\\\" quotes\""),
            Some("with \"escaped\" quotes".to_string())
        );
        assert_eq!(
            extract_echo("echo \"$var and `cmd`\""),
            Some("$var and `cmd`".to_string())
        );
        assert_eq!(
            extract_echo("  echo  \"  spaced  \"  "),
            Some("  spaced  ".to_string())
        );
        assert_eq!(extract_echo("echoing"), None);
        assert_eq!(extract_echo("echo-like"), None);
    }

    #[test]
    fn test_get_indent() {
        assert_eq!(get_indent("# comment"), "");
        assert_eq!(get_indent("    # indented"), "    ");
        assert_eq!(get_indent("\t# tabbed"), "\t");
        assert_eq!(get_indent("  \t  mixed"), "  \t  ");
        assert_eq!(get_indent("no indent"), "");
    }

    #[test]
    fn test_escape_unescape_roundtrip() {
        let test_cases = vec![
            "simple text",
            "text with \"quotes\"",
            "text with $variables",
            "text with `commands`",
            "text with \\backslashes",
            "complex: \"$var\" and `echo test` with \\path",
            "",
        ];

        for original in test_cases {
            let escaped = escape_for_echo(original);
            let unescaped = unescape_from_echo(&escaped);
            assert_eq!(original, unescaped, "Failed roundtrip for: {}", original);
        }
    }

    #[test]
    fn test_strip_color_codes() {
        assert_eq!(strip_color_codes("plain text"), "plain text");
        assert_eq!(strip_color_codes("\x1b[0;32mgreen\x1b[0m"), "green");
        assert_eq!(
            strip_color_codes("\x1b[1;31mred\x1b[0m and \x1b[0;34mblue\x1b[0m"),
            "red and blue"
        );
        assert_eq!(strip_color_codes("\x1b[0m"), "");
    }

    #[test]
    fn test_process_comment_to_echo_with_color() {
        let config = Config {
            shell: "bash".to_string(),
            shell_flags: vec![],
            comment_color: Some("\x1b[0;32m".to_string()),
        };

        assert_eq!(
            process_comment_to_echo("# test", &config),
            "echo -e \"\x1b[0;32mtest\x1b[0m\""
        );
        assert_eq!(
            process_comment_to_echo("  # indented", &config),
            "  echo -e \"\x1b[0;32mindented\x1b[0m\""
        );
    }

    #[test]
    fn test_process_comment_to_echo_without_color() {
        let config = Config::default();

        assert_eq!(process_comment_to_echo("# test", &config), "echo \"test\"");
        assert_eq!(
            process_comment_to_echo("  # indented", &config),
            "  echo \"indented\""
        );
    }

    #[test]
    fn test_process_echo_to_comment() {
        // Regular echoes become comments
        assert_eq!(process_echo_to_comment("echo \"test\""), "# test");
        assert_eq!(process_echo_to_comment("  echo 'indented'"), "  # indented");
        assert_eq!(process_echo_to_comment("echo"), "#");
        assert_eq!(process_echo_to_comment("echo -e"), "#");

        // Echoes that start with "# " become #\# comments
        assert_eq!(
            process_echo_to_comment("echo \"# with hash\""),
            "#\\# with hash"
        );
        assert_eq!(
            process_echo_to_comment("  echo '# indented hash'"),
            "  #\\# indented hash"
        );
        assert_eq!(process_echo_to_comment("echo \"#\""), "#\\#");

        // Colored echoes get stripped
        assert_eq!(
            process_echo_to_comment("echo -e \"\x1b[0;32mgreen\x1b[0m\""),
            "# green"
        );

        // Non-echoes stay the same
        assert_eq!(process_echo_to_comment("not an echo"), "not an echo");
        assert_eq!(
            process_echo_to_comment("# already comment"),
            "# already comment"
        );
    }

    #[test]
    fn test_bidirectional_conversion_with_config() {
        let config = Config {
            shell: "bash".to_string(),
            shell_flags: vec![],
            comment_color: Some("\x1b[0;32m".to_string()),
        };

        // Regular comment -> echo -> comment
        let comment_line = "    # Hello world";
        let echo_line = process_comment_to_echo(comment_line, &config);
        assert_eq!(echo_line, "    echo -e \"\x1b[0;32mHello world\x1b[0m\"");
        let back_to_comment = process_echo_to_comment(&echo_line);
        assert_eq!(back_to_comment, "    # Hello world");
    }

    #[test]
    fn test_process_script_content_with_config() {
        let config = Config {
            shell: "zsh".to_string(),
            shell_flags: vec!["-euo".to_string(), "pipefail".to_string()],
            comment_color: Some("\x1b[0;32m".to_string()),
        };

        let input = "#!/usr/bin/env bash\n# test comment\necho existing\n## private note";

        let result =
            process_script_content_with_config(input, Mode::CommentToEcho, &config).unwrap();
        let expected = "#!/usr/bin/env -S zsh -euo pipefail\necho -e \"\x1b[0;32mtest comment\x1b[0m\"\necho existing\n# private note\n";
        assert_eq!(result, expected);
    }
}
