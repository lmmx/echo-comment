use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

/// Helper to create a temporary script file with given content
fn create_temp_script(content: &str) -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(content.as_bytes())
        .expect("Failed to write to temp file");
    temp_file
}

/// Helper to run a binary and capture its output
fn run_binary(binary_name: &str, script_path: &str) -> (i32, String, String) {
    let output = Command::new(format!("target/debug/{}", binary_name))
        .arg(script_path)
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    (exit_code, stdout, stderr)
}

#[test]
fn test_echo_comment_basic_functionality() {
    let script_content = r#"#!/usr/bin/env bash
# Starting process
echo "Actual command"
# Finishing process
"#;

    let temp_script = create_temp_script(script_content);
    let script_path = temp_script.path().to_str().unwrap();

    let (exit_code, stdout, stderr) = run_binary("echo-comment", script_path);

    assert_eq!(
        exit_code, 0,
        "Script should exit successfully. stderr: {}",
        stderr
    );
    assert!(
        stdout.contains("Starting process"),
        "Should echo the first comment"
    );
    assert!(
        stdout.contains("Actual command"),
        "Should preserve existing echo"
    );
    assert!(
        stdout.contains("Finishing process"),
        "Should echo the last comment"
    );
}

#[test]
fn test_comment_echo_basic_functionality() {
    let script_content = r#"#!/usr/bin/env bash
echo "This will become a comment"
ls -la
echo "Another comment"
"#;

    let temp_script = create_temp_script(script_content);
    let script_path = temp_script.path().to_str().unwrap();

    let (exit_code, stdout, stderr) = run_binary("comment-echo", script_path);

    assert_eq!(
        exit_code, 0,
        "Script should exit successfully. stderr: {}",
        stderr
    );
    // The echo statements should be converted to comments, so we shouldn't see their output
    assert!(!stdout.contains("This will become a comment"));
    assert!(!stdout.contains("Another comment"));
    // But ls output should still be there (assuming current directory has files)
}

#[test]
fn test_no_echo_comments() {
    let script_content = r#"#!/usr/bin/env bash
## This should not be echoed
# This should be echoed
echo "Command output"
"#;

    let temp_script = create_temp_script(script_content);
    let script_path = temp_script.path().to_str().unwrap();

    let (exit_code, stdout, stderr) = run_binary("echo-comment", script_path);

    assert_eq!(
        exit_code, 0,
        "Script should exit successfully. stderr: {}",
        stderr
    );
    assert!(
        !stdout.contains("This should not be echoed"),
        "## comments should not be echoed"
    );
    assert!(
        stdout.contains("This should be echoed"),
        "# comments should be echoed"
    );
    assert!(
        stdout.contains("Command output"),
        "Regular commands should work"
    );
}

#[test]
fn test_escaped_hash_comments() {
    let script_content = r#"#!/usr/bin/env bash
#\# This should echo with a hash like this
echo "Normal output"
"#;

    let temp_script = create_temp_script(script_content);
    let script_path = temp_script.path().to_str().unwrap();

    let (exit_code, stdout, stderr) = run_binary("echo-comment", script_path);

    assert_eq!(
        exit_code, 0,
        "Script should exit successfully. stderr: {}",
        stderr
    );
    assert!(
        stdout.contains("# This should echo with a hash like this"),
        "Escaped hash comments should echo with hash prefix"
    );
    assert!(
        stdout.contains("Normal output"),
        "Regular commands should work"
    );
}

#[test]
fn test_bidirectional_conversion() {
    // Test that comment -> echo -> comment works correctly
    let original_content = r###"#!/usr/bin/env bash
# Step 1: Initialize
some_command
## Private note (should not be echoed)
# Step 2: Process
#\# Hash comment with hash
"###;

    let temp_script = create_temp_script(original_content);
    let script_path = temp_script.path().to_str().unwrap();

    // First convert comments to echoes
    let (exit_code1, _stdout1, stderr1) = run_binary("echo-comment", script_path);
    assert_eq!(
        exit_code1, 0,
        "First conversion should succeed. stderr: {}",
        stderr1
    );

    // Create a new script with echo statements
    let echo_script_content = r###"#!/usr/bin/env bash
echo "Step 1: Initialize"
true
# Private note (should not be echoed)
echo "Step 2: Process"
echo "# Hash comment with hash"
"###;

    let temp_echo_script = create_temp_script(echo_script_content);
    let echo_script_path = temp_echo_script.path().to_str().unwrap();

    // Convert echoes back to comments
    let (exit_code2, _stdout2, stderr2) = run_binary("comment-echo", echo_script_path);
    assert_eq!(
        exit_code2, 0,
        "Second conversion should succeed. stderr: {}",
        stderr2
    );

    // The outputs should match in terms of what gets printed
    // (though the exact format might differ due to shebang handling)
}

#[test]
fn test_script_arguments_passthrough() {
    let script_content = r#"#!/usr/bin/env bash
# Script started with args: $@
echo "Arg 1: $1"
echo "Arg 2: $2"
# Script completed
"#;

    let temp_script = create_temp_script(script_content);
    let script_path = temp_script.path().to_str().unwrap();

    // Run with arguments
    let output = Command::new("target/debug/echo-comment")
        .arg(script_path)
        .arg("hello")
        .arg("world")
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    assert_eq!(
        exit_code, 0,
        "Script should exit successfully. stderr: {}",
        stderr
    );
    assert!(
        stdout.contains("Script started with args:"),
        "Should echo start comment"
    );
    assert!(
        stdout.contains("Arg 1: hello"),
        "Should pass first argument"
    );
    assert!(
        stdout.contains("Arg 2: world"),
        "Should pass second argument"
    );
    assert!(
        stdout.contains("Script completed"),
        "Should echo end comment"
    );
}

#[test]
fn test_error_handling_missing_file() {
    let (exit_code, _stdout, stderr) = run_binary("echo-comment", "/nonexistent/file.sh");

    assert_ne!(exit_code, 0, "Should exit with error code for missing file");
    assert!(
        stderr.contains("Failed to read script"),
        "Should show error message"
    );
}

#[test]
fn test_error_handling_no_arguments() {
    let output = Command::new("target/debug/echo-comment")
        .output()
        .expect("Failed to execute binary");

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    assert_eq!(
        exit_code, 1,
        "Should exit with code 1 for missing arguments"
    );
    assert!(stderr.contains("Usage:"), "Should show usage message");
}

#[test]
fn test_special_characters_in_comments() {
    let script_content = r#"#!/usr/bin/env bash
# Comment with "quotes" and $variables and `backticks`
# Comment with emojis: 🚀 🎉 ✅
# Comment with special chars: & | > < * ? [ ] { } ( )
"#;

    let temp_script = create_temp_script(script_content);
    let script_path = temp_script.path().to_str().unwrap();

    let (exit_code, stdout, stderr) = run_binary("echo-comment", script_path);

    assert_eq!(
        exit_code, 0,
        "Script should exit successfully. stderr: {}",
        stderr
    );
    assert!(
        stdout.contains("quotes"),
        "Should handle quotes in comments"
    );
    assert!(
        stdout.contains("variables"),
        "Should handle dollar signs in comments"
    );
    assert!(
        stdout.contains("backticks"),
        "Should handle backticks in comments"
    );
    assert!(stdout.contains("🚀"), "Should handle emojis in comments");
    assert!(
        stdout.contains("& |"),
        "Should handle special shell characters"
    );
}

#[test]
fn test_empty_and_whitespace_lines() {
    let script_content = "#!/usr/bin/env bash\n\n# Comment\n\n    \necho 'test'\n\n";

    let temp_script = create_temp_script(script_content);
    let script_path = temp_script.path().to_str().unwrap();

    let (exit_code, stdout, stderr) = run_binary("echo-comment", script_path);

    assert_eq!(
        exit_code, 0,
        "Script should exit successfully. stderr: {}",
        stderr
    );
    assert!(
        stdout.contains("Comment"),
        "Should handle comments with empty lines around"
    );
    assert!(
        stdout.contains("test"),
        "Should handle commands with empty lines around"
    );
}
