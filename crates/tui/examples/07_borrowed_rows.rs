//! `COMPONENT_ARCHITECTURE.md` §17 example 7, verbatim (crate name is temporary: `junie_tui` → `junie_tui` at Slice 5).
#![expect(
    dead_code,
    missing_docs,
    missing_debug_implementations,
    reason = "verbatim from §17 example 7"
)]

use junie_tui::{
    Align, Cx, EmptyState, FgStep, GlyphRole, Id, ItemKey, List, ListAction, ListState, Part, Rect,
    Response, Role, RowUi, SelectMode, Ui, id,
};

pub struct Order {
    pub id: u64,
    pub customer: String,
    pub total_cents: i64,
    pub flagged: bool,
}

const ORDERS: Id = id!("orders");

struct Screen {
    orders: Vec<Order>,
    list: ListState,
    chosen: Option<u64>,
}

fn order_row(o: &Order, r: &mut RowUi<'_>) {
    if o.flagged {
        r.marker(GlyphRole::WarningMark);
    }
    r.label(&o.customer); // borrowed &str, one grapheme walk, 0 allocs
    r.part(Part::META, 12) // 12 columns reserved from the right
        .money(o.total_cents)
        .align(Align::Right) // formats into the cell, no String
        .tone(if o.total_cents < 0 {
            Role::Danger
        } else {
            Role::Fg(FgStep::Muted)
        });
}

/// Configuration and closures only — the rows are passed to each phase call (§21 item 1),
/// so the props never borrow `self.orders` and the action closure is free to mutate `self`.
fn orders_list() -> List<'static, Order, impl Fn(&Order) -> ItemKey, impl Fn(&Order, &mut RowUi<'_>)>
{
    List::new(ORDERS)
        .key(|o: &Order| ItemKey::num(o.id))
        .row(order_row)
        .select_mode(SelectMode::Single)
        .empty(EmptyState::Empty {
            title: "No orders",
            hint: Some("Adjust the filter"),
        })
}

impl Screen {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        orders_list()
            .update(cx, &mut self.list, &self.orders)
            .on_action(|a| {
                if let ListAction::Chose(k) = a {
                    self.chosen = self
                        .orders
                        .iter()
                        .find(|o| ItemKey::num(o.id) == k)
                        .map(|o| o.id);
                }
            })
    }
    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        orders_list().draw(ui, area, &self.list, &self.orders);
    }
}

fn main() {}
// Nothing is converted to owned strings, only visible rows invoke the renderer, and the
// action carries `ItemKey`, never a display index.
