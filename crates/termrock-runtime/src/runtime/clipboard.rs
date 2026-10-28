use core::fmt;

use crate::secret::Secret;

type CopyHook = Box<dyn Fn(&str)>;

#[derive(Default)]
pub struct Clipboard {
    text: Option<Secret>,
    copy_hook: Option<CopyHook>,
}

impl Clipboard {
    pub fn copy(&mut self, text: &str) {
        let mut secret = self.text.take().unwrap_or_default();
        secret.set(text);
        self.text = Some(secret);
        if let Some(hook) = self.copy_hook.as_ref() {
            hook(text);
        }
    }

    pub fn text(&self) -> Option<&str> {
        self.text.as_ref().map(Secret::expose)
    }

    pub fn clear(&mut self) {
        self.text = None;
    }

    pub fn set_copy_hook(&mut self, hook: impl Fn(&str) + 'static) {
        self.copy_hook = Some(Box::new(hook));
    }

    pub fn clear_copy_hook(&mut self) {
        self.copy_hook = None;
    }
}

impl fmt::Debug for Clipboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Clipboard")
            .field("text", &self.text.as_ref().map(|_| "[redacted]"))
            .field("has_copy_hook", &self.copy_hook.is_some())
            .finish()
    }
}
