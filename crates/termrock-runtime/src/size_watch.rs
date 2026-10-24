use crate::event::Input;

/// Reconciles actual terminal dimensions with delivered resize events.
///
/// This pure helper deduplicates resize notifications. It does not query the
/// terminal or dispatch inputs; the session adapter owns those operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SizeWatch {
    known: Option<(u16, u16)>,
}

impl SizeWatch {
    /// Create a watcher with an optional initial terminal size.
    pub fn new(size: Option<(u16, u16)>) -> Self {
        Self { known: size }
    }

    /// Record a size carried by an input. Other input kinds leave the size alone.
    pub fn observe(&mut self, input: &Input) {
        if let Input::Resize(width, height) = input {
            self.known = Some((*width, *height));
        }
    }

    /// Return one resize input when the actual dimensions differ from the last
    /// known size, including when no initial size was available.
    pub fn check(&mut self, actual: (u16, u16)) -> Option<Input> {
        if self.known == Some(actual) {
            return None;
        }
        self.known = Some(actual);
        Some(Input::Resize(actual.0, actual.1))
    }
}

#[cfg(test)]
mod tests {
    use super::SizeWatch;
    use crate::event::Input;

    #[test]
    fn tracks_actual_size_once_and_observes_only_resize_inputs() {
        let mut watch = SizeWatch::new(Some((120, 40)));
        assert_eq!(watch.check((120, 40)), None);
        watch.observe(&Input::Tick);
        assert_eq!(watch.check((120, 40)), None);
        assert_eq!(watch.check((100, 30)), Some(Input::Resize(100, 30)));
        assert_eq!(watch.check((100, 30)), None);
        watch.observe(&Input::Resize(72, 20));
        assert_eq!(watch.check((72, 20)), None);
    }

    #[test]
    fn unknown_and_zero_sizes_are_reconciled_without_special_cases() {
        let mut watch = SizeWatch::new(None);
        assert_eq!(watch.check((80, 24)), Some(Input::Resize(80, 24)));
        assert_eq!(watch.check((80, 24)), None);
        assert_eq!(watch.check((0, 0)), Some(Input::Resize(0, 0)));
        assert_eq!(watch.check((0, 0)), None);
    }

    #[test]
    fn historical_termrock_runtime_facade_path_remains_available() {
        let mut watch = termrock::runtime::SizeWatch::new(Some((120, 40)));
        assert_eq!(
            watch.check((100, 30)),
            Some(termrock::core::event::Input::Resize(100, 30))
        );
    }
}
