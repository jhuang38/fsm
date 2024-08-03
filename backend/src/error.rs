use std::env::VarError;
use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;
use std::marker::PhantomData;
use std::process::Termination;

use colored::Color;
use colored::Colorize;
use serde_json::de::Read;

#[derive(Debug)]
pub enum ErrorType {
    ConfigError,
    FilterError,
    FilepathError,
    ReaderError,
    WatcherError,
    SweepError,
    ApplicationError, // generic error type
}

#[derive(Debug)]
pub struct FsmError {
    error_type: ErrorType,
    message: String,
}

impl FsmError {
    pub fn new(error_type: ErrorType, message: String) -> Self {
        Self {
            error_type,
            message,
        }
    }
    pub fn get_error_message(&self) -> &str {
        &self.message
    }
}

// generic errors
impl Display for FsmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted = format!("{:#?} - {}", &self.error_type, &self.message).color(Color::Red);
        write!(f, "{}", formatted)
    }
}

impl Termination for FsmError {
    fn report(self) -> std::process::ExitCode {
        std::process::ExitCode::FAILURE
    }
}

impl From<VarError> for FsmError {
    fn from(err: VarError) -> Self {
        Self::new(ErrorType::ApplicationError, err.to_string())
    }
}

impl From<std::io::Error> for FsmError {
    fn from(err: std::io::Error) -> Self {
        Self::new(ErrorType::ApplicationError, err.to_string())
    }
}

impl From<serde_json::Error> for FsmError {
    fn from(err: serde_json::Error) -> Self {
        Self::new(ErrorType::ApplicationError, err.to_string())
    }
}
