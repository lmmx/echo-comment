use facet::Facet;
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[derive(Facet)]
struct Args {
    /// Script file to process and execute
    #[facet(positional)]
    script: String,
}

#[derive(Debug)]
enum Mode {
    CommentToEcho,  // comment-echo: comments become echo statements
    EchoToComment,  // echo-comment: echo statements become comments
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments - everything after the script file goes to the script
    let env_args: Vec<String> = std::env::args().collect();
    
    if env_args.len() < 2 {
        eprintln!("Usage: {} <script.sh> [args...]", env_args[0]);
        eprintln!("A bidirectional bash interpreter: converts comments ↔ echo statements");
        std::process::exit(1);
    }

    let script = env_args[1].clone();
    let script_args = env_args[2..].to_vec();

    // Determine mode based on how the binary was invoked
    let mode = determine_mode()?;

    // Read the input script
    let content = fs::read_to_string(&script).map_err(|e| {
        format!("Failed to read script '{}': {}", script, e)
    })?;
    
    // Create a temporary file for the processed script
    let mut temp_file = NamedTempFile::new()?;
    
    // Process each line based on mode
    for line in content.lines() {
        let processed_line = if line.starts_with("#!") {
            // Keep shebang as-is but ensure it's bash
            "#!/usr/bin/env bash".to_string()
        } else {
            match mode {
                Mode::CommentToEcho => process_comment_to_echo(line),
                Mode::EchoToComment => process_echo_to_comment(line),
            }
        };
        
        writeln!(temp_file, "{}", processed_line)?;
    }

    // Ensure the temp file is flushed and closed
    temp_file.flush()?;
    
    // Get the path before we close the file
    let temp_path = temp_file.path().to_owned();
    
    // Close the file explicitly to release the write handle
    drop(temp_file);

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
    cmd.args(&script_args);
    
    let status = cmd.status().map_err(|e| {
        format!("Failed to execute processed script: {}", e)
    })?;

    // The temp file will be automatically cleaned up when temp_path goes out of scope
    // (NamedTempFile cleans up based on the file path even after being dropped)

    // Exit with the same code as the script
    std::process::exit(status.code().unwrap_or(1));
}

fn determine_mode() -> Result<Mode, Box<dyn std::error::Error>> {
    let exe_path = std::env::current_exe()?;
    let exe_name = exe_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("comment-echo");

    match exe_name {
        "echo-comment" => Ok(Mode::EchoToComment),
        "comment-echo" => Ok(Mode::CommentToEcho),
        _ => {
            // If symlinked or renamed, try to detect from argv[0]
            let arg0 = std::env::args().next().unwrap_or_default();
            if arg0.contains("echo-comment") {
                Ok(Mode::EchoToComment)
            } else {
                Ok(Mode::CommentToEcho)
            }
        }
    }
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
    
    // Match: echo "content" or echo 'content'
    if let Some(rest) = trimmed.strip_prefix("echo ") {
        let rest = rest.trim();
        
        // Handle quoted strings
        if (rest.starts_with('"') && rest.ends_with('"')) || 
           (rest.starts_with('\'') && rest.ends_with('\'')) {
            let content = &rest[1..rest.len()-1];
            Some(unescape_from_echo(content))
        } else if rest.is_empty() {
            Some(String::new())
        } else {
            // Unquoted echo - take everything after "echo "
            Some(rest.to_string())
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
    }

    #[test]
    fn test_extract_echo() {
        assert_eq!(extract_echo("echo \"hello world\""), Some("hello world".to_string()));
        assert_eq!(extract_echo("    echo 'test'"), Some("test".to_string()));
        assert_eq!(extract_echo("echo"), Some(String::new()));
        assert_eq!(extract_echo("# comment"), None);
        assert_eq!(extract_echo("ls -la"), None);
    }

    #[test]
    fn test_get_indent() {
        assert_eq!(get_indent("# comment"), "");
        assert_eq!(get_indent("    # indented"), "    ");
        assert_eq!(get_indent("\t# tabbed"), "\t");
    }

    #[test]
    fn test_escape_unescape_roundtrip() {
        let original = "text with \"quotes\" and $vars and `commands`";
        let escaped = escape_for_echo(original);
        let unescaped = unescape_from_echo(&escaped);
        assert_eq!(original, unescaped);
    }

    #[test]
    fn test_bidirectional_conversion() {
        // Comment to echo to comment
        let comment_line = "    # Hello world";
        let echo_line = process_comment_to_echo(comment_line);
        assert_eq!(echo_line, "    echo \"Hello world\"");
        
        let back_to_comment = process_echo_to_comment(&echo_line);
        assert_eq!(back_to_comment, "    # Hello world");
    }
}