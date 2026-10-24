use core::cell::Cell;

use ratatui_core::layout::Position;
use termrock_collections::ScrollState;
use termrock_core::{Id, Part, PartRef};
use termrock_runtime::{App, Color, Cx, Rect, Response, ScrollRegion, Style, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const OWNER: Id = Id::root("recovered.scrollbar.visibility");
const FRAME_WIDTH: u16 = 12;
const FRAME_HEIGHT: u16 = 9;
const AREA: Rect = Rect::new(3, 2, 6, 4);
const SENTINEL_FG: Color = Color::Rgb(213, 174, 117);
const SENTINEL_BG: Color = Color::Rgb(37, 51, 69);
const SENTINEL_GLYPHS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

fn paint_outside_sentinels(ui: &mut Ui<'_>) {
    let frame = ui.full();
    let style = Style::new().fg(SENTINEL_FG).bg(SENTINEL_BG);
    for y in frame.y..frame.bottom() {
        for x in frame.x..frame.right() {
            if (AREA.x..AREA.right()).contains(&x) && (AREA.y..AREA.bottom()).contains(&y) {
                continue;
            }
            let row = usize::from(y.saturating_sub(frame.y));
            let column = usize::from(x.saturating_sub(frame.x));
            let offset = row
                .saturating_mul(usize::from(frame.width))
                .saturating_add(column)
                % SENTINEL_GLYPHS.len();
            let mut symbol_bytes = [0; 4];
            let symbol = char::from(SENTINEL_GLYPHS[offset]).encode_utf8(&mut symbol_bytes);
            ui.paint_str(Rect::new(x, y, 1, 1), symbol, style);
        }
    }
}

struct SentinelPage;

impl App for SentinelPage {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        paint_outside_sentinels(ui);
    }
}

struct ScrollbarPage {
    state: ScrollState,
    visible: bool,
    content_len: usize,
    returned_area: Cell<Option<Rect>>,
}

impl App for ScrollbarPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        ScrollRegion::new(OWNER)
            .scrollbar_visible(self.visible)
            .update(cx, &mut self.state, self.content_len)
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        paint_outside_sentinels(ui);

        let returned = ScrollRegion::new(OWNER)
            .scrollbar_visible(self.visible)
            .draw(ui, AREA, &self.state, self.content_len);
        self.returned_area.set(Some(returned));
    }
}

fn page(visible: bool, content_len: usize) -> ScrollbarPage {
    let mut state = ScrollState::new(content_len);
    state.set_viewport(usize::from(AREA.height));
    state.scroll_to(if content_len > usize::from(AREA.height) {
        7
    } else {
        0
    });
    ScrollbarPage {
        state,
        visible,
        content_len,
        returned_area: Cell::new(None),
    }
}

fn harness(visible: bool, content_len: usize) -> Harness<ScrollbarPage> {
    Harness::new(
        page(visible, content_len),
        Theme::junie(),
        FRAME_WIDTH,
        FRAME_HEIGHT,
    )
}

fn sentinel_control() -> Harness<SentinelPage> {
    Harness::new(SentinelPage, Theme::junie(), FRAME_WIDTH, FRAME_HEIGHT)
}

fn assert_outside_cells(h: &Harness<ScrollbarPage>, expected: &Harness<SentinelPage>) {
    let mut checked = 0;
    for y in 0..FRAME_HEIGHT {
        for x in 0..FRAME_WIDTH {
            if (AREA.x..AREA.right()).contains(&x) && (AREA.y..AREA.bottom()).contains(&y) {
                continue;
            }
            let position = Position::new(x, y);
            let actual = h
                .buffer()
                .cell(position)
                .expect("outside cell is inside the frame");
            let sentinel = expected
                .buffer()
                .cell(position)
                .expect("outside sentinel is inside the frame");
            assert_ne!(sentinel.symbol(), " ", "control paints ({x}, {y})");
            assert_eq!(actual, sentinel, "full outside cell at ({x}, {y})");
            checked += 1;
        }
    }
    let expected_outside = usize::from(FRAME_WIDTH)
        .saturating_mul(usize::from(FRAME_HEIGHT))
        .saturating_sub(usize::from(AREA.width).saturating_mul(usize::from(AREA.height)));
    assert_eq!(
        checked, expected_outside,
        "inspect every outside-frame cell"
    );
}

#[test]
fn fitting_scroll_region_keeps_full_area_and_matches_explicit_no_bar_control() {
    let sentinel = sentinel_control();
    let visible = harness(true, usize::from(AREA.height));
    let hidden = harness(false, usize::from(AREA.height));

    assert_eq!(visible.app().returned_area.get(), Some(AREA));
    assert_eq!(hidden.app().returned_area.get(), Some(AREA));
    assert!(
        visible
            .area_of_part(OWNER, PartRef::of(Part::TRACK))
            .is_none()
    );
    assert!(
        visible
            .area_of_part(OWNER, PartRef::of(Part::THUMB))
            .is_none()
    );

    let rightmost_x = AREA.right().saturating_sub(1);
    for y in AREA.y..AREA.bottom() {
        assert_eq!(
            visible.buffer().cell(Position::new(rightmost_x, y)),
            hidden.buffer().cell(Position::new(rightmost_x, y)),
            "fit path must keep the normal container fill at ({rightmost_x}, {y})"
        );
    }
    assert_outside_cells(&visible, &sentinel);
    assert_outside_cells(&hidden, &sentinel);
    assert!(visible.diagnostics().is_empty());
    assert!(hidden.diagnostics().is_empty());
}

#[test]
fn overflowing_scroll_region_reserves_and_paints_a_real_track_inside_its_area() {
    let sentinel = sentinel_control();
    let visible = harness(true, 30);
    let hidden = harness(false, 30);

    let expected_content = Rect {
        width: AREA.width.saturating_sub(1),
        ..AREA
    };
    assert_eq!(visible.app().returned_area.get(), Some(expected_content));
    assert_eq!(hidden.app().returned_area.get(), Some(AREA));

    let track = visible
        .area_of_part(OWNER, PartRef::of(Part::TRACK))
        .expect("overflow registers a visible track");
    let thumb = visible
        .area_of_part(OWNER, PartRef::of(Part::THUMB))
        .expect("overflow registers a visible thumb");
    let track_x = AREA.right().saturating_sub(1);
    assert_eq!(track, Rect::new(track_x, AREA.y, 1, AREA.height));
    assert_eq!(thumb.x, track_x);
    assert!(thumb.height > 0, "the visible thumb has nonzero height");
    assert!(thumb.y >= AREA.y && thumb.bottom() <= AREA.bottom());
    assert!(
        hidden
            .area_of_part(OWNER, PartRef::of(Part::TRACK))
            .is_none()
    );
    assert!(
        hidden
            .area_of_part(OWNER, PartRef::of(Part::THUMB))
            .is_none()
    );

    let mut changed_glyph = false;
    for y in track.y..track.bottom() {
        let position = Position::new(track.x, y);
        let shown = visible
            .buffer()
            .cell(position)
            .expect("track glyph is inside the frame");
        let control = hidden
            .buffer()
            .cell(position)
            .expect("no-bar control cell is inside the frame");
        if shown.symbol() != control.symbol() {
            changed_glyph = true;
            assert_ne!(shown.symbol(), " ", "track painting writes a real glyph");
        }
    }
    assert!(
        changed_glyph,
        "visible track glyphs differ from the no-bar control"
    );

    assert_outside_cells(&visible, &sentinel);
    assert_outside_cells(&hidden, &sentinel);
    assert!(visible.diagnostics().is_empty());
    assert!(hidden.diagnostics().is_empty());
}
