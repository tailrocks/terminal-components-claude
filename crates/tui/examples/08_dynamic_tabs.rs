//! `COMPONENT_ARCHITECTURE.md` §17 example 8, verbatim (crate name is temporary: `junie_tui` → `junie_tui` at Slice 5).
#![expect(
    dead_code,
    missing_docs,
    missing_debug_implementations,
    clippy::arithmetic_side_effects,
    reason = "verbatim from §17 example 8"
)]

use junie_tui::{
    Cx, GlyphRole, Id, ItemKey, Rect, Response, RowUi, Tabs, TabsAction, TabsState, Ui, id,
};

pub struct Doc {
    pub key: u64,
    pub title: String,
    pub dirty: bool,
}
const STRIP: Id = id!("strip");

struct Workspace {
    docs: Vec<Doc>,
    strip: TabsState,
    next_key: u64,
}

fn tab_view(d: &Doc, r: &mut RowUi<'_>) {
    r.label(&d.title);
    if d.dirty {
        r.marker(GlyphRole::Dirty);
    }
}

fn strip() -> Tabs<'static, Doc, impl Fn(&Doc) -> ItemKey, impl Fn(&Doc, &mut RowUi<'_>)> {
    Tabs::new(STRIP)
        .key(|d: &Doc| ItemKey::num(d.key))
        .row(tab_view)
        .allow_new(true)
        .closable(true)
}

impl Workspace {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        strip()
            .update(cx, &mut self.strip, &self.docs) // reconcile() runs first, every frame;
            .on_action(|a| match a {
                // the borrow of `self.docs` ended with `update`
                TabsAction::Activated(_k) => { /* the active key, not an index */ }
                TabsAction::Close(k) => self.docs.retain(|d| ItemKey::num(d.key) != k),
                TabsAction::New => {
                    self.next_key += 1;
                    self.docs.insert(
                        0,
                        Doc {
                            key: self.next_key,
                            title: "Untitled".into(),
                            dirty: false,
                        },
                    );
                }
            })
    }
    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        strip().draw(ui, area, &self.strip, &self.docs);
    }
}

fn main() {}
// Insert at position 0: the active tab, the strip window and any pending close still name
// the same `ItemKey`. Nothing is rebuilt; `TabsState` is never reconstructed.
