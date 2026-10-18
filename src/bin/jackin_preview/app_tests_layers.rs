




use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Position;

use junie_tui::core::event::MouseKind;
use junie_tui::core::id::WidgetId;
use junie_tui::ui::modal::CLOSE;

use crate::app::Route;
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};



fn backdrop(h: &H, hit: WidgetId) -> bool {
    use junie_tui::core::id::Part;
    h.app
        .shell
        .hits
        .part_of(hit)
        .is_some_and(|x| x.part.part == Part::BACKDROP)
}



fn outside(h: &H) -> Position {
    let area = h.term.backend().buffer().area;
    let cells = || {
        (1..area.height.saturating_sub(1))
            .flat_map(move |y| (0..area.width).map(move |x| Position::new(x, y)))
    };
    cells()
        .find(|p| h.app.shell.hits.hit(*p).is_none())
        .or_else(|| cells().find(|p| h.app.shell.hits.hit(*p).is_some_and(|id| backdrop(h, id))))
        .unwrap_or_else(|| panic!("no free cell outside the layer:\n{}", h.text()))
}




struct Kind {
    name: &'static str,
    open: fn() -> (H, Option<Option<WidgetId>>),
}

fn dialog() -> (H, Option<Option<WidgetId>>) {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Down);
    let owner = h.app.shell.focus.current();
    h.key(KeyCode::Char('d'));
    assert!(
        h.app
            .top_layer::<junie_tui::widgets::dialog::Dialog>()
            .is_some(),
        "{}",
        h.text()
    );
    (h, Some(owner))
}

fn picker() -> (H, Option<Option<WidgetId>>) {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    let owner = h.app.shell.focus.current();
    h.ctrl('\\');
    assert!(
        h.app
            .top_layer::<junie_tui::widgets::picker::Picker>()
            .is_some(),
        "{}",
        h.text()
    );
    (h, Some(owner))
}

fn browser() -> (H, Option<Option<WidgetId>>) {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::Char('n'));
    assert_eq!(h.app.route, Route::Prelude, "{}", h.text());
    assert!(
        h.app
            .top_layer::<crate::screens::modals::FileBrowser>()
            .is_some(),
        "{}",
        h.text()
    );

    (h, None)
}

fn choice() -> (H, Option<Option<WidgetId>>) {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    let owner = h.app.shell.focus.current();
    h.ctrl('q');
    assert!(
        h.app
            .top_layer::<crate::screens::modals::ChoiceDialog>()
            .is_some(),
        "{}",
        h.text()
    );
    (h, Some(owner))
}

fn form() -> (H, Option<Option<WidgetId>>) {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route, Route::Accounts);
    let owner = h.app.shell.focus.current();
    h.key(KeyCode::Char('a'));
    assert!(
        h.app
            .top_layer::<crate::screens::modals::FormDialog>()
            .is_some(),
        "{}",
        h.text()
    );
    (h, Some(owner))
}



fn op() -> (H, Option<Option<WidgetId>>) {
    let (mut h, _) = form();
    h.key(KeyCode::Enter);
    h.type_str("Team");
    for _ in 0..4 {
        h.key(KeyCode::Tab);
    }
    let owner = h.app.shell.focus.current();
    h.key(KeyCode::Enter);
    h.ticks(4);
    assert!(
        h.app
            .top_layer::<crate::screens::modals::OpFlow>()
            .is_some(),
        "{}",
        h.text()
    );
    assert_eq!(h.app.layer_count(), 2, "over the form");
    (h, Some(owner))
}

fn info() -> (H, Option<Option<WidgetId>>) {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    let owner = h.app.shell.focus.current();
    let (x, y) = h.find("Help").unwrap();
    h.click(x, y);
    let (x, y) = h.find("About jackin-preview").unwrap();
    h.click(x, y);
    assert!(
        h.app
            .top_layer::<crate::screens::modals::InfoDialog>()
            .is_some(),
        "{}",
        h.text()
    );
    (h, Some(owner))
}

fn help() -> (H, Option<Option<WidgetId>>) {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    let owner = h.app.shell.focus.current();
    h.key(KeyCode::Char('?'));
    assert!(
        h.app
            .top_layer::<crate::screens::modals::HelpOverlay>()
            .is_some(),
        "{}",
        h.text()
    );
    (h, Some(owner))
}

fn usage() -> (H, Option<Option<WidgetId>>) {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    let owner = h.app.shell.focus.current();
    let chip = h
        .app
        .shell
        .hits
        .area_of(crate::screens::capsule::USAGE_CHIP)
        .expect("the usage chip");
    let x = (chip.x..chip.right())
        .find(|&x| {
            h.app.shell.hits.hit(Position::new(x, chip.y))
                == Some(crate::screens::capsule::USAGE_CHIP)
        })
        .expect("a cell of the usage chip takes the click");
    h.click(x, chip.y);
    assert!(
        h.app
            .top_layer::<crate::screens::capsule::UsageDialog>()
            .is_some(),
        "{}",
        h.text()
    );
    (h, Some(owner))
}

fn inspect() -> (H, Option<Option<WidgetId>>) {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    let owner = h.app.shell.focus.current();
    h.key(KeyCode::F(10));
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert!(
        h.app
            .top_layer::<crate::screens::inspect::InspectChanges>()
            .is_some(),
        "{}",
        h.text()
    );
    (h, Some(owner))
}

const KINDS: [Kind; 10] = [
    Kind {
        name: "dialog",
        open: dialog,
    },
    Kind {
        name: "picker",
        open: picker,
    },
    Kind {
        name: "browser",
        open: browser,
    },
    Kind {
        name: "choice",
        open: choice,
    },
    Kind {
        name: "form",
        open: form,
    },
    Kind {
        name: "1password",
        open: op,
    },
    Kind {
        name: "info",
        open: info,
    },
    Kind {
        name: "help",
        open: help,
    },
    Kind {
        name: "usage",
        open: usage,
    },
    Kind {
        name: "inspect",
        open: inspect,
    },
];


fn assert_closed(name: &str, how: &str, h: &H, below: usize, owner: Option<Option<WidgetId>>) {
    assert_eq!(
        h.app.layer_count(),
        below,
        "{name}: {how} left it open\n{}",
        h.text()
    );
    if let Some(owner) = owner {
        assert_eq!(
            h.app.shell.focus.current(),
            owner,
            "{name}: K9: {how} did not give the owner its focus back"
        );
    }
}

#[test]
fn every_layer_kind_dismisses_under_its_policy_and_gives_focus_back() {
    for kind in &KINDS {
        let name = kind.name;


        let (mut h, owner) = (kind.open)();
        let below = h.app.layer_count() - 1;
        assert_ne!(
            Some(h.app.shell.focus.current()),
            owner,
            "{name}: focus moved into the layer"
        );

        for _ in 0..2 {
            if h.app.layer_count() > below {
                h.key(KeyCode::Esc);
            }
        }
        assert_closed(name, "Esc", &h, below, owner);



        let (mut h, owner) = (kind.open)();
        let policy = h.app.top_dismiss().expect("a layer is open");
        let p = outside(&h);
        h.mouse(MouseKind::Down, p.x, p.y);
        h.mouse(MouseKind::Up, p.x, p.y);
        if policy.outside_click {
            assert_closed(name, "an outside click", &h, below, owner);
        } else {
            assert_eq!(
                h.app.layer_count(),
                below + 1,
                "{name}: a refused outside click closed it"
            );
            assert!(
                h.app.shell.pointer.has_flash(),
                "{name}: a refused outside click did not flash the frame"
            );
        }



        let (mut h, owner) = (kind.open)();
        let close = h.app.shell.hits.area_of(CLOSE);
        if let Some(x) = close {
            assert!(
                policy.shows_close(),
                "{name}: × drawn but Esc may not close it"
            );
            h.click(x.x + 1, x.y);
            assert_closed(name, "×", &h, below, owner);
        }
    }
}




#[test]
fn the_failure_details_scrollbar_drags_through_the_layer() {
    let mut h = H::new(Scenario::LaunchFailure, Motion::Reduced, 0, 100, 24);
    for _ in 0..120 {
        if h.app
            .top_layer::<crate::screens::modals::InfoDialog>()
            .is_some()
        {
            break;
        }
        h.ticks(3);
    }
    let bar = junie_tui::widgets::scrollbar::id_for(WidgetId::of("cockpit.failure"));
    let track = h
        .app
        .shell
        .hits
        .area_of(bar)
        .unwrap_or_else(|| panic!("the detail scrolls at 100x24:\n{}", h.text()));
    let offset = |h: &H| {
        h.app
            .top_layer::<crate::screens::modals::InfoDialog>()
            .expect("the failure stays open")
            .detail_scroll
            .offset
    };
    assert_eq!(offset(&h), 0);
    h.mouse(MouseKind::Down, track.x, track.y);
    h.mouse(MouseKind::Drag, track.x, track.bottom() - 1);
    assert!(offset(&h) > 0, "the drag did not move the detail");
    h.mouse(MouseKind::Up, track.x, track.bottom() - 1);
}



#[test]
fn a_paste_into_the_palette_filters_it() {
    let (mut h, _) = picker();
    h.paste("split");
    let p = h
        .app
        .top_layer::<junie_tui::widgets::picker::Picker>()
        .expect("the palette stays open");
    assert_eq!(p.query.text(), "split");
    assert!(
        p.items
            .iter()
            .all(|i| i.label.to_lowercase().contains("split")),
        "the rows follow the query: {:?}",
        p.items.iter().map(|i| &i.label).collect::<Vec<_>>()
    );
}




#[test]
fn a_focus_change_reaches_the_form_under_the_layer_it_opened() {
    let (mut h, _) = form();
    h.key(KeyCode::Enter);
    h.type_str("Team");
    let name_editing = |h: &H| {
        h.app
            .top_layer::<crate::screens::modals::FormDialog>()
            .map(crate::screens::modals::FormDialog::is_editing)
    };
    assert_eq!(name_editing(&h), Some(true));
    let chooser = crate::screens::accounts::FORM.sub("op");
    let r = h
        .app
        .shell
        .hits
        .area_of(chooser)
        .unwrap_or_else(|| panic!("the 1Password chooser:\n{}", h.text()));
    h.click(r.x + 1, r.y);
    assert!(
        h.app
            .top_layer::<crate::screens::modals::OpFlow>()
            .is_some(),
        "{}",
        h.text()
    );
    h.key(KeyCode::Esc);
    assert_eq!(h.app.layer_count(), 1);
    assert_eq!(h.app.shell.focus.current(), Some(chooser), "K9");
    assert_eq!(name_editing(&h), Some(false), "the name edit ended on blur");
    let f = h
        .app
        .top_layer::<crate::screens::modals::FormDialog>()
        .unwrap();
    assert_eq!(f.text("name"), "Team", "and kept what was typed");
}






#[test]
fn only_the_new_workspace_browser_shows_the_flow_progress() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Prelude);
    assert!(
        h.app
            .top_layer::<crate::screens::modals::FileBrowser>()
            .is_some_and(|b| b.stepper.is_some()),
        "the flow's browser carries its progress"
    );
    assert!(h.text().contains("Source · Destination"), "{}", h.text());

    h.key(KeyCode::Backspace);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    for _ in 0..3 {
        h.key(KeyCode::Enter);
    }
    assert_eq!(h.app.route, Route::Editor, "{}", h.text());
    let (x, y) = h.find("Mounts").unwrap();
    h.click(x, y);
    let (x, y) = h.find("+ Add mount").unwrap();
    h.click(x + 2, y);
    h.key(KeyCode::Enter);
    let (x, y) = h
        .find("Browse")
        .unwrap_or_else(|| panic!("the mount form's folder chooser:\n{}", h.text()));
    h.click(x + 1, y);
    assert!(
        h.app
            .top_layer::<crate::screens::modals::FileBrowser>()
            .is_some(),
        "{}",
        h.text()
    );
    assert!(!h.text().contains("✓ Source"), "{}", h.text());
}




#[test]
fn the_form_footer_names_the_focused_button() {
    let (mut h, _) = form();
    let save = |h: &H| {
        h.app
            .top_layer::<crate::screens::modals::FormDialog>()
            .map(|f| (f.save.id, f.save.label.clone()))
            .unwrap()
    };
    let (id, label) = save(&h);
    for _ in 0..12 {
        if h.app.shell.focus.current() == Some(id) {
            break;
        }
        h.key(KeyCode::Tab);
    }
    assert_eq!(h.app.shell.focus.current(), Some(id), "{}", h.text());
    let footer = h.text().lines().next_back().unwrap_or("").to_owned();
    assert!(footer.contains(&format!("Enter {label}")), "{footer}");
    assert!(!footer.contains("Edit / Save"), "{footer}");
}
