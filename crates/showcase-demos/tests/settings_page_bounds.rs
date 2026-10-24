use showcase_demos::pages::{Page, settings::SettingsPage};
use termrock::{
    App, Color, Cx, Id, ItemKey, KeyCode, LayerId, Modifier, Part, PartRef, Rect, Response, Role,
    StateFlags, StylePatch, Theme, Ui,
};
use termrock_test_support::Harness;

const TABS: Id = Id::root("showcase_ui::pages::settings::settings.tabs");
const MEMBERS: Id = Id::root("showcase_ui::pages::settings::settings.members");
const INVITE: Id = Id::root("showcase_ui::pages::settings::settings.invite");
const REMOVE: Id = Id::root("showcase_ui::pages::settings::settings.remove");
const MEMBER_GRID: Id = Id::root("showcase_ui::pages::settings::settings.member_grid");
const MEMBER_GRID_EDITOR: Id = MEMBER_GRID.part(Part::TEXT);
const MEMBER_BACK: Id = Id::root("showcase_ui::pages::settings::settings.member_back");
const REMOVE_DIALOG: Id = Id::root("showcase_ui::pages::settings::settings.remove.dialog");

struct PageHost {
    page: SettingsPage,
    area: Rect,
}

impl App for PageHost {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.page.update(cx).response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let screen = ui.full();
        let sentinel = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Custom(Color::Rgb(17, 39, 71)))
                .set_bg(Role::Custom(Color::Rgb(83, 61, 29)))
                .add(Modifier::ITALIC),
        );
        let row = "S".repeat(usize::from(screen.width));
        for y in screen.y..screen.bottom() {
            let _ = ui.paint_str(Rect::new(screen.x, y, screen.width, 1), &row, sentinel);
        }
        self.page.draw(ui, self.area);
    }
}

fn host(width: u16, height: u16, area: Rect) -> Harness<PageHost> {
    Harness::new(
        PageHost {
            page: SettingsPage::new(),
            area,
        },
        Theme::junie(),
        width,
        height,
    )
}

fn inside(rect: Rect, point_x: u16, point_y: u16) -> bool {
    point_x >= rect.x && point_x < rect.right() && point_y >= rect.y && point_y < rect.bottom()
}

fn inside_any(rectangles: &[Rect], x: u16, y: u16) -> bool {
    rectangles.iter().any(|rect| inside(*rect, x, y))
}

fn contains_rect(outer: Rect, inner: Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.right() <= outer.right()
        && inner.bottom() <= outer.bottom()
}

fn assert_outside_cells_are_sentinel(h: &Harness<PageHost>, page_area: Rect) -> Vec<String> {
    changed_cells_outside(h, &[page_area])
}

fn changed_cells_outside(h: &Harness<PageHost>, allowed: &[Rect]) -> Vec<String> {
    let screen = *h.buffer().area();
    let mut changed = Vec::new();
    for y in screen.y..screen.bottom() {
        for x in screen.x..screen.right() {
            if inside_any(allowed, x, y) {
                continue;
            }
            let cell = h.cell(x, y);
            if (cell.symbol() != "S"
                || cell.fg != Color::Rgb(17, 39, 71)
                || cell.bg != Color::Rgb(83, 61, 29)
                || cell.modifier != Modifier::ITALIC
                || cell.underline_color != Color::Reset
                || cell.diff_option != Default::default())
                && changed.len() < 12
            {
                changed.push(format!("({x},{y})={cell:?}"));
            }
        }
    }
    changed
}

fn focus_rectangles_outside(h: &Harness<PageHost>, page_area: Rect) -> Vec<String> {
    h.ring()
        .entries()
        .iter()
        .filter(|entry| !contains_rect(page_area, entry.area))
        .map(|entry| format!("{:?} {:?}", entry.id, entry.area))
        .collect()
}

fn escaped_presentation(h: &Harness<PageHost>, allowed: &[Rect]) -> Vec<String> {
    let mut escaped = h
        .ring()
        .entries()
        .iter()
        .filter(|entry| !allowed.iter().any(|area| contains_rect(*area, entry.area)))
        .map(|entry| format!("focus {:?} {:?}", entry.id, entry.area))
        .collect::<Vec<_>>();
    for id in [TABS, MEMBERS, INVITE, REMOVE, MEMBER_GRID, MEMBER_BACK] {
        if let Some(area) = h.area_of(id)
            && !allowed.iter().any(|outer| contains_rect(*outer, area))
        {
            escaped.push(format!("hit {id:?} {area:?}"));
        }
    }
    if let Some(cursor) = h.cursor()
        && !allowed.iter().any(|area| inside(*area, cursor.x, cursor.y))
    {
        escaped.push(format!("cursor {cursor:?}"));
    }
    escaped
}

fn assert_contained(h: &Harness<PageHost>, allowed: &[Rect], context: &str) {
    let changed = changed_cells_outside(h, allowed);
    let escaped = escaped_presentation(h, allowed);
    assert!(
        changed.is_empty() && escaped.is_empty(),
        "{context}: outside writes {changed:?}; escaped presentation {escaped:?}"
    );
}

fn activate_tab(h: &mut Harness<PageHost>, index: usize) {
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::index(index)))
            .is_some(),
        "tab {index} has a real hit area"
    );
    let _ = h.click_part(TABS, PartRef::item(Part::TAB, ItemKey::index(index)));
}

fn assert_modal_underlay_is_only_dimmed(
    h: &Harness<PageHost>,
    plain: &Harness<PageHost>,
    page_area: Rect,
    dialog_area: Rect,
) {
    let theme = Theme::junie();
    for y in 0..h.buffer().area().height {
        for x in 0..h.buffer().area().width {
            if inside(page_area, x, y) || inside(dialog_area, x, y) {
                continue;
            }
            let before = plain.cell(x, y);
            let actual = h.cell(x, y);
            let backdrop_applies = y < h.buffer().area().bottom().saturating_sub(1);
            let expected_style = if backdrop_applies {
                theme.backdrop_style(before.style())
            } else {
                before.style()
            };
            assert_eq!(
                actual.symbol(),
                before.symbol(),
                "modal backdrop preserves underlay symbol at ({x},{y})"
            );
            assert_eq!(
                actual.fg,
                expected_style.fg.unwrap_or(before.fg),
                "modal backdrop foreground at ({x},{y})"
            );
            assert_eq!(
                actual.bg,
                expected_style.bg.unwrap_or(before.bg),
                "modal backdrop background at ({x},{y})"
            );
            assert_eq!(
                actual.underline_color,
                expected_style
                    .underline_color
                    .unwrap_or(before.underline_color),
                "modal backdrop underline at ({x},{y})"
            );
            assert_eq!(
                actual.modifier, expected_style.add_modifier,
                "modal backdrop modifiers at ({x},{y})"
            );
            assert_eq!(
                actual.diff_option, before.diff_option,
                "modal backdrop preserves cell diff metadata at ({x},{y})"
            );
        }
    }
}

#[test]
fn direct_unscoped_height_one_draw_keeps_header_and_contains_empty_body() {
    let area = Rect::new(7, 3, 20, 1);
    let h = host(48, 12, area);

    let changed_outside = assert_outside_cells_are_sentinel(&h, area);
    let escaped_focus = focus_rectangles_outside(&h, area);
    let header_ellipsis_is_preserved = h.cell(25, 3).symbol() == "…";
    let trailing_header_cell_is_blank = h.cell(26, 3).symbol() == " ";

    assert!(
        changed_outside.is_empty()
            && escaped_focus.is_empty()
            && header_ellipsis_is_preserved
            && trailing_header_cell_is_blank,
        "outside writes: {changed_outside:?}; escaped focus: {escaped_focus:?}; header ellipsis: {header_ellipsis_is_preserved}; trailing blank: {trailing_header_cell_is_blank}"
    );
}

#[test]
fn direct_page_short_and_offset_rectangles_contain_paint_hits_focus_and_cursor() {
    let cases = [
        Rect::new(0, 0, 72, 0),
        Rect::new(7, 3, 0, 5),
        Rect::new(7, 3, 20, 1),
        Rect::new(0, 0, 20, 2),
        Rect::new(7, 3, 20, 3),
        Rect::new(7, 3, 1, 3),
        Rect::new(7, 3, 2, 4),
        Rect::new(7, 3, 19, 4),
        Rect::new(7, 3, 20, 4),
        Rect::new(7, 3, 21, 4),
        Rect::new(7, 3, 72, 5),
        Rect::new(7, 3, 120, 20),
    ];
    for area in cases {
        let h = host(160, 42, area);
        assert_contained(&h, &[area], &format!("page area {area:?}"));
        if area.height <= 2 || area.width == 0 {
            assert!(
                h.ring().entries().is_empty(),
                "an empty body publishes no page controls at {area:?}: {:?}",
                h.ring().entries()
            );
        }
    }

    let area = Rect::new(7, 3, 20, 1);
    let h = host(48, 12, area);
    assert_eq!(
        h.cell(25, 3).symbol(),
        "…",
        "header clipping keeps its marker"
    );
    assert_eq!(
        h.cell(26, 3).symbol(),
        " ",
        "header keeps its trailing blank"
    );
}

#[test]
fn member_list_and_action_bands_exist_only_inside_their_clipped_rows() {
    let normal = Rect::new(7, 3, 80, 24);
    let mut h = host(110, 42, normal);
    activate_tab(&mut h, 1);
    assert!(h.text().contains("Mira Okafor"), "the Members tab is live");

    for width in [0, 1, 2] {
        let area = Rect::new(7, 3, width, 6);
        h.app_mut().area = area;
        h.draw();
        assert_contained(&h, &[area], &format!("narrow Members area {area:?}"));
        if let Some(list) = h.area_of(MEMBERS) {
            assert!(contains_rect(area, list));
            assert!(list.height <= 1);
        }
        assert!(h.area_of(INVITE).is_none());
    }

    for (area, list_height, action_visible) in [
        (Rect::new(7, 3, 20, 5), None, false),
        (Rect::new(7, 3, 20, 6), Some(1), false),
        (Rect::new(7, 3, 19, 18), Some(12), false),
        (Rect::new(7, 3, 21, 19), Some(12), true),
    ] {
        h.app_mut().area = area;
        h.draw();
        assert_contained(&h, &[area], &format!("Members area {area:?}"));

        match (list_height, h.area_of(MEMBERS)) {
            (None, None) => (),
            (Some(max_height), Some(list)) => {
                assert_eq!(list.y, area.y + 5, "list starts at body row three");
                assert!(list.height <= max_height);
                assert!(contains_rect(area, list));
            }
            (expected, actual) => panic!("list band {expected:?}, registered area {actual:?}"),
        }
        assert_eq!(
            h.area_of(INVITE).is_some(),
            action_visible,
            "the Invite action appears only when body row sixteen is visible"
        );
        assert_eq!(h.area_of(REMOVE).is_some(), action_visible);
    }
}

#[test]
fn normal_showcase_sizes_keep_real_settings_tabs_list_and_actions_reachable() {
    for (width, height) in [(72, 20), (80, 24), (120, 40)] {
        let area = Rect::new(0, 0, width, height);
        let mut h = host(width, height, area);
        assert!(h.text().contains("Project settings"));
        assert_contained(&h, &[area], &format!("normal {width}x{height}"));

        activate_tab(&mut h, 1);
        assert!(h.text().contains("Mira Okafor"));
        assert!(h.area_of(MEMBERS).is_some());
        assert!(h.area_of(INVITE).is_some());
        assert!(h.area_of(REMOVE).is_some());
        assert_contained(&h, &[area], &format!("Members {width}x{height}"));
    }
}

#[test]
fn tab_state_survives_reposition_and_hidden_controls_lose_their_old_hits() {
    let normal = Rect::new(7, 3, 80, 24);
    let mut h = host(110, 42, normal);

    activate_tab(&mut h, 1);
    assert!(format!("{:?}", h.app().page).contains("message: \"members\""));
    let hidden_tab_hit = h
        .area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::index(2)))
        .expect("the Security tab is reachable at normal size");
    let hidden_action_hit = h
        .area_of(INVITE)
        .expect("the Invite action is reachable at normal size");

    h.app_mut().area = Rect::new(7, 3, 80, 2);
    let _ = h.resize(110, 36);
    let short = h.app().area;
    assert_contained(&h, &[short], "first shortened frame");
    assert!(
        h.ring().entries().is_empty(),
        "the empty body has no page owners"
    );
    assert!(h.area_of(TABS).is_none());
    assert!(h.area_of(MEMBERS).is_none());
    assert!(h.area_of(INVITE).is_none());

    let before_hidden_clicks = format!("{:?}", h.app().page);
    let _ = h.click(
        hidden_tab_hit.x.saturating_add(hidden_tab_hit.width / 2),
        hidden_tab_hit.y.saturating_add(hidden_tab_hit.height / 2),
    );
    let _ = h.click(
        hidden_action_hit
            .x
            .saturating_add(hidden_action_hit.width / 2),
        hidden_action_hit
            .y
            .saturating_add(hidden_action_hit.height / 2),
    );
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        format!("{:?}", h.app().page),
        before_hidden_clicks,
        "old coordinates no longer activate clipped tab or action controls"
    );

    h.app_mut().area = Rect::new(0, 0, 90, 30);
    let _ = h.resize(90, 30);
    assert_contained(&h, &[h.app().area], "repositioned normal frame");
    assert!(format!("{:?}", h.app().page).contains("message: \"members\""));
    activate_tab(&mut h, 2);
    assert!(format!("{:?}", h.app().page).contains("message: \"security\""));
    activate_tab(&mut h, 0);
    assert!(format!("{:?}", h.app().page).contains("message: \"general settings\""));
}

#[test]
fn remove_dialog_keeps_its_own_layer_when_the_settings_body_is_empty() {
    let normal = Rect::new(7, 3, 80, 24);
    let mut h = host(110, 42, normal);
    activate_tab(&mut h, 1);
    assert!(h.area_of(REMOVE).is_some());
    let _ = h.click_id(REMOVE);
    assert!(
        h.is_open(REMOVE_DIALOG),
        "Remove opens the real modal layer"
    );
    assert!(h.text().contains("Remove member?"));

    let short = Rect::new(7, 3, 20, 1);
    let plain = host(110, 42, short);
    h.app_mut().area = short;
    h.draw();
    let dialog_area = h
        .layer_area(REMOVE_DIALOG)
        .expect("the modal retains an independently placed layer area");
    assert_modal_underlay_is_only_dimmed(&h, &plain, short, dialog_area);
    let escaped = escaped_presentation(&h, &[short, dialog_area]);
    assert!(
        escaped.is_empty(),
        "dialog presentation stays in its own area: {escaped:?}"
    );
    assert!(h.is_open(REMOVE_DIALOG));
    assert!(
        h.text().contains("Remove member?"),
        "the dialog remains painted"
    );
    assert!(
        h.ring()
            .entries()
            .iter()
            .all(|entry| { entry.layer != LayerId::PAGE || contains_rect(short, entry.area) }),
        "the modal does not restore hidden page controls: {:?}",
        h.ring().entries()
    );
    let focused = h.focus().expect("the modal retains a focused action");
    assert!(
        h.ring()
            .entries()
            .iter()
            .any(|entry| entry.id == focused && entry.layer != LayerId::PAGE),
        "modal focus stays in its independent layer"
    );

    let _ = h.key(KeyCode::Esc);
    assert!(
        !h.is_open(REMOVE_DIALOG),
        "the independent modal remains operable"
    );
    assert!(format!("{:?}", h.app().page).contains("message: \"remove cancelled\""));
    assert!(
        format!("{:?}", h.app().page).contains("name: \"Mira Okafor\""),
        "cancel preserves the selected member behind the short page"
    );
}

#[test]
fn member_selection_invite_and_remove_confirm_keep_the_existing_actions() {
    let area = Rect::new(0, 0, 90, 32);
    let mut h = host(90, 32, area);
    activate_tab(&mut h, 1);

    let second_member = PartRef::item(Part::ROW, ItemKey::num(2));
    assert!(h.area_of_part(MEMBERS, second_member).is_some());
    let _ = h.click_part(MEMBERS, second_member);
    assert!(format!("{:?}", h.app().page).contains("selected: 1"));
    assert!(format!("{:?}", h.app().page).contains("message: \"member selected\""));
    assert!(
        !h.text().contains("Member details"),
        "Chose keeps the List view"
    );

    assert!(h.tab_to(INVITE), "Invite remains a real keyboard target");
    let _ = h.key(KeyCode::Enter);
    assert!(format!("{:?}", h.app().page).contains("message: \"invite flow ready\""));

    let _ = h.click_id(REMOVE);
    assert!(
        h.is_open(REMOVE_DIALOG),
        "Remove opens through its registered hit"
    );
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    assert!(
        !h.is_open(REMOVE_DIALOG),
        "Enter confirms the real remove dialog"
    );
    assert!(format!("{:?}", h.app().page).contains("message: \"member removed\""));
    let text = h.text();
    assert!(text.contains("Mira Okafor"), "unselected member remains");
    assert!(
        !text.contains("Jonas Weber"),
        "the selected member was removed"
    );
}

#[test]
fn member_grid_edits_name_and_validated_role_then_returns_to_same_member() {
    let area = Rect::new(0, 0, 120, 40);
    let mut h = host(120, 40, area);
    activate_tab(&mut h, 1);
    assert!(h.tab_to(MEMBERS), "Members remains keyboard reachable");
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Enter);

    assert_eq!(h.focus(), Some(MEMBER_GRID));
    assert_contained(&h, &[area], "opened member Grid");
    assert!(h.text().contains("Member details"));
    assert!(h.text().contains("Jonas Weber"));
    assert!(h.text().contains("jonas@acme.dev"));
    assert!(h.text().contains("Admin"));
    assert!(h.text().contains("2 h ago"));
    assert_eq!(
        h.find("Admin").expect("Jonas role is visible").1,
        h.find("Jonas Weber").expect("selected row is visible").1,
        "the restored role belongs to the selected stable member"
    );

    let _ = h.key(KeyCode::Enter);
    assert!(h.state_of(MEMBER_GRID_EDITOR).contains(StateFlags::EDITING));
    let _ = h.ctrl('l');
    let _ = h.type_str("   ");
    let _ = h.key(KeyCode::Enter);
    let name_editor = h
        .area_of(MEMBER_GRID_EDITOR)
        .expect("invalid Name remains in the active editor");
    assert_eq!(
        h.cell(name_editor.right().saturating_sub(1), name_editor.y)
            .symbol(),
        "!",
        "the native editor marker identifies an invalid Name draft"
    );
    assert!(h.state_of(MEMBER_GRID_EDITOR).contains(StateFlags::EDITING));
    assert_eq!(h.focus(), Some(MEMBER_GRID_EDITOR));
    let _ = h.key(KeyCode::Esc);
    assert!(h.text().contains("Jonas Weber"));

    let _ = h.key(KeyCode::Enter);
    let _ = h.ctrl('l');
    let _ = h.type_str("Jonas Weber Jr");
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Jonas Weber Jr"));
    assert!(format!("{:?}", h.app().page).contains("message: \"member updated\""));

    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    assert!(
        !h.state_of(MEMBER_GRID_EDITOR).contains(StateFlags::EDITING),
        "Email is a real read-only Grid cell"
    );
    let _ = h.paste("wrong@example.dev");
    assert!(
        h.text().contains("jonas@acme.dev"),
        "read-only Email still shows its original value"
    );
    assert!(!h.text().contains("wrong@example.dev"));

    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    let _ = h.ctrl('l');
    let _ = h.type_str("Superuser");
    let _ = h.key(KeyCode::Enter);
    let role_editor = h
        .area_of(MEMBER_GRID_EDITOR)
        .expect("invalid Role remains in the active editor");
    assert_eq!(
        h.cell(role_editor.right().saturating_sub(1), role_editor.y)
            .symbol(),
        "!",
        "the native editor marker identifies an invalid Role draft"
    );
    assert!(h.state_of(MEMBER_GRID_EDITOR).contains(StateFlags::EDITING));
    assert_eq!(h.focus(), Some(MEMBER_GRID_EDITOR));
    let _ = h.key(KeyCode::Esc);
    assert!(
        h.text().contains("Admin"),
        "Esc cancels the invalid role draft"
    );

    let _ = h.key(KeyCode::Enter);
    let _ = h.ctrl('l');
    let _ = h.type_str("Owner");
    let _ = h.key(KeyCode::Enter);
    let owner_row = h.find("Owner").expect("edited role is visible").1;

    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    let _ = h.paste("10m ago");
    assert!(h.text().contains("2 h ago"));
    assert!(!h.text().contains("10m ago"));

    let _ = h.key(KeyCode::Home);
    assert_eq!(
        h.find("Jonas Weber Jr")
            .expect("Home reveals the edited Name cell")
            .1,
        owner_row,
        "the Name and Role edits remain on the same stable member"
    );
    assert!(
        h.text().contains("jonas@acme.dev"),
        "Email remains unchanged"
    );

    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(MEMBER_BACK), "Grid Tab reaches native Back");
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.focus(), Some(MEMBERS), "Back restores the List owner");
    assert!(
        h.area_of_part(MEMBERS, PartRef::item(Part::ROW, ItemKey::num(2)))
            .is_some()
    );
    assert!(format!("{:?}", h.app().page).contains("selected: 1"));
    assert!(h.text().contains("Jonas Weber Jr"));
    assert!(
        h.text().matches("Jonas Weber Jr").count() >= 2,
        "the list row and compact summary both use the committed name"
    );

    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.focus(), Some(MEMBER_GRID));
    let jonas = h
        .find("Jonas Weber Jr")
        .expect("the same member row reopens first");
    let mira = h
        .find("Mira Okafor")
        .expect("the neighboring member row remains");
    assert!(
        jonas.1 < mira.1,
        "editor restores the stable selected member ID"
    );
    assert_eq!(
        h.find("Owner").expect("the committed role persists").1,
        jonas.1
    );
    assert!(
        h.text().contains("jonas@acme.dev"),
        "read-only Email remains unchanged"
    );
    assert!(h.text().contains("2 h ago"), "read-only facts persist");
    let _ = h.key(KeyCode::Tab);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.focus(), Some(MEMBERS));

    let _ = h.click_id(REMOVE);
    assert!(h.is_open(REMOVE_DIALOG));
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    assert!(!h.is_open(REMOVE_DIALOG));
    let text = h.text();
    assert!(
        !text.contains("Jonas Weber Jr"),
        "Remove targets the same stable ID"
    );
    assert!(
        text.contains("Ana Costa"),
        "the next member remains untouched"
    );
    let diagnostics = h.diagnostics();
    assert!(
        diagnostics.is_empty(),
        "removing a member leaves no runtime diagnostics"
    );
}

#[test]
fn member_grid_back_refuses_an_active_draft_until_explicit_cancel() {
    let area = Rect::new(0, 0, 120, 40);
    let mut h = host(120, 40, area);
    activate_tab(&mut h, 1);
    assert!(h.tab_to(MEMBERS));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.focus(), Some(MEMBER_GRID));
    let _ = h.key(KeyCode::Enter);
    let _ = h.ctrl('l');
    let _ = h.type_str("Uncommitted Mira");
    assert!(h.text().contains("Uncommitted Mira"));

    let back = h.area_of(MEMBER_BACK).expect("Back has a real button area");
    let _ = h.click(back.x, back.y);
    assert!(h.text().contains("Member details"));
    assert!(h.text().contains("Uncommitted Mira"));
    assert!(
        h.state_of(MEMBER_GRID_EDITOR).contains(StateFlags::EDITING),
        "clicking Back cannot silently commit or discard a live draft"
    );
    assert_eq!(h.focus(), Some(MEMBER_GRID_EDITOR));

    let _ = h.key(KeyCode::Esc);
    assert!(!h.text().contains("Uncommitted Mira"));
    assert!(h.text().contains("Mira Okafor"));
    assert!(!h.state_of(MEMBER_GRID_EDITOR).contains(StateFlags::EDITING));
    let _ = h.click_id(MEMBER_BACK);
    assert!(!h.text().contains("Member details"));
    assert_eq!(h.focus(), Some(MEMBERS));
    let diagnostics = h.diagnostics();
    assert!(
        diagnostics.is_empty(),
        "cancelling the draft leaves no runtime diagnostics"
    );
}

#[test]
fn member_grid_back_remains_reachable_after_short_resize() {
    let area = Rect::new(0, 0, 120, 40);
    let mut h = host(120, 40, area);
    activate_tab(&mut h, 1);
    assert!(h.tab_to(MEMBERS));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.focus(), Some(MEMBER_GRID));

    h.app_mut().area = Rect::new(0, 0, 72, 20);
    let _ = h.resize(72, 20);
    assert_contained(&h, &[h.app().area], "member Grid at 72x20");
    let back = h
        .area_of(MEMBER_BACK)
        .expect("short layout keeps a real Back owner");
    assert!(contains_rect(h.app().area, back));
    assert!(h.area_of(MEMBER_GRID).is_some());

    h.app_mut().area = Rect::new(0, 0, 80, 18);
    let _ = h.resize(80, 18);
    assert_contained(&h, &[h.app().area], "member Grid at 80x18");
    let back = h
        .area_of(MEMBER_BACK)
        .expect("a shorter frame moves Back into the visible last row");
    assert!(contains_rect(h.app().area, back));
    assert!(h.tab_to(MEMBER_BACK));
    let _ = h.key(KeyCode::Enter);
    assert!(!h.text().contains("Member details"));
    assert_eq!(h.focus(), Some(MEMBERS));
    assert!(h.text().contains("Mira Okafor"));
    let diagnostics = h.diagnostics();
    assert!(
        diagnostics.is_empty(),
        "resizing and leaving the Grid leaves no runtime diagnostics"
    );
}
