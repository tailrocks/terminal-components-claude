//! Independent regressions found against tui-snap e45d3fa.
//! Run in a crate depending on tuisnap and the same vendored vt100.
#[cfg(test)]
mod tests {
    #[test]
    fn all_attribute_combinations_survive_full_and_diff_replay() {
        let mut bytes = Vec::new();
        let sgr = [1, 2, 3, 4, 7, 5, 8, 9];
        for bits in 0..256 {
            bytes.extend_from_slice(b"\x1b[0m");
            for (bit, value) in sgr.iter().enumerate() {
                if bits & (1 << bit) != 0 {
                    bytes.extend_from_slice(format!("\x1b[{value}m").as_bytes());
                }
            }
            bytes.push(b'X');
        }
        for disabled in [false, true] {
            let mut source = vt100::Parser::new(17, 16, 0);
            source.process(&bytes);
            if disabled { source.process(b"\x1b[?7l"); }
            let mut original = vt100::Parser::new(17, 16, 0);
            original.process(b"\x1b[?7l");
            for diff in [false, true] {
                let encoded = if diff { source.screen().state_diff(original.screen()) } else { source.screen().state_formatted() };
                let mut replay = vt100::Parser::new(17, 16, 0);
                replay.process(b"\x1b[?7l");
                replay.process(&encoded);
                for bits in 0..256 {
                    let c = replay.screen().cell(bits / 16, bits % 16).unwrap();
                    for (bit, actual) in [c.bold(), c.dim(), c.italic(), c.underline(), c.inverse(), c.blink(), c.hidden(), c.strikethrough()].into_iter().enumerate() {
                        assert_eq!(actual, bits & (1 << bit) != 0, "bits={bits},bit={bit},disabled={disabled},diff={diff}");
                    }
                    assert_eq!(c.contents(), "X");
                }
                assert_eq!(replay.screen().input_mode_formatted(), source.screen().input_mode_formatted());
            }
        }
    }

    #[test]
    fn state_diff_reenables_wrap_before_emitting_wrapped_contents() {
        let mut source = vt100::Parser::new(3, 8, 0);
        source.process(b"\x1b[?7l");
        let old = source.screen().clone();
        source.process(b"\x1b[?7hABCDEFGHIJ");
        let mut target = vt100::Parser::new(3, 8, 0);
        target.process(&old.state_formatted());
        let diff = source.screen().state_diff(&old);
        target.process(&diff);
        assert_eq!(source.screen().contents(), target.screen().contents(), "diff={:?}", String::from_utf8_lossy(&diff));
    }

    #[test]
    fn state_formatted_replays_on_nondefault_wrap_mode() {
        let mut source = vt100::Parser::new(3, 8, 0);
        source.process(b"ABCDEFGHIJ\x1b[?7l");
        let mut target = vt100::Parser::new(3, 8, 0);
        target.process(b"\x1b[?7l");
        target.process(&source.screen().state_formatted());
        assert_eq!(source.screen().contents(), target.screen().contents());
    }

    #[test]
    fn hidden_svg_preserves_visible_glyph_columns() {
        let mut frame = tuisnap::Frame::blank(4, 1, tuisnap::Provenance::now("test", "test", vec![]));
        frame.cells[0].symbol = "H".into();
        frame.cells[0].mods.hidden = true;
        frame.cells[1].symbol = "A".into();
        frame.cells[2].symbol = "H".into();
        frame.cells[2].mods.hidden = true;
        frame.cells[3].symbol = "B".into();
        let svg = tuisnap::render::render_svg(&frame, &tuisnap::Profile::default_profile());
        assert!(svg.contains("xml:space=\"preserve\"") || svg.contains("white-space:pre") || !svg.contains("> A B</text>"), "{svg}");
    }
}
