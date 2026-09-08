//! Application routes and typed intent reduction; shared runtime owns interaction.
use crate::{
    dispatch::{self, Destination},
    domain::{action::Scope, fixtures, ranking::Pin},
    scenario::{Motion, Scenario},
    screens::{
        actions::{Actions, Choice},
        activity::{self, Activities},
        clone::ClonePicker,
        dialogs::{self, ProductDialog},
        home::{self, HomeState},
        plan::{self, PlanState},
        plan_gate::PlanGate,
    },
    sim::{pg, world::World},
};
use junie_tui::{
    ActionKey, Binding, Brand, Chord, Constraints, Cx, Dialog, FgStep, Focusability, FrameRead,
    HintBar, HintLayer, Id, Intent, ItemKey, KeyCode, KeyMap, KeyPhase, Menu, MenuAction, MenuBar,
    MenuItem, MenuState, Part, PartRef, Phase, Rect, Response, Role, StatusAction, StatusBar,
    StatusItem, StylePatch, Surface, TooSmall, Ui,
};

const BRAND: Id = Id::root("holla.brand");
const MENU: Id = Id::root("holla.menu");
const HEADER: Id = Id::root("holla.header");
const STRIP: Id = Id::root("holla.activities");
const FOOTER: Id = Id::root("holla.footer");
const MIN_WIDTH: u16 = 72;
const MIN_HEIGHT: u16 = 20;
const HELP: ActionKey = ActionKey::application("holla.help");
const ABOUT: ActionKey = ActionKey::application("holla.about");
const QUIT: ActionKey = ActionKey::application("holla.quit");
const HOME: ActionKey = ActionKey::application("holla.home");
const NEXT_ACTIVITY: ActionKey = ActionKey::application("holla.next-activity");
const SCOPE: ActionKey = ActionKey::application("holla.scope");
const PREVIEW: ActionKey = ActionKey::application("holla.preview");
const ACTIONS: ActionKey = ActionKey::application("holla.actions");
const TOGGLE: ActionKey = ActionKey::application("holla.plan-toggle");
const CONTINUE: ActionKey = ActionKey::application("holla.plan-continue");
const OPEN_MENU: ActionKey = ActionKey::application("holla.open-menu");
const RESULTS: ActionKey = ActionKey::application("holla.results");
const QUERY_ESCAPE: ActionKey = ActionKey::application("holla.query-escape");
const PREVIOUS: ActionKey = ActionKey::application("holla.previous");
const INTERRUPT: ActionKey = ActionKey::application("holla.interrupt");
const ACTIVITY_SHORTCUTS: &[(char, ActionKey)] = &[
    ('1', ActionKey::application("holla.activity-1")),
    ('2', ActionKey::application("holla.activity-2")),
    ('3', ActionKey::application("holla.activity-3")),
    ('4', ActionKey::application("holla.activity-4")),
    ('5', ActionKey::application("holla.activity-5")),
    ('6', ActionKey::application("holla.activity-6")),
    ('7', ActionKey::application("holla.activity-7")),
    ('8', ActionKey::application("holla.activity-8")),
    ('9', ActionKey::application("holla.activity-9")),
];
const GLOBAL: &[Binding<ActionKey>] = &[
    Binding {
        action: HELP,
        chord: Some(Chord::key(KeyCode::F(1))),
        cmd: HELP,
        label: "Help",
        priority: 50,
        visible: true,
    },
    Binding {
        action: OPEN_MENU,
        chord: Some(Chord::key(KeyCode::F(10))),
        cmd: OPEN_MENU,
        label: "Menu",
        priority: 40,
        visible: true,
    },
    Binding {
        action: QUIT,
        chord: Some(Chord::with(
            KeyCode::Char('q'),
            junie_tui::KeyModifiers::CONTROL,
        )),
        cmd: QUIT,
        label: "Quit",
        priority: 20,
        visible: true,
    },
    Binding {
        action: NEXT_ACTIVITY,
        chord: Some(Chord::with(
            KeyCode::Char('a'),
            junie_tui::KeyModifiers::CONTROL,
        )),
        cmd: NEXT_ACTIVITY,
        label: "Activities",
        priority: 30,
        visible: true,
    },
    Binding {
        action: SCOPE,
        chord: Some(Chord::with(
            KeyCode::Char('s'),
            junie_tui::KeyModifiers::CONTROL,
        )),
        cmd: SCOPE,
        label: "Scope",
        priority: 40,
        visible: true,
    },
    Binding {
        action: PREVIEW,
        chord: Some(Chord::with(
            KeyCode::Char('p'),
            junie_tui::KeyModifiers::CONTROL,
        )),
        cmd: PREVIEW,
        label: "Preview",
        priority: 60,
        visible: true,
    },
    Binding {
        action: ACTIONS,
        chord: Some(Chord::with(
            KeyCode::Char('o'),
            junie_tui::KeyModifiers::CONTROL,
        )),
        cmd: ACTIONS,
        label: "Actions",
        priority: 60,
        visible: true,
    },
];
#[derive(Clone, Copy, PartialEq, Eq)]
enum Route {
    Home,
    Plan,
    Activity,
}
enum Overlay {
    Dialog(Box<ProductDialog>),
    Clone(ClonePicker),
    Actions(Actions),
    Gate(PlanGate),
}

/// Holla's deterministic launcher application. External commands remain fixture data.
///
/// Run with [`junie_tui::FeedbackClock::Simulation`] initialized to
/// [`Self::fixture_time_ms`]. An incompatible feedback clock pauses simulation
/// with a visible diagnostic before advancing any fixture state.
pub struct App {
    world: World,
    motion: Motion,
    route: Route,
    home: HomeState,
    plan: Option<PlanState>,
    activities: Activities,
    overlay: Option<Overlay>,
    retired_overlay: Option<Overlay>,
    menu: MenuState,
    keymap: KeyMap,
    status: Option<String>,
    status_until_ms: Option<i64>,
    last_step_at: Option<junie_tui::Moment>,
    quit: bool,
}
impl std::fmt::Debug for App {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("App")
            .field("scenario", &self.world.scenario)
            .field("motion", &self.motion)
            .finish_non_exhaustive()
    }
}
fn menu_bar() -> MenuBar<'static> {
    // The static menu data lives here, inside the one constructor both phases
    // call, so every configured row keeps exactly one construction site (§13).
    const FILE: &[MenuItem<'static>] = &[MenuItem::new(QUIT, "Quit").chord(Chord::with(
        KeyCode::Char('q'),
        junie_tui::KeyModifiers::CONTROL,
    ))];
    const GO: &[MenuItem<'static>] = &[MenuItem::new(HOME, "Home")];
    const HELP_MENU: &[MenuItem<'static>] = &[
        MenuItem::new(HELP, "Key reference").chord(Chord::key(KeyCode::F(1))),
        MenuItem::new(ABOUT, "About holla"),
    ];
    const MENUS: &[Menu<'static>] = &[
        Menu::new("File", FILE),
        Menu::new("Go", GO),
        Menu::new("Help", HELP_MENU),
    ];
    MenuBar::new(MENU, MENUS).chord_case(junie_tui::ChordCase::UppercaseAscii)
}

fn brand() -> Brand<'static> {
    Brand::new(BRAND, "holla❯").clickable(true)
}

fn header_bar<'a>(items: &'a [StatusItem<'a>]) -> StatusBar<'a> {
    StatusBar::new(HEADER).right(items)
}

fn gate_dialog() -> Dialog<'static> {
    Dialog::new(crate::screens::plan_gate::GATE)
}

impl App {
    /// Revision of applied in-memory simulation effects; never a real-host revision.
    pub fn effect_revision(&self) -> u64 {
        self.world.effect_revision
    }

    /// Virtual milliseconds in the deterministic fixture, independent of wall time.
    pub fn fixture_time_ms(&self) -> i64 {
        self.world.now_ms()
    }

    /// Construct one named fixture at a virtual millisecond frame.
    pub fn for_scenario(scenario: Scenario, motion: Motion, frame: u64) -> Self {
        let mut world = fixtures::world_for(scenario);
        world.seek(frame);
        world.clock.running = motion != Motion::Paused;
        let mut keymap = KeyMap::new();
        for binding in GLOBAL {
            if let Some(chord) = binding.chord {
                keymap = keymap.bind(KeyPhase::Capture, chord, binding.action);
            }
        }
        keymap = keymap
            .bind(KeyPhase::Bubble, Chord::key(KeyCode::Down), RESULTS)
            .bind(KeyPhase::Bubble, Chord::key(KeyCode::Up), PREVIOUS)
            .bind(
                KeyPhase::Capture,
                Chord::with(KeyCode::Char('c'), junie_tui::KeyModifiers::CONTROL),
                INTERRUPT,
            );
        Self {
            world,
            motion,
            route: Route::Home,
            home: HomeState::default(),
            plan: None,
            activities: Activities::default(),
            overlay: None,
            retired_overlay: None,
            menu: MenuState::default(),
            keymap,
            status: None,
            status_until_ms: None,
            last_step_at: None,
            quit: false,
        }
    }
    fn open_dialog(&mut self, intent: dialogs::Intent, cx: &mut Cx<'_>) {
        let dialog = ProductDialog::new(intent, &self.world);
        dialog.open(cx);
        self.overlay = Some(Overlay::Dialog(Box::new(dialog)));
    }
    fn destination(&mut self, destination: Destination, cx: &mut Cx<'_>) {
        match destination {
            Destination::Notice(message) => self.set_status(message),
            Destination::Preview(action) => self.open_dialog(dialogs::Intent::Preview(action), cx),
            Destination::Trust { action, file } => {
                self.open_dialog(dialogs::Intent::Trust { action, file }, cx);
            }
            Destination::Plan(review) => {
                self.plan = Some(PlanState::new(*review));
                self.route = Route::Plan;
                cx.focus(plan::STEPS);
            }
            Destination::Activity(id, message) => {
                self.activities.show(id);
                self.route = Route::Activity;
                self.set_status(message);
                cx.focus(activity::output_id(id));
            }
            Destination::Database => match (
                pg::Review::new(&self.world, pg::Operation::Cancel),
                pg::Review::new(&self.world, pg::Operation::Terminate),
            ) {
                (Ok(cancel), Ok(terminate)) => self.open_dialog(
                    dialogs::Intent::Database {
                        cancel: Some(cancel),
                        terminate: Some(terminate),
                    },
                    cx,
                ),
                (Err(error), _) | (_, Err(error)) => self.set_status(error.to_string()),
            },
            Destination::Monitor => self.open_dialog(dialogs::Intent::Monitor, cx),
            Destination::Clone => {
                if let Some(github) = &self.world.github {
                    let picker = ClonePicker::new(github);
                    picker.open(cx);
                    self.overlay = Some(Overlay::Clone(picker));
                }
            }
        }
    }
    fn next_activity(&mut self, cx: &mut Cx<'_>) {
        let ordered = activity::ordered(&self.world);
        let next = (self.route == Route::Activity)
            .then(|| {
                ordered
                    .iter()
                    .position(|activity| Some(activity.id) == self.activities.current_id())
            })
            .flatten()
            .and_then(|index| ordered.get(index.saturating_add(1)))
            .or_else(|| ordered.first())
            .map(|activity| activity.id);
        if let Some(id) = next {
            self.activities.show(id);
            self.route = Route::Activity;
            cx.focus(activity::output_id(id));
        }
    }
    fn command(&mut self, command: ActionKey, cx: &mut Cx<'_>) {
        match command {
            HELP => self.open_dialog(dialogs::Intent::Help, cx),
            ABOUT => self.open_dialog(dialogs::Intent::About, cx),
            QUIT => self.open_dialog(dialogs::Intent::Quit, cx),
            INTERRUPT => self.quit = true,
            HOME => {
                self.route = Route::Home;
                cx.focus(home::QUERY);
            }
            OPEN_MENU => {
                let _ = menu_bar().open_menu(cx, &mut self.menu, 0);
            }
            NEXT_ACTIVITY => self.next_activity(cx),
            SCOPE if self.route == Route::Home => {
                self.home.scope = self.home.scope.map_or_else(
                    || Scope::ORDER.first().copied(),
                    |scope| {
                        Scope::ORDER
                            .iter()
                            .position(|current| *current == scope)
                            .and_then(|index| Scope::ORDER.get(index.saturating_add(1)).copied())
                    },
                );
                self.set_status(format!(
                    "Scope: {}",
                    self.home.scope.map_or("all", Scope::label)
                ));
            }
            PREVIEW if self.route == Route::Home => {
                if let Some(action) = self.home.subject(&self.world, cx) {
                    if let crate::domain::action::Availability::NeedsTrust(file) =
                        &action.availability
                    {
                        self.open_dialog(
                            dialogs::Intent::Trust {
                                file: file.clone(),
                                action,
                            },
                            cx,
                        );
                    } else {
                        self.open_dialog(dialogs::Intent::Preview(action), cx);
                    }
                }
            }
            ACTIONS if self.route == Route::Home => {
                if let Some(action) = self.home.subject(&self.world, cx) {
                    let picker = Actions::new(action, &self.world);
                    picker.open(cx);
                    self.overlay = Some(Overlay::Actions(picker));
                }
            }
            TOGGLE
                if self.route == Route::Plan
                    && cx
                        .state(plan::STEPS)
                        .contains(junie_tui::StateFlags::FOCUSED) =>
            {
                if let Some(plan) = &mut self.plan {
                    let message = plan.toggle_selected();
                    self.set_status(message);
                }
            }
            CONTINUE if self.route == Route::Plan => {
                if let Some(plan) = &self.plan {
                    self.plan_event(plan.continuation(), cx);
                }
            }
            RESULTS if self.route == Route::Home => self.home.focus_first(&self.world, cx),
            PREVIOUS if self.route == Route::Home => cx.focus_prev(),
            QUERY_ESCAPE if self.route == Route::Home => {
                if self.home.scope.take().is_some() {
                    self.set_status("Scope: all".into());
                } else {
                    self.home.clear_query();
                }
            }
            command if self.route == Route::Activity => {
                if let Some(index) = ACTIVITY_SHORTCUTS
                    .iter()
                    .position(|(_, key)| *key == command)
                {
                    if let Some(activity) = activity::ordered(&self.world).get(index) {
                        let id = activity.id;
                        self.activities.show(id);
                        cx.focus(activity::output_id(id));
                    } else {
                        self.set_status(format!("No activity {}", index.saturating_add(1)));
                    }
                }
            }
            _ => {}
        }
    }
    fn action_choice(
        &mut self,
        choice: Choice,
        action: crate::domain::action::Action,
        cx: &mut Cx<'_>,
    ) {
        match choice {
            Choice::Run => {
                let destination = dispatch::invoke(&mut self.world, &action);
                self.destination(destination, cx);
            }
            Choice::Preview => self.open_dialog(dialogs::Intent::Preview(action), cx),
            Choice::Copy => self.set_status("Copied · simulated clipboard".into()),
            Choice::Alias => self.open_dialog(dialogs::Intent::Alias(action), cx),
            Choice::Pin => {
                if self.world.memory.pin_at(&self.world.cwd, &action.command) {
                    self.world
                        .memory
                        .pins
                        .retain(|pin| pin.path != self.world.cwd || pin.command != action.command);
                    self.set_status(format!("Unpinned {}", action.command));
                } else {
                    self.world.memory.pins.push(Pin {
                        path: self.world.cwd.clone(),
                        command: action.command.clone(),
                    });
                    self.set_status(format!("Pinned {} here", action.command));
                }
            }
            Choice::Hide => {
                if self
                    .world
                    .memory
                    .hidden_at(&self.world.cwd, &action.command)
                {
                    self.world
                        .memory
                        .hides
                        .retain(|pin| pin.path != self.world.cwd || pin.command != action.command);
                    self.set_status(format!("Unhidden {} here", action.command));
                } else {
                    self.world.memory.hides.push(Pin {
                        path: self.world.cwd.clone(),
                        command: action.command.clone(),
                    });
                    self.set_status(format!(
                        "Hidden {} here · Reset ranking restores",
                        action.command
                    ));
                }
            }
            Choice::Reset => {
                self.world.memory.reset_at(&self.world.cwd, &action.command);
                self.set_status(format!(
                    "Reset ranking for {} · pins, aliases, hides cleared",
                    action.command
                ));
            }
        }
    }
    fn update_overlay(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let Some(mut overlay) = self.overlay.take() else {
            return Response::ignored();
        };
        let (response, closed) = match &mut overlay {
            Overlay::Dialog(dialog) => {
                let (response, event) = dialog.update(cx, &mut self.world);
                let closed = event.is_some();
                if let Some(event) = event {
                    match event {
                        dialogs::Event::Close => {}
                        dialogs::Event::Quit => self.quit = true,
                        dialogs::Event::Destination(destination) => {
                            self.destination(*destination, cx);
                        }
                    }
                }
                (response, closed)
            }
            Overlay::Clone(picker) => {
                let (response, chosen, closed) = picker.update(cx);
                let done = chosen.is_some() || closed;
                if let Some((repo, login)) = chosen {
                    self.open_dialog(dialogs::Intent::Clone { repo, login }, cx);
                }
                (response, done)
            }
            Overlay::Actions(picker) => {
                let (response, choice, closed) = picker.update(cx);
                let done = choice.is_some() || closed;
                if let Some(choice) = choice {
                    match picker.reviewed_action(&self.world) {
                        Ok(action) => self.action_choice(choice, action, cx),
                        Err(reason) => self.set_status(reason.into()),
                    }
                }
                (response, done)
            }
            Overlay::Gate(gate) => {
                if let Some(plan) = &mut self.plan {
                    let (response, result) = gate.update(cx, &mut plan.review, &mut self.world);
                    let done = result.is_some();
                    if let Some(message) = result {
                        self.set_status(message);
                    }
                    (response, done)
                } else {
                    (Response::ignored(), true)
                }
            }
        };
        if !closed && self.overlay.is_none() {
            self.overlay = Some(overlay);
        } else {
            self.retired_overlay = Some(overlay);
        }
        response
    }
    fn strip_update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        for intent in cx.intents(STRIP) {
            if let Intent::Pointer {
                phase: Phase::Click | Phase::DoubleClick,
                part: PartRef {
                    item: Some(key), ..
                },
                ..
            } = intent
                && let Some(id) = self
                    .world
                    .activities
                    .iter()
                    .find(|activity| ItemKey::num(u64::from(activity.id)) == key)
                    .map(|activity| activity.id)
            {
                self.activities.show(id);
                self.route = Route::Activity;
                cx.focus(activity::output_id(id));
                return Response::changed();
            }
        }
        Response::ignored()
    }
    fn strip_draw(&self, ui: &mut Ui<'_>, area: Rect) {
        if self.world.activities.is_empty() || area.is_empty() {
            return;
        }
        ui.register_control(STRIP, area, Focusability::ClickOnly);
        let mut x = area.x;
        for (index, activity) in activity::ordered(&self.world).into_iter().enumerate() {
            let label = format!(
                " {} {} {} ",
                activity::state_glyph(activity.state),
                index.saturating_add(1),
                activity.name
            );
            let width = junie_tui::width(&label).min(area.right().saturating_sub(x));
            if width == 0 {
                break;
            }
            let row = Rect::new(x, area.y, width, 1);
            let key = ItemKey::num(u64::from(activity.id));
            let role = activity::state_role(activity.state);
            let patch = if self.route == Route::Activity
                && self.activities.current_id() == Some(activity.id)
            {
                StylePatch::new()
                    .set_fg(Role::Surface(Surface::Canvas))
                    .set_bg(role)
            } else {
                StylePatch::new().set_fg(role)
            };
            let style = ui.paint_patch(&patch);
            ui.paint_str(row, &label, style);
            ui.register_part(STRIP, PartRef::item(Part::ROW, key), row);
            x = x.saturating_add(width).saturating_add(1);
        }
        let faint = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint)));
        ui.paint_str(
            Rect::new(x, area.y, area.right().saturating_sub(x), 1),
            "0 home",
            faint,
        );
    }
    fn chrome(&self, ui: &mut Ui<'_>, area: Rect) {
        let header = Rect::new(area.x, area.y, area.width, 1);
        let brand = brand();
        let brand_width = brand
            .measure(ui, Constraints::loose(header.width, 1))
            .preferred
            .0;
        brand.draw(ui, Rect::new(header.x, header.y, brand_width, 1));
        let menu = menu_bar();
        let menu_width = menu
            .measure(
                ui,
                Constraints::loose(header.width.saturating_sub(brand_width), 1),
            )
            .preferred
            .0;
        menu.draw(
            ui,
            Rect::new(
                header.x.saturating_add(brand_width),
                header.y,
                menu_width,
                1,
            ),
            &self.menu,
        );
        let used = brand_width.saturating_add(menu_width).saturating_add(2);
        let identity = self.world.host.identity();
        let level = format!(
            "{} · {}×{}",
            ui.theme().capability.color.label(),
            area.width,
            area.height
        );
        let crumb = match self.route {
            Route::Home => self.world.cwd.clone(),
            Route::Plan => self.plan.as_ref().map_or_else(String::new, |plan| {
                format!("Plan · {}", plan.plan().title())
            }),
            Route::Activity => self.activities.current(&self.world).map_or_else(
                || "Activity".into(),
                |activity| format!("Activity · {}", activity.name),
            ),
        };
        let available = area.width.saturating_sub(used);
        let reserved = junie_tui::width(&identity)
            .saturating_add(junie_tui::width(&level))
            .saturating_add(junie_tui::width("? help"))
            .saturating_add(10);
        let crumb = junie_tui::truncate_middle(&crumb, available.saturating_sub(reserved));
        let mut items = Vec::new();
        if available.saturating_sub(reserved) >= 8 {
            items.push(
                StatusItem::new(&crumb)
                    .priority(7)
                    .tone(Role::Fg(FgStep::Secondary)),
            );
        }
        let scope = (self.route == Route::Home)
            .then_some(self.home.scope)
            .flatten()
            .map(|scope| format!("scope: {}", scope.label()));
        if let Some(scope) = scope.as_deref() {
            items.push(
                StatusItem::new(scope)
                    .priority(6)
                    .tone(Role::Fg(FgStep::Secondary)),
            );
        }
        items.extend(self.discovery_item());
        items.push(StatusItem::new(&identity).priority(9).tone(
            if self.world.host.env.sensitive() {
                Role::Warning
            } else {
                Role::Fg(FgStep::Secondary)
            },
        ));
        items.push(
            StatusItem::new(&level)
                .priority(1)
                .tone(Role::Fg(FgStep::Faint)),
        );
        items.push(
            StatusItem::new("? help")
                .priority(3)
                .key(ItemKey::text("help")),
        );
        header_bar(&items).draw(
            ui,
            Rect::new(area.x.saturating_add(used), area.y, available, 1),
        );
        self.strip_draw(
            ui,
            Rect::new(area.x, area.y.saturating_add(1), area.width, 1),
        );
        let footer = Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1);
        self.footer_draw(ui, footer);
    }
    fn discovery_item(&self) -> Option<StatusItem<'static>> {
        self.world.discovering().then(|| {
            // Discovery settles within the bounded fixture startup interval.
            let frame = usize::try_from(self.world.now_ms().div_euclid(80)).unwrap_or_default();
            StatusItem::new("discovering…")
                .spinner(frame)
                .priority(5)
                .tone(Role::Fg(FgStep::Secondary))
        })
    }
    fn footer_draw(&self, ui: &mut Ui<'_>, footer: Rect) {
        let hints = HintLayer::from_bindings(GLOBAL);
        let editing = match &self.overlay {
            Some(Overlay::Dialog(dialog)) => dialog.is_editing(),
            Some(Overlay::Gate(gate)) => gate.is_editing(),
            Some(Overlay::Clone(_) | Overlay::Actions(_)) => false,
            None => self.route == Route::Home && self.home.is_editing(),
        };
        let home_hints = (self.route == Route::Home
            && self.overlay.is_none()
            && !self.menu.is_open())
        .then(|| {
            HomeState::hints(
                &self.world,
                ui.state(home::ROWS)
                    .contains(junie_tui::StateFlags::FOCUSED),
            )
        });
        let actions_hints = matches!(self.overlay, Some(Overlay::Actions(_))).then(Actions::hints);
        let mut bar = HintBar::derived(FOOTER).screen(&hints);
        if let Some(mode_hints) = actions_hints.as_ref().or(home_hints.as_ref()) {
            bar = bar.mode(mode_hints);
        }
        bar.status_text(self.status.as_deref())
            .badge(editing.then_some("EDIT"))
            .centered(true)
            .draw(ui, footer);
    }
}
impl App {
    fn cadence(&self, cx: &Cx<'_>) -> u64 {
        if self.world.discovering() || cx.activation_feedback().is_some() {
            80
        } else {
            200
        }
    }
    fn synchronize_feedback(&mut self, cx: &mut Cx<'_>, clock: crate::clock::Clock) -> bool {
        let synchronized = u64::try_from(clock.now_ms).is_ok_and(|now| {
            cx.sync_feedback_time(junie_tui::SimulationMoment::from_millis(now))
                .is_ok()
        });
        if !synchronized {
            self.motion = Motion::Paused;
            self.world.clock.running = false;
            self.last_step_at = None;
            self.status = Some("Simulation animation paused: feedback clock mismatch.".into());
            self.status_until_ms = None;
        }
        synchronized
    }
    fn refresh_query_bindings(&mut self, cx: &Cx<'_>) {
        let enter = Chord::key(KeyCode::Enter);
        let space = Chord::key(KeyCode::Char(' '));
        self.keymap.remove(KeyPhase::Bubble, enter);
        self.keymap.remove(KeyPhase::Capture, space);
        if self.route == Route::Plan && self.overlay.is_none() && !self.menu.is_open() {
            self.keymap.add(KeyPhase::Bubble, enter, CONTINUE);
            if cx
                .state(plan::STEPS)
                .contains(junie_tui::StateFlags::FOCUSED)
            {
                self.keymap.add(KeyPhase::Capture, space, TOGGLE);
            }
        }
        let quit = Chord::key(KeyCode::Char('q'));
        self.keymap.remove(KeyPhase::Capture, quit);
        if too_small(cx.viewport()) {
            self.keymap.add(KeyPhase::Capture, quit, INTERRUPT);
        }
        for (character, command) in std::iter::once(&('0', HOME)).chain(ACTIVITY_SHORTCUTS) {
            let chord = Chord::key(KeyCode::Char(*character));
            self.keymap.remove(KeyPhase::Capture, chord);
            if self.route == Route::Activity {
                self.keymap.add(KeyPhase::Capture, chord, *command);
            }
        }
        for (character, command) in [('q', QUIT), ('?', HELP)] {
            let chord = Chord::key(KeyCode::Char(character));
            self.keymap.remove_before_typing(home::QUERY, chord);
            if self.home.query().is_empty() {
                self.keymap.add_before_typing(home::QUERY, chord, command);
            }
        }
        let escape = Chord::key(KeyCode::Esc);
        self.keymap.remove(KeyPhase::Capture, escape);
        if self.route == Route::Home
            && self.overlay.is_none()
            && !self.menu.is_open()
            && (self.home.scope.is_some() || !self.home.query().is_empty())
        {
            self.keymap.add(KeyPhase::Capture, escape, QUERY_ESCAPE);
        }
    }
    fn set_status(&mut self, message: String) {
        self.status = Some(message);
        self.status_until_ms = Some(self.world.now_ms().saturating_add(5_000));
    }
    fn plan_event(&mut self, event: plan::Event, cx: &mut Cx<'_>) {
        match event {
            plan::Event::Notice(message) => self.set_status(message),
            plan::Event::Gate => {
                if let Some(plan) = &self.plan {
                    let gate = PlanGate::new(&plan.review);
                    gate.open(cx);
                    self.overlay = Some(Overlay::Gate(gate));
                }
            }
        }
    }
    fn update_controls(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        // Closed overlays remain scrubbed until their final focus callbacks run.
        let retired = self
            .retired_overlay
            .take()
            .map_or_else(Response::ignored, |mut overlay| match &mut overlay {
                Overlay::Dialog(dialog) => dialog.poll(cx).0,
                Overlay::Gate(gate) => gate.poll(cx).0,
                Overlay::Clone(picker) => picker.update(cx).0,
                Overlay::Actions(picker) => picker.update(cx).0,
            });
        // Every retained owner receives lifecycle callbacks, regardless of route.
        // Product activation remains subject to modal, command and route policy.
        let (home_response, home_action) = self.home.update(&self.world, cx);
        let (plan_response, plan_event) = self
            .plan
            .as_mut()
            .map_or_else(|| (Response::ignored(), None), |plan| plan.update(cx));
        let activity_response = self.activities.update(&self.world, cx);
        let mut menu = menu_bar().update(cx, &mut self.menu);
        let brand = brand().update(cx);
        let items = [StatusItem::new("? help").key(ItemKey::text("help"))];
        let header = header_bar(&items).update(cx);
        let controls = retired | home_response | plan_response | activity_response;
        if too_small(cx.viewport()) {
            let overlay = self
                .overlay
                .as_mut()
                .map_or_else(Response::ignored, |overlay| match overlay {
                    Overlay::Dialog(dialog) => dialog.poll(cx).0,
                    Overlay::Gate(gate) => gate.poll(cx).0,
                    Overlay::Clone(picker) => picker.update(cx).0,
                    Overlay::Actions(picker) => picker.update(cx).0,
                });
            if cx.update_cause() == junie_tui::UpdateCause::Event && cx.command() == Some(INTERRUPT)
            {
                self.quit = true;
            }
            return controls
                | overlay
                | menu.erase()
                | brand.erase()
                | header.erase()
                | Response::consumed();
        }
        if self.overlay.is_some() {
            return controls
                | menu.erase()
                | brand.erase()
                | header.erase()
                | self.update_overlay(cx);
        }
        if let Some(command @ (INTERRUPT | QUIT)) = cx
            .command()
            .filter(|_| cx.update_cause() == junie_tui::UpdateCause::Event)
        {
            self.command(command, cx);
            return controls | menu.erase() | brand.erase() | header.erase() | Response::changed();
        }
        if let Some(MenuAction::Chosen(command)) = menu.take_action() {
            self.command(command, cx);
            return controls | menu.erase() | brand.erase() | header.erase();
        }
        if self.menu.is_open() {
            return controls | menu.erase() | brand.erase() | header.erase();
        }
        if let Some(command) = cx
            .command()
            .filter(|_| cx.update_cause() == junie_tui::UpdateCause::Event)
        {
            self.command(command, cx);
            return controls | menu.erase() | brand.erase() | header.erase() | Response::changed();
        }
        if brand.activated() {
            self.open_dialog(dialogs::Intent::About, cx);
            return controls | menu.erase() | brand.erase() | header.erase();
        }
        if matches!(header.action_ref(), Some(StatusAction::Chose(_))) {
            self.open_dialog(dialogs::Intent::Help, cx);
            return controls | menu.erase() | brand.erase() | header.erase();
        }
        let strip = self.strip_update(cx);
        match self.route {
            Route::Home => {
                if let Some(action) = home_action {
                    let destination = dispatch::invoke(&mut self.world, &action);
                    self.destination(destination, cx);
                }
            }
            Route::Plan => {
                if let Some(event) = plan_event {
                    self.plan_event(event, cx);
                }
            }
            Route::Activity => {}
        }
        controls | menu.erase() | brand.erase() | header.erase() | strip
    }
}
impl junie_tui::App for App {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut clock_response = Response::ignored();
        let synchronized = self.synchronize_feedback(cx, self.world.clock);
        if !synchronized {
            clock_response = Response::changed();
        }
        if synchronized && self.motion != Motion::Paused {
            let cadence = self.cadence(cx);
            let anchor = *self.last_step_at.get_or_insert(cx.now());
            let due = anchor.saturating_add(std::time::Duration::from_millis(cadence));
            if cx.update_cause() == junie_tui::UpdateCause::Tick && cx.now() >= due {
                // Validate the exact next clock before advancing any discovery state.
                let mut next_clock = self.world.clock;
                next_clock.advance(cadence as i64);
                if self.synchronize_feedback(cx, next_clock) {
                    let _ = self.world.tick(cadence as i64);
                    self.last_step_at = Some(cx.now());
                    if self
                        .status_until_ms
                        .is_some_and(|until| self.world.now_ms() >= until)
                    {
                        self.status = None;
                        self.status_until_ms = None;
                    }
                }
                clock_response = Response::changed();
            }
        }
        let plan_key = self.route == Route::Plan
            && self.overlay.is_none()
            && !self.menu.is_open()
            && !too_small(cx.viewport())
            && cx.activation_key().is_some();
        let response = self.update_controls(cx);
        if plan_key && response.is_consumed() {
            if matches!(self.overlay, Some(Overlay::Gate(_))) {
                cx.flash_activation(gate_dialog().input_id());
            } else if self.overlay.is_none()
                && !self.menu.is_open()
                && cx
                    .state(plan::STEPS)
                    .contains(junie_tui::StateFlags::FOCUSED)
                && let Some(step) = self.plan.as_ref().and_then(PlanState::selected)
            {
                cx.flash_activation_part(
                    plan::STEPS,
                    PartRef::item(Part::ROW, ItemKey::text(&step.id)),
                );
            }
        }
        self.refresh_query_bindings(cx);
        if self.motion != Motion::Paused
            && let Some(anchor) = self.last_step_at
        {
            cx.request_repaint_at(
                anchor.saturating_add(std::time::Duration::from_millis(self.cadence(cx))),
            );
        }
        clock_response | response
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        let base = ui.surface_style();
        ui.fill(area, base);
        if too_small(area) {
            TooSmall::new(Id::root("holla.too-small"), "holla❯")
                .minimum(MIN_WIDTH, MIN_HEIGHT)
                .draw(ui, area);
            return;
        }
        let body = Rect::new(
            area.x.saturating_add(1),
            area.y.saturating_add(2),
            area.width.saturating_sub(2),
            area.height.saturating_sub(4),
        );
        match self.route {
            Route::Home => self.home.draw(&self.world, ui, body),
            Route::Plan => {
                if let Some(plan) = &self.plan {
                    plan.draw(ui, body);
                }
            }
            Route::Activity => self.activities.draw(&self.world, ui, body),
        }
        self.chrome(ui, area);
        if let Some(overlay) = &self.overlay {
            match overlay {
                Overlay::Dialog(dialog) => dialog.draw(ui),
                Overlay::Clone(picker) => picker.draw(ui),
                Overlay::Actions(picker) => picker.draw(ui),
                Overlay::Gate(gate) => gate.draw(ui),
            }
        }
        // The update pass builds every prop the draw pass will render, so each
        // set of props keeps exactly one construction site (§13).
        let _ = gate_dialog();
    }
    fn should_quit(&self) -> bool {
        self.quit
    }
    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
    fn on_esc(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.route != Route::Home {
            self.route = Route::Home;
            cx.focus(home::QUERY);
            return Response::changed();
        }
        if self.home.scope.take().is_some() {
            return Response::changed();
        }
        if !self.home.query().is_empty() {
            self.home.clear_query();
            return Response::changed();
        }
        Response::ignored()
    }
}

fn too_small(area: Rect) -> bool {
    area.width < MIN_WIDTH || area.height < MIN_HEIGHT
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "Deterministic product routes and named fixtures must be present"
)]
mod tests {
    use super::*;
    use crate::sim::catalogue;
    use junie_tui::{Dialog, Theme};
    use junie_tui_testing::Harness;
    fn harness(app: App, theme: Theme, width: u16, height: u16) -> Harness<App> {
        let initial =
            junie_tui::SimulationMoment::from_millis(u64::try_from(app.fixture_time_ms()).unwrap());
        Harness::new_with_feedback_clock(
            app,
            theme,
            width,
            height,
            junie_tui::FeedbackClock::Simulation { initial },
        )
    }
    fn app(scenario: Scenario) -> Harness<App> {
        harness(
            App::for_scenario(scenario, Motion::Paused, 4_000),
            Theme::junie(),
            120,
            40,
        )
    }
    #[test]
    fn home_query_stays_armed_under_action_focus_with_shared_caret_and_paste() {
        let mut harness = app(Scenario::HardCases);
        assert!(harness.tab_to(home::ROWS));
        let _ = harness.type_str("git");
        assert_eq!(harness.focus(), Some(home::ROWS));
        assert_eq!(harness.app().home.query(), "git");
        assert!(harness.cursor().is_some());
        let _ = harness.paste(" q?0 ");
        assert_eq!(harness.app().home.query(), "git q?0 ");
        assert!(harness.app().overlay.is_none());
        let _ = harness.key(KeyCode::Backspace);
        assert_eq!(harness.app().home.query(), "git q?0");
        assert!(harness.app().home.rows(&harness.app().world).is_empty());
        assert_ne!(harness.focus(), Some(home::ROWS));
        assert!(harness.cursor().is_some());
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn empty_query_chrome_exceptions_and_modal_typing_do_not_leak() {
        for (character, title) in [('?', "Key reference"), ('q', "Quit holla?")] {
            let mut harness = app(Scenario::HardCases);
            assert!(harness.tab_to(home::ROWS));
            let _ = harness.key(KeyCode::Char(character));
            assert!(harness.text().contains(title));
            assert_eq!(harness.app().home.query(), "");
            let _ = harness.type_str("q?0text");
            assert_eq!(harness.app().home.query(), "");
            assert!(harness.app().overlay.is_some());
            let _ = harness.key(KeyCode::Esc);
            assert!(harness.app().overlay.is_none());
            let _ = harness.paste("q?");
            assert_eq!(harness.app().home.query(), "q?");
            assert!(harness.app().overlay.is_none());
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
    }
    #[test]
    fn unmatched_home_query_has_source_notice_and_no_empty_row_focus_stop() {
        let mut harness = app(Scenario::HardCases);
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.paste("  impossible-no-fixture-match  ");
        assert!(harness.app().home.rows(&harness.app().world).is_empty());
        assert!(
            harness
                .text()
                .contains("Nothing matches “impossible-no-fixture-match” here")
        );
        let revision = harness.app().effect_revision();
        let _ = harness.key(KeyCode::Down);
        assert_ne!(harness.focus(), Some(home::ROWS));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.app().home.query().is_empty());
        let _ = harness.key(KeyCode::Down);
        assert_eq!(harness.focus(), Some(home::ROWS));
        assert_eq!(harness.app().effect_revision(), revision);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn frame_seek_lands_on_discovered_state() {
        let settled = harness(
            App::for_scenario(Scenario::FirstUse, Motion::Paused, 4_000),
            Theme::junie(),
            120,
            40,
        );
        assert!(!settled.app().world.discovering());
        assert!(!settled.text().contains("discovering…"));
        let mut theme = Theme::junie();
        theme.design.motion.spinner_frames = &["a", "b", "c"];
        let early = harness(
            App::for_scenario(Scenario::FirstUse, Motion::Paused, 100),
            theme,
            120,
            40,
        );
        assert!(early.app().world.discovering());
        // Reference seek rounds100ms to160ms: virtual spinner frame2.
        assert!(
            early
                .text()
                .lines()
                .next()
                .unwrap()
                .contains("c discovering…")
        );
        assert!(early.diagnostics().is_empty(), "{:?}", early.diagnostics());
    }
    #[test]
    fn actions_picker_preserves_source_footer_height_and_query_isolation() {
        let mut h = app(Scenario::RustDirty);
        let _ = h.type_str("cargo build");
        let _ = h.ctrl('o');
        assert_eq!(
            h.layer_area(crate::screens::actions::PICKER)
                .map(|area| area.height),
            Some(12)
        );
        assert_eq!(h.cursor(), None);
        let text = h.text();
        let footer = text.lines().last().unwrap();
        assert!(
            footer.contains("↑↓ Move")
                && footer.contains("Enter Choose")
                && footer.contains("Esc Cancel")
        );
        assert!(!footer.contains("EDIT"));
        let _ = h.type_str("ignored");
        let _ = h.paste("ignored paste");
        assert_eq!(h.app().home.query(), "cargo build");
        assert!(h.text().contains("Set alias…"));
        let _ = h.key(KeyCode::Esc);
        assert!(h.app().overlay.is_none());
        assert_eq!(h.app().home.query(), "cargo build");
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
    #[test]
    fn home_hint_casing_preserves_lowercase_physical_shortcuts() {
        let mut h = app(Scenario::RustDirty);
        let text = h.text();
        let footer = text.lines().last().unwrap();
        assert!(footer.contains("Ctrl+S Scope"));
        assert!(footer.contains("q Quit"));
        let _ = h.ctrl('s');
        assert!(h.text().contains("scope: here"));
        assert!(h.tab_to(home::ROWS));
        let text = h.text();
        let footer = text.lines().last().unwrap();
        assert!(footer.contains("Ctrl+P Preview"));
        assert!(footer.contains("Ctrl+O Actions"));
        assert!(h.tab_to(home::QUERY));
        let _ = h.key(KeyCode::Char('q'));
        assert!(h.text().contains("Quit holla?"));
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
    #[test]
    fn home_product_hints_follow_query_rows_and_yield_to_modal_context() {
        let mut h = app(Scenario::RustDirty);
        assert!(h.text().lines().last().unwrap().contains("Run top match"));
        assert!(h.tab_to(home::ROWS));
        let footer = h.text().lines().last().unwrap().to_owned();
        assert!(footer.contains("Run") && footer.contains("Preview"));
        assert!(!footer.contains("Run top match"));
        let _ = h.key(KeyCode::F(1));
        assert!(!h.text().lines().last().unwrap().contains("Run top match"));
        assert!(!h.text().lines().last().unwrap().contains("Filter"));
        let _ = h.key(KeyCode::Esc);
        assert!(h.tab_to(home::QUERY));
        assert!(h.text().lines().last().unwrap().contains("Type Filter"));
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
    #[test]
    fn home_scope_is_projected_into_header_and_clears_with_escape() {
        let mut harness = app(Scenario::HardCases);
        let _ = harness.ctrl('s');
        let scope = harness.app().home.scope.unwrap();
        let label = format!("scope: {}", scope.label());
        assert!(harness.text().lines().next().unwrap().contains(&label));
        let _ = harness.key(KeyCode::Esc);
        assert!(!harness.text().lines().next().unwrap().contains("scope:"));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn home_escape_clears_scope_before_canonical_query() {
        let mut harness = app(Scenario::HardCases);
        assert!(harness.tab_to(home::ROWS));
        let _ = harness.type_str("git");
        let _ = harness.ctrl('s');
        assert!(harness.app().home.scope.is_some());
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.app().home.scope.is_none());
        assert_eq!(harness.app().home.query(), "git");
        let _ = harness.key(KeyCode::Esc);
        assert_eq!(harness.app().home.query(), "");
        let _ = harness.type_str("docker");
        assert_eq!(harness.app().home.query(), "docker");
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn home_navigation_uses_shared_boundaries_and_reveals_selected_rows() {
        let mut harness = harness(
            App::for_scenario(Scenario::HardCases, Motion::Paused, 4_000),
            Theme::junie(),
            120,
            20,
        );
        assert!(harness.tab_to(home::QUERY));
        let count = harness.app().home.rows(&harness.app().world).len();
        let _ = harness.key(KeyCode::Down);
        assert_eq!(harness.focus(), Some(home::ROWS));
        let first = harness
            .app()
            .home
            .selected(&harness.app().world)
            .unwrap()
            .id;
        let _ = harness.key(KeyCode::Up);
        assert_eq!(harness.focus(), Some(home::QUERY));
        let _ = harness.key(KeyCode::Down);
        for _ in 1..count {
            let _ = harness.key(KeyCode::Down);
        }
        assert_eq!(harness.focus(), Some(home::ROWS));
        let last = harness.app().home.selected(&harness.app().world).unwrap();
        assert_ne!(last.id, first);
        assert!(harness.text().contains(&last.title), "{}", harness.text());
        let _ = harness.key(KeyCode::Down);
        assert_ne!(harness.focus(), Some(home::ROWS));
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.key(KeyCode::Down);
        assert_eq!(
            harness
                .app()
                .home
                .selected(&harness.app().world)
                .unwrap()
                .id,
            first
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn activity_digits_are_route_scoped_and_cycle_from_home_starts_first() {
        let mut harness = app(Scenario::ActivitiesMulti);
        let ids: Vec<_> = activity::ordered(&harness.app().world)
            .iter()
            .map(|a| a.id)
            .collect();
        assert!(ids.len() >= 2);
        let _ = harness.type_str("12");
        assert_eq!(harness.app().home.query(), "12");
        let _ = harness.ctrl('a');
        assert_eq!(harness.app().activities.current_id(), Some(ids[0]));
        let _ = harness.key(KeyCode::Char('2'));
        assert_eq!(harness.app().activities.current_id(), Some(ids[1]));
        let _ = harness.key(KeyCode::Char('9'));
        assert_eq!(harness.app().status.as_deref(), Some("No activity 9"));
        let _ = harness.key(KeyCode::Char('0'));
        assert!(harness.app().route == Route::Home);
        let _ = harness.ctrl('a');
        assert_eq!(harness.app().activities.current_id(), Some(ids[0]));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn interrupt_quits_outside_modals_and_quit_chord_precedes_open_menu() {
        let mut harness = app(Scenario::HardCases);
        let _ = harness.key(KeyCode::F(1));
        let _ = harness.ctrl('c');
        assert!(!harness.app().quit);
        let _ = harness.key(KeyCode::Esc);
        let _ = harness.key(KeyCode::F(10));
        assert!(
            harness.app().menu.is_open(),
            "F10 must open the real menu before testing precedence"
        );
        let _ = harness.ctrl('q');
        assert!(harness.text().contains("Quit holla?"));
        assert!(!harness.app().quit);
        let mut harness = app(Scenario::HardCases);
        let _ = harness.key(KeyCode::F(10));
        let _ = harness.ctrl('c');
        assert!(harness.app().quit);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn menu_opens_before_publication_and_survives_focus_settlement() {
        let mut harness = app(Scenario::HardCases);
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str("git");
        let mut harness = harness.with_auto_draw(false);
        let _ = harness.key(KeyCode::F(10));
        assert!(harness.app().menu.is_open(), "event must open menu");
        harness.draw();
        assert!(
            harness.app().menu.is_open(),
            "publication must preserve menu"
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn too_small_surface_consumes_commands_and_quits_without_query_or_modal_effects() {
        for key in [KeyCode::Char('q'), KeyCode::Char('c')] {
            let mut harness = app(Scenario::HardCases);
            let _ = harness.type_str("git");
            let _ = harness.key(KeyCode::F(1));
            let revision = harness.app().effect_revision();
            let _ = harness.resize(40, 10);
            assert!(
                harness.diagnostics().is_empty(),
                "resize: {:?}",
                harness.diagnostics()
            );
            let _ = harness.key(KeyCode::F(1));
            assert!(!harness.app().quit);
            let _ = harness.resize(120, 40);
            assert!(harness.text().contains("Key reference"));
            assert!(
                harness.diagnostics().is_empty(),
                "restore: {:?}",
                harness.diagnostics()
            );
            let _ = harness.resize(40, 10);
            assert!(
                harness.diagnostics().is_empty(),
                "second resize: {:?}",
                harness.diagnostics()
            );
            if key == KeyCode::Char('c') {
                let _ = harness.ctrl('c');
            } else {
                let _ = harness.key(key);
            }
            assert!(harness.app().quit);
            assert_eq!(harness.app().home.query(), "git");
            assert_eq!(harness.app().effect_revision(), revision);
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
    }
    #[test]
    fn plan_physical_edges_leave_steps_without_changing_inclusion_or_effects() {
        let mut harness = app(Scenario::DockerCleanup);
        let _ = harness.type_str("docker system prune");
        let _ = harness.key(KeyCode::Enter);
        assert!(harness.app().route == Route::Plan);
        assert!(harness.tab_to(plan::STEPS));
        let _ = harness.key(KeyCode::Home);
        let included = harness.app().plan.as_ref().unwrap().plan().included_count();
        let revision = harness.app().effect_revision();
        let _ = harness.key(KeyCode::Up);
        assert_ne!(harness.focus(), Some(plan::STEPS));
        assert!(harness.tab_to(plan::STEPS));
        let _ = harness.key(KeyCode::End);
        let _ = harness.key(KeyCode::Down);
        assert_ne!(harness.focus(), Some(plan::STEPS));
        assert_eq!(
            harness.app().plan.as_ref().unwrap().plan().included_count(),
            included
        );
        assert_eq!(harness.app().effect_revision(), revision);
        assert!(harness.app().overlay.is_none());
        assert!(harness.activation_feedback().is_none());
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn plan_space_is_focus_bound_and_preserves_chrome_activation() {
        let mut harness = app(Scenario::DockerCleanup);
        let _ = harness.type_str("docker system prune");
        let _ = harness.key(KeyCode::Enter);
        let index = harness
            .app()
            .plan
            .as_ref()
            .unwrap()
            .plan()
            .steps()
            .iter()
            .position(|step| step.optional)
            .unwrap();
        for _ in 0..index {
            let _ = harness.key(KeyCode::Down);
        }
        let included = harness.app().plan.as_ref().unwrap().plan().included_count();
        assert!(harness.tab_to(MENU));
        let _ = harness.key(KeyCode::Char(' '));
        assert!(harness.app().menu.is_open());
        assert_eq!(
            harness.app().plan.as_ref().unwrap().plan().included_count(),
            included
        );
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.tab_to(MENU));
        let _ = harness.key(KeyCode::Enter);
        assert!(harness.app().menu.is_open());
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.tab_to(plan::STEPS));
        let _ = harness.key(KeyCode::Char(' '));
        assert!(harness.app().plan.as_ref().unwrap().plan().included_count() < included);
        assert!(harness.app().overlay.is_none());
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn plan_keyboard_feedback_uses_post_request_owner_and_double_click_cannot_open_gate() {
        let mut harness = harness(
            App::for_scenario(Scenario::DockerCleanup, Motion::Full, 4_000),
            Theme::junie(),
            120,
            40,
        );
        let _ = harness.type_str("docker system prune");
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.activation_feedback().is_none(),
            "Home query submission does not flash"
        );
        let step = harness
            .app()
            .plan
            .as_ref()
            .unwrap()
            .selected()
            .unwrap()
            .id
            .clone();
        let part = PartRef::item(Part::ROW, ItemKey::text(&step));
        let _ = harness.click_part(plan::STEPS, part);
        let _ = harness.click_part(plan::STEPS, part);
        assert!(harness.app().overlay.is_none());
        let _ = harness.advance(std::time::Duration::from_millis(80));
        assert_eq!(
            harness.activation_feedback().unwrap().remaining,
            std::time::Duration::from_millis(60)
        );
        let _ = harness.key(KeyCode::Char(' '));
        let feedback = harness.activation_feedback().unwrap();
        assert_eq!((feedback.owner, feedback.part), (plan::STEPS, part));
        assert_eq!(feedback.remaining, std::time::Duration::from_millis(140));
        let _ = harness.key(KeyCode::Enter);
        let gate = Dialog::new(crate::screens::plan_gate::GATE);
        assert_eq!(
            harness.activation_feedback().unwrap().owner,
            gate.input_id()
        );
        assert_eq!(harness.focus(), Some(gate.input_id()));
        let _ = harness.advance(std::time::Duration::from_millis(80));
        let _ = harness.advance(std::time::Duration::from_millis(80));
        assert!(harness.activation_feedback().is_none());
        let phrase = harness
            .app()
            .plan
            .as_ref()
            .unwrap()
            .plan()
            .phrase()
            .to_owned();
        let _ = harness.type_str(&phrase);
        let _ = harness.key(KeyCode::Enter);
        assert_eq!(harness.app().effect_revision(), 0, "editor Enter only arms");
        assert!(
            harness.activation_feedback().is_none(),
            "modal editing has no keyboard flash"
        );
        assert!(harness.tab_to(gate.action_id(1)));
        let _ = harness.key(KeyCode::Char(' '));
        assert_eq!(harness.app().effect_revision(), 1);
        assert!(harness.app().plan.as_ref().unwrap().plan().ran());
        assert!(
            harness.activation_feedback().is_none(),
            "modal confirmation preserves source feedback policy"
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn every_real_app_scenario_renders_purely_with_persistent_identity() {
        for scenario in Scenario::ALL {
            let mut harness = app(scenario);
            assert!(harness.text().contains("holla❯"));
            assert!(harness.text().contains(&harness.app().world.host.name));
            let catalogue = catalogue::catalogue(&harness.app().world);
            let time = harness.app().world.now_ms();
            let text = harness.text();
            harness.draw();
            harness.draw();
            assert_eq!(harness.text(), text);
            assert_eq!(catalogue::catalogue(&harness.app().world), catalogue);
            assert_eq!(harness.app().world.now_ms(), time);
        }
    }
    #[test]
    fn help_escape_and_quit_use_actual_dialog_routes() {
        let mut harness = app(Scenario::FirstUse);
        let _ = harness.key(KeyCode::F(1));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(matches!(harness.app().overlay, Some(Overlay::Dialog(_))));
        assert!(harness.text().contains("Key reference"));
        let _ = harness.key(KeyCode::Esc);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(harness.app().overlay.is_none());
        let _ = harness.ctrl('q');
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(harness.text().contains("Quit holla?"));
        let _ = harness.click_id(Dialog::new(dialogs::DIALOG).action_id(1));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(harness.app().quit);
    }
    #[test]
    fn filtered_cleanup_enters_review_and_exact_gate_applies_once() {
        let mut harness = app(Scenario::DockerCleanup);
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str("docker system prune");
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(matches!(harness.app().route, Route::Plan));
        assert_eq!(harness.app().world.effect_revision, 0);
        assert!(harness.tab_to(plan::STEPS));
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(matches!(harness.app().overlay, Some(Overlay::Gate(_))));
        let phrase = harness
            .app()
            .plan
            .as_ref()
            .unwrap()
            .plan()
            .phrase()
            .to_owned();
        let _ = harness.type_str(&phrase);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        let _ = harness.click_id(Dialog::new(crate::screens::plan_gate::GATE).action_id(1));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert_eq!(harness.app().world.effect_revision, 1);
        assert!(harness.app().plan.as_ref().unwrap().plan().ran());
        assert!(harness.app().overlay.is_none());
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert_eq!(harness.app().world.effect_revision, 1);
    }
    #[test]
    fn clone_picker_is_reachable_from_the_real_action_row() {
        let mut harness = app(Scenario::RustDirty);
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str("gh repo clone");
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(matches!(harness.app().overlay, Some(Overlay::Clone(_))));
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(matches!(harness.app().overlay, Some(Overlay::Dialog(_))));
        assert!(harness.text().contains("Primary branch"));
    }
    #[test]
    fn action_picker_refuses_memory_changes_after_context_moves() {
        let mut harness = app(Scenario::DockerCleanup);
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str("docker system prune");
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        let _ = harness.ctrl('o');
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(matches!(harness.app().overlay, Some(Overlay::Actions(_))));
        let pins = harness.app().world.memory.pins.clone();
        harness.app_mut().world.cwd = "/different-folder".into();
        let _ = harness.type_str("Pin here");
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(harness.app().overlay.is_none());
        assert_eq!(harness.app().world.memory.pins, pins);
        assert!(harness.app().status.as_deref().unwrap().contains("changed"));
    }
    #[test]
    fn ssh_preview_shows_literal_resolution_and_refuses_changed_target() {
        for changed in [false, true] {
            let mut harness = app(Scenario::RemoteHost);
            let host = harness.app().world.ssh[0].clone();
            assert!(harness.tab_to(home::QUERY));
            let _ = harness.type_str(&format!("ssh {}", host.alias));
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
            let _ = harness.key(KeyCode::Enter);
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
            assert!(matches!(harness.app().overlay, Some(Overlay::Dialog(_))));
            let text = harness.text();
            for expected in [
                &host.host_name,
                &host.user,
                &host.chain(),
                "Host key policy",
                "Multiplexing",
            ] {
                assert!(text.contains(expected), "missing {expected}");
            }
            if changed {
                harness.app_mut().world.ssh[0].host_name = "different.example".into();
            }
            let _ = harness.click_id(Dialog::new(dialogs::DIALOG).action_id(1));
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
            let status = harness.app().status.as_deref().unwrap();
            if changed {
                assert!(status.contains("SSH target changed"));
            } else {
                assert!(status.contains(&format!("Would run: ssh {}", host.alias)));
            }
            assert_eq!(harness.app().world.effect_revision, 0);
        }
    }
    #[test]
    fn routes_and_gate_deliver_owner_lifecycle() {
        let mut harness = app(Scenario::DockerCleanup);
        assert!(harness.diagnostics().is_empty());
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str("docker system prune");
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(
            harness.diagnostics().is_empty(),
            "plan {:?}",
            harness.diagnostics()
        );
        assert!(harness.tab_to(plan::STEPS));
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(
            harness.diagnostics().is_empty(),
            "gate {:?}",
            harness.diagnostics()
        );
        let _ = harness.key(KeyCode::Esc);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
        assert!(
            harness.diagnostics().is_empty(),
            "cancel {:?}",
            harness.diagnostics()
        );
    }
    #[test]
    fn activity_switching_and_home_keep_viewport_lifecycle_clean() {
        let mut harness = app(Scenario::ActivitiesMulti);
        for _ in 0..3 {
            let _ = harness.ctrl('a');
            assert!(matches!(harness.app().route, Route::Activity));
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
        let _ = harness.key(KeyCode::Char('0'));
        assert!(matches!(harness.app().route, Route::Home));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }

    #[test]
    fn coalesced_deadlines_preserve_fixed_steps_and_pause() {
        for motion in [Motion::Full, Motion::Reduced, Motion::Paused] {
            let mut harness = harness(
                App::for_scenario(Scenario::FirstUse, motion, 0),
                Theme::junie(),
                120,
                40,
            );
            harness.ticks(20);
            assert_eq!(harness.app().world.now_ms(), 0);
            let _ = harness.advance(std::time::Duration::from_millis(1_800));
            let step = if motion == Motion::Paused { 0 } else { 80 };
            assert_eq!(harness.app().world.now_ms(), step);
            harness.ticks(20);
            harness.draw();
            assert_eq!(harness.app().world.now_ms(), step);
            let _ = harness.advance(std::time::Duration::from_millis(80));
            assert_eq!(harness.app().world.now_ms(), step * 2);
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
    }
    #[test]
    fn pointer_feedback_changes_cadence_from_last_wake_and_expires_in_fixture_time() {
        for input_at in [5_u64, 100] {
            let mut harness = harness(
                App::for_scenario(Scenario::HardCases, Motion::Full, 4_000),
                Theme::junie(),
                120,
                40,
            );
            let _ = harness.advance(std::time::Duration::from_millis(input_at));
            let _ = harness.click_id(BRAND);
            assert_eq!(harness.app().fixture_time_ms(), 4_000);
            let _ = harness.advance(std::time::Duration::from_millis(
                80_u64.saturating_sub(input_at),
            ));
            assert_eq!(harness.app().fixture_time_ms(), 4_080);
            assert_eq!(
                harness.activation_feedback().unwrap().remaining,
                std::time::Duration::from_millis(60)
            );
            harness.ticks(5);
            let _ = harness.resize(121, 40);
            assert_eq!(harness.app().fixture_time_ms(), 4_080);
            let _ = harness.advance(std::time::Duration::from_millis(80));
            assert_eq!(harness.app().fixture_time_ms(), 4_160);
            assert!(harness.activation_feedback().is_none());
            let _ = harness.advance(std::time::Duration::from_millis(199));
            assert_eq!(harness.app().fixture_time_ms(), 4_160);
            let _ = harness.advance(std::time::Duration::from_millis(1));
            assert_eq!(harness.app().fixture_time_ms(), 4_360);
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
    }
    #[test]
    fn delayed_pointer_feedback_coalesces_and_paused_feedback_keeps_fixture_epoch() {
        for motion in [Motion::Full, Motion::Paused] {
            let mut harness = harness(
                App::for_scenario(Scenario::HardCases, motion, 4_000),
                Theme::junie(),
                120,
                40,
            );
            let _ = harness.advance(std::time::Duration::from_millis(5));
            let _ = harness.click_id(BRAND);
            let _ = harness.advance(std::time::Duration::from_millis(1_800));
            let step = if motion == Motion::Paused { 0 } else { 80 };
            assert_eq!(harness.app().fixture_time_ms(), 4_000 + step);
            assert_eq!(
                harness.activation_feedback().unwrap().remaining,
                std::time::Duration::from_millis(if motion == Motion::Paused { 140 } else { 60 })
            );
            harness.ticks(10);
            assert_eq!(harness.app().fixture_time_ms(), 4_000 + step);
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
    }
    #[test]
    fn feedback_clock_mismatch_and_backwards_epoch_refuse_all_fixture_progress() {
        let snapshot = |app: &App| {
            (
                app.fixture_time_ms(),
                app.effect_revision(),
                app.world
                    .discovery
                    .iter()
                    .map(|mark| (mark.domain, mark.at_ms, mark.fails, mark.done, mark.failed))
                    .collect::<Vec<_>>(),
                catalogue::catalogue(&app.world),
            )
        };
        for clock in [
            junie_tui::FeedbackClock::Elapsed,
            junie_tui::FeedbackClock::Simulation {
                initial: junie_tui::SimulationMoment::from_millis(5_000),
            },
        ] {
            let app = App::for_scenario(Scenario::HardCases, Motion::Full, 4_000);
            let expected = snapshot(&app);
            let mut harness = Harness::new_with_feedback_clock(app, Theme::junie(), 120, 40, clock);
            assert_eq!(snapshot(harness.app()), expected);
            assert!(!harness.app().world.clock.running);
            assert_eq!(harness.app().motion, Motion::Paused);
            assert!(harness.text().contains("feedback clock mismatch"));
            let _ = harness.advance(std::time::Duration::from_millis(1_800));
            assert_eq!(snapshot(harness.app()), expected);
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
    }

    #[test]
    fn modal_status_ages_only_with_admitted_virtual_steps() {
        let mut harness = harness(
            App::for_scenario(Scenario::FirstUse, Motion::Full, 4_000),
            Theme::junie(),
            120,
            40,
        );
        let _ = harness.ctrl('s');
        let _ = harness.key(KeyCode::F(1));
        for _ in 0..24 {
            let _ = harness.advance(std::time::Duration::from_millis(200));
            assert!(harness.app().status.is_some());
        }
        let _ = harness.advance(std::time::Duration::from_millis(200));
        assert_eq!(harness.app().world.now_ms(), 9_000);
        assert!(harness.app().status.is_none());
        assert!(harness.app().overlay.is_some());
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }

    #[test]
    fn early_input_and_unrelated_ticks_cannot_admit_simulation_time() {
        let mut harness = harness(
            App::for_scenario(Scenario::FirstUse, Motion::Full, 0),
            Theme::junie(),
            120,
            40,
        );
        for _ in 0..10 {
            let _ = harness.advance(std::time::Duration::from_micros(600));
            let _ = harness.tick();
        }
        let _ = harness.key(KeyCode::F(1));
        assert_eq!(harness.app().world.now_ms(), 0);
        let _ = harness.advance(std::time::Duration::from_millis(74));
        assert_eq!(harness.app().world.now_ms(), 80);
    }
    #[test]
    fn footer_status_survives_focused_hints_and_edit_badge_uses_top_context() {
        let mut harness = app(Scenario::DockerCleanup);
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str("docker system prune");
        let _ = harness.ctrl('s');
        assert!(harness.row(39).contains("EDIT"));
        assert!(harness.row(39).contains("Scope: here"));
        let _ = harness.key(KeyCode::F(1));
        assert!(!harness.row(39).contains("EDIT"));
        assert!(harness.row(39).contains("Scope: here"));
        let _ = harness.key(KeyCode::Esc);
        assert!(harness.row(39).contains("EDIT"));
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );

        let mut harness = app(Scenario::DockerCleanup);
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str("docker system prune");
        let _ = harness.key(KeyCode::Enter);
        assert!(harness.tab_to(plan::STEPS));
        let _ = harness.key(KeyCode::Enter);
        assert!(harness.row(39).contains("EDIT"));
        let _ = harness.key(KeyCode::Enter);
        assert!(!harness.row(39).contains("EDIT"));
        assert_eq!(harness.app().world.effect_revision, 0);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}

#[cfg(test)]
mod historical_tests;
