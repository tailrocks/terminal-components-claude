//! Secrets (`COMPONENT_ARCHITECTURE.md` §15, §21 item 30 P5).
//!
//! `Secret` is not `Clone`, not `PartialEq`, not `Serialize`; `Debug` and
//! `Display` redact; `write_mask` paints a **synthetic** tail derived from
//! the fingerprint, never the real characters; `zeroize` overwrites bytes
//! before they are released.

use core::fmt;

use crate::collection::CellUi;
use crate::id::fnv1a;
use crate::theme::GlyphRole;

/// A secret string.
#[derive(Default)]
pub struct Secret(String);

impl Secret {
    /// Wrap a string.
    pub const fn new(s: String) -> Self {
        Secret(s)
    }

    /// The raw value. Deliberately verbose.
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

    /// A stable 8-byte fingerprint (FNV-1a).
    pub fn fingerprint(&self) -> [u8; 8] {
        fnv1a(0xcbf2_9ce4_8422_2325, self.0.as_bytes()).to_le_bytes()
    }

    /// Replace the value, overwriting the old bytes first.
    pub fn set(&mut self, s: &str) {
        self.zeroize();
        self.0.push_str(s);
    }

    /// Paint `n` mask glyphs followed by a synthetic tail of
    /// `policy.synthetic_tail` characters derived from the fingerprint.
    /// No `String` of the secret is constructed.
    pub fn write_mask(&self, out: &mut CellUi<'_>, n: usize, policy: SecretPolicy) {
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
    ///
    /// **Known limit of safe-Rust zeroization** (MA-13): the fill writes into
    /// a buffer that is about to be dropped, and LLVM is permitted to remove
    /// a dead store. `black_box` and a `SeqCst` fence make the write
    /// observable to the optimiser, which is as far as `#![forbid(unsafe_code)]`
    /// reaches — a guaranteed wipe needs `core::ptr::write_volatile`, which
    /// this crate cannot use. What *is* guaranteed and is asserted by
    /// `secret::zeroize_overwrites_before_drop` is that the buffer is released
    /// and a fresh `expose()` is empty.
    pub fn zeroize(&mut self) {
        let mut bytes = core::mem::take(&mut self.0).into_bytes();
        bytes.fill(0);
        core::hint::black_box(&bytes);
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        bytes.clear();
        drop(bytes);
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
        f.write_str("Secret([redacted])")
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted]")
    }
}

/// How a masked field paints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SecretPolicy {
    /// The mask glyph.
    pub mask: GlyphRole,
    /// Synthetic tail length (never the real characters).
    pub synthetic_tail: usize,
}

impl Default for SecretPolicy {
    /// The library default: the [`GlyphRole::SecretMask`] bullet glyph and a
    /// two-character synthetic tail. **Not** `GlyphRole::Dirty`: that is the
    /// uncommitted-changes marker, and a theme that restyles it must not
    /// thereby restyle password masking (D-11).
    fn default() -> Self {
        SecretPolicy {
            mask: GlyphRole::SecretMask,
            synthetic_tail: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_and_display_redact() {
        let s = Secret::new("hunter2".to_owned());
        assert_eq!(format!("{s:?}"), "Secret([redacted])");
        assert_eq!(s.to_string(), "[redacted]");
        assert_eq!(s.expose(), "hunter2");
        assert_eq!(s.len(), 7);
        assert_eq!(
            s.fingerprint(),
            Secret::new("hunter2".to_owned()).fingerprint()
        );
        assert_ne!(
            s.fingerprint(),
            Secret::new("hunter3".to_owned()).fingerprint()
        );
    }

    /// §16.1's name (MA-13). Safe Rust cannot observe the zeroed bytes after
    /// the buffer is released, so this asserts the properties that *are*
    /// observable: the capacity is gone, a fresh `expose()` is empty, and the
    /// secret is reusable afterwards. The compiler-elision risk is named on
    /// `Secret::zeroize` itself.
    #[test]
    fn zeroize_overwrites_before_drop() {
        let mut s = Secret::new("hunter2".to_owned());
        assert_eq!(s.len(), 7);
        s.zeroize();
        assert!(s.is_empty());
        assert_eq!(s.expose(), "");
        assert_eq!(s.expose().len(), 0, "the buffer is released, not kept");
        s.set("again");
        assert_eq!(s.expose(), "again");
        // the default policy masks with the dedicated role, never `Dirty`
        assert_eq!(SecretPolicy::default().synthetic_tail, 2);
        assert_eq!(SecretPolicy::default().mask, GlyphRole::SecretMask);
        assert_ne!(SecretPolicy::default().mask, GlyphRole::Dirty);
    }
}
