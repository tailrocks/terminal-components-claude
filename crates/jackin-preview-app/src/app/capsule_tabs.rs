use std::collections::BTreeSet;

use termrock::{
    Family, GlyphRole, ItemKey, Modifier, Overlay, OverlayRule, Part, Role, Slot, StateFlags,
    StylePatch, Surface, Variant, navigation::TabAffixes,
};

use crate::{
    domain::instance::{AgentState, InstanceStatus, RunId},
    sim::{
        pty::{Daemon, Pane, PaneId},
        world::World,
    },
};

static OVERLAY_RULES: [OverlayRule; 16] = [
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::CONTAINER,
        StateFlags::empty(),
        StylePatch::new()
            .set_fg(Role::Fg(termrock::FgStep::Primary))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::TAB,
        StateFlags::empty(),
        StylePatch::new()
            .set_fg(Role::Fg(termrock::FgStep::Secondary))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::TAB,
        StateFlags::HOVERED,
        StylePatch::new()
            .set_fg(Role::Fg(termrock::FgStep::Secondary))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::TAB,
        StateFlags::ACTIVE,
        StylePatch::new()
            .set_fg(Role::Fg(termrock::FgStep::Primary))
            .set_bg(Role::Surface(Surface::Overlay))
            .add(Modifier::BOLD),
    ),
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::LABEL,
        StateFlags::empty(),
        StylePatch::new()
            .set_fg(Role::Fg(termrock::FgStep::Secondary))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::LABEL,
        StateFlags::HOVERED,
        StylePatch::new()
            .set_fg(Role::Fg(termrock::FgStep::Secondary))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::LABEL,
        StateFlags::ACTIVE,
        StylePatch::new()
            .set_fg(Role::Fg(termrock::FgStep::Primary))
            .set_bg(Role::Surface(Surface::Overlay))
            .add(Modifier::BOLD),
    ),
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::PREFIX,
        StateFlags::empty(),
        StylePatch::new()
            .set_fg(Role::Fg(termrock::FgStep::Muted))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::PREFIX,
        StateFlags::HOVERED,
        StylePatch::new()
            .set_fg(Role::Fg(termrock::FgStep::Muted))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::PREFIX,
        StateFlags::ACTIVE,
        StylePatch::new()
            .set_fg(Role::Fg(termrock::FgStep::Secondary))
            .set_bg(Role::Surface(Surface::Overlay))
            .remove(Modifier::BOLD),
    ),
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::META,
        StateFlags::empty(),
        StylePatch::new()
            .set_fg(Role::Fg(termrock::FgStep::Muted))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::META,
        StateFlags::HOVERED,
        StylePatch::new()
            .set_fg(Role::Fg(termrock::FgStep::Muted))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    ),
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::META,
        StateFlags::ACTIVE,
        StylePatch::new()
            .set_fg(Role::Fg(termrock::FgStep::Secondary))
            .set_bg(Role::Surface(Surface::Overlay))
            .remove(Modifier::BOLD),
    ),
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::NEW,
        StateFlags::empty(),
        StylePatch {
            glyph: Slot::Clear,
            ..StylePatch::new()
                .set_fg(Role::Fg(termrock::FgStep::Primary))
                .set_bg(Role::Surface(Surface::Canvas))
                .remove(Modifier::BOLD)
        },
    ),
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::RULE,
        StateFlags::empty(),
        StylePatch::new()
            .set_fg(Role::Custom(termrock::Color::Rgb(0x3a, 0x3a, 0x3a)))
            .set_glyph(GlyphRole::RuleQuiet),
    ),
    (
        Family::TABS,
        Variant::DEFAULT,
        Part::RULE,
        StateFlags::ACTIVE,
        StylePatch::new()
            .set_fg(Role::Accent)
            .set_glyph(GlyphRole::RuleActive),
    ),
];

pub(super) const OVERLAY: Overlay = Overlay::new(&OVERLAY_RULES);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) struct SourceBinding {
    pub(super) instance_id: String,
    pub(super) run_id: RunId,
    pub(super) anchor: PaneId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Target {
    Live(SourceBinding),
    AuthoredIndex(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CapsuleTab {
    pub(super) binding: SourceBinding,
    pub(super) key: ItemKey,
    label: String,
    prefix: String,
    suffix: String,
}

impl std::fmt::Display for CapsuleTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Projection {
    Authored {
        labels: Vec<String>,
        active_index: usize,
    },
    Live {
        items: Vec<CapsuleTab>,
        active_index: usize,
    },
    Invalid,
}

impl Projection {
    pub(super) fn active_key(&self) -> Option<(usize, ItemKey)> {
        match self {
            Self::Authored {
                labels,
                active_index,
            } => {
                let index = (*active_index).min(labels.len().saturating_sub(1));
                labels.get(index).map(|_| (index, ItemKey::index(index)))
            }
            Self::Live {
                items,
                active_index,
            } => items
                .get(*active_index)
                .map(|item| (*active_index, item.key)),
            Self::Invalid => None,
        }
    }

    pub(super) fn active_target(&self) -> Option<Target> {
        match self {
            Self::Authored {
                labels,
                active_index,
            } => {
                let index = (*active_index).min(labels.len().saturating_sub(1));
                labels.get(index).map(|_| Target::AuthoredIndex(index))
            }
            Self::Live {
                items,
                active_index,
            } => items
                .get(*active_index)
                .map(|item| Target::Live(item.binding.clone())),
            Self::Invalid => None,
        }
    }

    pub(super) fn target_for_key(&self, key: ItemKey) -> Option<Target> {
        match self {
            Self::Authored { labels, .. } => match key {
                ItemKey::Index(index) if index < labels.len() => Some(Target::AuthoredIndex(index)),
                ItemKey::Index(_) | ItemKey::Num(_) | ItemKey::Text(_) => None,
            },
            Self::Live { items, .. } => items
                .iter()
                .find(|item| item.key == key)
                .map(|item| Target::Live(item.binding.clone())),
            Self::Invalid => None,
        }
    }

    pub(super) fn target_label(&self, target: &Target) -> Option<String> {
        match (self, target) {
            (Self::Live { items, .. }, Target::Live(binding)) => items
                .iter()
                .find(|item| &item.binding == binding)
                .map(|item| item.label.clone()),
            (Self::Authored { .. }, Target::AuthoredIndex(_))
            | (Self::Live { .. }, Target::AuthoredIndex(_))
            | (Self::Authored { .. }, Target::Live(_))
            | (Self::Invalid, _) => None,
        }
    }
}

pub(super) fn item_key(item: &CapsuleTab) -> ItemKey {
    item.key
}

pub(super) fn item_affixes(item: &CapsuleTab) -> TabAffixes {
    TabAffixes::default()
        .prefix(item.prefix.as_str())
        .suffix(item.suffix.as_str())
        .plate_separators(true)
        .trailing_padding(1)
}

pub(super) fn project(
    world: &World,
    active_instance_id: Option<&str>,
    authored_title: &str,
    authored_title_index: usize,
) -> Projection {
    let Some(instance_id) = active_instance_id else {
        return authored_projection(world, None, authored_title, authored_title_index);
    };

    let mut matching_instances = world
        .instances
        .iter()
        .filter(|instance| instance.id.as_str() == instance_id);
    let Some(instance) = matching_instances.next() else {
        return Projection::Invalid;
    };
    if matching_instances.next().is_some() || instance.status != InstanceStatus::Running {
        return Projection::Invalid;
    }

    let Some(daemon) = world.daemons.get(instance_id) else {
        return authored_projection(world, None, authored_title, authored_title_index);
    };
    if daemon.tabs.is_empty() {
        return Projection::Live {
            items: Vec::new(),
            active_index: 0,
        };
    }
    if !valid_live_daemon(daemon) {
        return Projection::Invalid;
    }

    let mut items = Vec::with_capacity(daemon.tabs.len());
    let mut bindings = BTreeSet::new();
    let mut keys = BTreeSet::new();
    for (index, tab) in daemon.tabs.iter().enumerate() {
        let leaves = tab.leaves();
        let Some(anchor) = leaves.iter().min().copied() else {
            return Projection::Invalid;
        };
        let binding = SourceBinding {
            instance_id: instance.id.clone(),
            run_id: instance.run_id,
            anchor,
        };
        let key = item_key_for_binding(&binding);
        if !bindings.insert(binding.clone()) || !keys.insert(key) {
            return Projection::Invalid;
        }

        let label = daemon.tab_label(tab, &|pane| account_label(world, pane));
        let status = daemon.tab_state(tab);
        let prefix = match index {
            0..=8 => (index + 1).to_string(),
            9 => "0".to_owned(),
            _ => String::new(),
        };
        let suffix = status_suffix(status).to_owned();
        items.push(CapsuleTab {
            binding,
            key,
            label,
            prefix,
            suffix,
        });
    }

    Projection::Live {
        items,
        active_index: daemon.active,
    }
}

pub(super) fn resolve_target(
    world: &World,
    active_instance_id: Option<&str>,
    authored_title: &str,
    authored_title_index: usize,
    target: &Target,
) -> Option<usize> {
    let projection = project(
        world,
        active_instance_id,
        authored_title,
        authored_title_index,
    );
    match (projection, target) {
        (Projection::Live { items, .. }, Target::Live(binding)) => {
            items.iter().position(|item| &item.binding == binding)
        }
        (Projection::Authored { labels, .. }, Target::AuthoredIndex(index))
            if *index < labels.len() =>
        {
            Some(*index)
        }
        _ => None,
    }
}

pub(super) fn item_key_for_binding(binding: &SourceBinding) -> ItemKey {
    // The World/Daemon schema has no run epoch beyond RunId. An identical
    // same-id/same-run replacement that recycles the same pane IDs is therefore
    // indistinguishable here; future restart owners must invalidate this
    // binding or provide an incarnation token at their boundary.
    let encoded = format!(
        "jackin.capsule.tab/v1/{}/{}:{}:{}",
        binding.instance_id.len(),
        binding.instance_id,
        binding.run_id.value(),
        binding.anchor,
    );
    ItemKey::text(&encoded)
}

fn valid_live_daemon(daemon: &Daemon) -> bool {
    if daemon.tabs.is_empty() || daemon.active >= daemon.tabs.len() || daemon.next_id == u64::MAX {
        return false;
    }

    let pane_ids = daemon
        .panes
        .iter()
        .map(|pane| pane.id)
        .collect::<BTreeSet<_>>();
    if pane_ids.len() != daemon.panes.len()
        || pane_ids
            .last()
            .is_some_and(|largest| *largest >= daemon.next_id)
    {
        return false;
    }

    let mut referenced = BTreeSet::new();
    for tab in &daemon.tabs {
        let leaves = tab.leaves();
        let local = leaves.iter().copied().collect::<BTreeSet<_>>();
        if leaves.is_empty()
            || local.len() != leaves.len()
            || !local.contains(&tab.focused)
            || tab.zoomed.is_some_and(|zoomed| !local.contains(&zoomed))
        {
            return false;
        }
        for pane_id in leaves {
            if !pane_ids.contains(&pane_id) || !referenced.insert(pane_id) {
                return false;
            }
        }
    }
    referenced == pane_ids
}

fn account_label(world: &World, pane: &Pane) -> Option<String> {
    pane.proc
        .account
        .as_ref()
        .and_then(|id| world.accounts.get(id))
        .map(|account| account.display_name.clone())
}

fn status_suffix(status: AgentState) -> &'static str {
    match status {
        AgentState::Blocked => "●",
        AgentState::Working => "▶",
        AgentState::Idle => "◆",
        AgentState::Done | AgentState::Unknown => "",
    }
}

fn authored_projection(
    world: &World,
    daemon: Option<&Daemon>,
    authored_title: &str,
    authored_title_index: usize,
) -> Projection {
    let mut dynamic = daemon
        .map(|daemon| {
            daemon
                .tabs
                .iter()
                .map(|tab| {
                    let mut label = daemon.tab_label(tab, &|pane| account_label(world, pane));
                    let glyph = status_suffix(daemon.tab_state(tab));
                    if !glyph.is_empty() {
                        label.push(' ');
                        label.push_str(glyph);
                    }
                    label
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if dynamic.is_empty() {
        dynamic.push("Mix (3) ●".into());
    }
    if dynamic.len() < 2 {
        dynamic.push("Shell".into());
    }
    if !authored_title.is_empty()
        && let Some(label) = dynamic.get_mut(authored_title_index)
    {
        *label = authored_title.to_owned();
    }
    if dynamic.len() < 3 {
        dynamic.push("docs ●".into());
    }
    let labels = dynamic
        .into_iter()
        .enumerate()
        .map(|(index, label)| format!("{} {label} ", index.saturating_add(1)))
        .collect::<Vec<_>>();
    let active_index = daemon.map_or(authored_title_index, |daemon| daemon.active);
    Projection::Authored {
        labels,
        active_index,
    }
}

#[cfg(test)]
mod tests {
    use super::{Projection, Target, project, resolve_target};
    use crate::{
        app::{App, CAPSULE_TABS},
        domain::instance::{AgentState, InstanceStatus},
        scenario::{Motion, Scenario},
    };
    use std::collections::BTreeSet;
    use termrock::{ColorLevel, KeyCode, MouseKind, Part, PartRef, Theme};
    use termrock_test_support::Harness;

    #[test]
    fn instance_id_replacement_with_reused_pane_numbers_rejects_old_binding() {
        let mut app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40);
        let old_instance_id = app
            .active_instance
            .clone()
            .expect("CapsuleMulti has an active instance");
        let projection = project(&app.world, Some(&old_instance_id), "", 0);
        let target = projection.active_target().expect("live active tab");
        let old_run_id = app
            .world
            .instance(&old_instance_id)
            .expect("the original instance exists")
            .run_id;
        let old_panes = app.world.daemons[&old_instance_id]
            .panes
            .iter()
            .map(|pane| pane.id)
            .collect::<BTreeSet<_>>();
        assert!(matches!(&projection, Projection::Live { .. }));
        assert!(matches!(&target, Target::Live(_)));

        let replacement_id = format!("{old_instance_id}-replacement");
        let daemon = app
            .world
            .daemons
            .remove(&old_instance_id)
            .expect("the old instance owns a daemon");
        let instance = app
            .world
            .instances
            .iter_mut()
            .find(|instance| instance.id == old_instance_id)
            .expect("the old instance exists");
        instance.id = replacement_id.clone();
        app.world.daemons.insert(replacement_id.clone(), daemon);
        app.active_instance = Some(replacement_id.clone());
        let snapshot = app.world.daemons[&replacement_id].snapshot();
        app.world
            .instance_mut(&replacement_id)
            .expect("replacement instance exists")
            .daemon = snapshot;

        let new_panes = app.world.daemons[&replacement_id]
            .panes
            .iter()
            .map(|pane| pane.id)
            .collect::<BTreeSet<_>>();
        assert_eq!(old_panes, new_panes, "pane numbers were reused exactly");
        assert_eq!(
            app.world
                .instance(&replacement_id)
                .expect("replacement instance exists")
                .run_id,
            old_run_id,
            "the fixture keeps RunId while Instance.id changes"
        );
        assert_eq!(
            resolve_target(&app.world, Some(&replacement_id), "", 0, &target,),
            None,
            "the captured full binding still carries the old Instance.id"
        );
    }

    #[test]
    fn instance_id_replacement_with_reused_pane_numbers_invalidates_pending_title() {
        let mut harness = Harness::new(
            App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        )
        .with_color(ColorLevel::TrueColor);
        let old_instance_id = {
            let app = harness.app();
            app.world
                .instances
                .iter()
                .find(|instance| {
                    instance.status == InstanceStatus::Running
                        && app.world.daemons.contains_key(instance.id.as_str())
                })
                .expect("CapsuleMulti has a running daemon")
                .id
                .clone()
        };
        let now_ms = harness.app().world.now_ms();
        {
            let daemon = harness
                .app_mut()
                .world
                .daemons
                .get_mut(&old_instance_id)
                .expect("running daemon");
            while daemon.tabs.len() < 4 {
                daemon.new_tab(None, None, now_ms, false);
            }
            for index in 0..4 {
                daemon.tabs[index].custom_label = Some(format!("tab-{index:02}"));
            }
            for (index, state) in [AgentState::Blocked, AgentState::Working, AgentState::Idle]
                .into_iter()
                .enumerate()
            {
                let leaves = daemon.tabs[index].leaves();
                for pane_id in leaves {
                    daemon
                        .pane_mut(pane_id)
                        .expect("tab leaf has a live pane")
                        .proc
                        .state = state;
                }
            }
            daemon.active = 0;
        }
        let snapshot = harness.app().world.daemons[&old_instance_id].snapshot();
        harness
            .app_mut()
            .world
            .instance_mut(&old_instance_id)
            .expect("running instance")
            .daemon = snapshot;
        harness.draw();
        let old_run_id = harness
            .app()
            .world
            .instance(&old_instance_id)
            .expect("old instance exists")
            .run_id;
        let old_key = project(&harness.app().world, Some(&old_instance_id), "", 0)
            .active_key()
            .expect("active live tab has a native key")
            .1;
        let old_panes = harness.app().world.daemons[&old_instance_id]
            .panes
            .iter()
            .map(|pane| pane.id)
            .collect::<Vec<_>>();

        let tab_area = harness
            .area_of_part(CAPSULE_TABS, PartRef::item(Part::TAB, old_key))
            .expect("the original tab has a native owner");
        let rename_at = (tab_area.x.saturating_add(tab_area.width / 2), tab_area.y);
        let _ = harness.mouse(MouseKind::Secondary, rename_at.0, rename_at.1);
        assert!(
            harness.find("Change title…").is_some(),
            "native tab context menu is visible"
        );
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.find("Change tab title").is_some(),
            "native title dialog is visible"
        );
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.type_str("must not apply after instance replacement");

        let replacement_id = format!("{old_instance_id}-replacement");
        let daemon = harness
            .app_mut()
            .world
            .daemons
            .remove(&old_instance_id)
            .expect("old instance owns a daemon");
        {
            let app = harness.app_mut();
            let world = &mut app.world;
            let instance = world
                .instances
                .iter_mut()
                .find(|instance| instance.id == old_instance_id)
                .expect("old instance exists");
            instance.id = replacement_id.clone();
            world.daemons.insert(replacement_id.clone(), daemon);
            let snapshot = world.daemons[&replacement_id].snapshot();
            world
                .instance_mut(&replacement_id)
                .expect("replacement instance exists")
                .daemon = snapshot;
            app.active_instance = Some(replacement_id.clone());
        }

        let replacement = &harness.app().world.daemons[&replacement_id];
        let replacement_panes = replacement
            .panes
            .iter()
            .map(|pane| pane.id)
            .collect::<Vec<_>>();
        assert_eq!(old_panes, replacement_panes, "pane IDs are reused exactly");
        assert_eq!(
            harness
                .app()
                .world
                .instance(&replacement_id)
                .expect("replacement instance exists")
                .run_id,
            old_run_id,
            "the fixture keeps RunId constant while replacing Instance.id"
        );

        harness.draw();
        let _ = harness.key(KeyCode::Enter);
        assert!(
            harness.find("Tab changed · choose again").is_some(),
            "the pending title is rejected after the source instance changes"
        );
        assert!(
            harness.app().world.daemons[&replacement_id]
                .tabs
                .iter()
                .all(|tab| {
                    tab.custom_label.as_deref() != Some("must not apply after instance replacement")
                })
        );
        assert!(
            harness
                .area_of_part(CAPSULE_TABS, PartRef::item(Part::TAB, old_key),)
                .is_none(),
            "reused pane IDs do not keep the old source key"
        );
        let replacement_key = project(&harness.app().world, Some(&replacement_id), "", 0)
            .active_key()
            .expect("replacement active tab has a native key")
            .1;
        assert!(
            harness
                .area_of_part(CAPSULE_TABS, PartRef::item(Part::TAB, replacement_key),)
                .is_some()
        );
        assert_eq!(harness.top_layer(), termrock::LayerId::PAGE);
        assert_eq!(harness.runtime().capture_owner(), None);
        assert!(
            harness.diagnostics().is_empty(),
            "pending-title diagnostics: {:?}; frame={}",
            harness.diagnostics(),
            harness.text()
        );
    }
}
