use termrock::{Rect, Response, Ui};

/// The section tabs every playground page opens with.
pub const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
/// The caption that names the live section next to the tabs.
pub const SECTION_CAPTION: &str = "the live widget";
/// The width from which the caption has room beside the tabs.
pub const SECTION_WIDE: u16 = 76;

pub struct PageStatus(pub String);

pub struct PageUpdate {
    pub response: Response<()>,
    pub status: Option<PageStatus>,
}

impl From<Response<()>> for PageUpdate {
    fn from(response: Response<()>) -> Self {
        Self {
            response,
            status: None,
        }
    }
}

pub trait Page: Send {
    fn title(&self) -> &'static str;
    fn update(&mut self, cx: &mut termrock::Cx<'_>) -> PageUpdate;
    fn command(
        &mut self,
        _cx: &mut termrock::Cx<'_>,
        _action: termrock::ActionKey,
    ) -> Response<()> {
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>, area: Rect);
    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[]
    }
    fn editing(&self, _ui: &Ui<'_>) -> bool {
        false
    }
    fn seek_paused(&mut self, _frame: usize) {}
}

/// The palette the demo pages paint their chrome with.
///
/// The chrome blocks own the geometry; the pages own the colours. Every
/// demo page uses the same Junie palette, so the pages share one policy
/// value instead of repeating the literals.
pub fn chrome_ink() -> termrock::feedback::ChromeInk {
    use termrock::feedback::ChromeInk;
    ChromeInk {
        canvas: ChromeInk::rgb(0x000000),
        surface: ChromeInk::rgb(0x111111),
        ink: ChromeInk::rgb(0xffffff),
        secondary: ChromeInk::rgb(0xb3b3b3),
        muted: ChromeInk::rgb(0x8a8a8a),
        faint: ChromeInk::rgb(0x737373),
        rule: ChromeInk::rgb(0x3a3a3a),
        accent: ChromeInk::rgb(0x48e054),
        tab_active: ChromeInk::rgb(0x27272a),
    }
}

/// The selection fill behind chosen rows. Works around the junie theme
/// computing `accent_tint` to `#000f2e13` where the reference uses
/// `#173f1c`; other themes pass through untouched.
pub fn accent_tint_bg(theme: &termrock::Theme) -> u32 {
    let drifted = termrock::Color::from_u32(0x000f_2e13);
    if theme.color.accent_tint == drifted {
        0x0017_3f1c
    } else {
        color_u32(theme.color.accent_tint)
    }
}

/// Packs an [`termrock::Color`] into the `0x00rrggbb` form the page paint
/// helpers use. Non-RGB colours degrade to their index (or black).
pub fn color_u32(color: termrock::Color) -> u32 {
    match color {
        termrock::Color::Rgb(r, g, b) => (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b),
        termrock::Color::Indexed(i) => u32::from(i),
        _ => 0,
    }
}

/// Paint one text line per row of the area.
pub fn lines(ui: &mut Ui<'_>, area: Rect, text: &[&str]) {
    let style = ui.surface_style();
    for (offset, line) in text.iter().enumerate() {
        let Ok(offset) = u16::try_from(offset) else {
            break;
        };
        if offset >= area.height {
            break;
        }
        let row = Rect {
            y: area.y.saturating_add(offset),
            height: 1,
            ..area
        };
        let _ = ui.paint_str(row, line, style);
    }
}

pub mod author;
pub mod buttons;
pub mod chips;
pub mod chrome;
pub mod dialogs;
pub mod editable;
pub mod editor;
pub mod forms;
pub mod grid;
pub mod inputs;
pub mod lists;
pub mod overview;
pub mod panels;
pub mod pickers;
pub mod progress;
pub mod scrolling;
pub mod selectable;
pub mod settings;
pub mod sidebars;
pub mod tables;
pub mod taskrunner;
pub mod terminal;
pub mod textareas;
pub mod trees;
