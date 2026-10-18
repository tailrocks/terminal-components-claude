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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closures_and_fn_items_validate() {
        fn rule(s: &str) -> Result<(), FieldError> {
            if s.contains('@') {
                Ok(())
            } else {
                Err(FieldError::coded("Enter a valid address", "email"))
            }
        }
        assert!(Validate::check(&rule, "a@b").is_ok());
        let e = Validate::check(&rule, "ab").err();
        assert_eq!(e.as_ref().map(|e| e.code), Some(Some("email")));
        assert_eq!(
            e.map(|e| e.to_string()),
            Some("Enter a valid address".to_owned())
        );
        let min = |s: &str| {
            if s.len() >= 2 {
                Ok(())
            } else {
                Err(FieldError::new("too short"))
            }
        };
        assert!(min.check("ab").is_ok());
        assert!(NoValidate.check("").is_ok());
    }

    #[test]
    fn debug_redacts_owned_message() {
        let error = FieldError::new("swordfish".to_owned());
        let debug = format!("{error:?}");
        assert!(debug.contains("redacted"));
        assert!(!debug.contains("swordfish"));
    }
}
