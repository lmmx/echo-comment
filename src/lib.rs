use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[derive(Debug)]
pub enum Mode {
    CommentToEcho,  // comment-echo: comments become echo statements
    EchoToComment,  // echo-comment: echo statements become comments
}

pub fn run_script(script_path: &str, script_args: &[String], mode: Mode) -> Result<(), Box<dyn std::error::Error>> {
    // Read the input script
    let content = fs::read_to_string(script_path).map_err(|e| {
        format!("Failed to read script '{}': {}", script_path, e)
    })?;
    
    // Create a temporary file for the processed script
    let mut temp_file = NamedTempFile::new()?;
    
    // Always start with a proper shebang
    writeln!(temp_file, "#!/usr/bin/env bash")?;
    writeln!(temp_file, "set -euo pipefail")?;
    
    // Process each line based on mode
    for line in content.lines() {
        if line.starts_with("#!") {
            // Skip original shebang - we already added our own
            continue;
        }
        
        let processed_line = match mode {
            Mode::CommentToEcho => process_comment_to_echo(line),
            Mode::EchoToComment => process_echo_to_comment(line),
        };
        
        writeln!(temp_file, "{}", processed_line)?;
    }

    // Flush and persist the temp file to get a path we can execute
    temp_file.flush()?;
    let temp_path = temp_file.into_temp_path();

    // Make the temp file executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_path, perms)?;
    }

    // Execute the processed script with the remaining arguments
    let mut cmd = Command::new(&temp_path);
    cmd.args(script_args);
    
    let status = cmd.status().map_err(|e| {
        format!("Failed to execute processed script: {}", e)
    })?;

    // Clean up the temp file
    temp_path.close()?;

    // Exit with the same code as the script
    std::process::exit(status.code().unwrap_or(1));
}

fn process_comment_to_echo(line: &str) -> String {
    if let Some(comment) = extract_comment(line) {
        // Convert standalone comments to echo statements
        let indent = get_indent(line);
        format!("{}echo \"{}\"", indent, escape_for_echo(&comment))
    } else {
        // Keep other lines as-is
        line.to_string()
    }
}

fn process_echo_to_comment(line: &str) -> String {
    if let Some(echo_content) = extract_echo(line) {
        // Convert echo statements to comments
        let indent = get_indent(line);
        if echo_content.is_empty() {
            format!("{}#", indent)
        } else {
            format!("{}# {}", indent, echo_content)
        }
    } else {
        // Keep other lines as-is
        line.to_string()
    }
}

fn extract_comment(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("# ") {
        Some(rest.to_string())
    } else if trimmed == "#" {
        Some(String::new())
    } else {
        None
    }
}

fn extract_echo(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    
    // Match: echo "content" or echo 'content' or just "echo"
    if let Some(rest) = trimmed.strip_prefix("echo") {
        // Check if there's nothing after "echo" or just whitespace
        if rest.is_empty() || rest.trim().is_empty() {
            return Some(String::new());
        }
        
        // Handle the case where there's a space after echo
        if let Some(content) = rest.strip_prefix(' ') {
            let content = content.trim();
            
            // Handle quoted strings
            if (content.starts_with('"') && content.ends_with('"')) || 
               (content.starts_with('\'') && content.ends_with('\'')) {
                let inner = &content[1..content.len()-1];
                Some(unescape_from_echo(inner))
            } else if content.is_empty() {
                Some(String::new())
            } else {
                // Unquoted echo - take everything after "echo "
                Some(content.to_string())
            }
        } else {
            // This handles "echo" with no space after it
            Some(String::new())
        }
    } else {
        None
    }
}

fn get_indent(line: &str) -> String {
    line.chars()
        .take_while(|c| c.is_whitespace())
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_comment() {
        assert_eq!(extract_comment("# hello world"), Some("hello world".to_string()));
        assert_eq!(extract_comment("    # indented comment"), Some("indented comment".to_string()));
        assert_eq!(extract_comment("#"), Some(String::new()));
        assert_eq!(extract_comment("echo hello"), None);
        assert_eq!(extract_comment("#!/bin/bash"), None);

        // Edge cases
        assert_eq!(extract_comment("#no space"), None); // No space after #
        assert_eq!(extract_comment("  #  extra spaces  "), Some("extra spaces  ".to_string()));
        assert_eq!(extract_comment("\t# tab indented"), Some("tab indented".to_string()));
        assert_eq!(extract_comment("not # a comment"), None); // # not at start
    }

    #[test]
    fn test_extract_echo() {
        assert_eq!(extract_echo("echo \"hello world\""), Some("hello world".to_string()));
        assert_eq!(extract_echo("    echo 'test'"), Some("test".to_string()));
        assert_eq!(extract_echo("echo"), Some(String::new()));
        assert_eq!(extract_echo("# comment"), None);
        assert_eq!(extract_echo("ls -la"), None);

        // Edge cases
        assert_eq!(extract_echo("echo "), Some(String::new())); // echo with just space
        assert_eq!(extract_echo("echo unquoted text"), Some("unquoted text".to_string()));
        assert_eq!(extract_echo("echo \"\""), Some(String::new())); // empty quotes
        assert_eq!(extract_echo("echo ''"), Some(String::new())); // empty single quotes
        assert_eq!(extract_echo("echo \"with \\\"escaped\\\" quotes\""), Some("with \"escaped\" quotes".to_string()));
        assert_eq!(extract_echo("echo \"$var and `cmd`\""), Some("$var and `cmd`".to_string()));
        assert_eq!(extract_echo("  echo  \"  spaced  \"  "), Some("  spaced  ".to_string()));
        assert_eq!(extract_echo("echoing"), None); // Not exactly "echo"
        assert_eq!(extract_echo("echo-like"), None); // Not exactly "echo"
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
            "", // empty string
        ];

        for original in test_cases {
            let escaped = escape_for_echo(original);
            let unescaped = unescape_from_echo(&escaped);
            assert_eq!(original, unescaped, "Failed roundtrip for: {}", original);
        }
    }

    #[test]
    fn test_bidirectional_conversion() {
        let comment_line = "    # Hello world";
        let echo_line = process_comment_to_echo(comment_line);
        assert_eq!(echo_line, "    echo \"Hello world\"");

        let back_to_comment = process_echo_to_comment(&echo_line);
        assert_eq!(back_to_comment, "    # Hello world");
    }

    #[test]
    fn test_process_comment_to_echo() {
        assert_eq!(process_comment_to_echo("# test"), "echo \"test\"");
        assert_eq!(process_comment_to_echo("  # indented"), "  echo \"indented\"");
        assert_eq!(process_comment_to_echo("#"), "echo \"\"");
        assert_eq!(process_comment_to_echo("not a comment"), "not a comment");
        assert_eq!(process_comment_to_echo("echo already"), "echo already");
    }

    #[test]
    fn test_process_echo_to_comment() {
        assert_eq!(process_echo_to_comment("echo \"test\""), "# test");
        assert_eq!(process_echo_to_comment("  echo 'indented'"), "  # indented");
        assert_eq!(process_echo_to_comment("echo"), "#");
        assert_eq!(process_echo_to_comment("not an echo"), "not an echo");
        assert_eq!(process_echo_to_comment("# already comment"), "# already comment");
    }

    #[test]
    fn test_special_characters_in_comments() {
        // Test comments with special bash characters
        let special_comment = "# File: $HOME/test & echo \"hello\"";
        let echo_line = process_comment_to_echo(special_comment);
        assert_eq!(echo_line, "echo \"File: \\$HOME/test & echo \\\"hello\\\"\"");

        let back_to_comment = process_echo_to_comment(&echo_line);
        assert_eq!(back_to_comment, "# File: $HOME/test & echo \"hello\"");
    }

    #[test]
    fn test_empty_and_whitespace() {
        assert_eq!(process_comment_to_echo(""), "");
        assert_eq!(process_comment_to_echo("   "), "   ");
        assert_eq!(process_echo_to_comment(""), "");
        assert_eq!(process_echo_to_comment("   "), "   ");
    }
}
