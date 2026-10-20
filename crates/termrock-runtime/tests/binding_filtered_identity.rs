use std::cell::RefCell;
use std::sync::LazyLock;

use termrock_runtime::author::{
    ActionKey, Binding, BindingTableId, Chord, Diagnostic, Focusability, Id, KeyCode, KeyMap,
    StateFlags, Theme, extend_bindings, publish_bindings_filtered,
};
use termrock_runtime::{App, Cx, Response, Size, Ui};
use termrock_test_support::Harness;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Command {
    A,
    B,
    Both,
}

const OWNER: Id = Id::root("binding.filtered.owner");
const ACTION_A: ActionKey = ActionKey::custom("binding.filtered.a");
const ACTION_B: ActionKey = ActionKey::custom("binding.filtered.b");
const TABLE: &[Binding<Command>] = &[
    Binding {
        action: ACTION_A,
        chord: Some(Chord::key(KeyCode::Up)),
        cmd: Command::A,
        label: "Alpha",
        priority: 1,
        visible: true,
    },
    Binding {
        action: ACTION_B,
        chord: Some(Chord::key(KeyCode::Down)),
        cmd: Command::B,
        label: "Beta",
        priority: 1,
        visible: true,
    },
];
const BASE: &[Binding<()>] = &[Binding {
    action: ActionKey::custom("binding.filtered.base"),
    chord: Some(Chord::key(KeyCode::Enter)),
    cmd: (),
    label: "Base",
    priority: 0,
    visible: true,
}];

static NO_REMAP: LazyLock<KeyMap> = LazyLock::new(KeyMap::new);
static CONFLICTING_REMAP: LazyLock<KeyMap> =
    LazyLock::new(|| KeyMap::new().bind_component(OWNER, ACTION_A, Chord::key(KeyCode::Down)));

struct BindingPage {
    selected: Command,
    composite: bool,
    remapped: bool,
    hints: RefCell<Vec<&'static str>>,
}

impl App for BindingPage {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn min_size(&self) -> Size {
        Size::exact(0, 0)
    }

    fn keymap(&self) -> &KeyMap {
        if self.remapped {
            &CONFLICTING_REMAP
        } else {
            &NO_REMAP
        }
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(OWNER, ui.full(), Focusability::Focusable);
        if self.composite {
            ui.publish_bindings(OWNER, StateFlags::FOCUSED, BASE);
            extend_bindings(ui, OWNER, StateFlags::FOCUSED, TABLE, |command| {
                self.selected == Command::Both || self.selected == command
            })
            .expect("selected binding set is valid");
        } else {
            publish_bindings_filtered(ui, OWNER, StateFlags::FOCUSED, TABLE, |command| {
                self.selected == Command::Both || self.selected == command
            });
        }
        if let Some(hints) = ui.with_focused_hints(|_, layer| {
            layer
                .hints
                .iter()
                .filter(|hint| hint.label != "Base")
                .map(|hint| hint.label)
                .collect()
        }) {
            *self.hints.borrow_mut() = hints;
        }
    }
}

fn page(selected: Command, composite: bool, remapped: bool) -> BindingPage {
    BindingPage {
        selected,
        composite,
        remapped,
        hints: RefCell::new(Vec::new()),
    }
}

#[test]
fn filtered_static_table_refreshes_hints_for_each_selected_subset() {
    let mut harness = Harness::new(page(Command::A, false, false), Theme::junie(), 20, 2);
    for (selected, expected) in [
        (Command::B, "Beta"),
        (Command::A, "Alpha"),
        (Command::B, "Beta"),
        (Command::A, "Alpha"),
    ] {
        harness.app_mut().selected = selected;
        harness.draw();
        assert_eq!(*harness.app().hints.borrow(), [expected]);
    }
}

#[test]
fn composite_table_refreshes_hints_across_repeated_frame_swaps() {
    let mut harness = Harness::new(page(Command::A, true, false), Theme::junie(), 20, 2);
    for (selected, expected) in [(Command::B, "Beta"), (Command::A, "Alpha")]
        .into_iter()
        .cycle()
        .take(20)
    {
        harness.app_mut().selected = selected;
        harness.draw();
        assert_eq!(*harness.app().hints.borrow(), [expected]);
    }
}

#[test]
fn filtered_selection_and_keymap_remap_refresh_conflict_diagnostics() {
    let mut harness = Harness::new(page(Command::Both, false, true), Theme::junie(), 20, 2);
    let has_conflict = |harness: &Harness<BindingPage>| {
        harness.diagnostics().iter().any(|diagnostic| {
            matches!(diagnostic, Diagnostic::BindingConflict { a, b, .. } if *a == OWNER && *b == OWNER)
        })
    };

    let _ = harness.key(KeyCode::F(12));
    assert!(
        has_conflict(&harness),
        "diagnostics: {:?}",
        harness.diagnostics()
    );
    harness.app_mut().selected = Command::A;
    harness.draw();
    let _ = harness.key(KeyCode::F(12));
    assert!(!has_conflict(&harness));
    harness.app_mut().selected = Command::Both;
    harness.draw();
    let _ = harness.key(KeyCode::F(12));
    assert!(has_conflict(&harness));

    harness.app_mut().remapped = false;
    harness.draw();
    let _ = harness.key(KeyCode::F(12));
    assert!(!has_conflict(&harness));
    harness.app_mut().remapped = true;
    harness.draw();
    let _ = harness.key(KeyCode::F(12));
    assert!(has_conflict(&harness));
}

#[test]
fn public_binding_table_id_keeps_its_original_exhaustive_variant_set() {
    fn shape(identity: BindingTableId) -> u8 {
        match identity {
            BindingTableId::Static {
                command_type: _,
                address: _,
                len: _,
            } => 0,
            BindingTableId::Dynamic {
                revision: _,
                len: _,
            } => 1,
        }
    }

    assert_eq!(
        [
            shape(BindingTableId::Static {
                command_type: std::any::TypeId::of::<()>(),
                address: 0,
                len: 0,
            }),
            shape(BindingTableId::Dynamic {
                revision: 0,
                len: 0,
            }),
        ],
        [0, 1]
    );
}
