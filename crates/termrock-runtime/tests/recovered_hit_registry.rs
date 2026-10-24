use ratatui_core::layout::{Position, Rect};
use termrock::{Diagnostic, Hit, Id, LayerId, Part, PartRef, RegionKind, Registry};

const OWNER: Id = Id::root("hit_registry.recovered_duplicate");

#[test]
fn duplicate_control_diagnostic_keeps_second_registered_area_hittable() {
    let first = Rect::new(1, 1, 3, 2);
    let second = Rect::new(8, 4, 6, 5);
    let mut registry = Registry::new(1);

    assert_eq!(registry.register_control(OWNER, first, LayerId::PAGE), None);
    assert_eq!(
        registry.register_control(OWNER, second, LayerId::PAGE),
        Some(Diagnostic::DuplicateId {
            id: OWNER,
            first,
            second,
        })
    );

    assert_eq!(
        registry.hit(Position::new(2, 2)),
        Some(Hit {
            owner: OWNER,
            part: PartRef::of(Part::CONTAINER),
            layer: LayerId::PAGE,
            kind: RegionKind::Control,
            local: Position::new(1, 1),
        }),
        "the first registration remains hittable"
    );
    assert_eq!(
        registry.hit(Position::new(10, 6)),
        Some(Hit {
            owner: OWNER,
            part: PartRef::of(Part::CONTAINER),
            layer: LayerId::PAGE,
            kind: RegionKind::Control,
            local: Position::new(2, 2),
        }),
        "the diagnostic does not remove the second disjoint registration"
    );
    assert_eq!(registry.hit(Position::new(30, 20)), None);
}
