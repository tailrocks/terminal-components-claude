use core::fmt;
use termrock_core::id::fnv1a;
use termrock_theme::GlyphRole;

pub trait CellWriter {
    fn glyphs(&mut self, mask: GlyphRole, n: usize);
    fn text(&mut self, s: &str);
}

pub use termrock_core::secret::wipe_string;

#[derive(Default)]
pub struct Secret(String);

impl Secret {
    pub const fn new(s: String) -> Self {
        Secret(s)
    }

    pub fn expose(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn fingerprint(&self) -> [u8; 8] {
        let h = fnv1a(0xcbf2_9ce4_8422_2325, self.0.as_bytes());
        h.to_le_bytes()
    }

    pub fn set(&mut self, s: &str) {
        self.zeroize();
        self.0.push_str(s);
    }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SecretPolicy {
    pub mask: GlyphRole,
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
