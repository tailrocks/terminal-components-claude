// Appended inside the actual, pinned conformance integration target.
// The wrapper consumes the existing registry invocation, not a second manual list.
macro_rules! conformance_suite {
    ($($name:ident => $case:ty),+ $(,)?) => {
        junie_tui_testing::conformance_suite!($($name => $case),+);
        #[test]
        fn architecture_bootstrap_registered_states() {
            $(architecture_bootstrap_case::<$case>();)+
        }
    };
}

fn architecture_bootstrap_case<C: Conformance>() {
    use junie_tui_testing::conformance::driver::CaseApp;
    println!("ARCHCASE|{}|{:?}|{:?}", C::NAME, C::id(), C::PARTS);
    for required in mono_states_required_by(C::caps()) {
        assert!(C::mono_states().contains(&required), "capability-implied state was omitted");
    }
    for state in C::mono_states() {
        for (width, height) in [(40, 12), (8, 4), (0, 0)] {
            println!("ARCHBEGIN|{}|{}|{width}|{height}", C::NAME, state.bits());
            let fixture = C::mono_fixture(*state);
            let mut prepared = Harness::new(CaseApp::<C>::new(fixture.clone()), fixture.theme.clone(), width, height);
            if !C::mono_setup_chords(*state).is_empty() {
                let _ = prepared.tab_to(C::control_id());
                for chord in C::mono_setup_chords(*state) {
                    let _ = prepared.key_mod(chord.code, chord.mods);
                }
            }
            let mut app = CaseApp::<C>::new(fixture.force(*state));
            app.st = prepared.app().st.clone();
            let captured = Harness::new(app, Theme::junie(), width, height);
            println!("ARCHSTATE|{}|{}|{:?}", C::NAME, state.bits(), captured.app().st);
            for cell in captured.buffer().content() {
                println!("ARCHACTUALCELL|{}|{}|{:?}", C::NAME, state.bits(), cell);
            }
            println!("ARCHEND|{}|{}|{width}|{height}", C::NAME, state.bits());
        }
    }
}

#[test]
fn architecture_bootstrap_actual_row_columns_and_measure() {
    const OWNER: Id = Id::root("architecture.bootstrap.owner");
    let mut scene = Scene::new("architecture-row", Theme::junie(), junie_tui::ColorLevel::TrueColor, 40, 12);
    println!("ARCHROWBEGIN");
    println!("ARCHROWPARTS|{:?}|{:?}", Part::custom("external.row.part"), Part::custom("external.column.part"));
    scene.draw(|ui, _| {
        // Same-id child is a genuine library component resolution.
        Button::new(OWNER, "Child").draw(ui, Rect::new(1, 1, 12, 1));
        {
            let mut row = RowUi::new(ui, OWNER, Family::LIST, Variant::DEFAULT, StateFlags::empty(), ItemKey::num(1), Rect::new(1, 3, 30, 1));
            assert_eq!(row.owner(), OWNER, "observation cannot invent a fake owner ID");
            row.part(Part::custom("external.row.part"), 4).text("ROW");
            let mut columns = row.columns(&[junie_tui::Track::Fixed(8), junie_tui::Track::Flex(1)]);
            columns.cell_part(0, Part::custom("external.column.part")).text("COLUMN");
            columns.cell(1).text("CELL");
        }
        Button::new(OWNER, "After").draw(ui, Rect::new(1, 5, 12, 1));
        List::<&str>::new(OWNER).draw(ui, Rect::new(1, 7, 30, 2), &junie_tui::ListState::default(), &["Default row"]);
        let before = ui.styled_queries().len();
        let _ = Button::new(OWNER, "Measure only").measure(ui, junie_tui::Constraints::loose(40, 12));
        assert_eq!(before, ui.styled_queries().len(), "measurement must not create a paint query");
    });
    println!("ARCHROWEND");
    assert!(scene.buffer().content().iter().any(|cell| cell.symbol() == "R"));
}

#[test]
fn architecture_bootstrap_actual_button_slots() {
    const OWNER: Id = Id::root("architecture.bootstrap.slot");
    for status in [Status::Ready, Status::Busy, Status::Loading] {
        let mut paintable = Vec::new();
        for selected in Button::PARTS {
            let used = std::cell::Cell::new(Rect::ZERO);
            let sentinel = |ui: &mut Ui<'_>, area: Rect| {
                used.set(area);
                ui.paint_str(area, "~", ui.surface_style());
            };
            let mut plain = Scene::new("plain-slot", Theme::junie(), junie_tui::ColorLevel::TrueColor, 40, 12);
            let mut altered = Scene::new("altered-slot", Theme::junie(), junie_tui::ColorLevel::TrueColor, 40, 12);
            let area = Rect::new(2, 2, 12, 1);
            let normal = std::cell::Cell::new(Rect::ZERO);
            plain.draw(|ui, _| normal.set(Button::new(OWNER, "Go").icon(GlyphRole::Error).checked(true).status(status).draw(ui, area)));
            altered.draw(|ui, _| {
                assert_eq!(normal.get(), Button::new(OWNER, "Go").icon(GlyphRole::Error).checked(true).status(status).slot(*selected, &sentinel).draw(ui, area));
            });
            let mut changed = 0;
            for y in 0..12 {
                for x in 0..40 {
                    let position = Position::new(x, y);
                    if plain.buffer().cell(position) != altered.buffer().cell(position) {
                        assert!(used.get().contains(position), "slot changed neighboring cells");
                        changed += 1;
                    }
                }
            }
            if changed > 0 {
                paintable.push(*selected);
                println!("ARCHSLOT|{status:?}|{selected:?}|{:?}|{changed}", used.get());
            } else {
                assert!(![Part::GUTTER, Part::ICON, Part::MARKER, Part::LABEL].contains(selected), "documented slot ignored actual paint");
            }
        }
        paintable.sort();
        let mut documented = architecture_documented_button_slots().to_vec();
        documented.sort();
        assert_eq!(paintable, documented, "rustdoc slot set differs from actually paintable set");
    }
}

fn architecture_documented_button_slots() -> &'static [Part] {
    // Replaced from the actual pinned public Overrides rustdoc, never a second
    // independently maintained component slot declaration.
    &[/* ARCHITECTURE_DOCUMENTED_BUTTON_SLOTS */]
}

#[derive(Debug)]
struct ArchitectureSlotApp {
    status: Status,
    replacement: Option<Part>,
    activations: u32,
}
impl ArchitectureSlotApp {
    fn props(status: Status) -> Button<'static> {
        Button::new(Id::root("architecture.bootstrap.live-slot"), "Go")
            .icon(GlyphRole::Error).checked(true).status(status)
    }
}
impl junie_tui::App for ArchitectureSlotApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Self::props(self.status).update(cx).on_action(|_| self.activations += 1)
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let sentinel = |ui: &mut Ui<'_>, area: Rect| { ui.paint_str(area, "~", ui.surface_style()); };
        let props = Self::props(self.status);
        if let Some(part) = self.replacement {
            props.slot(part, &sentinel).draw(ui, Rect::new(2, 2, 12, 1));
        } else {
            props.draw(ui, Rect::new(2, 2, 12, 1));
        }
    }
}

#[test]
fn architecture_bootstrap_actual_slot_hit_focus() {
    const OWNER: Id = Id::root("architecture.bootstrap.live-slot");
    for status in [Status::Ready, Status::Busy, Status::Loading] {
        for part in [Part::GUTTER, Part::ICON, Part::MARKER, Part::LABEL] {
            let mut plain = Harness::new(ArchitectureSlotApp { status, replacement: None, activations: 0 }, Theme::junie(), 40, 12);
            let mut altered = Harness::new(ArchitectureSlotApp { status, replacement: Some(part), activations: 0 }, Theme::junie(), 40, 12);
            assert_eq!(plain.area_of(OWNER), altered.area_of(OWNER));
            assert_eq!(plain.focus(), altered.focus());
            assert_eq!(plain.ring().reachable().map(|entry| entry.id).collect::<Vec<_>>(), altered.ring().reachable().map(|entry| entry.id).collect::<Vec<_>>());
            for target in Button::PARTS {
                assert_eq!(plain.area_of_part(OWNER, PartRef::of(*target)), altered.area_of_part(OWNER, PartRef::of(*target)));
            }
            let _ = plain.click_id(OWNER);
            let _ = altered.click_id(OWNER);
            let _ = plain.key(KeyCode::Enter);
            let _ = altered.key(KeyCode::Enter);
            assert_eq!(plain.app().activations, altered.app().activations);
            assert_eq!(plain.focus(), altered.focus());
            if matches!(status, Status::Ready) { assert_eq!(plain.app().activations, 2); }
            else { assert_eq!(plain.app().activations, 0); }
            println!("ARCHSLOTHIT|{status:?}|{part:?}|{:?}|{}", plain.area_of(OWNER), plain.app().activations);
        }
    }
}
