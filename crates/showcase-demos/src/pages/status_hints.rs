use std::{borrow::Cow, cell::Cell, time::Duration};

use termrock::{
    ActionKey, Binding, Button, Chord, ColorLevel, Cx, Focusability, FrameRead, Hint, HintBar,
    HintKey, HintLayer, Id, Intent, ItemKey, KeyCode, Moment, Panel, PanelKind, Part, ProgressBar,
    Rect, Response, Role, ScopeId, ScrollRegion, ScrollState, Select, SelectState, Spinner,
    StateFlags, Status, StylePatch, Tabs, TabsState, Toggle, TooSmall, Ui, UpdateCause,
};

use super::{Page, PageStatus, PageUpdate, frame};

const ID: Id = Id::root("showcase_ui::pages::status_hints");
const PAGE_TABS: Id = ID.sub("page_tabs");
const PLAY_SCROLL: Id = ID.sub("play_scroll");
const OPTIONS_SCROLL: Id = ID.sub("options_scroll");
const STATES_SCROLL: Id = ID.sub("states_scroll");
const COMPARE_LEVELS: Id = ID.sub("states_compare_levels");
const PAGE_SCROLL_UP: ActionKey = ActionKey::application("showcase.status_hints.page_scroll_up");
const PAGE_SCROLL_DOWN: ActionKey =
    ActionKey::application("showcase.status_hints.page_scroll_down");
const PAGE_SCROLL_BINDINGS: &[Binding<()>] = &[
    Binding {
        action: PAGE_SCROLL_UP,
        chord: Some(Chord::key(KeyCode::PageUp)),
        cmd: (),
        label: "Scroll up",
        priority: 70,
        visible: true,
    },
    Binding {
        action: PAGE_SCROLL_DOWN,
        chord: Some(Chord::key(KeyCode::PageDown)),
        cmd: (),
        label: "Scroll down",
        priority: 70,
        visible: true,
    },
];
const POST_INFO: Id = ID.sub("post_info");
const POST_WARNING: Id = ID.sub("post_warning");
const POST_ERROR: Id = ID.sub("post_error");
const RUN_JOB: Id = ID.sub("run_job");
const SPINNER: Id = ID.sub("spinner");
const PROGRESS: Id = ID.sub("progress");
const TOO_SMALL: Id = ID.sub("too_small");
const PLAYGROUND_SCROLL_OWNERS: [Id; 4] = [POST_INFO, POST_WARNING, POST_ERROR, RUN_JOB];
const STATE_SCROLL_OWNERS: [Id; 1] = [COMPARE_LEVELS];

const SECTIONS: &[&str] = &["Playground", "Options", "States"];
const TAB_HINTS: &[(&str, &str)] = &[("← →", "Section"), ("1–3", "Jump"), ("Enter", "Activate")];
const SELECT_HINTS: &[(&str, &str)] = &[("↑ ↓", "Choose"), ("Enter", "Open / choose")];
const TOGGLE_HINTS: &[(&str, &str)] = &[("Space", "Toggle"), ("Enter", "Toggle")];
const SCROLL_HINTS: &[(&str, &str)] = &[("Wheel", "Scroll"), ("PgUp / PgDn", "Page")];
const ACTIVATE_HINTS: &[(&str, &str)] = &[("Enter", "Activate")];

const OPT_LAYER: usize = 0;
const OPT_ALIGN: usize = 1;
const OPT_WIDTH: usize = 2;
const OPT_COMPACT: usize = 3;
const OPT_SHARE: usize = 4;
const OPT_PRIORITY: usize = 5;
const OPT_STATUS: usize = 6;
const OPT_BADGE: usize = 7;
const OPT_HIDDEN: usize = 8;
const OPT_TOO_SMALL: usize = 9;
const OPT_QUIT_HINT: usize = 10;
const OPT_BRAND: usize = 11;
const OPTION_COUNT: usize = 12;
const OPTION_ROWS: usize = OPTION_COUNT * 2;
const LABEL_WIDTH: u16 = 22;
const NARROW_BAR_WIDTH: u16 = 44;
const ALIGNMENT_BAR_WIDTH: u16 = 64;
const TOO_SMALL_WIDTH: u16 = 44;
const TOO_SMALL_HEIGHT: u16 = 7;
const CUSTOM_QUIT_HINT: &str = "q Quit · resize to continue";
const STATE_LABEL_WIDTH: u16 = 15;
const STATE_COLUMN_WIDTH: u16 = 40;
const STATE_SPECIMEN_WIDTH: u16 = 39;
const STATE_NARROW_WIDTH: u16 = 24;
const STATE_COUNT: usize = 5;
const STATE_GRID_WIDTH: u16 = STATE_LABEL_WIDTH + STATE_COLUMN_WIDTH * 3;
const LEVEL_PANEL_GAP: u16 = 2;
const STATE_GRID_ROWS: usize = STATE_COUNT + 1;
const STATE_PANEL_ROWS: usize = STATE_GRID_ROWS + 1;

const ON_OFF: &[&str] = &["off", "on"];
const NEEDS_NARROW: &[(usize, usize)] = &[(OPT_WIDTH, 1)];
const NEEDS_SHARE: &[(usize, usize)] = &[(OPT_WIDTH, 1), (OPT_STATUS, 1)];
const AUTO_NOTE: &str = "Auto: pinned when the key is in theme.hintbar.pinned_keys (Esc) · Hint::pinned keeps F5 · Normal drops first";

#[derive(Clone, Copy)]
struct OptionSpec {
    key: &'static str,
    label: &'static str,
    values: &'static [&'static str],
    toggle: bool,
    needs: &'static [(usize, usize)],
}

const OPTIONS: [OptionSpec; OPTION_COUNT] = [
    OptionSpec {
        key: "statushints.layer",
        label: "Top layer",
        values: &["screen", "menu", "modal"],
        toggle: false,
        needs: &[],
    },
    OptionSpec {
        key: "statushints.align",
        label: "Align",
        values: &["start", "center"],
        toggle: false,
        needs: &[],
    },
    OptionSpec {
        key: "statushints.width",
        label: "Bar width",
        values: &["full", "narrow"],
        toggle: false,
        needs: &[],
    },
    OptionSpec {
        key: "statushints.compact",
        label: "Compact keys",
        values: ON_OFF,
        toggle: true,
        needs: NEEDS_NARROW,
    },
    OptionSpec {
        key: "statushints.share",
        label: "Status share %",
        values: &["50", "30", "80"],
        toggle: false,
        needs: NEEDS_SHARE,
    },
    OptionSpec {
        key: "statushints.priority",
        label: "F5 priority",
        values: &["auto", "normal", "pinned"],
        toggle: false,
        needs: NEEDS_NARROW,
    },
    OptionSpec {
        key: "statushints.status",
        label: "Status",
        values: &["none", "plain", "warning", "error", "parts"],
        toggle: false,
        needs: &[],
    },
    OptionSpec {
        key: "statushints.badge",
        label: "EDIT badge",
        values: ON_OFF,
        toggle: true,
        needs: &[],
    },
    OptionSpec {
        key: "statushints.hidden",
        label: "Hidden files shown",
        values: ON_OFF,
        toggle: true,
        needs: &[],
    },
    OptionSpec {
        key: "statushints.too_small_min",
        label: "Too small below",
        values: &["72x20", "100x30"],
        toggle: false,
        needs: &[],
    },
    OptionSpec {
        key: "statushints.quit_hint",
        label: "Too-small quit hint",
        values: &["default", "custom"],
        toggle: false,
        needs: &[],
    },
    OptionSpec {
        key: "statushints.brand",
        label: "Too-small brand",
        values: &["title", "accent"],
        toggle: false,
        needs: &[],
    },
];

const OPTION_IDS: [Id; OPTION_COUNT] = [
    ID.sub("option_layer"),
    ID.sub("option_align"),
    ID.sub("option_width"),
    ID.sub("option_compact"),
    ID.sub("option_share"),
    ID.sub("option_priority"),
    ID.sub("option_status"),
    ID.sub("option_badge"),
    ID.sub("option_hidden"),
    ID.sub("option_too_small"),
    ID.sub("option_quit_hint"),
    ID.sub("option_brand"),
];
const OPTION_SCROLL_OWNERS: [Id; 3] = [
    OPTION_IDS[OPT_COMPACT],
    OPTION_IDS[OPT_BADGE],
    OPTION_IDS[OPT_HIDDEN],
];

const OPTION_FOCUS_ROWS: [(Id, usize); OPTION_COUNT] = [
    (OPTION_IDS[0], 0),
    (OPTION_IDS[1], 2),
    (OPTION_IDS[2], 4),
    (OPTION_IDS[3], 6),
    (OPTION_IDS[4], 8),
    (OPTION_IDS[5], 10),
    (OPTION_IDS[6], 12),
    (OPTION_IDS[7], 14),
    (OPTION_IDS[8], 16),
    (OPTION_IDS[9], 18),
    (OPTION_IDS[10], 20),
    (OPTION_IDS[11], 22),
];

const OPTION_DEFAULTS: [usize; OPTION_COUNT] = [0; OPTION_COUNT];
const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];
const STATE_NAMES: [&str; 5] = ["Rest", "Overflow", "Error", "Editing", "Narrow"];
const COLUMN_NAMES: [&str; 3] = ["Start", "Center", "Pinned F5"];

const CLI_ARGUMENTS: &[(&str, &str)] = &[
    (
        "--theme <THEME>",
        "Theme to use: junie or paper [default: junie]",
    ),
    (
        "--color <LEVEL>",
        "Colour level: truecolor, 256, 16, or none",
    ),
    ("--page <NAME>", "The page to open [default: Overview]"),
    (
        "--motion <MOTION>",
        "Animation mode: full, reduced, or paused [default: full]",
    ),
    ("--frame <N>", "Frame to render while paused (0–10000)"),
    ("-h, --help", "Print help"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ScrollFocusFrame {
    area: Rect,
    focused: Option<Id>,
    content_len: usize,
}

#[derive(Debug)]
enum PlaygroundRow {
    Caption(&'static str),
    Layer {
        label: &'static str,
        layer: Option<HintLayer>,
        id: Id,
    },
    Resolved {
        layer: HintLayer,
        status: Status,
    },
    Detail(String),
    Alignment {
        label: &'static str,
        centered: bool,
        id: Id,
    },
    Priority {
        label: &'static str,
        width: Option<u16>,
        id: Id,
    },
    Bar {
        label: &'static str,
        layer: HintLayer,
        id: Id,
        status: Status,
    },
    ToggleHints,
    Posts,
    Job,
    TooSmall,
    Blank,
    Argument(String),
}

#[derive(Debug)]
pub struct StatusHintsPage {
    page_tabs: TabsState,
    options: [usize; OPTION_COUNT],
    option_states: [SelectState; OPTION_COUNT],
    play_scroll: ScrollState,
    options_scroll: ScrollState,
    states_scroll: ScrollState,
    play_scroll_frame: Cell<Option<ScrollFocusFrame>>,
    play_reveal_request: Cell<Option<Id>>,
    options_scroll_frame: Cell<Option<ScrollFocusFrame>>,
    options_reveal_request: Cell<Option<Id>>,
    last_page_width: Cell<Option<u16>>,
    compare_levels: bool,
    job: Option<f64>,
    spinner_frame: usize,
    paused_motion: bool,
    next_tick: Option<Moment>,
}

impl StatusHintsPage {
    pub fn new() -> Self {
        Self {
            page_tabs: TabsState::default(),
            options: OPTION_DEFAULTS,
            option_states: std::array::from_fn(|_| SelectState::default()),
            play_scroll: ScrollState::default(),
            options_scroll: ScrollState::default(),
            states_scroll: ScrollState::default(),
            play_scroll_frame: Cell::new(None),
            play_reveal_request: Cell::new(None),
            options_scroll_frame: Cell::new(None),
            options_reveal_request: Cell::new(None),
            last_page_width: Cell::new(None),
            compare_levels: false,
            job: None,
            spinner_frame: 0,
            paused_motion: false,
            next_tick: None,
        }
    }

    fn section(&self) -> usize {
        match self.page_tabs.active().or(self.page_tabs.cursor()) {
            Some(ItemKey::Index(index)) => index.min(SECTIONS.len().saturating_sub(1)),
            _ => 0,
        }
    }

    fn option_enabled(&self, index: usize) -> bool {
        OPTIONS[index]
            .needs
            .iter()
            .all(|(dependency, value)| self.options[*dependency] == *value)
    }

    fn hint_priority(&self) -> u8 {
        match self.options[OPT_PRIORITY] {
            1 => termrock::shell::HINT_NORMAL,
            2 => termrock::shell::HINT_PINNED,
            _ => termrock::shell::HINT_AUTO,
        }
    }

    fn option_detail(&self, index: usize) -> String {
        let spec = OPTIONS[index];
        let dependency = match spec.needs {
            [] => String::new(),
            [(option, value)] => format!(
                "Needs {} = {}",
                OPTIONS[*option].label, OPTIONS[*option].values[*value]
            ),
            [(first, first_value), (second, second_value)] => format!(
                "Needs {} = {} and {} = {}",
                OPTIONS[*first].label,
                OPTIONS[*first].values[*first_value],
                OPTIONS[*second].label,
                OPTIONS[*second].values[*second_value]
            ),
            _ => String::from("Additional requirements"),
        };
        let default = format!("Default: {}", spec.values[OPTION_DEFAULTS[index]]);
        if dependency.is_empty() {
            default
        } else {
            format!("{dependency} · {default}")
        }
    }

    fn screen_layer(&self) -> HintLayer {
        let hidden = self.options[OPT_HIDDEN] == 1;
        let compact = self.options[OPT_COMPACT] == 1;
        let mut layer = HintLayer {
            hints: vec![
                hint("Enter", hint_label(compact, "Open"), 0),
                hint("Tab", hint_label(compact, "Next pane"), 0),
                hint("/", hint_label(compact, "Filter"), 0),
                hint(
                    "Ctrl+H",
                    if compact {
                        ""
                    } else if hidden {
                        "Hide hidden"
                    } else {
                        "Show hidden"
                    },
                    0,
                ),
                hint("?", hint_label(compact, "Help"), 0),
                hint("F5", hint_label(compact, "Refresh"), self.hint_priority()),
            ],
            centered: self.options[OPT_ALIGN] == 1,
            ..HintLayer::default()
        };
        layer.badge = (self.options[OPT_BADGE] == 1).then_some("EDIT");
        layer.status = match self.options[OPT_STATUS] {
            1 => Some(Cow::Borrowed("3 rows selected")),
            2 => Some(Cow::Borrowed("2 unsaved changes")),
            3 => Some(Cow::Borrowed("Connection lost")),
            4 => Some(Cow::Borrowed("saved payments.settlements")),
            _ => None,
        };
        layer
    }

    fn toggle_layer(&self) -> HintLayer {
        let mut layer = self.screen_layer();
        layer.status = Some(Cow::Borrowed(if self.options[OPT_HIDDEN] == 1 {
            "hidden files shown"
        } else {
            "hidden files hidden"
        }));
        layer
    }

    fn menu_layer(compact: bool) -> HintLayer {
        HintLayer {
            hints: vec![
                hint("↑↓", hint_label(compact, "Move"), 0),
                hint("Enter", hint_label(compact, "Choose"), 0),
                hint("Esc", hint_label(compact, "Close"), 0),
            ],
            ..HintLayer::default()
        }
    }

    fn modal_layer(compact: bool) -> HintLayer {
        HintLayer {
            hints: vec![
                hint("Enter", hint_label(compact, "Delete"), 0),
                hint("Esc", hint_label(compact, "Cancel"), 0),
            ],
            ..HintLayer::default()
        }
    }

    fn resolved_layer(&self) -> (HintLayer, Status) {
        let screen = self.screen_layer();
        let compact = self.options[OPT_COMPACT] == 1;
        let menu = Self::menu_layer(compact);
        let modal = Self::modal_layer(compact);
        let layers = [
            (self.options[OPT_LAYER] >= 2).then_some(&modal),
            (self.options[OPT_LAYER] >= 1).then_some(&menu),
            Some(&screen),
        ];
        let status = if self.options[OPT_LAYER] == 0 && self.options[OPT_STATUS] == 3 {
            Status::Error
        } else {
            Status::Ready
        };
        (
            HintBar::resolve(&layers).cloned().unwrap_or_default(),
            status,
        )
    }

    fn style_readout(&self) -> String {
        format!(
            "HintBarStyle · align {} · status_min_share {}% · compact {} · status {} · badge {} · F5 priority {}",
            if self.options[OPT_ALIGN] == 1 {
                "Center"
            } else {
                "Start"
            },
            OPTIONS[OPT_SHARE].values[self.options[OPT_SHARE]],
            if self.options[OPT_COMPACT] == 1 {
                "on"
            } else {
                "off"
            },
            OPTIONS[OPT_STATUS].values[self.options[OPT_STATUS]],
            if self.options[OPT_BADGE] == 1 {
                "EDIT"
            } else {
                "off"
            },
            OPTIONS[OPT_PRIORITY].values[self.options[OPT_PRIORITY]],
        )
    }

    fn playground_rows(&self, width: u16) -> Vec<PlaygroundRow> {
        let mut rows = Vec::new();
        let text_width = width.saturating_sub(2).max(1);
        let screen = self.screen_layer();
        let compact = self.options[OPT_COMPACT] == 1;
        let menu = (self.options[OPT_LAYER] >= 1).then(|| Self::menu_layer(compact));
        let modal = (self.options[OPT_LAYER] >= 2).then(|| Self::modal_layer(compact));
        let layers = [modal.clone(), menu.clone(), Some(screen.clone())];
        let (resolved, resolved_status) = self.resolved_layer();

        rows.push(PlaygroundRow::Caption(
            "Layers · HintBar::resolve: the topmost layer that exists wins",
        ));
        rows.push(PlaygroundRow::Layer {
            label: "modal layer",
            layer: layers[0].clone(),
            id: ID.sub("layer_modal"),
        });
        rows.push(PlaygroundRow::Layer {
            label: "menu layer",
            layer: layers[1].clone(),
            id: ID.sub("layer_menu"),
        });
        rows.push(PlaygroundRow::Layer {
            label: "screen layer",
            layer: layers[2].clone(),
            id: ID.sub("layer_screen"),
        });
        rows.push(PlaygroundRow::Resolved {
            layer: resolved,
            status: resolved_status,
        });
        push_wrapped_detail(
            &mut rows,
            "Focused controls may supply their own native hint layer.",
            text_width,
        );
        push_wrapped_detail(&mut rows, &self.style_readout(), text_width);

        rows.push(PlaygroundRow::Caption("Alignment · HintAlign"));
        rows.push(PlaygroundRow::Alignment {
            label: "HintAlign::Start",
            centered: false,
            id: ID.sub("align_start"),
        });
        rows.push(PlaygroundRow::Alignment {
            label: "HintAlign::Center",
            centered: true,
            id: ID.sub("align_center"),
        });
        rows.push(PlaygroundRow::Alignment {
            label: "Theme::centred_hints",
            centered: true,
            id: ID.sub("align_layer_centered"),
        });

        rows.push(PlaygroundRow::Caption(
            "Priorities · HintPriority: normal hints drop from the right, pinned ones last",
        ));
        rows.push(PlaygroundRow::Priority {
            label: "all fit",
            width: None,
            id: ID.sub("priority_fit"),
        });
        rows.push(PlaygroundRow::Priority {
            label: "36 columns",
            width: Some(36),
            id: ID.sub("priority_narrow"),
        });
        rows.push(PlaygroundRow::Priority {
            label: "24 columns",
            width: Some(24),
            id: ID.sub("priority_tight"),
        });
        push_wrapped_detail(&mut rows, AUTO_NOTE, text_width);

        rows.push(PlaygroundRow::Caption("Statuses · the right edge"));
        rows.push(PlaygroundRow::Bar {
            label: "render_status",
            layer: status_layer(Some("Saved")),
            id: ID.sub("status_plain"),
            status: Status::Ready,
        });
        rows.push(PlaygroundRow::Bar {
            label: "render_toned",
            layer: status_layer(Some("Connection lost")),
            id: ID.sub("status_error"),
            status: Status::Error,
        });
        rows.push(PlaygroundRow::Bar {
            label: "EDIT badge",
            layer: badge_layer(),
            id: ID.sub("status_badge"),
            status: Status::Ready,
        });
        rows.push(PlaygroundRow::Bar {
            label: "Warning text",
            layer: status_layer(Some("2 unsaved changes")),
            id: ID.sub("status_warning"),
            status: Status::Ready,
        });
        rows.push(PlaygroundRow::Bar {
            label: "Status parts",
            layer: status_layer(Some("saved payments.settlements")),
            id: ID.sub("status_parts"),
            status: Status::Ready,
        });
        push_wrapped_detail(
            &mut rows,
            "Status text is shown at the right edge of the hint bar.",
            text_width,
        );

        rows.push(PlaygroundRow::Caption("Toggle hints and the reveal key"));
        rows.push(PlaygroundRow::ToggleHints);
        push_wrapped_detail(
            &mut rows,
            "The hidden-files label and status change with the option.",
            text_width,
        );

        rows.push(PlaygroundRow::Caption(
            "Status line · StatusLine: the shell's footer message",
        ));
        rows.push(PlaygroundRow::Posts);
        push_wrapped_detail(
            &mut rows,
            "Info and warning expire after 5 s; error messages stay visible.",
            text_width,
        );

        rows.push(PlaygroundRow::Caption(
            "Motion · --motion full | reduced | paused (Inspector: shell.motion)",
        ));
        rows.push(PlaygroundRow::Job);
        push_wrapped_detail(&mut rows, self.motion_note(), text_width);

        rows.push(PlaygroundRow::Caption(
            "Terminal too small · TooSmall: what an app draws below its minimum size",
        ));
        rows.push(PlaygroundRow::TooSmall);
        rows.extend((1..TOO_SMALL_HEIGHT).map(|_| PlaygroundRow::Blank));

        rows.push(PlaygroundRow::Caption(
            "Command line · Showcase CLI options",
        ));
        for (flag, help) in CLI_ARGUMENTS {
            push_hanging_argument(&mut rows, flag, help, text_width);
        }
        rows
    }

    fn playground_focus_rows(&self, rows: &[PlaygroundRow]) -> Vec<(Id, usize)> {
        let mut focus_rows = Vec::new();
        for (row, item) in rows.iter().enumerate() {
            match item {
                PlaygroundRow::Posts => {
                    focus_rows.extend([(POST_INFO, row), (POST_WARNING, row), (POST_ERROR, row)]);
                }
                PlaygroundRow::Job => focus_rows.push((RUN_JOB, row)),
                _ => {}
            }
        }
        focus_rows
    }

    fn draw_playground(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let rows = self.playground_rows(area.width);
        let content_len = rows.len();
        let focus_rows = self.playground_focus_rows(&rows);
        let focused = focused_owner(ui, &focus_rows);
        let (view_state, request, focus_frame) = scroll_view_revealing_focus(
            &self.play_scroll,
            area,
            content_len,
            focused,
            self.play_scroll_frame.get(),
            self.play_reveal_request.get(),
            &focus_rows,
        );
        self.play_reveal_request.set(request);
        self.play_scroll_frame.set(Some(focus_frame));
        let view = ScrollRegion::new(PLAY_SCROLL).preserve_underlay(true).draw(
            ui,
            area,
            &view_state,
            content_len,
        );
        let projected = ScrollRegion::view(&view_state, view, content_len);
        let first = projected.offset().min(content_len);
        let end = content_len.min(first.saturating_add(usize::from(view.height)));

        if let Some(panel_start) = rows
            .iter()
            .position(|row| matches!(row, PlaygroundRow::TooSmall))
            .filter(|start| {
                let panel_end = start.saturating_add(usize::from(TOO_SMALL_HEIGHT));
                *start < end && panel_end > first
            })
        {
            self.draw_too_small_panel(ui, view, first, panel_start);
        }

        ui.with_area(view, |ui| {
            for (source, row) in rows.iter().enumerate() {
                if source < first || source >= end {
                    self.register_playground_focus_only(ui, row);
                    continue;
                }
                if matches!(row, PlaygroundRow::TooSmall | PlaygroundRow::Blank) {
                    continue;
                }
                let y = view
                    .y
                    .saturating_add(u16::try_from(source - first).unwrap_or(u16::MAX));
                self.draw_play_row(ui, row, Rect::new(view.x, y, view.width, 1));
            }
        });
    }

    fn register_playground_focus_only(&self, ui: &mut Ui<'_>, row: &PlaygroundRow) {
        match row {
            PlaygroundRow::Posts => {
                for id in [POST_INFO, POST_WARNING, POST_ERROR] {
                    ui.register_focus_only(id, Focusability::Focusable);
                    publish_page_scroll_bindings(ui, id);
                }
            }
            PlaygroundRow::Job => {
                ui.register_focus_only(RUN_JOB, Focusability::Focusable);
                publish_page_scroll_bindings(ui, RUN_JOB);
            }
            _ => {}
        }
    }

    fn draw_too_small_panel(&self, ui: &mut Ui<'_>, view: Rect, offset: usize, source: usize) {
        let width = TOO_SMALL_WIDTH.min(view.width.saturating_sub(2));
        if width == 0 {
            return;
        }
        let source_y = u16::try_from(source).unwrap_or(u16::MAX);
        let offset_y = u16::try_from(offset).unwrap_or(u16::MAX);
        let y = if source >= offset {
            view.y.saturating_add(source_y.saturating_sub(offset_y))
        } else {
            view.y.saturating_sub(offset_y.saturating_sub(source_y))
        };
        let panel_area = Rect::new(view.x.saturating_add(2), y, width, TOO_SMALL_HEIGHT);
        let panel = Panel::new(ID.sub("too_small_panel")).kind(PanelKind::Framed);
        let (minimum_width, minimum_height) = if self.options[OPT_TOO_SMALL] == 1 {
            (100, 30)
        } else {
            (72, 20)
        };
        let brand_parts = [(
            Part::TITLE,
            StylePatch::new().set_fg(Role::Custom(ui.theme_ref().color.accent)),
        )];
        ui.with_area(view, |ui| {
            panel.draw(ui, panel_area, |ui, inner| {
                let custom_quit_hint = |ui: &mut Ui<'_>, slot: Rect| {
                    let text = termrock::text::truncate(CUSTOM_QUIT_HINT, inner.width);
                    let width = termrock::text::width(&text).min(inner.width);
                    let x = inner
                        .x
                        .saturating_add(inner.width.saturating_sub(width) / 2);
                    let style = ui
                        .style(
                            termrock::Family::TOO_SMALL,
                            termrock::Variant::DEFAULT,
                            Part::ACTIONS,
                            StateFlags::empty(),
                        )
                        .style;
                    let _ = ui.paint_str(Rect::new(x, slot.y, width, 1), &text, style);
                };
                let mut screen = TooSmall::new(TOO_SMALL, "junie showcase")
                    .minimum(minimum_width, minimum_height);
                if self.options[OPT_QUIT_HINT] == 1 {
                    screen = screen.slot(Part::ACTIONS, &custom_quit_hint);
                }
                if self.options[OPT_BRAND] == 1 {
                    screen.patch_part(&brand_parts).draw(ui, inner);
                } else {
                    screen.draw(ui, inner);
                }
            });
        });
    }

    fn draw_play_row(&self, ui: &mut Ui<'_>, row: &PlaygroundRow, area: Rect) {
        match row {
            PlaygroundRow::Caption(text) => caption(ui, area, text),
            PlaygroundRow::Layer { label, layer, id } => {
                draw_layer_row(ui, area, LABEL_WIDTH, label, layer.as_ref(), *id)
            }
            PlaygroundRow::Resolved { layer, status } => {
                paint_label(ui, area, LABEL_WIDTH, "resolved →");
                let available = area.width.saturating_sub(LABEL_WIDTH);
                let width = if self.options[OPT_WIDTH] == 1 {
                    available.min(NARROW_BAR_WIDTH)
                } else {
                    available
                };
                let bar = Rect::new(area.x.saturating_add(LABEL_WIDTH), area.y, width, 1);
                HintBar::new(ID.sub("resolved"), layer)
                    .status(*status)
                    .draw(ui, bar);
            }
            PlaygroundRow::Detail(text) => detail(ui, area, text),
            PlaygroundRow::Alignment {
                label,
                centered,
                id,
            } => draw_bar_row(
                ui,
                area,
                LABEL_WIDTH,
                label,
                alignment_layer(*centered),
                *id,
                Status::Ready,
            ),
            PlaygroundRow::Priority { label, width, id } => draw_priority_row(
                ui,
                area,
                LABEL_WIDTH,
                label,
                *width,
                *id,
                self.hint_priority(),
            ),
            PlaygroundRow::Bar {
                label,
                layer,
                id,
                status,
            } => draw_bar_row(ui, area, LABEL_WIDTH, label, layer.clone(), *id, *status),
            PlaygroundRow::ToggleHints => draw_bar_row(
                ui,
                area,
                LABEL_WIDTH,
                "toggle_hint",
                self.toggle_layer(),
                ID.sub("hidden_toggle"),
                Status::Ready,
            ),
            PlaygroundRow::Posts => self.draw_post_buttons(ui, area),
            PlaygroundRow::Job => self.draw_job(ui, area),
            PlaygroundRow::Argument(text) => draw_argument(ui, area, text),
            PlaygroundRow::TooSmall | PlaygroundRow::Blank => {}
        }
    }

    fn draw_post_buttons(&self, ui: &mut Ui<'_>, area: Rect) {
        let mut x = area.x.saturating_add(2);
        for (id, label) in [
            (POST_INFO, "Post info"),
            (POST_WARNING, "Post warning"),
            (POST_ERROR, "Post error"),
        ] {
            let button = Button::new(id, label);
            let width = button
                .measure(
                    ui,
                    termrock::Constraints::loose(area.right().saturating_sub(x), 1),
                )
                .preferred
                .0;
            if width > 0 {
                button.draw(ui, Rect::new(x, area.y, width, 1));
            } else {
                ui.register_focus_only(id, Focusability::Focusable);
            }
            publish_page_scroll_bindings(ui, id);
            x = x.saturating_add(width).saturating_add(2);
        }
    }

    fn draw_job(&self, ui: &mut Ui<'_>, area: Rect) {
        let mut x = area.x.saturating_add(2);
        let run = Button::new(RUN_JOB, "Run job");
        let run_width = run
            .measure(
                ui,
                termrock::Constraints::loose(area.right().saturating_sub(x), 1),
            )
            .preferred
            .0;
        if run_width > 0 {
            run.draw(ui, Rect::new(x, area.y, run_width, 1));
        } else {
            ui.register_focus_only(RUN_JOB, Focusability::Focusable);
        }
        publish_page_scroll_bindings(ui, RUN_JOB);
        x = x.saturating_add(run_width).saturating_add(2);
        let spinner_width = 22.min(area.right().saturating_sub(x));
        let label = if self.job.is_some() {
            "Indexing"
        } else {
            "Idle"
        };
        Spinner::new(SPINNER)
            .label(label)
            .frame(self.spinner_frame)
            .draw(ui, Rect::new(x, area.y, spinner_width, 1));
        x = x.saturating_add(spinner_width).saturating_add(1);
        let progress_width = area.right().saturating_sub(x);
        ProgressBar::new(PROGRESS)
            .label("Progress")
            .ratio(self.job.unwrap_or(0.0))
            .done(self.job == Some(1.0))
            .draw(ui, Rect::new(x, area.y, progress_width, 1));
    }

    fn motion_note(&self) -> &'static str {
        if self.paused_motion {
            "Paused: ticks are frozen; --frame N selects the deterministic spinner frame."
        } else {
            "Full: the spinner turns and the job progress advances on each tick."
        }
    }

    fn draw_options(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let focused = focused_owner(ui, &OPTION_FOCUS_ROWS);
        let (view_state, request, focus_frame) = scroll_view_revealing_focus(
            &self.options_scroll,
            area,
            OPTION_ROWS,
            focused,
            self.options_scroll_frame.get(),
            self.options_reveal_request.get(),
            &OPTION_FOCUS_ROWS,
        );
        self.options_reveal_request.set(request);
        self.options_scroll_frame.set(Some(focus_frame));
        let view = ScrollRegion::new(OPTIONS_SCROLL)
            .preserve_underlay(true)
            .draw(ui, area, &view_state, OPTION_ROWS);
        let projected = ScrollRegion::view(&view_state, view, OPTION_ROWS);
        let first = projected.offset().min(OPTION_ROWS);
        let label_width = view.width.saturating_sub(24).min(26).max(12);
        let control_x = view.x.saturating_add(label_width).saturating_add(1);
        let control_width = view.right().saturating_sub(control_x);
        let end = OPTION_ROWS.min(first.saturating_add(usize::from(view.height)));
        ui.with_area(view, |ui| {
            for source in 0..OPTION_ROWS {
                let logical = source / 2;
                let spec = OPTIONS[logical];
                let visible = first <= source && source < end;
                if source % 2 == 1 {
                    if visible {
                        let y = view
                            .y
                            .saturating_add(u16::try_from(source - first).unwrap_or(u16::MAX));
                        detail(
                            ui,
                            Rect::new(view.x, y, view.width, 1),
                            &self.option_detail(logical),
                        );
                    }
                    continue;
                }

                let id = OPTION_IDS[logical];
                let enabled = self.option_enabled(logical);
                if !visible {
                    self.register_option_focus_only(ui, logical);
                    if spec.toggle {
                        publish_page_scroll_bindings(ui, id);
                    } else if !self.option_states[logical].is_open() {
                        publish_page_scroll_bindings(ui, id);
                    }
                    continue;
                }

                let y = view
                    .y
                    .saturating_add(u16::try_from(source - first).unwrap_or(u16::MAX));
                let row = Rect::new(view.x, y, view.width, 1);
                if spec.toggle {
                    if row.is_empty() {
                        self.register_option_focus_only(ui, logical);
                    } else {
                        Toggle::new(id, spec.label)
                            .on(self.options[logical] == 1)
                            .disabled(!enabled)
                            .draw(ui, row);
                    }
                    publish_page_scroll_bindings(ui, id);
                } else {
                    paint_label(ui, row, label_width, spec.label);
                    let control = Rect::new(control_x, y, control_width, 1);
                    if control.width > 0 {
                        Select::new(id)
                            .disabled(!enabled)
                            .popup_rows(u16::try_from(spec.values.len()).unwrap_or(u16::MAX))
                            .draw(ui, control, &self.option_states[logical], spec.values);
                    } else {
                        Select::new(id)
                            .disabled(!enabled)
                            .register_focus_only_in_form(ui, false);
                        if !self.option_states[logical].is_open() {
                            publish_page_scroll_bindings(ui, id);
                        }
                    }
                }
            }
        });
    }

    fn register_option_focus_only(&self, ui: &mut Ui<'_>, index: usize) {
        let id = OPTION_IDS[index];
        if OPTIONS[index].toggle {
            Toggle::new(id, OPTIONS[index].label)
                .disabled(!self.option_enabled(index))
                .register_focus_only_in_form(ui, false);
        } else {
            Select::new(id)
                .disabled(!self.option_enabled(index))
                .register_focus_only_in_form(ui, false);
        }
    }

    fn draw_states(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let header = Rect::new(area.x, area.y, area.width, 1);
        if header.is_empty() {
            Toggle::new(COMPARE_LEVELS, "Compare levels").register_focus_only_in_form(ui, false);
        } else {
            Toggle::new(COMPARE_LEVELS, "Compare levels")
                .on(self.compare_levels)
                .draw(ui, header);
        }
        publish_page_scroll_bindings(ui, COMPARE_LEVELS);
        let body = Rect {
            y: area.y.saturating_add(2),
            height: area.height.saturating_sub(2),
            ..area
        };
        let (content_len, _) = state_content_layout(body.width, body.height, self.compare_levels);
        let view = ScrollRegion::new(STATES_SCROLL)
            .preserve_underlay(true)
            .draw(ui, body, &self.states_scroll, content_len);
        let projected = ScrollRegion::view(&self.states_scroll, view, content_len);
        let first = projected.offset().min(content_len);
        ui.with_area(view, |ui| {
            for source in first..content_len.min(first.saturating_add(usize::from(view.height))) {
                let y = view
                    .y
                    .saturating_add(u16::try_from(source - first).unwrap_or(u16::MAX));
                let row_area = Rect::new(view.x, y, view.width, 1);
                if source == 0 {
                    if !self.compare_levels {
                        self.draw_state_columns(ui, row_area);
                    } else {
                        self.draw_compare_row(ui, row_area, source, view.width);
                    }
                    continue;
                }
                if self.compare_levels {
                    self.draw_compare_row(ui, row_area, source, view.width);
                } else {
                    let level = ui.theme_ref().capability.color;
                    let level_index = LEVELS
                        .iter()
                        .position(|candidate| *candidate == level)
                        .unwrap_or(0);
                    self.draw_state_row(ui, row_area, source - 1, level, level_index);
                }
            }
        });
    }

    fn draw_compare_row(&self, ui: &mut Ui<'_>, area: Rect, source: usize, width: u16) {
        let per_band = levels_per_band(width);
        let stride = STATE_PANEL_ROWS.saturating_add(usize::from(LEVEL_PANEL_GAP));
        let band = source / stride;
        let within = source % stride;
        if within >= STATE_PANEL_ROWS {
            return;
        }
        for panel_column in 0..per_band {
            let level_index = band.saturating_mul(per_band).saturating_add(panel_column);
            let Some(level) = LEVELS.get(level_index).copied() else {
                break;
            };
            let x = area.x.saturating_add(
                u16::try_from(panel_column)
                    .unwrap_or(u16::MAX)
                    .saturating_mul(STATE_GRID_WIDTH.saturating_add(LEVEL_PANEL_GAP)),
            );
            let panel_width = STATE_GRID_WIDTH.min(area.right().saturating_sub(x));
            let panel = Rect::new(x, area.y, panel_width, 1);
            if within == 0 {
                caption(ui, panel, level_label(level));
            } else if within == 1 {
                self.draw_state_columns(ui, panel);
            } else {
                self.draw_state_row(ui, panel, within - 2, level, level_index);
            }
        }
    }

    fn draw_state_columns(&self, ui: &mut Ui<'_>, area: Rect) {
        let label_width = STATE_LABEL_WIDTH;
        paint_label(ui, area, label_width, "State");
        for (index, name) in COLUMN_NAMES.iter().enumerate() {
            if !state_column_fits(area, index) {
                break;
            }
            let x = area
                .x
                .saturating_add(label_width)
                .saturating_add(STATE_COLUMN_WIDTH.saturating_mul(index as u16));
            let style = super::heading_style(ui, termrock::Family::PANEL, termrock::Part::TITLE);
            let _ = ui.paint_str(Rect::new(x, area.y, STATE_SPECIMEN_WIDTH, 1), name, style);
        }
    }

    fn draw_state_row(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        row: usize,
        level: ColorLevel,
        level_index: usize,
    ) {
        let label_width = STATE_LABEL_WIDTH;
        paint_label(ui, area, label_width, STATE_NAMES[row]);
        for column in 0..3 {
            if !state_column_fits(area, column) {
                break;
            }
            let x = area
                .x
                .saturating_add(label_width)
                .saturating_add(STATE_COLUMN_WIDTH.saturating_mul(column as u16));
            let mut cell = Rect::new(x, area.y, STATE_SPECIMEN_WIDTH, 1);
            let layer = state_layer(row, column);
            let status = if row == 2 {
                Status::Error
            } else {
                Status::Ready
            };
            if row == 4 {
                cell.width = STATE_NARROW_WIDTH.min(cell.width);
            }
            let projected = ui.theme_ref().downgrade(level);
            ui.with_theme(&projected, |ui| {
                HintBar::new(
                    ID.sub("state_cell")
                        .index(level_index * STATE_COUNT * 3 + row * 3 + column),
                    &layer,
                )
                .status(status)
                .draw(ui, cell);
            });
        }
    }

    fn update_options(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let content_len = cx
            .layout(OPTIONS_SCROLL)
            .map_or(OPTION_ROWS, |layout| layout.content_len);
        let mut response = update_focus_scroll(
            cx,
            OPTIONS_SCROLL,
            &mut self.options_scroll,
            content_len,
            &self.options_scroll_frame,
            &self.options_reveal_request,
            &OPTION_FOCUS_ROWS,
        );
        let mut page_up = false;
        let mut page_down = false;
        for index in 0..OPTION_COUNT {
            if OPTIONS[index].toggle {
                let mut value = self.options[index] == 1;
                response |= Toggle::new(OPTION_IDS[index], OPTIONS[index].label)
                    .on(value)
                    .disabled(!self.option_enabled(index))
                    .update(cx, &mut value)
                    .erase();
                self.options[index] = usize::from(value);
            } else {
                let id = OPTION_IDS[index];
                let was_open = self.option_states[index].is_open() || cx.is_open(id);
                response |= Select::new(OPTION_IDS[index])
                    .disabled(!self.option_enabled(index))
                    .popup_rows(u16::try_from(OPTIONS[index].values.len()).unwrap_or(u16::MAX))
                    .update_in_form(
                        cx,
                        &mut self.option_states[index],
                        &mut self.options[index],
                        OPTIONS[index].values,
                        false,
                    )
                    .erase();
                let is_open = self.option_states[index].is_open() || cx.is_open(id);
                if !was_open && !is_open {
                    let (up, down) = select_page_scroll_intents(cx, id);
                    page_up |= up;
                    page_down |= down;
                }
            }
        }
        response |= scroll_page(&mut self.options_scroll, page_up, page_down);
        response
    }

    fn update_playground(&mut self, cx: &mut Cx<'_>) -> (Response<()>, Option<PageStatus>) {
        let rows = self.playground_rows(self.last_page_width.get().unwrap_or(cx.viewport().width));
        let fallback_len = rows.len();
        let content_len = cx
            .layout(PLAY_SCROLL)
            .map_or(fallback_len, |layout| layout.content_len);
        let focus_rows = self.playground_focus_rows(&rows);
        let mut response = update_focus_scroll(
            cx,
            PLAY_SCROLL,
            &mut self.play_scroll,
            content_len,
            &self.play_scroll_frame,
            &self.play_reveal_request,
            &focus_rows,
        );
        let mut status = None;
        let info = Button::new(POST_INFO, "Post info").update(cx);
        if info.activated() {
            status = Some(PageStatus(String::from("Saved 3 rows")));
        }
        response |= info.erase();
        let warning = Button::new(POST_WARNING, "Post warning").update(cx);
        if warning.activated() {
            status = Some(PageStatus(String::from("2 rows skipped")));
        }
        response |= warning.erase();
        let error = Button::new(POST_ERROR, "Post error").update(cx);
        if error.activated() {
            status = Some(PageStatus(String::from("Connection lost")));
        }
        response |= error.erase();
        let run = Button::new(RUN_JOB, "Run job").update(cx);
        if run.activated() {
            self.job = Some(0.0);
            self.next_tick = None;
        }
        response |= run.erase();
        (response, status)
    }

    fn update_states(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let layout = cx.layout(STATES_SCROLL);
        let (width, height) = layout.map_or_else(
            || {
                let viewport = cx.viewport();
                (
                    self.last_page_width.get().unwrap_or(viewport.width),
                    viewport.height,
                )
            },
            |layout| {
                (
                    layout.cols,
                    u16::try_from(layout.viewport_len).unwrap_or(u16::MAX),
                )
            },
        );
        let (content_len, _) = state_content_layout(width, height, self.compare_levels);
        let mut response =
            ScrollRegion::new(STATES_SCROLL).update(cx, &mut self.states_scroll, content_len);
        let mut compare = self.compare_levels;
        response |= Toggle::new(COMPARE_LEVELS, "Compare levels")
            .on(compare)
            .update(cx, &mut compare)
            .erase();
        if compare != self.compare_levels {
            self.compare_levels = compare;
            self.states_scroll.scroll_to(0);
        }
        response
    }
}

impl Default for StatusHintsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for StatusHintsPage {
    fn title(&self) -> &'static str {
        "Status & hints"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let previous = self.section();
        let mut response = Tabs::new(PAGE_TABS)
            .update(cx, &mut self.page_tabs, SECTIONS)
            .erase();
        let section = self.section();
        let page_scroll_owners = match section {
            0 => &PLAYGROUND_SCROLL_OWNERS[..],
            1 => &OPTION_SCROLL_OWNERS[..],
            _ => &STATE_SCROLL_OWNERS[..],
        };
        let page_up = binding_intent(cx, PAGE_TABS, PAGE_SCROLL_UP)
            || page_scroll_owners
                .iter()
                .any(|owner| binding_intent(cx, *owner, PAGE_SCROLL_UP));
        let page_down = binding_intent(cx, PAGE_TABS, PAGE_SCROLL_DOWN)
            || page_scroll_owners
                .iter()
                .any(|owner| binding_intent(cx, *owner, PAGE_SCROLL_DOWN));
        if section != previous {
            self.play_scroll.scroll_to(0);
            self.options_scroll.scroll_to(0);
            self.states_scroll.scroll_to(0);
            self.play_scroll_frame.set(None);
            self.play_reveal_request.set(None);
            self.options_scroll_frame.set(None);
            self.options_reveal_request.set(None);
        }
        let mut status = None;
        if section == 0 {
            let (updated, page_status) = self.update_playground(cx);
            response |= updated;
            status = page_status;
        } else if section == 1 {
            response |= self.update_options(cx);
        } else {
            response |= self.update_states(cx);
        }
        if page_up || page_down {
            let scroll = match section {
                0 => &mut self.play_scroll,
                1 => &mut self.options_scroll,
                _ => &mut self.states_scroll,
            };
            response |= scroll_page(scroll, page_up, page_down);
        }

        let active = self.job.is_some_and(|progress| progress < 1.0);
        if active && !self.paused_motion {
            let now = cx.now();
            let interval = Duration::from_millis(80);
            let deadline = *self
                .next_tick
                .get_or_insert_with(|| now.saturating_add(interval));
            if cx.update_cause() == UpdateCause::Tick && now >= deadline {
                if let Some(progress) = self.job {
                    let next = (progress + 0.02).min(1.0);
                    self.job = Some(next);
                    response |= Response::changed();
                    if next == 1.0 {
                        self.next_tick = None;
                        status = Some(PageStatus(String::from("Indexing finished")));
                    }
                }
                self.spinner_frame = self.spinner_frame.wrapping_add(1);
                response = response.repaint();
                if self.job.is_some_and(|progress| progress < 1.0) {
                    self.next_tick = Some(now.saturating_add(interval));
                }
            }
            if cx.top_layer() == termrock::LayerId::PAGE
                && let Some(deadline) = self.next_tick
            {
                cx.request_repaint_at(deadline);
            }
        } else {
            self.next_tick = None;
        }
        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        self.last_page_width.set(Some(area.width));
        frame(
            ui,
            area,
            self.title(),
            "Live status, hint layers, options, and native state",
            |ui, body| {
                if body.is_empty() {
                    return;
                }
                let tabs_area = Rect {
                    height: 2.min(body.height),
                    ..body
                };
                let tabs = Tabs::new(PAGE_TABS);
                let layout = tabs.layout_default(
                    tabs_area,
                    tabs_area.intersection(ui.full()),
                    &self.page_tabs,
                    SECTIONS,
                    None,
                );
                if tabs.draw_layout(ui, &layout).is_err() {
                    return;
                }
                ui.publish_additional_bindings(
                    PAGE_TABS,
                    ui.state(PAGE_TABS),
                    PAGE_SCROLL_BINDINGS,
                );
                let content_top = body.y.saturating_add(3);
                if content_top >= body.bottom() {
                    return;
                }
                let content = Rect {
                    y: content_top,
                    height: body.bottom().saturating_sub(content_top),
                    ..body
                };
                match self.section() {
                    0 => self.draw_playground(ui, content),
                    1 => self.draw_options(ui, content),
                    _ => self.draw_states(ui, content),
                }
            },
        );
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(PAGE_TABS).contains(StateFlags::FOCUSED) {
            return TAB_HINTS;
        }
        if self.section() == 1 {
            if OPTION_IDS
                .iter()
                .any(|id| ui.state(*id).contains(StateFlags::FOCUSED))
            {
                return SELECT_HINTS;
            }
            if ui.state(OPTIONS_SCROLL).contains(StateFlags::FOCUSED) {
                return SCROLL_HINTS;
            }
        }
        if self.section() == 2 {
            if ui.state(COMPARE_LEVELS).contains(StateFlags::FOCUSED) {
                return TOGGLE_HINTS;
            }
            if ui.state(STATES_SCROLL).contains(StateFlags::FOCUSED) {
                return SCROLL_HINTS;
            }
        }
        if [POST_INFO, POST_WARNING, POST_ERROR, RUN_JOB]
            .into_iter()
            .any(|id| ui.state(id).contains(StateFlags::FOCUSED))
        {
            return ACTIVATE_HINTS;
        }
        if ui.state(PLAY_SCROLL).contains(StateFlags::FOCUSED) {
            return SCROLL_HINTS;
        }
        &[]
    }

    fn seek_paused(&mut self, frame: usize) {
        self.paused_motion = true;
        self.spinner_frame = frame;
        self.next_tick = None;
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        OPTIONS
            .iter()
            .position(|option| option.key == key)
            .map(|index| self.options[index])
    }

    fn set_option(&mut self, key: &str, value: usize) {
        if let Some(index) = OPTIONS.iter().position(|option| option.key == key) {
            self.options[index] = value.min(OPTIONS[index].values.len().saturating_sub(1));
        }
    }
}

fn hint(key: &'static str, label: &'static str, priority: u8) -> Hint {
    Hint {
        key: HintKey::Label(key),
        label,
        priority,
    }
}

fn alignment_layer(centered: bool) -> HintLayer {
    HintLayer {
        hints: vec![
            hint("↑↓", "Move", 0),
            hint("Enter", "Open", 0),
            hint("Esc", "Back", 0),
        ],
        centered,
        ..HintLayer::default()
    }
}

fn priority_layer(priority: u8) -> HintLayer {
    HintLayer {
        hints: vec![
            hint("Enter", "Open", 0),
            hint("Tab", "Next pane", 0),
            hint("Space", "Select", 0),
            hint("Esc", "Back", 0),
            hint("F5", "Refresh", priority),
        ],
        ..HintLayer::default()
    }
}

fn status_layer(status: Option<&'static str>) -> HintLayer {
    HintLayer {
        hints: vec![hint("Ctrl+S", "Save", 0), hint("Esc", "Back", 0)],
        status: status.map(Cow::Borrowed),
        ..HintLayer::default()
    }
}

fn badge_layer() -> HintLayer {
    HintLayer {
        hints: vec![hint("Ctrl+S", "Save", 0), hint("Esc", "Back", 0)],
        badge: Some("EDIT"),
        ..HintLayer::default()
    }
}

fn state_layer(row: usize, column: usize) -> HintLayer {
    let mut hints = vec![
        hint("↑↓", "Move", 0),
        hint("Enter", "Open", 0),
        hint("/", "Filter", 0),
        hint("Esc", "Back", 0),
    ];
    if column == 2 {
        hints.push(hint("F5", "Refresh", termrock::shell::HINT_PINNED));
    }
    let mut layer = HintLayer {
        hints,
        centered: column == 1,
        ..HintLayer::default()
    };
    match row {
        1 => {
            layer.status = Some(Cow::Borrowed(
                "Saved 3 rows to payments.settlements · 2 warnings",
            ))
        }
        2 => layer.status = Some(Cow::Borrowed("Connection lost")),
        3 => layer.badge = Some("EDIT"),
        _ => {}
    }
    layer
}

fn level_label(level: ColorLevel) -> &'static str {
    match level {
        ColorLevel::TrueColor => "TrueColor",
        ColorLevel::Ansi256 => "Ansi256",
        ColorLevel::Ansi16 => "Ansi16",
        ColorLevel::Mono => "Mono",
        _ => "Other",
    }
}

fn levels_per_band(width: u16) -> usize {
    [4usize, 2]
        .into_iter()
        .find(|count| {
            STATE_GRID_WIDTH.saturating_mul(*count as u16)
                + LEVEL_PANEL_GAP.saturating_mul((*count as u16).saturating_sub(1))
                <= width
        })
        .unwrap_or(1)
}

fn state_column_fits(area: Rect, column: usize) -> bool {
    let x = area.x.saturating_add(STATE_LABEL_WIDTH).saturating_add(
        STATE_COLUMN_WIDTH.saturating_mul(u16::try_from(column).unwrap_or(u16::MAX)),
    );
    x.saturating_add(STATE_SPECIMEN_WIDTH) <= area.right()
}

fn state_content_layout(width: u16, height: u16, compare_levels: bool) -> (usize, u16) {
    let content_len = state_content_len(width, compare_levels);
    if content_len > usize::from(height) {
        let view_width = width.saturating_sub(1);
        (state_content_len(view_width, compare_levels), view_width)
    } else {
        (content_len, width)
    }
}

fn state_content_len(width: u16, compare_levels: bool) -> usize {
    if !compare_levels {
        return STATE_GRID_ROWS;
    }
    let per_band = levels_per_band(width);
    let bands = LEVELS.len().div_ceil(per_band);
    bands.saturating_mul(STATE_PANEL_ROWS).saturating_add(
        bands
            .saturating_sub(1)
            .saturating_mul(usize::from(LEVEL_PANEL_GAP)),
    )
}

fn focus_row(owner: Id, rows: &[(Id, usize)]) -> Option<usize> {
    rows.iter()
        .find_map(|(candidate, row)| (*candidate == owner).then_some(*row))
}

fn focused_owner(ui: &Ui<'_>, rows: &[(Id, usize)]) -> Option<Id> {
    rows.iter()
        .map(|(owner, _)| *owner)
        .find(|owner| ui.state(*owner).contains(StateFlags::FOCUSED))
}

fn scroll_view_revealing_focus(
    state: &ScrollState,
    area: Rect,
    content_len: usize,
    focused: Option<Id>,
    previous: Option<ScrollFocusFrame>,
    requested: Option<Id>,
    focus_rows: &[(Id, usize)],
) -> (ScrollState, Option<Id>, ScrollFocusFrame) {
    let transitioned = previous.is_none_or(|frame| {
        frame.area != area || frame.focused != focused || frame.content_len != content_len
    });
    let requested = requested.filter(|owner| Some(*owner) == focused);
    let owner = requested.or_else(|| transitioned.then_some(focused).flatten());
    let mut view = ScrollRegion::view(state, area, content_len);
    if let Some(row) = owner.and_then(|owner| focus_row(owner, focus_rows)) {
        let _ = view.reveal_span(row, 1);
    }
    let pending = owner.filter(|_| view.offset() != state.offset());
    (
        view,
        pending,
        ScrollFocusFrame {
            area,
            focused,
            content_len,
        },
    )
}

fn update_focus_scroll(
    cx: &mut Cx<'_>,
    scroll_id: Id,
    state: &mut ScrollState,
    content_len: usize,
    frame: &Cell<Option<ScrollFocusFrame>>,
    requested: &Cell<Option<Id>>,
    focus_rows: &[(Id, usize)],
) -> Response<()> {
    let wheel = cx
        .intents(scroll_id)
        .any(|intent| matches!(intent, Intent::Wheel { .. }));
    let mut response = ScrollRegion::new(scroll_id).update(cx, state, content_len);
    let focused = cx
        .focused_in_scope(ScopeId::ROOT)
        .filter(|owner| focus_row(*owner, focus_rows).is_some());
    let previous = frame.get();

    if wheel {
        requested.set(None);
        if let Some(mut previous) = previous {
            previous.focused = focused;
            previous.content_len = content_len;
            frame.set(Some(previous));
        }
        return response;
    }

    let pending = requested.take().filter(|owner| Some(*owner) == focused);
    let focus_changed = previous.map_or(focused.is_some(), |frame| frame.focused != focused);
    let owner = pending.or_else(|| focus_changed.then_some(focused).flatten());
    if let Some(owner) = owner
        && let Some(row) = focus_row(owner, focus_rows)
    {
        let before = state.offset();
        let _ = ScrollRegion::new(scroll_id).prepare(cx, state, content_len);
        let _ = state.reveal_span(row, 1);
        if state.offset() != before {
            response |= Response::changed();
        }
    }
    response
}

fn binding_intent(cx: &Cx<'_>, owner: Id, action: ActionKey) -> bool {
    cx.intents(owner)
        .any(|intent| matches!(intent, Intent::Binding(found) if found == action))
}

fn publish_page_scroll_bindings(ui: &mut Ui<'_>, owner: Id) {
    let flags = ui.state(owner);
    if flags.contains(StateFlags::FOCUSED) && !ui.is_inert() && !ui.is_covered_by_native_layer() {
        ui.publish_additional_bindings(owner, flags, PAGE_SCROLL_BINDINGS);
    }
}

fn select_page_scroll_intents(cx: &Cx<'_>, owner: Id) -> (bool, bool) {
    let select: termrock::LabelSelect<'static> = Select::new(owner);
    let bindings = termrock::Bindings::bindings(&select, termrock::BindingState::default());
    let mut page_up = false;
    let mut page_down = false;
    for intent in cx.intents(owner) {
        match intent {
            Intent::Binding(action) => {
                if action == PAGE_SCROLL_UP
                    || matches!(
                        Binding::command(bindings, action),
                        Some(termrock::SelectCmd::PageUp)
                    )
                {
                    page_up = true;
                } else if action == PAGE_SCROLL_DOWN
                    || matches!(
                        Binding::command(bindings, action),
                        Some(termrock::SelectCmd::PageDown)
                    )
                {
                    page_down = true;
                }
            }
            Intent::Key(key) => match key.code {
                KeyCode::PageUp => page_up = true,
                KeyCode::PageDown => page_down = true,
                _ => {}
            },
            _ => {}
        }
    }
    (page_up, page_down)
}

fn scroll_page(scroll: &mut ScrollState, page_up: bool, page_down: bool) -> Response<()> {
    if !page_up && !page_down {
        return Response::ignored();
    }
    let before = scroll.offset();
    if page_up {
        scroll.page_up();
    } else {
        scroll.page_down();
    }
    if scroll.offset() == before {
        Response::consumed()
    } else {
        Response::changed()
    }
}

fn hint_label(compact: bool, label: &'static str) -> &'static str {
    if compact { "" } else { label }
}

fn push_wrapped_detail(rows: &mut Vec<PlaygroundRow>, text: &str, width: u16) {
    rows.extend(
        termrock::text::wrap(text, width.max(1))
            .into_iter()
            .map(PlaygroundRow::Detail),
    );
}

fn push_hanging_argument(rows: &mut Vec<PlaygroundRow>, flag: &str, help: &str, width: u16) {
    const ARG_COLUMN: usize = 24;
    let prefix = format!("{flag:<ARG_COLUMN$}");
    if termrock::text::width(&prefix).saturating_add(termrock::text::width(help)) <= width {
        rows.push(PlaygroundRow::Argument(format!("{prefix}{help}")));
        return;
    }
    let room = width.saturating_sub(ARG_COLUMN as u16).max(10);
    for (index, line) in termrock::text::wrap(help, room).into_iter().enumerate() {
        if index == 0 {
            rows.push(PlaygroundRow::Argument(format!("{prefix}{line}")));
        } else {
            rows.push(PlaygroundRow::Argument(format!(
                "{}{line}",
                " ".repeat(ARG_COLUMN)
            )));
        }
    }
}

fn draw_argument(ui: &mut Ui<'_>, area: Rect, text: &str) {
    detail(ui, area, text);
}

fn caption(ui: &mut Ui<'_>, area: Rect, text: &str) {
    let style = super::heading_style(ui, termrock::Family::PANEL, termrock::Part::TITLE);
    let _ = ui.paint_str(area, text, style);
}

fn detail(ui: &mut Ui<'_>, area: Rect, text: &str) {
    let style = ui.surface_style();
    let _ = ui.paint_str(area, text, style);
}

fn paint_label(ui: &mut Ui<'_>, area: Rect, width: u16, text: &str) {
    if width == 0 {
        return;
    }
    let style = ui.surface_style();
    let _ = ui.paint_str(Rect { width, ..area }, text, style);
}

fn draw_layer_row(
    ui: &mut Ui<'_>,
    area: Rect,
    label_width: u16,
    label: &str,
    layer: Option<&HintLayer>,
    id: Id,
) {
    paint_label(ui, area, label_width, label);
    let bar = Rect::new(
        area.x.saturating_add(label_width),
        area.y,
        area.width.saturating_sub(label_width),
        1,
    );
    if let Some(layer) = layer {
        HintBar::new(id, layer).draw(ui, bar);
    } else {
        detail(ui, bar, "(none)");
    }
}

fn draw_bar_row(
    ui: &mut Ui<'_>,
    area: Rect,
    label_width: u16,
    label: &str,
    layer: HintLayer,
    id: Id,
    status: Status,
) {
    paint_label(ui, area, label_width, label);
    let width = area
        .width
        .saturating_sub(label_width)
        .min(ALIGNMENT_BAR_WIDTH);
    let bar = Rect::new(area.x.saturating_add(label_width), area.y, width, 1);
    HintBar::new(id, &layer).status(status).draw(ui, bar);
}

fn draw_priority_row(
    ui: &mut Ui<'_>,
    area: Rect,
    label_width: u16,
    label: &str,
    max_width: Option<u16>,
    id: Id,
    priority: u8,
) {
    paint_label(ui, area, label_width, label);
    let available = area.width.saturating_sub(label_width);
    let width = max_width.map_or(available, |width| width.min(available));
    let bar = Rect::new(area.x.saturating_add(label_width), area.y, width, 1);
    let layer = priority_layer(priority);
    HintBar::new(id, &layer).draw(ui, bar);
}
