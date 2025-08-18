use echo_comment::{EchoCommentError, Result};
use std::error::Error;
use std::io;

#[test]
fn test_error_display_messages() {
    let file_read_error = EchoCommentError::FileRead {
        path: "/test/path.sh".to_string(),
        source: io::Error::new(io::ErrorKind::NotFound, "No such file"),
    };

    let display_msg = format!("{}", file_read_error);
    assert!(display_msg.contains("Failed to read script '/test/path.sh'"));
    assert!(display_msg.contains("No such file"));

    let file_write_error = EchoCommentError::FileWrite {
        source: io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied"),
    };

    let display_msg = format!("{}", file_write_error);
    assert!(display_msg.contains("Failed to write processed script"));
    assert!(display_msg.contains("Permission denied"));

    let script_exec_error = EchoCommentError::ScriptExecution {
        message: "Command failed".to_string(),
        source: io::Error::new(io::ErrorKind::Other, "Process error"),
    };

    let display_msg = format!("{}", script_exec_error);
    assert!(display_msg.contains("Command failed"));
    assert!(display_msg.contains("Process error"));
}

#[test]
fn test_error_source_chain() {
    let inner_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let error = EchoCommentError::FileRead {
        path: "/test".to_string(),
        source: inner_error,
    };

    // Test that we can access the source error
    assert!(error.source().is_some());
    let source = error.source().unwrap();
    assert_eq!(source.to_string(), "File not found");
}

#[test]
fn test_result_type_alias() {
    // Test that our Result type alias works
    fn returns_result() -> Result<String> {
        Ok("success".to_string())
    }

    fn returns_error() -> Result<String> {
        Err(EchoCommentError::FileWrite {
            source: io::Error::new(io::ErrorKind::Other, "test error"),
        })
    }

    assert!(returns_result().is_ok());
    assert!(returns_error().is_err());
}

#[test]
fn test_error_debug_format() {
    let error = EchoCommentError::TempFileCreation {
        source: io::Error::new(io::ErrorKind::Other, "temp error"),
    };

    let debug_output = format!("{:?}", error);
    assert!(debug_output.contains("TempFileCreation"));
    assert!(debug_output.contains("temp error"));
}

#[cfg(unix)]
#[test]
fn test_unix_permission_error() {
    let error = EchoCommentError::PermissionSet {
        source: io::Error::new(io::ErrorKind::PermissionDenied, "chmod failed"),
    };

    let display_msg = format!("{}", error);
    assert!(display_msg.contains("Failed to set executable permissions"));
    assert!(display_msg.contains("chmod failed"));
}
