//! Pickers: a searchable modal list with groups, scopes, tags and an
//! alternate action. The page opens three flavours from buttons.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::focus::Focus;
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::Scrollable;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layer::{DismissReason, Dismissed, LayerCx, LayerTag, ShellLayers, Taken};
use junie_tui::ui::layers::LayerResult;
use junie_tui::ui::text::fuzzy;
use junie_tui::widgets::button::{Button, row_layout};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::picker::{Picker, PickerAction, PickerItem};
use junie_tui::widgets::props::{self, Prop};
use junie_tui::widgets::tree::TreeNode;

const ID: WidgetId = WidgetId::of("pickers");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Quick,
    Tabs,
    Level,
}

impl Kind {
    const ALL: [Kind; 3] = [Kind::Quick, Kind::Tabs, Kind::Level];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    All,
    Files,
    Tasks,
}

struct Entry {
    label: String,
    detail: String,
    glyph: &'static str,
    group: &'static str,
}

fn flatten(nodes: &[TreeNode], path: &str, out: &mut Vec<Entry>) {
    for n in nodes {
        let p = if path.is_empty() {
            n.label.clone()
        } else {
            format!("{path}/{}", n.label)
        };
        if n.children.is_empty() {
            out.push(Entry {
                label: n.label.clone(),
                detail: p.clone(),
                glyph: "F",
                group: "Files",
            });
        } else {
            flatten(&n.children, &p, out);
        }
    }
}

pub struct PickersPage {
    buttons: Vec<Button>,
    /// The open picker on the library's layer stack, which gives the
    /// button that opened it its focus back when it closes (K9). Its
    /// kind is the tag's `n`.
    layers: ShellLayers<LayerResult>,
    scope: Scope,
    entries: Vec<Entry>,
    tabs: Vec<String>,
    level: usize,
    chosen: Option<(String, String)>,
    opened: u32,
}

impl PickersPage {
    pub fn new() -> Self {
        let mut entries = Vec::new();
        flatten(&crate::data::project_tree(), "", &mut entries);
        for t in crate::data::tasks().into_iter().take(12) {
            entries.push(Entry {
                label: t.name,
                detail: format!("#{} · {}", t.id, t.owner),
                glyph: "T",
                group: "Tasks",
            });
        }
        Self {
            buttons: vec![
                Button::primary(ID.sub("quick"), "Open quickly"),
                Button::secondary(ID.sub("tabs"), "Switch tab"),
                Button::secondary(ID.sub("level"), "Choose a level"),
            ],
            layers: ShellLayers::new(),
            scope: Scope::All,
            entries,
            tabs: vec![
                "Query 1".into(),
                "orders".into(),
                "order_items".into(),
                "History".into(),
            ],
            level: 3,
            chosen: None,
            opened: 0,
        }
    }

    fn kind(&self) -> Option<Kind> {
        self.layers.top().map(|e| Kind::ALL[e.tag.n])
    }

    fn picker_mut(&mut self) -> Option<&mut Picker> {
        self.layers.top_as_mut::<Picker>()
    }

    /// Refill the open picker's rows (a query, scope or row change).
    fn refill(&mut self, kind: Kind) {
        let mut layers = std::mem::take(&mut self.layers);
        if let Some(p) = layers.top_as_mut::<Picker>() {
            self.fill(kind, p);
        }
        self.layers = layers;
    }

    fn open(&mut self, kind: Kind, focus: &mut Focus) {
        let mut p = match kind {
            Kind::Quick => {
                let mut p = Picker::new(ID.sub("picker.quick"), "Open quickly");
                p.placeholder = "Files and tasks…".into();
                p.width = 80;
                p
            }
            Kind::Tabs => {
                let mut p = Picker::new(ID.sub("picker.tabs"), "Open tabs");
                p.placeholder = "Filter tabs…".into();
                p.width = 48;
                p
            }
            Kind::Level => {
                let mut p = Picker::new(ID.sub("picker.level"), "Safe Mode · this connection");
                p.searchable = false;
                p.width = 112;
                p
            }
        };
        self.opened += 1;
        self.fill(kind, &mut p);
        let n = Kind::ALL.iter().position(|k| *k == kind).unwrap_or(0);
        self.layers
            .open(Box::new(p), LayerTag::new("picker").n(n), (), focus);
    }

    fn fill(&self, kind: Kind, p: &mut Picker) {
        let q = p.query.clone();
        let items = match kind {
            Kind::Quick => {
                let mut ranked: Vec<(u32, PickerItem)> = self
                    .entries
                    .iter()
                    .filter(|e| match self.scope {
                        Scope::All => true,
                        Scope::Files => e.group == "Files",
                        Scope::Tasks => e.group == "Tasks",
                    })
                    .filter_map(|e| {
                        let (pen, matched) = fuzzy(&e.label, &q)?;
                        Some((
                            pen + if e.group == "Files" { 0 } else { 5 },
                            PickerItem {
                                label: e.label.clone(),
                                detail: e.detail.clone(),
                                glyph: e.glyph,
                                group: e.group,
                                tag: None,
                                matched,
                                disabled: false,
                                key: String::new(),
                            },
                        ))
                    })
                    .collect();
                ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.label.cmp(&b.1.label)));
                p.scope = Some(format!(
                    "{} · Tab scope",
                    match self.scope {
                        Scope::All => "All",
                        Scope::Files => "Files",
                        Scope::Tasks => "Tasks",
                    }
                ));
                ranked.into_iter().map(|r| r.1).collect()
            }
            Kind::Tabs => self
                .tabs
                .iter()
                .enumerate()
                .filter_map(|(i, t)| {
                    let (_, matched) = fuzzy(t, &q)?;
                    Some(PickerItem {
                        label: t.clone(),
                        detail: if i == 0 {
                            "query".into()
                        } else {
                            "public · data".into()
                        },
                        glyph: if i == 0 { "≡" } else { "T" },
                        group: "Open tabs",
                        tag: if i == 1 { Some("active") } else { None },
                        matched,
                        disabled: false,
                        key: String::new(),
                    })
                })
                .collect(),
            Kind::Level => [
                (
                    "Silent",
                    "Writes run without asking. Destructive statements still confirm.",
                ),
                ("Alert", "Every write asks for confirmation before it runs."),
                (
                    "Alert (Full)",
                    "Every statement, reads included, asks for confirmation.",
                ),
                (
                    "Safe Mode",
                    "Writes ask for confirmation and a deliberate acknowledgement.",
                ),
                (
                    "Safe Mode (Full)",
                    "Every statement asks for confirmation and a deliberate acknowledgement.",
                ),
                (
                    "Read-Only",
                    "Writes are refused. Reads and exports still work.",
                ),
            ]
            .iter()
            .enumerate()
            .map(|(i, (l, d))| PickerItem {
                label: (*l).to_owned(),
                detail: (*d).to_owned(),
                glyph: " ",
                group: "",
                tag: if i == self.level {
                    Some("current")
                } else {
                    None
                },
                matched: vec![],
                disabled: false,
                key: String::new(),
            })
            .collect(),
        };
        p.set_items(items);
        if kind == Kind::Level {
            p.cursor = self.level;
        }
    }

    /// Handle what the open picker reported: an event keeps it open, a
    /// result closed it (and the stack gave focus back, K9).
    fn on_taken(&mut self, kind: Kind, taken: Taken<LayerResult>, cx: &mut PageCtx) {
        let (ev, picker) = match taken {
            Taken::Event(LayerResult::Picker(ev)) => (ev, None),
            Taken::Closed(entry, LayerResult::Picker(ev)) => (ev, entry.get::<Picker>().cloned()),
            _ => return,
        };
        match ev {
            PickerAction::QueryChanged => self.refill(kind),
            PickerAction::Chosen(i) | PickerAction::ChosenAlt(i) => {
                let alt = matches!(ev, PickerAction::ChosenAlt(_));
                if let Some(it) = picker.as_ref().and_then(|p| p.items.get(i)) {
                    if kind == Kind::Level {
                        self.level = i;
                    }
                    self.chosen = Some((it.label.clone(), it.detail.clone()));
                    cx.status(format!(
                        "{} {}",
                        if alt { "Opened in a new tab:" } else { "Chose" },
                        it.label
                    ));
                }
            }
            PickerAction::Secondary(i) => {
                if kind == Kind::Tabs && i < self.tabs.len() && self.tabs.len() > 1 {
                    let name = self.tabs.remove(i);
                    cx.status(format!("Closed {name}"));
                    self.refill(kind);
                }
            }
            // free text is not an item in any showcase picker: the query stays,
            // the picker stays open
            PickerAction::Submit => {}
            PickerAction::NextScope | PickerAction::PrevScope => {
                if kind == Kind::Quick {
                    let back = ev == PickerAction::PrevScope;
                    self.scope = match (self.scope, back) {
                        (Scope::All, false) | (Scope::Tasks, true) => Scope::Files,
                        (Scope::Files, false) | (Scope::All, true) => Scope::Tasks,
                        (Scope::Tasks, false) | (Scope::Files, true) => Scope::All,
                    };
                    self.refill(kind);
                }
            }
            PickerAction::Cancelled | PickerAction::Back => {}
        }
    }

    /// Take everything the open picker reported.
    fn drain(&mut self, cx: &mut PageCtx) {
        while let Some(kind) = self.kind() {
            let Some(taken) = self.layers.take(cx.focus) else {
                break;
            };
            self.on_taken(kind, taken, cx);
        }
    }
}

impl Page for PickersPage {
    fn layer_open(&self) -> bool {
        !self.layers.is_empty()
    }
    fn title(&self) -> &'static str {
        "Pickers"
    }
    fn blurb(&self) -> &'static str {
        "One modal list for files, tabs and levels: search, scope, tag, alternate action"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let rows = crate::pages::layout::rows(area, &[7, 1, 0]);
        let panel = Panel::card(Some("Open a picker"));
        let bg = panel.bg(t);
        let inner = panel.render(rows[0], buf, t);
        let widths: Vec<u16> = self.buttons.iter().map(|b| b.width()).collect();
        let rects = row_layout(Rect::new(inner.x, inner.y, inner.width, 1), &widths, 2);
        for (b, r) in self.buttons.iter_mut().zip(rects) {
            b.render(r, buf, ctx, bg);
        }
        if inner.y + 2 < inner.bottom() {
            buf.set_string(
            inner.x,
            inner.y + 2,
            junie_tui::ui::text::truncate(
                "Quick: fuzzy over files and tasks, Tab cycles the scope, Alt+Enter is the alternate action · Tabs: Delete closes a row · Level: no search box",
                inner.width as usize,
            ),
            t.muted().bg(bg),
        );
        }

        let panel = Panel::card(Some("Result"));
        let bg = panel.bg(t);
        let inner = panel.render(
            Rect::new(rows[2].x, rows[2].y, rows[2].width, rows[2].height.min(8)),
            buf,
            t,
        );
        let (label, detail) = self
            .chosen
            .clone()
            .unwrap_or_else(|| ("nothing yet".into(), "—".into()));
        let props = vec![
            Prop::new("Chosen", label),
            Prop::new("Detail", detail),
            Prop::new(
                "Level",
                [
                    "Silent",
                    "Alert",
                    "Alert (Full)",
                    "Safe Mode",
                    "Safe Mode (Full)",
                    "Read-Only",
                ][self.level],
            ),
            Prop::new("Open tabs", self.tabs.join(" · ")),
            Prop::new("Pickers opened", self.opened.to_string()),
        ];
        props::render(inner, buf, t, &props, bg);

        if let Some(kind) = self.kind()
            && let Some(p) = self.picker_mut()
        {
            ctx.begin_modal(buf);
            let hints = match kind {
                Kind::Quick => {
                    "↑↓ Move · Enter Open · Alt+Enter New tab · Tab Scope · Esc Clear / Close"
                }
                Kind::Tabs => "↑↓ Move · Enter Switch · Delete Close tab · Esc Close",
                Kind::Level => "↑↓ Move · Enter Set level · Esc Keep",
            };
            let screen = *buf.area();
            p.hints = hints.to_owned();
            p.draw(screen, buf, ctx);
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        if let Some(top) = self.layers.top_mut() {
            let mut lcx = LayerCx {
                focus: cx.focus,
                ring: cx.ring,
                app: &(),
            };
            let out = match ev {
                PageEvent::Key(key) => top.layer.on_key(key, &mut lcx).or(Outcome::Changed),
                // a click outside the picker is a dismissal under its policy
                PageEvent::Click { id, pos } if top.layer.owns(*id) => {
                    top.layer.on_click(*id, *pos, &mut lcx)
                }
                PageEvent::Click { .. } => {
                    return match self.layers.dismiss(DismissReason::OutsideClick, cx.focus) {
                        Dismissed::Refused => Outcome::Consumed,
                        _ => Outcome::Changed,
                    };
                }
                PageEvent::Wheel { id, delta } if top.layer.owns(*id) => top
                    .layer
                    .on_wheel(*delta, ratatui::layout::Position::default()),
                _ => return Outcome::Ignored,
            };
            self.drain(cx);
            return out;
        }
        match ev {
            PageEvent::Key(key) => {
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                let Some(i) = self.buttons.iter().position(|b| b.id == f) else {
                    return Outcome::Ignored;
                };
                let (o, act) = self.buttons[i].on_key(key);
                if act {
                    self.open(Kind::ALL[i], cx.focus);
                }
                o
            }
            PageEvent::Click { id, .. } => {
                let Some(i) = self.buttons.iter().position(|b| b.id == *id) else {
                    return Outcome::Ignored;
                };
                if self.buttons[i].on_click() {
                    self.open(Kind::ALL[i], cx.focus);
                }
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, _focus: Option<WidgetId>) -> Vec<Hint> {
        if !self.layers.is_empty() {
            vec![("Esc", "Close")]
        } else {
            vec![("Enter", "Open"), ("Tab", "Next")]
        }
    }
}
