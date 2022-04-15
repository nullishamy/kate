use thiserror::Error;

#[derive(Debug)]
pub struct InternalData {
    pub source_file: Option<String>,
    pub message: Option<String>,
}

impl InternalData {
    pub fn with_file(source_file: &str, message: &str) -> Self {
        Self {
            source_file: Some(source_file.to_string()),
            message: Some(message.to_string()),
        }
    }

    pub fn with_message(message: &str) -> Self {
        Self {
            source_file: None,
            message: Some(message.to_string()),
        }
    }

    pub fn unknown() -> Self {
        Self {
            source_file: None,
            message: None,
        }
    }

    pub fn message(&self) -> String {
        (&self.message)
            .as_ref()
            .unwrap_or(&"unknown".to_string())
            .to_owned()
    }

    pub fn file(&self) -> String {
        (&self.source_file)
            .as_ref()
            .unwrap_or(&"unknown".to_string())
            .to_owned()
    }
}

#[derive(Error, Debug)]
pub enum InternalError {
    #[error("classfile '{}' was incorrectly formatted (reason: {})", .0.file(), .0.message())]
    ClassFileFormat(InternalData),
}
