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
    ActionKey, Binding, Brand, Chord, Constraints, Cx, FgStep, Focusability, FrameRead, HintBar,
    HintLayer, Id, Intent, ItemKey, KeyCode, KeyMap, KeyPhase, Menu, MenuAction, MenuBar, MenuItem,
    MenuState, Part, PartRef, Phase, Rect, Response, Role, StatusAction, StatusBar, StatusItem,
    StylePatch, Surface, TooSmall, Ui,
};

const BRAND: Id = Id::root("holla.brand");
const MENU: Id = Id::root("holla.menu");
const HEADER: Id = Id::root("holla.header");
const STRIP: Id = Id::root("holla.activities");
const FOOTER: Id = Id::root("holla.footer");
const HELP: ActionKey = ActionKey::application("holla.help");
const ABOUT: ActionKey = ActionKey::application("holla.about");
const QUIT: ActionKey = ActionKey::application("holla.quit");
const HOME: ActionKey = ActionKey::application("holla.home");
const NEXT_ACTIVITY: ActionKey = ActionKey::application("holla.next-activity");
const SCOPE: ActionKey = ActionKey::application("holla.scope");
const PREVIEW: ActionKey = ActionKey::application("holla.preview");
const ACTIONS: ActionKey = ActionKey::application("holla.actions");
const TOGGLE: ActionKey = ActionKey::application("holla.plan-toggle");
const OPEN_MENU: ActionKey = ActionKey::application("holla.open-menu");
const RESULTS: ActionKey = ActionKey::application("holla.results");
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Route {
    Home,
    Plan,
    Activity,
}
enum Overlay {
    Dialog(ProductDialog),
    Clone(ClonePicker),
    Actions(Actions),
    Gate(PlanGate),
}

/// Holla's deterministic launcher application. External commands remain fixture data.
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
    applied_elapsed_ms: u128,
    quit: bool,
}
impl App {
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
            .bind(KeyPhase::Capture, Chord::key(KeyCode::Char(' ')), TOGGLE)
            .bind(KeyPhase::Bubble, Chord::key(KeyCode::Down), RESULTS)
            .bind(KeyPhase::Capture, Chord::key(KeyCode::Char('0')), HOME);
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
            applied_elapsed_ms: 0,
            quit: false,
        }
    }
    fn open_dialog(&mut self, intent: dialogs::Intent, cx: &mut Cx<'_>) {
        let dialog = ProductDialog::new(intent, &self.world);
        dialog.open(cx);
        self.overlay = Some(Overlay::Dialog(dialog));
    }
    fn destination(&mut self, destination: Destination, cx: &mut Cx<'_>) {
        match destination {
            Destination::Notice(message) => self.set_status(message),
            Destination::Preview(action) => self.open_dialog(dialogs::Intent::Preview(action), cx),
            Destination::Trust { action, file } => {
                self.open_dialog(dialogs::Intent::Trust { action, file }, cx)
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
    fn command(&mut self, command: ActionKey, cx: &mut Cx<'_>) {
        match command {
            HELP => self.open_dialog(dialogs::Intent::Help, cx),
            ABOUT => self.open_dialog(dialogs::Intent::About, cx),
            QUIT => self.open_dialog(dialogs::Intent::Quit, cx),
            HOME => {
                self.route = Route::Home;
                cx.focus(home::QUERY);
            }
            OPEN_MENU => {
                let _ = MenuBar::new(MENU, MENUS).open_menu(cx, &mut self.menu, 0);
            }
            NEXT_ACTIVITY => {
                let ordered = activity::ordered(&self.world);
                let next = ordered
                    .iter()
                    .position(|activity| Some(activity.id) == self.activities.current_id())
                    .and_then(|index| ordered.get(index.saturating_add(1)))
                    .or_else(|| ordered.first())
                    .map(|activity| activity.id);
                if let Some(id) = next {
                    self.activities.show(id);
                    self.route = Route::Activity;
                    cx.focus(activity::output_id(id));
                }
            }
            SCOPE if self.route == Route::Home => {
                self.home.scope = match self.home.scope {
                    None => Some(Scope::Here),
                    Some(Scope::Here) => Some(Scope::Project),
                    Some(Scope::Project) => Some(Scope::Workspace),
                    Some(Scope::Workspace) => Some(Scope::Host),
                    Some(Scope::Host) => Some(Scope::Personal),
                    Some(Scope::Personal) => None,
                };
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
            TOGGLE if self.route == Route::Plan => {
                if let Some(plan) = &mut self.plan {
                    let message = plan.toggle_selected();
                    self.set_status(message);
                }
            }
            RESULTS if self.route == Route::Home => cx.focus(home::ROWS),
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
                            self.destination(destination, cx)
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
                phase: Phase::Click,
                part: PartRef {
                    item: Some(key), ..
                },
                ..
            } = intent
            {
                if let Some(id) = self
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
        let brand = Brand::new(BRAND, "holla❯").clickable(true);
        let brand_width = brand
            .measure(ui, Constraints::loose(header.width, 1))
            .preferred
            .0;
        brand.draw(ui, Rect::new(header.x, header.y, brand_width, 1));
        let menu = MenuBar::new(MENU, MENUS);
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
        StatusBar::new(HEADER).right(&items).draw(
            ui,
            Rect::new(area.x.saturating_add(used), area.y, available, 1),
        );
        self.strip_draw(
            ui,
            Rect::new(area.x, area.y.saturating_add(1), area.width, 1),
        );
        let footer = Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1);
        let mut hints = HintLayer::from_bindings(GLOBAL);
        hints.status = self.status.clone().map(Into::into);
        HintBar::derived(FOOTER).screen(&hints).draw(ui, footer);
    }
}
impl App {
    fn set_status(&mut self, message: String) {
        self.status = Some(message);
        self.status_until_ms = Some(self.world.now_ms().saturating_add(5_000));
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
        let mut menu = MenuBar::new(MENU, MENUS).update(cx, &mut self.menu);
        let brand = Brand::new(BRAND, "holla❯").clickable(true).update(cx);
        let items = [StatusItem::new("? help").key(ItemKey::text("help"))];
        let header = StatusBar::new(HEADER).right(&items).update(cx);
        let controls = retired | home_response | plan_response | activity_response;
        if self.overlay.is_some() {
            return controls
                | menu.erase()
                | brand.erase()
                | header.erase()
                | self.update_overlay(cx);
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
            Route::Plan => match plan_event {
                Some(plan::Event::Notice(message)) => self.set_status(message),
                Some(plan::Event::Gate) => {
                    if let Some(plan) = &self.plan {
                        let gate = PlanGate::new(plan.plan());
                        gate.open(cx);
                        self.overlay = Some(Overlay::Gate(gate));
                    }
                }
                None => {}
            },
            Route::Activity => {}
        }
        controls | menu.erase() | brand.erase() | header.erase() | strip
    }
}
impl junie_tui::App for App {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut clock_response = Response::ignored();
        if self.motion != Motion::Paused {
            let elapsed_ms = cx.now().as_duration().as_millis();
            let delta = elapsed_ms.saturating_sub(self.applied_elapsed_ms);
            self.applied_elapsed_ms = elapsed_ms;
            if delta > 0 {
                let interval = i64::try_from(delta).unwrap_or(i64::MAX);
                let _ = self.world.tick(interval);
                clock_response = Response::changed();
            }
            if self
                .status_until_ms
                .is_some_and(|until| self.world.now_ms() >= until)
            {
                self.status = None;
                self.status_until_ms = None;
                clock_response = Response::changed();
            }
        }
        let response = self.update_controls(cx);
        if self.motion != Motion::Paused {
            let cadence = if self.world.discovering() { 80 } else { 200 };
            let delay = self.status_until_ms.map_or(cadence, |until| {
                cadence.min(u64::try_from(until.saturating_sub(self.world.now_ms())).unwrap_or(0))
            });
            cx.request_repaint_at(
                cx.now()
                    .saturating_add(std::time::Duration::from_millis(delay)),
            );
        }
        clock_response | response
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        let base = ui.surface_style();
        ui.fill(area, base);
        if area.width < 72 || area.height < 20 {
            TooSmall::new(Id::root("holla.too-small"), "holla")
                .minimum(72, 20)
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
                    plan.draw(ui, body)
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

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "Deterministic product routes and named fixtures must be present"
)]
mod tests {
    use super::*;
    use crate::sim::catalogue;
    use junie_tui::{Dialog, Theme};
    use junie_tui_testing::Harness;
    fn app(scenario: Scenario) -> Harness<App> {
        Harness::new(
            App::for_scenario(scenario, Motion::Paused, 4_000),
            Theme::junie(),
            120,
            40,
        )
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
    fn elapsed_time_advances_once_and_paused_frames_stay_exact() {
        for motion in [Motion::Full, Motion::Reduced, Motion::Paused] {
            let mut harness = Harness::new(
                App::for_scenario(Scenario::FirstUse, motion, 0),
                Theme::junie(),
                120,
                40,
            );
            harness.ticks(20);
            assert_eq!(harness.app().world.now_ms(), 0);
            let _ = harness.advance(std::time::Duration::from_millis(1_800));
            let expected = if motion == Motion::Paused { 0 } else { 1_800 };
            assert_eq!(harness.app().world.now_ms(), expected);
            harness.ticks(20);
            harness.draw();
            assert_eq!(harness.app().world.now_ms(), expected);
            assert!(
                harness.diagnostics().is_empty(),
                "{:?}",
                harness.diagnostics()
            );
        }
    }

    #[test]
    fn status_and_discovery_age_while_modal_owns_focus() {
        let mut harness = Harness::new(
            App::for_scenario(Scenario::FirstUse, Motion::Full, 0),
            Theme::junie(),
            120,
            40,
        );
        let _ = harness.ctrl('s');
        assert!(harness.app().status.is_some());
        let _ = harness.key(KeyCode::F(1));
        assert!(harness.app().overlay.is_some());
        let _ = harness.advance(std::time::Duration::from_millis(4_999));
        assert!(harness.app().status.is_some());
        assert!(!harness.app().world.discovering());
        let _ = harness.advance(std::time::Duration::from_millis(1));
        assert_eq!(harness.app().world.now_ms(), 5_000);
        assert!(harness.app().status.is_none());
        assert!(harness.app().overlay.is_some());
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }

    #[test]
    fn fractional_elapsed_input_does_not_discard_clock_remainder() {
        let mut harness = Harness::new(
            App::for_scenario(Scenario::FirstUse, Motion::Full, 0),
            Theme::junie(),
            120,
            40,
        );
        for _ in 0..10 {
            let _ = harness.advance(std::time::Duration::from_micros(600));
            let _ = harness.tick();
        }
        assert_eq!(harness.app().world.now_ms(), 6);
    }
}
