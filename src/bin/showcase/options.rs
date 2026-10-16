//! Live options (plan §7, SCP3): one declaration per knob, read by the
//! Inspector's Options tab, a page's Options section and `--opt`.
//!
//! An option is a key (`shell.<name>` for the shell-wide ones,
//! `<page slug>.<name>` for a page's own), a label and a closed list of
//! values. Setting one rebuilds the configuration through the same builder
//! calls an application makes (`Theme::for_level_with`, `with_glyphs`,
//! `PointerConfig::focus_on`, ...); see `App::apply_shell`.
//!
//! The controls are library widgets: an `off | on` option is a `Toggle`,
//! any other a `Select` ([`OptionsView`]). Values live only for the
//! session; nothing is written to disk.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::app::PageId;
use crate::pages::layout::Sections;
use crate::pages::{Hint, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layout::StackChild;
use junie_tui::widgets::choice::Toggle;
use junie_tui::widgets::select::{Select, SelectAction};

/// What setting an option changes, so the coverage gate knows what to
/// compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// The rendered frame (colours, glyphs, strings, a widget's look).
    Frame,
    /// Behaviour only (pointer timing, key presets, motion): the frame is
    /// the same until the user acts, the live configuration is not.
    Behaviour,
}

/// One live option.
#[derive(Debug, Clone, Copy)]
pub struct Opt {
    /// `shell.<name>` or `<page slug>.<name>`.
    pub key: &'static str,
    pub label: &'static str,
    /// The values `--opt` accepts and the control offers, in order.
    pub values: &'static [&'static str],
    /// Index into [`Self::values`] of the default.
    pub default: usize,
    pub effect: Effect,
    /// Options the effect needs first (`(key, value)`): level overrides
    /// only show at the level they override.
    pub needs: &'static [(&'static str, &'static str)],
}

impl Opt {
    const fn new(key: &'static str, label: &'static str, values: &'static [&'static str]) -> Self {
        Self {
            key,
            label,
            values,
            default: 0,
            effect: Effect::Frame,
            needs: &[],
        }
    }
    const fn default(mut self, i: usize) -> Self {
        self.default = i;
        self
    }
    const fn behaviour(mut self) -> Self {
        self.effect = Effect::Behaviour;
        self
    }
    const fn needs(mut self, needs: &'static [(&'static str, &'static str)]) -> Self {
        self.needs = needs;
        self
    }
    /// An `off | on` option, shown as a `Toggle`.
    pub fn is_toggle(&self) -> bool {
        self.values == OFF_ON
    }
    /// Index of `value` (any case).
    pub fn index_of(&self, value: &str) -> Option<usize> {
        self.values
            .iter()
            .position(|v| v.eq_ignore_ascii_case(value.trim()))
    }
}

const OFF_ON: &[&str] = &["off", "on"];

/// The shell-wide options, in Options-tab order.
pub static SHELL: &[Opt] = &[
    // appearance
    Opt::new(
        "shell.color",
        "Colour level",
        &["truecolor", "256", "16", "mono"],
    ),
    Opt::new(
        "shell.overrides",
        "Level overrides",
        &["none", "ascii-mono"],
    )
    .needs(&[("shell.color", "mono")]),
    Opt::new("shell.glyphs", "Glyphs", &["unicode", "ascii"]),
    // the copy shows where a widget says something (a picker's placeholder, a
    // "… N more" cut, the help footer), not on a page at rest
    Opt::new("shell.strings", "Strings", &["default", "ascii"]).behaviour(),
    Opt::new("shell.motion", "Motion", &["full", "reduced", "paused"]).behaviour(),
    // pointer
    Opt::new("shell.focus_on", "Focus on", &["press", "release"])
        .default(1)
        .behaviour(),
    Opt::new(
        "shell.double_click_ms",
        "Double-click ms",
        &["250", "500", "800"],
    )
    .default(1)
    .behaviour(),
    Opt::new(
        "shell.double_click_slop",
        "Double-click slop",
        &["0", "1", "2"],
    )
    .default(1)
    .behaviour(),
    Opt::new("shell.wheel_lines", "Wheel lines", &["1", "3", "5"])
        .default(1)
        .behaviour(),
    Opt::new(
        "shell.hover_refresh_after_scroll",
        "Re-hover on scroll",
        OFF_ON,
    )
    .default(1)
    .behaviour(),
    Opt::new("shell.flash_ms", "Press flash ms", &["70", "140", "280"])
        .default(1)
        .behaviour(),
    Opt::new(
        "shell.secondary_outside",
        "Right-click outside",
        &["reopen", "close", "ignore"],
    )
    .behaviour(),
    Opt::new("shell.end_edit_on_dead_space", "End edit on blank", OFF_ON)
        .default(1)
        .behaviour(),
    Opt::new("shell.autoscroll", "Edge autoscroll", OFF_ON)
        .default(1)
        .behaviour(),
    Opt::new("shell.autoscroll_ms", "Autoscroll ms", &["25", "50", "100"])
        .default(1)
        .behaviour(),
    Opt::new("shell.track_repeat", "Track repeat", OFF_ON)
        .default(1)
        .behaviour(),
    Opt::new(
        "shell.track_repeat_ms",
        "Track repeat ms",
        &["150/25", "300/50", "500/100"],
    )
    .default(1)
    .behaviour(),
    Opt::new("shell.track_click", "Track click", &["page", "jump"]).behaviour(),
    Opt::new("shell.pointer_shapes", "Pointer shapes", OFF_ON)
        .default(1)
        .behaviour(),
    Opt::new(
        "shell.caret_shape",
        "Caret shape",
        &["bar", "block", "underline", "terminal"],
    )
    .behaviour(),
    Opt::new("shell.click_skips_animation", "Click skips intro", OFF_ON)
        .default(1)
        .behaviour(),
    Opt::new("shell.tooltips", "Tooltips", OFF_ON)
        .default(1)
        .behaviour(),
    Opt::new("shell.reveal", "Reveal cut text", OFF_ON)
        .default(1)
        .behaviour(),
    // keys and editing
    Opt::new(
        "shell.platform",
        "Client platform",
        &["auto", "macos", "other"],
    )
    .behaviour(),
    Opt::new(
        "shell.field_keys",
        "Field keys",
        &["auto", "emacs", "desktop"],
    )
    .behaviour(),
    Opt::new(
        "shell.edit_entry",
        "Edit entry",
        &["immediate", "type-to-edit", "explicit"],
    )
    .behaviour(),
];

/// A page's own option table. Each page step (SC3 onward) adds its page's
/// arm with keys `<slug>.<name>`, and the page implements
/// `Page::option_value` and `Page::set_option` for them.
pub fn page_options(page: PageId) -> &'static [Opt] {
    let _ = page;
    &[]
}

/// Every option `--opt` accepts: the shell-wide ones, then each page's.
pub fn all() -> impl Iterator<Item = &'static Opt> {
    SHELL.iter().chain(
        crate::app::NAV_ENTRIES
            .iter()
            .flat_map(|e| page_options(e.id).iter()),
    )
}

/// The option named `key`.
pub fn find(key: &str) -> Option<&'static Opt> {
    let key = key.trim();
    all().find(|o| o.key.eq_ignore_ascii_case(key))
}

/// The page that owns a page option's key (`None` for `shell.*`).
pub fn page_of(key: &str) -> Option<PageId> {
    let (scope, _) = key.split_once('.')?;
    if scope == "shell" {
        return None;
    }
    PageId::from_name(scope)
}

/// One validated `--opt` assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptSet {
    pub key: &'static str,
    /// Index into the option's values.
    pub value: usize,
}

/// Parse `KEY=VALUE` (`--opt`). An unknown key lists the valid keys; an
/// invalid value lists the option's values.
pub fn parse_opt(arg: &str) -> Result<OptSet, String> {
    let Some((key, value)) = arg.split_once('=') else {
        return Err(format!(
            "expected KEY=VALUE, got {arg:?}; valid keys: {}",
            keys()
        ));
    };
    let Some(opt) = find(key) else {
        return Err(format!(
            "unknown option {:?}; valid keys: {}",
            key.trim(),
            keys()
        ));
    };
    let Some(i) = opt.index_of(value) else {
        return Err(format!(
            "invalid value {:?} for {}; valid values: {}",
            value.trim(),
            opt.key,
            opt.values.join(", ")
        ));
    };
    Ok(OptSet {
        key: opt.key,
        value: i,
    })
}

fn keys() -> String {
    all().map(|o| o.key).collect::<Vec<_>>().join(", ")
}

/// The shell-wide option values of a session (indices into each
/// option's values, in [`SHELL`] order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellOptions {
    values: Vec<usize>,
}

impl Default for ShellOptions {
    fn default() -> Self {
        Self {
            values: SHELL.iter().map(|o| o.default).collect(),
        }
    }
}

impl ShellOptions {
    fn index(key: &str) -> usize {
        SHELL
            .iter()
            .position(|o| o.key == key)
            .unwrap_or_else(|| panic!("no shell option {key}"))
    }
    /// The value index of `key`.
    pub fn get(&self, key: &str) -> usize {
        self.values[Self::index(key)]
    }
    /// The value of `key`, as `--opt` spells it.
    pub fn value(&self, key: &str) -> &'static str {
        let i = Self::index(key);
        SHELL[i].values[self.values[i]]
    }
    /// Set `key` to value index `value` (clamped to its values).
    pub fn set(&mut self, key: &str, value: usize) {
        let i = Self::index(key);
        self.values[i] = value.min(SHELL[i].values.len() - 1);
    }
    pub fn on(&self, key: &str) -> bool {
        self.value(key) == "on"
    }
}

// ---------------------------------------------------------------- controls

enum Control {
    Toggle(Toggle),
    Select(Box<Select>),
}

impl Control {
    fn id(&self) -> WidgetId {
        match self {
            Self::Toggle(t) => t.id,
            Self::Select(s) => s.id,
        }
    }
    fn height(&self) -> u16 {
        match self {
            Self::Toggle(_) => 1,
            // label row and field row
            Self::Select(_) => 2,
        }
    }
    fn owns(&self, id: WidgetId) -> bool {
        match self {
            Self::Toggle(t) => t.id == id,
            Self::Select(s) => s.owns(id),
        }
    }
}

/// One row of an [`OptionsView`]: a group heading or an option's control.
enum Row {
    Heading(&'static str),
    /// A group with no options yet, and why.
    Note(&'static str),
    Option(&'static Opt, Control),
}

/// The controls for a list of option groups, stacked and scrolling by
/// row: the Inspector's Options tab, and a page's Options section.
///
/// The view holds no values: [`Self::sync`] reads them from their owner
/// before each frame, and an event that changes one returns it as an
/// [`OptSet`] for the owner to apply.
pub struct OptionsView {
    rows: Vec<Row>,
    sections: Sections,
}

/// A group of the view: its heading, its options, and what to show when it
/// has none.
pub struct Group {
    pub heading: &'static str,
    pub opts: &'static [Opt],
    pub empty: &'static str,
}

impl OptionsView {
    pub fn new(id: WidgetId, groups: &[Group]) -> Self {
        let mut rows = Vec::new();
        for g in groups {
            rows.push(Row::Heading(g.heading));
            if g.opts.is_empty() {
                rows.push(Row::Note(g.empty));
            }
            for opt in g.opts {
                let cid = id.sub(opt.key);
                let control = if opt.is_toggle() {
                    Control::Toggle(Toggle::new(cid, opt.label, opt.default == 1))
                } else {
                    Control::Select(Box::new(Select::new(
                        cid,
                        opt.label,
                        opt.values,
                        opt.default,
                    )))
                };
                rows.push(Row::Option(opt, control));
            }
        }
        let mut sections = Sections::new(id.sub("rows"));
        sections.stack = sections.stack.gap(0);
        Self { rows, sections }
    }

    fn controls(&self) -> impl Iterator<Item = (&'static Opt, &Control)> {
        self.rows.iter().filter_map(|r| match r {
            Row::Option(o, c) => Some((*o, c)),
            _ => None,
        })
    }

    fn controls_mut(&mut self) -> impl Iterator<Item = (&'static Opt, &mut Control)> {
        self.rows.iter_mut().filter_map(|r| match r {
            Row::Option(o, c) => Some((*o, c)),
            _ => None,
        })
    }

    /// The control id of option `key`.
    #[allow(dead_code)] // tests reach controls by key
    pub fn control_id(&self, key: &str) -> Option<WidgetId> {
        self.controls()
            .find(|(o, _)| o.key == key)
            .map(|(_, c)| c.id())
    }

    /// Load the current value of every option from its owner. An open
    /// select keeps its list.
    pub fn sync(&mut self, value: impl Fn(&Opt) -> usize) {
        for (opt, c) in self.controls_mut() {
            let v = value(opt);
            match c {
                Control::Toggle(t) => t.on = v == 1,
                Control::Select(s) if !s.open => {
                    s.selected = v;
                    s.cursor = v;
                }
                Control::Select(_) => {}
            }
        }
    }

    /// Whether `id` is one of the view's targets (a control, a select's
    /// option or backdrop, the scroll body or its bar).
    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.sections.id
            || id == junie_tui::widgets::scrollbar::id_for(self.sections.id)
            || self.controls().any(|(_, c)| c.owns(id))
    }

    /// Whether `id` is one of the view's option controls.
    pub fn has_control(&self, id: WidgetId) -> bool {
        self.controls().any(|(_, c)| c.id() == id)
    }

    /// Whether an option's select has its list open.
    pub fn popup_open(&self) -> bool {
        self.controls()
            .any(|(_, c)| matches!(c, Control::Select(s) if s.open))
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let t = ctx.theme;
        let children: Vec<StackChild> = self
            .rows
            .iter()
            .enumerate()
            .map(|(i, r)| match r {
                // a blank row above every group but the first
                Row::Heading(_) => StackChild::fixed(1 + u16::from(i > 0)),
                Row::Note(_) => StackChild::fixed(1),
                Row::Option(_, c) => StackChild::fixed(c.height()),
            })
            .collect();
        let focus = self.rows.iter().position(|r| match r {
            Row::Option(_, c) => ctx.interaction.has_focus(c.id()),
            _ => false,
        });
        let l = self.sections.layout(area, &children, focus, buf, ctx);
        for (i, row) in self.rows.iter_mut().enumerate() {
            let slot = l.slot(i);
            match (row, slot) {
                (Row::Heading(h), Some(r)) => {
                    let y = r.bottom() - 1;
                    buf.set_string(r.x, y, *h, t.title().bg(bg));
                }
                (Row::Note(n), Some(r)) => {
                    let text = junie_tui::ui::text::truncate(n, r.width as usize);
                    buf.set_string(r.x, r.y, text, t.faint().bg(bg));
                }
                (Row::Option(_, Control::Toggle(tg)), Some(r)) => tg.render(r, buf, ctx, bg),
                (Row::Option(_, Control::Select(s)), Some(r)) => s.render(r, buf, ctx, bg),
                // scrolled out: still a Tab stop, so focus scrolls it in
                (Row::Option(_, c), None) => ctx.focus_stop(c.id()),
                _ => {}
            }
        }
    }

    /// A focus change: an open list closes when its select loses focus.
    pub fn on_blur(&mut self, from: Option<WidgetId>) -> Outcome {
        let mut out = Outcome::Ignored;
        for (_, c) in self.controls_mut() {
            if let Control::Select(s) = c
                && Some(s.id) == from
            {
                out = out.or(s.on_blur());
            }
        }
        out
    }

    /// Route `ev` to the control it targets. Returns the outcome and the
    /// option it changed, if any; `None` when the view does not own it.
    pub fn handle(
        &mut self,
        ev: &PageEvent,
        cx: &mut PageCtx,
    ) -> Option<(Outcome, Option<OptSet>)> {
        if let Some(o) = self.sections.handle(ev) {
            return Some((o, None));
        }
        let changed = |opt: &'static Opt, i: usize| {
            Some(OptSet {
                key: opt.key,
                value: i,
            })
        };
        let focus = cx.focus.current();
        for (opt, c) in self.controls_mut() {
            match (c, ev) {
                (Control::Toggle(t), PageEvent::Key(key)) if focus == Some(t.id) => {
                    let o = t.on_key(key);
                    let set = (o == Outcome::Changed).then(|| changed(opt, usize::from(t.on)));
                    return Some((o, set.flatten()));
                }
                (Control::Toggle(t), PageEvent::Click { id, .. }) if *id == t.id => {
                    cx.focus.focus(t.id);
                    let o = t.on_click();
                    let set = (o == Outcome::Changed).then(|| changed(opt, usize::from(t.on)));
                    return Some((o, set.flatten()));
                }
                (Control::Select(s), PageEvent::Key(key)) if focus == Some(s.id) => {
                    let (o, a) = s.on_key(key);
                    let set = a.and_then(|SelectAction::Changed(i)| changed(opt, i));
                    return Some((o, set));
                }
                (Control::Select(s), PageEvent::Click { id, .. }) if s.owns(*id) => {
                    if *id == s.id {
                        cx.focus.focus(s.id);
                    }
                    let (o, a) = s.on_click(*id);
                    let set = a.and_then(|SelectAction::Changed(i)| changed(opt, i));
                    return Some((o, set));
                }
                (Control::Select(s), PageEvent::Press { id, pos }) if s.owns(*id) => {
                    // a press opens the list, so it takes focus with it
                    if *id == s.id {
                        cx.focus.focus(s.id);
                    }
                    return Some((s.on_press(*id, *pos), None));
                }
                (Control::Select(s), PageEvent::Drag { pressed, pos }) if s.owns(*pressed) => {
                    return Some((s.on_drag(*pressed, *pos), None));
                }
                (Control::Select(s), PageEvent::Release { pressed, pos }) if s.owns(*pressed) => {
                    let (o, a) = s.on_release(*pos);
                    let set = a.and_then(|SelectAction::Changed(i)| changed(opt, i));
                    return Some((o, set));
                }
                (Control::Select(s), PageEvent::Wheel { id, delta }) if s.open && s.owns(*id) => {
                    return Some((s.on_wheel(*delta), None));
                }
                _ => {}
            }
        }
        None
    }

    /// Footer hints for the focused control; `None` when focus is not in
    /// the view.
    pub fn hints(&self, focus: Option<WidgetId>) -> Option<Vec<Hint>> {
        let (_, c) = self.controls().find(|(_, c)| Some(c.id()) == focus)?;
        Some(match c {
            Control::Toggle(_) => vec![("Space", "Toggle")],
            Control::Select(s) if s.open => {
                vec![("↑↓", "Choose"), ("Enter", "Set"), ("Esc", "Close")]
            }
            Control::Select(_) => vec![("Enter", "Open"), ("↑↓", "Choose")],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unique_and_scoped() {
        let mut seen = std::collections::HashSet::new();
        for o in all() {
            assert!(seen.insert(o.key), "duplicate option {}", o.key);
            let (scope, name) = o.key.split_once('.').expect("scoped key");
            assert!(!name.is_empty());
            assert!(
                scope == "shell" || PageId::from_name(scope).is_some(),
                "{}",
                o.key
            );
            assert!(o.default < o.values.len(), "{}", o.key);
            for (k, v) in o.needs {
                assert!(find(k).and_then(|n| n.index_of(v)).is_some(), "{}", o.key);
            }
        }
    }

    #[test]
    fn parse_opt_accepts_and_explains() {
        assert_eq!(
            parse_opt("shell.color=mono"),
            Ok(OptSet {
                key: "shell.color",
                value: 3
            })
        );
        assert_eq!(parse_opt(" Shell.Glyphs = ASCII ").map(|s| s.value), Ok(1));
        let unknown = parse_opt("shell.bogus=1").unwrap_err();
        assert!(
            unknown.contains("unknown option \"shell.bogus\""),
            "{unknown}"
        );
        assert!(
            unknown.contains("shell.color, shell.overrides"),
            "{unknown}"
        );
        let bad = parse_opt("shell.color=purple").unwrap_err();
        assert!(
            bad.contains("valid values: truecolor, 256, 16, mono"),
            "{bad}"
        );
        assert!(parse_opt("shell.color").unwrap_err().contains("KEY=VALUE"));
    }
}
