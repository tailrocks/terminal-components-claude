use termrock::author::VerticalWindow;
use termrock::core::keymap::{Nav, NavKeys};
use termrock::feedback::ChordText;
use termrock::{
    ActionKey, Binding, BindingState, Bindings, Button, Chord, Constraints, Cx, Dismiss, Family,
    Focusability, FrameRead, HelpCmd, HelpOverlay, HelpOverlayState, HelpSection, Id, Intent,
    ItemKey, Key, KeyCode, KeyMap, KeyModifiers, KeyPhase, LayerSize, Panel, PanelKind, Part, Rect,
    Response, ScrollRegion, ScrollState, ScrollbarGutter, Select, SelectState, StateFlags, Tabs,
    TabsState, Ui, Variant, width, wrap,
};

use super::{Page, PageStatus, PageUpdate, frame_with_body_rows};

const PAGE_TABS: Id = Id::root("showcase_ui::pages::help_keys.page_tabs");
const PLAY_SCROLL: Id = Id::root("showcase_ui::pages::help_keys.play_scroll");
const OPTIONS_SCROLL: Id = Id::root("showcase_ui::pages::help_keys.options_scroll");
const STATES_SCROLL: Id = Id::root("showcase_ui::pages::help_keys.states_scroll");
const OPEN_HELP: Id = Id::root("showcase_ui::pages::help_keys.open_help");
const HELP: Id = Id::root("showcase_ui::pages::help_keys.help");
const HELP_COMMANDS: Id = Id::root("showcase_ui::pages::help_keys.help_commands");
const HELP_KEYS_NOOP: ActionKey = ActionKey::application("showcase.help_keys.noop");
const SCROLL_PAGE_UP: ActionKey = ActionKey::application("showcase.help_keys.page_up");
const SCROLL_PAGE_DOWN: ActionKey = ActionKey::application("showcase.help_keys.page_down");
const SCROLL_HOME: ActionKey = ActionKey::application("showcase.help_keys.home");
const SCROLL_END: ActionKey = ActionKey::application("showcase.help_keys.end");
const SCROLL_VIM_UP: ActionKey = ActionKey::application("showcase.help_keys.vim_up");
const SCROLL_VIM_DOWN: ActionKey = ActionKey::application("showcase.help_keys.vim_down");
const SCROLL_VIM_HOME: ActionKey = ActionKey::application("showcase.help_keys.vim_home");
const SCROLL_VIM_END: ActionKey = ActionKey::application("showcase.help_keys.vim_end");
const MODIFIED_NAV_UP: ActionKey = ActionKey::application("showcase.help_keys.modified_up");
const MODIFIED_NAV_DOWN: ActionKey = ActionKey::application("showcase.help_keys.modified_down");
const MODIFIED_NAV_PAGE_UP: ActionKey =
    ActionKey::application("showcase.help_keys.modified_page_up");
const MODIFIED_NAV_PAGE_DOWN: ActionKey =
    ActionKey::application("showcase.help_keys.modified_page_down");
const MODIFIED_NAV_HOME: ActionKey = ActionKey::application("showcase.help_keys.modified_home");
const MODIFIED_NAV_END: ActionKey = ActionKey::application("showcase.help_keys.modified_end");
const REFERENCE_PANEL: Id = Id::root("showcase_ui::pages::help_keys.reference_panel");
const HELP_PANEL: Id = Id::root("showcase_ui::pages::help_keys.help_panel");

const PAGE_SECTIONS: &[&str] = &["Playground", "Options", "States"];
const PAGE_BODY_START_ROW: u16 = 5;
const REFERENCE_PANEL_WIDTH: u16 = 58;
const WIDE_LAYOUT: u16 = 90;
const OPTION_COUNT: usize = 8;
const OPTION_ROWS: usize = OPTION_COUNT;
const STATE_ROWS: usize = 3;

const OPTION_IDS: [Id; OPTION_COUNT] = [
    Id::root("showcase_ui::pages::help_keys.option.width"),
    Id::root("showcase_ui::pages::help_keys.option.key_max_share"),
    Id::root("showcase_ui::pages::help_keys.option.gap"),
    Id::root("showcase_ui::pages::help_keys.option.title"),
    Id::root("showcase_ui::pages::help_keys.option.max_percent"),
    Id::root("showcase_ui::pages::help_keys.option.nav"),
    Id::root("showcase_ui::pages::help_keys.option.keymap"),
    Id::root("showcase_ui::pages::help_keys.option.dismiss"),
];

const OPTION_KEYS: [&str; OPTION_COUNT] = [
    "helpkeys.width",
    "helpkeys.key_max_share",
    "helpkeys.gap",
    "helpkeys.title",
    "helpkeys.max_percent",
    "helpkeys.nav",
    "helpkeys.keymap",
    "helpkeys.dismiss",
];

const OPTION_LABELS: [&str; OPTION_COUNT] = [
    "Reference width",
    "Key column share",
    "Column gap",
    "Overlay title",
    "Overlay size",
    "Scroll keys",
    "Close keys",
    "Dismiss",
];

const OPTION_ITEMS: [&[&str]; OPTION_COUNT] = [
    &["full", "40", "28"],
    &["40", "25", "60"],
    &["3", "6"],
    &["default", "custom"],
    &["80", "60", "100"],
    &["default", "vim"],
    &["default", "f1"],
    &["light", "escape"],
];

const TABS_HINTS: &[(&str, &str)] = &[("← →", "Section"), ("1–3", "Jump"), ("Enter", "Activate")];
const SELECT_HINTS: &[(&str, &str)] = &[("↑ ↓", "Choose"), ("Enter", "Choose")];
const SCROLL_HINTS: &[(&str, &str)] = &[("Wheel", "Scroll"), ("PgUp/Dn", "Scroll")];
const OPEN_HINTS: &[(&str, &str)] = &[("Enter / Space", "Open help")];
const HELP_DEFAULT_HINTS: &[(&str, &str)] = &[
    ("↑↓ PgUp/Dn", "Scroll"),
    ("Home End", "First / last"),
    ("? q Enter Esc", "Close"),
    ("Tab / Shift+Tab", "Focus controls"),
];
const HELP_F1_HINTS: &[(&str, &str)] = &[
    ("↑↓ PgUp/Dn", "Scroll"),
    ("Home End", "First / last"),
    ("F1 Enter Esc", "Close"),
    ("Tab / Shift+Tab", "Focus controls"),
];
const HELP_VIM_HINTS: &[(&str, &str)] = &[
    ("↑↓ j/k J/K", "Scroll"),
    ("PgUp/Dn Home/End g/G", "Scroll"),
    ("? q Enter Esc", "Close"),
    ("Tab / Shift+Tab", "Focus controls"),
];
const HELP_VIM_F1_HINTS: &[(&str, &str)] = &[
    ("↑↓ j/k J/K", "Scroll"),
    ("PgUp/Dn Home/End g/G", "Scroll"),
    ("F1 Enter Esc", "Close"),
    ("Tab / Shift+Tab", "Focus controls"),
];

const HELP_NAVIGATION: &[(&str, &str)] = &[
    ("Ctrl+P", "open a file by name"),
    ("Ctrl+G", "go to line"),
    ("Ctrl+Tab", "next editor tab"),
    ("Alt+←/→", "back / forward"),
    ("F12", "go to definition"),
];
const HELP_EDITING: &[(&str, &str)] = &[
    ("Ctrl+Z", "undo"),
    ("Ctrl+Y", "redo"),
    ("Ctrl+D", "select the next match"),
    ("Alt+↑/↓", "move the line"),
    ("Ctrl+/", "toggle a comment"),
    ("Tab", "indent the selection"),
];
const HELP_SEARCH: &[(&str, &str)] = &[
    ("Ctrl+F", "find in the file"),
    ("Enter", "next match"),
    ("Shift+Enter", "previous match"),
    ("Ctrl+H", "replace"),
];
const HELP_VIEW: &[(&str, &str)] = &[
    ("Ctrl+B", "show / hide the sidebar"),
    ("Ctrl+J", "show / hide the panel"),
    ("Ctrl+=", "zoom in"),
    ("Ctrl+-", "zoom out"),
];
const HELP_WINDOW: &[(&str, &str)] = &[
    ("Ctrl+W", "close the tab"),
    ("Ctrl+Shift+T", "reopen the closed tab"),
    ("Ctrl+\\", "split the editor"),
    ("Ctrl+Q", "quit"),
];
const HELP_SECTIONS: [HelpSection<'static>; 5] = [
    HelpSection::plain("Navigation", HELP_NAVIGATION),
    HelpSection::plain("Editing", HELP_EDITING),
    HelpSection::plain("Search", HELP_SEARCH),
    HelpSection::plain("View", HELP_VIEW),
    HelpSection::plain("Window", HELP_WINDOW),
];

const REFERENCE: &[ReferenceEntry] = &[
    ReferenceEntry::Section("Results"),
    ReferenceEntry::Key("↑↓ PgUp PgDn", "move between rows"),
    ReferenceEntry::Key("Enter", "open the row in the detail pane"),
    ReferenceEntry::Key("Ctrl+C", "copy the selected cells as tab-separated text"),
    ReferenceEntry::Section("Query"),
    ReferenceEntry::Key("Ctrl+Enter", "run the statement under the cursor"),
    ReferenceEntry::Key(
        "Ctrl+Shift+Enter",
        "run every statement in the editor, top to bottom",
    ),
    ReferenceEntry::Key("Esc", "cancel the running query"),
];
const STATE_REFERENCE: &[ReferenceEntry] = &[
    ReferenceEntry::Section("Query"),
    ReferenceEntry::Key("Ctrl+Enter", "run the statement"),
    ReferenceEntry::Key("Esc", "cancel"),
];
const STATE_LABELS: [&str; STATE_ROWS] = ["rest", "overflow", "narrow"];

const NOTE: &str = "It opens as the shell's modal and never over another one. The showcase's own ? opens one too, listing the shell's keys.";
const SCROLL_BINDINGS: &[Binding<HelpKeysCommand>] = &[
    Binding {
        action: SCROLL_PAGE_UP,
        chord: Some(Chord::key(KeyCode::PageUp)),
        cmd: HelpKeysCommand::PageUp,
        label: "Page up",
        priority: 70,
        visible: true,
    },
    Binding {
        action: SCROLL_PAGE_DOWN,
        chord: Some(Chord::key(KeyCode::PageDown)),
        cmd: HelpKeysCommand::PageDown,
        label: "Page down",
        priority: 70,
        visible: true,
    },
];
const HOME_END_BINDINGS: &[Binding<HelpKeysCommand>] = &[
    Binding {
        action: SCROLL_HOME,
        chord: Some(Chord::key(KeyCode::Home)),
        cmd: HelpKeysCommand::Home,
        label: "First",
        priority: 70,
        visible: true,
    },
    Binding {
        action: SCROLL_END,
        chord: Some(Chord::key(KeyCode::End)),
        cmd: HelpKeysCommand::End,
        label: "Last",
        priority: 70,
        visible: true,
    },
];
const VIM_BINDINGS: &[Binding<HelpKeysCommand>] = &[
    Binding {
        action: SCROLL_HOME,
        chord: Some(Chord::key(KeyCode::Home)),
        cmd: HelpKeysCommand::Home,
        label: "First",
        priority: 70,
        visible: true,
    },
    Binding {
        action: SCROLL_END,
        chord: Some(Chord::key(KeyCode::End)),
        cmd: HelpKeysCommand::End,
        label: "Last",
        priority: 70,
        visible: true,
    },
    Binding {
        action: SCROLL_VIM_UP,
        chord: Some(Chord::key(KeyCode::Char('K'))),
        cmd: HelpKeysCommand::VimUp,
        label: "Scroll",
        priority: 70,
        visible: true,
    },
    Binding {
        action: SCROLL_VIM_DOWN,
        chord: Some(Chord::key(KeyCode::Char('J'))),
        cmd: HelpKeysCommand::VimDown,
        label: "Scroll",
        priority: 70,
        visible: true,
    },
    Binding {
        action: SCROLL_VIM_HOME,
        chord: Some(Chord::key(KeyCode::Char('g'))),
        cmd: HelpKeysCommand::Home,
        label: "First",
        priority: 70,
        visible: true,
    },
    Binding {
        action: SCROLL_VIM_END,
        chord: Some(Chord::key(KeyCode::Char('G'))),
        cmd: HelpKeysCommand::End,
        label: "Last",
        priority: 70,
        visible: true,
    },
];

#[derive(Clone, Copy)]
enum ReferenceEntry {
    Section(&'static str),
    Key(&'static str, &'static str),
}

#[derive(Clone)]
struct ReferenceLine {
    text: String,
    key: Option<String>,
    heading: bool,
}

struct PlaygroundLayout {
    wide: bool,
    reference_width: u16,
    reference_height: usize,
    help_width: u16,
    help_panel_height: usize,
    content_len: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HelpKeysCommand {
    PageUp,
    PageDown,
    Home,
    End,
    VimUp,
    VimDown,
}

pub struct HelpKeysPage {
    page_tabs: TabsState,
    values: [usize; OPTION_COUNT],
    selects: [SelectState; OPTION_COUNT],
    play_scroll: ScrollState,
    options_scroll: ScrollState,
    states_scroll: ScrollState,
    help_state: HelpOverlayState,
    help_open: bool,
}

impl HelpKeysPage {
    pub fn new() -> Self {
        Self {
            page_tabs: TabsState::default(),
            values: [0; OPTION_COUNT],
            selects: std::array::from_fn(|_| SelectState::default()),
            play_scroll: ScrollState::default(),
            options_scroll: ScrollState::default(),
            states_scroll: ScrollState::default(),
            help_state: HelpOverlayState::default(),
            help_open: false,
        }
    }

    fn selected_section(&self) -> usize {
        match self.page_tabs.active().or(self.page_tabs.cursor()) {
            Some(ItemKey::Index(index)) => index.min(PAGE_SECTIONS.len().saturating_sub(1)),
            _ => 0,
        }
    }

    fn effective_section(&self, tabs_area: Rect) -> usize {
        if tabs_area.is_empty() {
            0
        } else {
            self.selected_section()
        }
    }

    fn help_overlay(&self) -> HelpOverlay<'static> {
        let title = if self.values[3] == 1 {
            "Editor keys"
        } else {
            "Keyboard shortcuts"
        };
        HelpOverlay::new(HELP, "Editor", &HELP_SECTIONS).title(title)
    }

    fn nav_keys(&self) -> NavKeys {
        if self.values[5] == 1 {
            NavKeys::VIM
        } else {
            NavKeys::default()
        }
    }

    fn apply_keymap(&self, keymap: &mut KeyMap) {
        let overlay = self.help_overlay();
        let owner = overlay.command_id();
        let nav = self.nav_keys();
        for binding in overlay.bindings(BindingState::default()) {
            let Some(chord) = binding.chord else {
                continue;
            };
            if matches!(binding.cmd, HelpCmd::Prev | HelpCmd::Next)
                && matches!(chord.code, KeyCode::Char(_))
                && nav
                    .action(&Key {
                        code: chord.code,
                        mods: chord.mods,
                    })
                    .is_none()
            {
                keymap.remove_component(owner, binding.action);
                if self.help_open {
                    keymap.add(KeyPhase::Capture, chord, HELP_KEYS_NOOP);
                }
            }
            if self.values[6] == 1 && binding.cmd == HelpCmd::Close {
                match chord.code {
                    KeyCode::Char('?') => {
                        keymap.remap_component(owner, binding.action, Chord::key(KeyCode::F(1)))
                    }
                    KeyCode::Char('q') => keymap.remove_component(owner, binding.action),
                    _ => {}
                }
            }
        }

        if !self.help_open {
            return;
        }
        for code in [KeyCode::Left, KeyCode::Right] {
            keymap.add(KeyPhase::Capture, Chord::key(code), HELP_KEYS_NOOP);
        }
        if nav.vim {
            for code in [
                KeyCode::Char('h'),
                KeyCode::Char('H'),
                KeyCode::Char('l'),
                KeyCode::Char('L'),
            ] {
                let key = Key {
                    code,
                    mods: KeyModifiers::NONE,
                };
                if matches!(nav.action(&key), Some(Nav::Left | Nav::Right)) {
                    keymap.add(KeyPhase::Capture, key.chord(), HELP_KEYS_NOOP);
                }
            }
        }
        let codes = [
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::PageUp,
            KeyCode::PageDown,
            KeyCode::Home,
            KeyCode::End,
            KeyCode::Char('j'),
            KeyCode::Char('k'),
            KeyCode::Char('J'),
            KeyCode::Char('K'),
            KeyCode::Char('h'),
            KeyCode::Char('H'),
            KeyCode::Char('l'),
            KeyCode::Char('L'),
            KeyCode::Char('g'),
            KeyCode::Char('G'),
        ];
        for bits in 1..=KeyModifiers::all().bits() {
            let modifiers = KeyModifiers::from_bits_truncate(bits);
            for code in codes {
                if matches!(code, KeyCode::Char(_))
                    && modifiers.difference(KeyModifiers::SHIFT).is_empty()
                {
                    continue;
                }
                let key = Key {
                    code,
                    mods: modifiers,
                };
                let Some(nav_action) = nav.action(&key) else {
                    continue;
                };
                let action = match nav_action {
                    Nav::Up => MODIFIED_NAV_UP,
                    Nav::Down => MODIFIED_NAV_DOWN,
                    Nav::Left | Nav::Right => HELP_KEYS_NOOP,
                    Nav::PageUp => MODIFIED_NAV_PAGE_UP,
                    Nav::PageDown => MODIFIED_NAV_PAGE_DOWN,
                    Nav::Home => MODIFIED_NAV_HOME,
                    Nav::End => MODIFIED_NAV_END,
                };
                keymap.add(KeyPhase::Capture, key.chord(), action);
            }
        }
    }

    fn apply_navigation(&mut self, nav: Nav) -> Response<()> {
        let scroll = self.help_state.scroll_mut();
        let before = scroll.offset();
        match nav {
            Nav::Up => scroll.scroll_by(-1),
            Nav::Down => scroll.scroll_by(1),
            Nav::PageUp => scroll.page_up(),
            Nav::PageDown => scroll.page_down(),
            Nav::Home => scroll.jump_start(),
            Nav::End => scroll.jump_end(),
            Nav::Left | Nav::Right => return Response::consumed(),
        }
        if scroll.offset() == before {
            Response::consumed()
        } else {
            Response::changed()
        }
    }

    fn update_help_navigation(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let active = cx.is_open(HELP)
            && cx
                .layer_opening(HELP)
                .is_some_and(|opening| cx.top_layer() == opening.layer_id());
        if !active {
            return Response::ignored();
        }

        let mut response = Response::ignored();
        if let Some(action) = cx.command() {
            let nav = match action {
                MODIFIED_NAV_UP => Some(Nav::Up),
                MODIFIED_NAV_DOWN => Some(Nav::Down),
                MODIFIED_NAV_PAGE_UP => Some(Nav::PageUp),
                MODIFIED_NAV_PAGE_DOWN => Some(Nav::PageDown),
                MODIFIED_NAV_HOME => Some(Nav::Home),
                MODIFIED_NAV_END => Some(Nav::End),
                _ => None,
            };
            if let Some(nav) = nav {
                response |= self.apply_navigation(nav);
            }
        }

        let nav = self.nav_keys();
        let bindings = if nav.vim {
            VIM_BINDINGS
        } else {
            HOME_END_BINDINGS
        };
        for intent in cx.intents(HELP_COMMANDS) {
            let Intent::Binding(action) = intent else {
                continue;
            };
            let Some(command) = Binding::command(bindings, action) else {
                continue;
            };
            let (key, expected) = match command {
                HelpKeysCommand::Home => (
                    Key {
                        code: KeyCode::Home,
                        mods: KeyModifiers::NONE,
                    },
                    Nav::Home,
                ),
                HelpKeysCommand::End => (
                    Key {
                        code: KeyCode::End,
                        mods: KeyModifiers::NONE,
                    },
                    Nav::End,
                ),
                HelpKeysCommand::VimUp => (
                    Key {
                        code: KeyCode::Char('K'),
                        mods: KeyModifiers::NONE,
                    },
                    Nav::Up,
                ),
                HelpKeysCommand::VimDown => (
                    Key {
                        code: KeyCode::Char('J'),
                        mods: KeyModifiers::NONE,
                    },
                    Nav::Down,
                ),
                HelpKeysCommand::PageUp | HelpKeysCommand::PageDown => continue,
            };
            if nav.action(&key) == Some(expected) {
                response |= self.apply_navigation(expected);
            }
        }
        response
    }

    fn overlay_size(&self, viewport: Rect) -> LayerSize {
        let percent = match self.values[4] {
            1 => 60,
            2 => 100,
            _ => 80,
        };
        let percent_of = |dimension: u16| {
            u16::try_from(u32::from(dimension) * percent / 100).unwrap_or(u16::MAX)
        };
        LayerSize::Fixed(
            viewport
                .width
                .saturating_sub(4)
                .min(percent_of(viewport.width))
                .max(16),
            viewport
                .height
                .saturating_sub(3)
                .min(percent_of(viewport.height))
                .max(8),
        )
    }

    fn overlay_dismiss(&self) -> Dismiss {
        if self.values[7] == 1 {
            Dismiss::ESC
        } else {
            Dismiss::ESC_AND_OUTSIDE
        }
    }

    fn reference_width(&self, available: u16) -> u16 {
        let requested = match self.values[0] {
            1 => 40,
            2 => 28,
            _ => REFERENCE_PANEL_WIDTH.saturating_sub(4),
        };
        requested.min(available)
    }

    fn reference_lines(&self, available: u16, narrow: bool, overflow: bool) -> Vec<ReferenceLine> {
        let available = if narrow { available.min(24) } else { available };
        let share = match self.values[1] {
            1 => 25,
            2 => 60,
            _ => 40,
        };
        let gap = if self.values[2] == 1 { 6 } else { 3 };
        let mut entries = REFERENCE.to_vec();
        if overflow {
            entries.push(ReferenceEntry::Key(
                "Ctrl+Shift+F",
                "find in every open file, and list each match under its file",
            ));
        }
        Self::format_reference_lines(&entries, available, share, gap)
    }

    fn state_reference_lines(
        available: u16,
        narrow: bool,
        overflow: bool,
        share: u16,
    ) -> Vec<ReferenceLine> {
        let available = if narrow { available.min(24) } else { available };
        let mut entries = STATE_REFERENCE.to_vec();
        if overflow {
            entries.push(ReferenceEntry::Key(
                "Ctrl+Shift+F",
                "find in every open file, and list each match under its file",
            ));
        }
        Self::format_reference_lines(&entries, available, share, 3)
    }

    fn format_reference_lines(
        entries: &[ReferenceEntry],
        available: u16,
        share: u16,
        gap: u16,
    ) -> Vec<ReferenceLine> {
        let widest = entries
            .iter()
            .filter_map(|entry| match entry {
                ReferenceEntry::Key(keys, _) => Some(width(keys)),
                ReferenceEntry::Section(_) => None,
            })
            .max()
            .unwrap_or(0);
        let key_width = widest.min(
            u16::try_from(usize::from(available) * usize::from(share) / 100).unwrap_or(u16::MAX),
        );
        let gap_text = " ".repeat(usize::from(gap.max(3)));
        let mut lines = Vec::new();
        for (index, entry) in entries.iter().enumerate() {
            match entry {
                ReferenceEntry::Section(title) => {
                    if index > 0 {
                        lines.push(ReferenceLine {
                            text: String::new(),
                            key: None,
                            heading: false,
                        });
                    }
                    lines.push(ReferenceLine {
                        text: (*title).to_owned(),
                        key: None,
                        heading: true,
                    });
                }
                ReferenceEntry::Key(keys, action) => {
                    let key_padding = usize::from(key_width.saturating_sub(width(keys)));
                    let prefix_width = width(keys)
                        .saturating_add(u16::try_from(key_padding).unwrap_or(u16::MAX))
                        .saturating_add(gap.max(3));
                    let action_width = available.saturating_sub(prefix_width).max(1);
                    let action_lines = wrap(action, action_width);
                    let first = action_lines.first().map_or("", String::as_str);
                    lines.push(ReferenceLine {
                        text: format!("{keys}{}{gap_text}{first}", " ".repeat(key_padding)),
                        key: Some((*keys).to_owned()),
                        heading: false,
                    });
                    let hanging = " ".repeat(usize::from(prefix_width));
                    lines.extend(action_lines.iter().skip(1).map(|line| ReferenceLine {
                        text: format!("{hanging}{line}"),
                        key: None,
                        heading: false,
                    }));
                }
            }
        }
        lines
    }

    fn reference_panel_metrics(&self, ui: &mut Ui<'_>, width: u16) -> (u16, usize) {
        let panel = Panel::new(REFERENCE_PANEL)
            .kind(PanelKind::Card)
            .title("KeyReference");
        let inner = panel_inner(ui, &panel, Rect::new(0, 0, width, 4));
        let reference_width = self.reference_width(inner.width);
        let height = self
            .reference_lines(reference_width, false, false)
            .len()
            .saturating_add(3);
        (reference_width, height)
    }

    fn help_panel_metrics(&self, ui: &mut Ui<'_>, width: u16) -> (usize, usize) {
        let panel = Panel::new(HELP_PANEL)
            .kind(PanelKind::Card)
            .title("HelpOverlay");
        let inner = panel_inner(ui, &panel, Rect::new(0, 0, width, 4));
        let fact_height = self.help_facts_line_count(ui, inner.width);
        let panel_height = fact_height.saturating_add(5);
        let note_height = wrap(NOTE, width.max(1)).len();
        let total_height = panel_height.saturating_add(1).saturating_add(note_height);
        (panel_height, total_height)
    }

    fn playground_layout(&self, ui: &mut Ui<'_>, width: u16) -> PlaygroundLayout {
        let wide = width >= WIDE_LAYOUT;
        let reference_panel_width = if wide {
            REFERENCE_PANEL_WIDTH.min(width.saturating_sub(31))
        } else {
            width
        };
        let (reference_width, reference_height) =
            self.reference_panel_metrics(ui, reference_panel_width);
        let help_width = if wide {
            width.saturating_sub(reference_panel_width.saturating_add(1))
        } else {
            width
        };
        let (help_panel_height, help_total_height) = self.help_panel_metrics(ui, help_width);
        let content_len = if wide {
            reference_height.max(help_total_height)
        } else {
            reference_height
                .saturating_add(1)
                .saturating_add(help_total_height)
        };
        PlaygroundLayout {
            wide,
            reference_width,
            reference_height,
            help_width,
            help_panel_height,
            content_len,
        }
    }

    fn help_facts(&self, ui: &Ui<'_>) -> [(&'static str, String); 6] {
        let title = if self.values[3] == 1 {
            "Editor keys"
        } else {
            "Keyboard shortcuts"
        };
        let max_percent = match self.values[4] {
            1 => 60,
            2 => 100,
            _ => 80,
        };
        let overlay = self.help_overlay();
        let close = overlay
            .bindings(BindingState::default())
            .iter()
            .filter(|binding| binding.cmd == HelpCmd::Close)
            .filter_map(|binding| {
                ui.effective_chord(overlay.command_id(), binding.action, binding.chord)
            })
            .map(|chord| ChordText::of(chord).as_str().to_owned())
            .collect::<Vec<_>>()
            .join(" ");
        let dismiss = if self.values[7] == 1 {
            "Esc and Done"
        } else {
            "Esc, Done, outside click"
        };
        [
            ("Title", title.to_owned()),
            ("Scope", "Editor".to_owned()),
            ("Max share", format!("{max_percent}%")),
            (
                "Scroll keys",
                if self.nav_keys().vim {
                    "↑↓ PgUp PgDn Home End j/k J/K g/G".to_owned()
                } else {
                    "↑↓ PgUp PgDn Home End".to_owned()
                },
            ),
            ("Close keys", close),
            ("Dismiss", dismiss.to_owned()),
        ]
    }

    fn help_facts_line_count(&self, ui: &Ui<'_>, width: u16) -> usize {
        let label_width = width.min(12);
        let value_width = width.saturating_sub(label_width).saturating_sub(1).max(1);
        self.help_facts(ui)
            .iter()
            .map(|(_, value)| wrap(value, value_width).len().max(1))
            .sum()
    }

    fn draw_reference_lines_window(
        &self,
        ui: &mut Ui<'_>,
        window: VerticalWindow,
        source_start: usize,
        x: u16,
        line_width: u16,
        lines: &[ReferenceLine],
    ) {
        let header = ui
            .style(
                Family::HELP,
                Variant::DEFAULT,
                Part::HEADER,
                StateFlags::empty(),
            )
            .style;
        let action = ui
            .style(
                Family::HELP,
                Variant::DEFAULT,
                Part::ACTION,
                StateFlags::empty(),
            )
            .style;
        let key = ui
            .style(
                Family::HELP,
                Variant::DEFAULT,
                Part::KEY,
                StateFlags::empty(),
            )
            .style;
        for (index, line) in lines.iter().enumerate() {
            let source_row = source_start.saturating_add(index);
            let Some(y) = window.physical_row_for_source(source_row) else {
                continue;
            };
            let row = Rect::new(x, y, line_width, 1);
            if line.heading {
                ui.paint_str(row, &line.text, header);
            } else {
                ui.paint_str(row, &line.text, action);
                if let Some(key_text) = &line.key {
                    ui.paint_str(
                        Rect::new(x, y, width(key_text).min(line_width), 1),
                        key_text,
                        key,
                    );
                }
            }
        }
    }

    fn draw_reference_panel(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        window: VerticalWindow,
        reference_width: u16,
    ) {
        let lines = self.reference_lines(reference_width, false, false);
        let title = format!("KeyReference at width {reference_width}");
        let meta = format!(
            "share {}% · gap {}",
            [40, 25, 60][self.values[1].min(2)],
            [3, 6][self.values[2].min(1)]
        );
        let gutter = ui
            .style(
                Family::HELP,
                Variant::DEFAULT,
                Part::GUTTER,
                StateFlags::empty(),
            )
            .style;
        let panel = Panel::new(REFERENCE_PANEL)
            .kind(PanelKind::Card)
            .title(&title)
            .meta(&meta);
        let inner = panel_inner(ui, &panel, area);
        let edge = inner.x.saturating_add(reference_width);
        let _ = panel.draw_window(ui, area, window, |ui, body_window| {
            let Some(body_window) = body_window else {
                return;
            };
            let visible = body_window.visible_body();
            self.draw_reference_lines_window(ui, body_window, 0, visible.x, visible.width, &lines);
            if edge < inner.right() {
                for source_row in 0..lines.len() {
                    if let Some(y) = body_window.physical_row_for_source(source_row) {
                        ui.paint_str(Rect::new(edge, y, 1, 1), "┊", gutter);
                    }
                }
            }
        });
    }

    fn draw_help_facts_window(&self, ui: &mut Ui<'_>, window: VerticalWindow) {
        let area = window.visible_body();
        let rows = self.help_facts(ui);
        let label_width = area.width.min(12);
        let value_x = area.x.saturating_add(label_width).saturating_add(1);
        let value_width = area.right().saturating_sub(value_x).max(1);
        let label_style = ui
            .style(
                Family::HELP,
                Variant::DEFAULT,
                Part::HEADER,
                StateFlags::empty(),
            )
            .style;
        let value_style = ui
            .style(
                Family::HELP,
                Variant::DEFAULT,
                Part::ACTION,
                StateFlags::empty(),
            )
            .style;
        let mut source_row = 2usize;
        for (label, value) in rows {
            let value_lines = wrap(&value, value_width);
            if let Some(y) = window.physical_row_for_source(source_row) {
                ui.paint_str(
                    Rect::new(area.x, y, label_width.min(width(label)), 1),
                    label,
                    label_style,
                );
            }
            for (line_index, line) in value_lines.iter().enumerate() {
                if let Some(y) =
                    window.physical_row_for_source(source_row.saturating_add(line_index))
                {
                    ui.paint_str(Rect::new(value_x, y, value_width, 1), line, value_style);
                }
            }
            source_row = source_row.saturating_add(value_lines.len().max(1));
        }
    }

    fn draw_help_panel(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        panel_window: Option<VerticalWindow>,
        page_window: VerticalWindow,
        note_start: usize,
    ) {
        let panel = Panel::new(HELP_PANEL)
            .kind(PanelKind::Card)
            .title("HelpOverlay");
        let inner = panel_inner(ui, &panel, area);
        let button = Button::new(OPEN_HELP, "Open help overlay");
        let button_width = button
            .measure(ui, Constraints::loose(inner.width, 1))
            .preferred
            .0
            .min(inner.width);
        let note_lines = wrap(NOTE, area.width.max(1));
        if let Some(panel_window) = panel_window {
            let _ = panel.draw_window(ui, area, panel_window, |ui, body_window| {
                let Some(body_window) = body_window else {
                    return;
                };
                let body = body_window.visible_body();
                if button_width > 0
                    && let Some(y) = body_window.physical_row_for_source(0)
                {
                    button.draw(ui, Rect::new(body.x, y, button_width, 1));
                }
                self.draw_help_facts_window(ui, body_window);
            });
        }
        let note_style = ui
            .style(
                Family::HELP,
                Variant::DEFAULT,
                Part::DETAIL,
                StateFlags::empty(),
            )
            .style;
        for (index, line) in note_lines.iter().enumerate() {
            if let Some(y) = page_window.physical_row_for_source(note_start.saturating_add(index)) {
                ui.paint_str(Rect::new(area.x, y, area.width, 1), line, note_style);
            }
        }
    }

    fn draw_playground_content(&self, ui: &mut Ui<'_>, window: VerticalWindow) {
        let visible = window.visible_body();
        let layout = self.playground_layout(ui, visible.width);
        let left_width = if layout.wide {
            REFERENCE_PANEL_WIDTH.min(visible.width.saturating_sub(31))
        } else {
            visible.width
        };
        let reference_height = u16::try_from(layout.reference_height).unwrap_or(u16::MAX);
        let reference_area = Rect::new(visible.x, 0, left_width, reference_height);
        if let Some(reference_window) =
            nested_window(window, 0, layout.reference_height, visible.x, left_width)
        {
            self.draw_reference_panel(ui, reference_area, reference_window, layout.reference_width);
        }

        let (help_x, help_start) = if layout.wide {
            (visible.x.saturating_add(left_width).saturating_add(1), 0)
        } else {
            (visible.x, layout.reference_height.saturating_add(1))
        };
        let help_panel_height = u16::try_from(layout.help_panel_height).unwrap_or(u16::MAX);
        let help_area = Rect::new(
            help_x,
            u16::try_from(help_start).unwrap_or(u16::MAX),
            layout.help_width,
            help_panel_height,
        );
        let panel_window = nested_window(
            window,
            help_start,
            layout.help_panel_height,
            help_x,
            layout.help_width,
        );
        let note_start = help_start
            .saturating_add(layout.help_panel_height)
            .saturating_add(1);
        self.draw_help_panel(ui, help_area, panel_window, window, note_start);
    }

    fn draw_playground(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let content_width = area.width.saturating_sub(1);
        let content_len = self.playground_layout(ui, content_width).content_len;
        let viewport = ScrollRegion::new(PLAY_SCROLL)
            .scrollbar_gutter(ScrollbarGutter::Stable)
            .draw(ui, area, &self.play_scroll, content_len);
        ui.register_focus_only(PLAY_SCROLL, Focusability::Focusable);
        register_scroll_keys(ui, PLAY_SCROLL);
        if let Some(window) = scroll_window(viewport, content_len, &self.play_scroll) {
            ui.with_area(viewport, |ui| self.draw_playground_content(ui, window));
        }
    }

    fn draw_options(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let view =
            ScrollRegion::new(OPTIONS_SCROLL).draw(ui, area, &self.options_scroll, OPTION_ROWS);
        ui.register_focus_only(OPTIONS_SCROLL, Focusability::Focusable);
        register_scroll_keys(ui, OPTIONS_SCROLL);
        let projected = ScrollRegion::view(&self.options_scroll, view, OPTION_ROWS);
        let visible = Rect {
            height: view
                .height
                .min(u16::try_from(OPTION_ROWS).unwrap_or(u16::MAX)),
            ..view
        };
        let Ok(window) =
            VerticalWindow::new(visible, OPTION_ROWS, projected.offset().min(OPTION_ROWS))
        else {
            return;
        };
        for index in 0..OPTION_COUNT {
            let select = Select::new(OPTION_IDS[index]).popup_rows(3);
            let Some(y) = window.physical_row_for_source(index) else {
                select.register_focus_only_in_form(ui, false);
                continue;
            };
            let label_width = view.width.saturating_sub(16).min(34);
            let control_x = view.x.saturating_add(label_width).saturating_add(1);
            let control_width = view.right().saturating_sub(control_x);
            ui.paint_str(
                Rect::new(view.x, y, label_width.min(width(OPTION_LABELS[index])), 1),
                OPTION_LABELS[index],
                ui.surface_style(),
            );
            if control_width > 0 {
                let control_area = Rect::new(control_x, y, control_width, 1);
                select.draw(ui, control_area, &self.selects[index], OPTION_ITEMS[index]);
            } else {
                select.register_focus_only_in_form(ui, false);
            }
        }
    }

    fn state_grid_widths(width: u16) -> (u16, u16) {
        let label_width = width.min(15).min(width / 3);
        let column_width = (width.saturating_sub(label_width) / 2).min(40);
        (label_width, column_width)
    }

    fn state_content_len(&self, width: u16) -> usize {
        let (_, column_width) = Self::state_grid_widths(width);
        let cell_width = column_width.saturating_sub(1);
        if cell_width == 0 {
            return STATE_ROWS.saturating_add(1);
        }
        let measure_width = cell_width.min(32);
        1usize.saturating_add(
            (0..STATE_ROWS)
                .map(|index| {
                    let narrow = index == 2;
                    let overflow = index == 1;
                    let reference =
                        Self::state_reference_lines(measure_width, narrow, overflow, 40);
                    let narrow_keys =
                        Self::state_reference_lines(measure_width, narrow, overflow, 25);
                    reference.len().max(narrow_keys.len()).max(1)
                })
                .sum::<usize>(),
        )
    }

    fn draw_states_content(&self, ui: &mut Ui<'_>, window: VerticalWindow) {
        let area = window.visible_body();
        let (label_width, column_width) = Self::state_grid_widths(area.width);
        let cell_width = column_width.saturating_sub(1);
        let first_x = area.x.saturating_add(label_width);
        let second_x = first_x.saturating_add(column_width);
        let heading = ui
            .style(
                Family::LIST,
                Variant::DEFAULT,
                Part::META,
                StateFlags::empty(),
            )
            .style;
        let state_style = ui
            .style(
                Family::PANEL,
                Variant::DEFAULT,
                Part::TITLE,
                StateFlags::empty(),
            )
            .style;
        if let Some(y) = window.physical_row_for_source(0) {
            ui.paint_str(Rect::new(first_x, y, cell_width, 1), "Reference", heading);
            ui.paint_str(
                Rect::new(second_x, y, cell_width, 1),
                "Narrow keys",
                heading,
            );
        }
        if cell_width == 0 {
            for (index, label) in STATE_LABELS.iter().enumerate() {
                if let Some(y) = window.physical_row_for_source(index.saturating_add(1)) {
                    ui.paint_str(Rect::new(area.x, y, label_width, 1), label, state_style);
                }
            }
            return;
        }
        let measure_width = cell_width.min(32);
        let mut source_row = 1usize;
        for index in 0..STATE_ROWS {
            let narrow = index == 2;
            let overflow = index == 1;
            let reference = Self::state_reference_lines(cell_width, narrow, overflow, 40);
            let narrow_keys = Self::state_reference_lines(cell_width, narrow, overflow, 25);
            let reference_height =
                Self::state_reference_lines(measure_width, narrow, overflow, 40).len();
            let narrow_keys_height =
                Self::state_reference_lines(measure_width, narrow, overflow, 25).len();
            let height = reference_height.max(narrow_keys_height).max(1);
            if let Some(y) = window.physical_row_for_source(source_row) {
                ui.paint_str(
                    Rect::new(area.x, y, label_width, 1),
                    STATE_LABELS[index],
                    state_style,
                );
            }
            self.draw_reference_lines_window(
                ui, window, source_row, first_x, cell_width, &reference,
            );
            self.draw_reference_lines_window(
                ui,
                window,
                source_row,
                second_x,
                cell_width,
                &narrow_keys,
            );
            source_row = source_row.saturating_add(height);
        }
    }

    fn draw_states(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let content_len = self.state_content_len(area.width.saturating_sub(1));
        let view = ScrollRegion::new(STATES_SCROLL)
            .scrollbar_gutter(ScrollbarGutter::Stable)
            .draw(ui, area, &self.states_scroll, content_len);
        ui.register_focus_only(STATES_SCROLL, Focusability::Focusable);
        register_scroll_keys(ui, STATES_SCROLL);
        if let Some(window) = scroll_window(view, content_len, &self.states_scroll) {
            ui.with_area(view, |ui| self.draw_states_content(ui, window));
        }
    }
}

impl Default for HelpKeysPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for HelpKeysPage {
    fn title(&self) -> &'static str {
        "Help & keys"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let tabs_area = cx.area(PAGE_TABS).unwrap_or_default();
        let previous_section = self.effective_section(tabs_area);
        let mut response = Tabs::new(PAGE_TABS)
            .update(cx, &mut self.page_tabs, PAGE_SECTIONS)
            .erase();
        let section = self.effective_section(tabs_area);
        if section != previous_section {
            match section {
                0 => self.play_scroll.scroll_to(0),
                1 => self.options_scroll.scroll_to(0),
                2 => self.states_scroll.scroll_to(0),
                _ => {}
            }
        }

        if section == 0 {
            let content_len = cx.layout(PLAY_SCROLL).map_or(0, |facts| facts.content_len);
            response |=
                ScrollRegion::new(PLAY_SCROLL).update(cx, &mut self.play_scroll, content_len);
            response |= update_scroll_keys(cx, PLAY_SCROLL, &mut self.play_scroll);
        } else if section == 1 {
            response |=
                ScrollRegion::new(OPTIONS_SCROLL).update(cx, &mut self.options_scroll, OPTION_ROWS);
            response |= update_scroll_keys(cx, OPTIONS_SCROLL, &mut self.options_scroll);
        } else {
            let content_len = cx
                .layout(STATES_SCROLL)
                .map_or(0, |facts| facts.content_len);
            response |=
                ScrollRegion::new(STATES_SCROLL).update(cx, &mut self.states_scroll, content_len);
            response |= update_scroll_keys(cx, STATES_SCROLL, &mut self.states_scroll);
        }

        let mut status = None;
        for index in 0..OPTION_COUNT {
            let id = OPTION_IDS[index];
            if (section == 1 && cx.area(id).is_some())
                || self.selects[index].is_open()
                || cx.is_open(id)
            {
                let select = Select::new(id).popup_rows(3);
                response |= select
                    .update_in_form(
                        cx,
                        &mut self.selects[index],
                        &mut self.values[index],
                        OPTION_ITEMS[index],
                        false,
                    )
                    .erase();
            }
        }

        let open = Button::new(OPEN_HELP, "Open help overlay");
        if section == 0 && cx.area(OPEN_HELP).is_some() {
            let activation = open.update(cx);
            if activation.activated()
                && !cx.is_open(HELP)
                && cx.top_layer() == termrock::LayerId::PAGE
            {
                let overlay = self.help_overlay();
                cx.open_layer(
                    HELP,
                    overlay
                        .layer(cx)
                        .size(self.overlay_size(cx.viewport()))
                        .dismiss(self.overlay_dismiss()),
                );
                self.help_state = HelpOverlayState::default();
                status = Some(PageStatus("Opened Editor help".to_owned()));
            }
            response |= activation.erase();
        }

        let overlay = self.help_overlay();
        response |= overlay.update(cx, &mut self.help_state).erase();
        if cx.is_open(HELP) {
            cx.resize_layer(HELP, self.overlay_size(cx.viewport()));
        }
        response |= self.update_help_navigation(cx);
        self.help_open = cx.is_open(HELP)
            && cx
                .layer_opening(HELP)
                .is_some_and(|opening| cx.top_layer() == opening.layer_id());

        PageUpdate { response, status }
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        OPTION_KEYS
            .iter()
            .position(|candidate| *candidate == key)
            .map(|index| self.values[index])
    }

    fn set_option(&mut self, key: &str, value: usize) {
        if let Some(index) = OPTION_KEYS.iter().position(|candidate| *candidate == key) {
            self.values[index] = value.min(OPTION_ITEMS[index].len().saturating_sub(1));
        }
    }

    fn keymap(&self, keymap: &mut KeyMap) {
        self.apply_keymap(keymap);
    }

    fn blocks_shell_commands(&self, cx: &Cx<'_>) -> bool {
        cx.is_open(HELP)
            && cx
                .layer_opening(HELP)
                .is_some_and(|opening| cx.top_layer() == opening.layer_id())
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let tabs_area = section_tabs_area(area);
        let tabs = Tabs::new(PAGE_TABS);
        let previous =
            ui.layout(PAGE_TABS)
                .map(|facts| termrock::navigation::tabs::TabWindowReceipt {
                    viewport_len: facts.viewport_len,
                    content_len: facts.content_len,
                });
        let tabs_layout =
            tabs.layout_default(tabs_area, area, &self.page_tabs, PAGE_SECTIONS, previous);
        let section = if tabs_layout.visible_area.is_empty() {
            0
        } else {
            self.selected_section()
        };
        frame_with_body_rows(
            ui,
            area,
            self.title(),
            "HelpOverlay and keyboard-reference layout",
            PAGE_BODY_START_ROW,
            |ui, body| {
                if !tabs_layout.visible_area.is_empty() {
                    let _ = tabs.draw_layout(ui, &tabs_layout);
                }
                match section {
                    1 => self.draw_options(ui, body),
                    2 => self.draw_states(ui, body),
                    _ => self.draw_playground(ui, body),
                }
            },
        );
        ui.layer(HELP, |ui, layer| {
            let bindings = if self.nav_keys().vim {
                VIM_BINDINGS
            } else {
                HOME_END_BINDINGS
            };
            ui.command_scope(HELP_COMMANDS, bindings, |ui| {
                self.help_overlay().draw(ui, layer, &self.help_state);
            });
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.help_open {
            return match (self.nav_keys().vim, self.values[6] == 1) {
                (false, false) => HELP_DEFAULT_HINTS,
                (false, true) => HELP_F1_HINTS,
                (true, false) => HELP_VIM_HINTS,
                (true, true) => HELP_VIM_F1_HINTS,
            };
        }
        if ui.state(PAGE_TABS).contains(StateFlags::FOCUSED) {
            return TABS_HINTS;
        }
        if OPTION_IDS
            .iter()
            .any(|id| ui.state(*id).contains(StateFlags::FOCUSED))
        {
            return SELECT_HINTS;
        }
        if [PLAY_SCROLL, OPTIONS_SCROLL, STATES_SCROLL]
            .iter()
            .any(|id| ui.state(*id).contains(StateFlags::FOCUSED))
        {
            return SCROLL_HINTS;
        }
        if ui.state(OPEN_HELP).contains(StateFlags::FOCUSED) {
            return OPEN_HINTS;
        }
        match self.selected_section() {
            1 => SELECT_HINTS,
            2 => SCROLL_HINTS,
            _ => OPEN_HINTS,
        }
    }
}

fn section_tabs_area(area: Rect) -> Rect {
    let y = area.y.saturating_add(2).min(area.bottom());
    Rect {
        y,
        height: area.bottom().saturating_sub(y).min(2),
        ..area
    }
}

fn panel_inner(ui: &mut Ui<'_>, panel: &Panel<'_>, area: Rect) -> Rect {
    let surface = ui.theme().raise(ui.surface());
    ui.with_surface(surface, |ui| panel.inner(ui, area))
}

fn scroll_window(view: Rect, content_len: usize, state: &ScrollState) -> Option<VerticalWindow> {
    if view.is_empty() || content_len == 0 {
        return None;
    }
    let first = ScrollRegion::view(state, view, content_len)
        .offset()
        .min(content_len);
    let visible_rows = usize::from(view.height).min(content_len.saturating_sub(first));
    if visible_rows == 0 {
        return None;
    }
    let visible = Rect {
        height: u16::try_from(visible_rows).ok()?,
        ..view
    };
    VerticalWindow::new(visible, content_len, first).ok()
}

fn nested_window(
    parent: VerticalWindow,
    source_start: usize,
    source_rows: usize,
    x: u16,
    width: u16,
) -> Option<VerticalWindow> {
    if source_rows == 0 || width == 0 {
        return None;
    }
    let visible = parent.visible_source_range();
    let start = visible.start.max(source_start);
    let end = visible.end.min(source_start.saturating_add(source_rows));
    if start >= end {
        return None;
    }
    let y = parent.physical_row_for_source(start)?;
    let height = u16::try_from(end.saturating_sub(start)).ok()?;
    VerticalWindow::new(
        Rect::new(x, y, width, height),
        source_rows,
        start.saturating_sub(source_start),
    )
    .ok()
}

fn register_scroll_keys(ui: &mut Ui<'_>, id: Id) {
    ui.publish_bindings(id, ui.state(id), SCROLL_BINDINGS);
}

fn update_scroll_keys(cx: &mut Cx<'_>, id: Id, state: &mut ScrollState) -> Response<()> {
    let mut response = Response::ignored();
    for intent in cx.intents(id) {
        let Intent::Binding(action) = intent else {
            continue;
        };
        match Binding::command(SCROLL_BINDINGS, action) {
            Some(HelpKeysCommand::PageUp) => state.page_up(),
            Some(HelpKeysCommand::PageDown) => state.page_down(),
            _ => continue,
        }
        response |= Response::changed().for_id(id);
    }
    response
}
