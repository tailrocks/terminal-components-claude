use std::sync::{Arc, Mutex};

use showcase_demos::pages::{Page, master_detail::MasterDetailPage};
use termrock::{
    App, Axis, Cx, Id, ItemKey, KeyCode, KeyModifiers, Part, PartRef, Rect, Response, Theme, Ui,
};
use termrock_test_support::Harness;

const PAGE_TABS: Id = Id::root("showcase_ui::pages::master_detail.page_tabs");
const WIDGET_BASE: Id = Id::root("showcase_ui::pages::master_detail.widget");
const ROOT: Id = WIDGET_BASE.index(0);
const MASTER: Id = ROOT.index(1);
const PROPS: Id = ROOT.index(3);
const DETAIL: Id = PROPS;
const OPEN: Id = ROOT.index(4);
const TOGGLE: Id = ROOT.index(5);
const MASTER_PANEL: Id = ROOT.index(6);
const DETAIL_PANEL: Id = ROOT.index(7);
const WIDTH: Id = Id::root("showcase_ui::pages::master_detail.option.width");
const COLLAPSE: Id = Id::root("showcase_ui::pages::master_detail.option.collapse");
const SIZE: Id = Id::root("showcase_ui::pages::master_detail.option.size");
const SUMMARY: Id = Id::root("showcase_ui::pages::master_detail.option.summary");
const ESC: Id = Id::root("showcase_ui::pages::master_detail.option.esc");
const KEYS: Id = Id::root("showcase_ui::pages::master_detail.option.keys");
const STEP: Id = Id::root("showcase_ui::pages::master_detail.option.step");
const GAP: Id = Id::root("showcase_ui::pages::master_detail.option.gap");
const STATES: Id = Id::root("showcase_ui::pages::master_detail.states_scroll");

struct PageHost {
    page: MasterDetailPage,
    status: Option<String>,
    copied: Arc<Mutex<Option<String>>>,
}

impl App for PageHost {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let copied = Arc::clone(&self.copied);
        cx.set_clipboard_copy_hook(move |text| {
            if let Ok(mut copied) = copied.lock() {
                *copied = Some(text.to_owned());
            }
        });
        let update = self.page.update(cx);
        self.status = update.status.map(|status| status.0);
        update.response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.page.draw(ui, ui.full());
    }
}

struct OffsetHost {
    page: MasterDetailPage,
    area: Rect,
}

impl App for OffsetHost {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.page.update(cx).response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        for y in full.y..full.bottom() {
            ui.paint_str(
                Rect::new(full.x, y, full.width, 1),
                &"S".repeat(usize::from(full.width)),
                ui.surface_style(),
            );
        }
        self.page.draw(ui, self.area);
    }
}

fn host(width: u16, height: u16) -> Harness<PageHost> {
    Harness::new(
        PageHost {
            page: MasterDetailPage::new(),
            status: None,
            copied: Arc::new(Mutex::new(None)),
        },
        Theme::junie(),
        width,
        height,
    )
}

fn section(h: &mut Harness<PageHost>, index: usize) {
    let _ = h.click_part(PAGE_TABS, PartRef::item(Part::TAB, ItemKey::index(index)));
}

fn debug(h: &Harness<PageHost>) -> String {
    format!("{:?}", h.app().page)
}

fn select_index(h: &mut Harness<PageHost>, id: Id, current: usize, target: usize) {
    assert!(
        h.tab_to(id),
        "option {id:?} is available in the Options section"
    );
    let _ = h.key(KeyCode::Enter);
    let direction = if target >= current {
        KeyCode::Down
    } else {
        KeyCode::Up
    };
    for _ in 0..current.abs_diff(target) {
        let _ = h.key(direction);
    }
    let _ = h.key(KeyCode::Enter);
}

fn select_table(h: &mut Harness<PageHost>, name: &str) {
    let part = PartRef::item(Part::ROW, ItemKey::text(name));
    assert!(
        h.area_of_part(MASTER, part).is_some(),
        "table row {name} is drawn; master={:?}\n{}\npage={:?}",
        h.area_of(MASTER),
        h.text(),
        h.app().page
    );
    let _ = h.click_part(MASTER, part);
}

fn seam_x(h: &Harness<PageHost>) -> u16 {
    h.area_of_part(ROOT, PartRef::of(Part::SEAM))
        .expect("resizable split draws its interactive seam")
        .x
}

fn panel_gap(h: &Harness<PageHost>) -> u16 {
    let master = h
        .area_of(MASTER_PANEL)
        .expect("live split draws its master panel");
    let detail = h
        .area_of(DETAIL_PANEL)
        .expect("live split draws its detail panel");
    detail.x.saturating_sub(master.right())
}

fn panel_extent(h: &Harness<PageHost>) -> u16 {
    let master = h
        .area_of(MASTER_PANEL)
        .expect("live layout draws its master panel");
    h.area_of(DETAIL_PANEL).map_or(master.width, |detail| {
        detail.right().saturating_sub(master.x)
    })
}

fn reference_seam(h: &Harness<PageHost>, label: &str) -> (u16, String) {
    let (_, label_y) = h
        .find(label)
        .unwrap_or_else(|| panic!("reference label {label:?} is visible\n{}", h.text()));
    let row_y = label_y + 1;
    let separators = (0..120)
        .filter(|x| matches!(h.cell(*x, row_y).symbol(), "│" | "┃"))
        .collect::<Vec<_>>();
    assert_eq!(
        separators.len(),
        3,
        "sample row has a master card edge, the real seam, and a detail card edge\n{}",
        h.row(row_y)
    );
    let x = separators[1];
    (x, h.cell(x, row_y).symbol().to_owned())
}

fn reference_seam_at_small_width(
    h: &Harness<PageHost>,
    label: &str,
) -> Option<(u16, u16, String, String, Vec<u16>)> {
    let (_, label_y) = h.find(label)?;
    let row_y = label_y + 1;
    let view = h.area_of(STATES)?;
    let content_right = view.right().saturating_sub(1);
    let separators = (view.x..content_right)
        .filter(|x| matches!(h.cell(*x, row_y).symbol(), "│" | "┃"))
        .collect::<Vec<_>>();
    if separators.len() != 3 {
        return None;
    }
    let x = separators[1];
    let cell = h.cell(x, row_y);
    Some((
        x,
        row_y,
        cell.symbol().to_owned(),
        format!("{:?}/{:?}/{:?}", cell.fg, cell.bg, cell.modifier),
        separators,
    ))
}

#[test]
fn canonical_defaults_and_all_ten_rows_drive_real_facts_copy_and_open_status() {
    let mut h = host(120, 40);
    let state = debug(&h);
    for default in [
        "width: 0",
        "collapse: 0",
        "size: 0",
        "summary: 0",
        "esc_returns: 1",
        "keys: 0",
        "step: 0",
        "gap: 0",
    ] {
        assert!(
            state.contains(default),
            "canonical page default {default:?}: {state}"
        );
    }
    assert!(h.text().contains("Tables"));
    assert!(h.text().contains("orders"));
    assert!(h.text().contains("Schema"));
    assert!(h.text().contains("shop.public.orders"));
    for name in [
        "orders",
        "order_items",
        "customers",
        "products",
        "invoices",
        "payments",
        "shipments",
        "audit_log",
        "sessions",
        "feature_flags",
    ] {
        select_table(&mut h, name);
        let _ = h.click_id(OPEN);
        assert_eq!(
            h.app().status.as_deref(),
            Some(format!("Opened {name}").as_str()),
            "Open reports the selected row and does not replace it with an external action"
        );
    }
    assert!(
        [
            "Schema", "Rows", "Size", "Columns", "Indexes", "Owner", "Analyzed", "Path"
        ]
        .iter()
        .all(|fact| h.text().contains(fact)),
        "all eight facts are real PropsList rows\n{}",
        h.text()
    );

    select_table(&mut h, "customers");
    let path = PartRef::item(Part::ROW, ItemKey::text("path"));
    assert!(
        h.area_of_part(PROPS, path).is_some(),
        "the selected Path row is addressable"
    );
    let _ = h.click_part(PROPS, path);
    let _ = h.key(KeyCode::Char('y'));
    assert_eq!(
        h.app()
            .copied
            .lock()
            .expect("clipboard hook lock")
            .as_deref(),
        Some("shop.public.customers")
    );
    assert_eq!(
        h.app().status.as_deref(),
        Some("Copied shop.public.customers")
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn options_change_real_width_collapse_summary_gap_and_resize_policies() {
    let mut h = host(120, 40);
    section(&mut h, 1);
    select_index(&mut h, WIDTH, 0, 2);
    select_index(&mut h, COLLAPSE, 0, 1);
    select_index(&mut h, SUMMARY, 0, 2);
    select_index(&mut h, GAP, 0, 2);
    let state = debug(&h);
    for selected in ["width: 2", "collapse: 1", "summary: 2", "gap: 2"] {
        assert!(state.contains(selected), "selection {selected:?}: {state}");
    }
    assert!(h.area_of(SUMMARY).is_some());
    assert!(h.area_of(GAP).is_some());

    section(&mut h, 0);
    let master = h
        .area_of(MASTER)
        .expect("collapsed master list remains live");
    assert!(master.width <= 72, "master width {master:?}\n{}", h.text());
    assert!(h.area_of_part(ROOT, PartRef::of(Part::SEAM)).is_none());
    assert!(
        h.text().contains("audit_log"),
        "the summary master remains visible\n{}",
        h.text()
    );

    section(&mut h, 1);
    select_index(&mut h, WIDTH, 2, 0);
    select_index(&mut h, COLLAPSE, 1, 2);
    section(&mut h, 0);
    let master = h
        .area_of(MASTER)
        .expect("collapsed master list remains live");
    assert!(master.width <= 120, "master width {master:?}\n{}", h.text());
    let seam = h.area_of_part(ROOT, PartRef::of(Part::SEAM));
    assert!(
        seam.is_none(),
        "120 is below the configured 140-column split threshold"
    );
    assert!(h.text().contains("Tables"));

    section(&mut h, 1);
    select_index(&mut h, COLLAPSE, 2, 0);
    select_index(&mut h, SIZE, 0, 1);
    section(&mut h, 0);
    assert!(
        h.area_of_part(ROOT, PartRef::of(Part::SEAM)).is_none(),
        "fixed sizing keeps both panes but does not publish an interactive seam"
    );
    assert!(
        h.area_of(MASTER).is_some(),
        "fixed detail keeps the table list"
    );
    assert!(
        h.area_of(PROPS).is_some(),
        "fixed detail keeps its fact rows"
    );
    assert!(h.text().contains("Schema"));

    section(&mut h, 1);
    select_index(&mut h, SIZE, 1, 2);
    section(&mut h, 0);
    assert!(
        h.area_of_part(ROOT, PartRef::of(Part::SEAM)).is_none(),
        "fixed master share remains noninteractive"
    );
    assert!(
        h.area_of(MASTER).is_some(),
        "fixed master keeps the table list"
    );
    assert!(
        h.area_of(PROPS).is_some(),
        "fixed master keeps its fact rows"
    );

    section(&mut h, 1);
    select_index(&mut h, SIZE, 2, 0);
    select_index(&mut h, KEYS, 0, 1);
    select_index(&mut h, STEP, 0, 2);
    section(&mut h, 0);
    assert!(h.tab_to(MASTER));
    let before = seam_x(&h);
    let _ = h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    assert_eq!(
        seam_x(&h),
        before + 5,
        "custom resize binding applies the five-cell option"
    );

    section(&mut h, 1);
    select_index(&mut h, KEYS, 1, 2);
    section(&mut h, 0);
    assert!(h.tab_to(MASTER));
    let before = seam_x(&h);
    let _ = h.key_mod(KeyCode::Char('l'), KeyModifiers::CONTROL);
    let _ = h.key_mod(KeyCode::Right, KeyModifiers::ALT.union(KeyModifiers::SHIFT));
    assert_eq!(seam_x(&h), before, "none publishes no resize command");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn remaining_option_values_change_live_width_summary_step_and_gap() {
    let mut h = host(140, 40);
    section(&mut h, 1);
    select_index(&mut h, WIDTH, 0, 1);
    select_index(&mut h, STEP, 0, 1);
    select_index(&mut h, GAP, 0, 1);
    section(&mut h, 0);

    assert_eq!(
        panel_extent(&h),
        100,
        "the 100-column option caps the live split"
    );
    assert_eq!(panel_gap(&h), 1, "the one-column gap is live geometry");
    assert!(
        !h.text().contains("→ orders · 12,408 rows"),
        "zero summary rows remove the narrow summary: {}",
        h.text()
    );
    assert!(h.tab_to(MASTER));
    let before = seam_x(&h);
    let _ = h.key_mod(KeyCode::Right, KeyModifiers::ALT.union(KeyModifiers::SHIFT));
    assert_eq!(seam_x(&h), before + 1, "the one-cell step is live");

    section(&mut h, 1);
    select_index(&mut h, WIDTH, 1, 2);
    select_index(&mut h, SUMMARY, 0, 1);
    select_index(&mut h, GAP, 1, 2);
    section(&mut h, 0);
    assert_eq!(panel_extent(&h), 72, "the 72-column option is live");
    assert!(h.area_of_part(ROOT, PartRef::of(Part::SEAM)).is_none());
    assert!(
        !h.text().contains("→ orders · 12,408 rows"),
        "the zero-row summary remains hidden after the width change"
    );
    section(&mut h, 1);
    select_index(&mut h, SUMMARY, 1, 2);
    section(&mut h, 0);
    assert!(h.text().contains("→ orders · 12,408 rows · 3.1 MB"));
    assert!(h.text().contains("14 columns · 4 indexes · owner app"));

    section(&mut h, 1);
    select_index(&mut h, WIDTH, 2, 0);
    select_index(&mut h, COLLAPSE, 0, 1);
    section(&mut h, 0);
    assert!(
        h.area_of_part(ROOT, PartRef::of(Part::SEAM)).is_some(),
        "fill width above the 108-column threshold returns to split mode"
    );
    assert_eq!(panel_gap(&h), 4, "the four-column gap is live geometry");
    section(&mut h, 1);
    select_index(&mut h, COLLAPSE, 1, 2);
    section(&mut h, 0);
    assert!(
        h.area_of_part(ROOT, PartRef::of(Part::SEAM)).is_some(),
        "the 140-column threshold remains split at its inclusive boundary"
    );
    let _ = h.resize(139, 40);
    assert!(
        h.area_of_part(ROOT, PartRef::of(Part::SEAM)).is_none(),
        "one column below the 140-column threshold uses the collapsed layout"
    );
    assert_eq!(panel_extent(&h), 139);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn default_resize_and_real_seam_drag_use_cell_deltas() {
    let mut h = host(120, 40);
    assert!(h.tab_to(MASTER));
    let initial = seam_x(&h);
    let _ = h.key_mod(KeyCode::Right, KeyModifiers::ALT.union(KeyModifiers::SHIFT));
    assert_eq!(
        seam_x(&h),
        initial + 2,
        "the page default resize step is two cells"
    );

    let seam = h
        .area_of_part(ROOT, PartRef::of(Part::SEAM))
        .expect("default mode has a real seam hit target");
    let before = seam.x;
    let _ = h.drag((seam.x, seam.y + 5), (seam.x + 8, seam.y + 5));
    assert_eq!(
        seam_x(&h),
        before + 8,
        "the canonical drag changes the live master pane by exactly eight terminal cells"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn narrow_tab_escape_toggle_and_esc_option_preserve_drawer_focus() {
    let mut h = host(80, 24);
    section(&mut h, 1);
    select_index(&mut h, WIDTH, 0, 2);
    section(&mut h, 0);
    let master = h.area_of(MASTER).expect("narrow master list remains live");
    assert!(master.width <= 72, "master width {master:?}\n{}", h.text());
    assert!(h.area_of_part(ROOT, PartRef::of(Part::SEAM)).is_none());
    assert!(h.tab_to(MASTER));
    assert_eq!(h.focus(), Some(MASTER));
    let _ = h.key(KeyCode::Tab);
    assert_eq!(
        h.focus(),
        Some(DETAIL),
        "Tab from the selected master opens its detail drawer; page={:?}\n{}",
        h.app().page,
        h.text()
    );
    assert!(debug(&h).contains("drawer_open: true"));
    assert!(
        h.text().contains("Esc returns to tables"),
        "the enabled option retains its truthful drawer promise\n{}",
        h.text()
    );
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.focus(), Some(MASTER));
    assert!(debug(&h).contains("drawer_open: false"));

    let _ = h.click_id(TOGGLE);
    assert!(debug(&h).contains("drawer_open: true"));
    let _ = h.click_id(TOGGLE);
    assert!(debug(&h).contains("drawer_open: false"));

    section(&mut h, 1);
    select_index(&mut h, ESC, 1, 0);
    section(&mut h, 0);
    assert!(h.tab_to(MASTER));
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(DETAIL));
    assert!(
        !h.text().contains("Esc returns to tables"),
        "the drawer hides the Esc promise when the option is off\n{}",
        h.text()
    );
    let _ = h.key(KeyCode::Esc);
    assert_eq!(
        h.focus(),
        Some(DETAIL),
        "Esc is no longer advertised as returning to the table list"
    );
    assert!(debug(&h).contains("drawer_open: true"));

    section(&mut h, 1);
    select_index(&mut h, ESC, 0, 1);
    section(&mut h, 0);
    assert!(
        h.text().contains("Esc returns to tables"),
        "enabling the option restores its drawer promise\n{}",
        h.text()
    );
    assert!(h.tab_to(DETAIL));
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.focus(), Some(MASTER));
    assert!(debug(&h).contains("drawer_open: false"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn four_reference_states_render_real_shared_controls_and_stay_clipped() {
    let mut h = host(120, 40);
    section(&mut h, 2);
    for label in ["Rest", "Hover", "Pressed", "Narrow"] {
        assert!(
            h.text().contains(label),
            "reference state {label}\n{}",
            h.text()
        );
    }
    assert!(h.text().contains("orders"));
    assert!(h.text().contains("Schema"));
    let rest = reference_seam(&h, "Rest");
    let hover = reference_seam(&h, "Hover");
    let pressed = reference_seam(&h, "Pressed");
    assert_eq!(rest.0, hover.0);
    assert_eq!(rest.0, pressed.0);
    assert_eq!(rest.1, "│", "rest seam uses the quiet separator glyph");
    assert_eq!(hover.1, "┃", "hover target changes the live seam glyph");
    assert_eq!(pressed.1, "┃", "pressed target changes the live seam glyph");
    assert!(
        h.find("→ products · 942 rows · 312 kB").is_some(),
        "the Narrow specimen paints the real one-row summary from its live data"
    );

    let offset = Harness::new(
        OffsetHost {
            page: MasterDetailPage::new(),
            area: Rect::new(5, 3, 72, 12),
        },
        Theme::junie(),
        90,
        24,
    );
    assert_eq!(offset.cell(0, 0).symbol(), "S");
    assert_eq!(offset.cell(4, 3).symbol(), "S");
    assert!(
        offset
            .area_of(MASTER)
            .is_some_and(|area| area.x >= 5 && area.right() <= 77),
        "offset child remains inside the requested parent clip: {:?}",
        offset.area_of(MASTER),
    );
    assert!(
        offset.diagnostics().is_empty(),
        "{:?}",
        offset.diagnostics()
    );
}

#[test]
fn states_gallery_scrolls_to_the_later_reference_rows() {
    let mut h = host(80, 24);
    section(&mut h, 2);
    let view = h.area_of(STATES).expect("real state gallery scroll region");
    let _ = h.wheel(Axis::V, 8, view.x + 1, view.y + 1);
    assert!(
        h.text().contains("Pressed"),
        "later state rows can be reached\n{}",
        h.text()
    );
    assert!(
        h.text().contains("Narrow"),
        "last state sample can be reached\n{}",
        h.text()
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn small_scrolling_reference_gallery_keeps_three_real_split_states_inert() {
    let mut styles = Vec::new();
    for (index, label) in ["Rest", "Hover", "Pressed"].into_iter().enumerate() {
        let mut h = host(80, 24);
        section(&mut h, 2);
        let view = h.area_of(STATES).expect("real state gallery viewport");
        assert_eq!(
            view.width, 80,
            "the 80-column host includes its scroll track"
        );
        if index == 2 {
            let _ = h.wheel(Axis::V, 8, view.x + 1, view.y + 1);
        }
        let (seam_x, seam_y, glyph, style, separators) = reference_seam_at_small_width(&h, label)
            .unwrap_or_else(|| panic!("{label} is a split at 80x24\n{}", h.text()));
        assert_eq!(glyph, if index == 0 { "│" } else { "┃" });
        assert!(
            seam_x >= view.x + 24 && seam_x < view.right().saturating_sub(34),
            "{label} seam respects both configured pane minima: view={view:?}, separators={separators:?}"
        );
        assert_eq!(
            separators.len(),
            3,
            "{label} paints both cards and the split seam"
        );
        assert_eq!(
            h.cell(view.right() - 1, seam_y).symbol(),
            "┃",
            "the separately reserved rightmost scroll-track column remains present"
        );
        styles.push(style);

        let focus_ids: Vec<_> = h.ring().reachable().map(|entry| entry.id).collect();
        let sample_root = WIDGET_BASE.index(index + 1);
        for id in [
            sample_root,
            sample_root.index(1),
            sample_root.index(2),
            sample_root.index(3),
            sample_root.index(4),
            sample_root.index(5),
            sample_root.index(6),
            sample_root.index(7),
        ] {
            assert!(
                h.area_of(id).is_none(),
                "reference id {id:?} has no hit target"
            );
            assert!(
                !focus_ids.contains(&id),
                "reference id {id:?} has no focus stop"
            );
        }
        assert!(
            h.area_of_part(sample_root, PartRef::of(Part::SEAM))
                .is_none(),
            "reference seam has no hit target"
        );
        let focus_before = h.focus();
        let _ = h.drag((seam_x, seam_y), (seam_x + 4, seam_y));
        assert_eq!(h.runtime().capture_owner(), None, "reference seam is inert");
        assert_eq!(h.focus(), focus_before, "reference seam cannot take focus");
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
    assert_ne!(
        styles[0], styles[1],
        "hover changes the resolved seam style"
    );
    assert_ne!(
        styles[1], styles[2],
        "press changes the resolved seam style"
    );

    let mut narrow = host(80, 24);
    section(&mut narrow, 2);
    let view = narrow.area_of(STATES).expect("real state gallery viewport");
    let _ = narrow.wheel(Axis::V, 8, view.x + 1, view.y + 1);
    assert!(
        narrow.find("Narrow").is_some(),
        "Narrow label is exposed by scrolling"
    );
    assert!(
        narrow.text().contains("→ products · 942 rows · 312 kB"),
        "Narrow preserves the canonical one-row summary after scrolling\n{}",
        narrow.text()
    );
    assert!(reference_seam_at_small_width(&narrow, "Narrow").is_none());
    let narrow_root = WIDGET_BASE.index(4);
    assert!(narrow.area_of(narrow_root).is_none());
    assert!(
        narrow
            .area_of_part(narrow_root, PartRef::of(Part::SEAM))
            .is_none(),
        "Narrow reference has no split hit target"
    );
    assert_eq!(narrow.runtime().capture_owner(), None);
    assert!(
        narrow.diagnostics().is_empty(),
        "{:?}",
        narrow.diagnostics()
    );
}
