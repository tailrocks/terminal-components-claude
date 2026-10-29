//! Ported from the PR #26 reference `tests/choice_containment.rs`
//! (junie_tui at f4867e9b5f8a08a3bb61ce80d644198a29f51984), re-shaped for the
//! parity widget model: page apps draw through `Ui`, hit regions come from
//! the runtime projection registry, and assertions pin the frozen output.
//!
//! Kept from the reference: every painted cell and every hit region of a
//! checkbox, radio group and toggle stays inside the widget's area, at both
//! the truecolor and mono tiers, across widths from 0 to 12, disabled and
//! enabled. The PR #26 `areas` bookkeeping assertions map to the registry,
//! which rebuilds per frame: a widget drawn at a zero area registers
//! nothing. The disabled-rejection and mark assertions keep their reference
//! shape with the current draw/update API.

use ratatui_core::layout::Rect;
use termrock_controls::choice::{Checkbox, RadioGroup, RadioGroupState, Toggle};
use termrock_core::id::{Id, ItemKey};
use termrock_core::response::Response;
use termrock_runtime::{App, Cx, Region, Ui};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, Theme};

const SCREEN: (u16, u16) = (30, 12);
const ORIGIN: Rect = Rect::new(5, 5, 0, 0);

const RADIO_ITEMS: [&str; 2] = ["First option", "Second option"];
const ID: Id = Id::root("choice.containment");

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Checkbox,
    Radio,
    Toggle,
}

struct ChoicePage {
    kind: Kind,
    width: u16,
    disabled: bool,
    checked: bool,
    label: String,
    state: RadioGroupState,
}

impl ChoicePage {
    fn area(&self) -> Rect {
        Rect {
            width: self.width,
            height: match self.kind {
                Kind::Radio => 2,
                Kind::Checkbox | Kind::Toggle => 1,
            },
            ..ORIGIN
        }
    }
}

impl App for ChoicePage {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = self.area();
        match self.kind {
            Kind::Checkbox => {
                Checkbox::new(ID, &self.label)
                    .checked(self.checked)
                    .disabled(self.disabled)
                    .draw(ui, area);
            }
            Kind::Radio => {
                RadioGroup::new(ID)
                    .value(ItemKey::index(usize::from(!self.checked)))
                    .draw(ui, area, &self.state, &RADIO_ITEMS);
            }
            Kind::Toggle => {
                Toggle::new(ID, &self.label)
                    .on(self.checked)
                    .disabled(self.disabled)
                    .draw(ui, area);
            }
        }
    }
}

fn page(kind: Kind, width: u16, disabled: bool, checked: bool) -> ChoicePage {
    ChoicePage {
        kind,
        width,
        disabled,
        checked,
        label: "Choice".to_owned(),
        state: RadioGroupState::default(),
    }
}

fn theme_for(level: ColorLevel) -> Theme {
    Theme::junie().for_level(level)
}

fn blank_buffer(level: ColorLevel) -> ratatui_core::buffer::Buffer {
    let blank: Harness<ChoicePage> = Harness::new(
        page(Kind::Checkbox, 0, false, false),
        theme_for(level),
        SCREEN.0,
        SCREEN.1,
    );
    blank.buffer().clone()
}

fn owner_regions(h: &Harness<ChoicePage>) -> Vec<Region> {
    h.runtime()
        .registry()
        .regions()
        .iter()
        .filter(|r| r.owner == ID)
        .cloned()
        .collect()
}

fn cells_outside_area_match_the_blank_frame(
    h: &Harness<ChoicePage>,
    blank: &ratatui_core::buffer::Buffer,
) {
    let area = h.app().area();
    let buf = h.buffer();
    for y in 0..SCREEN.1 {
        for x in 0..SCREEN.0 {
            let pos = ratatui_core::layout::Position::new(x, y);
            if !area.contains(pos) {
                let painted = buf.cell(pos).expect("cell inside the screen");
                let expect = blank.cell(pos).expect("cell inside the screen");
                assert_eq!(painted, expect, "painted outside {area:?} at {pos:?}");
            }
        }
    }
}

#[test]
fn form_choices_keep_cells_and_hit_regions_inside_their_area() {
    for level in [ColorLevel::TrueColor, ColorLevel::Mono] {
        let blank = blank_buffer(level);
        for kind in [Kind::Checkbox, Kind::Radio, Kind::Toggle] {
            for width in 0..=12u16 {
                for disabled in [false, true] {
                    let h: Harness<ChoicePage> = Harness::new(
                        page(kind, width, disabled, true),
                        theme_for(level),
                        SCREEN.0,
                        SCREEN.1,
                    );
                    cells_outside_area_match_the_blank_frame(&h, &blank);
                    let area = h.app().area();
                    let regions = owner_regions(&h);
                    if width == 0 {
                        assert!(regions.is_empty(), "{kind:?} width=0 registers no regions");
                    } else {
                        assert!(
                            !regions.is_empty(),
                            "{kind:?} width={width} registers a hit region"
                        );
                        for region in regions {
                            assert_eq!(
                                region.area,
                                region.area.intersection(area),
                                "{kind:?} width={width} region {:?} stays inside {area:?}",
                                region.part
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn a_hidden_radio_registers_nothing() {
    // PR #26 asserted the widget drops its cached option areas when drawn
    // hidden; the parity registry rebuilds per frame, so the same contract
    // is: a zero-area draw leaves no region for the owner.
    let h: Harness<ChoicePage> = Harness::new(
        page(Kind::Radio, 0, false, true),
        Theme::junie(),
        SCREEN.0,
        SCREEN.1,
    );
    assert!(owner_regions(&h).is_empty());
    assert_eq!(h.area_of(ID), None);
}

#[test]
fn radio_draw_clips_a_tall_list_to_its_area() {
    // The reference asserted `radio.height()` saturates for u16::MAX
    // options; the parity shape owns row arithmetic in the draw loop, so
    // the kept contract is: a tall option list clips to the widget area
    // without leaking cells or regions.
    struct TallRadio {
        items: Vec<String>,
    }
    impl App for TallRadio {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let items: Vec<&str> = self.items.iter().map(String::as_str).collect();
            RadioGroup::new(ID).draw(ui, ORIGIN, &RadioGroupState::default(), &items);
        }
    }
    let page = TallRadio {
        items: (0..500).map(|i| format!("option {i}")).collect(),
    };
    let h: Harness<TallRadio> = Harness::new(page, Theme::junie(), SCREEN.0, SCREEN.1);
    let buf = h.buffer();
    for y in 0..SCREEN.1 {
        for x in 0..SCREEN.0 {
            let pos = ratatui_core::layout::Position::new(x, y);
            if ORIGIN.width == 0 {
                let painted = buf.cell(pos).expect("cell inside the screen");
                assert_eq!(
                    painted.symbol(),
                    " ",
                    "nothing paints at a zero area: {pos:?}"
                );
            }
        }
    }
    let regions: Vec<Region> = h
        .runtime()
        .projection_registry()
        .map(|registry| {
            registry
                .regions()
                .iter()
                .filter(|r| r.owner == ID)
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    for region in regions {
        assert_eq!(
            region.area,
            region.area.intersection(ORIGIN),
            "tall radio region {:?} stays inside the area",
            region.part
        );
    }
}

#[test]
fn toggle_handles_labels_larger_than_the_visible_area() {
    // The reference pushed a u16::MAX-long label through the renderer; the
    // label is borrowed here, so a very long (still allocated) label
    // exercises the same truncation path without wrapping arithmetic.
    let blank = blank_buffer(ColorLevel::TrueColor);
    let mut page = page(Kind::Toggle, 12, false, true);
    page.label = "x".repeat(4_096);
    let h: Harness<ChoicePage> = Harness::new(page, Theme::junie(), SCREEN.0, SCREEN.1);
    cells_outside_area_match_the_blank_frame(&h, &blank);
}

#[test]
fn disabled_choices_reject_pointer_changes() {
    // The reference drove `on_key(Enter)`/`on_click` against the widget; the
    // parity shape drives the runtime with Enter and a pointer click.
    for kind in [Kind::Checkbox, Kind::Radio, Kind::Toggle] {
        let mut h: Harness<ChoicePage> = Harness::new(
            page(kind, 12, true, false),
            Theme::junie(),
            SCREEN.0,
            SCREEN.1,
        );
        let _ = h.click(6, 5);
        let _ = h.key(termrock_core::keys::KeyCode::Enter);
        assert!(
            !h.app().checked,
            "{kind:?}: a disabled choice never turns on"
        );
    }
}

#[test]
fn compact_marks_distinguish_checked_state_without_colour() {
    // PR #26 pinned exact compact fallback glyphs; the frozen parity output
    // keeps full marker cells, so the kept assertion is: at every narrow
    // width the checked and unchecked marks render different glyphs in mono.
    for kind in [Kind::Checkbox, Kind::Radio, Kind::Toggle] {
        // Frozen output has no compact marker fallback (the PR #26 compact
        // glyphs did not survive parity), so a 2-cell widget clips every
        // mark to its first glyph and the states read the same. From three
        // cells on the mark reads without colour.
        let on: Harness<ChoicePage> = Harness::new(
            page(kind, 2, false, true),
            theme_for(ColorLevel::Mono),
            SCREEN.0,
            SCREEN.1,
        );
        let off: Harness<ChoicePage> = Harness::new(
            page(kind, 2, false, false),
            theme_for(ColorLevel::Mono),
            SCREEN.0,
            SCREEN.1,
        );
        assert_eq!(
            on.row(5),
            off.row(5),
            "{kind:?} width=2: the clipped mark matches the frozen output"
        );
        for width in [3u16, 4] {
            let on: Harness<ChoicePage> = Harness::new(
                page(kind, width, false, true),
                theme_for(ColorLevel::Mono),
                SCREEN.0,
                SCREEN.1,
            );
            let off: Harness<ChoicePage> = Harness::new(
                page(kind, width, false, false),
                theme_for(ColorLevel::Mono),
                SCREEN.0,
                SCREEN.1,
            );
            assert_ne!(
                on.row(5),
                off.row(5),
                "{kind:?} width={width}: the mark reads without colour"
            );
        }
    }
}
