use std::ops::Range;

use termrock_text::TextBuffer;

fn assert_state(
    buffer: &TextBuffer,
    expected_text: &str,
    expected_cursor: usize,
    expected_selection: Option<Range<usize>>,
    context: &str,
) {
    assert_eq!(buffer.text(), expected_text, "{context}: text");
    assert_eq!(buffer.cursor_offset(), expected_cursor, "{context}: cursor");
    assert_eq!(
        buffer.selection(),
        expected_selection,
        "{context}: selection"
    );
}

#[test]
fn insertion_rebases_forward_and_reverse_selection_with_right_sticky_ties() {
    struct Case {
        label: &'static str,
        anchor: usize,
        cursor: usize,
        at: usize,
        text: &'static str,
        expected_text: &'static str,
        expected_cursor: usize,
        expected_selection: Option<Range<usize>>,
    }

    let cases = [
        Case {
            label: "forward before",
            anchor: 2,
            cursor: 4,
            at: 0,
            text: "X",
            expected_text: "Xabcdef",
            expected_cursor: 5,
            expected_selection: Some(3..5),
        },
        Case {
            label: "reverse before",
            anchor: 4,
            cursor: 2,
            at: 0,
            text: "X",
            expected_text: "Xabcdef",
            expected_cursor: 3,
            expected_selection: Some(3..5),
        },
        Case {
            label: "forward after",
            anchor: 2,
            cursor: 4,
            at: 5,
            text: "X",
            expected_text: "abcdeXf",
            expected_cursor: 4,
            expected_selection: Some(2..4),
        },
        Case {
            label: "reverse after",
            anchor: 4,
            cursor: 2,
            at: 5,
            text: "X",
            expected_text: "abcdeXf",
            expected_cursor: 2,
            expected_selection: Some(2..4),
        },
        Case {
            label: "forward inside",
            anchor: 2,
            cursor: 4,
            at: 3,
            text: "X",
            expected_text: "abcXdef",
            expected_cursor: 5,
            expected_selection: Some(2..5),
        },
        Case {
            label: "reverse inside",
            anchor: 4,
            cursor: 2,
            at: 3,
            text: "X",
            expected_text: "abcXdef",
            expected_cursor: 2,
            expected_selection: Some(2..5),
        },
        Case {
            label: "forward at start",
            anchor: 2,
            cursor: 4,
            at: 2,
            text: "X",
            expected_text: "abXcdef",
            expected_cursor: 5,
            expected_selection: Some(3..5),
        },
        Case {
            label: "reverse at start",
            anchor: 4,
            cursor: 2,
            at: 2,
            text: "X",
            expected_text: "abXcdef",
            expected_cursor: 3,
            expected_selection: Some(3..5),
        },
        Case {
            label: "forward at end",
            anchor: 2,
            cursor: 4,
            at: 4,
            text: "X",
            expected_text: "abcdXef",
            expected_cursor: 5,
            expected_selection: Some(2..5),
        },
        Case {
            label: "reverse at end",
            anchor: 4,
            cursor: 2,
            at: 4,
            text: "X",
            expected_text: "abcdXef",
            expected_cursor: 2,
            expected_selection: Some(2..5),
        },
    ];

    for case in cases {
        let mut buffer = TextBuffer::single("abcdef");
        buffer.select_range(case.anchor, case.cursor);
        buffer.insert_at(case.at, case.text);
        assert_state(
            &buffer,
            case.expected_text,
            case.expected_cursor,
            case.expected_selection,
            case.label,
        );
    }
}

#[test]
fn insertion_preserves_collapsed_and_absent_anchors_and_clamps_offsets() {
    let mut collapsed = TextBuffer::single("abcd");
    collapsed.select_range(2, 2);
    collapsed.insert_at(1, "X");
    assert_state(&collapsed, "aXbcd", 3, None, "collapsed selection shifted");

    let mut no_anchor = TextBuffer::single("abcd");
    no_anchor.set_cursor_line_col(0, 2);
    no_anchor.insert_at(2, "X");
    assert_state(&no_anchor, "abXcd", 3, None, "cursor tie is right-sticky");

    let mut at_end = TextBuffer::single("ab");
    at_end.insert_at(usize::MAX, "X");
    assert_state(&at_end, "abX", 3, None, "MAX clamps to end");

    let mut selected_at_end = TextBuffer::single("ab");
    selected_at_end.select_range(2, 1);
    selected_at_end.insert_at(usize::MAX, "X");
    assert_state(
        &selected_at_end,
        "abX",
        1,
        Some(1..3),
        "selection endpoint at clamped end shifts",
    );
}

#[test]
fn empty_and_single_line_normalized_insertions_leave_state_unchanged() {
    let mut buffer = TextBuffer::single("abc");
    buffer.select_range(1, 2);
    buffer.insert_at(1, "");
    assert_state(&buffer, "abc", 2, Some(1..2), "empty insertion");
    buffer.insert_at(1, "\r\n");
    assert_state(
        &buffer,
        "abc",
        2,
        Some(1..2),
        "newline-only insertion filtered",
    );

    let mut normalized = TextBuffer::single("abcd");
    normalized.select_range(1, 3);
    normalized.insert_at(1, "X\r\nY");
    assert_state(
        &normalized,
        "aXYbcd",
        5,
        Some(3..5),
        "single-line insertion rebases by filtered bytes without replacing selection",
    );
}

#[test]
fn insertion_floors_to_graphemes_and_rebases_by_normalized_bytes() {
    struct Case {
        label: &'static str,
        original: &'static str,
        at: usize,
        expected: &'static str,
        expected_cursor: usize,
    }
    let cases = [
        Case {
            label: "combining grapheme",
            original: "e\u{301}x",
            at: 2,
            expected: "Xe\u{301}x",
            expected_cursor: 5,
        },
        Case {
            label: "wide CJK scalar",
            original: "日本x",
            at: 2,
            expected: "X日本x",
            expected_cursor: 8,
        },
        Case {
            label: "ZWJ grapheme",
            original: "👩\u{200d}💻x",
            at: 5,
            expected: "X👩\u{200d}💻x",
            expected_cursor: 13,
        },
        Case {
            label: "regional indicator pair",
            original: "🇺🇸x",
            at: 5,
            expected: "X🇺🇸x",
            expected_cursor: 10,
        },
    ];
    for case in cases {
        let mut buffer = TextBuffer::single(case.original);
        buffer.insert_at(case.at, "X");
        assert_state(
            &buffer,
            case.expected,
            case.expected_cursor,
            None,
            case.label,
        );
    }

    let mut joined = TextBuffer::single("ex");
    joined.select_range(1, 2);
    joined.insert_at(1, "\u{301}");
    assert_state(
        &joined,
        "e\u{301}x",
        4,
        Some(3..4),
        "insertion joins the left grapheme while preserving right-sticky endpoints",
    );

    let mut multiline = TextBuffer::multi("ab\ncd");
    multiline.select_range(1, 4);
    multiline.insert_at(0, "\r\nZ\r");
    assert_state(
        &multiline,
        "\nZ\nab\ncd",
        7,
        Some(4..7),
        "multiline insertion rebases by normalized LF bytes",
    );
}

#[test]
fn removal_rebases_selection_endpoints_for_relative_ranges() {
    struct Case {
        label: &'static str,
        anchor: usize,
        cursor: usize,
        start: usize,
        end: usize,
        expected_text: &'static str,
        expected_cursor: usize,
        expected_selection: Option<Range<usize>>,
    }

    let cases = [
        Case {
            label: "forward before",
            anchor: 2,
            cursor: 4,
            start: 0,
            end: 1,
            expected_text: "bcdef",
            expected_cursor: 3,
            expected_selection: Some(1..3),
        },
        Case {
            label: "reverse before",
            anchor: 4,
            cursor: 2,
            start: 0,
            end: 1,
            expected_text: "bcdef",
            expected_cursor: 1,
            expected_selection: Some(1..3),
        },
        Case {
            label: "forward after",
            anchor: 2,
            cursor: 4,
            start: 5,
            end: 6,
            expected_text: "abcde",
            expected_cursor: 4,
            expected_selection: Some(2..4),
        },
        Case {
            label: "reverse after",
            anchor: 4,
            cursor: 2,
            start: 5,
            end: 6,
            expected_text: "abcde",
            expected_cursor: 2,
            expected_selection: Some(2..4),
        },
        Case {
            label: "forward inside",
            anchor: 2,
            cursor: 4,
            start: 3,
            end: 4,
            expected_text: "abcef",
            expected_cursor: 3,
            expected_selection: Some(2..3),
        },
        Case {
            label: "reverse inside",
            anchor: 4,
            cursor: 2,
            start: 3,
            end: 4,
            expected_text: "abcef",
            expected_cursor: 2,
            expected_selection: Some(2..3),
        },
        Case {
            label: "forward crosses start",
            anchor: 2,
            cursor: 4,
            start: 1,
            end: 3,
            expected_text: "adef",
            expected_cursor: 2,
            expected_selection: Some(1..2),
        },
        Case {
            label: "reverse crosses start",
            anchor: 4,
            cursor: 2,
            start: 1,
            end: 3,
            expected_text: "adef",
            expected_cursor: 1,
            expected_selection: Some(1..2),
        },
        Case {
            label: "forward crosses end",
            anchor: 2,
            cursor: 4,
            start: 3,
            end: 5,
            expected_text: "abcf",
            expected_cursor: 3,
            expected_selection: Some(2..3),
        },
        Case {
            label: "reverse crosses end",
            anchor: 4,
            cursor: 2,
            start: 3,
            end: 5,
            expected_text: "abcf",
            expected_cursor: 2,
            expected_selection: Some(2..3),
        },
        Case {
            label: "forward encloses selection",
            anchor: 2,
            cursor: 4,
            start: 1,
            end: 5,
            expected_text: "af",
            expected_cursor: 1,
            expected_selection: None,
        },
        Case {
            label: "reverse encloses selection",
            anchor: 4,
            cursor: 2,
            start: 1,
            end: 5,
            expected_text: "af",
            expected_cursor: 1,
            expected_selection: None,
        },
        Case {
            label: "forward anchor at range end",
            anchor: 2,
            cursor: 4,
            start: 0,
            end: 2,
            expected_text: "cdef",
            expected_cursor: 2,
            expected_selection: Some(0..2),
        },
        Case {
            label: "reverse anchor at range end",
            anchor: 4,
            cursor: 2,
            start: 0,
            end: 2,
            expected_text: "cdef",
            expected_cursor: 0,
            expected_selection: Some(0..2),
        },
        Case {
            label: "forward cursor at range start",
            anchor: 2,
            cursor: 4,
            start: 4,
            end: 5,
            expected_text: "abcdf",
            expected_cursor: 4,
            expected_selection: Some(2..4),
        },
        Case {
            label: "reverse anchor at range start",
            anchor: 4,
            cursor: 2,
            start: 4,
            end: 5,
            expected_text: "abcdf",
            expected_cursor: 2,
            expected_selection: Some(2..4),
        },
    ];

    for case in cases {
        let mut buffer = TextBuffer::single("abcdef");
        buffer.select_range(case.anchor, case.cursor);
        buffer.remove_range(case.start..case.end);
        assert_state(
            &buffer,
            case.expected_text,
            case.expected_cursor,
            case.expected_selection,
            case.label,
        );
    }

    let mut fully_removed = TextBuffer::single("abcdef");
    fully_removed.select_range(4, 2);
    fully_removed.remove_range(0..usize::MAX);
    assert_state(
        &fully_removed,
        "",
        0,
        None,
        "full removal clamps to text end",
    );
}

#[test]
fn empty_reversed_and_out_of_range_removals_are_noops_before_expansion() {
    let mut buffer = TextBuffer::single("e\u{301}x");
    let expected = buffer.text().to_owned();
    let cursor = buffer.cursor_offset();
    for range in [
        1..1,
        Range { start: 3, end: 2 },
        100..usize::MAX,
        usize::MAX..usize::MAX,
    ] {
        buffer.remove_range(range);
        assert_state(
            &buffer,
            &expected,
            cursor,
            None,
            "empty, reversed, or clamped-empty range",
        );
    }

    let mut empty = TextBuffer::single("");
    empty.remove_range(0..usize::MAX);
    assert_state(&empty, "", 0, None, "empty text remains empty");
}

#[test]
fn removal_expands_nonempty_ranges_to_complete_graphemes() {
    struct Case {
        label: &'static str,
        original: &'static str,
        start: usize,
        end: usize,
        expected_text: &'static str,
        expected_cursor: usize,
    }
    let cases = [
        Case {
            label: "combining grapheme",
            original: "e\u{301}x",
            start: 1,
            end: 2,
            expected_text: "x",
            expected_cursor: 1,
        },
        Case {
            label: "wide CJK scalar",
            original: "x日y",
            start: 2,
            end: 3,
            expected_text: "xy",
            expected_cursor: 2,
        },
        Case {
            label: "ZWJ grapheme",
            original: "👩\u{200d}💻x",
            start: 5,
            end: 6,
            expected_text: "x",
            expected_cursor: 1,
        },
        Case {
            label: "regional indicator scalar",
            original: "🇺x",
            start: 1,
            end: 2,
            expected_text: "x",
            expected_cursor: 1,
        },
    ];
    for case in cases {
        let mut buffer = TextBuffer::single(case.original);
        buffer.remove_range(case.start..case.end);
        assert_state(
            &buffer,
            case.expected_text,
            case.expected_cursor,
            None,
            case.label,
        );
    }

    let mut beyond_end = TextBuffer::single("a日z");
    beyond_end.remove_range(2..usize::MAX);
    assert_state(
        &beyond_end,
        "a",
        1,
        None,
        "interior start and MAX end clamp through end",
    );
}

#[test]
fn deletion_join_repairs_anchor_floor_and_cursor_ceil() {
    let mut combining = TextBuffer::multi("e\n\u{301}");
    combining.select_range(1, 2);
    combining.remove_range(1..2);
    assert_state(
        &combining,
        "e\u{301}",
        3,
        Some(0..3),
        "removing newline joins a combining sequence",
    );

    let mut regional_indicators = TextBuffer::single("🇺 🇸");
    regional_indicators.select_range(4, 5);
    regional_indicators.remove_range(4..5);
    assert_state(
        &regional_indicators,
        "🇺🇸",
        8,
        Some(0..8),
        "removing separator joins regional indicators and repairs both endpoints",
    );
}
