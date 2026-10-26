use termrock::{
    Button, Checkbox, Cx, FgStep, FrameRead, Id, ItemKey, Panel, PanelKind, Rect, Response,
    ScrollRegion, ScrollState, Select, SelectState, StateFlags, Tabs, TabsState, Ui, Variant,
    author::VerticalWindow,
    feedback::{Notice, NoticeKind},
};

use super::{Page, PageStatus, PageUpdate, card_panel_window, frame};

const PAGE_TABS: Id = Id::root("showcase_ui::pages::notices.page_tabs");
const PLAY_SCROLL: Id = Id::root("showcase_ui::pages::notices.play_scroll");
const OPTIONS_SCROLL: Id = Id::root("showcase_ui::pages::notices.options_scroll");
const STATES_SCROLL: Id = Id::root("showcase_ui::pages::notices.states_scroll");
const CONNECTION_PANEL: Id = Id::root("showcase_ui::pages::notices.connection_panel");
const QUERY_PANEL: Id = Id::root("showcase_ui::pages::notices.query_panel");
const INLINE_PANEL: Id = Id::root("showcase_ui::pages::notices.inline_panel");
const STATES_DETAIL_PANEL: Id = Id::root("showcase_ui::pages::notices.states_detail_panel");
const STATES_HINT_PANEL: Id = Id::root("showcase_ui::pages::notices.states_hint_panel");
const CONNECTION_REVEAL: Id = Id::root("showcase_ui::pages::notices.reveal.connection");
const QUERY_REVEAL: Id = Id::root("showcase_ui::pages::notices.reveal.query");
const INLINE_REVEALS: [Id; 4] = [
    Id::root("showcase_ui::pages::notices.reveal.inline.0"),
    Id::root("showcase_ui::pages::notices.reveal.inline.1"),
    Id::root("showcase_ui::pages::notices.reveal.inline.2"),
    Id::root("showcase_ui::pages::notices.reveal.inline.3"),
];
pub const NOTICE_REVEAL_OWNERS: [Id; 6] = [
    CONNECTION_REVEAL,
    QUERY_REVEAL,
    INLINE_REVEALS[0],
    INLINE_REVEALS[1],
    INLINE_REVEALS[2],
    INLINE_REVEALS[3],
];
const TEST_CONNECTION: Id = Id::root("showcase_ui::pages::notices.test_connection");
const KIND: Id = Id::root("showcase_ui::pages::notices.option.kind");
const DETAIL_LINES: Id = Id::root("showcase_ui::pages::notices.option.detail_lines");
const MESSAGE_LINES: Id = Id::root("showcase_ui::pages::notices.option.message_lines");
const DETAIL_TONE: Id = Id::root("showcase_ui::pages::notices.option.detail_tone");
const WIDTH: Id = Id::root("showcase_ui::pages::notices.option.width");
const DETAIL: Id = Id::root("showcase_ui::pages::notices.option.detail");
const HINT: Id = Id::root("showcase_ui::pages::notices.option.hint");
const MARK_CUT: Id = Id::root("showcase_ui::pages::notices.option.mark_cut");

const PAGE_TABS_ITEMS: &[&str] = &["Playground", "Options", "States"];
const KIND_ITEMS: &[&str] = &["Error", "Warning", "Info", "Success"];
const DETAIL_LINE_ITEMS: &[&str] = &["2 lines", "1 line", "4 lines"];
const MESSAGE_LINE_ITEMS: &[&str] = &["2 lines", "1 line"];
const DETAIL_TONE_ITEMS: &[&str] = &["Secondary", "Muted"];
const WIDTH_ITEMS: &[&str] = &["Wide", "Narrow"];

const ADDRESS: &str = "acme_app@analytics.acme.io:5432/analytics";
const CONNECTION_META: &str = "postgres · connection test";
const QUERY_META: &str = "error card · 0.4 ms";
const INLINE_META: &str = "inline · one per kind, the glyph carries it";
const SQL_DETAIL: &str = "LINE 1: SELECT id, email, created_at FORM users WHERE created_at > now() - interval '7 days' ORDER BY created_at DESC LIMIT 200";
const OVERFLOW_DETAIL: &str = "FATAL: password authentication failed after the server checked the saved credential, account status, database role, connection policy, and configured authentication method. Review the keychain entry and connection settings before trying again.";
const OVERFLOW_HINT: &str = "The connection worker will retry after checking each configured endpoint and the current network route. Wait for the next status update or test the connection again.";

const TAB_HINTS: &[(&str, &str)] = &[("← →", "Section"), ("1–3", "Jump"), ("Enter", "Activate")];
const SELECT_HINTS: &[(&str, &str)] = &[("↑ ↓", "Choose"), ("Enter", "Open / choose")];
const CHECKBOX_HINTS: &[(&str, &str)] = &[("Space", "Toggle")];
const BUTTON_HINTS: &[(&str, &str)] = &[("Enter", "Test connection")];
const SCROLL_HINTS: &[(&str, &str)] = &[("Wheel", "Scroll")];

const OUTCOMES: [(NoticeKind, &str, &str); 3] = [
    (
        NoticeKind::ERROR,
        "Authentication failed",
        concat!(
            "FATAL: password authentication failed for user \"acme_app\" (SQLSTATE 28P01). Check the password in the keychain or use “Prompt for password”. ",
            "The connection report keeps the complete diagnostic so a clipped row can be inspected without changing its visible preview. Verify the credential source, account status, database role, network policy, and configured authentication method before retrying. ",
            "The saved profile is shared by the analytics console and scheduled reports; confirm that a credential refresh will not interrupt an active export. If the keychain entry is valid, ask the database owner to confirm the role mapping and server-side password policy. ",
            "Recovery notes: retain the original SQLSTATE, endpoint, and request time when filing a support report. Do not include the password itself. Compare the connection settings with the deployment record, then test again after the policy or credential has been corrected. ",
            "Unicode audit identifiers: 北京, combining e\u{301}, and family 👩‍👩‍👧‍👦. Complete source tail: NOTICE_REVEAL_SOURCE_TAIL."
        ),
    ),
    (
        NoticeKind::WARNING,
        "Could not reach the host",
        "Connection timed out after 10 s (analytics.acme.io:5432). The host may be behind a VPN or the port may be blocked.",
    ),
    (
        NoticeKind::SUCCESS,
        "Connected in 182 ms",
        "PostgreSQL 16.4 · 42 schemas · TLS 1.3",
    ),
];

#[derive(Clone, Copy, Debug)]
pub struct NoticeRevealTarget {
    pub owner: Id,
    pub notice: Notice<'static>,
    pub logical_area: Rect,
    pub window: VerticalWindow,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoticeRevealPageState {
    pub content_revision: u64,
    pub play_scroll_offset: usize,
}

#[derive(Debug)]
pub struct NoticesPage {
    page_tabs: TabsState,
    outcome: usize,
    kind: usize,
    detail_lines: usize,
    message_lines: usize,
    detail_tone: usize,
    width: usize,
    show_detail: bool,
    show_hint: bool,
    mark_cut: bool,
    kind_state: SelectState,
    detail_lines_state: SelectState,
    message_lines_state: SelectState,
    detail_tone_state: SelectState,
    width_state: SelectState,
    play_scroll: ScrollState,
    options_scroll: ScrollState,
    states_scroll: ScrollState,
}

impl NoticesPage {
    #[must_use]
    pub fn new() -> Self {
        Self {
            page_tabs: TabsState::default(),
            outcome: 0,
            kind: 0,
            detail_lines: 0,
            message_lines: 0,
            detail_tone: 0,
            width: 0,
            show_detail: true,
            show_hint: true,
            mark_cut: true,
            kind_state: SelectState::default(),
            detail_lines_state: SelectState::default(),
            message_lines_state: SelectState::default(),
            detail_tone_state: SelectState::default(),
            width_state: SelectState::default(),
            play_scroll: ScrollState::default(),
            options_scroll: ScrollState::default(),
            states_scroll: ScrollState::default(),
        }
    }

    fn section(&self) -> usize {
        match self.page_tabs.active().or(self.page_tabs.cursor()) {
            Some(ItemKey::Index(index)) => index,
            _ => 0,
        }
    }

    fn detail_cap(&self) -> usize {
        [2, 1, 4][self.detail_lines.min(2)]
    }

    fn message_cap(&self) -> usize {
        [2, 1][self.message_lines.min(1)]
    }

    fn detail_tone(&self) -> FgStep {
        if self.detail_tone == 1 {
            FgStep::Muted
        } else {
            FgStep::Secondary
        }
    }

    fn content_width(&self, width: u16) -> u16 {
        if self.width == 1 {
            width.min(44)
        } else {
            width.min(96)
        }
    }

    fn styled(&self, notice: Notice<'static>) -> Notice<'static> {
        notice
            .message_lines(self.message_cap())
            .max_lines(if self.show_detail {
                self.detail_cap()
            } else {
                0
            })
            .detail_tone(self.detail_tone())
            .mark_cut(self.mark_cut)
    }

    fn with_hint(&self, notice: Notice<'static>, hint: &'static str) -> Notice<'static> {
        let notice = self.styled(notice);
        if self.show_hint {
            notice.hint(hint)
        } else {
            notice
        }
    }

    fn connection(&self) -> Notice<'static> {
        let (kind, message, detail) = OUTCOMES[self.outcome];
        self.with_hint(
            Notice::new(message).kind(kind).detail(detail),
            "Use Test connection to try again",
        )
    }

    fn query(&self) -> Notice<'static> {
        let kind = [
            NoticeKind::ERROR,
            NoticeKind::WARNING,
            NoticeKind::INFO,
            NoticeKind::SUCCESS,
        ][self.kind.min(3)];
        self.with_hint(
            Notice::error("syntax error at or near \"FORM\"")
                .kind(kind)
                .detail(SQL_DETAIL),
            "Inspect the statement detail below",
        )
    }

    fn inline(&self) -> [Notice<'static>; 4] {
        [
            self.styled(Notice::error("Port must be a number between 1 and 65535")),
            self.styled(
                Notice::warning("3 rows changed outside this session")
                    .detail("Reload to see them; saving now overwrites them"),
            ),
            self.styled(Notice::info("Read-only replica: edits are disabled")),
            self.styled(Notice::success("Saved 12 rows in 84 ms")),
        ]
    }

    #[must_use]
    pub fn reveal_state(&self) -> Option<NoticeRevealPageState> {
        if self.section() != 0 {
            return None;
        }

        let content_revision = (self.outcome as u64)
            | ((self.kind.min(3) as u64) << 2)
            | ((self.detail_lines.min(2) as u64) << 4)
            | ((self.message_lines.min(1) as u64) << 6)
            | ((self.detail_tone.min(1) as u64) << 7)
            | ((self.width.min(1) as u64) << 8)
            | (u64::from(self.show_detail) << 9)
            | (u64::from(self.show_hint) << 10)
            | (u64::from(self.mark_cut) << 11);

        Some(NoticeRevealPageState {
            content_revision,
            play_scroll_offset: self.play_scroll.offset(),
        })
    }

    fn play_content_height(&self, ui: &Ui<'_>, outer_width: u16) -> usize {
        let width = self.content_width(self.panel_inner_width(ui, outer_width));
        let connection = self.connection().height_for_width(ui, width);
        let query = self.query().height_for_width(ui, width);
        let inline = self
            .inline()
            .iter()
            .map(|notice| usize::from(notice.height_for_width(ui, width)))
            .sum::<usize>()
            .saturating_add(3);
        usize::from(connection)
            .saturating_add(7)
            .saturating_add(1)
            .saturating_add(usize::from(query))
            .saturating_add(3)
            .saturating_add(1)
            .saturating_add(inline)
            .saturating_add(3)
    }

    fn panel_inner_width(&self, ui: &Ui<'_>, width: u16) -> u16 {
        Panel::new(CONNECTION_PANEL)
            .kind(PanelKind::Card)
            .title("prod-replica")
            .inner(ui, Rect::new(0, 0, width, 8))
            .width
    }

    fn options_content_height(&self) -> usize {
        11
    }

    fn states_content_height(&self, ui: &Ui<'_>, outer_width: u16) -> usize {
        let panel_width = self.panel_inner_width(ui, outer_width);
        let width = self.content_width(panel_width);
        let detail = self.state_rows(true, ui, panel_width, width);
        let hint = self.state_rows(false, ui, panel_width, width);
        6usize
            .saturating_add(detail.into_iter().map(usize::from).sum::<usize>())
            .saturating_add(hint.into_iter().map(usize::from).sum::<usize>())
            .saturating_add(7)
    }

    fn state_samples(&self, with_detail: bool) -> [(&'static str, Notice<'static>); 4] {
        if with_detail {
            [
                (
                    "Rest",
                    self.styled(
                        Notice::error("Authentication failed")
                            .detail("Password rejected for acme_app"),
                    ),
                ),
                (
                    "Error",
                    self.styled(
                        Notice::error(
                            "Authentication failed while checking the saved database credential",
                        )
                        .detail("The saved credential was rejected"),
                    ),
                ),
                (
                    "Overflow",
                    self.styled(Notice::error("The password was rejected").detail(OVERFLOW_DETAIL)),
                ),
                (
                    "Narrow",
                    self.styled(
                        Notice::error("Detail narrow tail remains available")
                            .detail("Narrow specimen"),
                    ),
                ),
            ]
        } else {
            [
                (
                    "Rest",
                    self.with_hint(
                        Notice::warning("Disk almost full"),
                        "Free space to keep writing",
                    ),
                ),
                (
                    "Error",
                    self.with_hint(
                        Notice::error(
                            "Replica connection failed while checking the configured database service",
                        ),
                        "The configured service rejected the connection",
                    ),
                ),
                (
                    "Overflow",
                    self.with_hint(Notice::warning("Retry is available"), OVERFLOW_HINT),
                ),
                (
                    "Narrow",
                    self.with_hint(
                        Notice::success("Hint narrow tail remains available"),
                        "Connection test completed",
                    ),
                ),
            ]
        }
    }

    fn state_rows(
        &self,
        with_detail: bool,
        ui: &Ui<'_>,
        panel_inner_width: u16,
        content_width: u16,
    ) -> [u16; 4] {
        let rows = self.state_samples(with_detail);
        let mut heights = [0; 4];
        for (index, (_, notice)) in rows.iter().enumerate() {
            let available = specimen_notice_width(panel_inner_width, content_width, index == 3);
            heights[index] = notice.height_for_width(ui, available);
        }
        heights
    }

    fn update_options(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        response |= Select::new(KIND)
            .popup_rows(4)
            .update_in_form(cx, &mut self.kind_state, &mut self.kind, KIND_ITEMS, false)
            .erase();
        response |= Select::new(DETAIL_LINES)
            .popup_rows(3)
            .update_in_form(
                cx,
                &mut self.detail_lines_state,
                &mut self.detail_lines,
                DETAIL_LINE_ITEMS,
                false,
            )
            .erase();
        response |= Select::new(MESSAGE_LINES)
            .popup_rows(2)
            .update_in_form(
                cx,
                &mut self.message_lines_state,
                &mut self.message_lines,
                MESSAGE_LINE_ITEMS,
                false,
            )
            .erase();
        response |= Select::new(DETAIL_TONE)
            .popup_rows(2)
            .update_in_form(
                cx,
                &mut self.detail_tone_state,
                &mut self.detail_tone,
                DETAIL_TONE_ITEMS,
                false,
            )
            .erase();
        response |= Select::new(WIDTH)
            .popup_rows(2)
            .update_in_form(
                cx,
                &mut self.width_state,
                &mut self.width,
                WIDTH_ITEMS,
                false,
            )
            .erase();
        response |= Checkbox::new(DETAIL, "Show detail")
            .checked(self.show_detail)
            .update(cx, &mut self.show_detail)
            .erase();
        response |= Checkbox::new(HINT, "Show hint")
            .checked(self.show_hint)
            .update(cx, &mut self.show_hint)
            .erase();
        response |= Checkbox::new(MARK_CUT, "Show cut marker")
            .checked(self.mark_cut)
            .update(cx, &mut self.mark_cut)
            .erase();
        response
    }

    fn paint_options(&self, ui: &mut Ui<'_>, window: VerticalWindow) {
        let view = window.visible_body();
        let gutter = 2u16.min(view.width);
        let column_width = view.width.saturating_sub(gutter) / 2;
        let right_x = view.x.saturating_add(column_width).saturating_add(gutter);
        let title_style = ui.surface_style().fg(ui.theme_ref().color.fg[1]);
        let rows = [
            ("Query kind", KIND),
            ("Detail lines", DETAIL_LINES),
            ("Message lines", MESSAGE_LINES),
            ("Detail tone", DETAIL_TONE),
            ("Width", WIDTH),
        ];
        for (index, (label, id)) in rows.into_iter().enumerate() {
            let row = index.saturating_mul(2);
            if let Some(label_y) = window.physical_row_for_source(row) {
                paint_label(
                    ui,
                    Rect::new(view.x, label_y, column_width, 1),
                    label,
                    title_style,
                );
            }
            let Some(input_y) = window.physical_row_for_source(row.saturating_add(1)) else {
                continue;
            };
            let input_area = Rect::new(view.x, input_y, column_width, 1);
            match id {
                KIND => {
                    let _ = Select::new(KIND).draw(ui, input_area, &self.kind_state, KIND_ITEMS);
                }
                DETAIL_LINES => {
                    let _ = Select::new(DETAIL_LINES).draw(
                        ui,
                        input_area,
                        &self.detail_lines_state,
                        DETAIL_LINE_ITEMS,
                    );
                }
                MESSAGE_LINES => {
                    let _ = Select::new(MESSAGE_LINES).draw(
                        ui,
                        input_area,
                        &self.message_lines_state,
                        MESSAGE_LINE_ITEMS,
                    );
                }
                DETAIL_TONE => {
                    let _ = Select::new(DETAIL_TONE).draw(
                        ui,
                        input_area,
                        &self.detail_tone_state,
                        DETAIL_TONE_ITEMS,
                    );
                }
                WIDTH => {
                    let _ = Select::new(WIDTH).draw(ui, input_area, &self.width_state, WIDTH_ITEMS);
                }
                _ => {}
            }
        }
        for (row, id, label, checked) in [
            (1usize, DETAIL, "Show detail", self.show_detail),
            (3usize, HINT, "Show hint", self.show_hint),
            (5usize, MARK_CUT, "Show cut marker", self.mark_cut),
        ] {
            let Some(y) = window.physical_row_for_source(row) else {
                continue;
            };
            Checkbox::new(id, label).checked(checked).draw(
                ui,
                Rect::new(right_x, y, view.right().saturating_sub(right_x), 1),
            );
        }
    }

    fn draw_playground(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        revision: u64,
    ) -> [Option<NoticeRevealTarget>; 6] {
        let mut reveal_targets = [None; 6];
        let content_len = scroll_content_len(area, &self.play_scroll, |width| {
            self.play_content_height(ui, width)
        });
        let view = ScrollRegion::new(PLAY_SCROLL).draw(ui, area, &self.play_scroll, content_len);
        let Some(content_window) = scroll_content_window(view, content_len, &self.play_scroll)
        else {
            return reveal_targets;
        };
        let content_width = self.content_width(self.panel_inner_width(ui, view.width));
        let connection = self.connection();
        let query = self.query();
        let inline = self.inline();
        let connection_height = connection.height_for_width(ui, content_width);
        let query_height = query.height_for_width(ui, content_width);
        let inline_height = inline
            .iter()
            .map(|notice| usize::from(notice.height_for_width(ui, content_width)))
            .sum::<usize>()
            .saturating_add(3);
        let connection_card_height = connection_height.saturating_add(7);
        let query_start = usize::from(connection_card_height).saturating_add(1);
        let query_card_height = query_height.saturating_add(3);
        let inline_start = query_start
            .saturating_add(usize::from(query_card_height))
            .saturating_add(1);

        ui.with_area(view, |ui| {
            if let Some((connection_area, connection_window)) =
                child_window(content_window, 0, connection_card_height, view.width)
                && card_panel_window(
                    ui,
                    CONNECTION_PANEL,
                    "prod-replica",
                    Some(CONNECTION_META),
                    connection_area,
                    connection_window,
                    |ui, inner_window| {
                        let Some(inner_window) = inner_window else {
                            return;
                        };
                        let inner = inner_window.visible_body();
                        if let Some(y) = inner_window.physical_row_for_source(0) {
                            paint_label(
                                ui,
                                Rect::new(inner.x, y, inner.width, 1),
                                ADDRESS,
                                ui.surface_style().fg(ui.theme_ref().color.fg[1]),
                            );
                        }
                        if let Some((logical_area, notice_window)) = child_window(
                            inner_window,
                            2,
                            connection_height,
                            content_width.min(inner.width),
                        ) {
                            connection.draw_window(ui, notice_window);
                            reveal_targets[0] = Some(NoticeRevealTarget {
                                owner: CONNECTION_REVEAL,
                                notice: connection,
                                logical_area,
                                window: notice_window,
                                revision,
                            });
                        }
                        let button_row = usize::from(connection_height).saturating_add(3);
                        if let Some(y) = inner_window.physical_row_for_source(button_row) {
                            Button::new(TEST_CONNECTION, "Test connection")
                                .variant(Variant::SECONDARY)
                                .draw(ui, Rect::new(inner.x, y, inner.width, 1));
                        }
                    },
                )
                .is_err()
            {
                return;
            }

            if let Some((query_area, query_window)) =
                child_window(content_window, query_start, query_card_height, view.width)
                && card_panel_window(
                    ui,
                    QUERY_PANEL,
                    "Result 1",
                    Some(QUERY_META),
                    query_area,
                    query_window,
                    |ui, inner_window| {
                        let Some(inner_window) = inner_window else {
                            return;
                        };
                        if let Some((logical_area, notice_window)) = child_window(
                            inner_window,
                            0,
                            query_height,
                            content_width.min(inner_window.visible_body().width),
                        ) {
                            query.draw_window(ui, notice_window);
                            reveal_targets[1] = Some(NoticeRevealTarget {
                                owner: QUERY_REVEAL,
                                notice: query,
                                logical_area,
                                window: notice_window,
                                revision,
                            });
                        }
                    },
                )
                .is_err()
            {
                return;
            }

            let inline_card_height =
                u16::try_from(inline_height.saturating_add(3)).unwrap_or(u16::MAX);
            if let Some((inline_area, inline_window)) =
                child_window(content_window, inline_start, inline_card_height, view.width)
            {
                let _ = card_panel_window(
                    ui,
                    INLINE_PANEL,
                    "New connection",
                    Some(INLINE_META),
                    inline_area,
                    inline_window,
                    |ui, inner_window| {
                        let Some(inner_window) = inner_window else {
                            return;
                        };
                        let inner = inner_window.visible_body();
                        let mut source_row = 0usize;
                        for (index, notice) in inline.iter().enumerate() {
                            let width = content_width.min(inner.width);
                            let height = notice.height_for_width(ui, width);
                            if let Some((logical_area, notice_window)) =
                                child_window(inner_window, source_row, height, width)
                            {
                                notice.draw_window(ui, notice_window);
                                reveal_targets[index + 2] = Some(NoticeRevealTarget {
                                    owner: INLINE_REVEALS[index],
                                    notice: *notice,
                                    logical_area,
                                    window: notice_window,
                                    revision,
                                });
                            }
                            source_row = source_row
                                .saturating_add(usize::from(height))
                                .saturating_add(1);
                        }
                    },
                );
            }
        });
        reveal_targets
    }

    fn draw_states(&self, ui: &mut Ui<'_>, area: Rect) {
        let content_len = scroll_content_len(area, &self.states_scroll, |width| {
            self.states_content_height(ui, width)
        });
        let view =
            ScrollRegion::new(STATES_SCROLL).draw(ui, area, &self.states_scroll, content_len);
        let Some(content_window) = scroll_content_window(view, content_len, &self.states_scroll)
        else {
            return;
        };
        let panel_inner_width = self.panel_inner_width(ui, view.width);
        let width = self.content_width(panel_inner_width);
        let detail_rows = self.state_rows(true, ui, panel_inner_width, width);
        let hint_rows = self.state_rows(false, ui, panel_inner_width, width);
        let detail_height = detail_rows
            .into_iter()
            .fold(0u16, |sum, h| sum.saturating_add(h).saturating_add(1))
            .saturating_add(2);
        let hint_start = usize::from(detail_height).saturating_add(1);
        let hint_height = hint_rows
            .into_iter()
            .fold(0u16, |sum, h| sum.saturating_add(h).saturating_add(1))
            .saturating_add(2);
        let detail_samples = self.state_samples(true);
        let hint_samples = self.state_samples(false);

        ui.with_area(view, |ui| {
            if let Some((detail_area, detail_window)) =
                child_window(content_window, 0, detail_height, view.width)
                && card_panel_window(
                    ui,
                    STATES_DETAIL_PANEL,
                    "With detail",
                    Some("real Notice specimens · inert reference rendering"),
                    detail_area,
                    detail_window,
                    |ui, inner_window| {
                        if let Some(inner_window) = inner_window {
                            draw_reference_rows(ui, inner_window, &detail_samples, width);
                        }
                    },
                )
                .is_err()
            {
                return;
            }
            if let Some((hint_area, hint_window)) =
                child_window(content_window, hint_start, hint_height, view.width)
            {
                let _ = card_panel_window(
                    ui,
                    STATES_HINT_PANEL,
                    "With hint",
                    Some("real Notice specimens · no input ownership"),
                    hint_area,
                    hint_window,
                    |ui, inner_window| {
                        if let Some(inner_window) = inner_window {
                            draw_reference_rows(ui, inner_window, &hint_samples, width);
                        }
                    },
                );
            }
        });
    }

    fn draw_page(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        revision: u64,
    ) -> [Option<NoticeRevealTarget>; 6] {
        let mut reveal_targets = [None; 6];
        ui.with_area(area, |ui| {
            frame(
                ui,
                area,
                self.title(),
                "feedback · real controls",
                |ui, body| {
                    if body.is_empty() {
                        return;
                    }
                    let tabs_height = body.height.min(2);
                    let tabs_area = Rect {
                        height: tabs_height,
                        ..body
                    };
                    let tabs = Tabs::new(PAGE_TABS);
                    let previous = ui.layout(PAGE_TABS).map(|facts| {
                        termrock::navigation::tabs::TabWindowReceipt {
                            viewport_len: facts.viewport_len,
                            content_len: facts.content_len,
                        }
                    });
                    let layout = tabs.layout_default(
                        tabs_area,
                        ui.full(),
                        &self.page_tabs,
                        PAGE_TABS_ITEMS,
                        previous,
                    );
                    if tabs.draw_layout(ui, &layout).is_err() {
                        let _ = tabs.draw(ui, tabs_area, &self.page_tabs, PAGE_TABS_ITEMS);
                    }
                    let used_height = body.height.min(3);
                    let content = Rect {
                        y: body.y.saturating_add(used_height),
                        height: body.height.saturating_sub(used_height),
                        ..body
                    };
                    if content.is_empty() {
                        return;
                    }
                    match self.section() {
                        0 => reveal_targets = self.draw_playground(ui, content, revision),
                        1 => {
                            let scroll_len = self.options_content_height();
                            let view = ScrollRegion::new(OPTIONS_SCROLL).draw(
                                ui,
                                content,
                                &self.options_scroll,
                                scroll_len,
                            );
                            if let Some(window) =
                                scroll_content_window(view, scroll_len, &self.options_scroll)
                            {
                                ui.with_area(view, |ui| self.paint_options(ui, window));
                            }
                        }
                        _ => self.draw_states(ui, content),
                    }
                },
            );
        });
        reveal_targets
    }

    pub fn draw_with_reveal_targets(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        revision: u64,
    ) -> [Option<NoticeRevealTarget>; 6] {
        self.draw_page(ui, area, revision)
    }
}

impl Default for NoticesPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for NoticesPage {
    fn title(&self) -> &'static str {
        "Notices"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Tabs::new(PAGE_TABS)
            .update(cx, &mut self.page_tabs, PAGE_TABS_ITEMS)
            .erase();
        let mut status = None;
        match self.section() {
            0 => {
                let content_len = cx
                    .layout(PLAY_SCROLL)
                    .map_or(0, |layout| layout.content_len);
                response |=
                    ScrollRegion::new(PLAY_SCROLL).update(cx, &mut self.play_scroll, content_len);
                let button = Button::new(TEST_CONNECTION, "Test connection").update(cx);
                if button.activated() {
                    self.outcome = (self.outcome + 1) % OUTCOMES.len();
                    status = Some(PageStatus(OUTCOMES[self.outcome].1.to_owned()));
                }
                response |= button.erase();
            }
            1 => {
                let content_len = cx
                    .layout(OPTIONS_SCROLL)
                    .map_or(0, |layout| layout.content_len);
                response |= ScrollRegion::new(OPTIONS_SCROLL).update(
                    cx,
                    &mut self.options_scroll,
                    content_len,
                );
                response |= self.update_options(cx);
            }
            _ => {
                let content_len = cx
                    .layout(STATES_SCROLL)
                    .map_or(0, |layout| layout.content_len);
                response |= ScrollRegion::new(STATES_SCROLL).update(
                    cx,
                    &mut self.states_scroll,
                    content_len,
                );
            }
        }
        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let _ = self.draw_page(ui, area, 0);
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if ui.state(PAGE_TABS).contains(StateFlags::FOCUSED) {
            return TAB_HINTS;
        }
        if ui.state(TEST_CONNECTION).contains(StateFlags::FOCUSED) {
            return BUTTON_HINTS;
        }
        if [KIND, DETAIL_LINES, MESSAGE_LINES, DETAIL_TONE, WIDTH]
            .iter()
            .any(|id| ui.state(*id).contains(StateFlags::FOCUSED))
        {
            return SELECT_HINTS;
        }
        if [DETAIL, HINT, MARK_CUT]
            .iter()
            .any(|id| ui.state(*id).contains(StateFlags::FOCUSED))
        {
            return CHECKBOX_HINTS;
        }
        match self.section() {
            0..=2 => SCROLL_HINTS,
            _ => &[],
        }
    }
}

fn draw_reference_rows(
    ui: &mut Ui<'_>,
    inner_window: VerticalWindow,
    samples: &[(&str, Notice<'static>); 4],
    width: u16,
) {
    let inner = inner_window.visible_body();
    let label_width = 10.min(inner.width / 3);
    let notice_x = inner.x.saturating_add(label_width).saturating_add(1);
    let mut source_row = 0usize;
    for (label, notice) in samples {
        let sample_width = specimen_notice_width(inner.width, width, *label == "Narrow");
        let height = notice.height_for_width(ui, sample_width);
        if let Some(y) = inner_window.physical_row_for_source(source_row) {
            paint_label(
                ui,
                Rect::new(inner.x, y, label_width, 1),
                label,
                ui.surface_style().fg(ui.theme_ref().color.fg[1]),
            );
        }
        if let Some((_, notice_window)) =
            child_window_at(inner_window, source_row, height, notice_x, sample_width)
        {
            ui.reference(None, |ui| notice.draw_window(ui, notice_window));
        }
        source_row = source_row
            .saturating_add(usize::from(height))
            .saturating_add(1);
    }
}

fn specimen_notice_width(inner_width: u16, content_width: u16, narrow: bool) -> u16 {
    let label_width = 10.min(inner_width / 3);
    let notice_width = inner_width
        .saturating_sub(label_width.saturating_add(1))
        .min(content_width);
    if narrow {
        notice_width.min(18)
    } else {
        notice_width
    }
}

fn scroll_content_len(
    area: Rect,
    state: &ScrollState,
    mut measure: impl FnMut(u16) -> usize,
) -> usize {
    let content_len = measure(area.width);
    let mut projected = *state;
    projected.apply_layout(usize::from(area.height), content_len);
    if projected.overflows() {
        measure(area.width.saturating_sub(1))
    } else {
        content_len
    }
}

fn scroll_content_window(
    view: Rect,
    logical_rows: usize,
    state: &ScrollState,
) -> Option<VerticalWindow> {
    if view.width == 0 || view.height == 0 || logical_rows == 0 {
        return None;
    }

    let first_source_row = ScrollRegion::view(state, view, logical_rows).offset();
    let first_source_row = first_source_row.min(logical_rows);
    let visible_rows = usize::from(view.height).min(logical_rows.saturating_sub(first_source_row));
    if visible_rows == 0 {
        return None;
    }
    let visible_body = Rect {
        height: u16::try_from(visible_rows).ok()?,
        ..view
    };
    VerticalWindow::new(visible_body, logical_rows, first_source_row).ok()
}

fn child_window(
    parent: VerticalWindow,
    source_start: usize,
    logical_rows: u16,
    width: u16,
) -> Option<(Rect, VerticalWindow)> {
    child_window_at(
        parent,
        source_start,
        logical_rows,
        parent.visible_body().x,
        width,
    )
}

fn child_window_at(
    parent: VerticalWindow,
    source_start: usize,
    logical_rows: u16,
    x: u16,
    width: u16,
) -> Option<(Rect, VerticalWindow)> {
    if logical_rows == 0 || width == 0 {
        return None;
    }
    let parent_area = parent.visible_body();
    if x < parent_area.x || x >= parent_area.right() {
        return None;
    }
    let width = width.min(parent_area.right().saturating_sub(x));
    if width == 0 {
        return None;
    }
    let source_end = source_start.checked_add(usize::from(logical_rows))?;
    let visible = parent.visible_source_range();
    let first = source_start.max(visible.start);
    let end = source_end.min(visible.end);
    if first >= end {
        return None;
    }

    let physical_y = parent.physical_row_for_source(first)?;
    let visible_height = u16::try_from(end.saturating_sub(first)).ok()?;
    let visible_body = Rect::new(x, physical_y, width, visible_height);
    let window = VerticalWindow::new(
        visible_body,
        usize::from(logical_rows),
        first - source_start,
    )
    .ok()?;
    let logical_area = Rect::new(visible_body.x, physical_y, visible_body.width, logical_rows);
    Some((logical_area, window))
}

fn paint_label(ui: &mut Ui<'_>, area: Rect, text: &str, style: termrock::author::PaintStyle) {
    if area.is_empty() || text.is_empty() {
        return;
    }
    let _ = ui.paint_str(area, text, style);
}
