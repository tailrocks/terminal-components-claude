#![allow(
    dead_code,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    clippy::arithmetic_side_effects,
    clippy::doc_markdown,
    clippy::explicit_iter_loop,
    clippy::indexing_slicing,
    clippy::many_single_char_names,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::too_many_lines,
    clippy::unwrap_used,
    clippy::expect_used
)]

mod support;

use jackin_preview_app::{
    App, CAPSULE_TABS, Motion, RunId, Scenario,
    domain::agent::Agent,
    domain::instance::{AgentState, InstanceStatus},
    sim::pty::SplitDir,
};
use support::H;
use termrock::{Axis, ColorLevel, ItemKey, KeyCode, MouseKind, Part, PartRef, Rect, Theme};
use termrock_test_support::Harness;

fn capsule(width: u16, height: u16) -> H {
    H::new(Scenario::CapsuleMulti, Motion::Paused, 40, width, height)
}

fn live_instance_id(h: &H) -> String {
    let world = &h.app().world;
    world
        .instances
        .iter()
        .find(|instance| {
            instance.status == InstanceStatus::Running
                && world.daemons.contains_key(instance.id.as_str())
        })
        .expect("CapsuleMulti has a running daemon")
        .id
        .clone()
}

fn sync_snapshot(h: &mut H, instance_id: &str) {
    let snapshot = h
        .app()
        .world
        .daemons
        .get(instance_id)
        .expect("running daemon")
        .snapshot();
    h.app_mut()
        .world
        .instance_mut(instance_id)
        .expect("running instance")
        .daemon = snapshot;
    h.draw();
}

fn populate_tabs(h: &mut H, count: usize) {
    let instance_id = live_instance_id(h);
    let now_ms = h.app().world.now_ms();
    {
        let daemon = h
            .app_mut()
            .world
            .daemons
            .get_mut(&instance_id)
            .expect("running daemon");
        while daemon.tabs.len() < count {
            daemon.new_tab(None, None, now_ms, false);
        }
        for index in 0..count {
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
    sync_snapshot(h, &instance_id);
}

fn tab_anchor(h: &H, index: usize) -> u64 {
    let instance_id = live_instance_id(h);
    h.app().world.daemons[&instance_id].tabs[index]
        .leaves()
        .into_iter()
        .min()
        .expect("tab has a leaf")
}

fn tab_key(h: &H, index: usize) -> ItemKey {
    let instance_id = live_instance_id(h);
    let instance = h
        .app()
        .world
        .instance(&instance_id)
        .expect("running instance");
    let anchor = tab_anchor(h, index);
    let encoded = format!(
        "jackin.capsule.tab/v1/{}/{}:{}:{}",
        instance.id.len(),
        instance.id,
        instance.run_id.value(),
        anchor,
    );
    ItemKey::text(&encoded)
}

fn tab_area(h: &H, index: usize) -> Option<Rect> {
    h.harness
        .area_of_part(CAPSULE_TABS, PartRef::item(Part::TAB, tab_key(h, index)))
}

fn rect_text(h: &H, area: Rect) -> String {
    let mut text = String::new();
    for x in area.x..area.right() {
        text.push_str(
            h.harness
                .buffer()
                .cell((x, area.y))
                .expect("part lies in the frame")
                .symbol(),
        );
    }
    text
}

fn tab_position(h: &H, index: usize) -> (u16, u16) {
    let area = tab_area(h, index).expect("visible native tab has a registered owner");
    (area.x.saturating_add(area.width / 2), area.y)
}

fn begin_context_rename(h: &mut H, index: usize) {
    let (x, y) = tab_position(h, index);
    h.mouse(MouseKind::Secondary, x, y);
    assert!(h.text().contains("Change title…"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Change tab title"), "{}", h.text());
    h.key(KeyCode::Enter);
}

fn finish_title(h: &mut H, title: &str) {
    h.type_str(title);
    h.key(KeyCode::Enter);
}

fn witness_harness(theme: Theme, color: ColorLevel) -> H {
    witness_harness_at_width(theme, color, 400)
}

fn witness_harness_at_width(theme: Theme, color: ColorLevel, width: u16) -> H {
    H {
        harness: Harness::new(
            App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40),
            theme,
            width,
            40,
        )
        .with_color(color),
    }
}

fn prepare_native_witness(h: &mut H) -> String {
    let instance_id = live_instance_id(h);
    let now_ms = h.app().world.now_ms();
    {
        let daemon = h
            .app_mut()
            .world
            .daemons
            .get_mut(&instance_id)
            .expect("running daemon");
        while daemon.tabs.len() < 12 {
            daemon.new_tab(None, None, now_ms, false);
        }
        for (index, tab) in daemon.tabs.iter_mut().enumerate() {
            tab.custom_label = Some(format!("witness-tab-{index:02}"));
        }
        daemon.tabs[0].custom_label = Some("witness authored title".to_owned());
        daemon.tabs[1].custom_label = None;
        for pane_id in daemon.tabs[1].leaves() {
            let pane = daemon.pane_mut(pane_id).expect("tab leaf has a pane");
            pane.proc.agent = Some(Agent::ClaudeCode);
            pane.proc.account = Some("acct-claude-personal".to_owned());
        }
        for (index, state) in [
            AgentState::Blocked,
            AgentState::Working,
            AgentState::Idle,
            AgentState::Done,
            AgentState::Unknown,
        ]
        .into_iter()
        .enumerate()
        {
            for pane_id in daemon.tabs[index].leaves() {
                daemon
                    .pane_mut(pane_id)
                    .expect("tab leaf has a pane")
                    .proc
                    .state = state;
            }
        }
        daemon.active = 0;
    }
    sync_snapshot(h, &instance_id);
    "CAPSULE ACCOUNT WITNESS".clone_into(
        &mut h
            .app_mut()
            .world
            .accounts
            .get_mut("acct-claude-personal")
            .expect("fixture account")
            .display_name,
    );
    h.draw();
    instance_id
}

fn prepare_more_native_witness(h: &mut H) -> String {
    let instance_id = prepare_native_witness(h);
    {
        let daemon = h
            .app_mut()
            .world
            .daemons
            .get_mut(&instance_id)
            .expect("running daemon");
        daemon.tabs[1].custom_label = Some("witness visible other tab".to_owned());
        daemon.tabs[11].custom_label = None;
        for pane_id in daemon.tabs[11].leaves() {
            let pane = daemon.pane_mut(pane_id).expect("tab leaf has a pane");
            pane.proc.agent = Some(Agent::ClaudeCode);
            pane.proc.account = Some("acct-claude-personal".to_owned());
            pane.proc.state = AgentState::Blocked;
        }
    }
    "CAPSULE MORE ACCOUNT WITNESS".clone_into(
        &mut h
            .app_mut()
            .world
            .accounts
            .get_mut("acct-claude-personal")
            .expect("fixture account")
            .display_name,
    );
    sync_snapshot(h, &instance_id);
    instance_id
}

fn native_witness_active_source(h: &H, instance_id: &str) -> String {
    let instance = h.app().world.instance(instance_id);
    let daemon = h.app().world.daemons.get(instance_id);
    let active_source = daemon.and_then(|daemon| {
        daemon.tabs.get(daemon.active).map(|tab| {
            let anchor = tab.leaves().into_iter().min();
            let label = daemon.tab_label(tab, &|pane| {
                pane.proc
                    .account
                    .as_ref()
                    .and_then(|id| h.app().world.accounts.get(id))
                    .map(|account| account.display_name.clone())
            });
            (daemon.active, anchor, label)
        })
    });
    format!(
        "fixture_instance={instance_id:?} instance_status={:?} run_id={:?} daemon_present={} active_tab_source={active_source:?}",
        instance.map(|instance| &instance.status),
        instance.map(|instance| &instance.run_id),
        daemon.is_some(),
    )
}

fn native_witness_frame_origin(h: &H) -> (u16, u16) {
    let area = *h.harness.buffer().area();
    (area.x, area.y)
}

fn native_witness_tabs_point(h: &H) -> (u16, u16) {
    h.harness.area_of(CAPSULE_TABS).map_or_else(
        || native_witness_frame_origin(h),
        |area| (area.x.saturating_add(2), area.y),
    )
}

fn write_native_witness_frame(
    h: &H,
    root: &std::path::Path,
    name: &str,
    source: &str,
    instance_id: &str,
    observations: &str,
) {
    use std::fmt::Write as _;

    let mut output = String::new();
    let buffer = h.harness.buffer();
    let area = *buffer.area();
    let runtime = h.harness.runtime();
    let regions = runtime.registry().regions();
    let generations = regions
        .iter()
        .map(|region| region.generation)
        .collect::<std::collections::BTreeSet<_>>();
    let _ = writeln!(
        output,
        "source={source} frame={name} dimensions={}x{}",
        area.width, area.height
    );
    let _ = writeln!(output, "journey_observations={observations}");
    let _ = writeln!(
        output,
        "active_source={}",
        native_witness_active_source(h, instance_id)
    );
    let title_visibility = [
        "witness authored title",
        "CAPSULE ACCOUNT WITNESS",
        "CAPSULE MORE ACCOUNT WITNESS",
        "witness visible other tab",
        "Changed witness account",
        "Changed witness title",
    ]
    .into_iter()
    .map(|title| (title, h.harness.find(title)))
    .collect::<Vec<_>>();
    let _ = writeln!(output, "title_visibility={title_visibility:?}");
    let _ = writeln!(output, "text:\n{}", h.text());
    let _ = writeln!(output, "region_generations={generations:?}");
    let _ = writeln!(output, "focus={:?}", h.harness.focus());
    let _ = writeln!(output, "hover={:?}", h.harness.hover());
    let _ = writeln!(output, "top_layer={:?}", h.harness.top_layer());
    let _ = writeln!(
        output,
        "capsule_overflow_menu_open={}",
        h.harness.is_open(CAPSULE_TABS.sub("overflow-menu"))
    );
    let _ = writeln!(output, "capture_owner={:?}", runtime.capture_owner());
    let _ = writeln!(output, "cursor={:?}", h.harness.cursor());
    let _ = writeln!(output, "diagnostics={:?}", h.harness.diagnostics());
    let _ = writeln!(
        output,
        "diagnostics_dropped={}",
        runtime.diagnostics_dropped()
    );
    let _ = writeln!(output, "ordered_focus_ring:");
    for (index, entry) in runtime.ring().entries().iter().enumerate() {
        let _ = writeln!(output, "{index}: {entry:?}");
    }
    let _ = writeln!(output, "ordered_regions:");
    for (index, region) in regions.iter().enumerate() {
        let _ = writeln!(output, "{index}: {region:?}");
    }
    let _ = writeln!(output, "complete_cells:");
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            if let Some(cell) = buffer.cell((x, y)) {
                let _ = writeln!(output, "({x},{y})={cell:?}");
            }
        }
    }
    std::fs::write(root.join(format!("{name}.txt")), output)
        .expect("native witness writes into its unique external root");
}

fn write_native_witness_source_map(h: &H, root: &std::path::Path, name: &str) {
    use std::fmt::Write as _;

    let instance_id = live_instance_id(h);
    let instance = h
        .app()
        .world
        .instance(&instance_id)
        .expect("witness instance exists");
    let daemon = &h.app().world.daemons[&instance_id];
    let mut output = String::new();
    let _ = writeln!(
        output,
        "instance_id={instance_id} run_id={}",
        instance.run_id.value()
    );
    for (index, tab) in daemon.tabs.iter().enumerate() {
        let Some(anchor) = tab.leaves().into_iter().min() else {
            let _ = writeln!(output, "{index}: invalid empty tab");
            continue;
        };
        let key = format!(
            "jackin.capsule.tab/v1/{}/{}:{}:{}",
            instance_id.len(),
            instance_id,
            instance.run_id.value(),
            anchor,
        );
        let legacy = h.harness.area_of_part(
            CAPSULE_TABS,
            PartRef::item(Part::TAB, ItemKey::index(index)),
        );
        // Candidate keys are reported for migration and row-owner comparison
        // only. Input uses native Right/More and physical row clicks; no keyed
        // activation is sent.
        let candidate = h
            .harness
            .area_of_part(CAPSULE_TABS, PartRef::item(Part::TAB, ItemKey::text(&key)));
        let label = daemon.tab_label(tab, &|pane| {
            pane.proc
                .account
                .as_ref()
                .and_then(|id| h.app().world.accounts.get(id))
                .map(|account| account.display_name.clone())
        });
        let prefix = match index {
            0..=8 => (index + 1).to_string(),
            9 => "0".to_owned(),
            _ => String::new(),
        };
        let _ = writeln!(
            output,
            "index={index} prefix={prefix:?} label={label:?} status={:?} anchor={anchor} legacy_index_owner={legacy:?} candidate_text_key={key:?} candidate_owner={candidate:?}",
            daemon.tab_state(tab)
        );
    }
    std::fs::write(root.join(format!("{name}-index-to-source-map.txt")), output)
        .expect("native witness writes source map into its unique external root");
}

#[test]
fn capsule_native_witness_exports_complete_preimage_postimage_frames() {
    let Some(root) = std::env::var_os("JACKIN_CAPSULE_TABS_WITNESS_DIR") else {
        return;
    };
    let source = std::env::var("JACKIN_CAPSULE_TABS_WITNESS_SOURCE")
        .expect("set source to preimage or postimage for the paired witness");
    assert!(matches!(source.as_str(), "preimage" | "postimage"));
    let root = std::path::PathBuf::from(root);
    assert!(root.is_absolute(), "witness root must be absolute");
    let cwd = std::env::current_dir().expect("test current directory");
    assert!(
        !root.starts_with(&cwd),
        "witness artifacts must stay outside the repository"
    );
    assert!(!root.exists(), "each paired run needs a unique output root");
    std::fs::create_dir_all(&root).expect("create unique external witness root");

    let configurations = [("junie", Theme::junie()), ("paper", Theme::paper())]
        .into_iter()
        .flat_map(|(theme_name, theme)| {
            [
                ("truecolor", ColorLevel::TrueColor),
                ("mono", ColorLevel::Mono),
            ]
            .into_iter()
            .map(move |(color_name, color)| (theme_name, theme.clone(), color_name, color))
        })
        .collect::<Vec<_>>();

    // Export every preservation control before native journeys. The active
    // change here is a truthful fixture mutation, not an input result.
    for (theme_name, theme, color_name, color) in &configurations {
        let prefix = format!("{theme_name}-{color_name}");
        let mut h = witness_harness((*theme).clone(), *color);
        let instance_id = prepare_native_witness(&mut h);
        write_native_witness_source_map(&h, &root, &prefix);
        write_native_witness_frame(
            &h,
            &root,
            &format!("{prefix}-full-fit-initial"),
            &source,
            &instance_id,
            "fixture prepared; no native input",
        );

        let strip = h.harness.area_of(CAPSULE_TABS);
        let account = h.harness.find("CAPSULE ACCOUNT WITNESS");
        let hover_at = account.unwrap_or_else(|| native_witness_tabs_point(&h));
        let hover_response = h.harness.mouse(MouseKind::Move, hover_at.0, hover_at.1);
        write_native_witness_frame(
            &h,
            &root,
            &format!("{prefix}-inactive-hover"),
            &source,
            &instance_id,
            &format!(
                "strip_owner={strip:?} hover_target={account:?} input=Move@{hover_at:?} response={hover_response:?}"
            ),
        );

        let mut source_changed = witness_harness((*theme).clone(), *color);
        let source_changed_id = prepare_native_witness(&mut source_changed);
        source_changed
            .app_mut()
            .world
            .daemons
            .get_mut(&source_changed_id)
            .expect("running daemon")
            .active = 1;
        sync_snapshot(&mut source_changed, &source_changed_id);
        write_native_witness_frame(
            &source_changed,
            &root,
            &format!("{prefix}-source-driven-active-change"),
            &source,
            &source_changed_id,
            "fixture source mutation daemon.active=1; no native input",
        );

        let mut no_daemon = witness_harness((*theme).clone(), *color);
        let no_daemon_id = prepare_native_witness(&mut no_daemon);
        no_daemon.app_mut().world.daemons.remove(&no_daemon_id);
        no_daemon.draw();
        write_native_witness_frame(
            &no_daemon,
            &root,
            &format!("{prefix}-no-daemon-authored"),
            &source,
            &no_daemon_id,
            "daemon removed from prepared fixture; no native input",
        );

        let mut empty_daemon = witness_harness((*theme).clone(), *color);
        let empty_id = prepare_native_witness(&mut empty_daemon);
        empty_daemon
            .app_mut()
            .world
            .daemons
            .get_mut(&empty_id)
            .expect("running daemon")
            .tabs
            .clear();
        sync_snapshot(&mut empty_daemon, &empty_id);
        write_native_witness_source_map(&empty_daemon, &root, &format!("{prefix}-empty-daemon"));
        write_native_witness_frame(
            &empty_daemon,
            &root,
            &format!("{prefix}-empty-daemon"),
            &source,
            &empty_id,
            "daemon tabs cleared; no native input",
        );

        for count in [1, 2] {
            let mut short = witness_harness((*theme).clone(), *color);
            let short_id = prepare_native_witness(&mut short);
            {
                let daemon = short
                    .app_mut()
                    .world
                    .daemons
                    .get_mut(&short_id)
                    .expect("running daemon");
                while daemon.tabs.len() > count {
                    let last = daemon.tabs.len() - 1;
                    daemon.close_tab(last);
                }
            }
            sync_snapshot(&mut short, &short_id);
            write_native_witness_source_map(&short, &root, &format!("{prefix}-{count}-live-tabs"));
            write_native_witness_frame(
                &short,
                &root,
                &format!("{prefix}-{count}-live-tabs"),
                &source,
                &short_id,
                &format!("prepared {count}-tab source; no native input"),
            );
        }

        let mut manual_initial = witness_harness_at_width((*theme).clone(), *color, 96);
        let manual_initial_id = prepare_native_witness(&mut manual_initial);
        write_native_witness_source_map(
            &manual_initial,
            &root,
            &format!("{prefix}-manual-narrow-initial"),
        );
        write_native_witness_frame(
            &manual_initial,
            &root,
            &format!("{prefix}-manual-narrow-initial"),
            &source,
            &manual_initial_id,
            "narrow fixture prepared; no native input",
        );

        let mut more_initial = witness_harness_at_width((*theme).clone(), *color, 96);
        let more_initial_id = prepare_more_native_witness(&mut more_initial);
        write_native_witness_source_map(
            &more_initial,
            &root,
            &format!("{prefix}-more-narrow-initial"),
        );
        write_native_witness_frame(
            &more_initial,
            &root,
            &format!("{prefix}-more-narrow-initial"),
            &source,
            &more_initial_id,
            "More fixture prepared; no native input",
        );
    }

    // Input sequences stay fixed and bounded. Unsupported preimage outcomes
    // remain in the receipts and do not stop later controls.
    for (theme_name, theme, color_name, color) in &configurations {
        let prefix = format!("{theme_name}-{color_name}");
        let mut h = witness_harness((*theme).clone(), *color);
        let instance_id = prepare_native_witness(&mut h);
        write_native_witness_source_map(&h, &root, &format!("{prefix}-keyboard"));
        let tab_to_reached = h.harness.tab_to(CAPSULE_TABS);
        let right_response = h.harness.key(KeyCode::Right);
        write_native_witness_frame(
            &h,
            &root,
            &format!("{prefix}-keyboard-attempt"),
            &source,
            &instance_id,
            &format!("tab_to_return={tab_to_reached} input=Right response={right_response:?}"),
        );

        let mut manual = witness_harness_at_width((*theme).clone(), *color, 96);
        let manual_id = prepare_native_witness(&mut manual);
        write_native_witness_source_map(&manual, &root, &format!("{prefix}-manual"));
        let strip = manual.harness.area_of(CAPSULE_TABS);
        let wheel_at = native_witness_tabs_point(&manual);
        let mut wheel_observations = Vec::new();
        for step in 0..12 {
            let response = manual.harness.wheel(Axis::H, 1, wheel_at.0, wheel_at.1);
            let title_visible = manual.harness.find("witness authored title");
            wheel_observations.push((step, response, title_visible));
        }
        write_native_witness_frame(
            &manual,
            &root,
            &format!("{prefix}-manual-window"),
            &source,
            &manual_id,
            &format!(
                "strip_owner={strip:?} wheel_at={wheel_at:?} wheel_attempts={wheel_observations:?}"
            ),
        );
        let tab_to_reached = manual.harness.tab_to(CAPSULE_TABS);
        let right_response = manual.harness.key(KeyCode::Right);
        write_native_witness_frame(
            &manual,
            &root,
            &format!("{prefix}-active-follow-attempt"),
            &source,
            &manual_id,
            &format!("tab_to_return={tab_to_reached} input=Right response={right_response:?}"),
        );

        let mut more = witness_harness_at_width((*theme).clone(), *color, 96);
        let more_id = prepare_more_native_witness(&mut more);
        write_native_witness_source_map(&more, &root, &format!("{prefix}-more"));
        let more_owner = more.harness.area_of_part(
            CAPSULE_TABS,
            PartRef::item(Part::OVERFLOW, ItemKey::index(2)),
        );
        let more_click_outcome = more_owner.as_ref().map(|area| {
            let point = (
                area.x.saturating_add(area.width / 2),
                area.y.saturating_add(area.height / 2),
            );
            let response = more.harness.click(point.0, point.1);
            (point, response)
        });
        let menu = CAPSULE_TABS.sub("overflow-menu");
        let list = menu.sub("items");
        let key = tab_key(&more, 11);
        let candidate_row_before = more
            .harness
            .area_of_part(list, PartRef::item(Part::ROW, key));
        let legacy_row_before = more
            .harness
            .area_of_part(list, PartRef::item(Part::ROW, ItemKey::index(11)));
        let row_before = candidate_row_before.or(legacy_row_before);
        let menu_area = more.harness.layer_area(menu);
        let row_click_at = row_before
            .map(|area| (area.x.saturating_add(1), area.y))
            .or_else(|| menu_area.map(|area| (area.x.saturating_add(1), area.y)))
            .unwrap_or_else(|| native_witness_frame_origin(&more));
        let label_before = more.harness.find("CAPSULE MORE ACCOUNT WITNESS");
        write_native_witness_frame(
            &more,
            &root,
            &format!("{prefix}-more-open"),
            &source,
            &more_id,
            &format!(
                "more_owner={more_owner:?} owner_center_click_outcome={more_click_outcome:?} no_click_owner_absent={} menu_area={menu_area:?} candidate_row={candidate_row_before:?} legacy_row={legacy_row_before:?} row_click_at={row_click_at:?} target_label={label_before:?}",
                more_owner.is_none()
            ),
        );

        {
            more.app_mut()
                .world
                .accounts
                .get_mut("acct-claude-personal")
                .expect("fixture account")
                .display_name = "Changed witness account".to_owned();
            let daemon = more
                .app_mut()
                .world
                .daemons
                .get_mut(&more_id)
                .expect("running daemon");
            daemon.tabs[11].custom_label = Some("Changed witness title".to_owned());
            for pane_id in daemon.tabs[11].leaves() {
                daemon
                    .pane_mut(pane_id)
                    .expect("tab leaf has a pane")
                    .proc
                    .state = AgentState::Working;
            }
        }
        sync_snapshot(&mut more, &more_id);
        let candidate_row_after = more
            .harness
            .area_of_part(list, PartRef::item(Part::ROW, key));
        let legacy_row_after = more
            .harness
            .area_of_part(list, PartRef::item(Part::ROW, ItemKey::index(11)));
        write_native_witness_frame(
            &more,
            &root,
            &format!("{prefix}-more-source-changed"),
            &source,
            &more_id,
            &format!(
                "candidate_row_before={candidate_row_before:?} legacy_row_before={legacy_row_before:?} candidate_row_after={candidate_row_after:?} legacy_row_after={legacy_row_after:?}"
            ),
        );
        // Reuse the same visible row coordinate in each source. The
        // source map records any index-to-key owner migration separately.
        let row_response = more.harness.click(row_click_at.0, row_click_at.1);
        write_native_witness_frame(
            &more,
            &root,
            &format!("{prefix}-more-selected"),
            &source,
            &more_id,
            &format!("input=Click@{row_click_at:?} response={row_response:?}"),
        );
    }
}

use std::cell::Cell;

use jackin_preview_app::CAPSULE_PANES;
use termrock::Id;

#[derive(Debug)]
struct StopCapsuleDuringUpdate {
    app: App,
    instance_id: String,
    stop_next_update: Cell<bool>,
    stopped_with_more_open: Cell<bool>,
    stopped_with_more_focused: Cell<bool>,
    stop_count: Cell<u8>,
}

impl termrock::App for StopCapsuleDuringUpdate {
    fn update(&mut self, cx: &mut termrock::Cx<'_>) -> termrock::Response<()> {
        if cx.update_cause() == termrock::UpdateCause::Event && self.stop_next_update.get() {
            self.stop_next_update.set(false);
            let menu = CAPSULE_TABS.sub("overflow-menu");
            self.stopped_with_more_open.set(cx.is_open(menu));
            self.stopped_with_more_focused
                .set(cx.focused_in_scope(termrock::ScopeId::new(menu)) == Some(menu.sub("items")));
            self.stop_count.set(self.stop_count.get().saturating_add(1));
            self.app
                .world
                .instance_mut(&self.instance_id)
                .expect("running instance")
                .status = InstanceStatus::CleanExited;
        }
        termrock::App::update(&mut self.app, cx)
    }

    fn draw(&self, ui: &mut termrock::Ui<'_>) {
        termrock::App::draw(&self.app, ui);
    }

    fn should_quit(&self) -> bool {
        termrock::App::should_quit(&self.app)
    }

    fn keymap(&self) -> &termrock::KeyMap {
        termrock::App::keymap(&self.app)
    }

    fn min_size(&self) -> termrock::Size {
        termrock::App::min_size(&self.app)
    }

    fn on_esc(&mut self, cx: &mut termrock::Cx<'_>) -> termrock::Response<()> {
        termrock::App::on_esc(&mut self.app, cx)
    }
}

impl StopCapsuleDuringUpdate {
    fn queue_stop(&self) {
        self.stop_next_update.set(true);
    }

    fn source(&self) -> &App {
        &self.app
    }
}

#[test]
fn full_fit_live_tabs_keep_native_cells_status_affixes_and_state_styles() {
    let mut h = capsule(220, 40);
    populate_tabs(&mut h, 12);

    let first_area = tab_area(&h, 0).expect("first live tab is registered");
    let second_area = tab_area(&h, 1).expect("second live tab is registered");
    let last_area = tab_area(&h, 11).expect("all full-fit live tabs are registered");
    let first_text = rect_text(&h, first_area);
    let tenth_text = rect_text(&h, tab_area(&h, 9).expect("tenth tab is registered"));
    let eleventh_text = rect_text(&h, tab_area(&h, 10).expect("eleventh tab is registered"));
    assert!(first_text.contains("1 tab-00"), "{first_text:?}");
    assert!(first_text.contains('●'), "{first_text:?}");
    assert!(rect_text(&h, second_area).contains("2 tab-01"));
    assert!(rect_text(&h, second_area).contains('▶'));
    assert!(rect_text(&h, tab_area(&h, 2).expect("third tab is registered")).contains('◆'));
    assert!(tenth_text.contains("0 tab-09"), "{tenth_text:?}");
    assert!(
        eleventh_text.trim_start().starts_with("tab-10"),
        "{eleventh_text:?}"
    );
    assert!(rect_text(&h, last_area).trim_start().starts_with("tab-11"));

    let active = h
        .harness
        .find("1 tab-00")
        .expect("active prefix and label are visible");
    let inactive = h
        .harness
        .find("2 tab-01")
        .expect("inactive prefix and label are visible");
    let colors = h.harness.runtime().theme().color;
    let active_prefix = h.harness.buffer().cell(active).expect("active prefix cell");
    let active_label = h
        .harness
        .buffer()
        .cell((active.0.saturating_add(2), active.1))
        .expect("active label cell");
    let inactive_prefix = h
        .harness
        .buffer()
        .cell(inactive)
        .expect("inactive prefix cell");
    let inactive_label = h
        .harness
        .buffer()
        .cell((inactive.0.saturating_add(2), inactive.1))
        .expect("inactive label cell");
    assert_eq!(active_prefix.fg, colors.fg[1]);
    assert_eq!(active_label.fg, colors.fg[0]);
    assert_eq!(active_label.bg, colors.surfaces[3]);
    assert!(active_label.modifier.contains(termrock::Modifier::BOLD));
    assert_eq!(inactive_prefix.fg, colors.fg[2]);
    assert_eq!(inactive_label.fg, colors.fg[1]);
    assert_eq!(inactive_label.bg, colors.surfaces[0]);
    assert!(!inactive_label.modifier.contains(termrock::Modifier::BOLD));

    let active_rule = h
        .harness
        .buffer()
        .cell((first_area.x, first_area.y.saturating_add(1)))
        .expect("active underline cell");
    let quiet_rule = h
        .harness
        .buffer()
        .cell((second_area.x, second_area.y.saturating_add(1)))
        .expect("inactive rule cell");
    assert_eq!(active_rule.fg, colors.accent);
    assert_eq!(quiet_rule.fg, termrock::Color::Rgb(0x3a, 0x3a, 0x3a));

    let (hover_x, hover_y) = tab_position(&h, 1);
    h.mouse(MouseKind::Move, hover_x, hover_y);
    let hover = h
        .harness
        .buffer()
        .cell((inactive.0.saturating_add(2), inactive.1))
        .expect("hovered label cell");
    assert_eq!(hover.fg, colors.fg[1]);
    assert_eq!(hover.bg, colors.surfaces[0]);
    assert!(!hover.modifier.contains(termrock::Modifier::BOLD));
}

#[test]
fn live_label_uses_the_current_account_display_name() {
    let mut h = capsule(120, 40);
    let instance_id = live_instance_id(&h);
    let account_name = h
        .app()
        .world
        .accounts
        .get("acct-claude-personal")
        .expect("fixture account is present")
        .display_name
        .clone();
    let leaves = h.app().world.daemons[&instance_id].tabs[0].leaves();
    let remaining_pane = leaves[0];
    let removed_panes = leaves.into_iter().skip(1).collect::<Vec<_>>();
    {
        let daemon = h
            .app_mut()
            .world
            .daemons
            .get_mut(&instance_id)
            .expect("running daemon");
        for removed_pane in removed_panes {
            assert!(!daemon.close_pane(removed_pane));
        }
        daemon.tabs[0].custom_label = None;
        let pane = daemon
            .pane_mut(remaining_pane)
            .expect("remaining tab leaf has a pane");
        pane.proc.agent = Some(Agent::ClaudeCode);
        pane.proc.account = Some("acct-claude-personal".to_owned());
    }
    sync_snapshot(&mut h, &instance_id);
    let tab = rect_text(&h, tab_area(&h, 0).expect("real tab owner is registered"));
    let expected = format!("{} ({account_name})", Agent::ClaudeCode.label());
    assert!(tab.contains(expected.as_str()), "{tab:?}");
}

#[test]
fn tabs_focus_arrows_and_native_more_layer_restore_focus() {
    let mut h = capsule(96, 40);
    populate_tabs(&mut h, 12);
    let instance_id = live_instance_id(&h);
    let active_leaves = h.app().world.daemons[&instance_id]
        .active_tab()
        .expect("running daemon has an active tab")
        .leaves();
    let menu = CAPSULE_TABS.sub("overflow-menu");
    let list = menu.sub("items");

    assert!(
        active_leaves.len() > 1,
        "fixture contains intermediate panes"
    );
    let first_pane = CAPSULE_PANES.item(ItemKey::num(active_leaves[0]));
    assert!(
        h.harness.tab_to(first_pane),
        "native ring reaches the first active pane; focus={:?}; frame={}",
        h.focus(),
        h.text(),
    );
    let mut focus_path = vec![h.focus()];
    for _ in 0..16 {
        if h.focus() == Some(CAPSULE_TABS) {
            break;
        }
        let _ = h.harness.key(KeyCode::Tab);
        focus_path.push(h.focus());
    }
    assert_eq!(
        h.focus(),
        Some(CAPSULE_TABS),
        "native Tab progression must reach Tabs through intermediate owners: {focus_path:?}"
    );
    assert!(
        focus_path.iter().any(|owner| {
            active_leaves
                .iter()
                .skip(1)
                .any(|pane_id| *owner == Some(CAPSULE_PANES.item(ItemKey::num(*pane_id))))
        }),
        "the native ring must advance through another active pane before Tabs: {focus_path:?}"
    );
    assert_eq!(h.focus(), Some(CAPSULE_TABS));
    h.key(KeyCode::Right);
    assert_eq!(h.app().world.daemons[&instance_id].active, 1);
    assert_eq!(
        h.focus(),
        Some(CAPSULE_TABS),
        "right activation retains Tabs focus; diagnostics={:?}; frame={}",
        h.harness.diagnostics(),
        h.text(),
    );
    h.key(KeyCode::Left);
    assert_eq!(h.app().world.daemons[&instance_id].active, 0);
    assert_eq!(
        h.focus(),
        Some(CAPSULE_TABS),
        "left activation retains Tabs focus; diagnostics={:?}; frame={}",
        h.harness.diagnostics(),
        h.text(),
    );

    h.key(KeyCode::Down);
    assert!(h.harness.is_open(menu), "Down opens native hidden tabs");
    assert_eq!(h.focus(), Some(list));
    let keyboard_close = h.harness.key(KeyCode::Esc);
    assert!(!h.harness.is_open(menu));
    assert_eq!(
        h.focus(),
        Some(CAPSULE_TABS),
        "keyboard More close restores its native owner; response={keyboard_close:?}; ring={:?}; layer={:?}; diagnostics={:?}; frame={}",
        h.harness.runtime().ring().entries(),
        h.harness.top_layer(),
        h.harness.diagnostics(),
        h.text(),
    );

    let more = h
        .harness
        .area_of_part(
            CAPSULE_TABS,
            PartRef::item(Part::OVERFLOW, ItemKey::index(2)),
        )
        .expect("native More owner is registered");
    h.click(more.x.saturating_add(more.width / 2), more.y);
    assert!(
        h.harness.is_open(menu),
        "pointer opens the native More menu"
    );
    assert_eq!(h.focus(), Some(list));
    h.key(KeyCode::Esc);
    assert!(!h.harness.is_open(menu));
    assert_eq!(
        h.focus(),
        Some(CAPSULE_TABS),
        "pointer-opened More restores Tabs focus; ring={:?}; layer={:?}; diagnostics={:?}; frame={}",
        h.harness.runtime().ring().entries(),
        h.harness.top_layer(),
        h.harness.diagnostics(),
        h.text(),
    );
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn manual_wheel_window_survives_draw_then_active_change_follows_into_view() {
    let mut h = capsule(96, 40);
    populate_tabs(&mut h, 12);
    let instance_id = live_instance_id(&h);
    let strip = h
        .harness
        .area_of(CAPSULE_TABS)
        .expect("native Tabs owner has a committed area");
    let at = (strip.x.saturating_add(2), strip.y);
    let mut hidden_active = false;
    for _ in 0..12 {
        if tab_area(&h, 0).is_none() {
            hidden_active = true;
            break;
        }
        let response = h.harness.wheel(Axis::H, 1, at.0, at.1);
        if !response.is_changed() {
            break;
        }
    }
    assert!(hidden_active, "manual wheel moves the window past tab zero");
    assert_eq!(h.app().world.daemons[&instance_id].active, 0);
    h.draw();
    assert!(
        tab_area(&h, 0).is_none(),
        "a fresh draw keeps manual position"
    );
    assert_eq!(h.app().world.daemons[&instance_id].active, 0);

    assert!(h.harness.tab_to(CAPSULE_TABS));
    h.key(KeyCode::Right);
    assert_eq!(h.app().world.daemons[&instance_id].active, 1);
    assert!(tab_area(&h, 1).is_some(), "active change reveals tab one");
    assert_eq!(h.harness.top_layer(), termrock::LayerId::PAGE);
    assert_eq!(h.harness.runtime().capture_owner(), None);
    assert_eq!(h.harness.diagnostics(), []);
}

#[test]
fn more_keeps_opening_label_and_status_while_current_live_tab_changes() {
    let mut h = capsule(96, 40);
    populate_tabs(&mut h, 20);
    let instance_id = live_instance_id(&h);
    let target_index = 19;
    let account_id = "acct-claude-personal";
    let old_account_name = h
        .app()
        .world
        .accounts
        .get(account_id)
        .expect("fixture account is present")
        .display_name
        .clone();
    {
        let daemon = h
            .app_mut()
            .world
            .daemons
            .get_mut(&instance_id)
            .expect("running daemon");
        daemon.tabs[18].custom_label =
            Some("hidden menu sizing witness label wider than the account row".to_owned());
        let leaves = daemon.tabs[target_index].leaves();
        daemon.tabs[target_index].custom_label = None;
        for pane_id in leaves {
            let pane = daemon.pane_mut(pane_id).expect("tab leaf has a pane");
            pane.proc.agent = Some(Agent::ClaudeCode);
            pane.proc.account = Some(account_id.to_owned());
            pane.proc.state = AgentState::Blocked;
        }
    }
    sync_snapshot(&mut h, &instance_id);
    assert!(
        tab_area(&h, target_index).is_none(),
        "target is initially hidden"
    );

    let more = h
        .harness
        .area_of_part(
            CAPSULE_TABS,
            PartRef::item(Part::OVERFLOW, ItemKey::index(2)),
        )
        .expect("native More owner is registered");
    h.click(more.x.saturating_add(more.width / 2), more.y);
    let menu = CAPSULE_TABS.sub("overflow-menu");
    let list = menu.sub("items");
    assert!(h.harness.is_open(menu));
    assert_eq!(h.focus(), Some(list));
    h.key(KeyCode::End);
    let key = tab_key(&h, target_index);
    let row = h.harness.area_of_part(list, PartRef::item(Part::ROW, key));
    assert!(
        row.is_some(),
        "hidden source row is not visible; frame={:?}; menu={:?}; list={:?}; focus={:?}; layer={:?}",
        h.text(),
        h.harness.area_of(menu),
        h.harness.area_of(list),
        h.focus(),
        h.harness.top_layer(),
    );
    let row = row.expect("the hidden source is a real More row");
    let frozen = rect_text(&h, row);
    assert!(frozen.contains(old_account_name.as_str()), "{frozen:?}");
    assert!(frozen.contains('●'), "the captured status affix is visible");

    {
        let account = h
            .app_mut()
            .world
            .accounts
            .get_mut(account_id)
            .expect("fixture account");
        account.display_name = "Changed fixture account".to_owned();
        let daemon = h
            .app_mut()
            .world
            .daemons
            .get_mut(&instance_id)
            .expect("running daemon");
        let leaves = daemon.tabs[target_index].leaves();
        daemon.tabs[target_index].custom_label = Some("Updated tab title".to_owned());
        for pane_id in leaves {
            daemon
                .pane_mut(pane_id)
                .expect("tab leaf has a pane")
                .proc
                .state = AgentState::Working;
        }
    }
    sync_snapshot(&mut h, &instance_id);
    let row = h
        .harness
        .area_of_part(list, PartRef::item(Part::ROW, key))
        .expect("frozen row keeps its native owner while key remains live");
    assert_eq!(rect_text(&h, row), frozen);
    assert!(!frozen.contains("Updated tab title"));
    assert!(!frozen.contains('▶'));

    let selection_response = h.harness.click(row.x.saturating_add(1), row.y);
    assert_eq!(h.app().world.daemons[&instance_id].active, target_index);
    assert!(!h.harness.is_open(menu));
    let selected_area = tab_area(&h, target_index);
    let tab_rows: Vec<_> = (0..20)
        .map(|index| (index, tab_key(&h, index), tab_area(&h, index)))
        .collect();
    let tab_regions: Vec<_> = h
        .harness
        .runtime()
        .registry()
        .regions()
        .iter()
        .filter(|region| region.owner == CAPSULE_TABS)
        .map(|region| (region.part, region.area, region.layer, region.kind))
        .collect();
    assert!(
        selected_area.is_some(),
        "selected target follows into the strip; response={selection_response:?}; active={}; target_area={selected_area:?}; tabs_area={:?}; focus={:?}; tabs_state={:?}; target_rows={tab_rows:?}; tab_regions={tab_regions:?}; frame={}",
        h.app().world.daemons[&instance_id].active,
        h.harness.area_of(CAPSULE_TABS),
        h.focus(),
        h.harness.runtime().state_of(CAPSULE_TABS),
        h.text(),
    );
    let selected = rect_text(
        &h,
        selected_area.expect("selected target follows into the strip"),
    );
    assert!(selected.contains("Updated tab title"), "{selected:?}");
    assert!(selected.contains('▶'), "{selected:?}");
    assert_eq!(
        h.app()
            .world
            .accounts
            .get(account_id)
            .expect("fixture account is present")
            .display_name,
        "Changed fixture account"
    );
    assert_eq!(h.harness.top_layer(), termrock::LayerId::PAGE);
    assert_eq!(h.harness.runtime().capture_owner(), None);
    assert!(
        h.harness.diagnostics().is_empty(),
        "{:?}",
        h.harness.diagnostics()
    );
}

#[test]
fn overflow_more_selects_a_hidden_source_key_and_removed_selection_fails_closed() {
    let mut h = capsule(96, 40);
    populate_tabs(&mut h, 12);
    let hidden_index = (0..12)
        .find(|index| tab_area(&h, *index).is_none())
        .expect("narrow strip has hidden real tabs");
    assert!(hidden_index > 0, "active tab remains visible");

    let more = h
        .harness
        .area_of_part(
            CAPSULE_TABS,
            PartRef::item(Part::OVERFLOW, ItemKey::index(2)),
        )
        .expect("native More owner is registered");
    h.click(more.x.saturating_add(more.width / 2), more.y);
    let menu = CAPSULE_TABS.sub("overflow-menu");
    let list = menu.sub("items");
    assert!(h.harness.is_open(menu));
    let target = tab_key(&h, hidden_index);
    let row = h
        .harness
        .area_of_part(list, PartRef::item(Part::ROW, target))
        .expect("hidden item is an actual native menu row");
    let expected = format!("tab-{hidden_index:02}");
    assert!(rect_text(&h, row).contains(expected.as_str()));
    h.click(row.x.saturating_add(1), row.y);
    assert_eq!(
        h.app().world.daemons[&live_instance_id(&h)].active,
        hidden_index
    );
    assert!(!h.harness.is_open(menu));
    assert_eq!(h.focus(), Some(CAPSULE_TABS));

    let mut stale = capsule(96, 40);
    populate_tabs(&mut stale, 12);
    let removed_index = (0..12)
        .find(|index| tab_area(&stale, *index).is_none())
        .expect("narrow strip has a hidden item");
    let more = stale
        .harness
        .area_of_part(
            CAPSULE_TABS,
            PartRef::item(Part::OVERFLOW, ItemKey::index(2)),
        )
        .expect("More owner is registered");
    stale.click(more.x.saturating_add(more.width / 2), more.y);
    let menu = CAPSULE_TABS.sub("overflow-menu");
    assert!(stale.harness.is_open(menu));

    let instance_id = live_instance_id(&stale);
    {
        let daemon = stale
            .app_mut()
            .world
            .daemons
            .get_mut(&instance_id)
            .expect("running daemon");
        daemon.close_tab(removed_index);
    }
    sync_snapshot(&mut stale, &instance_id);
    stale.key(KeyCode::Enter);
    assert!(!stale.harness.is_open(menu));
    assert_eq!(stale.app().world.daemons[&instance_id].active, 0);
    assert_eq!(stale.app().world.daemons[&instance_id].tabs.len(), 11);
}

#[test]
fn secondary_rename_re_resolves_after_reorder_and_split_changes() {
    let mut reordered = capsule(120, 40);
    populate_tabs(&mut reordered, 4);
    let instance_id = live_instance_id(&reordered);
    let target_anchor = tab_anchor(&reordered, 0);
    begin_context_rename(&mut reordered, 0);
    reordered.type_str("renamed after reorder");
    {
        let daemon = reordered
            .app_mut()
            .world
            .daemons
            .get_mut(&instance_id)
            .expect("running daemon");
        daemon.tabs.swap(0, 1);
    }
    sync_snapshot(&mut reordered, &instance_id);
    reordered.key(KeyCode::Enter);
    let daemon = &reordered.app().world.daemons[&instance_id];
    let renamed = daemon
        .tabs
        .iter()
        .find(|tab| tab.leaves().into_iter().min() == Some(target_anchor))
        .expect("bound tab survived reorder");
    assert_eq!(
        renamed.custom_label.as_deref(),
        Some("renamed after reorder")
    );
    assert_eq!(daemon.tabs[0].custom_label.as_deref(), Some("tab-01"));

    for new_first in [true, false] {
        let mut split = capsule(120, 40);
        populate_tabs(&mut split, 4);
        let instance_id = live_instance_id(&split);
        let target_anchor = tab_anchor(&split, 0);
        begin_context_rename(&mut split, 0);
        split.type_str(if new_first {
            "renamed after new-first split"
        } else {
            "renamed after new-second split"
        });
        let now_ms = split.app().world.now_ms();
        let new_pane = split
            .app_mut()
            .world
            .daemons
            .get_mut(&instance_id)
            .expect("running daemon")
            .split(SplitDir::Horizontal, new_first, None, None, now_ms, false)
            .expect("focused pane split succeeds");
        if !new_first {
            let closed_tab = split
                .app_mut()
                .world
                .daemons
                .get_mut(&instance_id)
                .expect("running daemon")
                .close_pane(new_pane);
            assert!(!closed_tab, "a nonanchor split leaf does not close its tab");
        }
        sync_snapshot(&mut split, &instance_id);
        split.key(KeyCode::Enter);
        let daemon = &split.app().world.daemons[&instance_id];
        let target = daemon
            .tabs
            .iter()
            .find(|tab| tab.leaves().into_iter().min() == Some(target_anchor))
            .expect("minimum live pane keeps the same tab binding");
        assert_eq!(
            target.custom_label.as_deref(),
            Some(if new_first {
                "renamed after new-first split"
            } else {
                "renamed after new-second split"
            })
        );
    }
}

#[test]
fn double_click_renames_and_stale_anchor_or_run_targets_do_not_rename_another_tab() {
    let mut doubled = capsule(120, 40);
    populate_tabs(&mut doubled, 4);
    let (x, y) = tab_position(&doubled, 1);
    let _ = doubled.harness.double_click(x, y);
    assert!(
        doubled.text().contains("Change tab title"),
        "{}",
        doubled.text()
    );
    assert_eq!(
        doubled.focus(),
        Some(Id::root("jackin.preview").sub("capsule-input")),
        "native rename opens with title input focused"
    );
    doubled.key(KeyCode::Enter);
    finish_title(&mut doubled, "double-click title");
    let instance_id = live_instance_id(&doubled);
    let actual_title = doubled.app().world.daemons[&instance_id].tabs[1]
        .custom_label
        .as_deref();
    assert_eq!(
        actual_title,
        Some("double-click title"),
        "title state after native double-click flow; focus={:?}; layer={:?}; diagnostics={:?}; frame={}",
        doubled.focus(),
        doubled.harness.top_layer(),
        doubled.harness.diagnostics(),
        doubled.text(),
    );
    assert_eq!(doubled.harness.top_layer(), termrock::LayerId::PAGE);
    assert_eq!(doubled.harness.runtime().capture_owner(), None);
    assert_eq!(doubled.harness.diagnostics(), []);

    let mut closed_anchor = capsule(120, 40);
    populate_tabs(&mut closed_anchor, 4);
    let instance_id = live_instance_id(&closed_anchor);
    let target_anchor = tab_anchor(&closed_anchor, 0);
    begin_context_rename(&mut closed_anchor, 0);
    closed_anchor.type_str("must not apply after anchor close");
    let closed_tab = closed_anchor
        .app_mut()
        .world
        .daemons
        .get_mut(&instance_id)
        .expect("running daemon")
        .close_pane(target_anchor);
    assert!(!closed_tab, "the split tab retains its nonanchor pane");
    sync_snapshot(&mut closed_anchor, &instance_id);
    closed_anchor.key(KeyCode::Enter);
    assert!(closed_anchor.text().contains("Tab changed · choose again"));
    assert_eq!(
        closed_anchor.app().world.daemons[&instance_id].tabs[0]
            .custom_label
            .as_deref(),
        Some("tab-00")
    );

    let mut changed_run = capsule(120, 40);
    populate_tabs(&mut changed_run, 4);
    let instance_id = live_instance_id(&changed_run);
    begin_context_rename(&mut changed_run, 0);
    changed_run.type_str("must not apply after run replacement");
    let next_run = RunId::new(
        changed_run
            .app()
            .world
            .instance(&instance_id)
            .expect("running instance")
            .run_id
            .value()
            .saturating_add(1),
    );
    changed_run
        .app_mut()
        .world
        .instance_mut(&instance_id)
        .expect("running instance")
        .run_id = next_run;
    sync_snapshot(&mut changed_run, &instance_id);
    changed_run.key(KeyCode::Enter);
    assert!(changed_run.text().contains("Tab changed · choose again"));
    assert_eq!(
        changed_run.app().world.daemons[&instance_id].tabs[0]
            .custom_label
            .as_deref(),
        Some("tab-00")
    );
}

#[test]
fn tab_removal_invalidates_pending_rename_and_close_remains_notice_only() {
    let mut removed = capsule(120, 40);
    populate_tabs(&mut removed, 4);
    let instance_id = live_instance_id(&removed);
    begin_context_rename(&mut removed, 0);
    removed.type_str("must not apply after removal");
    removed
        .app_mut()
        .world
        .daemons
        .get_mut(&instance_id)
        .expect("running daemon")
        .close_tab(0);
    sync_snapshot(&mut removed, &instance_id);
    removed.key(KeyCode::Enter);
    assert!(removed.text().contains("Tab changed · choose again"));
    assert_eq!(removed.app().world.daemons[&instance_id].tabs.len(), 3);
    assert!(
        removed.app().world.daemons[&instance_id]
            .tabs
            .iter()
            .all(|tab| tab.custom_label.as_deref() != Some("must not apply after removal"))
    );

    let mut close_notice = capsule(120, 40);
    populate_tabs(&mut close_notice, 4);
    let instance_id = live_instance_id(&close_notice);
    let original_len = close_notice.app().world.daemons[&instance_id].tabs.len();
    close_notice.ctrl('b');
    close_notice.key(KeyCode::Char('m'));
    close_notice.key(KeyCode::End);
    close_notice.key(KeyCode::Enter);
    assert!(close_notice.text().contains("Close tab?"));
    assert_eq!(
        close_notice.app().world.daemons[&instance_id].tabs.len(),
        original_len
    );
}

#[test]
fn malformed_live_daemon_fails_closed_without_falling_back_to_authored_rows() {
    let mut h = capsule(120, 40);
    populate_tabs(&mut h, 4);
    let instance_id = live_instance_id(&h);
    let old_key = tab_key(&h, 0);
    {
        let daemon = h
            .app_mut()
            .world
            .daemons
            .get_mut(&instance_id)
            .expect("running daemon");
        daemon.tabs[1].root = daemon.tabs[0].root.clone();
    }
    sync_snapshot(&mut h, &instance_id);
    assert!(
        tab_area(&h, 0).is_none(),
        "invalid live source has no fake hit owner"
    );
    assert!(
        h.harness
            .area_of_part(CAPSULE_TABS, PartRef::item(Part::TAB, old_key))
            .is_none()
    );
    assert!(!h.text().contains("Mix (3) ●"), "{}", h.text());
}

#[test]
fn authored_no_daemon_fixture_rows_remain_available() {
    let mut h = capsule(120, 40);
    let instance_id = live_instance_id(&h);
    h.app_mut().world.daemons.remove(&instance_id);
    h.draw();
    let text = h.text();
    assert_ne!(
        h.focus(),
        Some(CAPSULE_TABS),
        "the authored fallback does not retain live-source tab focus"
    );
    assert!(text.contains("Mix (3) ●"), "{text}");
    assert!(text.contains("Shell"), "{text}");
    assert!(
        h.harness
            .area_of_part(CAPSULE_TABS, PartRef::item(Part::TAB, ItemKey::index(0)))
            .is_some()
    );
    assert!(
        h.harness
            .area_of_part(CAPSULE_TABS, PartRef::of(Part::NEW))
            .is_none(),
        "the default Tabs fixture does not enable the New control"
    );
}

#[test]
fn empty_daemon_clears_authored_rows_without_creating_live_tab_owners() {
    let mut h = capsule(120, 40);
    let instance_id = live_instance_id(&h);
    h.app_mut()
        .world
        .daemons
        .get_mut(&instance_id)
        .expect("running daemon")
        .tabs
        .clear();
    sync_snapshot(&mut h, &instance_id);

    assert!(!h.text().contains("Mix (3) ●"), "{}", h.text());
    assert!(
        h.harness
            .area_of_part(CAPSULE_TABS, PartRef::item(Part::TAB, ItemKey::index(0)))
            .is_none(),
        "an empty daemon has no authored or synthetic live owner"
    );
    assert!(
        h.harness
            .area_of_part(CAPSULE_TABS, PartRef::of(Part::NEW))
            .is_none(),
        "the default Tabs fixture has no New hit owner"
    );
    assert!(
        h.harness.diagnostics().is_empty(),
        "empty-daemon diagnostics: {:?}",
        h.harness.diagnostics()
    );
    let strip = h
        .harness
        .area_of(CAPSULE_TABS)
        .expect("empty daemon still has a native tab strip");
    assert!(strip.height >= 2, "the old strip includes its quiet rule");
    let colors = h.harness.runtime().theme().color;
    for x in strip.x..strip.right() {
        let blank = h
            .harness
            .buffer()
            .cell((x, strip.y))
            .expect("label row cell is inside the frame");
        assert_eq!(blank.symbol(), " ", "empty daemon label row x={x}");
        assert_eq!(blank.fg, colors.fg[0], "empty daemon label fg x={x}");
        assert_eq!(blank.bg, colors.surfaces[0], "empty daemon label bg x={x}");
        assert!(
            !blank.modifier.contains(termrock::Modifier::BOLD),
            "empty daemon label modifier x={x}"
        );

        let rule = h
            .harness
            .buffer()
            .cell((x, strip.y.saturating_add(1)))
            .expect("rule row cell is inside the frame");
        assert_eq!(
            rule.fg,
            termrock::Color::Rgb(0x3a, 0x3a, 0x3a),
            "empty daemon quiet rule x={x}"
        );
        assert_eq!(rule.bg, colors.surfaces[0], "empty daemon rule bg x={x}");
    }
}

#[test]
fn removed_or_stopped_active_source_never_falls_back_during_pending_actions() {
    let mut removed_title = capsule(120, 40);
    populate_tabs(&mut removed_title, 4);
    let removed_id = live_instance_id(&removed_title);
    begin_context_rename(&mut removed_title, 0);
    removed_title.type_str("stale removed source title");
    removed_title
        .app_mut()
        .world
        .instances
        .retain(|instance| instance.id != removed_id);
    removed_title.draw();
    assert!(!removed_title.text().contains("Mix (3) ●"));
    removed_title.key(KeyCode::Enter);
    assert!(removed_title.text().contains("Tab changed · choose again"));
    assert_eq!(
        removed_title.app().world.daemons[&removed_id].tabs[0]
            .custom_label
            .as_deref(),
        Some("tab-00")
    );
    assert!(
        removed_title
            .harness
            .area_of_part(CAPSULE_TABS, PartRef::item(Part::TAB, ItemKey::index(0)))
            .is_none()
    );
    assert_eq!(removed_title.harness.diagnostics(), []);
    assert_eq!(removed_title.harness.top_layer(), termrock::LayerId::PAGE);
    assert_eq!(removed_title.harness.runtime().capture_owner(), None);

    let mut stopped_more = capsule(96, 40);
    populate_tabs(&mut stopped_more, 12);
    let stopped_id = live_instance_id(&stopped_more);
    assert!(stopped_more.harness.tab_to(CAPSULE_TABS));
    stopped_more.key(KeyCode::Down);
    let menu = CAPSULE_TABS.sub("overflow-menu");
    assert!(stopped_more.harness.is_open(menu));
    stopped_more
        .app_mut()
        .world
        .instance_mut(&stopped_id)
        .expect("running instance")
        .status = InstanceStatus::CleanExited;
    stopped_more.draw();
    assert!(!stopped_more.text().contains("Mix (3) ●"));
    let preview = Id::root("jackin.preview");
    let stopped_before_enter_regions: Vec<_> = stopped_more
        .harness
        .runtime()
        .registry()
        .regions()
        .iter()
        .filter(|region| region.layer == stopped_more.harness.top_layer())
        .map(|region| (region.owner, region.part, region.area, region.kind))
        .collect();
    let stopped_before_enter = (
        stopped_more.harness.top_layer(),
        stopped_more.focus(),
        stopped_more.harness.is_open(menu),
        stopped_more
            .harness
            .is_open(preview.sub("capsule-tab-menu")),
        stopped_more
            .harness
            .is_open(preview.sub("capsule-command-palette")),
        stopped_more.harness.is_open(preview.sub("capsule-help")),
        stopped_more.harness.runtime().ring().entries().to_vec(),
        stopped_before_enter_regions,
        stopped_more.text(),
    );
    assert!(!stopped_more.harness.is_open(menu));
    assert_eq!(stopped_more.harness.top_layer(), termrock::LayerId::PAGE);
    assert_eq!(stopped_more.harness.runtime().capture_owner(), None);
    assert!(
        stopped_more.harness.diagnostics().is_empty(),
        "stopped draw retires More without diagnostics before any fresh key; before={stopped_before_enter:?}; diagnostics={:?}",
        stopped_more.harness.diagnostics(),
    );
    let enter_response = stopped_more.harness.key(KeyCode::Enter);
    let stopped_after_enter_regions: Vec<_> = stopped_more
        .harness
        .runtime()
        .registry()
        .regions()
        .iter()
        .filter(|region| region.layer == stopped_more.harness.top_layer())
        .map(|region| (region.owner, region.part, region.area, region.kind))
        .collect();
    assert_eq!(stopped_more.app().world.daemons[&stopped_id].active, 0);
    assert_eq!(stopped_more.app().world.daemons[&stopped_id].tabs.len(), 12);
    assert!(
        stopped_more
            .harness
            .area_of_part(CAPSULE_TABS, PartRef::item(Part::TAB, ItemKey::index(0)))
            .is_none()
    );
    assert!(!stopped_more.harness.is_open(menu));
    let menu_bar = preview.sub("capsule-menu-bar");
    assert!(
        stopped_more.harness.is_open(menu_bar),
        "a fresh Enter after More already retired keeps native MenuBar behavior; before={stopped_before_enter:?}; response={enter_response:?}; focus={:?}; regions={stopped_after_enter_regions:?}; ring={:?}; diagnostics={:?}; frame={}",
        stopped_more.focus(),
        stopped_more.harness.runtime().ring().entries(),
        stopped_more.harness.diagnostics(),
        stopped_more.text(),
    );
    assert_ne!(stopped_more.harness.top_layer(), termrock::LayerId::PAGE);
    assert_eq!(stopped_more.harness.runtime().capture_owner(), None);
    assert_eq!(stopped_more.harness.diagnostics(), []);

    let mut stale_app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40);
    let stale_id = stale_app
        .world
        .instances
        .iter()
        .find(|instance| {
            instance.status == InstanceStatus::Running
                && stale_app.world.daemons.contains_key(instance.id.as_str())
        })
        .expect("CapsuleMulti has a running daemon")
        .id
        .clone();
    let now_ms = stale_app.world.now_ms();
    {
        let daemon = stale_app
            .world
            .daemons
            .get_mut(&stale_id)
            .expect("running daemon");
        while daemon.tabs.len() < 12 {
            daemon.new_tab(None, None, now_ms, false);
        }
        for index in 0..12 {
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
    let snapshot = stale_app.world.daemons[&stale_id].snapshot();
    stale_app
        .world
        .instance_mut(&stale_id)
        .expect("running instance")
        .daemon = snapshot;
    let mut stale_more = Harness::new(
        StopCapsuleDuringUpdate {
            app: stale_app,
            instance_id: stale_id.clone(),
            stop_next_update: Cell::new(false),
            stopped_with_more_open: Cell::new(false),
            stopped_with_more_focused: Cell::new(false),
            stop_count: Cell::new(0),
        },
        Theme::junie(),
        96,
        40,
    );
    assert!(stale_more.tab_to(CAPSULE_TABS));
    let _ = stale_more.key(KeyCode::Down);
    assert!(stale_more.is_open(menu));
    stale_more.app().queue_stop();
    let stale_before_regions: Vec<_> = stale_more
        .runtime()
        .registry()
        .regions()
        .iter()
        .filter(|region| region.layer == stale_more.top_layer())
        .map(|region| (region.owner, region.part, region.area, region.kind))
        .collect();
    let stale_before_enter = (
        stale_more.top_layer(),
        stale_more.focus(),
        stale_more.is_open(menu),
        stale_more.is_open(menu_bar),
        stale_more.is_open(preview.sub("capsule-tab-menu")),
        stale_more.is_open(preview.sub("capsule-command-palette")),
        stale_more.find("Change tab title"),
        stale_more.runtime().ring().entries().to_vec(),
        stale_before_regions,
        stale_more.text(),
    );
    let stale_enter_response = stale_more.key(KeyCode::Enter);
    let stale_after_regions: Vec<_> = stale_more
        .runtime()
        .registry()
        .regions()
        .iter()
        .filter(|region| region.layer == stale_more.top_layer())
        .map(|region| (region.owner, region.part, region.area, region.kind))
        .collect();
    let stale_after_enter = (
        stale_more.top_layer(),
        stale_more.focus(),
        stale_more.is_open(menu),
        stale_more.is_open(menu_bar),
        stale_more.is_open(preview.sub("capsule-tab-menu")),
        stale_more.is_open(preview.sub("capsule-command-palette")),
        stale_more.find("Change tab title"),
        stale_more.runtime().ring().entries().to_vec(),
        stale_after_regions,
        stale_more.text(),
    );
    assert!(stale_more.app().stopped_with_more_open.get());
    assert!(stale_more.app().stopped_with_more_focused.get());
    assert_eq!(stale_more.app().stop_count.get(), 1);
    assert_eq!(
        stale_more
            .app()
            .source()
            .world
            .instance(&stale_id)
            .unwrap()
            .status,
        InstanceStatus::CleanExited
    );
    assert_eq!(stale_more.app().source().world.daemons[&stale_id].active, 0);
    assert_eq!(
        stale_more.app().source().world.daemons[&stale_id]
            .tabs
            .len(),
        12
    );
    assert!(!stale_more.is_open(menu));
    assert_eq!(
        stale_more.top_layer(),
        termrock::LayerId::PAGE,
        "stale More Enter must not open another layer; before={stale_before_enter:?}; response={stale_enter_response:?}; after={stale_after_enter:?}; diagnostics={:?}",
        stale_more.diagnostics(),
    );
    assert_eq!(stale_more.runtime().capture_owner(), None);
    assert!(
        stale_more
            .area_of_part(CAPSULE_TABS, PartRef::item(Part::TAB, ItemKey::index(0)))
            .is_none(),
        "stale More choice cannot revive stopped Tabs; response={stale_enter_response:?}; focus={:?}; diagnostics={:?}; frame={}",
        stale_more.focus(),
        stale_more.diagnostics(),
        stale_more.text(),
    );
    assert_eq!(stale_more.diagnostics(), []);
}

#[test]
fn one_and_two_live_tabs_have_only_real_actionable_owners() {
    for count in [1, 2] {
        let mut h = capsule(120, 40);
        populate_tabs(&mut h, 3);
        let instance_id = live_instance_id(&h);
        {
            let daemon = h
                .app_mut()
                .world
                .daemons
                .get_mut(&instance_id)
                .expect("running daemon");
            while daemon.tabs.len() > count {
                let last = daemon.tabs.len() - 1;
                daemon.close_tab(last);
            }
        }
        sync_snapshot(&mut h, &instance_id);
        assert!(tab_area(&h, 0).is_some());
        if count == 2 {
            assert!(tab_area(&h, 1).is_some());
        }
        assert!(
            h.harness
                .area_of_part(CAPSULE_TABS, PartRef::item(Part::TAB, ItemKey::index(2)))
                .is_none()
        );
        assert!(!h.text().contains("Mix (3) ●"), "{}", h.text());
        assert!(!h.text().contains("docs ●"), "{}", h.text());
    }
}

#[test]
fn a_retired_pane_keeps_focus_on_the_new_active_pane() {
    let mut h = capsule(120, 40);
    populate_tabs(&mut h, 3);
    let instance_id = live_instance_id(&h);
    let (pane_one, pane_four) = {
        let daemon = &h.app().world.daemons[&instance_id];
        (daemon.tabs[0].focused, daemon.tabs[1].focused)
    };
    let pane_one_owner = CAPSULE_PANES.item(ItemKey::num(pane_one));
    let pane_four_owner = CAPSULE_PANES.item(ItemKey::num(pane_four));

    h.tab_to(pane_one_owner);
    assert_eq!(h.focus(), Some(pane_one_owner));
    assert_eq!(h.harness.top_layer(), termrock::LayerId::PAGE);

    h.app_mut()
        .world
        .daemons
        .get_mut(&instance_id)
        .expect("running daemon")
        .active = 1;
    sync_snapshot(&mut h, &instance_id);

    assert_eq!(h.app().world.daemons[&instance_id].active, 1);
    assert_eq!(
        h.focus(),
        Some(pane_four_owner),
        "retiring the focused pane returns focus to the active tab's pane; diagnostics={:?}; frame={}",
        h.harness.diagnostics(),
        h.text(),
    );
    assert!(
        h.harness.cursor().is_some(),
        "active pane cursor is visible"
    );
    assert!(
        h.harness
            .area_of(pane_four_owner)
            .is_some_and(|area| !area.is_empty())
    );
    assert_eq!(h.harness.top_layer(), termrock::LayerId::PAGE);
    assert_eq!(h.harness.runtime().capture_owner(), None);
    assert_eq!(h.harness.diagnostics(), []);
}

#[test]
fn source_change_preserves_valid_tabs_focus_and_navigation() {
    let mut h = capsule(120, 40);
    populate_tabs(&mut h, 3);
    let instance_id = live_instance_id(&h);

    h.tab_to(CAPSULE_TABS);
    assert_eq!(h.focus(), Some(CAPSULE_TABS));
    assert_eq!(h.harness.top_layer(), termrock::LayerId::PAGE);
    assert_eq!(h.harness.runtime().capture_owner(), None);

    h.app_mut()
        .world
        .daemons
        .get_mut(&instance_id)
        .expect("running daemon")
        .active = 1;
    sync_snapshot(&mut h, &instance_id);

    assert_eq!(h.app().world.daemons[&instance_id].active, 1);
    assert_eq!(
        h.focus(),
        Some(CAPSULE_TABS),
        "a source mutation leaves valid native Tabs focus in place; diagnostics={:?}; frame={}",
        h.harness.diagnostics(),
        h.text(),
    );
    assert_eq!(h.harness.cursor(), None);
    assert_eq!(h.harness.top_layer(), termrock::LayerId::PAGE);
    assert_eq!(h.harness.runtime().capture_owner(), None);
    assert_eq!(h.harness.diagnostics(), []);

    let _ = h.harness.key(KeyCode::Right);
    assert_eq!(h.app().world.daemons[&instance_id].active, 2);
    assert_eq!(h.focus(), Some(CAPSULE_TABS));
    assert_eq!(h.harness.cursor(), None);
    assert_eq!(h.harness.top_layer(), termrock::LayerId::PAGE);
    assert_eq!(h.harness.runtime().capture_owner(), None);
    assert_eq!(h.harness.diagnostics(), []);
}
