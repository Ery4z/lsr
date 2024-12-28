
use std::{fmt, path::PathBuf};

#[derive(Debug)]
pub enum FSError {
    PathNotFound(PathBuf),
    PathIsNotADirectory(PathBuf),
    PermissionError(PathBuf),
    PathEncodingError(PathBuf),
    UnknownError(String),
}

impl fmt::Display for FSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FSError::PathNotFound(path) => write!(f, "Path not found: {}", path.display()),
            FSError::PathIsNotADirectory(path) => {
                write!(f, "Path is not a directory: {}", path.display())
            }
            FSError::PermissionError(path) => {
                write!(f, "Permission denied: {}", path.display())
            }
            FSError::PathEncodingError(path) => write!(
                f,
                "Path encoding error: Path contains invalid UTF-8: {}",
                path.display()
            ),
            FSError::UnknownError(err) => write!(f, "Unknown error: {}", err),
        }
    }
}

impl std::error::Error for FSError {}
