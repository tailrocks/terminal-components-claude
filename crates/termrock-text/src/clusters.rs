use core::fmt;

use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};

const INLINE: usize = 64;

fn wipe(bytes: &mut [u8]) {
    bytes.fill(0);
    core::hint::black_box(&bytes);
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

pub struct ClusterScratch {
    inline: [u8; INLINE],
    heap: String,
    len: usize,
}

impl ClusterScratch {
    pub const fn new() -> Self {
        ClusterScratch {
            inline: [0; INLINE],
            heap: String::new(),
            len: 0,
        }
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn text(&self) -> &str {
        if self.len > INLINE {
            return self.heap.as_str();
        }
        self.inline.get(..self.len).map_or("", Self::checked_text)
    }

    fn checked_text(bytes: &[u8]) -> &str {
        core::str::from_utf8(bytes).unwrap_or("")
    }

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

    pub(crate) fn clear(&mut self) {
        wipe(&mut self.inline);
        self.clear_heap();
        self.len = 0;
    }

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
