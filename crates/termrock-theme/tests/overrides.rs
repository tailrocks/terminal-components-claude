#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )
)]

use ratatui_core::style::Color;
use termrock_controls::Button;
use termrock_core::Id;
use termrock_core::id::PartRef;
use termrock_runtime::{Cell, Position, Rect, Ui};
use termrock_test_support::Scene;
use termrock_theme::{
    ColorLevel, Family, Overlay, OverlayRule, Part, Role, StateFlags, StylePatch, Theme, Variant,
};

const A: Id = Id::root("ov.a");
const B: Id = Id::root("ov.b");
const C: Id = Id::root("ov.c");

const LAYOUT: [(Id, Variant, Rect); 3] = [
    (A, Variant::PRIMARY, Rect::new(0, 0, 20, 1)),
    (B, Variant::PRIMARY, Rect::new(0, 2, 20, 1)),
    (C, Variant::SECONDARY, Rect::new(0, 4, 20, 1)),
];

fn scene(name: &'static str, theme: Theme) -> Scene {
    Scene::new(name, theme, ColorLevel::TrueColor, 30, 6)
}

type PaintOne<'a> = &'a dyn Fn(&mut Ui<'_>);
type Wrap<'a> = &'a dyn Fn(&mut Ui<'_>, usize, PaintOne<'_>);

fn paint(ui: &mut Ui<'_>, wrap: Wrap<'_>) {
    for (i, (id, variant, rect)) in LAYOUT.iter().enumerate() {
        wrap(ui, i, &|ui: &mut Ui<'_>| {
            Button::new(*id, "Label").variant(*variant).draw(ui, *rect);
        });
    }
}

fn plain(ui: &mut Ui<'_>, _i: usize, f: PaintOne<'_>) {
    f(ui);
}

fn label_fg(s: &Scene, i: usize) -> Option<Color> {
    let r = LAYOUT[i].2;
    s.buffer().cell((r.x + 1, r.y)).map(|c| c.fg)
}

fn label_bg(s: &Scene, i: usize) -> Option<Color> {
    let r = LAYOUT[i].2;
    s.buffer().cell((r.x + 1, r.y)).map(|c| c.bg)
}

fn baseline(name: &'static str) -> Scene {
    let mut s = scene(name, Theme::junie());
    s.draw(|ui, _| paint(ui, &plain));
    s
}

#[test]
fn global_family_override_changes_every_button() {
    let before = baseline("overrides::before");
    let theme = Theme::junie().override_family(Family::BUTTON, |e| {
        e.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut after = scene("overrides::family", theme);
    after.draw(|ui, _| paint(ui, &plain));

    let info = Theme::junie().color.info;
    for i in 0..LAYOUT.len() {
        assert_eq!(
            label_fg(&after, i),
            Some(info),
            "button {i} was not reached"
        );
        assert_ne!(label_fg(&before, i), Some(info), "button {i} already was");
    }

    let mut list_before = scene("overrides::list_before", Theme::junie());
    list_before.draw(|ui, _| {
        let s = ui
            .style(
                Family::LIST,
                Variant::DEFAULT,
                Part::LABEL,
                StateFlags::empty(),
            )
            .style;
        ui.paint_str(Rect::new(0, 0, 10, 1), "row", s);
    });
    let theme2 = Theme::junie().override_family(Family::BUTTON, |e| {
        e.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut list_after = scene("overrides::list_after", theme2);
    list_after.draw(|ui, _| {
        let s = ui
            .style(
                Family::LIST,
                Variant::DEFAULT,
                Part::LABEL,
                StateFlags::empty(),
            )
            .style;
        ui.paint_str(Rect::new(0, 0, 10, 1), "row", s);
    });
    assert_eq!(list_before.digest(), list_after.digest());
}

#[test]
fn global_variant_override_changes_only_that_variant() {
    let before = baseline("overrides::before");
    let theme = Theme::junie().override_variant(Family::BUTTON, Variant::PRIMARY, |e| {
        e.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Warning));
    });
    let mut after = scene("overrides::variant", theme);
    after.draw(|ui, _| paint(ui, &plain));

    let warning = Theme::junie().color.warning;

    assert_eq!(label_fg(&after, 0), Some(warning));
    assert_eq!(label_fg(&after, 1), Some(warning));

    assert_eq!(label_fg(&after, 2), label_fg(&before, 2));
    assert_eq!(label_bg(&after, 2), label_bg(&before, 2));
}

#[test]
fn scoped_overlay_changes_only_the_subtree() {
    static RULES: [OverlayRule; 1] = [(
        Family::BUTTON,
        Variant::PRIMARY,
        Part::LABEL,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Danger),
    )];
    let overlay = Overlay::new(&RULES);
    let before = baseline("overrides::before");

    let theme = Theme::junie();
    let fingerprint = theme.fingerprint();
    let mut after = scene("overrides::overlay", theme.clone());
    after.draw(|ui, _| {
        paint(ui, &|ui, i, f| {
            if i == 1 {
                ui.with_overlay(&overlay, |ui| f(ui));
            } else {
                f(ui);
            }
        });
    });

    let danger = Theme::junie().color.danger;
    assert_eq!(label_fg(&after, 1), Some(danger), "the subtree is restyled");
    assert_eq!(label_fg(&after, 0), label_fg(&before, 0), "sibling above");
    assert_eq!(label_fg(&after, 2), label_fg(&before, 2), "sibling below");
    assert_eq!(
        theme.fingerprint(),
        fingerprint,
        "a scope never mutates the theme"
    );
}

#[test]
fn instance_patch_changes_only_one_instance() {
    const PATCH: [(Part, StylePatch); 1] = [(Part::LABEL, StylePatch::new().set_fg(Role::Accent))];
    let before = baseline("overrides::before");
    let mut after = scene("overrides::instance", Theme::junie());
    after.draw(|ui, _| {
        for (i, (id, variant, rect)) in LAYOUT.iter().enumerate() {
            let b = Button::new(*id, "Label").variant(*variant);
            if i == 1 {
                b.patch_part(&PATCH).draw(ui, *rect);
            } else {
                b.draw(ui, *rect);
            }
        }
    });

    assert_eq!(label_fg(&after, 1), Some(Theme::junie().color.accent));

    assert_eq!(label_fg(&after, 0), label_fg(&before, 0));
    assert_eq!(label_fg(&after, 2), label_fg(&before, 2));
}

#[test]
fn part_slot_replaces_the_part_and_keeps_hit_regions() {
    let before = baseline("overrides::before");
    let before_regions = before
        .registry()
        .map(|r| r.regions().len())
        .expect("the scene owns a runtime");
    let before_area = before
        .registry()
        .and_then(|r| r.area_of(B))
        .expect("B registered");

    let mut after = scene("overrides::slot", Theme::junie());
    let gutter = |ui: &mut Ui<'_>, r: Rect| {
        let s = ui.surface_style();
        ui.paint_str(r, "!", s);
    };
    after.draw(|ui, _| {
        for (i, (id, variant, rect)) in LAYOUT.iter().enumerate() {
            let b = Button::new(*id, "Label").variant(*variant);
            if i == 1 {
                b.slot(Part::GUTTER, &gutter).draw(ui, *rect);
            } else {
                b.draw(ui, *rect);
            }
        }
    });

    let r = LAYOUT[1].2;
    assert_eq!(
        after.buffer().cell((r.x, r.y)).map(Cell::symbol),
        Some("!"),
        "the slot paints the gutter cell"
    );
    assert_ne!(
        before.buffer().cell((r.x, r.y)).map(Cell::symbol),
        Some("!")
    );

    assert_eq!(
        after.buffer().cell((r.x + 1, r.y)).map(Cell::symbol),
        Some("L")
    );

    assert_eq!(
        after.registry().map(|r| r.regions().len()),
        Some(before_regions)
    );
    assert_eq!(
        after.registry().and_then(|r| r.area_of(B)),
        Some(before_area)
    );
    let centre = Position::new(
        before_area.x + before_area.width / 2,
        before_area.y + before_area.height / 2,
    );
    let hit = after
        .registry()
        .and_then(|r| r.hit(centre))
        .expect("the slot did not remove the button's hit region");
    assert_eq!(hit.owner, B);
    assert_eq!(hit.part, PartRef::of(Part::CONTAINER));
    assert_eq!(
        after.ring().map(|r| r.reachable().count()),
        before.ring().map(|r| r.reachable().count())
    );
}
