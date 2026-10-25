use core::fmt;
use std::borrow::Cow;

use crate::secret::wipe_string;

#[derive(Clone, PartialEq, Eq)]
pub struct FieldError {
    pub message: Cow<'static, str>,
    pub code: Option<&'static str>,
}

impl Drop for FieldError {
    fn drop(&mut self) {
        let message = core::mem::replace(&mut self.message, Cow::Borrowed(""));
        if let Cow::Owned(message) = message {
            wipe_string(message);
        }
    }
}

impl FieldError {
    pub fn new(message: impl Into<Cow<'static, str>>) -> Self {
        FieldError {
            message: message.into(),
            code: None,
        }
    }

    pub fn coded(message: impl Into<Cow<'static, str>>, code: &'static str) -> Self {
        FieldError {
            message: message.into(),
            code: Some(code),
        }
    }
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl fmt::Debug for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FieldError")
            .field("message", &"[redacted]")
            .field("code", &self.code)
            .finish()
    }
}

impl core::error::Error for FieldError {}

pub trait Validate {
    fn check(&self, s: &str) -> Result<(), FieldError>;
}

impl<F: Fn(&str) -> Result<(), FieldError>> Validate for F {
    fn check(&self, s: &str) -> Result<(), FieldError> {
        self(s)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoValidate;

impl Validate for NoValidate {
    fn check(&self, _s: &str) -> Result<(), FieldError> {
        Ok(())
    }
}
