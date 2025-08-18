use echo_comment::{Mode, process_script_content};

#[test]
fn test_process_script_content_comprehensive() {
    let input = r###"#!/usr/bin/env bash
set -euo pipefail

# Regular comment that should be echoed
some_command

## Private comment that should not be echoed
another_command

#\# Comment that should echo with hash
final_command

#
"###;

    // Test CommentToEcho mode
    let result = process_script_content(input, Mode::CommentToEcho).unwrap();
    let lines: Vec<&str> = result.lines().collect();

    assert_eq!(lines[0], "#!/usr/bin/env bash");
    assert_eq!(lines[1], "set -euo pipefail");
    assert_eq!(lines[2], "");
    assert_eq!(lines[3], "echo \"Regular comment that should be echoed\"");
    assert_eq!(lines[4], "some_command");
    assert_eq!(lines[5], "");
    assert_eq!(lines[6], "# Private comment that should not be echoed");
    assert_eq!(lines[7], "another_command");
    assert_eq!(lines[8], "");
    assert_eq!(lines[9], "echo \"# Comment that should echo with hash\"");
    assert_eq!(lines[10], "final_command");
    assert_eq!(lines[11], "");
    assert_eq!(lines[12], "echo \"\"");

    // Test EchoToComment mode
    let echo_input = r###"#!/usr/bin/env bash
echo "This becomes a comment"
some_command
echo "# This becomes an escaped comment"
"###;

    let result = process_script_content(echo_input, Mode::EchoToComment).unwrap();
    let lines: Vec<&str> = result.lines().collect();

    assert_eq!(lines[0], "#!/usr/bin/env bash");
    assert_eq!(lines[1], "# This becomes a comment");
    assert_eq!(lines[2], "some_command");
    assert_eq!(lines[3], "#\\# This becomes an escaped comment");
}

#[test]
fn test_shebang_handling() {
    let input_with_bash_shebang = "#!/bin/bash\n# comment\necho test";
    let input_with_env_shebang = "#!/usr/bin/env zsh\n# comment\necho test";
    let input_with_custom_shebang = "#!/usr/bin/env -S bash -euo pipefail\n# comment\necho test";
    let input_without_shebang = "# comment\necho test";

    for input in [
        input_with_bash_shebang,
        input_with_env_shebang,
        input_with_custom_shebang,
        input_without_shebang,
    ] {
        let result = process_script_content(input, Mode::CommentToEcho).unwrap();
        assert!(
            result.starts_with("#!/usr/bin/env bash\n"),
            "Should always use bash shebang regardless of input. Got: {}",
            result.lines().next().unwrap_or("")
        );
    }
}

#[test]
fn test_complex_indentation_scenarios() {
    let input = r#"if condition; then
    # Indented comment
    if nested; then
        ## Deeply indented no-echo
        #\# Deeply indented with hash
    fi
fi"#;

    let result = process_script_content(input, Mode::CommentToEcho).unwrap();
    let lines: Vec<&str> = result.lines().collect();

    // Skip the shebang line
    assert_eq!(lines[1], "if condition; then");
    assert_eq!(lines[2], "    echo \"Indented comment\"");
    assert_eq!(lines[3], "    if nested; then");
    assert_eq!(lines[4], "        # Deeply indented no-echo");
    assert_eq!(lines[5], "        echo \"# Deeply indented with hash\"");
    assert_eq!(lines[6], "    fi");
    assert_eq!(lines[7], "fi");
}

#[test]
fn test_edge_cases_with_special_content() {
    let test_cases = vec![
        ("# ", "echo \"\""),     // Comment with just space
        ("#", "echo \"\""),      // Comment with no space
        ("##", "#"),             // No-echo with no content
        ("## ", "#"),            // No-echo with just space
        ("#\\#", "echo \"#\""),  // Escaped hash with no content
        ("#\\# ", "echo \"#\""), // Escaped hash with just space
    ];

    for (input, expected) in test_cases {
        let result = process_script_content(input, Mode::CommentToEcho).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines[1], expected, "Failed for input: '{}'", input);
    }
}

#[test]
fn test_multiline_echo_scenarios() {
    let input = r#"echo "first line"
echo "second line"
normal_command
echo "third line"
"#;

    let result = process_script_content(input, Mode::EchoToComment).unwrap();
    let lines: Vec<&str> = result.lines().collect();

    assert_eq!(lines[1], "# first line");
    assert_eq!(lines[2], "# second line");
    assert_eq!(lines[3], "normal_command");
    assert_eq!(lines[4], "# third line");
}

#[test]
fn test_mixed_quote_styles() {
    let input = r#"echo 'single quoted'
echo "double quoted"
echo unquoted text
echo ""
echo ''
"#;

    let result = process_script_content(input, Mode::EchoToComment).unwrap();
    let lines: Vec<&str> = result.lines().collect();

    assert_eq!(lines[1], "# single quoted");
    assert_eq!(lines[2], "# double quoted");
    assert_eq!(lines[3], "# unquoted text");
    assert_eq!(lines[4], "#");
    assert_eq!(lines[5], "#");
}

#[test]
fn test_roundtrip_conversion_preserves_meaning() {
    let original_scripts = vec![
        "# Simple comment\ncommand\n## Private note",
        "    # Indented comment\n    command",
        "#\\# Hash comment\necho existing",
        "# Comment with \"quotes\" and $vars",
        "## Multiple\n## Private\n## Comments",
    ];

    for script in original_scripts {
        // Convert to echo, then back to comment
        let echo_version = process_script_content(script, Mode::CommentToEcho).unwrap();
        let back_to_comment = process_script_content(&echo_version, Mode::EchoToComment).unwrap();

        // The meaning should be preserved (though format might differ due to shebang)
        // We'll check that the non-shebang content matches expected patterns
        let original_lines: Vec<&str> = script.lines().collect();
        let final_lines: Vec<&str> = back_to_comment.lines().skip(1).collect(); // Skip added shebang

        // For regular comments, the roundtrip should preserve them
        for (i, original_line) in original_lines.iter().enumerate() {
            if original_line.trim_start().starts_with("# ") {
                assert_eq!(
                    final_lines.get(i),
                    Some(original_line),
                    "Regular comment not preserved in roundtrip"
                );
            }
        }
    }
}

#[test]
fn test_error_conditions() {
    // Empty string should work
    let result = process_script_content("", Mode::CommentToEcho);
    assert!(result.is_ok());

    // Very long content should work
    let long_content = "# comment\n".repeat(1000);
    let result = process_script_content(&long_content, Mode::CommentToEcho);
    assert!(result.is_ok());
}

#[test]
fn test_whitespace_preservation() {
    let input = "\t# Tab indented\n  # Space indented\n\t  # Mixed indented";

    let result = process_script_content(input, Mode::CommentToEcho).unwrap();
    let lines: Vec<&str> = result.lines().collect();

    assert_eq!(lines[1], "\techo \"Tab indented\"");
    assert_eq!(lines[2], "  echo \"Space indented\"");
    assert_eq!(lines[3], "\t  echo \"Mixed indented\"");
}
