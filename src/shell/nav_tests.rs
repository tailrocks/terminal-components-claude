use super::*;
use crate::ui::leave::Resolved;
use crate::widgets::dialog::DialogResult;

fn strings() -> &'static Strings {
    &Strings::DEFAULT
}

#[test]
fn push_back_and_pop_to_walk_one_stack() {
    let mut r = Router::new("root");
    let k0 = r.key();
    r.push("a");
    r.push("b");
    assert_eq!(r.depth(), 3);
    assert_eq!(*r.current(), "b");
    assert_ne!(
        r.key(),
        k0,
        "another page shows: focus upkeep sees a new key"
    );
    assert_eq!(r.back(), Some("b"));
    assert_eq!(r.pop_to(0), vec!["a"]);
    assert_eq!(r.back(), None, "the root stays");
    assert_eq!(r.key(), k0, "the root keeps its key");
    assert!(r.pop_to(0).is_empty());
}

#[test]
fn replace_and_reset_drop_what_they_cover() {
    let mut r = Router::new(1);
    r.push(2);
    let k = r.key();
    assert_eq!(r.replace(3), 2);
    assert_ne!(r.key(), k, "a replaced page is another page");
    assert_eq!(r.stack().copied().collect::<Vec<_>>(), [1, 3]);
    assert_eq!(r.reset(9), vec![1, 3]);
    assert_eq!(r.depth(), 1);
}

#[test]
fn tabs_keep_their_own_stacks() {
    let mut r = Router::with_tabs(vec!["here"]);
    r.push("disk");
    let t = r.open_tab("activity");
    assert_eq!((t, r.active(), *r.current()), (1, 1, "activity"));
    assert!(r.select(0));
    assert_eq!(*r.current(), "disk", "the first tab kept its stack");
    assert!(!r.select(0), "already shown");
    assert!(!r.select(7));
    assert_eq!(r.close_tab(1), Some(vec!["activity"]));
    assert_eq!(r.close_tab(0), None, "the last tab stays");
    r.open_tab("x");
    r.open_tab("y");
    assert_eq!(r.active(), 2);
    r.close_tab(2);
    assert_eq!(r.active(), 1, "closing the shown tab shows the one before");
    assert_eq!(r.pages().count(), 3);
}

#[test]
fn apply_builds_new_routes_and_reports_what_it_dropped() {
    let mut r: Router<String> = Router::new("root".into());
    let build = |n: u8| format!("page {n}");
    assert_eq!(r.apply(Nav::Push(1), build), Some(vec![]));
    assert_eq!(r.apply(Nav::Push(2), build), Some(vec![]));
    assert_eq!(
        r.apply(Nav::PopTo(0), build),
        Some(vec!["page 1".into(), "page 2".into()])
    );
    assert_eq!(r.apply(Nav::Back, build), None, "nothing to go back to");
    assert_eq!(r.apply(Nav::Tab(3), build), None);
    assert_eq!(r.apply(Nav::Reset(5), build), Some(vec!["root".into()]));
    assert_eq!(r.current(), "page 5");
}

#[test]
fn only_a_move_that_drops_a_page_asks_for_its_stakes() {
    let mut r = Router::new(0);
    let mut guard = LeaveGuard::new(WidgetId::of("leave"));
    let dirty = || Stakes::Unsaved { changes: 2 };

    let nav = r.request(&mut guard, Nav::Push(1), || unreachable!(), strings());
    assert_eq!(nav.unwrap(), Nav::Push(1));
    r.push(1);
    assert!(
        r.request(&mut guard, Nav::Tab(0), || unreachable!(), strings())
            .is_ok()
    );

    let d = r
        .request(&mut guard, Nav::Back, dirty, strings())
        .unwrap_err();
    assert!(guard.is_dialog(d.id));
    assert_eq!(
        guard.resolve(DialogResult::Action(1)),
        Resolved::Discard(Leave::To(Nav::Back))
    );

    assert!(
        r.request(&mut guard, Nav::Replace(3), || Stakes::Clean, strings())
            .is_ok()
    );
}

#[test]
fn dropped_by_names_the_pages_a_move_leaves() {
    let mut r = Router::new('a');
    r.push('b');
    r.push('c');
    let names = |nav: Nav<char>| r.dropped_by(&nav).copied().collect::<String>();
    assert_eq!(names(Nav::Back), "c");
    assert_eq!(names(Nav::PopTo(0)), "cb");
    assert_eq!(names(Nav::Reset('z')), "cba");
    assert_eq!(names(Nav::Replace('z')), "c");
    assert_eq!(names(Nav::Push('z')), "");
    let root = Router::new('a');
    assert_eq!(root.dropped_by(&Nav::<char>::Back).count(), 0);
}

#[test]
fn a_breadcrumb_click_pops_to_its_level() {
    let mut r = Router::new("Disk");
    r.push("Usage");
    r.push("Top files");
    let mut crumb = r.breadcrumb(WidgetId::of("crumb"), |p| (*p).to_owned());
    assert_eq!(crumb.segments, ["Disk", "Usage", "Top files"]);
    let first = crumb.segment_id(0);
    assert_eq!(r.crumb_click::<()>(&crumb, 0, first), Some(Nav::PopTo(0)));

    crumb.segments.insert(0, "Screens".into());
    assert_eq!(r.crumb_click::<()>(&crumb, 1, crumb.segment_id(0)), None);
    assert_eq!(
        r.crumb_click::<()>(&crumb, 1, crumb.segment_id(2)),
        Some(Nav::PopTo(1))
    );
    assert_eq!(
        r.crumb_click::<()>(&crumb, 1, crumb.segment_id(3)),
        None,
        "the current page is no link"
    );
}
