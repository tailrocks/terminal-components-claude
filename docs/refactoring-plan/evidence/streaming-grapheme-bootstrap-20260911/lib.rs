#![forbid(unsafe_code)]
#![cfg(test)]
//! Disposable storage/consumer feasibility only; not a TC replacement implementation.
use std::fmt::{self, Write};
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};
// Same safe best-effort clearing contract as pinned TC Secret; no erasure guarantee.
trait Wipe {
    fn wipe(&mut self);
}
impl Wipe for [u8] {
    fn wipe(&mut self) {
        self.fill(0);
        std::hint::black_box(&self);
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
    }
}

const INLINE: usize = 64;
struct Scratch {
    inline: [u8; INLINE],
    heap: String,
    len: usize,
    peak: usize,
    validated_bytes: std::cell::Cell<usize>,
}
impl Default for Scratch {
    fn default() -> Self {
        Self {
            inline: [0; INLINE],
            heap: String::new(),
            len: 0,
            peak: 0,
            validated_bytes: std::cell::Cell::new(0),
        }
    }
}
impl Scratch {
    fn text(&self) -> &str {
        if self.len > INLINE {
            return self.heap.as_str();
        }
        self.checked_text(&self.inline[..self.len])
    }
    fn checked_text<'a>(&self, bytes: &'a [u8]) -> &'a str {
        self.validated_bytes
            .set(self.validated_bytes.get() + bytes.len());
        std::str::from_utf8(bytes).expect("scratch receives complete UTF-8 scalars")
    }
    fn push(&mut self, scalar: &str) {
        let end = self
            .len
            .checked_add(scalar.len())
            .expect("fixture input length fits usize");
        if end <= INLINE {
            self.inline[self.len..end].copy_from_slice(scalar.as_bytes());
        } else {
            if self.len <= INLINE {
                self.heap
                    .push_str(std::str::from_utf8(&self.inline[..self.len]).expect("inline UTF-8"));
                self.inline.wipe();
            }
            self.heap.push_str(scalar);
        }
        self.len = end;
        self.peak = self.peak.max(end);
    }
    fn discard_prefix(&mut self, bytes: usize) {
        let rest = self.len - bytes;
        if self.len > INLINE {
            assert!(
                rest <= 4,
                "one scalar lookahead beyond the completed cluster"
            );
            self.inline[..rest].copy_from_slice(&self.heap.as_bytes()[bytes..self.len]);
            self.clear_heap();
        } else {
            self.inline.copy_within(bytes..self.len, 0);
            self.inline[rest..].wipe();
        }
        self.len = rest;
    }
    fn clear(&mut self) {
        self.inline.wipe();
        self.clear_heap();
        self.len = 0;
    }
    fn clear_heap(&mut self) {
        let mut bytes = std::mem::take(&mut self.heap).into_bytes();
        bytes.as_mut_slice().wipe();
        bytes.clear();
        self.heap = String::from_utf8(bytes).expect("empty bytes are UTF-8; capacity is reused");
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        self.clear();
    }
}

struct Stream<'a, F: FnMut(&str) -> bool> {
    scratch: &'a mut Scratch,
    cursor: GraphemeCursor,
    start: usize,
    sink: F,
    visits: usize,
    done: bool,
}
impl<'a, F: FnMut(&str) -> bool> Stream<'a, F> {
    fn new(scratch: &'a mut Scratch, sink: F) -> Self {
        scratch.clear();
        scratch.peak = 0;
        scratch.validated_bytes.set(0);
        Self {
            scratch,
            cursor: GraphemeCursor::new(0, usize::MAX, true),
            start: 0,
            sink,
            visits: 0,
            done: false,
        }
    }
    fn finish(&mut self) {
        if self.scratch.len != 0 {
            (self.sink)(self.scratch.text());
        }
        self.scratch.clear();
    }
}
impl<F: FnMut(&str) -> bool> Drop for Stream<'_, F> {
    fn drop(&mut self) {
        self.scratch.clear();
    }
}
impl<F: FnMut(&str) -> bool> Write for Stream<'_, F> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for scalar in text.chars() {
            if self.done {
                break;
            }
            let mut encoded = [0; 4];
            self.scratch.push(scalar.encode_utf8(&mut encoded));
            encoded.wipe();
            loop {
                self.visits += 1;
                match self.cursor.next_boundary(self.scratch.text(), self.start) {
                    Ok(Some(end)) => {
                        let bytes = end - self.start;
                        if !(self.sink)(&self.scratch.text()[..bytes]) {
                            self.done = true;
                            self.scratch.clear();
                            break;
                        }
                        self.scratch.discard_prefix(bytes);
                        self.start = end;
                    }
                    Err(GraphemeIncomplete::NextChunk) => break,
                    Err(GraphemeIncomplete::PreContext(end)) => {
                        let available = end
                            .checked_sub(self.start)
                            .expect("cursor retains prior-boundary context");
                        self.cursor
                            .provide_context(&self.scratch.text()[..available], self.start);
                    }
                    unexpected => panic!("unexpected forward cursor result: {unexpected:?}"),
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use junie_tui::{ColorLevel, Id, List, ListState, Theme};
    use junie_tui_testing::{
        Scene,
        perf::{self, Counting},
    };
    use ratatui_core::buffer::CellWidth;
    use std::cell::{Cell, RefCell};
    use unicode_segmentation::UnicodeSegmentation;
    #[global_allocator]
    static ALLOCATOR: Counting = Counting;

    struct Fragmented<'a> {
        text: &'a str,
        split: usize,
        calls: &'a Cell<usize>,
        writes: &'a Cell<usize>,
    }
    impl fmt::Display for Fragmented<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.calls.set(self.calls.get() + 1);
            f.write_str(&self.text[..self.split])?;
            self.writes.set(self.writes.get() + 1);
            f.write_str("")?;
            self.writes.set(self.writes.get() + 1);
            f.write_str(&self.text[self.split..])?;
            self.writes.set(self.writes.get() + 1);
            Ok(())
        }
    }
    #[test]
    fn exact_clusters_all_fragment_boundaries_and_bounded_work() {
        let long = format!("a{}z", "\u{301}".repeat(4096));
        let corpus = [
            "",
            "plain",
            "界半",
            "e\u{301}x",
            "👩‍👩‍👧‍👦!",
            "🇺🇸🇫🇷🇯",
            "\r\nX",
            "क्\u{200d}ष",
            "a\u{301}\u{301}",
            &long,
        ];
        for text in corpus {
            for split in text
                .char_indices()
                .map(|(i, _)| i)
                .chain(std::iter::once(text.len()))
            {
                let mut scratch = Scratch::default();
                let mut actual = Vec::new();
                let mut stream = Stream::new(&mut scratch, |s: &str| {
                    actual.push(s.to_owned());
                    true
                });
                stream.write_str(&text[..split]).unwrap();
                stream.write_str("").unwrap();
                stream.write_str(&text[split..]).unwrap();
                stream.finish();
                assert!(stream.visits <= 3 * text.chars().count() + 1);
                drop(stream);
                assert_eq!(
                    actual,
                    text.graphemes(true).collect::<Vec<_>>(),
                    "split {split}"
                );
                assert_eq!(scratch.len, 0);
                assert!(scratch.inline.iter().all(|b| *b == 0));
                let largest = text.graphemes(true).map(str::len).max().unwrap_or(0);
                assert!(scratch.peak <= largest + 4);
                assert!(
                    scratch.validated_bytes.get() <= 4 * INLINE * (text.chars().count() + 1),
                    "revalidated growing heap prefix"
                );
            }
        }
    }

    fn measure(text: &str, split: usize, scratch: &mut Scratch) -> (usize, usize, usize, usize) {
        let before = perf::allocs();
        let bytes_before = perf::bytes();
        let mut clusters = 0;
        let mut stream = Stream::new(scratch, |_| {
            clusters += 1;
            true
        });
        stream.write_str(&text[..split]).unwrap();
        stream.write_str(&text[split..]).unwrap();
        stream.finish();
        drop(stream);
        (
            perf::allocs() - before,
            perf::bytes() - bytes_before,
            clusters,
            scratch.peak,
        )
    }
    #[test]
    fn honest_cold_warm_and_fresh_owner_allocation_counts() {
        let long = format!("a{}z", "\u{301}".repeat(4096));
        for (name, text) in [
            ("ascii", "hello"),
            ("cjk", "中文"),
            ("combining", "e\u{301}"),
            ("zwj", "👩‍👩‍👧‍👦"),
            ("long", long.as_str()),
        ] {
            let split = text.char_indices().nth(1).map_or(text.len(), |(i, _)| i);
            let mut scratch = Scratch::default();
            let cold = measure(text, split, &mut scratch);
            let capacity = scratch.heap.capacity();
            let warm = measure(text, split, &mut scratch);
            let fresh = measure(text, split, &mut Scratch::default());
            eprintln!(
                "{name}: cold={cold:?} warm={warm:?} fresh={fresh:?} retained_capacity={capacity}"
            );
            assert_eq!(warm.0, 0);
            assert_eq!(cold.0, fresh.0);
            if name != "long" {
                assert_eq!(cold.0, 0);
            } else {
                assert!(cold.0 > 0);
                assert!(cold.0 <= 9, "4096-mark cold scratch ceiling");
                assert!(capacity <= 2 * cold.3);
            }
        }
    }
    #[test]
    fn public_list_consumer_cells_one_original_display_call_and_allocation_attribution() {
        let long = format!("a{}z", "\u{301}".repeat(256));
        for text in ["e\u{301}", "👩‍👩‍👧‍👦!", "🇺🇸🇫🇷", "界x", long.as_str()]
        {
            for width in [1, 2, 3, 8, 20] {
                let calls = Cell::new(0);
                let writes = Cell::new(0);
                let callbacks = Cell::new(0);
                let item = Fragmented {
                    text,
                    split: text.char_indices().nth(1).map_or(text.len(), |(i, _)| i),
                    calls: &calls,
                    writes: &writes,
                };
                let scratch = RefCell::new(Scratch::default());
                let paint_allocs = Cell::new(0);
                let format_allocs = Cell::new(0);
                let mut expected =
                    Scene::new("unsplit", Theme::junie(), ColorLevel::TrueColor, width, 1);
                expected.draw(|ui, area| {
                    List::new(Id::root("row")).draw(ui, area, &ListState::default(), &[text]);
                });
                let mut actual =
                    Scene::new("stream", Theme::junie(), ColorLevel::TrueColor, width, 1);
                for pass in 0..2 {
                    actual.draw(|ui, area| {
                        List::new(Id::root("row"))
                            .row(|item: &&Fragmented<'_>, row: &mut junie_tui::RowUi<'_>| {
                                callbacks.set(callbacks.get() + 1);
                                let mut remaining = row.area().width;
                                let mut clipped = false;
                                let mut scratch = scratch.borrow_mut();
                                let before = perf::allocs();
                                let mut painted = 0;
                                let mut stream = Stream::new(&mut scratch, |cluster: &str| {
                                    let size = cluster.cell_width();
                                    if clipped || size > remaining {
                                        clipped = true;
                                        return false;
                                    }
                                    let paint_before = perf::allocs();
                                    row.label_fmt(format_args!("{cluster}"));
                                    painted += perf::allocs() - paint_before;
                                    remaining = remaining.saturating_sub(size);
                                    remaining != 0
                                });
                                write!(&mut stream, "{item}").unwrap();
                                stream.finish();
                                drop(stream);
                                paint_allocs.set(painted);
                                format_allocs.set(perf::allocs() - before - painted);
                            })
                            .draw(ui, area, &ListState::default(), &[&item]);
                    });
                    assert_eq!(
                        actual.buffer(),
                        expected.buffer(),
                        "width={width} pass={pass}"
                    );
                    assert_eq!(calls.get(), callbacks.get());
                    assert_eq!(writes.get(), 3 * callbacks.get());
                    if pass == 1 {
                        assert_eq!(format_allocs.get(), 0);
                    } else if text.len() == 514 {
                        assert!(format_allocs.get() <= 5, "256-mark cold scratch ceiling");
                    }
                    eprintln!(
                        "consumer bytes={} width={width} pass={pass} scratch_allocs={} output_and_style_allocs={}",
                        text.len(),
                        format_allocs.get(),
                        paint_allocs.get()
                    );
                }
            }
        }
    }

    #[test]
    fn formatting_error_and_unwind_clear_incomplete_storage() {
        struct Fails<'a>(&'a str);
        impl fmt::Display for Fails<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.0)?;
                Err(fmt::Error)
            }
        }
        let mut scratch = Scratch::default();
        let prefix = format!("a{}", "\u{301}".repeat(256));
        let mut painted = String::new();
        {
            let mut stream = Stream::new(&mut scratch, |s: &str| {
                painted.push_str(s);
                true
            });
            assert!(write!(&mut stream, "{}", Fails(&prefix)).is_err());
            stream.finish();
        }
        assert_eq!(
            painted, prefix,
            "a returned fmt::Error still preserves its written prefix"
        );
        {
            let mut stream = Stream::new(&mut scratch, |_| true);
            stream.write_str(&"a\u{301}".repeat(30)).unwrap();
        }
        assert_eq!(scratch.len, 0);
        assert!(scratch.inline.iter().all(|b| *b == 0));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut stream = Stream::new(&mut scratch, |_| true);
            stream
                .write_str(&format!("a{}", "\u{301}".repeat(256)))
                .unwrap();
            panic!("intentional formatter failure");
        }));
        assert!(result.is_err());
        assert_eq!(scratch.len, 0);
        assert!(scratch.heap.is_empty());
    }

    #[test]
    fn clipping_stops_projection_not_original_formatter_effects() {
        let text = format!("xa{}", "\u{301}".repeat(4096));
        let calls = Cell::new(0);
        let writes = Cell::new(0);
        let item = Fragmented {
            text: &text,
            split: 1,
            calls: &calls,
            writes: &writes,
        };
        let mut scratch = Scratch::default();
        let before = perf::allocs();
        let mut emitted = 0;
        {
            let mut stream = Stream::new(&mut scratch, |s| {
                assert_eq!(s, "x");
                emitted += 1;
                false
            });
            write!(&mut stream, "{item}").unwrap();
            stream.finish();
            assert!(stream.visits <= 3);
        }
        assert_eq!(perf::allocs() - before, 0);
        assert_eq!((calls.get(), writes.get(), emitted), (1, 3, 1));
        assert_eq!(scratch.peak, 2);
    }

    #[test]
    fn long_short_long_keeps_only_high_water_capacity_and_fresh_owner_is_cold() {
        let long = format!("a{}z", "\u{301}".repeat(4096));
        let mut scratch = Scratch::default();
        let cold = measure(&long, 1, &mut scratch);
        let high_water_capacity = scratch.heap.capacity();
        let short = measure("abc", 1, &mut scratch);
        assert_eq!(short.0, 0);
        assert_eq!(scratch.heap.capacity(), high_water_capacity);
        assert!(scratch.heap.is_empty());
        assert!(scratch.inline.iter().all(|b| *b == 0));
        assert_eq!(scratch.len, 0);
        let warm = measure(&long, 1, &mut scratch);
        let fresh = measure(&long, 1, &mut Scratch::default());
        assert_eq!((warm.0, fresh.0), (0, cold.0));
        assert!((1..=9).contains(&cold.0));
        assert!(high_water_capacity < 2 * (8193 + 4));
        assert!(
            high_water_capacity > 2 * ("abc".len() + 4),
            "retention is not bounded by the last short draw"
        );
    }

    #[test]
    fn public_consumer_allocation_resize_long_short_long_matches_each_first_frame() {
        let long = format!("a{}z", "\u{301}".repeat(256));
        let scratch = RefCell::new(Scratch::default());
        let mut actual = Scene::new(
            "resize-stream",
            Theme::junie(),
            ColorLevel::TrueColor,
            20,
            1,
        );
        let mut expected = Scene::new("resize-whole", Theme::junie(), ColorLevel::TrueColor, 20, 1);
        let mut retained = 0;
        for (pass, (text, width)) in [(long.as_str(), 20), ("abc", 8), (long.as_str(), 20)]
            .into_iter()
            .enumerate()
        {
            expected.draw(|ui, mut area| {
                area.width = width;
                List::new(Id::root("row")).draw(ui, area, &ListState::default(), &[text]);
            });
            let scratch_events = Cell::new(0);
            actual.draw(|ui, mut area| {
                area.width = width;
                List::new(Id::root("row"))
                    .row(|text: &&str, row: &mut junie_tui::RowUi<'_>| {
                        let mut remaining = row.area().width;
                        let mut scratch = scratch.borrow_mut();
                        let before = perf::allocs();
                        let mut paint_events = 0;
                        let mut stream = Stream::new(&mut scratch, |cluster: &str| {
                            let width = cluster.cell_width();
                            if width > remaining {
                                return false;
                            }
                            let before = perf::allocs();
                            row.label_fmt(format_args!("{cluster}"));
                            paint_events += perf::allocs() - before;
                            remaining -= width;
                            remaining != 0
                        });
                        stream.write_str(&text[..1]).unwrap();
                        stream.write_str(&text[1..]).unwrap();
                        stream.finish();
                        drop(stream);
                        scratch_events.set(perf::allocs() - before - paint_events);
                    })
                    .draw(ui, area, &ListState::default(), &[text]);
            });
            assert_eq!(
                actual.buffer(),
                expected.buffer(),
                "first frame after allocation resize {pass}"
            );
            if pass == 0 {
                assert!((1..=5).contains(&scratch_events.get()));
                retained = scratch.borrow().heap.capacity();
            } else {
                assert_eq!(scratch_events.get(), 0);
                assert_eq!(scratch.borrow().heap.capacity(), retained);
            }
            assert!(scratch.borrow().heap.is_empty());
            assert_eq!(scratch.borrow().len, 0);
        }
    }
}
