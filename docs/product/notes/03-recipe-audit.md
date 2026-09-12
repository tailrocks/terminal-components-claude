# 03 · Recipe audit: assembling `holla` exactly like `jackin-preview` / `tablepro`

Audited against the working tree at `src/` (library crate `junie_tui`, edition 2024, rust 1.98 toolchain, `rust-version = "1.88"`).
Every type name, signature and constant below is copied from source; line refs point at the precedent to crib from.

Precedents:
- `src/bin/jackin_preview/` — deterministic sim + scenarios + motion + virtual clock + menu bar + hint bar. **Primary template for holla.**
- `src/bin/tablepro/` — simpler shell (one modal slot, wall-clock `Instant`), `Dialog::facts` with typed token, `keyhint::render` footer.

---

## 0. Cargo wiring

`Cargo.toml` registers each binary explicitly (no autobins reliance):

```toml
[[bin]]
name = "holla"
path = "src/bin/holla/main.rs"
```

Deps already present: `ratatui = "0.30"` (`crossterm_0_29` feature), `unicode-width`, `unicode-segmentation`. No new deps needed. Library is imported as `junie_tui::…` (`[lib] name = "junie_tui"`).

Verify gate (README.md:74-77): `cargo fmt --check` · `cargo clippy --all-targets -- -D warnings` · `cargo test`.

---

## 1. `main.rs` — args, env, runtime, ticks/clock/frames

Copy `src/bin/jackin_preview/main.rs` wholesale, rename. Shape:

```rust
mod app; #[cfg(test)] mod app_tests; mod clock; mod domain; mod scenario; mod screens; mod sim;

use junie_tui::core::event::{Input, Outcome};
use junie_tui::theme::{ColorLevel, Theme};

struct Options { level: ColorLevel, scenario: Scenario, motion: Motion, frame: u64 }

fn parse_args() -> Options {
    let mut level = ColorLevel::detect();          // NO_COLOR → Mono; COLORTERM=truecolor → TrueColor; *256color* → Ansi256; else Ansi16
    let mut scenario = Scenario::FirstUse; let mut motion = None; let mut frame = 0;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() { match a.as_str() {
        "--color" | "-c"    => level = match args.next().as_deref() { Some("truecolor")|Some("24bit") => ColorLevel::TrueColor, Some("256") => ColorLevel::Ansi256, Some("16") => ColorLevel::Ansi16, Some("none")|Some("mono") => ColorLevel::Mono, other => { eprintln!("unknown --color value {other:?}; use truecolor|256|16|none"); std::process::exit(2) } },
        "--scenario" | "-s" => scenario = Scenario::from_name(&args.next().unwrap_or_default()).unwrap_or_else(|| { /* eprintln names from Scenario::ALL; exit(2) */ }),
        "--motion" | "-m"   => motion = Some(Motion::from_name(&args.next().unwrap_or_default()).unwrap_or_else(|| { eprintln!("unknown motion …; use full|reduced|paused"); std::process::exit(2) })),
        "--frame" | "-f"    => frame = args.next().and_then(|v| v.parse().ok()).unwrap_or_else(|| { eprintln!("--frame needs a tick number"); std::process::exit(2) }),
        "-h" | "--help"     => { println!("holla — … USAGE: holla [--scenario NAME] [--motion full|reduced|paused] [--frame N] [--color …]"); std::process::exit(0) }
        _ => {} } }
    let no_motion = std::env::var_os("HOLLA_NO_MOTION").is_some_and(|v| !v.is_empty() && v != "0");   // jackin: JACKIN_NO_MOTION
    Options { level, scenario, motion: Motion::resolve(motion, no_motion), frame }
}

fn main() -> std::io::Result<()> {
    let opts = parse_args();
    let theme = Theme::for_level(opts.level);
    let mut app = App::for_scenario(opts.scenario, opts.motion, opts.frame, theme);
    let _ = junie_tui::runtime::drain_pending_input();   // stale keys from launching shell must not skip an opening sequence
    junie_tui::runtime::run(&mut app)
}

impl junie_tui::runtime::Application for App {
    fn handle(&mut self, input: Input) -> Outcome { App::handle(self, input) }
    fn render(&mut self, frame: &mut ratatui::Frame) { App::render(self, frame) }
    fn should_quit(&self) -> bool { self.quit }
    fn tick_interval(&self) -> std::time::Duration { App::tick_interval(self) }
}
```

`runtime::run` (src/runtime.rs:107): `TerminalSession::enter()` (raw mode, alt screen, mouse capture, bracketed paste, panic hook restores) → `event_loop` → `session.leave()`. Loop: draw when dirty; `event::poll(wait)` where `wait = tick_interval - elapsed`; coalesces queued events; `Input::from_crossterm` (Key Press|Repeat only; Left/Right mouse; wheel; Resize; Paste); every `Outcome::Changed` marks dirty; after `interval` elapsed delivers `Input::Tick`; quit after tick draws one final frame.

### scenario.rs (copy `src/bin/jackin_preview/scenario.rs`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum Scenario { … }
impl Scenario { pub const ALL: [Scenario; N]; pub fn name(self) -> &'static str; pub fn from_name(s: &str) -> Option<Self> { Self::ALL.into_iter().find(|sc| sc.name() == s) } }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] pub enum Motion { #[default] Full, Reduced, Paused }
impl Motion { pub fn from_name(&str) -> Option<Self>; pub fn resolve(cli: Option<Motion>, no_motion_env: bool) -> Motion { cli.unwrap_or(if no_motion_env { Reduced } else { Full }) } }
```
Keep the `names_round_trip` test.

### clock.rs (copy `src/bin/jackin_preview/clock.rs`)
```rust
pub const EPOCH_SECS: i64 = 1_788_401_640;           // fixed fixture epoch (2026-09-03 09:14 UTC+7)
pub struct Clock { pub now_ms: i64, pub running: bool }
impl Clock { pub const fn new() -> Self; pub fn advance(&mut self, interval_ms: i64) { if self.running { self.now_ms += interval_ms } }
             pub fn now_secs(&self) -> i64; pub fn hhmm(secs) -> String; pub fn stamp(secs) -> String; pub fn weekday(secs) -> &'static str; pub fn ago(&self, then_secs) -> String; pub fn reset_label(&self, then_secs) -> String }
pub fn format_duration(secs: u64) -> String   // "38 s", "2 h 14 min", two units max
```
**Never read wall-clock.** Every timestamp/animation derives from `world.clock.now_ms`. (tablepro uses `std::time::Instant` for status/flash — do NOT copy that; it breaks determinism.)

### How ticks / paused / frame produce a deterministic picture (app.rs)
- `App::for_scenario` (jackin app.rs:165): `world = fixtures::world_for(scenario); world.clock.running = motion != Motion::Paused;` then `app.start(frame)`.
- `start(frame)`: route-specific seeking. Intro: `IntroState::new(motion, frame)` positions the ritual at tick `frame`; if `frame >= rain::INTRO_END` skip straight to manager. Cockpit: after `go(Go::Launch{..})`, `cockpit.seek(frame, &mut world, &mut cx)` replays `frame` ticks synchronously. Outro: `OutroState::new(motion, elapsed, frame)`.
- `tick_interval()` (app.rs:315): `if motion == Paused { 500ms } else { route.tick_ms(animating()) }` — jackin uses `rain::TICK_MS` (33 ms) for rituals, 80 ms animating, 200 ms idle.
- `on_tick()` (app.rs:356): `let interval = self.route.tick_ms(true) as i64; let msgs = self.world.tick(interval);` — **the interval passed to the world is the constant per route, not measured time**, so N ticks always == N×interval virtual ms. `World::tick` returns `vec![]` immediately when `!clock.running` (paused): nothing moves, so `--motion paused --frame N` renders the same frame forever.
- `Interaction.tick = (world.now_ms() / 80) as u64` drives spinners (`progress::spinner_frame(tick)`); flash = `(WidgetId, until_ms)`; status = `(String, Tone, until_ms)` with `now_ms + 5_000`.
- Reduced motion: intro/outro show one static boundary frame + "Enter Continue"; handoff `finish_handoff()` immediately.

---

## 2. App struct shape, Cx, Request, Screen, modal stack, focus, hits, key routing

### Shared contract (`src/bin/jackin_preview/screens/mod.rs`) — copy verbatim, prune variants
```rust
pub struct ModalTag { pub kind: &'static str, pub key: String, pub n: usize }   // ::new(kind).key(k).n(n)
pub enum Modal { Dialog(Dialog), Picker(Picker), Browser(FileBrowser), Choice(ChoiceDialog), Form(FormDialog), Op(OpFlow), Info(InfoDialog), Help(HelpOverlay), Custom(Box<dyn CustomModal>) }   // #[allow(clippy::large_enum_variant)]
pub enum ModalResult { Dialog { action: Option<usize>, text: Option<String> }, Picked(usize), PickedAlt(usize), Scope, Cancelled, Browser(_), Choice(Option<usize>), Form(Option<FormValues>), FormAction(String, FormValues), Op(_), Info(InfoResult), Custom(String) }
pub enum Go { Manager, Settings, … , Quit }          // navigation requests
pub enum Request { Status(String), Error(String), Open(Box<Modal>, ModalTag), Close, Go(Go), Copy(String), Help, WithForm(Box<dyn FnOnce(&mut FormDialog)>) }
pub struct Cx<'a> { pub focus: &'a mut Focus, pub ring: &'a FocusRing, pub requests: Vec<Request> }
impl Cx<'_> { fn focus_next; fn status(s); fn error(s); fn open(modal, tag); fn close(); fn go(g); fn help(); fn copy(s); fn with_form(f) }
pub trait Screen {
    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome;                       // required
    fn on_click(&mut self, id: WidgetId, pos: Position, w: &mut World, cx: &mut Cx) -> Outcome { Ignored }
    fn on_double_click(..) / on_drag(pressed, pos, w) / on_secondary(..) / on_press(..) / on_release(..) / on_wheel(id, delta: i32, pos, w) / on_paste(text, w) / on_tick(w, cx) / on_msg(&Msg, w, cx) / on_modal(&ModalTag, ModalResult, w, cx) -> Outcome  // all default Ignored
    fn picker_items(&mut self, tag: &ModalTag, query: &str, w: &World) -> Option<Vec<PickerItem>> { None }
    fn form_changed(&mut self, tag, form: &mut FormDialog, w) {}
    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World);          // required
    fn hints(&self, focus: Option<WidgetId>, w: &World) -> Vec<Hint>;                            // required
    fn crumb(&self, w: &World) -> String;                                                        // required ("Workspaces › api-server")
    fn strip_right(&self, w: &World) -> Vec<Segment> { vec![] }
    fn is_editing(&self) -> bool { false }  fn animating(&self, w: &World) -> bool { false }
    fn enter(&mut self, w: &mut World, cx: &mut Cx) {}
    fn primary_focus(&self) -> Option<WidgetId>;                                                 // required
    fn on_esc_top(&mut self, w: &mut World, cx: &mut Cx) -> Outcome { cx.go(Go::Manager); Changed }
}
pub trait CustomModal { on_key(key, focus, ring, w) ; on_click(id,pos,focus,w); on_wheel(delta,pos); on_drag; on_tick(w); render(screen, buf, ctx, w); done() -> Option<ModalResult>; initial_focus(); hints(); cancel_on_outside_click() -> bool {true} }
pub fn plural(n: usize, one: &str, many: &str) -> String
```
tablepro's leaner variant (app.rs:51-84): `enum Request { OpenDialog(Box<Dialog>), Status(String), … }`, `Cx { focus, ring, requests }` with `open(d: Dialog)`.

### App struct (jackin app.rs:132-161)
```rust
pub struct App {
    pub theme: Theme, pub scenario: Scenario, pub motion: Motion, pub world: World,
    pub route: Route,                       // #[derive(Clone, Copy, PartialEq, Eq)] enum Route { Intro, Manager, …, Outro }
    pub screens: Screens,                   // struct of concrete screens; `fn get_mut(&mut self, Route) -> Option<&mut dyn Screen>` + `get()`
    modals: Vec<ModalEntry>,                // struct ModalEntry { modal: Modal, tag: ModalTag, owner: Route, saved_focus: Option<WidgetId> }
    pub focus: Focus, pub ring: FocusRing, pub hits: HitRegistry,
    pub hover: Option<WidgetId>, pub pressed: Option<WidgetId>, hover_suppressed: bool,
    flash: Option<(WidgetId, i64)>, pub status: Option<(String, Tone, i64)>,
    pub size: (u16, u16), pub quit: bool, too_small: bool,
    host_menu: MenuBar, host_menu_route: Route, last_click: Option<(WidgetId, i64)>,
    intro_guard: u8, pub clipboard_gen: u32, exit_after_status: Option<i64>,
}
```
tablepro: `pub modal: Option<Modal>` (single slot) + `saved_focus: Option<WidgetId>`; `#[allow(clippy::large_enum_variant)] pub enum Modal { Dialog(Dialog), Picker(PickerKind, Picker, Option<SwitcherIndex>), Filter(FilterEditor) }`.

### Per-frame render (jackin app.rs:2250-2278; identical in tablepro 2090-2116)
```rust
pub fn render(&mut self, frame: &mut Frame) {
    let area = frame.area(); self.size = (area.width, area.height);
    let theme = self.theme;
    let mut hits = HitRegistry::default(); let mut ring = FocusRing::default();   // REBUILT EVERY FRAME
    let interaction = self.interaction();
    let cursor;
    { let buf = frame.buffer_mut();
      let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
      self.draw(area, buf, &mut ctx); cursor = ctx.cursor; }
    self.hits = hits; self.ring = ring;
    if self.modals.is_empty() {
        if !self.too_small && !self.focus.current().is_some_and(|c| self.ring.contains(c)) {
            let pf = self.screens.get(self.route).and_then(|s| s.primary_focus());
            self.focus.set(pf.filter(|p| self.ring.contains(*p)).or(self.ring.first()));   // initial focus = screen.primary_focus() else first in ring
        }
    } else { self.focus.ensure_valid(&self.ring); }
    if let Some(pos) = cursor { frame.set_cursor_position(pos); }
}
fn interaction(&self) -> Interaction {
    let flash = match self.flash { Some((id, until)) if self.world.now_ms() < until => Some(id), _ => None };
    Interaction { focus: self.focus.current(), hover: self.hover, pressed: self.pressed, flash, focus_hidden: false, hover_suppressed: self.hover_suppressed, tick: (self.world.now_ms() / 80) as u64 }
}
```
Pitfall: focus ring order == render order (Tab order == reading order). Widgets register via `ctx.control(id, area, disabled)` / `ctx.clickable(id, area)` / `ctx.scrollable(id, area)`; `ctx.inert = true` disables all registration (used for handoff frames). `ctx.begin_modal()` pushes a barrier on both hits and ring so only the modal's controls are reachable.

### Input routing (jackin app.rs:340-683)
`handle(Input)`: Resize → store size, Changed · Tick → `on_tick` · Paste → topmost modal else screen · Key → `hover_suppressed = true; on_key` · Mouse → `on_mouse`.

`on_key` ladder (in order):
1. `too_small` → `q`/Ctrl+C quits; everything else Consumed.
2. Ritual routes (Intro/Outro/Handoff): Enter/Esc skip; Ctrl+C quits; else Consumed. `intro_guard` (3 ticks) swallows keys right after start unless Paused.
3. Ctrl+C with no modal (outside Cockpit/Capsule) → quit.
4. `if !modals.is_empty() { return self.modal_key(key) }` — **modal wins over everything**.
5. `editing = screen.is_editing()`; `host = matches!(route, host routes)`.
6. Open menu bar: `host_menu.on_key(&key)` → `MenuBarEvent::Chosen(mi, ii)` → `run_host_menu(&label)`; `Brand` → about.
7. `F10` (host, !editing) → `sync_host_menu(); host_menu.open_menu(0)`.
8. Global chords (host, !editing): `?` help, `u`/`c`/`s` nav, Ctrl+Q quit-confirm.
9. Screen: `screen.on_key(&key, &mut world, &mut cx)`, then `apply_requests(cx.requests, route)`. If consumed: set `flash` on Enter/Space for focused id (140 ms), return.
10. Fallback: `Tab` → `focus.next(&ring)`, `BackTab` → `focus.prev(&ring)`, `q` (host, !editing) → Manager: `Go::Quit` else `Go::Manager`, `Esc` → `screen.on_esc_top(...)`.

Cx construction idiom (repeat everywhere):
```rust
let mut cx = Cx { focus: &mut self.focus, ring: &self.ring, requests: vec![] };
let o = match self.screens.get_mut(route) { Some(s) => s.on_key(&key, &mut self.world, &mut cx), None => Outcome::Ignored };
let reqs = std::mem::take(&mut cx.requests);
o.or(self.apply_requests(reqs, route))
```

Esc ladder inside screens (tablepro `esc_ladder` app.rs:779): unmaximize → tabstrip→explorer → clear filter → Consumed. jackin: each screen's `on_key` handles inner Esc levels, `on_esc_top` decides leaving (default `cx.go(Go::Manager)`).

### Modal stack (jackin app.rs:1229-1281)
```rust
fn push_modal(&mut self, modal: Modal, tag: ModalTag, owner: Route) {
    let initial = match &modal { Modal::Dialog(d) => Some(d.initial_focus), Modal::Picker(p) => Some(p.id), Modal::Info(i) => Some(i.initial_focus()), Modal::Help(h) => Some(h.id), … };
    self.modals.push(ModalEntry { modal, tag, owner, saved_focus: self.focus.current() });
    self.focus.set(initial); self.hover = None; self.pressed = None;
}
fn pop_modal(&mut self) -> Option<ModalEntry> { let e = self.modals.pop(); if let Some(e) = &e { self.focus.set(e.saved_focus); } e }
fn deliver(&mut self, entry: ModalEntry, result: ModalResult) -> Outcome { /* screens.get_mut(entry.owner).on_modal(&entry.tag, result, world, cx) then apply_requests; .or(Changed) */ }
```
`modal_key` per variant (app.rs:1333): Dialog → `d.on_key(&key, &mut self.focus, &self.ring)`; if `d.result.is_some()` → `action = DialogResult::Action(i) → Some(i)`, `cancel = action.is_none() || action == d.cancel_index`; pop; special-case `tag.kind == "quit"`; else `deliver(entry, ModalResult::Dialog{action, text})`. Picker → `p.on_key(&key)` → `QueryChanged` → `refresh_picker()` (asks owner `picker_items(tag, query, world)` → `p.set_items(items)`); `Chosen(i)` → pop + `deliver(Picked(i))`; `NextScope` → owner `on_modal(tag, ModalResult::Scope)` then refresh; `Cancelled` → pop + deliver Cancelled.
`apply_requests` (app.rs:1951): Status/Error → `set_status(&s, tone)`; Open(m, tag) → `push_modal(*m, tag, owner)`; Close → pop; Go(g) → `self.go(g)`; Help → `open_help()`; Copy → clipboard.

### Mouse (jackin app.rs:1534-1753; tablepro 1858-2049)
- `Move`: `hover = hits.hit(pos)`; `hover_suppressed = false`; Changed iff hover changed or was suppressed.
- `Down`: `hit = hits.hit(pos); pressed = hit; hover = hit;` if no modal and `ring.contains(id)` → `focus.focus(id)`; `screen.on_press`.
- `Up`: `pressed.take()`; no hit + modal + pressed.is_none() → `modal_outside_click()` (Dialog: `d.on_click_outside()` unless `DialogBody::Facts{ack: Some(_)}`; Picker/Op/Info/Help: pop); `pressed != Some(id)` → Changed (no click); else `flash = (id, now+140)`, double-click if same id within 500 ms virtual; modal → `modal_click(id, pos)`; host menu `owns(id)` → `host_menu.on_click(id)`; open menu + click elsewhere → close; strip ids (`STRIP_HELP` etc.); else `screen.on_double_click` / `on_click`.
- `Drag`: `hover = hits.hit(pos)`; needs `pressed`; screen `on_drag(pressed, pos, world)`.
- `Secondary` (right button): screen `on_secondary(id, pos, …)` → context menus.
- Wheel: `delta = ±3`; modal → its `on_wheel(delta)`; else `hits.hit_scroll(pos)` → `screen.on_wheel(id, delta, pos, world)`.

---

## 3. Chrome: menu bar + Lockup, blank row, body, blank row, hint bar; too-small; backdrop

### Frame layout (jackin `draw_frame` app.rs:2380-2415)
```rust
pub const MIN_WIDTH: u16 = 72;  pub const MIN_HEIGHT: u16 = 20;
pub const BRAND_MARK: &str = "jackin❯";          // holla: choose one mark string; EVERY Lockup uses this constant
let header = Rect::new(area.x, area.y, area.width, 1);                           // row 0: menu bar
let footer = Rect::new(area.x, area.bottom() - 1, area.width, 1);                // last row: hint bar
let body   = Rect::new(area.x + 1, area.y + 2, area.width.saturating_sub(2), area.height.saturating_sub(4));   // row 1 blank, 1-col side margins, row h-2 blank
fill(buf, area, t.base());
self.draw_host_menu(header, buf, ctx, route);   // or draw_strip for non-menu routes
screen.render(body, buf, ctx, &self.world);
self.draw_footer(footer, buf, false);
self.host_menu.render_open(area, buf, ctx);     // dropdown LAST so it sits on top of the body
```
Modals are drawn after `draw_frame` (app.rs:2348): pop top entry, `d.render(area, buf, ctx)` / `p.render(area, buf, ctx, &hints)` / …, push back, then `draw_footer(footer, buf, true)` again so the hint row shows the modal's hints.

### Menu bar with Lockup + right-aligned identity/breadcrumb (app.rs:699-743, 838-880)
```rust
const HOST_MENU: WidgetId = WidgetId::of("host.menu");
fn build_host_menu(route: Route) -> MenuBar {
    let file: Vec<MenuItem> = vec![ MenuItem::new("New workspace…").shortcut("n"), MenuItem::new("Delete workspace…").shortcut("d").danger().separator(), MenuItem::new("Quit").shortcut("Ctrl+Q") ];
    let go = vec![ MenuItem::new("Workspace manager").shortcut("Esc"), … ];
    let help = vec![ MenuItem::new("Key reference").shortcut("?"), MenuItem::new("About holla") ];
    MenuBar::new(HOST_MENU, vec![("File", file), ("Go", go), ("Help", help)]).brand(Lockup::new(BRAND_MARK))
}
fn draw_host_menu(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, route: Route) {
    let t = self.theme;
    self.sync_host_menu();                                   // rebuild when route changed (menus are per-route data)
    self.host_menu.on_hover(ctx.interaction.hover);          // hovering another label while open switches menus
    self.host_menu.render(area, buf, ctx, t.canvas);         // draws brand pill + labels; fills row bg
    let used = self.host_menu.areas.iter().map(|r| r.right()).max().unwrap_or(area.x).max(self.host_menu.brand_area.right());
    let rest = Rect::new(used + 2, area.y, area.right().saturating_sub(used + 2), 1);
    let mut right = vec![];
    if let Some(s) = self.screens.get(route) { right.push(Segment::new(s.crumb(&self.world), Tone::Secondary).priority(7)); right.extend(s.strip_right(&self.world)); }
    right.push(Segment::new(state, tone).priority(6));       // e.g. "inside the Construct"
    right.push(Segment::new(format!("{} working", progress::spinner_frame(ctx.interaction.tick)), Tone::Secondary).priority(4));  // only when jobs pending
    segments::render(rest, buf, ctx, &[], &right, t.canvas); // left empty, right list right-aligned; low priority drops first
}
```
Menu events: keyboard `F10` opens menu 0; while open `host_menu.on_key(&key) -> (Outcome, Option<MenuBarEvent>)` handles ←→/hjkl, ↑↓, Enter, Esc. Mouse: `host_menu.owns(id)` → `on_click(id) -> (Outcome, Option<MenuBarEvent>)` — `MenuBarEvent::{Opened(i), Chosen(menu, item), Closed, Brand}`. Resolve chosen label via `self.host_menu.menus[mi][ii].label.clone()` and dispatch by label string (`run_host_menu`), which re-injects the shortcut key via `self.handle(Input::Key(Key{code, mods}))` so menus and keys can never disagree.
Menu-open hint layer: `hint("← →","Menu"), hint("↑↓","Move"), hint("Enter","Choose"), hint("Esc","Close")`.
Brand click/Enter → `MenuBarEvent::Brand` → open About `InfoDialog::new(WidgetId::of("host.about"), "About", props).width(64)` (props via `Prop::new(label, value)`).

Non-menu identity strip alternative (app.rs:2433, tablepro 2189): `let pill_w = Lockup::new(BRAND_MARK).render(area.x + 1, area.y, buf, &t);` then `segments::render(rest, buf, ctx, &left, &right, t.canvas)`; right includes `Segment::new(format!("{} · {}×{}", t.level.label(), size.0, size.1), Tone::Faint).priority(1)` and `Segment::new("? help", Tone::Muted).clickable(STRIP_HELP).priority(4)`.

### Hint bar layers & precedence (app.rs:2518-2642)
```rust
use junie_tui::widgets::hintbar::{HintBar, HintLayer};
use junie_tui::widgets::keyhint::{Hint, hint};
let modal_layer  = modal.then(|| HintLayer::new(hints.clone()));                                   // topmost modal's hints
let menu_layer   = (!modal && is_host && self.host_menu.is_open()).then(|| HintLayer::new(vec![…]));
let screen_layer = (!modal && !hints.is_empty()).then(|| HintLayer::new(hints.clone()));          // screen.hints(focus, world)
let fallback     = HintLayer::new(vec![hint("?", "Help"), hint("Esc", "Back")]);
let mut layer = HintBar::resolve(&[modal_layer, menu_layer, screen_layer, Some(fallback)]);        // first Some wins
layer.badge = editing.then_some(("EDIT", BadgeKind::Edit));
layer.status = status.map(|(s, tone)| (truncate(s, area.width.saturating_sub(4) as usize), tone));  // status ALWAYS wins the right edge
layer.centered = true;
HintBar::render(area, buf, &t, &layer);   // returns #hints that fit; dropped ones leave a faint "…"
```
Per-modal hints (copy): Picker `[Type Filter?] ↑↓ Move · Enter Choose · Esc Clear|Cancel`; Dialog Facts `← → Choose · Enter Confirm · Esc Cancel`; Dialog Text adds `y / n Quick answer`; Info `↑↓ Move · y Copy · Esc Close`; Help `↑↓ Scroll · Esc Close`.
**Rule:** pickers draw no hint row of their own — pass `""` as `hints` to `Picker::render` (jackin `modal_hints` returns `String::new()`). tablepro (older) passes a hint string to the picker and uses `keyhint::render(area, buf, &t, &hints, badge, status)`; prefer the jackin HintBar form.

### Too-small notice (app.rs:2283-2310) — exact text
```rust
self.too_small = area.width < MIN_WIDTH || area.height < MIN_HEIGHT;
let lines = [
    (&*format!(" {BRAND_MARK} "), Lockup::style(&t)),               // tablepro: ("TablePro", t.title())
    ("Terminal too small", t.secondary()),
    (&*format!("Need {MIN_WIDTH}×{MIN_HEIGHT}, have {}×{}", area.width, area.height), t.muted()),
    ("q Quit", t.faint()),
];
let y0 = area.y + area.height.saturating_sub(5) / 2;   // rows y0..y0+2, then blank, "q Quit" at y0+4; centered horizontally by `width(text)`
```
Test asserts `"Terminal too small"` and recovery on resize.

### Backdrop dimming (modals.rs:35-95 `modal_frame`, dialog.rs:357, tablepro 2344-2376)
```rust
let dim = Rect::new(screen.x, screen.y, screen.width, screen.height.saturating_sub(1));   // footer row stays live
for pos in dim.positions() { if let Some(c) = buf.cell_mut(pos) { let st = t.backdrop(c.style()); c.set_style(st); c.modifier = Modifier::empty(); } }
ctx.begin_modal();                                                                        // barrier: page below is unreachable
let area = junie_tui::ui::popup::place(screen, Rect::ZERO, w, h, Placement::Center);
let bg = t.surface_elevated; fill(buf, area, Style::new().bg(bg));
Block::new().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(t.border(true).bg(bg)).render(area, buf);
ctx.hits.register(WidgetId::of("modal.surface"), area);                                  // clicks on the surface don't count as "outside"
```
`Dialog::render` and `Picker::render` do all of this internally; only custom modals need `modal_frame`.

---

## 4. Dialogs

```rust
use junie_tui::widgets::dialog::{Dialog, DialogBody, DialogResult};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::props::Prop;

Dialog::confirm(id, title, text, confirm_label)      // [Cancel(subtle), OK(primary)], cancel_index Some(0), width 54, initial_focus = OK
Dialog::destructive(id, title, text, confirm_label)  // [Cancel(secondary), Confirm(DANGER)], initial_focus = Cancel
Dialog::prompt(id, title, input: TextInput, confirm) // body Input; initial_focus = input
Dialog::facts(id, title, facts: Vec<Prop>, code: Vec<String>, token: Option<&str>, confirm: Button)
    // [Cancel(secondary), confirm]; width 66; body = DialogBody::Facts{facts, code, ack: token.map(AckInput{ input: TextInput::new(id.sub("ack"), &format!("Type {tok} to confirm")).plain_label(), token })}
    // initial_focus = ack input if token else Cancel
pub struct Dialog { pub id, pub title, pub body: DialogBody, pub actions: Vec<Button>, pub cancel_index: Option<usize>, pub width: u16, pub area: Rect, pub result: Option<DialogResult>, pub initial_focus: WidgetId }
pub enum DialogResult { Action(usize), Cancelled }
d.armed() -> bool   // true when no token, or typed text.trim() == token — the confirm button is disabled/refused until armed
d.on_key(&key, &mut focus, &ring) -> Outcome; d.on_click(id, pos, &mut focus) -> Outcome; d.on_click_outside() -> Outcome; d.on_paste(text); d.is_editing(); d.render(screen, buf, ctx)
```
Typed-token example (tablepro app.rs:959-981, 1076-1110):
```rust
let token = deliberate.then(|| stmt.target().unwrap_or("yes").to_owned());
let confirm = if dangerous { Button::danger(SAFETY_DIALOG.sub("ok"), "Execute") } else { Button::primary(SAFETY_DIALOG.sub("ok"), "Execute") };
let mut d = Dialog::facts(SAFETY_DIALOG, title, facts, code_lines, token.as_deref(), confirm);
if token.is_none() { d.initial_focus = if dangerous { d.actions[0].id } else { d.actions[1].id }; }   // Cancel for dangerous, Execute otherwise
d.width = 74;
self.open_dialog(d);
```
Facts rows: `Prop::new("Action", s).tone(Tone::Error)`, `.wrap()` for long text, `.tone(Tone::Muted)` for meta. Confirm enables via `armed()`: Enter in the ack input only advances focus (`focus.next`) — the operator must reach the button deliberately.
Close-only dialog pattern: `let mut d = Dialog::facts(id, title, props, lines, None, Button::secondary(id.sub("close"), "Close")); d.actions.remove(0); d.cancel_index = Some(0); d.initial_focus = d.actions[0].id; d.width = 80;`
Result handling: `DialogResult::Action(1)` == confirm for the 2-button constructors; `Action(0)` or `Cancelled` == cancel. Text-body dialogs answer `y`/`n`.

---

## 5. Widget API card (constructor · render · events)

All widgets: `render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color)` unless noted; they register their own hits/focus; `bg` is the container plane (`t.canvas` / `t.surface` / `t.surface_elevated`). Row ids: `id.child(i)`; `locate(id)`/`owns(id)` map a hit id back to a row.

| Widget | Constructor / state | Events |
|---|---|---|
| **Picker** (`widgets::picker`) | `let mut p = Picker::new(id, "Title"); p.placeholder = "…".into(); p.width = 72; p.scope = Some("All · Tab scope".into()); p.searchable = false; p.max_rows = 12; p.empty_text; p.status = PickerStatus::Loading(String)/Error{message,detail}; p.set_items(Vec<PickerItem>); p.set_cursor(i)` · `PickerItem { label: String, detail: String, glyph: &'static str, group: &'static str, tag: Option<&'static str>, matched: Vec<usize>, disabled: bool }` · `pub query: String` | `on_key(&key) -> (Outcome, Option<PickerEvent>)` — `QueryChanged, Chosen(i), ChosenAlt(i), Secondary(i), NextScope (Tab), Cancelled (Esc), Back (Backspace on empty)`; `on_click(id) -> Option<PickerEvent>`; `on_wheel(delta) -> Outcome`; `render(screen, buf, ctx, hints: &str)` (pass `""`) |
| **ListBox** (`widgets::list`) | `ListBox::new(id, vec![ListItem::new("label").meta("meta").disabled(false)], SelectMode::Single|Multi).empty_text("…")`; `pub cursor, pub chosen: Option<usize>, pub checked: Vec<bool>, pub scroll` | `on_key(&key) -> Outcome`; `activate(i)`; `on_click(row)`; `on_wheel(delta)`; `on_scrollbar(pos)`; `locate/owns`; `row_id(i)` |
| **TreeView** (`widgets::tree`) | `TreeView::new(id, vec![TreeNode::dir("label", children), TreeNode::leaf("x"), TreeNode::leaf_meta("x","meta"), TreeNode::lazy("x"), TreeNode::note("empty")]).glyph("▪")` — first level auto-expanded; `pub cursor, pub selected: Option<Path>, pub expanded: HashSet<Path>`; `rows() -> &[FlatRow]`; `set_children(path, kids)`; `set_busy(path, bool)`; `set_filter(Option<&str>)`; `reveal(path)`; `expand_all/collapse_all` | `on_key(&key) -> (Outcome, Option<TreeEvent>)` — `Expand(Path), Activate(Path)`; `toggle(i)`; `on_click_row(i)`; `on_click_toggle(i)`; `on_wheel`; `locate(id) -> Option<(usize, bool /*toggle*/)>` |
| **StepRail** (`widgets::steps`) | `StepRail::new(id, Stage::ALL.iter().map(|s| Step::new(s.label())).collect()).selectable(false)`; `pub numbered: bool`; `set_state(i, StepState)`; `set_meta(i, Option<String>)`; `frontier()`, `counts() -> (done, skipped, failed?)`, `failed()` · `StepState::{Queued, Running, Done, Skipped, Failed, Blocked}` (`label()`, `terminal()`) | `on_key -> Outcome` (only when selectable: ↑↓ j k Home End); `on_click(row)`; `on_wheel`; `on_scrollbar` |
| **Tabs** (`widgets::tabs`) | `Tabs::new(id, &["A","B"])` or `Tabs::with_items(id, vec![TabItem::new("l").prefix("1").suffix("▶").closable()])`; `pub active, cursor, first, areas: Vec<Rect>, allow_new, quiet`; `set_active(i)`; `remove(i)`; **render needs height 2** (labels + underline row); jackin rebuilds items each frame keeping `first` | `on_key -> (Outcome, Option<TabEvent>)` — `Activated(i), Close(i), New`; `on_click(id)`; ids `tab_id(i)`, `close_id(i)`, `new_id()`, `left_id()`, `right_id()` |
| **TextViewport** (log/terminal, `widgets::viewport`) | `TextViewport::new(id).max_lines(2000).wrap(false)`; `with_lines(id, Vec<Line>)`; `pub follow: bool`; `push(Line)`, `set_lines`, `replace_last`, `clear`; `Line = Vec<Span>`; `Span::new("text", Tone::Muted).bold().italic().underline().reversed()`, `Span::plain`, `Span::muted`; `selected_text()`, `clear_selection()`, `set_follow(bool)`, `is_at_tail()`, `pos_at(pos)` | `on_key -> (Outcome, Option<ViewportEvent>)` — `Copy(String) (y), SelectionChanged, FollowChanged(bool)`; keys ↑↓ j k PgUp PgDn Home End g G f; `on_click(pos)`, `on_drag(pos)`, `select_word_at(pos)`, `on_wheel`, `on_scrollbar`, `owns(id)` |
| **ScrollPanel** (plain string log, `widgets::panel`) | `ScrollPanel::new(id, Vec<String>).wrap(true)`; `push(String)`; `pub follow` | `on_key -> Outcome`; `on_wheel`; `on_scrollbar`; `render(area, buf, ctx, bg, style_line: fn(&Theme, &str) -> Style)` |
| **Panel** (container) | `Panel::card(Some("Title")).focused(f).meta("right text")` or `Panel::framed(..)`; `let inner = panel.render(area, buf, &t); let bg = panel.bg(&t);` (card → `t.surface`, framed → `t.canvas`) | none (static chrome) |
| **Props** (`widgets::props`) | static: `props::render(area, buf, &t, &[Prop::new("k","v").tone(Tone::Secondary).wrap().copyable()], bg) -> rows_used`; interactive: `PropsList::new(id, props)`, `set_props` | `PropsList::on_key -> (Outcome, Option<PropsEvent>)` — `Copy(i) (y), Activate(i) (Enter)`; `on_click(row)`; `on_wheel` |
| **Progress** (`widgets::progress`) | `render_bar(area, buf, ctx, label, ratio: f64, ProgressStatus::{Active,Done,Error,Paused}, bg)`; `render_indeterminate(area, buf, ctx, label, bg)`; `render_spinner(area, buf, ctx, label, bg)`; `spinner_frame(tick) -> &'static str`; `Meter::new(Some(pct)).value("38%").tone(MeterTone::{Normal,Level(l),Warning,Exhausted,Stale,Refreshing,Error,Unknown}).visual(MeterVisual::{Line,Block}).render(area, buf, ctx, bg)` | none |
| **EmptyState** (`widgets::empty`) | `empty::render(area, buf, &t, &EmptyState::new("Nothing here").hint("Press n to add"), bg)`; `EmptyState::error("Could not load")` → bold `!` prefix | none |
| **Segments** (`widgets::segments`) | `segments::render(area, buf, ctx, &left, &right, bg)`; `Segment::new(text, Tone).bold().clickable(id).priority(0..=9)` — lower priority drops first; clickable ones register hits | click ids resolved by the app (`if id == STRIP_HELP …`) |
| **StatusBar** (`widgets::statusbar`) | `let mut bar = StatusBar::new(); bar.left.push(StatusItem::new("PR #482", Tone::Normal).strong().clickable(id).priority(10)); bar.center.push(..); bar.right.push(StatusItem::new("Session", Tone::Muted).meter(Some(38), MeterTone::Normal).chip().priority(9)); bar.render(area, buf, ctx)` — own plane (`surface_elevated`), drops center → right → left by priority | click ids via `StatusItem::clickable` |
| **TextInput** (`widgets::input`) | `TextInput::new(id, "Label").value("v").placeholder("p").help("h").required(true).plain_label().masked().reveal_tail(4).validator(f)`; `pub editing, pub error: Option<String>`; `text()`, `begin_edit()`, `commit()`, `cancel()`, `validate()`; `TextInput::HEIGHT` | `on_key -> (Outcome, Option<InputEvent>)` — `Committed, Cancelled, CommittedTab{backward}, Changed`; `on_paste`; `on_click(pos, was_focused)` |
| **MenuBar / ContextMenu** (`widgets::menu`) | see §3; `ContextMenu::new(id, items).anchor(rect, Placement::{Below,Above,Right}).title("t")` or `.at(pos)`; `render(screen, buf, ctx)` | `on_key -> (Outcome, Option<MenuEvent>)` — `Chosen(i), Dismissed`; `on_click(id) -> Option<MenuEvent>`; `on_click_outside()` |
| **Lockup** (`widgets::brand`) | `Lockup::new(BRAND_MARK)` / `Lockup::compact(..)`; `width()`; `Lockup::style(&t)`; `render(x, y, buf, &t) -> u16`; `render_clickable(x, y, buf, ctx, id) -> u16` | none |
| **Button** | `Button::primary/secondary/subtle/danger(id, "Label")`; `width()`; `button::row_layout_right(area, &widths, gap) -> Vec<Rect>` | `on_key -> (Outcome, bool fired)`; `on_click() -> bool` |

Pitfall (widget state ownership): widgets are plain structs owned by the screen; they cache `area`/`areas` from the last render, so hit resolution (`locate`) is only valid after a render. Rebuild-per-frame widgets (Tabs in capsule) must carry over `first`/`cursor` manually.

---

## 6. Theme API

```rust
use junie_tui::theme::{Theme, ColorLevel, Tone, ButtonKind, BadgeKind, SyntaxTone};
Theme::junie() (const) · Theme::for_level(ColorLevel) · ColorLevel::{TrueColor, Ansi256, Ansi16, Mono}::detect()/label()
```
Colour fields: `canvas, surface, surface_elevated, surface_overlay, field, field_hover, popover, highlight, highlight_danger, error_soft, border_subtle, border_strong, text_primary, text_secondary, text_muted, text_faint, text_ghost, text_on_accent, accent, accent_hover, accent_pressed, accent_bg, accent_bg_subtle, focus, disabled, error, error_bg, warning, success, info` + `level: ColorLevel`.
Style resolvers (all `-> Style` unless noted): `base()` (primary on canvas), `on(bg)`, `primary()`, `secondary()`, `muted()`, `faint()`, `accent_fg()`, `error_fg()`, `title()`, `label(focused)`, `key_hint_key()`, `key_hint_action()`, `border(focused)`, `backdrop(style) -> Style`, `row(VisualState, bg)`, `lift(bg) -> Color`, `gutter(VisualState, bg, on_accent)`, `button(ButtonKind, VisualState, bg)`, `field_style(VisualState)`, `placeholder(VisualState)`, `selection()`, `scrollbar_track()`, `scrollbar_thumb(focused, hovered)`, `tone(Tone) -> Color`, `syntax(SyntaxTone)`, `badge(BadgeKind)`.
Enums: `Tone::{Normal, Secondary, Muted, Faint, Error, Warning, Success}` (never accent); `ButtonKind::{Primary, Secondary, Subtle, Danger, Toggle}`; `BadgeKind::{Edit}`; `SyntaxTone::{Keyword, Ident, Number, Str, Operator, Punct, Comment, Plain}`.
Rule: **no app code spells an RGB**; ask the theme. `VisualState` comes from `ctx.state(id)`.
Text helpers: `junie_tui::ui::text::{width, truncate, truncate_middle, wrap}`; popup placement `junie_tui::ui::popup::{place, Placement}`; `junie_tui::ui::layout::{Split, SplitDir, Maximized}`.

---

## 7. sim / domain layering (jackin)

- `domain/` = plain data (workspace, instance, account, usage…). No rendering, no services.
- `sim/world.rs` `World` = the whole fixture state: `scenario, clock: Clock, arbiter, workspaces, instances, daemons: BTreeMap<InstanceId, Daemon>, accounts, op, fs, github, clipboard, jobs: Vec<Job>, daemon_health, …`.
  - `World::now_ms()/now_secs()`; `schedule(delay_ms, Msg)` pushes `Job { due_ms, msg }` sorted; `tick(interval_ms) -> Vec<Msg>`: `clock.advance(interval)`; if `!clock.running` return empty; pop jobs with `due_ms <= now`; `d.tick(now)` for daemons of Running instances; typed `Msg` enum (`WorkspaceSaved{id, ok}`, `Refreshed{ok}`, `Takeover{instance, by}` …) dispatched by the app to the current screen then fallbacks (`dispatch_msg` app.rs:446).
- `domain/fixtures.rs` `pub fn world_for(scenario: Scenario) -> World` matches on scenario: `base_world(scenario)` (constants: `HOME`, `Clock::new()`, `Arbiter::new(0)`, empty vectors) → `populated(scenario, rich)` adds workspaces/instances/accounts → per-scenario tweaks (`OutroLast`: one instance, `arbiter.entered_at_ms = Some(-8_040_000)`; `HardCases`: `daemon_health = Stale`, `refresh_fails`, `takeover_at_ms = Some(45_000)`). All timestamps computed from `clock.now_secs()` ± offsets → deterministic.
- `sim/launch.rs` `LaunchRun::new(plan, agent, container, run_id)` with fixed `durations: [u64; 11]` in ticks; `advance() -> Vec<LaunchEvent>` emits `StageChanged(Stage, StepState)`, `BuildLine(String)` (from a const `BUILD_LOG: [&str; 44]`, proportional to elapsed/duration), `Failed(LaunchFailure)`, `Ready`. The cockpit screen maps events onto `StepRail::set_state(stage.index(), state)` and pushes build lines into a `TextViewport`.
- `sim/pty.rs` PTY synthesis: `AgentProcess { script: Vec<Step>, pc, next_at_ms, reply_queue: VecDeque<(i64, Line)>, state: AgentState, … }`; `Step::{Emit(delay_ms, Line), State(AgentState), Touch(&'static str), Await}`; `tick(now_ms) -> Vec<Line>` drains due replies and walks the script until an `Emit` is not yet due or an `Await`. `Pane { proc, term: TextViewport (id = WidgetId::of("capsule.pane").child(id), .max_lines(SCROLLBACK), follow=true), input }`; `Pane::tick(now_ms) -> bool` pushes lines into `term` and refreshes the prompt row; `type_char(c, now_ms, ws)` echoes, `on_input` produces scripted replies scheduled at `now + delay`. `Daemon { tabs: Vec<Tab>, panes, active, workspace, attached_by }`; `Daemon::tick(now_ms) -> bool` ticks every pane.
- Determinism contract: same `(scenario, motion, frame, size)` ⇒ same buffer (test `reduced_motion_and_paused_frames_are_deterministic`).

---

## 8. Tests (`app_tests.rs`, `#[cfg(test)] mod app_tests;` in main.rs)

Harness `H` (jackin app_tests.rs:15-113; tablepro identical minus scenario):
```rust
use ratatui::{Terminal, backend::TestBackend}; use ratatui::crossterm::event::{KeyCode, KeyModifiers}; use ratatui::layout::Position;
use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome}; use junie_tui::theme::Theme;
pub struct H { pub app: App, pub term: Terminal<TestBackend> }
impl H {
    pub fn new(scenario: Scenario, motion: Motion, frame: u64, w: u16, h: u16) -> Self { let app = App::for_scenario(scenario, motion, frame, Theme::junie()); let term = Terminal::new(TestBackend::new(w, h)).unwrap(); let mut hh = Self { app, term }; hh.draw(); hh }
    pub fn draw(&mut self) { self.term.draw(|f| self.app.render(f)).unwrap(); }
    pub fn key(&mut self, code: KeyCode) -> Outcome { let o = self.app.handle(Input::Key(Key { code, mods: KeyModifiers::NONE })); self.draw(); o }
    pub fn ctrl(&mut self, c: char) -> Outcome   // KeyModifiers::CONTROL
    pub fn type_str(&mut self, s: &str)          // key per char
    pub fn ticks(&mut self, n: usize)            // n × Input::Tick then draw
    pub fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) -> Outcome
    pub fn click(&mut self, x: u16, y: u16)      // Down then Up
    pub fn resize(&mut self, w: u16, h: u16)     // backend_mut().resize + Input::Resize
    pub fn tab_to(&mut self, id: WidgetId)       // Tab ≤24 times until focus == id, else panic with ring
    pub fn text(&self) -> String                 // buffer symbols row by row, '\n' joined
    pub fn find(&self, needle: &str) -> Option<(u16, u16)>   // grapheme-exact search → (x, y) for clicks
}
```
Assertion idioms: `assert!(h.text().contains("Terminal too small"), "{}", h.text())`; `assert_eq!(h.app.route, Route::Manager)`; `assert_eq!(h.app.focus.current(), Some(crate::screens::manager::TREE))` (export widget-id consts as `pub const`); row helpers in app_tests_chrome.rs: `fn row(h: &H, y: u16) -> String` (`h.text().lines().nth(y)`), `fn last_row(h) -> String`; `assert_eq!(t.matches("Enter Choose").count(), 1)` proves the hint bar is the single hint surface.
Determinism test (app_tests.rs:139): `let a = H::new(S, Motion::Paused, 282, 100, 30); let b = …same…; assert_eq!(a.text(), b.text());` and `p.ticks(5); assert!(text still shows frame 45)`.
Chrome test (app_tests_chrome.rs:22): `row(&h, 0)` contains `"jackin❯"`, `"File"`, `"Help"`; `F10` opens (`h.key(KeyCode::F(10))`), `Right` switches, `Esc` closes; click via `h.find("View")`.
Baselines: only the showcase has `tests/showcase_baseline.txt` (FNV digest of every cell per page at 120×40 and 80×24, `UPDATE_BASELINE=1 cargo test --bin showcase showcase_visual_baseline` regenerates; showcase app_tests.rs:622). **jackin and tablepro have no baseline file** — they rely on text assertions + `j_*`/`t_*` PNG captures. Holla may add one using the same digest loop (`for cell in buf.content { hash over "{symbol}|{fg:?}|{bg:?}|{modifier:?};" }`).

---

## 9. Visual baseline (`tools/tuisnap_baseline.sh`)

The tmux/Python capture harness this note originally documented (capture.sh +
ansi2png.py + env.sh) was removed 2026-09-12; the `shots/` corpus it produced
(`f_*` showcase, `s_*` component pages, `t_*` tablepro, `j_*` jackin, `h_*`
holla) is frozen historical evidence. The current recipe is the tuisnap
snapshot store at `shots/tuisnap/`:

```sh
tools/tuisnap_baseline.sh                      # build + capture the 367-capture matrix
open shots/tuisnap/report.html                 # review every actual
tuisnap accept --store shots/tuisnap --all     # approve after review
tuisnap report --store shots/tuisnap           # re-verify: approved frames must report matched
APPS=holla ONLY='holla_rust-dirty' SKIP_BUILD=1 tools/tuisnap_baseline.sh   # subset re-run
```

- Naming: `<app>_<surface>_<state>_<cols>x<rows>_<color>`; the matrix and its
  rationale live in `docs/baseline/tuisnap-coverage.md`; the runner is the
  executable source of truth. Sizes used: 120×40 (primary review), 80×24
  (canonical default), 100×30 (holla mono + tablepro drawer breakpoint),
  160×50 (wide holla), 72×20 (documented minimum, spot only).
- Paused-frame determinism: captures pass `--scenario … --motion paused
  --frame N`; holla/jackin frames are exact under the seeded sim, and
  `HOLLA_NO_HISTORY=1` (exported by the runner) suppresses history side
  effects.
- Git tracking: `approved/` and `frames/*.{ansi,txt}` are tracked; `actual/`,
  `diff/`, `report.html` and `frames/*.{png,html}` are ignored deterministic
  re-renders (`tuisnap render`, `tuisnap report`).
- The runner strips `NO_COLOR` from its environment (`PRESERVE_NO_COLOR=1`
  opts out) — an exported NO_COLOR silently poisons every colour capture.
- Lanes tuisnap CLI cannot drive (mouse hover/drag, mid-session resize,
  wall-clock motion phases, Alt+Enter/Alt+0..9 chords, the showcase progress
  page) survive only as frozen legacy frames; the successor path is tuisnap's
  Rust PTY API (`Session::click/drag/resize`) from a Rust test — see the
  coverage doc's honest-gap section.

---

## 10. clippy / fmt / naming gotchas observed

- Toolchain: edition 2024, `let … && let … chains` used everywhere (`if let Some(x) = a && cond {}`); rustc 1.98 installed.
- `cargo clippy --all-targets -- -D warnings` is the gate: test modules are linted too.
- Explicit allows used (copy where the same shape appears): `#[allow(clippy::large_enum_variant)]` on `enum Modal` (jackin screens/mod.rs:97, tablepro app.rs:111 with a trailing comment) and tab enums; `#[allow(clippy::too_many_arguments)]` on `modal_frame` and fixture builders. No crate-level `#![allow]`, no `clippy.toml`/`rustfmt.toml`.
- Module naming: binary dir uses snake_case (`jackin_preview`), `[[bin]] name` uses kebab-case (`jackin-preview`); main.rs declares `mod app; #[cfg(test)] mod app_tests; #[cfg(test)] mod app_tests_chrome; …`. Screens live in `screens/{mod,…}.rs`, sims in `sim/`, data in `domain/`.
- Unused-var suppression style in codebase: `let _ = (tag, owner);` rather than underscore-prefixing when the value is deliberately computed.
- `Outcome::or` combinator (Changed dominates) — prefer `o.or(other)` over manual matching; `Outcome::consumed()`.
- `Key` helpers: `key.is(KeyCode::Esc)`, `key.is_char('q')`, `key.ctrl_char('c')`, `key.plain()` (shift allowed), `key.alt()`.
- `Rect::ZERO`, `area.intersection(*buf.area())` guards at the top of every widget render.
- `fill(buf, area, style)` from `junie_tui::ui::ctx` clears symbols to `" "` — use it before drawing any surface.
- Don't call `Read`-style verification after edits; tests + clippy are the proof.

---

## Pitfall list (short)

1. Never use `std::time::Instant` in holla (tablepro does; jackin does not). Status/flash/double-click timing use `world.now_ms()`.
2. `Interaction.tick` must derive from virtual ms (`now_ms / 80`), otherwise spinners break paused determinism.
3. Rebuild `HitRegistry`/`FocusRing` every frame in `render`; store them back on `self`; then fix focus (`primary_focus` → ring.first()).
4. `push_modal` sets focus to the modal's initial id and clears hover/pressed; `pop_modal` restores `saved_focus`.
5. Draw order: frame → screen body → footer → `host_menu.render_open` → modal → footer again (modal hints). Dropdowns/popovers must be rendered after the body so their hits shadow it (later registration wins).
6. `Lockup::new(BRAND_MARK)` — one `pub const BRAND_MARK: &str = "…❯"` in app.rs, referenced by the menu bar `.brand(...)`, the too-small notice and any capsule-style chrome. Tests grep for it.
7. Right-aligned identity on the menu row: compute `used = max(areas.right(), brand_area.right())`, then `segments::render(rest, buf, ctx, &[], &right, t.canvas)`.
8. Menus are data: dispatch chosen items by `label` string and re-inject the shortcut key through `self.handle(Input::Key(..))`.
9. `Picker::render(.., hints: "")` — the HintBar is the only hint surface; `picker.searchable=false` for fixed choices.
10. `Dialog::facts` initial focus: ack input when token, else set `d.initial_focus = d.actions[0|1].id` explicitly (Cancel for destructive intent).
11. Widget-id constants: `pub const TREE: WidgetId = WidgetId::of("manager.tree");` (const fn FNV) — export from the screen so tests can `tab_to(TREE)`.
12. `too_small` is decided inside `draw` from the frame area (not from Resize), so tests must `draw()` after `resize()` (the harness does).
13. `Tabs::render` requires a 2-row area; `TextInput::HEIGHT`/`Select::HEIGHT` are the field heights.
14. The tuisnap runner strips `NO_COLOR` from its environment (`PRESERVE_NO_COLOR=1` opts out) — an exported NO_COLOR silently turns every colour capture mono.
