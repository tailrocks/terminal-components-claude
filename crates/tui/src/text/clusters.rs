//! One logical-grapheme projection over fragments (`COMPONENT_ARCHITECTURE.md`
//! §22 F10, BA-FMT).
//!
//! Styled fragments are a single logical line before Unicode segmentation: a
//! cluster spanning fragments takes its style from the fragment holding its
//! first byte (oracle `viewport.rs:19-21`). [`ClusterFeed`] segments an
//! arbitrary fragment sequence incrementally — no whole-line materialization,
//! no source cloning, no per-fragment segmentation — and the borrowed span
//! painter plus the streaming `Display` painter share it, so both agree on
//! boundaries and first-byte style ownership.

#[cfg(test)]
use core::cell::Cell;
use core::fmt;

use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};

/// Inline pending capacity in bytes. One unfinished cluster plus at most one
/// scalar lookahead stays on the stack; longer clusters spill to a reusable
/// heap overflow. The inline size bounds stack use, never cluster length:
/// there is no truncated "maximum cluster".
const INLINE: usize = 64;

/// Overwrite bytes best-effort, following the [`Secret`](crate::secret)
/// contract: fill, `black_box` against dead-store elimination, and a
/// compiler fence. No guaranteed-erasure claim over allocator copies.
fn wipe(bytes: &mut [u8]) {
    bytes.fill(0);
    core::hint::black_box(&bytes);
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

/// Pending-cluster scratch: a 64-byte inline buffer plus reusable heap
/// overflow holding only the unfinished cluster and at most one scalar
/// lookahead — never the full line or document.
///
/// [`ClusterFeed::new`] clears content while retaining capacity, so a reused
/// owner (the frame scratch behind [`Ui`](crate::ui::Ui)) stays warm and a
/// fresh owner is cold. `Debug` reports lengths only, never bytes; the
/// scratch is not `Clone`.
pub(crate) struct ClusterScratch {
    inline: [u8; INLINE],
    heap: String,
    len: usize,
    peak: usize,
    #[cfg(test)]
    validated: Cell<usize>,
}

impl ClusterScratch {
    /// Empty scratch with no heap allocation.
    pub(crate) const fn new() -> Self {
        ClusterScratch {
            inline: [0; INLINE],
            heap: String::new(),
            len: 0,
            peak: 0,
            #[cfg(test)]
            validated: Cell::new(0),
        }
    }

    /// Pending bytes held right now.
    pub(crate) const fn len(&self) -> usize {
        self.len
    }

    /// Whether no cluster is pending.
    pub(crate) const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Largest pending length since the last [`ClusterFeed::new`] reset.
    /// Bounded by the largest cluster encountered plus one scalar lookahead.
    pub(crate) const fn peak(&self) -> usize {
        self.peak
    }

    /// Retained heap capacity (lifetime high-water reuse, not live content).
    pub(crate) fn capacity(&self) -> usize {
        self.heap.capacity()
    }

    /// Bytes UTF-8-validated since the last reset (unit-test work witness).
    /// The heap path uses the already-valid `String` without revalidation,
    /// so only bounded inline prefixes ever count here.
    #[cfg(test)]
    pub(crate) const fn validated_bytes(&self) -> usize {
        self.validated.get()
    }

    /// Whether the scratch holds no live content and its inline bytes are
    /// wiped (unit-test cleanup witness). Heap spare capacity is verified
    /// by construction in [`ClusterScratch::clear_heap`].
    #[cfg(test)]
    pub(crate) fn content_is_wiped(&self) -> bool {
        self.len == 0 && self.heap.is_empty() && self.inline.iter().all(|b| *b == 0)
    }

    /// The pending text. The heap path is `O(1)`: `String` storage is valid
    /// by construction and is never revalidated per scalar.
    fn text(&self) -> &str {
        if self.len > INLINE {
            return self.heap.as_str();
        }
        self.inline
            .get(..self.len)
            .map_or("", |bytes| self.checked_text(bytes))
    }

    /// Validate a bounded inline prefix. Every push appends one complete
    /// scalar, so inline content is valid UTF-8 by construction; the empty
    /// fallback only fails closed.
    fn checked_text<'a>(&self, bytes: &'a [u8]) -> &'a str {
        #[cfg(test)]
        self.validated
            .set(self.validated.get().saturating_add(bytes.len()));
        core::str::from_utf8(bytes).unwrap_or("")
    }

    /// Append one complete scalar to the pending cluster.
    fn push(&mut self, scalar: &str) {
        let Some(end) = self.len.checked_add(scalar.len()) else {
            return;
        };
        if end <= INLINE {
            if let Some(slot) = self.inline.get_mut(self.len..end) {
                slot.copy_from_slice(scalar.as_bytes());
            }
        } else {
            if self.len <= INLINE {
                if let Some(inline) = self.inline.get(..self.len) {
                    self.heap
                        .push_str(core::str::from_utf8(inline).unwrap_or(""));
                }
                wipe(&mut self.inline);
            }
            self.heap.push_str(scalar);
        }
        self.len = end;
        self.peak = self.peak.max(end);
    }

    /// Drop an emitted `bytes` prefix, keeping the unfinished lookahead.
    /// Per-scalar feeding leaves at most one scalar behind the boundary, so
    /// the remainder returns to the inline buffer and the heap keeps only
    /// reusable capacity.
    fn discard_prefix(&mut self, bytes: usize) {
        let rest = self.len.saturating_sub(bytes);
        if self.len > INLINE {
            if rest <= INLINE {
                if let (Some(dst), Some(src)) = (
                    self.inline.get_mut(..rest),
                    self.heap.as_bytes().get(bytes..self.len),
                ) {
                    dst.copy_from_slice(src);
                }
                self.clear_heap();
            } else {
                // Unreachable under per-scalar feeding (lookahead is one
                // scalar); drain rather than strand or truncate heap bytes.
                let upto = bytes.min(self.heap.len());
                if self.heap.is_char_boundary(upto) {
                    self.heap.drain(..upto);
                } else {
                    self.clear_heap();
                    self.len = 0;
                    return;
                }
            }
        } else {
            self.inline.copy_within(bytes.min(self.len)..self.len, 0);
            if let Some(tail) = self.inline.get_mut(rest.min(INLINE)..) {
                wipe(tail);
            }
        }
        self.len = rest;
    }

    /// Wipe content while retaining heap capacity for warm reuse.
    pub(crate) fn clear(&mut self) {
        wipe(&mut self.inline);
        self.clear_heap();
        self.len = 0;
    }

    /// Best-effort wipe of the heap allocation, keeping its capacity: extend
    /// to capacity so spare bytes (which can retain an earlier longer
    /// cluster) are writable too, exactly like
    /// [`wipe_string`](crate::secret::wipe_string), then reuse the empty
    /// allocation.
    fn clear_heap(&mut self) {
        let mut bytes = core::mem::take(&mut self.heap).into_bytes();
        bytes.resize(bytes.capacity(), 0);
        wipe(bytes.as_mut_slice());
        bytes.clear();
        self.heap = String::from_utf8(bytes).unwrap_or_default();
    }
}

impl Default for ClusterScratch {
    fn default() -> Self {
        ClusterScratch::new()
    }
}

impl fmt::Debug for ClusterScratch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClusterScratch")
            .field("len", &self.len)
            .field("peak", &self.peak)
            .field("capacity", &self.heap.capacity())
            .finish_non_exhaustive()
    }
}

impl Drop for ClusterScratch {
    fn drop(&mut self) {
        self.clear();
    }
}

/// Incremental logical-grapheme segmenter over a fragment sequence.
///
/// Feed fragments in order with [`push`](ClusterFeed::push); each completed
/// cluster is emitted with the style tag of the fragment holding its first
/// byte. The trailing cluster is held until more input arrives or
/// [`finish`](ClusterFeed::finish) flushes it, so empty fragments and
/// fragment boundaries never split combining marks, ZWJ sequences,
/// regional-indicator pairs, or CRLF. A sink that refuses a cluster (clipped
/// output) stops further segmentation: later fragments are accepted without
/// scratch growth.
pub(crate) struct ClusterFeed<'s, S: Copy> {
    scratch: &'s mut ClusterScratch,
    cursor: GraphemeCursor,
    start: usize,
    pending_style: Option<S>,
    done: bool,
}

impl<S: Copy> fmt::Debug for ClusterFeed<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClusterFeed")
            .field("scratch", &self.scratch)
            .field("start", &self.start)
            .field("pending_style", &self.pending_style.is_some())
            .field("done", &self.done)
            .finish_non_exhaustive()
    }
}

impl<'s, S: Copy> ClusterFeed<'s, S> {
    /// Borrow `scratch` for one fragment sequence, clearing its content
    /// while retaining capacity, and resetting the per-use peak and work
    /// witnesses.
    pub(crate) fn new(scratch: &'s mut ClusterScratch) -> Self {
        scratch.clear();
        scratch.peak = 0;
        #[cfg(test)]
        scratch.validated.set(0);
        ClusterFeed {
            scratch,
            cursor: GraphemeCursor::new(0, usize::MAX, true),
            start: 0,
            pending_style: None,
            done: false,
        }
    }

    /// Push one fragment, emitting each newly completed cluster as
    /// `(cluster, first-byte style)`. The sink returns whether output can
    /// admit another cluster; a refusal stops segmentation without error.
    pub(crate) fn push(&mut self, fragment: &str, style: S, emit: &mut dyn FnMut(&str, S) -> bool) {
        for scalar in fragment.chars() {
            if self.done {
                return;
            }
            if self.scratch.is_empty() {
                self.pending_style = Some(style);
            }
            let mut encoded = [0u8; 4];
            // `encode_utf8` borrows `encoded`; copy the bytes out so the
            // scratch push does not alias the stack slot across calls.
            let scalar_len = scalar.encode_utf8(&mut encoded).len();
            let Some(scalar) = encoded.get(..scalar_len) else {
                continue;
            };
            let Some(scalar) = core::str::from_utf8(scalar).ok() else {
                continue;
            };
            self.scratch.push(scalar);
            wipe(&mut encoded);
            self.drain(style, emit);
        }
    }

    /// Flush the trailing cluster (its first-byte style, or `style` when no
    /// fragment was ever pushed) and clear the scratch. A refused sink still
    /// clears without emitting again.
    pub(crate) fn finish(&mut self, style: S, emit: &mut dyn FnMut(&str, S) -> bool) {
        if self.done || self.scratch.is_empty() {
            self.scratch.clear();
            self.pending_style = None;
            return;
        }
        let style = self.pending_style.unwrap_or(style);
        emit(self.scratch.text(), style);
        self.scratch.clear();
        self.pending_style = None;
    }

    /// Emit every cluster the cursor can confirm from the pending bytes.
    /// The first emission carries the pending cluster's first-fragment
    /// style; later emissions in this push started in the current fragment.
    /// Un-confirmable bytes stay pending; cursor states unreachable in a
    /// forward walk hold their bytes rather than mis-splitting them.
    fn drain(&mut self, style: S, emit: &mut dyn FnMut(&str, S) -> bool) {
        loop {
            let boundary = self.cursor.next_boundary(self.scratch.text(), self.start);
            match boundary {
                Ok(Some(end)) => {
                    let Some(bytes) = end.checked_sub(self.start) else {
                        return;
                    };
                    let Some(cluster) = self.scratch.text().get(..bytes) else {
                        return;
                    };
                    let emit_style = self.pending_style.unwrap_or(style);
                    if !emit(cluster, emit_style) {
                        self.done = true;
                        self.scratch.clear();
                        self.pending_style = None;
                        return;
                    }
                    self.scratch.discard_prefix(bytes);
                    self.start = end;
                    self.pending_style = Some(style);
                }
                Ok(None) | Err(GraphemeIncomplete::NextChunk) => return,
                Err(GraphemeIncomplete::PreContext(end)) => {
                    let Some(available) = end.checked_sub(self.start) else {
                        return;
                    };
                    let Some(context) = self.scratch.text().get(..available) else {
                        return;
                    };
                    self.cursor.provide_context(context, self.start);
                }
                Err(GraphemeIncomplete::PrevChunk | GraphemeIncomplete::InvalidOffset) => return,
            }
        }
    }
}
