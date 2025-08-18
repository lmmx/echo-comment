use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

macro_rules! debug {
    ($($arg:tt)*) => {
        if std::env::var("ECHO_COMMENT_DEBUG").is_ok() {
            eprintln!($($arg)*);
        }
    };
}

#[derive(Debug)]
pub enum Mode {
    CommentToEcho,  // echo-comment: comments become echo statements
    EchoToComment,  // comment-echo: echo statements become comments
}

pub fn run_script(script_path: &str, script_args: &[String], mode: Mode) -> Result<(), Box<dyn std::error::Error>> {
    debug!("DEBUG: Processing script: {}", script_path);
    debug!("DEBUG: Mode: {:?}", mode);

    // Read the input script
    let content = fs::read_to_string(script_path).map_err(|e| {
        format!("Failed to read script '{}': {}", script_path, e)
    })?;

    debug!("DEBUG: Script content:\n{}", content);
    
    // Create a temporary file for the processed script
    let mut temp_file = NamedTempFile::new()?;
    
    // Always start with a proper shebang
    writeln!(temp_file, "#!/usr/bin/env bash")?;
    
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
        
        let processed_line = match mode {
            Mode::CommentToEcho => process_comment_to_echo(line),
            Mode::EchoToComment => process_echo_to_comment(line),
        };

        debug!("{} -> {}", line, processed_line);

        if let Err(e) = writeln!(temp_file, "{}", processed_line) {
            debug!("ERROR writing to temp file: {}", e);
            return Err(e.into());
        }
    }

    debug!("Finished processing all lines");

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
        match comment {
            CommentType::Regular(content) => {
                // Convert regular comments to echo statements
                let indent = get_indent(line);
                format!("{}echo \"{}\"", indent, escape_for_echo(&content))
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
                format!("{}echo \"{}\"", indent, escape_for_echo(&echo_content))
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
        
        // Check if the echo content starts with "# " - if so, escape it
        if let Some(content) = echo_content.strip_prefix("# ") {
            // Remove "# " prefix
            if content.is_empty() {
                format!("{}#\\#", indent)
            } else {
                format!("{}#\\# {}", indent, content)
            }
        } else if echo_content == "#" {
            format!("{}#\\#", indent)
        } else if echo_content.is_empty() {
            format!("{}#", indent)
        } else {
            format!("{}# {}", indent, echo_content)
        }
    } else {
        // Keep other lines as-is
        line.to_string()
    }
}

#[derive(Debug, PartialEq)]
enum CommentType {
    Regular(String),      // # comment -> echo "comment"
    NoEcho(String),       // ## comment -> # comment (no echo)
    EscapedHash(String),  // #\# comment -> echo "# comment"
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
    
    // Match exactly "echo" followed by end-of-string or whitespace
    if trimmed == "echo" {
        return Some(String::new());
    }
    
    // Match "echo " (with space) for content after
    if let Some(rest) = trimmed.strip_prefix("echo ") {
        let content = rest.trim();
        
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
    fn test_extract_comment_types() {
        // Regular comments (will be echoed)
        assert_eq!(extract_comment("# hello world"), Some(CommentType::Regular("hello world".to_string())));
        assert_eq!(extract_comment("    # indented comment"), Some(CommentType::Regular("indented comment".to_string())));
        assert_eq!(extract_comment("#"), Some(CommentType::Regular(String::new())));
        
        // No-echo comments (## -> #)
        assert_eq!(extract_comment("## private comment"), Some(CommentType::NoEcho("private comment".to_string())));
        assert_eq!(extract_comment("  ## indented private"), Some(CommentType::NoEcho("indented private".to_string())));
        assert_eq!(extract_comment("##"), Some(CommentType::NoEcho(String::new())));
        
        // Escaped hash comments (#\# -> echo "#")
        assert_eq!(extract_comment("#\\# with hash"), Some(CommentType::EscapedHash("with hash".to_string())));
        assert_eq!(extract_comment("  #\\# indented hash"), Some(CommentType::EscapedHash("indented hash".to_string())));
        assert_eq!(extract_comment("#\\#"), Some(CommentType::EscapedHash(String::new())));
        
        // Non-comments
        assert_eq!(extract_comment("echo hello"), None);
        assert_eq!(extract_comment("#!/bin/bash"), None);
        assert_eq!(extract_comment("#no space"), None);
        assert_eq!(extract_comment("not # a comment"), None);
    }

    #[test]
    fn test_extract_echo() {
        assert_eq!(extract_echo("echo \"hello world\""), Some("hello world".to_string()));
        assert_eq!(extract_echo("    echo 'test'"), Some("test".to_string()));
        assert_eq!(extract_echo("echo"), Some(String::new()));
        assert_eq!(extract_echo("# comment"), None);
        assert_eq!(extract_echo("ls -la"), None);
        
        // Edge cases
        assert_eq!(extract_echo("echo "), Some(String::new()));
        assert_eq!(extract_echo("echo unquoted text"), Some("unquoted text".to_string()));
        assert_eq!(extract_echo("echo \"\""), Some(String::new()));
        assert_eq!(extract_echo("echo ''"), Some(String::new()));
        assert_eq!(extract_echo("echo \"with \\\"escaped\\\" quotes\""), Some("with \"escaped\" quotes".to_string()));
        assert_eq!(extract_echo("echo \"$var and `cmd`\""), Some("$var and `cmd`".to_string()));
        assert_eq!(extract_echo("  echo  \"  spaced  \"  "), Some("  spaced  ".to_string()));
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
    fn test_process_comment_to_echo() {
        // Regular comments become echoes
        assert_eq!(process_comment_to_echo("# test"), "echo \"test\"");
        assert_eq!(process_comment_to_echo("  # indented"), "  echo \"indented\"");
        assert_eq!(process_comment_to_echo("#"), "echo \"\"");
        
        // ## comments become # comments (no echo)
        assert_eq!(process_comment_to_echo("## private"), "# private");
        assert_eq!(process_comment_to_echo("  ## indented private"), "  # indented private");
        assert_eq!(process_comment_to_echo("##"), "#");
        
        // #\# comments become echo "# content"
        assert_eq!(process_comment_to_echo("#\\# with hash"), "echo \"# with hash\"");
        assert_eq!(process_comment_to_echo("  #\\# indented hash"), "  echo \"# indented hash\"");
        assert_eq!(process_comment_to_echo("#\\#"), "echo \"#\"");
        
        // Non-comments stay the same
        assert_eq!(process_comment_to_echo("not a comment"), "not a comment");
        assert_eq!(process_comment_to_echo("echo already"), "echo already");
    }

    #[test]
    fn test_process_echo_to_comment() {
        // Regular echoes become comments
        assert_eq!(process_echo_to_comment("echo \"test\""), "# test");
        assert_eq!(process_echo_to_comment("  echo 'indented'"), "  # indented");
        assert_eq!(process_echo_to_comment("echo"), "#");
        
        // Echoes that start with "# " become #\# comments
        assert_eq!(process_echo_to_comment("echo \"# with hash\""), "#\\# with hash");
        assert_eq!(process_echo_to_comment("  echo '# indented hash'"), "  #\\# indented hash");
        assert_eq!(process_echo_to_comment("echo \"#\""), "#\\#");
        
        // Non-echoes stay the same
        assert_eq!(process_echo_to_comment("not an echo"), "not an echo");
        assert_eq!(process_echo_to_comment("# already comment"), "# already comment");
    }

    #[test]
    fn test_bidirectional_conversion() {
        // Regular comment -> echo -> comment
        let comment_line = "    # Hello world";
        let echo_line = process_comment_to_echo(comment_line);
        assert_eq!(echo_line, "    echo \"Hello world\"");
        let back_to_comment = process_echo_to_comment(&echo_line);
        assert_eq!(back_to_comment, "    # Hello world");
        
        // No-echo comment -> stays comment
        let no_echo_comment = "    ## Private note";
        let processed = process_comment_to_echo(no_echo_comment);
        assert_eq!(processed, "    # Private note");
        
        // Escaped hash comment -> echo with hash -> escaped comment
        let escaped_comment = "    #\\# Show hash";
        let echo_with_hash = process_comment_to_echo(escaped_comment);
        assert_eq!(echo_with_hash, "    echo \"# Show hash\"");
        let back_to_escaped = process_echo_to_comment(&echo_with_hash);
        assert_eq!(back_to_escaped, "    #\\# Show hash");
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

    #[test]
    fn test_edge_cases_with_escapes() {
        // Test whitespace handling in extracted content
        assert_eq!(extract_comment("#  extra spaces  "), Some(CommentType::Regular("extra spaces  ".to_string())));
        assert_eq!(extract_comment("##  extra spaces  "), Some(CommentType::NoEcho("extra spaces  ".to_string())));
        assert_eq!(extract_comment("#\\#  extra spaces  "), Some(CommentType::EscapedHash("extra spaces  ".to_string())));
        
        // Test tab indentation
        assert_eq!(extract_comment("\t# tab indented"), Some(CommentType::Regular("tab indented".to_string())));
        assert_eq!(extract_comment("\t## tab private"), Some(CommentType::NoEcho("tab private".to_string())));
        assert_eq!(extract_comment("\t#\\# tab hash"), Some(CommentType::EscapedHash("tab hash".to_string())));
    }
}
