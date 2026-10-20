use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::hit::Axis;
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::core::keymap::unassigned_chord;
use crate::core::keymap::{Actions, KeyBinding, KeyMap, NavKeys, lookup};
use crate::core::list::{Cursor, Navigable, RowSource};
use crate::theme::{ButtonKind, Kind};
use crate::ui::context::ContextConfig;
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::text::{truncate, width};
use crate::ui::tooltip::Tip;
use crate::widgets::icon_button::IconButton;
use crate::widgets::menu::{ContextMenu, MenuAction, MenuItem};

pub fn overflow_run(widths: &[u16], avail: u16, gap: u16) -> (usize, Option<String>) {
    let total: u32 = widths.iter().map(|&w| w as u32).sum::<u32>()
        + gap as u32 * widths.len().saturating_sub(1) as u32;
    if total <= avail as u32 {
        return (widths.len(), None);
    }
    let n = widths.len();
    for k in (0..n).rev() {
        let marker = format!("+{}", n - k);
        let used: u32 = widths[..k]
            .iter()
            .map(|&w| w as u32 + gap as u32)
            .sum::<u32>()
            + width(&marker) as u32;
        if used <= avail as u32 {
            return (k, Some(marker));
        }
    }
    (0, None)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chip {
    pub label: String,
    pub enabled: bool,
    pub removable: bool,
    pub error: bool,
}

impl Chip {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            enabled: true,
            removable: true,
            error: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChipBar {
    pub id: WidgetId,
    pub chips: Vec<Chip>,
    pub cursor: usize,
    pub add_label: Option<String>,

    pub lead: Option<String>,
    pub area: Rect,

    pub overflow_marker: bool,

    pub nav: NavKeys,

    pub keymap: KeyMap<ChipCmd>,

    pub close_button: IconButton,

    pub overflow_menu: bool,

    pub more_button: IconButton,

    pub digit_toggles: bool,

    pub context_menu: bool,

    pub context: ContextConfig,

    menu: Option<(ContextMenu, BarMenu)>,

    shown: usize,

    more_area: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipCmd {
    Remove,

    Add,

    ClearAll,

    ShowHidden,

    Lead,
}

impl Actions for ChipCmd {
    fn label(self) -> &'static str {
        match self {
            ChipCmd::Remove => "Remove",
            ChipCmd::Add => "Add",
            ChipCmd::ClearAll => "Clear all",
            ChipCmd::ShowHidden => "Show hidden",
            ChipCmd::Lead => "Switch the match mode",
        }
    }

    fn default_keymap() -> KeyMap<Self> {
        vec![
            (KeyBinding::key(KeyCode::Delete), ChipCmd::Remove),
            (KeyBinding::key(KeyCode::Backspace), ChipCmd::Remove),
            (KeyBinding::char('x'), ChipCmd::Remove),
            (KeyBinding::char('+'), ChipCmd::Add),
            (KeyBinding::char('X'), ChipCmd::ClearAll),
            (KeyBinding::key(KeyCode::Down), ChipCmd::ShowHidden),
            (KeyBinding::char('m'), ChipCmd::Lead),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipAction {
    Activate(usize),
    Toggle(usize),
    Remove(usize),
    Add,
    Lead,
    ClearAll,
}

impl ChipBar {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            chips: vec![],
            cursor: 0,
            add_label: Some("+ Add filter".into()),
            lead: None,
            area: Rect::ZERO,
            overflow_marker: true,
            nav: NavKeys::default(),
            keymap: ChipCmd::default_keymap(),
            close_button: IconButton::close(WidgetId::of("chips.remove")),
            overflow_menu: true,
            more_button: IconButton::new(WidgetId::of("chips.more"), "+"),
            digit_toggles: false,
            context_menu: true,
            context: ContextConfig::DEFAULT,
            menu: None,
            shown: 0,
            more_area: Rect::ZERO,
        }
    }

    pub fn close_target_w(mut self, cells: u16) -> Self {
        self.close_button = self.close_button.plate(cells);
        self
    }

    pub fn close_button(mut self, button: IconButton) -> Self {
        self.close_button = button;
        self
    }

    pub fn more_button(mut self, button: IconButton) -> Self {
        self.more_button = button;
        self
    }

    pub fn overflow_menu(mut self, on: bool) -> Self {
        self.overflow_menu = on;
        self
    }

    pub fn more_id(&self) -> WidgetId {
        self.id.part(PartRef::of(Part::OVERFLOW))
    }

    pub fn menu_id(&self) -> WidgetId {
        self.id.sub("more-menu")
    }

    pub fn menu_open(&self) -> bool {
        self.menu.is_some()
    }

    fn open_menu(&mut self) {
        let hidden: Vec<usize> = (self.shown.min(self.chips.len())..self.chips.len()).collect();
        if hidden.is_empty() {
            return;
        }
        let items = hidden
            .iter()
            .map(|&i| MenuItem::new(self.chips[i].label.clone()))
            .collect();
        let menu =
            ContextMenu::new(self.menu_id(), items).anchor_control(self.more_id(), self.more_area);
        self.menu = Some((menu, BarMenu::Hidden(hidden)));
    }

    pub fn context_items(&self, i: usize) -> Vec<(ChipAction, MenuItem)> {
        use crate::ui::context::{ContextCmd, command_items};
        let Some(chip) = self.chips.get(i) else {
            return Vec::new();
        };
        let toggle = MenuItem::new(if chip.enabled { "Turn off" } else { "Turn on" })
            .shortcut("Space")
            .separator();
        let cmds = [
            ContextCmd::new(ChipCmd::Remove)
                .unless((!chip.removable).then_some("This filter stays")),
            ContextCmd::new(ChipCmd::ClearAll).danger(),
        ];
        let mut items = vec![(ChipAction::Toggle(i), toggle)];
        items.extend(
            command_items(&self.keymap, &cmds)
                .into_iter()
                .map(|(c, it)| {
                    let a = match c {
                        ChipCmd::Remove => ChipAction::Remove(i),
                        _ => ChipAction::ClearAll,
                    };
                    (a, it)
                }),
        );
        items
    }

    fn open_chip_menu(&mut self, i: usize, anchor: crate::ui::context::ContextAnchor) {
        let (actions, items): (Vec<_>, Vec<_>) = self.context_items(i).into_iter().unzip();
        if items.is_empty() {
            return;
        }
        let menu = anchor.apply(ContextMenu::new(self.menu_id(), items));
        self.menu = Some((menu, BarMenu::Chip(actions)));
    }

    fn menu_event(&mut self, ev: Option<MenuAction>) -> Option<ChipAction> {
        match ev? {
            MenuAction::Chosen(k) => match self.menu.take()?.1 {
                BarMenu::Hidden(hidden) => {
                    let i = *hidden.get(k)?;
                    self.cursor = i;
                    Some(ChipAction::Activate(i))
                }
                BarMenu::Chip(actions) => actions.get(k).copied(),
            },
            MenuAction::Dismissed => {
                self.menu = None;
                None
            }
        }
    }

    pub fn digit_toggles(mut self, on: bool) -> Self {
        self.digit_toggles = on;
        self
    }

    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    pub fn keymap(mut self, keymap: KeyMap<ChipCmd>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn overflow_marker(mut self, on: bool) -> Self {
        self.overflow_marker = on;
        self
    }

    pub fn chip_id(&self, i: usize) -> WidgetId {
        self.id.part(PartRef::item(Part::ROW, ItemKey::index(i)))
    }
    pub fn close_id(&self, i: usize) -> WidgetId {
        self.id.part(PartRef::item(Part::CLOSE, ItemKey::index(i)))
    }
    pub fn add_id(&self) -> WidgetId {
        self.id.part(PartRef::of(Part::NEW))
    }

    pub fn lead_id(&self) -> WidgetId {
        self.id.part(PartRef::of(Part::TITLE))
    }

    fn stops(&self) -> usize {
        self.chips.len() + usize::from(self.add_label.is_some())
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<ChipAction>) {
        if let Some((menu, _)) = self.menu.as_mut() {
            let (o, ev) = menu.on_key(key);
            return (o, self.menu_event(ev));
        }
        let n = self.stops();
        if n == 0 {
            return (Outcome::Ignored, None);
        }
        self.cursor = self.cursor.min(n - 1);
        if self.digit_toggles
            && key.plain()
            && let KeyCode::Char(c @ '1'..='9') = key.code
        {
            let i = c as usize - '1' as usize;
            if i < self.chips.len() {
                self.cursor = i;
                return (Outcome::Changed, Some(ChipAction::Toggle(i)));
            }
        }
        if self.context_menu && self.cursor < self.chips.len() && self.context.keys.matches(key) {
            let anchor = crate::ui::context::ContextAnchor::Control {
                id: self.chip_id(self.cursor),
                fallback: self.area,
            };
            self.open_chip_menu(self.cursor, anchor);
            return (Outcome::Changed, None);
        }
        let action = lookup(&self.keymap, key);
        if unassigned_chord(key, &self.keymap) {
            return (Outcome::Ignored, None);
        }
        match action {
            Some(ChipCmd::Remove)
                if self.cursor < self.chips.len() && self.chips[self.cursor].removable =>
            {
                return (Outcome::Changed, Some(ChipAction::Remove(self.cursor)));
            }
            Some(ChipCmd::Add) => return (Outcome::Changed, Some(ChipAction::Add)),
            Some(ChipCmd::ClearAll) => return (Outcome::Changed, Some(ChipAction::ClearAll)),
            Some(ChipCmd::ShowHidden)
                if self.overflow_menu && self.overflow_marker && self.shown < self.chips.len() =>
            {
                self.open_menu();
                return (Outcome::Changed, None);
            }
            Some(ChipCmd::Lead) if self.lead.is_some() => {
                return (Outcome::Changed, Some(ChipAction::Lead));
            }
            _ => {}
        }
        let to = match self.nav.action(key).filter(|_| key.plain()) {
            Some(nav) => self.cursor().apply(nav, self, 1).index(),
            None => None,
        };
        if let Some(to) = to {
            self.cursor = to;
            return (Outcome::Changed, None);
        }
        match key.code {
            KeyCode::Enter => {
                if self.cursor < self.chips.len() {
                    (Outcome::Changed, Some(ChipAction::Activate(self.cursor)))
                } else {
                    (Outcome::Changed, Some(ChipAction::Add))
                }
            }
            KeyCode::Char(' ') if self.cursor < self.chips.len() => {
                (Outcome::Changed, Some(ChipAction::Toggle(self.cursor)))
            }
            _ => (Outcome::Ignored, None),
        }
    }

    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<ChipAction>) {
        if let Some((menu, _)) = self.menu.as_mut()
            && menu.contains_id(id)
        {
            let ev = menu.on_click(id);
            return (Outcome::Changed, self.menu_event(ev));
        }
        if id == self.more_id() {
            if self.menu.take().is_none() {
                self.open_menu();
            }
            return (Outcome::Changed, None);
        }

        self.menu = None;
        if id == self.add_id() {
            self.cursor = self.chips.len();
            return (Outcome::Changed, Some(ChipAction::Add));
        }
        if id == self.lead_id() {
            return (Outcome::Changed, Some(ChipAction::Lead));
        }
        for i in 0..self.chips.len() {
            if self.close_id(i) == id {
                return (Outcome::Changed, Some(ChipAction::Remove(i)));
            }
            if self.chip_id(i) == id {
                self.cursor = i;
                return (Outcome::Changed, Some(ChipAction::Activate(i)));
            }
        }
        (Outcome::Ignored, None)
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = ctx.interaction.focused_under(self.id, self.menu.is_some());
        let plane = t.plane_of(bg);
        let mut x = area.x;
        let y = area.y;
        if let Some(lead) = &self.lead {
            let lid = self.lead_id();
            let mut s = ctx.state(lid);
            s.focused = false;
            let st = t.interactive(Kind::Title, s, plane).style;
            let text = format!(" {lead} ");
            buf.set_string(x, y, &text, st);
            ctx.register_part(
                self.id,
                PartRef::of(Part::TITLE),
                Rect::new(x, y, width(&text) as u16, 1),
            );
            x += width(&text) as u16 + 1;
        }
        let chip_w =
            |c: &Chip| 1 + width(&c.label) as u16 + 1 + if c.removable { 2 } else { 0 } + 1;
        let widths: Vec<u16> = self.chips.iter().map(chip_w).collect();
        let (shown, marker) = overflow_run(&widths, area.right().saturating_sub(x), 1);
        self.shown = shown;
        let mut more_area = Rect::ZERO;
        for (i, chip) in self.chips.iter().enumerate() {
            if i >= shown {
                if let Some(m) = marker.as_ref().filter(|_| self.overflow_marker) {
                    if self.overflow_menu {
                        let left = x.saturating_sub(1).max(area.x);
                        let r = Rect::new(left, y, x + width(m) as u16 + 1 - left, 1)
                            .intersection(area);
                        more_area = self
                            .more_button
                            .with_glyph(m)
                            .part_of(self.id, PartRef::of(Part::OVERFLOW))
                            .render_in(
                                r,
                                x,
                                buf,
                                ctx,
                                ratatui::style::Style::new().bg(bg),
                                Some(
                                    match crate::core::keymap::label_for(
                                        &self.keymap,
                                        ChipCmd::ShowHidden,
                                    ) {
                                        Some(k) => Tip::new(t.strings.tip_overflow).shortcut(&k),
                                        None => Tip::new(t.strings.tip_overflow),
                                    },
                                ),
                            );
                    } else {
                        buf.set_string(x, y, m, t.muted().bg(bg));
                    }
                    x += width(m) as u16 + 1;
                }
                break;
            }
            let cid = self.chip_id(i);

            let mut s = ctx.child_state(self.id, cid, i == self.cursor);
            s.focused = focused && i == self.cursor;
            s.selected = chip.enabled;
            let it = t.interactive(Kind::Chip, s, plane);
            let mut st = it.style;
            if chip.error {
                st = st.fg(t.legible(t.error, st.bg.unwrap_or(bg)));
            }
            let label_w = width(&chip.label) as u16;
            let w = widths[i];
            let r = Rect::new(x, y, w, 1);
            fill(buf, r, st);
            buf.set_string(x, y, it.gutter, it.gutter_style);
            buf.set_string(x + 1, y, truncate(&chip.label, label_w as usize), st);
            if chip.removable {
                ctx.register_part(self.id, PartRef::item(Part::ROW, ItemKey::index(i)), r);

                let gx = x + 2 + label_w;
                let cw = self.close_button.width(t).max(1).min(w);
                let left = gx
                    .saturating_sub(self.close_button.pad)
                    .max(x)
                    .min(x + w - cw);
                let tip = Tip::new(t.strings.tip_remove);
                let tip = match crate::core::keymap::label_for(&self.keymap, ChipCmd::Remove) {
                    Some(k) => tip.shortcut(&k),
                    None => tip,
                };
                self.close_button
                    .part_of(self.id, PartRef::item(Part::CLOSE, ItemKey::index(i)))
                    .render_in(Rect::new(left, y, cw, 1), gx, buf, ctx, st, Some(tip));
            } else {
                ctx.register_part(self.id, PartRef::item(Part::ROW, ItemKey::index(i)), r);
            }
            x += w + 1;
        }
        if let Some(add) = &self.add_label {
            let aid = self.add_id();
            let mut s = ctx.child_state(self.id, aid, self.cursor == self.chips.len());
            s.focused = focused && self.cursor == self.chips.len();
            let it = t.interactive(Kind::Button(ButtonKind::Subtle), s, plane);
            let st = it.style;
            let w = width(add) as u16 + 2;
            if x + w <= area.right() {
                let r = Rect::new(x, y, w, 1);
                fill(buf, r, st);
                buf.set_string(x, y, it.gutter, it.gutter_style);
                buf.set_string(x + 1, y, add, st);
                ctx.register_part(self.id, PartRef::of(Part::NEW), r);
            }
        }
        if !ctx.inert {
            ctx.ring.register(self.id);
        }
        self.more_area = more_area;
        if more_area.is_empty() && matches!(self.menu, Some((_, BarMenu::Hidden(_)))) {
            self.menu = None;
        }
        if let Some((menu, _)) = self.menu.as_mut() {
            menu.render(*buf.area(), buf, ctx);
        }
    }
}

impl crate::ui::measure::Measure for ChipBar {
    fn measure(&self, _avail: ratatui::layout::Size, _ctx: &RenderCtx) -> ratatui::layout::Size {
        let lead = self.lead.as_ref().map_or(0, |l| width(l) as u16 + 3);
        let chips: u16 = self
            .chips
            .iter()
            .map(|c| 1 + width(&c.label) as u16 + 1 + if c.removable { 2 } else { 0 } + 1 + 1)
            .sum();
        let add = self.add_label.as_ref().map_or(0, |a| width(a) as u16 + 2);

        let w = lead
            + if add > 0 {
                chips + add
            } else {
                chips.saturating_sub(1)
            };
        ratatui::layout::Size::new(w, 1)
    }
}

impl RowSource for ChipBar {
    fn len(&self) -> usize {
        self.stops()
    }
    fn label(&self, i: usize) -> &str {
        self.chips.get(i).map_or("", |c| c.label.as_str())
    }
}

impl Navigable for ChipBar {
    fn cursor(&self) -> Cursor {
        Cursor::new(self.cursor).axis(Axis::Horizontal)
    }
}

impl crate::ui::specimen::Specimen for ChipBar {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(
            state,
            Rest | Hover | Focused | FocusedHover | Error | Overflow | Narrow
        )
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        use crate::ui::specimen::SpecimenState;
        let mut b = self.clone();
        b.menu = None;
        let v = state.visual();
        if v.hovered && !b.chips.is_empty() {
            ctx.interaction.hover = Some(b.chip_id(0));
        }
        match state {
            SpecimenState::Error => {
                if let Some(c) = b.chips.first_mut() {
                    c.error = true;
                }
            }
            SpecimenState::Overflow => {
                for l in ["region = 'eu'", "plan = 'pro'", "seats > 5"] {
                    b.chips.push(Chip::new(l));
                }
            }
            _ => {}
        }
        b.render(area, buf, ctx, bg);
    }
}

impl crate::core::pointer::MouseTarget for ChipBar {
    type Action = ChipAction;

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn route(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> Option<crate::core::response::Response<ChipAction>> {
        let menu = self.menu.as_ref().map(|(m, _)| m.id);
        (ev.on(self.id) || menu.is_some_and(|m| ev.on(m))).then(|| self.pointer(ev))
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<ChipAction> {
        use crate::core::intent::Phase;

        if let Some((axis, delta)) = ev.wheel()
            && let Some((menu, _)) = self.menu.as_mut()
        {
            return crate::core::scroll::Scrollable::wheel(&mut menu.list, axis, delta).lift();
        }
        match ev.phase() {
            Some(Phase::Click) => self.on_click(ev.id).into(),

            Some(Phase::Secondary) if self.context_menu => {
                let Some(i) = (0..self.chips.len())
                    .find(|&i| ev.id == self.chip_id(i) || ev.id == self.close_id(i))
                else {
                    return crate::core::response::Response::ignored();
                };
                if self.context.moves_cursor {
                    self.cursor = i;
                }
                self.open_chip_menu(i, crate::ui::context::ContextAnchor::Pointer(ev.pos()));
                crate::core::response::Response::changed()
            }
            _ => crate::core::response::Response::ignored(),
        }
    }
}

#[derive(Debug, Clone)]
enum BarMenu {
    Hidden(Vec<usize>),

    Chip(Vec<ChipAction>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_shows_every_chip_and_the_add_button() {
        use crate::core::focus::FocusRing;
        use crate::core::hit::HitRegistry;
        use crate::ui::measure::Measure;
        let t = crate::theme::Theme::junie();
        let mut bar = ChipBar::new(WidgetId::of("bar"));
        bar.chips = vec![Chip::new("status = 'ok'"), Chip::new("total > 9")];
        let (mut hits, mut ring) = (HitRegistry::default(), FocusRing::default());
        let w = {
            let ctx = RenderCtx::new(
                &t,
                crate::ui::ctx::Interaction::default(),
                &mut hits,
                &mut ring,
            );
            bar.measure(ratatui::layout::Size::new(1, 1), &ctx).width
        };
        let draw = |bar: &mut ChipBar, w: u16, hits: &mut HitRegistry, ring: &mut FocusRing| {
            let mut ctx = RenderCtx::new(&t, crate::ui::ctx::Interaction::default(), hits, ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, w, 1));
            bar.render(Rect::new(0, 0, w, 1), &mut buf, &mut ctx, t.canvas);
            (0..w).map(|x| buf[(x, 0)].symbol()).collect::<String>()
        };
        let add = bar.add_label.clone().expect("an add button");
        let row = draw(&mut bar, w, &mut hits, &mut ring);
        assert_eq!(bar.shown, 2, "{row:?}");
        assert!(row.contains(add.as_str()), "{row:?}");
        let row = draw(&mut bar, w - 1, &mut hits, &mut ring);
        assert!(bar.shown < 2 || !row.contains(add.as_str()), "{row:?}");
    }
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::{ColorLevel, Plane, Theme};
    use crate::ui::ctx::Interaction;

    fn draw(bar: &mut ChipBar, t: &Theme, interaction: Interaction) -> Buffer {
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(t, interaction, &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 1));
        bar.render(Rect::new(0, 0, 40, 1), &mut buf, &mut ctx, t.canvas);
        buf
    }

    fn bar() -> ChipBar {
        let mut bar = ChipBar::new(WidgetId::of("chips"));
        let mut off = Chip::new("beta");
        off.enabled = false;
        bar.chips = vec![Chip::new("alpha"), off];
        bar
    }

    #[test]
    fn a_right_click_on_a_chip_offers_toggle_remove_and_clear_all() {
        use crate::core::intent::Phase;
        use crate::core::pointer::{MouseTarget, PointerEvent};
        use ratatui::crossterm::event::KeyModifiers;
        let t = Theme::junie();
        let screen = Rect::new(0, 0, 40, 8);
        let frame = |b: &mut ChipBar| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut buf = Buffer::empty(screen);
            {
                let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
                b.render(Rect::new(0, 0, 40, 1), &mut buf, &mut ctx, t.canvas);
                ctx.flush_overlays(&mut buf);
            }
            (buf, hits)
        };
        let find = |buf: &Buffer, label: &str| {
            (0..screen.height).find_map(|y| {
                let row: String = (0..screen.width)
                    .map(|x| buf[(x, y)].symbol().to_owned())
                    .collect();
                row.find(label)
                    .map(|i| ratatui::layout::Position::new(row[..i].chars().count() as u16, y))
            })
        };
        let ev = |hits: &HitRegistry, phase, pos| {
            PointerEvent::new(
                hits,
                phase,
                hits.hit(pos).expect("a target"),
                pos,
                1,
                KeyModifiers::NONE,
            )
        };
        let mut b = bar();
        let (buf, hits) = frame(&mut b);
        let beta = find(&buf, "beta").expect("the second chip");
        let r = b
            .route(&ev(&hits, Phase::Secondary, beta))
            .expect("the bar's");
        assert_eq!(r.outcome(), Outcome::Changed);
        assert_eq!(b.cursor, 1, "the menu is about the clicked chip");
        let (buf, hits) = frame(&mut b);
        assert!(find(&buf, "Turn on").is_some(), "beta is off: Turn on");
        let remove = b.context_items(1);
        assert_eq!(
            remove[1].1.shortcut.as_deref(),
            Some("Delete"),
            "from the key map"
        );
        let clear = find(&buf, "Clear all").expect("Clear all row");
        let mut r = b
            .route(&ev(&hits, Phase::Click, clear))
            .expect("the menu's row");
        assert_eq!(r.take_action(), Some(ChipAction::ClearAll));
        assert!(!b.menu_open());

        b.cursor = 0;
        frame(&mut b);
        let shift_f10 = Key {
            code: KeyCode::F(10),
            mods: KeyModifiers::SHIFT,
        };
        assert_eq!(b.on_key(&shift_f10).0, Outcome::Changed);
        assert!(b.menu_open());
        let enter = Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        };
        assert_eq!(
            b.on_key(&enter),
            (Outcome::Changed, Some(ChipAction::Toggle(0)))
        );

        b.context_menu = false;
        let (_, hits) = frame(&mut b);
        let alpha = ratatui::layout::Position::new(2, 0);
        assert!(
            b.route(&ev(&hits, Phase::Secondary, alpha))
                .is_none_or(|r| r.outcome() == Outcome::Ignored)
        );
    }

    #[test]
    fn digit_toggles_switch_the_numbered_chip() {
        let key = |c| Key {
            code: KeyCode::Char(c),
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        let mut b = bar();
        assert_eq!(b.on_key(&key('2')), (Outcome::Ignored, None));
        let mut b = bar().digit_toggles(true);
        assert_eq!(
            b.on_key(&key('2')),
            (Outcome::Changed, Some(ChipAction::Toggle(1)))
        );
        assert_eq!(b.cursor, 1);

        assert_eq!(b.on_key(&key('3')), (Outcome::Ignored, None));
    }

    #[test]
    fn the_lead_key_runs_the_lead_like_a_click() {
        let m = Key {
            code: KeyCode::Char('m'),
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        let mut b = bar();
        assert_eq!(b.on_key(&m), (Outcome::Ignored, None));
        b.lead = Some("match all ▾".into());
        assert_eq!(b.on_key(&m), (Outcome::Changed, Some(ChipAction::Lead)));

        let lead = b.lead_id();
        assert_eq!(b.on_click(lead).1, Some(ChipAction::Lead));
    }

    #[test]
    fn hovering_the_close_glyph_does_not_hover_the_chip_and_it_presses() {
        let t = Theme::junie();
        let mut b = bar();
        let x_id = b.close_id(0);

        let rest = draw(&mut b, &t, Interaction::default());
        let x = (0..40).find(|&x| rest[(x, 0)].symbol() == "×").unwrap();
        let hover = draw(
            &mut b,
            &t,
            Interaction {
                hover: Some(x_id),
                ..Default::default()
            },
        );
        assert_eq!(hover[(2, 0)].bg, rest[(2, 0)].bg, "chip body stays at rest");
        assert_ne!(hover[(x, 0)].bg, rest[(x, 0)].bg, "× shows its own hover");
        let pressed = draw(
            &mut b,
            &t,
            Interaction {
                hover: Some(x_id),
                pressed: Some(x_id),
                ..Default::default()
            },
        );
        assert_ne!(pressed[(x, 0)].bg, hover[(x, 0)].bg, "× shows a press");
        assert_eq!(
            pressed[(x, 0)].bg,
            t.ladder(Plane::Raised).pressed,
            "× presses on its chip's plane"
        );
    }

    #[test]
    fn the_remove_plate_lights_as_wide_as_it_hits() {
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            let mut b = bar();
            b.chips[0].removable = true;
            let x_id = b.close_id(0);
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut rest = Buffer::empty(Rect::new(0, 0, 40, 1));
            {
                let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
                b.render(Rect::new(0, 0, 40, 1), &mut rest, &mut ctx, t.canvas);
            }
            let plate = hits.area_of(x_id).expect("the × registers");
            assert_eq!(plate.width, 3, "{level:?}");
            assert_eq!(rest[(plate.x + 1, 0)].symbol(), t.glyphs.close);
            let look = |buf: &Buffer, x: u16| {
                let c = &buf[(x, 0)];
                ((level != ColorLevel::Mono).then_some(c.bg), c.modifier)
            };
            for (name, inter) in [
                (
                    "hover",
                    Interaction {
                        hover: Some(x_id),
                        ..Default::default()
                    },
                ),
                (
                    "pressed",
                    Interaction {
                        hover: Some(x_id),
                        pressed: Some(x_id),
                        ..Default::default()
                    },
                ),
            ] {
                let lit = draw(&mut b, &t, inter);
                for x in plate.left()..plate.right() {
                    assert_ne!(look(&lit, x), look(&rest, x), "{name} {x} {level:?}");
                    assert_eq!(look(&lit, x), look(&lit, plate.x), "{name} {x} {level:?}");
                }
                assert_eq!(look(&lit, plate.x - 1), look(&rest, plate.x - 1));
            }
        }
    }

    #[test]
    fn an_off_chip_label_is_readable_and_distinct_from_an_on_chip_at_every_level() {
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            let mut b = bar();
            let buf = draw(&mut b, &t, Interaction::default());
            let on = &buf[(1, 0)];
            let x = (0..40).find(|&x| buf[(x, 0)].symbol() == "b").unwrap();
            let off = &buf[(x, 0)];
            assert_ne!(off.fg, off.bg, "{level:?}: off label on its plane");
            assert_ne!(
                (on.fg, on.modifier),
                (off.fg, off.modifier),
                "{level:?}: on and off chips look alike"
            );
            if level == ColorLevel::Mono {
                assert_ne!(on.modifier, off.modifier, "mono: on and off differ");
            }
        }
    }
}

#[cfg(test)]
mod overflow_tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    #[test]
    fn a_run_that_does_not_fit_keeps_room_for_its_count() {
        assert_eq!(overflow_run(&[5, 5, 5], 17, 1), (3, None));

        assert_eq!(overflow_run(&[5, 5, 5], 16, 1), (2, Some("+1".into())));

        assert_eq!(overflow_run(&[9, 9], 4, 1), (0, Some("+2".into())));
        assert_eq!(overflow_run(&[9, 9], 1, 1), (0, None));
    }

    fn render(bar: &mut ChipBar, w: u16) -> String {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let area = Rect::new(0, 0, w, 1);
        let mut buf = Buffer::empty(area);
        bar.render(area, &mut buf, &mut ctx, theme.canvas);
        (0..w).map(|x| buf[(x, 0)].symbol().to_owned()).collect()
    }

    #[test]
    fn chips_that_do_not_fit_are_counted_with_a_marker() {
        let mut bar = ChipBar::new(WidgetId::of("c"));
        bar.add_label = None;
        bar.chips = ["alpha", "beta", "gamma", "delta"]
            .iter()
            .map(|l| Chip::new(l))
            .collect();
        let line = render(&mut bar, 24);
        assert!(line.contains("alpha"), "{line:?}");
        assert!(line.contains("+3") || line.contains("+2"), "{line:?}");
        assert!(!line.contains('…'), "{line:?}");
        let mut off = bar.clone().overflow_marker(false);
        let line = render(&mut off, 24);
        assert!(!line.contains('+'), "{line:?}");
    }

    fn render_hits(bar: &mut ChipBar, w: u16) -> (Buffer, HitRegistry) {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, w, 12));
        bar.render(Rect::new(0, 0, w, 1), &mut buf, &mut ctx, theme.canvas);
        ctx.flush_overlays(&mut buf);
        drop(ctx);
        (buf, hits)
    }

    fn rows(buf: &Buffer) -> String {
        (0..buf.area.height)
            .map(|y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, y)].symbol().to_owned())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn the_plus_n_marker_opens_a_menu_of_the_hidden_chips() {
        let mut bar = ChipBar::new(WidgetId::of("c"));
        bar.add_label = None;
        bar.chips = ["alpha", "beta", "gamma", "delta"]
            .iter()
            .map(|l| Chip::new(l))
            .collect();
        let (_, hits) = render_hits(&mut bar, 24);
        let more = hits
            .area_of(bar.more_id())
            .expect("the +N marker is a target");

        let (buf, hits) = render_hits(&mut bar, 24);
        let plate: String = (more.left()..more.right())
            .map(|x| buf[(x, 0)].symbol().to_owned())
            .collect();
        assert!(plate.starts_with(" +") && plate.ends_with(' '), "{plate:?}");
        assert_eq!(hits.part_of(bar.more_id()).map(|h| h.owner), Some(bar.id));
        assert_eq!(bar.on_click(bar.more_id()).1, None);
        assert!(bar.menu_open());
        let (buf, hits) = render_hits(&mut bar, 24);
        let text = rows(&buf);
        assert!(text.contains("delta"), "the last chip is listed:\n{text}");
        let hidden = bar.chips.len() - bar.shown;
        let last = bar.menu.as_ref().unwrap().0.row_id(hidden - 1);
        assert!(hits.area_of(last).is_some());
        assert_eq!(hits.part_of(last).map(|h| h.owner), Some(bar.menu_id()));
        assert_eq!(bar.on_click(last).1, Some(ChipAction::Activate(3)));
        assert_eq!(bar.cursor, 3);
        assert!(!bar.menu_open());

        bar.on_click(bar.more_id());
        let esc = Key {
            code: KeyCode::Esc,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        assert_eq!(bar.on_key(&esc).1, None);
        assert!(!bar.menu_open());
        bar.on_click(bar.more_id());
        let (_, hits) = render_hits(&mut bar, 24);
        let outside = hits
            .hit(ratatui::layout::Position::new(more.x, 11))
            .expect("the menu's backdrop");
        assert_eq!(hits.part_of(outside).map(|h| h.owner), Some(bar.menu_id()));
        bar.on_click(outside);
        assert!(!bar.menu_open());

        let mut plain = bar.clone().overflow_menu(false);
        let (_, hits) = render_hits(&mut plain, 24);
        assert!(hits.area_of(plain.more_id()).is_none());
    }

    #[test]
    fn down_opens_the_menu_of_the_hidden_chips() {
        let key = |code| Key {
            code,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        let mut bar = ChipBar::new(WidgetId::of("c"));
        bar.add_label = None;
        bar.chips = vec![Chip::new("alpha")];
        render_hits(&mut bar, 24);
        assert_eq!(bar.on_key(&key(KeyCode::Down)).1, None);
        assert!(!bar.menu_open(), "nothing hidden: no menu");
        bar.chips = ["alpha", "beta", "gamma", "delta"]
            .iter()
            .map(|l| Chip::new(l))
            .collect();
        render_hits(&mut bar, 24);
        assert_eq!(bar.on_key(&key(KeyCode::Down)).0, Outcome::Changed);
        assert!(bar.menu_open());
        assert_eq!(
            crate::core::keymap::label_for(&bar.keymap, ChipCmd::ShowHidden).as_deref(),
            Some("↓")
        );
    }

    #[test]
    fn the_focused_bar_under_its_open_menu_shows_no_focus() {
        let mut bar = ChipBar::new(WidgetId::of("c"));
        bar.add_label = None;
        bar.chips = ["alpha", "beta", "gamma", "delta"]
            .iter()
            .map(|l| Chip::new(l))
            .collect();
        let theme = Theme::junie();
        let draw = |bar: &mut ChipBar| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let inter = Interaction {
                focus: Some(bar.id),
                ..Default::default()
            };
            let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 24, 12));
            bar.render(Rect::new(0, 0, 24, 1), &mut buf, &mut ctx, theme.canvas);
            ctx.flush_overlays(&mut buf);
            drop(ctx);
            rows(&buf)
        };
        assert_eq!(draw(&mut bar).matches('▎').count(), 1);
        bar.on_click(bar.more_id());
        assert!(bar.menu_open());
        let text = draw(&mut bar);
        assert!(!text.lines().next().unwrap().contains('▎'), "{text}");
        assert_eq!(text.matches('▎').count(), 0, "{text}");
    }

    #[test]
    fn the_close_target_is_three_cells_wide_by_default() {
        let mut bar = ChipBar::new(WidgetId::of("c"));
        bar.chips = vec![Chip::new("alpha"), Chip::new("beta")];
        let (buf, hits) = render_hits(&mut bar, 40);
        let x = (0..40).find(|&x| buf[(x, 0)].symbol() == "×").unwrap();
        let at = |x| hits.hit(ratatui::layout::Position::new(x, 0));
        assert_eq!(at(x), Some(bar.close_id(0)));
        assert_eq!(at(x - 1), Some(bar.close_id(0)));
        assert_eq!(at(x + 1), Some(bar.close_id(0)));
        assert_eq!(at(x - 2), Some(bar.chip_id(0)));
        let mut one = bar.clone().close_target_w(1);
        let (_, hits) = render_hits(&mut one, 40);
        assert_eq!(
            hits.hit(ratatui::layout::Position::new(x - 1, 0)),
            Some(one.chip_id(0))
        );
    }
}
