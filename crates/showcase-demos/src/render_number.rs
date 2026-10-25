#[derive(Debug)]
pub struct RenderNumber {
    bytes: [u8; 32],
    start: usize,
}

impl RenderNumber {
    pub fn new(value: usize) -> Self {
        Self::with_selected_suffix(value, false)
    }

    pub fn selected(value: usize) -> Self {
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

    pub fn as_str(&self) -> &str {
        self.bytes
            .get(self.start..)
            .and_then(|bytes| core::str::from_utf8(bytes).ok())
            .unwrap_or_default()
    }
}
