// Inserted as a child of the existing private rowui::tests module.
mod architecture_nested {
    use super::*;

    #[test]
    fn architecture_bootstrap_nested_owner_restores_after_callback() {
        const INNER: Id = Id::root("architecture.nested.inner");
        const BEFORE: Part = Part::custom("architecture.outer.before");
        const INSIDE: Part = Part::custom("architecture.inner.column");
        const AFTER: Part = Part::custom("architecture.outer.after");
        let buffer = paint(Rect::new(0, 0, 30, 1), |outer| {
            let previous_owner = outer.owner();
            outer.part(BEFORE, 2).text("A");
            {
                let mut inner = RowUi::new(&mut outer.ui, INNER, Family::LIST, Variant::DEFAULT,
                    StateFlags::empty(), ItemKey::num(2), Rect::new(0, 1, 20, 1));
                let mut columns = inner.columns(&[Track::Fixed(6), Track::Flex(1)]);
                columns.cell_part(0, INSIDE).text("B");
                columns.cell(1).text("D");
            }
            // ARCHITECTURE_NESTED_RETURN: independent lost-restoration fault.
            assert_eq!(outer.owner(), previous_owner, "nested return leaked the inner owner");
            outer.part(AFTER, 2).text("C");
            let queries = outer.ui.styled_queries();
            assert!(queries.iter().any(|(id, _, _, part, _)| *id == previous_owner && *part == BEFORE));
            assert!(queries.iter().any(|(id, _, _, part, _)| *id == INNER && *part == INSIDE));
            assert!(queries.iter().any(|(id, _, _, part, _)| *id == previous_owner && *part == AFTER));
            assert!(!queries.iter().any(|(id, _, _, part, _)| *id == INNER && *part == AFTER));
            println!("ARCHNESTED|{previous_owner:?}|{INNER:?}|{BEFORE:?}|{INSIDE:?}|{AFTER:?}");
        });
        for expected in ["A", "B", "C", "D"] {
            assert!(buffer.content().iter().any(|cell| cell.symbol() == expected), "nested real cell missing");
        }
    }
}
