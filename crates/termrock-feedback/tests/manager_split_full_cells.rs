use ratatui_core::style::Color;
use termrock_core::id::Id;
use termrock_feedback::{
    ManagerSplit, SplitAlert, SplitInstance, SplitMode, SplitMount, SplitWorkspace,
    paint_manager_detail_drawer, paint_manager_split,
};
use termrock_test_support::Scene;
use termrock_theme::{CapabilityPalettes, ColorLevel, ManagerPresentationProfile, Theme};

const MENUS: [&str; 3] = ["File", "Go", "Help"];
const MOUNTS: [SplitMount<'static>; 2] = [
    SplitMount {
        source: "~/src/payments-platform",
        mode: "rw",
        isolation: "worktree",
    },
    SplitMount {
        source: "~/src/shared-libs",
        mode: "ro",
        isolation: "shared",
    },
];
const WORKSPACES: [SplitWorkspace<'static>; 4] = [
    SplitWorkspace {
        name: "payments-platform",
        expandable: true,
    },
    SplitWorkspace {
        name: "infra-control-plane",
        expandable: true,
    },
    SplitWorkspace {
        name: "release-automation",
        expandable: true,
    },
    SplitWorkspace {
        name: "customer-portal",
        expandable: true,
    },
];
const INSTANCES: [SplitInstance<'static>; 2] = [
    SplitInstance {
        id: "7f3a",
        role: "the-architect",
        agent: "Claude Code",
        status: "running",
        live: true,
        dirty: false,
    },
    SplitInstance {
        id: "c41e",
        role: "reviewer",
        agent: "Codex",
        status: "preserved · dirty",
        live: false,
        dirty: true,
    },
];
const ACCOUNT_LINES: [&str; 2] = [
    "Claude · Personal ★ · Claude · Work · Codex · Primary ★ ·",
    "Grok · Team ★ · OpenCode · Go subscription ★",
];

fn frame() -> ManagerSplit<'static> {
    ManagerSplit {
        brand_id: Id::root("jackin.preview").sub("manager-brand"),
        mode: SplitMode::Saved,
        menus: &MENUS,
        workspace_name: "payments-platform",
        workdir: "/workspace/payments-platform",
        cwd_display: "~/workspace/payments-platform",
        mounts: &MOUNTS,
        role_line: "the-architect ★ · allowed 3 of 46",
        env_line: "5 vars · 2 [op]",
        account_lines: &ACCOUNT_LINES,
        policy_line: "git pull enabled · keep awake on · dirty exit ask",
        workspaces: &WORKSPACES,
        instances: &INSTANCES,
        running: 2,
        live_here: 1,
        daemon_ago: "3 s ago",
        alert: None,
    }
}

fn render_frame(theme: Theme, color: ColorLevel, split: &ManagerSplit<'_>, drawer: bool) -> Scene {
    let mut scene = Scene::new("manager-profile-full-cell", theme, color, 120, 40);
    scene.draw(|ui, area| {
        if drawer {
            paint_manager_detail_drawer(ui, area, split);
        } else {
            paint_manager_split(ui, area, split);
        }
    });
    scene
}

fn render(theme: Theme, color: ColorLevel) -> Scene {
    let split = frame();
    render_frame(theme, color, &split, false)
}

fn full_cell_digest(scene: &Scene) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0100_0000_01b3;
    let mut hash = OFFSET;
    for (index, cell) in scene.buffer().content.iter().enumerate() {
        let record = format!("{index}|{cell:?}");
        for byte in record.bytes().chain([0xff]) {
            hash = (hash ^ u64::from(byte)).wrapping_mul(PRIME);
        }
    }
    hash
}

fn symbol(scene: &Scene, x: u16, y: u16) -> Option<&str> {
    scene.buffer().cell((x, y)).map(|cell| cell.symbol())
}

fn foreground(scene: &Scene, x: u16, y: u16) -> Option<Color> {
    scene.buffer().cell((x, y)).map(|cell| cell.fg)
}

#[test]
fn declared_junie_mono_shared_manager_matches_historical_full_cells() {
    let theme = Theme::junie();
    assert_eq!(
        theme.manager_presentation_profile(),
        ManagerPresentationProfile::Junie
    );
    let scene = render(theme, ColorLevel::Mono);
    assert_eq!(full_cell_digest(&scene), 0xb2d8_a7f1_c5e3_6435);
    assert_eq!(symbol(&scene, 76, 0), Some("W"));
    assert_eq!(symbol(&scene, 107, 0), Some("t"));
    assert_eq!(symbol(&scene, 110, 0), Some("2"));
    assert_eq!(symbol(&scene, 3, 4), Some("▎"));
    assert_eq!(symbol(&scene, 41, 36), Some("▎"));
}

#[test]
fn standard_junie_mono_and_paper_use_standard_profile_route() {
    let standard = Theme::junie()
        .builder()
        .manager_presentation_profile(ManagerPresentationProfile::Standard)
        .build();
    assert_eq!(
        full_cell_digest(&render(standard, ColorLevel::Mono)),
        0x6632_c1da_d11d_5c67,
        "default policy preserves current Junie Mono output"
    );
    let paper = Theme::paper();
    assert_eq!(
        paper.manager_presentation_profile(),
        ManagerPresentationProfile::Standard
    );
    for level in [ColorLevel::Mono, ColorLevel::TrueColor] {
        let default_paper = render(paper.clone(), level);
        let explicit_standard = render(
            paper
                .clone()
                .builder()
                .manager_presentation_profile(ManagerPresentationProfile::Standard)
                .build(),
            level,
        );
        assert_eq!(
            default_paper.buffer().content,
            explicit_standard.buffer().content,
            "default Paper declaration follows the Standard profile route at {level:?}"
        );
    }
}

#[test]
fn junie_profile_keeps_non_mono_full_cells_unchanged() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
    ] {
        let junie = render(Theme::junie(), level);
        let standard = render(
            Theme::junie()
                .builder()
                .manager_presentation_profile(ManagerPresentationProfile::Standard)
                .build(),
            level,
        );
        assert_eq!(
            junie.buffer().content,
            standard.buffer().content,
            "the profile only changes its declared Mono policy at {level:?}"
        );
    }
}

#[test]
fn custom_palette_does_not_select_junie_mono_layout_implicitly() {
    let tokens = Theme::junie().color;
    let from_tokens = Theme::from_tokens(tokens);
    let custom_palette = from_tokens
        .clone()
        .builder()
        .capability_palettes(CapabilityPalettes::new(tokens))
        .build();

    for theme in [from_tokens, custom_palette] {
        assert_eq!(
            theme.manager_presentation_profile(),
            ManagerPresentationProfile::Standard
        );
        let scene = render(theme, ColorLevel::Mono);
        assert_eq!(symbol(&scene, 73, 0), Some("W"));
        assert_eq!(symbol(&scene, 107, 0), Some("●"));
        assert_eq!(symbol(&scene, 42, 3), Some("─"));
        assert_eq!(symbol(&scene, 3, 4), Some(" "));
        assert_eq!(symbol(&scene, 41, 36), Some(" "));
        assert_eq!(symbol(&scene, 1, 39), Some("E"));
    }
}

#[test]
fn junie_mono_keeps_empty_expanded_degraded_and_drawer_layouts() {
    let theme = Theme::junie();

    let mut empty = frame();
    empty.mode = SplitMode::Empty;
    let empty_scene = render_frame(theme.clone(), ColorLevel::Mono, &empty, false);
    assert_eq!(symbol(&empty_scene, 72, 0), Some("W"));
    assert_eq!(symbol(&empty_scene, 42, 3), Some("─"));
    assert_eq!(symbol(&empty_scene, 1, 39), Some("E"));
    assert_eq!(foreground(&empty_scene, 72, 0), Some(Color::White));

    let mut expanded = frame();
    expanded.mode = SplitMode::Expanded;
    let expanded_scene = render_frame(theme.clone(), ColorLevel::Mono, &expanded, false);
    assert_eq!(symbol(&expanded_scene, 53, 0), Some("W"));
    assert_eq!(symbol(&expanded_scene, 107, 0), Some("●"));
    assert_eq!(symbol(&expanded_scene, 42, 3), Some("─"));
    assert_eq!(symbol(&expanded_scene, 1, 39), Some("E"));
    assert_eq!(foreground(&expanded_scene, 53, 0), Some(Color::White));

    let mut degraded = frame();
    degraded.alert = Some(SplitAlert {
        index_error: Some("index unavailable"),
        daemon_stale: true,
    });
    let degraded_scene = render_frame(theme.clone(), ColorLevel::Mono, &degraded, false);
    assert_eq!(symbol(&degraded_scene, 41, 0), Some("W"));
    assert_eq!(symbol(&degraded_scene, 42, 3), Some("─"));
    assert_eq!(symbol(&degraded_scene, 1, 39), Some("E"));
    assert_eq!(foreground(&degraded_scene, 41, 0), Some(Color::White));

    let drawer = frame();
    let drawer_scene = render_frame(theme, ColorLevel::Mono, &drawer, true);
    assert_eq!(symbol(&drawer_scene, 73, 0), Some("W"));
    assert_eq!(symbol(&drawer_scene, 107, 0), Some("●"));
    assert_eq!(symbol(&drawer_scene, 42, 3), Some("─"));
    assert_eq!(symbol(&drawer_scene, 41, 2), Some("▎"));
    assert_eq!(symbol(&drawer_scene, 42, 2), Some("C"));
    assert_eq!(symbol(&drawer_scene, 1, 39), Some("↑"));
    assert_eq!(foreground(&drawer_scene, 42, 2), Some(Color::White));
}

#[test]
fn junie_mono_saved_presentation_keeps_frame_data_dynamic() {
    let original = frame();
    let mut changed = frame();
    changed.running = 7;
    changed.workspace_name = "payments-platform-next";
    changed.workdir = "/workspace/payments-platform-next";

    let original_scene = render_frame(Theme::junie(), ColorLevel::Mono, &original, false);
    let changed_scene = render_frame(Theme::junie(), ColorLevel::Mono, &changed, false);

    assert_eq!(symbol(&original_scene, 110, 0), Some("2"));
    assert_eq!(symbol(&changed_scene, 110, 0), Some("7"));
    assert_eq!(symbol(&original_scene, 79, 2), Some(" "));
    assert_eq!(symbol(&changed_scene, 79, 2), Some("-"));
    assert_eq!(symbol(&original_scene, 84, 4), Some(" "));
    assert_eq!(symbol(&changed_scene, 84, 4), Some("-"));
}
