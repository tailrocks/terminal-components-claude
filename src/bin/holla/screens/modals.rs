//! Custom modals: fixed-choice action menus and searchable pickers behind
//! the shared CustomModal contract. Modals stay pure — world mutations
//! happen in the owning screen when the result lands.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::picker::{Picker, PickerEvent, PickerItem};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use crate::screens::{CustomModal, ModalResult};
use crate::sim::world::World;

/// A picker whose items carry opaque payloads; the result is
/// `ModalResult::Custom(payload)` so the owning screen can act without the
/// modal knowing the world.
pub struct PickerModal {
    picker: Picker,
    /// (payload, label, detail) — the full list; the picker shows the
    /// query-filtered view.
    master: Vec<(String, String, String)>,
    result: Option<ModalResult>,
}

impl PickerModal {
    pub fn new(
        id: WidgetId,
        title: &str,
        items: Vec<(String, String, String)>,
        searchable: bool,
    ) -> Self {
        let mut picker = Picker::new(id, title);
        picker.searchable = searchable;
        let mut m = Self {
            picker,
            master: items,
            result: None,
        };
        m.refilter();
        m
    }

    fn refilter(&mut self) {
        let q = self.picker.query.trim().to_lowercase();
        let items: Vec<PickerItem> = self
            .master
            .iter()
            .filter(|(_, label, detail)| {
                q.is_empty()
                    || label.to_lowercase().contains(&q)
                    || detail.to_lowercase().contains(&q)
            })
            .map(|(_, label, detail)| PickerItem {
                label: label.clone(),
                detail: detail.clone(),
                glyph: "",
                group: "",
                tag: None,
                matched: vec![],
                disabled: false,
            })
            .collect();
        self.picker.set_items(items);
    }

    fn payload(&self, row: usize) -> Option<String> {
        // Map the filtered row back to its master payload.
        let label = self.picker.items.get(row)?.label.clone();
        self.master
            .iter()
            .find(|(_, l, _)| *l == label)
            .map(|(p, _, _)| p.clone())
    }
}

impl CustomModal for PickerModal {
    fn on_key(&mut self, key: &Key, _focus: &mut Focus, _ring: &FocusRing, _w: &World) -> Outcome {
        let (outcome, event) = self.picker.on_key(key);
        match event {
            Some(PickerEvent::Chosen(i)) => {
                if let Some(p) = self.payload(i) {
                    self.result = Some(ModalResult::Custom(p));
                }
                Outcome::Changed
            }
            Some(PickerEvent::Cancelled) => {
                self.result = Some(ModalResult::Cancelled);
                Outcome::Changed
            }
            Some(PickerEvent::QueryChanged) => {
                self.refilter();
                Outcome::Changed
            }
            _ => outcome,
        }
    }

    fn on_click(
        &mut self,
        id: WidgetId,
        _pos: Position,
        _focus: &mut Focus,
        _w: &World,
    ) -> Outcome {
        match self.picker.on_click(id) {
            Some(PickerEvent::Chosen(i)) => {
                if let Some(p) = self.payload(i) {
                    self.result = Some(ModalResult::Custom(p));
                }
                Outcome::Changed
            }
            Some(_) => Outcome::Changed,
            None => Outcome::Ignored,
        }
    }

    fn on_wheel(&mut self, delta: i32, _pos: Position) -> Outcome {
        self.picker.on_wheel(delta)
    }

    fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _w: &World) {
        self.picker.render(screen, buf, ctx, "");
    }

    fn done(&mut self) -> Option<ModalResult> {
        self.result.take()
    }

    fn initial_focus(&self) -> WidgetId {
        self.picker.row_id(0)
    }

    fn hints(&self) -> Vec<Hint> {
        vec![
            hint("↑↓", "Move"),
            hint("Enter", "Choose"),
            hint("Esc", "Cancel"),
        ]
    }
}
