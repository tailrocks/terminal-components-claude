//! Secrets (`COMPONENT_ARCHITECTURE.md` §15, §21 item 30 P5).
//!
//! `Secret` is not `Clone`, not `PartialEq`, not `Serialize`; `Debug` and
//! `Display` redact; `write_mask` paints a **synthetic** tail derived from
//! the fingerprint, never the real characters; `zeroize` overwrites bytes
//! before they are released.

use core::fmt;
use termrock_core::id::fnv1a;
use termrock_theme::GlyphRole;

/// Abstraction for writing masked cells.
pub trait CellWriter {
    /// Paint `n` glyphs of the specified role.
    fn glyphs(&mut self, mask: GlyphRole, n: usize);
    /// Paint text.
    fn text(&mut self, s: &str);
}

/// Overwrite every byte in a `String` allocation before releasing it.
pub fn wipe_string(value: String) {
    let mut bytes = value.into_bytes();
    let capacity = bytes.capacity();
    bytes.resize(capacity, 0);
    bytes.fill(0);
    core::hint::black_box(&bytes);
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    drop(bytes);
}

/// A secret string.
#[derive(Default)]
pub struct Secret(String);

impl Secret {
    /// Wrap a string.
    pub const fn new(s: String) -> Self {
        Secret(s)
    }

    /// The raw value for crate-internal consumers.
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Whether the secret is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The byte length.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Compute an 8-byte deterministic fingerprint of the secret bytes.
    pub fn fingerprint(&self) -> [u8; 8] {
        let h = fnv1a(0xcbf2_9ce4_8422_2325, self.0.as_bytes());
        h.to_le_bytes()
    }

    /// Set a new value, wiping the old one.
    pub fn set(&mut self, s: &str) {
        self.zeroize();
        self.0.push_str(s);
    }

    /// Paint `n` mask glyphs followed by a synthetic tail of
    /// `policy.synthetic_tail` characters derived from the fingerprint.
    /// No `String` of the secret is constructed.
    pub fn write_mask<W: CellWriter>(&self, out: &mut W, n: usize, policy: SecretPolicy) {
        out.glyphs(policy.mask, n);
        let tail = policy.synthetic_tail.min(8);
        if tail == 0 || self.0.is_empty() {
            return;
        }
        let fp = self.fingerprint();
        let mut buf = [0u8; 8];
        for (slot, b) in buf.iter_mut().zip(fp.iter()) {
            let v = b % 36;
            *slot = if v < 10 {
                b'0'.saturating_add(v)
            } else {
                b'a'.saturating_add(v.saturating_sub(10))
            };
        }
        let s = core::str::from_utf8(buf.get(..tail).unwrap_or(&[])).unwrap_or("");
        out.text(s);
    }

    /// Overwrite every byte with zero, then release the buffer.
    pub fn zeroize(&mut self) {
        wipe_string(core::mem::take(&mut self.0));
        self.0 = String::new();
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED {} bytes]", self.0.len())
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

/// Policy for rendering masked secret values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SecretPolicy {
    /// Mask glyph role.
    pub mask: GlyphRole,
    /// Number of synthetic tail characters.
    pub synthetic_tail: usize,
}

impl Default for SecretPolicy {
    fn default() -> Self {
        SecretPolicy {
            mask: GlyphRole::SecretMask,
            synthetic_tail: 2,
        }
    }
}
