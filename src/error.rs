use std::fmt;
use std::io;

pub type Result<T> = std::result::Result<T, EchoCommentError>;

#[derive(Debug)]
pub enum EchoCommentError {
    FileRead {
        path: String,
        source: io::Error,
    },
    FileWrite {
        source: io::Error,
    },
    ScriptExecution {
        message: String,
        source: io::Error,
    },
    TempFileCreation {
        source: io::Error,
    },
    #[cfg(unix)]
    PermissionSet {
        source: io::Error,
    },
}

impl fmt::Display for EchoCommentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EchoCommentError::FileRead { path, source } => {
                write!(f, "Failed to read script '{}': {}", path, source)
            }
            EchoCommentError::FileWrite { source } => {
                write!(f, "Failed to write processed script: {}", source)
            }
            EchoCommentError::ScriptExecution { message, source } => {
                write!(f, "{}: {}", message, source)
            }
            EchoCommentError::TempFileCreation { source } => {
                write!(f, "Failed to create temporary file: {}", source)
            }
            #[cfg(unix)]
            EchoCommentError::PermissionSet { source } => {
                write!(f, "Failed to set executable permissions: {}", source)
            }
        }
    }
}

impl std::error::Error for EchoCommentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EchoCommentError::FileRead { source, .. }
            | EchoCommentError::FileWrite { source }
            | EchoCommentError::ScriptExecution { source, .. }
            | EchoCommentError::TempFileCreation { source } => Some(source),
            #[cfg(unix)]
            EchoCommentError::PermissionSet { source } => Some(source),
        }
    }
}
