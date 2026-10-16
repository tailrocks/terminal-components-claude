//! Status & hints (plan §7, SC4): the footer hint bar an app shell owns.
//!
//! Playground: layer precedence (`HintBar::resolve`, the topmost layer
//! that exists wins), alignment (`HintAlign`), the theme's `HintBarStyle`
//! (status share, compact keys, pinned keys), hint priorities
//! (`HintPriority`, `Hint::pinned`), statuses (`HintLayer::status`,
//! `status_parts`, the EDIT badge, `keyhint::render_status`), a toggle
//! hint and the reveal-key advert. Options flip each of them on the
//! resolved bar. States: the bar in every state it has, drawn by
//! `HintLayer` itself.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::app::PageId;
use crate::options::page_options;
use crate::pages::template::{Section, Template};
use crate::pages::{Cx, Hint, Page, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::{BadgeKind, HintAlign, HintBarStyle, StatusKind, Theme, Tone};
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::reveal::Reveal;
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::widgets::hintbar::{HintBar, HintLayer};
use junie_tui::widgets::keyhint::{self, Hint as KeyHint, HintPriority, hint, toggle_hint};

const ID: WidgetId = WidgetId::of("statushints");

/// Width of the resolved bar under `statushints.width=narrow`: too narrow
/// for every hint, so priorities decide what stays.
const NARROW: u16 = 44;

/// Column where a row's bar starts, after its label.
const LABEL_W: u16 = 22;

pub struct StatusHintsPage {
    template: Template,
    /// Value indices of the page options, in `page_options` order.
    opts: Vec<usize>,
}

impl StatusHintsPage {
    pub fn new() -> Self {
        Self {
            template: Template::new(ID, states_grid()),
            opts: page_options(PageId::StatusHints)
                .iter()
                .map(|o| o.default)
                .collect(),
        }
    }

    /// The value of option `key` as the table spells it.
    fn opt(&self, key: &str) -> &'static str {
        let table = page_options(PageId::StatusHints);
        let i = table
            .iter()
            .position(|o| o.key == key)
            .unwrap_or_else(|| panic!("no option {key}"));
        table[i].values[self.opts[i]]
    }

    /// The screen layer's hints, the last one at the chosen priority.
    fn screen_hints(&self) -> Vec<KeyHint> {
        let priority = match self.opt("statushints.priority") {
            "normal" => HintPriority::Normal,
            "pinned" => HintPriority::Pinned,
            _ => HintPriority::Auto,
        };
        vec![
            hint("Enter", "Open"),
            hint("Tab", "Next pane"),
            hint("/", "Filter"),
            toggle_hint(
                "Ctrl+H",
                self.opt("statushints.hidden") == "on",
                "Hide hidden",
                "Show hidden",
            ),
            hint("?", "Help"),
            KeyHint::new("F5", "Refresh").priority(priority),
        ]
    }

    /// The layers that exist, topmost first: a modal, an open menu, the
    /// screen.
    fn layers(&self) -> [Option<HintLayer>; 3] {
        let top = self.opt("statushints.layer");
        let modal = (top == "modal")
            .then(|| HintLayer::new(vec![hint("Enter", "Delete"), hint("Esc", "Cancel")]));
        let menu = (top != "screen").then(|| {
            HintLayer::new(vec![
                hint("↑↓", "Move"),
                hint("Enter", "Choose"),
                hint("Esc", "Close"),
            ])
        });
        let mut screen = HintLayer::new(self.screen_hints());
        match self.opt("statushints.status") {
            "plain" => screen = screen.status("3 rows selected", Tone::Secondary),
            "warning" => screen = screen.status("2 unsaved changes", Tone::Warning),
            "error" => screen = screen.status("Connection lost", Tone::Error),
            "parts" => {
                screen = screen.status_parts(
                    vec![
                        ("saved ".to_owned(), Tone::Muted),
                        ("payments.settlements".to_owned(), Tone::Secondary),
                    ],
                    Tone::Secondary,
                )
            }
            _ => {}
        }
        if self.opt("statushints.badge") == "on" {
            screen = screen.badge("EDIT", BadgeKind::Edit);
        }
        if self.opt("statushints.align") == "center" {
            screen = screen.align(HintAlign::Center);
        }
        [modal, menu, Some(screen)]
    }

    /// The theme's hint-bar style with the page's options.
    fn style(&self, t: &Theme) -> HintBarStyle {
        HintBarStyle {
            compact: self.opt("statushints.compact") == "on",
            status_min_share: self.opt("statushints.share").parse().unwrap_or(50),
            ..t.hintbar
        }
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let style = self.style(t);
        let mut y = area.y;
        let caption = |buf: &mut Buffer, y: u16, text: &str| {
            if y < area.bottom() {
                let text = junie_tui::ui::text::truncate(text, area.width as usize);
                buf.set_string(area.x, y, text, t.title().bg(t.canvas));
            }
        };
        let label = |buf: &mut Buffer, y: u16, text: &str| {
            if y < area.bottom() {
                buf.set_string(area.x + 2, y, text, t.muted().bg(t.canvas));
            }
        };
        let row = |y: u16, w: u16| {
            let x = area.x + LABEL_W;
            Rect::new(x, y, w.min(area.right().saturating_sub(x)), 1)
        };
        let full = area.width.saturating_sub(LABEL_W);

        // layer precedence
        caption(
            buf,
            y,
            "Layers · HintBar::resolve: the topmost layer that exists wins",
        );
        y += 1;
        let layers = self.layers();
        for (name, layer) in ["modal", "menu", "screen"].iter().zip(&layers) {
            if y >= area.bottom() {
                break;
            }
            match layer {
                Some(l) => {
                    label(buf, y, &format!("{name} layer"));
                    let mut plain = l.clone();
                    plain.status = None;
                    plain.status_parts.clear();
                    keyhint::render_parts(
                        row(y, full),
                        buf,
                        t,
                        &plain.hints,
                        None,
                        None,
                        &[],
                        &HintBarStyle {
                            align: HintAlign::Start,
                            ..style
                        },
                    );
                }
                None => label(buf, y, &format!("{name} layer  (none)")),
            }
            y += 1;
        }
        let resolved = HintBar::resolve(&layers);
        let w = if self.opt("statushints.width") == "narrow" {
            NARROW
        } else {
            full
        };
        if y < area.bottom() {
            label(buf, y, "resolved →");
            let r = row(y, w);
            junie_tui::ui::ctx::fill(buf, r, ratatui::style::Style::new().bg(t.surface));
            let s = HintBarStyle {
                align: resolved.align.unwrap_or(style.align),
                ..style
            };
            keyhint::render_parts(
                r,
                buf,
                t,
                &resolved.hints,
                resolved.badge,
                resolved
                    .status
                    .as_ref()
                    .map(|(s, tone)| (s.as_str(), *tone)),
                &resolved.status_parts,
                &s,
            );
        }
        y += 1;
        if y < area.bottom() {
            let s = &style;
            let text = format!(
                "HintBarStyle · align {:?} · status_min_share {}% · compact {} · pinned_keys {}",
                resolved.align.unwrap_or(s.align),
                s.status_min_share,
                if s.compact { "on" } else { "off" },
                s.pinned_keys.join(" "),
            );
            buf.set_string(
                area.x + 2,
                y,
                junie_tui::ui::text::truncate(&text, area.width.saturating_sub(2) as usize),
                t.faint().bg(t.canvas),
            );
        }
        y += 2;

        // alignment
        caption(buf, y, "Alignment · HintAlign");
        y += 1;
        let hints = [
            hint("↑↓", "Move"),
            hint("Enter", "Open"),
            hint("Esc", "Back"),
        ];
        for (name, align) in [
            ("HintAlign::Start", HintAlign::Start),
            ("HintAlign::Center", HintAlign::Center),
        ] {
            if y >= area.bottom() {
                break;
            }
            label(buf, y, name);
            keyhint::render_aligned(row(y, full.min(64)), buf, t, &hints, None, None, align);
            y += 1;
        }
        y += 1;

        // priorities in a narrow row
        caption(
            buf,
            y,
            "Priorities · HintPriority: normal hints drop from the right, pinned ones last",
        );
        y += 1;
        let ranked = [
            hint("Enter", "Open").priority(HintPriority::Normal),
            hint("Tab", "Next pane"),
            hint("Space", "Select").priority(HintPriority::Normal),
            hint("Esc", "Back"),
            hint("F5", "Refresh").pinned(),
        ];
        for (name, w) in [("all fit", 72u16), ("36 columns", 36), ("24 columns", 24)] {
            if y >= area.bottom() {
                break;
            }
            label(buf, y, name);
            keyhint::render(row(y, w), buf, t, &ranked, None, None);
            y += 1;
        }
        if y < area.bottom() {
            buf.set_string(
                area.x + 2,
                y,
                junie_tui::ui::text::truncate(
                    "Auto: pinned when the key is in theme.hintbar.pinned_keys (Esc) · Hint::pinned keeps F5 · Normal drops first",
                    area.width.saturating_sub(2) as usize,
                ),
                t.faint().bg(t.canvas),
            );
        }
        y += 2;

        // statuses
        caption(buf, y, "Statuses · the right edge");
        y += 1;
        let short = [hint("Ctrl+S", "Save"), hint("Esc", "Back")];
        if y < area.bottom() {
            label(buf, y, "render_status");
            keyhint::render_status(
                row(y, full.min(64)),
                buf,
                t,
                &short,
                None,
                Some(("Saved", Some(StatusKind::Ok))),
            );
        }
        y += 1;
        if y < area.bottom() {
            label(buf, y, "render_toned");
            keyhint::render_toned(
                row(y, full.min(64)),
                buf,
                t,
                &short,
                None,
                Some(("Connection lost", Tone::Error)),
            );
        }
        y += 1;
        if y < area.bottom() {
            label(buf, y, "EDIT badge");
            keyhint::render(
                row(y, full.min(64)),
                buf,
                t,
                &keyhint::EDIT_FIELD,
                Some(("EDIT", BadgeKind::Edit)),
                None,
            );
        }
        y += 2;

        // toggle hint and reveal key
        caption(buf, y, "Toggle hints and the reveal key");
        y += 1;
        if y < area.bottom() {
            label(buf, y, "toggle_hint");
            let on = self.opt("statushints.hidden") == "on";
            keyhint::render(
                row(y, full.min(64)),
                buf,
                t,
                &[toggle_hint("Ctrl+H", on, "Hide hidden", "Show hidden")],
                None,
                Some(if on {
                    "hidden files shown"
                } else {
                    "hidden files hidden"
                }),
            );
        }
        y += 1;
        if y < area.bottom()
            && let Some(key) = Reveal::default().key_label()
        {
            label(buf, y, "reveal key");
            keyhint::render(
                row(y, full.min(64)),
                buf,
                t,
                &[hint("↑↓", "Move"), KeyHint::new(key, t.strings.reveal_open)],
                None,
                Some("shown while a row is cut"),
            );
        }
    }
}

/// The States grid: the hint bar at rest, with a long status, an error
/// status, the EDIT badge and in the narrow slot, drawn by `HintLayer`.
fn states_grid() -> StateGrid {
    use SpecimenState::*;
    let hints = vec![
        hint("↑↓", "Move"),
        hint("Enter", "Open"),
        hint("/", "Filter"),
        hint("Esc", "Back"),
    ];
    StateGrid::new(ID.sub("states"))
        .rows(&[Rest, Overflow, Error, Editing, Narrow])
        .column_width(40)
        .column("Start", HintLayer::new(hints.clone()))
        .column(
            "Center",
            HintLayer::new(hints.clone()).align(HintAlign::Center),
        )
        .column(
            "Pinned F5",
            HintLayer::new(
                hints
                    .iter()
                    .cloned()
                    .chain([hint("F5", "Refresh").pinned()])
                    .collect(),
            ),
        )
}

impl Page for StatusHintsPage {
    fn title(&self) -> &'static str {
        "Status & hints"
    }

    fn blurb(&self) -> &'static str {
        "The footer hint bar: layers, alignment, priorities, statuses, badges"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let body = self.template.render(area, buf, ctx);
        match self.template.section() {
            Section::Playground => self.render_playground(body, buf, ctx),
            Section::Options => self.template.render_options(body, buf, ctx),
            Section::States => self.template.render_states(body, buf, ctx),
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut Cx) -> Outcome {
        if let Some(o) = self.template.handle(ev, cx) {
            return o;
        }
        Outcome::Ignored
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        self.template.hints(focus).unwrap_or_default()
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        let i = page_options(PageId::StatusHints)
            .iter()
            .position(|o| o.key == key)?;
        Some(self.opts[i])
    }

    fn set_option(&mut self, key: &str, value: usize) {
        let table = page_options(PageId::StatusHints);
        if let Some(i) = table.iter().position(|o| o.key == key) {
            self.opts[i] = value.min(table[i].values.len() - 1);
        }
    }

    fn template(&self) -> Option<&Template> {
        Some(&self.template)
    }

    fn template_mut(&mut self) -> Option<&mut Template> {
        Some(&mut self.template)
    }
}

pub fn page() -> Box<dyn Page> {
    Box::new(StatusHintsPage::new())
}
