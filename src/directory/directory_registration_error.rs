use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum DirectoryRegistrationError {
    Conflict,
}

impl Error for DirectoryRegistrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for DirectoryRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectoryRegistrationError::Conflict => {
                write!(f, "id is already registered at directory")
            }
        }
    }
}
