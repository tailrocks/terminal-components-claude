#![doc = r" One logical-grapheme projection over fragments (`COMPONENT_ARCHITECTURE.md`"]
#![doc = r" §22 F10, BA-FMT)."]
#![doc = r""]
#![doc = r" Styled fragments are a single logical line before Unicode segmentation: a"]
#![doc = r" cluster spanning fragments takes its style from the fragment holding its"]
#![doc = r" first byte (oracle `viewport.rs:19-21`). [`ClusterFeed`] segments an"]
#![doc = r" arbitrary fragment sequence incrementally — no whole-line materialization,"]
#![doc = r" no source cloning, and no per-fragment segmentation. The borrowed span"]
#![doc = r" painter preserves boundaries and first-byte style ownership across spans."]

use core::fmt;

use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};

#[doc = r" Inline pending capacity in bytes. One unfinished cluster plus at most one"]
#[doc = r" scalar lookahead stays on the stack; longer clusters spill to a reusable"]
#[doc = r" heap overflow. The inline size bounds stack use, never cluster length:"]
#[doc = r#" there is no truncated "maximum cluster"."#]
const INLINE: usize = 64;

#[doc = r" Overwrite bytes best-effort, following the [`Secret`](crate::secret)"]
#[doc = r" contract: fill, `black_box` against dead-store elimination, and a"]
#[doc = r" compiler fence. No guaranteed-erasure claim over allocator copies."]
fn wipe(bytes: &mut [u8]) {
    bytes.fill(0);
    core::hint::black_box(&bytes);
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

#[doc = r" Pending-cluster scratch: a 64-byte inline buffer plus reusable heap"]
#[doc = r" overflow holding only the unfinished cluster and at most one scalar"]
#[doc = r" lookahead — never the full line or document."]
#[doc = r""]
#[doc = r" [`ClusterFeed::new`] clears content while retaining capacity, so a reused"]
#[doc = r" owner (the frame scratch behind `Ui` (in `termrock-runtime`)) stays warm and a"]
#[doc = r" fresh owner is cold. `Debug` reports lengths only, never bytes; the"]
#[doc = r" scratch is not `Clone`."]
pub struct ClusterScratch {
    inline: [u8; INLINE],
    heap: String,
    len: usize,
}

impl ClusterScratch {
    #[doc = r" Empty scratch with no heap allocation."]
    pub const fn new() -> Self {
        ClusterScratch {
            inline: [0; INLINE],
            heap: String::new(),
            len: 0,
        }
    }

    #[doc = r" Whether no cluster is pending."]
    pub(crate) const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[doc = r" The pending text. The heap path is `O(1)`: `String` storage is valid"]
    #[doc = r" by construction and is never revalidated per scalar."]
    fn text(&self) -> &str {
        if self.len > INLINE {
            return self.heap.as_str();
        }
        self.inline.get(..self.len).map_or("", Self::checked_text)
    }

    #[doc = r" Validate a bounded inline prefix. Every push appends one complete"]
    #[doc = r" scalar, so inline content is valid UTF-8 by construction; the empty"]
    #[doc = r" fallback only fails closed."]
    fn checked_text(bytes: &[u8]) -> &str {
        core::str::from_utf8(bytes).unwrap_or("")
    }

    #[doc = r" Append one complete scalar to the pending cluster."]
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
    }

    #[doc = r" Drop an emitted `bytes` prefix, keeping the unfinished lookahead."]
    #[doc = r" Per-scalar feeding leaves at most one scalar behind the boundary, so"]
    #[doc = r" the remainder returns to the inline buffer and the heap keeps only"]
    #[doc = r" reusable capacity."]
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

            if let Some(tail) = self.inline.get_mut(rest.min(INLINE)..self.len) {
                wipe(tail);
            }
        }
        self.len = rest;
    }

    #[doc = r" Wipe content while retaining heap capacity for warm reuse."]
    pub(crate) fn clear(&mut self) {
        wipe(&mut self.inline);
        self.clear_heap();
        self.len = 0;
    }

    #[doc = r" Best-effort wipe of the heap allocation, keeping its capacity: extend"]
    #[doc = r" to capacity so spare bytes (which can retain an earlier longer"]
    #[doc = r" cluster) are writable too, exactly like"]
    #[doc = r" [`wipe_string`](crate::secret::wipe_string), then reuse the empty"]
    #[doc = r" allocation."]
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
            .field("capacity", &self.heap.capacity())
            .finish_non_exhaustive()
    }
}

impl Drop for ClusterScratch {
    fn drop(&mut self) {
        self.clear();
    }
}

#[doc = r" Incremental logical-grapheme segmenter over a fragment sequence."]
#[doc = r""]
#[doc = r" Feed fragments in order with [`push`](ClusterFeed::push); each completed"]
#[doc = r" cluster is emitted with the style tag of the fragment holding its first"]
#[doc = r" byte. The trailing cluster is held until more input arrives or"]
#[doc = r" [`finish`](ClusterFeed::finish) flushes it, so empty fragments and"]
#[doc = r" fragment boundaries never split combining marks, ZWJ sequences,"]
#[doc = r" regional-indicator pairs, or CRLF. A sink that refuses a cluster (clipped"]
#[doc = r" output) stops further segmentation: later fragments are accepted without"]
#[doc = r" scratch growth."]
pub struct ClusterFeed<'s, S: Copy> {
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
    #[doc = r" Borrow `scratch` for one fragment sequence, clearing its content"]
    #[doc = r" while retaining capacity."]
    pub fn new(scratch: &'s mut ClusterScratch) -> Self {
        scratch.clear();
        ClusterFeed {
            scratch,
            cursor: GraphemeCursor::new(0, usize::MAX, true),
            start: 0,
            pending_style: None,
            done: false,
        }
    }

    #[doc = r" Push one fragment, emitting each newly completed cluster as"]
    #[doc = r" `(cluster, first-byte style)`. The sink returns whether output can"]
    #[doc = r" admit another cluster; a refusal stops segmentation without error."]
    pub fn push(&mut self, fragment: &str, style: S, emit: &mut dyn FnMut(&str, S) -> bool) {
        for (cursor, scalar) in fragment.char_indices() {
            if self.done {
                return;
            }
            if self.scratch.is_empty() {
                self.pending_style = Some(style);
            }
            let next = cursor.saturating_add(scalar.len_utf8());
            let Some(piece) = fragment.get(cursor..next) else {
                break;
            };
            self.scratch.push(piece);
            self.drain(style, emit);
        }
    }

    #[doc = r" Flush the trailing cluster (its first-byte style, or `style` when no"]
    #[doc = r" fragment was ever pushed) and clear the scratch. A refused sink still"]
    #[doc = r" clears without emitting again."]
    pub fn finish(&mut self, style: S, emit: &mut dyn FnMut(&str, S) -> bool) {
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

    #[doc = r" Emit every cluster the cursor can confirm from the pending bytes."]
    #[doc = r" The first emission carries the pending cluster's first-fragment"]
    #[doc = r" style; later emissions in this push started in the current fragment."]
    #[doc = r" Un-confirmable bytes stay pending; cursor states unreachable in a"]
    #[doc = r" forward walk hold their bytes rather than mis-splitting them."]
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
                Ok(None)
                | Err(
                    GraphemeIncomplete::NextChunk
                    | GraphemeIncomplete::PrevChunk
                    | GraphemeIncomplete::InvalidOffset,
                ) => return,
                Err(GraphemeIncomplete::PreContext(end)) => {
                    let Some(available) = end.checked_sub(self.start) else {
                        return;
                    };
                    let Some(context) = self.scratch.text().get(..available) else {
                        return;
                    };
                    self.cursor.provide_context(context, self.start);
                }
            }
        }
    }
}
