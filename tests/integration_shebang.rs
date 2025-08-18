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

/// Helper to run echo-comment with specific shell flags
fn run_echo_comment_with_flags(script_path: &str, shell_flags: &str) -> (i32, String, String) {
    let output = Command::new("target/debug/echo-comment")
        .arg(format!("--shell-flags={}", shell_flags))
        .arg(script_path)
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    (exit_code, stdout, stderr)
}

/// Helper to run echo-comment with shell flags and color
fn run_echo_comment_with_flags_and_color(
    script_path: &str,
    shell_flags: &str,
    color: &str,
) -> (i32, String, String) {
    let output = Command::new("target/debug/echo-comment")
        .arg(format!("--shell-flags={}", shell_flags))
        .arg("--color")
        .arg(color)
        .arg(script_path)
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    (exit_code, stdout, stderr)
}

/// Helper to run echo-comment without any flags (normal mode)
fn run_echo_comment_normal(script_path: &str) -> (i32, String, String) {
    let output = Command::new("target/debug/echo-comment")
        .arg(script_path)
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    (exit_code, stdout, stderr)
}

#[test]
fn test_shell_flags_pipefail_behavior() {
    let script_content = r#"#!/usr/bin/env -S echo-comment
# Test comment: this grep call will fail and we won't get the Python
echo "hello" | grep "world"
# hiss
python -c "print('Goodbye')"
"#;

    let temp_script = create_temp_script(script_content);
    let script_path = temp_script.path().to_str().unwrap();

    // Test without shell flags (normal bash behavior)
    let (exit_code_normal, stdout_normal, stderr_normal) = run_echo_comment_normal(script_path);

    // Test with -euo pipefail flags
    let (exit_code_pipefail, stdout_pipefail, stderr_pipefail) =
        run_echo_comment_with_flags(script_path, "-euo pipefail");

    // Normal mode should continue execution despite grep failure
    assert_eq!(
        exit_code_normal, 0,
        "Normal mode should complete successfully. stderr: {}",
        stderr_normal
    );
    assert!(
        stdout_normal
            .contains("Test comment: this grep call will fail and we won't get the Python"),
        "Should echo the first comment in normal mode"
    );
    assert!(
        stdout_normal.contains("hiss"),
        "Should echo the second comment in normal mode"
    );
    assert!(
        stdout_normal.contains("Goodbye"),
        "Should execute Python command in normal mode despite grep failure"
    );

    // Pipefail mode should stop execution after grep failure
    assert_ne!(
        exit_code_pipefail, 0,
        "Pipefail mode should exit with error code after grep failure. stderr: {}",
        stderr_pipefail
    );
    assert!(
        stdout_pipefail
            .contains("Test comment: this grep call will fail and we won't get the Python"),
        "Should echo the first comment in pipefail mode"
    );
    assert!(
        !stdout_pipefail.contains("hiss"),
        "Should NOT echo the second comment in pipefail mode (execution stopped)"
    );
    assert!(
        !stdout_pipefail.contains("Goodbye"),
        "Should NOT execute Python command in pipefail mode (execution stopped)"
    );
}

#[test]
fn test_shell_flags_with_color_option() {
    let script_content = r#"#!/usr/bin/env echo-comment
# Test comment: this grep call will fail and we won't get the Python
echo "hello" | grep "world"
# hiss
python -c "print('Goodbye')"
"#;

    let temp_script = create_temp_script(script_content);
    let script_path = temp_script.path().to_str().unwrap();

    // Test with both shell flags and green color
    let (exit_code, stdout, stderr) =
        run_echo_comment_with_flags_and_color(script_path, "-euo pipefail", "green");

    // Should exit with error due to pipefail
    assert_ne!(
        exit_code, 0,
        "Should exit with error code due to pipefail. stderr: {}",
        stderr
    );

    // Should contain the first comment with green ANSI color codes
    assert!(
        stdout.contains("Test comment: this grep call will fail and we won't get the Python"),
        "Should echo the first comment"
    );

    // Check for green ANSI color codes (\x1b[0;32m for green, \x1b[0m for reset)
    assert!(
        stdout.contains("\x1b[0;32m"),
        "Should contain green ANSI color code in output"
    );
    assert!(
        stdout.contains("\x1b[0m"),
        "Should contain ANSI reset code in output"
    );

    // Should NOT contain subsequent comments/output due to pipefail
    assert!(
        !stdout.contains("hiss"),
        "Should NOT echo the second comment (execution stopped due to pipefail)"
    );
    assert!(
        !stdout.contains("Goodbye"),
        "Should NOT execute Python command (execution stopped due to pipefail)"
    );
}

#[test]
fn test_color_option_without_shell_flags() {
    let script_content = r#"#!/usr/bin/env echo-comment
# This comment should be green
echo "Regular command"
# Another green comment
"#;

    let temp_script = create_temp_script(script_content);
    let script_path = temp_script.path().to_str().unwrap();

    // Test with just color option, no special shell flags
    let output = Command::new("target/debug/echo-comment")
        .arg("--color")
        .arg("green")
        .arg(script_path)
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    assert_eq!(exit_code, 0, "Should exit successfully. stderr: {}", stderr);

    // All comments should be present and colored
    assert!(
        stdout.contains("This comment should be green"),
        "Should echo the first comment"
    );
    assert!(
        stdout.contains("Another green comment"),
        "Should echo the second comment"
    );
    assert!(
        stdout.contains("Regular command"),
        "Should execute regular command"
    );

    // Check for green ANSI color codes
    assert!(
        stdout.contains("\x1b[0;32m"),
        "Should contain green ANSI color code in output"
    );
    assert!(
        stdout.contains("\x1b[0m"),
        "Should contain ANSI reset code in output"
    );
}

#[test]
fn test_multiple_color_options() {
    let script_content = r#"#!/usr/bin/env echo-comment
# Red comment test
echo "Command output"
# Another red comment
"#;

    let temp_script = create_temp_script(script_content);
    let script_path = temp_script.path().to_str().unwrap();

    // Test with red color
    let output = Command::new("target/debug/echo-comment")
        .arg("--color")
        .arg("red")
        .arg(script_path)
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    assert_eq!(exit_code, 0, "Should exit successfully. stderr: {}", stderr);

    // Check for red ANSI color codes (\x1b[0;31m for red)
    assert!(
        stdout.contains("\x1b[0;31m"),
        "Should contain red ANSI color code in output"
    );
    assert!(
        stdout.contains("Red comment test"),
        "Should echo the comments"
    );
    assert!(
        stdout.contains("Another red comment"),
        "Should echo all comments"
    );
}

#[test]
fn test_shell_flags_errexit_behavior() {
    let script_content = r#"#!/usr/bin/env echo-comment
# Starting test
false
# This should not be reached with -e flag
echo "This should not execute"
"#;

    let temp_script = create_temp_script(script_content);
    let script_path = temp_script.path().to_str().unwrap();

    // Test without -e flag (should continue)
    let (exit_code_normal, stdout_normal, _) = run_echo_comment_normal(script_path);

    // Test with -e flag (should exit on false)
    let (exit_code_errexit, stdout_errexit, stderr_errexit) =
        run_echo_comment_with_flags(script_path, "-e");

    // Normal mode should continue despite false command
    assert_eq!(
        exit_code_normal, 0,
        "Normal mode should complete successfully"
    );
    assert!(
        stdout_normal.contains("Starting test"),
        "Should echo first comment in normal mode"
    );
    assert!(
        stdout_normal.contains("This should not be reached with -e flag"),
        "Should echo second comment in normal mode"
    );

    // Errexit mode should stop after false command
    assert_ne!(
        exit_code_errexit, 0,
        "Errexit mode should exit with error code. stderr: {}",
        stderr_errexit
    );
    assert!(
        stdout_errexit.contains("Starting test"),
        "Should echo first comment in errexit mode"
    );
    assert!(
        !stdout_errexit.contains("This should not be reached with -e flag"),
        "Should NOT echo second comment in errexit mode (execution stopped)"
    );
}
