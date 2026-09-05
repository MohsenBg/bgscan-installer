use std::fmt;

#[derive(Debug)]
pub struct UnsupportedPlatformError(pub String);

impl fmt::Display for UnsupportedPlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unsupported operating system: {}", self.0)
    }
}

impl std::error::Error for UnsupportedPlatformError {}

#[derive(Debug)]
pub struct UnsupportedArchError(pub String);

impl fmt::Display for UnsupportedArchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unsupported architecture: {}", self.0)
    }
}

impl std::error::Error for UnsupportedArchError {}
