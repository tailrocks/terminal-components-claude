//! The five customisation scenarios of goal §15 / `COMPONENT_ARCHITECTURE.md`
//! §11.3, proven on the prototype before the API is frozen (Slice 2
//! acceptance condition 6).
//!
//! Each test asserts the **scope** of an override, not only its effect: an
//! override that changes more than it should is exactly the defect the
//! six-level precedence chain exists to prevent. Colour is read off the
//! painted cells, so what is asserted is what a terminal would show.
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

use tui_next::{
    Button, Color, ColorLevel, Family, Id, Overlay, OverlayRule, Part, PartRef, Position, Rect,
    Role, StateFlags, StylePatch, Theme, Ui, Variant,
};
use tui_next_testing::Scene;

const A: Id = Id::root("ov.a");
const B: Id = Id::root("ov.b");
const C: Id = Id::root("ov.c");

/// `(id, variant, rect)` for the three buttons every scenario paints.
const LAYOUT: [(Id, Variant, Rect); 3] = [
    (A, Variant::PRIMARY, Rect::new(0, 0, 20, 1)),
    (B, Variant::PRIMARY, Rect::new(0, 2, 20, 1)),
    (C, Variant::SECONDARY, Rect::new(0, 4, 20, 1)),
];

fn scene(name: &'static str, theme: Theme) -> Scene {
    Scene::new(name, theme, ColorLevel::TrueColor, 30, 6)
}

/// Paint the three buttons; `wrap` receives the index and may push a scope.
fn paint(ui: &mut Ui<'_>, wrap: &dyn Fn(&mut Ui<'_>, usize, &dyn Fn(&mut Ui<'_>))) {
    for (i, (id, variant, rect)) in LAYOUT.iter().enumerate() {
        wrap(ui, i, &|ui: &mut Ui<'_>| {
            Button::new(*id, "Label").variant(*variant).draw(ui, *rect);
        });
    }
}

fn plain(ui: &mut Ui<'_>, _i: usize, f: &dyn Fn(&mut Ui<'_>)) {
    f(ui);
}

/// The label colour of button `i`: the second column of its row, which is the
/// first label cell (column 0 is the gutter).
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

/// Goal §15 scenario 4 (precedence 4, family-wide): one `override_family`
/// call reaches every button of every variant, and nothing else.
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
    // a family override reaches only its family: the list's label is untouched
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

/// Goal §15 scenario 5 (precedence 4, one variant): the same call keyed on a
/// variant reaches that variant's instances and leaves the siblings alone.
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
    // A and B are PRIMARY
    assert_eq!(label_fg(&after, 0), Some(warning));
    assert_eq!(label_fg(&after, 1), Some(warning));
    // C is SECONDARY and is byte-identical to the un-overridden run
    assert_eq!(label_fg(&after, 2), label_fg(&before, 2));
    assert_eq!(label_bg(&after, 2), label_bg(&before, 2));
}

/// Goal §15 scenario 6 (precedence 5): a `with_overlay` scope is a draw-time
/// stack, so it reaches the subtree it wraps and unwinds afterwards. It never
/// mutates the theme.
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
            // only the middle button is inside the scope
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

/// Goal §15 scenario 7 (precedence 6): `.patch_part` reaches one instance and
/// costs nothing per frame — the patch is a `const`.
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
    // A is the same variant as B and is untouched, so the patch is not a
    // variant-level change wearing an instance's name
    assert_eq!(label_fg(&after, 0), label_fg(&before, 0));
    assert_eq!(label_fg(&after, 2), label_fg(&before, 2));
}

/// Goal §15 scenario 8: `.slot` replaces one part's **painting** and keeps
/// everything else — layout, hit regions, focus registration and the other
/// parts.
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
        after.buffer().cell((r.x, r.y)).map(|c| c.symbol()),
        Some("!"),
        "the slot paints the gutter cell"
    );
    assert_ne!(
        before.buffer().cell((r.x, r.y)).map(|c| c.symbol()),
        Some("!")
    );
    // the label is still the component's own
    assert_eq!(
        after.buffer().cell((r.x + 1, r.y)).map(|c| c.symbol()),
        Some("L")
    );
    // …and the hit regions, the ring entry and the geometry are unchanged
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
