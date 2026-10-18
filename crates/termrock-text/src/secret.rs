#![doc = r" Secrets (`COMPONENT_ARCHITECTURE.md` §15, §21 item 30 P5)."]
#![doc = r""]
#![doc = r" `Secret` is not `Clone`, not `PartialEq`, not `Serialize`; `Debug` and"]
#![doc = r" `Display` redact; `write_mask` paints a **synthetic** tail derived from"]
#![doc = r" the fingerprint, never the real characters; `zeroize` overwrites bytes"]
#![doc = r" before they are released."]

use core::fmt;
use termrock_core::id::fnv1a;
use termrock_theme::GlyphRole;

#[doc = r" Abstraction for writing masked cells."]
pub trait CellWriter {
    #[doc = r" Paint `n` glyphs of the specified role."]
    fn glyphs(&mut self, mask: GlyphRole, n: usize);
    #[doc = r" Paint text."]
    fn text(&mut self, s: &str);
}

pub use termrock_core::secret::wipe_string;

#[doc = r" A secret string."]
#[derive(Default)]
pub struct Secret(String);

impl Secret {
    #[doc = r" Wrap a string."]
    pub const fn new(s: String) -> Self {
        Secret(s)
    }

    #[doc = r" The raw value for crate-internal consumers."]
    pub fn expose(&self) -> &str {
        &self.0
    }

    #[doc = r" Whether the secret is empty."]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[doc = r" The byte length."]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[doc = r" Compute an 8-byte deterministic fingerprint of the secret bytes."]
    pub fn fingerprint(&self) -> [u8; 8] {
        let h = fnv1a(0xcbf2_9ce4_8422_2325, self.0.as_bytes());
        h.to_le_bytes()
    }

    #[doc = r" Set a new value, wiping the old one."]
    pub fn set(&mut self, s: &str) {
        self.zeroize();
        self.0.push_str(s);
    }

    #[doc = r" Paint `n` mask glyphs followed by a synthetic tail of"]
    #[doc = r" `policy.synthetic_tail` characters derived from the fingerprint."]
    #[doc = r" No `String` of the secret is constructed."]
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

    #[doc = r" Overwrite every byte with zero, then release the buffer."]
    #[doc = r" Overwrite every byte with zero, then release the buffer."]
    #[doc = r""]
    #[doc = r" **Known limit of safe-Rust zeroization** (MA-13): the fill writes into"]
    #[doc = r" a buffer that is about to be dropped, and LLVM is permitted to remove"]
    #[doc = r" a dead store. `black_box` and a `SeqCst` fence make the write"]
    #[doc = r" observable to the optimiser, which is as far as `#![forbid(unsafe_code)]`"]
    #[doc = r" reaches — a guaranteed wipe needs `core::ptr::write_volatile`, which"]
    #[doc = r" this crate cannot use. What *is* guaranteed and is asserted by"]
    #[doc = r" `secret::zeroize_overwrites_before_drop` is that the buffer is released"]
    #[doc = r" and a fresh `expose()` is empty."]
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
        f.write_str("Secret([redacted])")
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret([redacted])")
    }
}

#[doc = r" Policy for rendering masked secret values."]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SecretPolicy {
    #[doc = r" Mask glyph role."]
    pub mask: GlyphRole,
    #[doc = r" Number of synthetic tail characters."]
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
