//! Small decimal readouts used while painting. No heap-backed formatting.

/// Decimal `usize`, optionally followed by the fixed selection-count suffix.
/// Thirty-two bytes hold every supported `usize` (at most 20 digits) plus 9 suffix bytes.
#[derive(Debug)]
pub(crate) struct RenderNumber {
    bytes: [u8; 32],
    start: usize,
}

impl RenderNumber {
    pub(crate) fn new(value: usize) -> Self {
        Self::with_selected_suffix(value, false)
    }

    pub(crate) fn selected(value: usize) -> Self {
        Self::with_selected_suffix(value, true)
    }

    fn with_selected_suffix(mut value: usize, selected: bool) -> Self {
        let suffix = if selected {
            b" selected".as_slice()
        } else {
            &[]
        };
        let mut result = Self {
            bytes: [0; 32],
            start: 32_usize.saturating_sub(suffix.len()),
        };
        if let Some(tail) = result.bytes.get_mut(result.start..) {
            tail.copy_from_slice(suffix);
        }
        loop {
            result.start = result.start.saturating_sub(1);
            if let Some(slot) = result.bytes.get_mut(result.start) {
                *slot = b'0'.saturating_add(u8::try_from(value % 10).unwrap_or_default());
            }
            value /= 10;
            if value == 0 {
                break;
            }
        }
        result
    }

    pub(crate) fn as_str(&self) -> &str {
        self.bytes
            .get(self.start..)
            .and_then(|bytes| core::str::from_utf8(bytes).ok())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::RenderNumber;

    #[test]
    fn zero_dimensions_and_maximum_counts_fit_without_truncation() {
        for value in [0, 9, 10, 80, 160, usize::from(u16::MAX), usize::MAX] {
            assert_eq!(RenderNumber::new(value).as_str(), value.to_string());
            assert_eq!(
                RenderNumber::selected(value).as_str(),
                format!("{value} selected")
            );
        }
    }
}
