use core::fmt;

use crate::secret::Secret;

type CopyHook = Box<dyn Fn(&str)>;

/// Runtime-owned clipboard storage shared by fields in one runtime.
///
/// The in-memory copy is wiped when replaced, cleared, or dropped. Integrations can install
/// a copy hook to forward text to a platform clipboard without making the runtime depend on
/// a particular operating-system clipboard API.
#[derive(Default)]
pub struct Clipboard {
    text: Option<Secret>,
    copy_hook: Option<CopyHook>,
}

impl Clipboard {
    /// Copy text into the runtime clipboard and notify the optional integration hook.
    pub fn copy(&mut self, text: &str) {
        let mut secret = self.text.take().unwrap_or_default();
        secret.set(text);
        self.text = Some(secret);
        if let Some(hook) = self.copy_hook.as_ref() {
            hook(text);
        }
    }

    /// Borrow the current clipboard contents.
    pub fn text(&self) -> Option<&str> {
        self.text.as_ref().map(Secret::expose)
    }

    /// Clear the current clipboard contents, wiping its backing storage.
    pub fn clear(&mut self) {
        self.text = None;
    }

    /// Install a platform or test integration hook. Replaces any existing hook.
    pub fn set_copy_hook(&mut self, hook: impl Fn(&str) + 'static) {
        self.copy_hook = Some(Box::new(hook));
    }

    /// Remove the optional copy hook.
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

#[cfg(test)]
mod tests {
    use super::Clipboard;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn clipboard_storage_is_independent_and_debug_redacts_text() {
        let mut first = Clipboard::default();
        let second = Clipboard::default();
        first.copy("first-field-secret");

        assert_eq!(first.text(), Some("first-field-secret"));
        assert_eq!(second.text(), None);
        assert!(!format!("{first:?}").contains("first-field-secret"));

        first.clear();
        assert_eq!(first.text(), None);
    }

    #[test]
    fn copy_hook_receives_each_copy_and_can_be_cleared() {
        let seen = Rc::new(RefCell::new(String::new()));
        let output = seen.clone();
        let mut clipboard = Clipboard::default();
        clipboard.set_copy_hook(move |text| output.borrow_mut().push_str(text));
        clipboard.copy("abc");
        clipboard.clear_copy_hook();
        clipboard.copy("def");

        assert_eq!(seen.borrow().as_str(), "abc");
        assert_eq!(clipboard.text(), Some("def"));
    }
}
