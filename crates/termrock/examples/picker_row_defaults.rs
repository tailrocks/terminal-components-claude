//! Public consumer example for Picker row ink defaults and owner precedence.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use termrock::{
    App, CursorStyle, Cx, Family, FgStep, FilterList, FilterListState, FilterPolicy, Id, Item,
    ItemKey, ItemRowLayout, Part, Rect, Response, Role, Runtime, StylePatch, Theme, Variant,
};

const LIST: Id = Id::root("picker.defaults");

struct PickerRows {
    list: FilterList<'static, Item<'static>>,
    state: FilterListState,
    items: [Item<'static>; 2],
    cursor: CursorStyle,
}

impl PickerRows {
    fn new(cursor: CursorStyle) -> Self {
        Self {
            list: FilterList::new(LIST)
                .filter(FilterPolicy::Caller)
                .item_layout(ItemRowLayout::Columns)
                .cursor_style(cursor),
            state: FilterListState::default(),
            items: [
                Item::new(ItemKey::text("open"), "Configuration")
                    .glyph("⚙")
                    .detail("settings.toml")
                    .tag("edited")
                    .group("Workspace"),
                Item::new(ItemKey::text("review"), "Unicode résumé")
                    .glyph("文")
                    .detail("notes.md")
                    .tag("ready")
                    .group("Workspace"),
            ],
            cursor,
        }
    }
}

impl App for PickerRows {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == termrock::UpdateCause::Bootstrap {
            cx.focus(LIST);
        }
        self.list.update(cx, &mut self.state, &self.items).erase()
    }

    fn draw(&self, ui: &mut termrock::Ui<'_>) {
        let area = match self.cursor {
            CursorStyle::Bar => Rect::new(1, 1, 62, 4),
            CursorStyle::Fill => Rect::new(1, 5, 62, 4),
        };
        self.list.draw(ui, area, &self.state, &self.items);
    }
}

fn render(theme: Theme, cursor: CursorStyle) -> Result<(), Box<dyn std::error::Error>> {
    let area = Rect::new(0, 0, 64, 10);
    let mut runtime = Runtime::new(PickerRows::new(cursor), theme);
    let prepared = runtime.prepare(area)?;
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height))?;
    terminal.draw(|frame| {
        runtime
            .draw_prepared(prepared, frame)
            .expect("prepared area matches the framework terminal");
    })?;
    runtime.acknowledge_presented(prepared)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Both cursor policies consume live rows through the same public component.
    render(Theme::junie(), CursorStyle::Bar)?;
    render(Theme::junie(), CursorStyle::Fill)?;

    // An authored same-value base remains authoritative when focused.
    let owner_theme = Theme::junie().define_family(Family::PICKER, |family| {
        family
            .part(Part::ICON)
            .base(StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
    });
    let _owner_default = owner_theme.resolve(
        Family::PICKER,
        Variant::DEFAULT,
        Part::ICON,
        termrock::StateFlags::FOCUSED,
        termrock::Surface::Canvas,
    );
    render(owner_theme, CursorStyle::Bar)
}
