use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use termrock::Color as RColor;
use termrock::Input as RtInput;
use termrock::runtime::stub::{Stub, deliver};
use termrock::{
    App, Brand, Buffer, Button, Checkbox, Constraints, Cx, Empty, EmptyState, FgStep, Field,
    FieldError, Flow, Focusability, Id, KeyCode, KeyModifiers, Maximized, Modifier, MouseKind,
    Panel, PanelKind, Part, PartRef, Position, PropsAction, PropsList, PropsRow, PropsState,
    RadioGroup, RadioGroupAction, RadioGroupState, Rect, ReferenceState, ReferenceTarget, Response,
    Role, Runtime, Size, SplitAxis, SplitModel, SplitPane, SplitPaneState, StateFlags, Status,
    TextAction, TextArea, TextAreaState, TextInput, TextInputState, TextViewport, Theme, Toggle,
    Ui, Variant, ViewportLine, ViewportState,
};
use termrock_test_support::Harness;
use tuiscotti::render::frame_from_screen;
use tuiscotti::tui::{MouseButton, Wheel};
use tuiscotti::{Cell as TCell, Color as TColor, Frame, Mods, Provenance, Rgb, UnderlineStyle};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, HOLLA, JACKIN, SHOWCASE, TABLEPRO};

const BOOT: &str = "Junie Design system";

const PORT_OF: &str = "91aba5102f5b4dabaff506894f8b19166afb27a6";

fn s2_case(slug: &str, args: &'static [&'static str], cols: u16, rows: u16) -> Case {
    Case::dynamic(
        format!("journeys/showcase/s2/{slug}"),
        SHOWCASE,
        args,
        cols,
        rows,
        Color::Truecolor,
        BOOT,
    )
}

fn s2_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

fn write_provenance(dir: &Path, case: &Case) {
    let subject =
        support::try_resolve_bin(case.bin).unwrap_or_else(|e| panic!("resolve {}: {e}", case.bin));
    let modified = subject
        .mtime_unix
        .map_or_else(|| "unknown".to_string(), |m| m.to_string());
    let argv = support::argv_for(case);
    let body = format!(
        "profile: tuiscotti-default\ntransport: pty\nport_of: {PORT_OF}\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join("provenance.txt"), &body)
        .unwrap_or_else(|e| panic!("write provenance.txt: {e}"));
    eprintln!("s2 provenance: {body}");
}

fn checkpoint(s: &mut Session, dir: &Path, name: &str) {
    let obs = s
        .inner
        .observe_now()
        .unwrap_or_else(|e| panic!("checkpoint `{name}` sample failed: {e:#}"));
    let text = support::screen_text(&obs.screen);
    let body = format!(
        "# checkpoint {name} {}x{}\n{text}\n",
        obs.screen.cols(),
        obs.screen.rows()
    );
    let path = dir.join(format!("{name}.txt"));
    std::fs::write(&path, body).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    eprintln!("checkpoint {name} ({} bytes visible text)", text.len());
}

fn case_timeout(case: &Case) -> Duration {
    Duration::from_millis(case.timeout_ms)
}

fn live_text(s: &mut Session) -> String {
    let obs = s
        .inner
        .observe_now()
        .unwrap_or_else(|e| panic!("live sample failed: {e:#}"));
    support::screen_text(&obs.screen)
}

fn live_frame(s: &mut Session) -> Frame {
    let screen = s
        .inner
        .snapshot()
        .unwrap_or_else(|e| panic!("live frame sample failed: {e:#}"));
    frame_from_screen(&screen, "default")
}

fn find_pos(s: &mut Session, needle: &str, timeout: Duration) -> (u16, u16) {
    let obs = support::wait_screen(
        s,
        timeout,
        &format!("`{needle}` never appeared"),
        |screen| support::screen_find(screen, needle).is_some(),
    );
    support::screen_find(&obs.screen, needle).expect("wait passed with the needle on screen")
}

fn hover_over(s: &mut Session, needle: &str, timeout: Duration) -> (u16, u16) {
    let (row, col) = find_pos(s, needle, timeout);
    Input::move_to(col, row).send(s);
    std::thread::sleep(Duration::from_millis(120));
    (row, col)
}

fn click_at(s: &mut Session, needle: &str, timeout: Duration) -> (u16, u16) {
    let (row, col) = find_pos(s, needle, timeout);
    Input::down(MouseButton::Left, col, row).send(s);
    Input::up(MouseButton::Left, col, row).send(s);
    std::thread::sleep(Duration::from_millis(120));
    (row, col)
}

fn wheel_at(s: &mut Session, col: u16, row: u16, dir: Wheel, notches: u32) {
    for _ in 0..notches {
        Input::wheel(dir, col, row).send(s);
        std::thread::sleep(Duration::from_millis(120));
    }
}

fn type_text(s: &mut Session, text: &str) {
    s.type_text(text).expect("type_text");
    std::thread::sleep(Duration::from_millis(120));
}

fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line holds both {needle:?} and {also:?}\n{text}"
    );
}

fn assert_line_lacks(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        !text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: a line holds both {needle:?} and {also:?}\n{text}"
    );
}

fn sentinel_buf(cols: u16, rows: u16) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, cols, rows));
    for y in 0..rows {
        for x in 0..cols {
            if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                cell.set_symbol("·");
            }
        }
    }
    buf
}

fn assert_surround_intact(buf: &Buffer, area: Rect, what: &str) {
    let (w, h) = (buf.area.width, buf.area.height);
    for y in 0..h {
        for x in 0..w {
            if x >= area.x && x < area.right() && y >= area.y && y < area.bottom() {
                continue;
            }
            assert_eq!(
                buf[(x, y)].symbol(),
                "·",
                "{what}: surround cell ({x},{y}) touched"
            );
        }
    }
}

fn buf_row_text(buf: &Buffer, y: u16) -> String {
    (0..buf.area.width)
        .map(|x| buf[(x, y)].symbol().to_owned())
        .collect()
}

fn buf_is_bold(buf: &Buffer, x: u16, y: u16) -> bool {
    buf[(x, y)].modifier.contains(Modifier::BOLD)
}

fn buf_is_underlined(buf: &Buffer, x: u16, y: u16) -> bool {
    buf[(x, y)].modifier.contains(Modifier::UNDERLINED)
}

fn render_isolated(
    cols: u16,
    rows: u16,
    ticks: u64,
    target: Option<(Id, ReferenceState)>,
    draw: impl FnOnce(&mut Ui<'_>, Rect),
) -> Buffer {
    let mut runtime = Runtime::new(Stub::default(), Theme::junie());
    let _ = runtime.initialize();
    for _ in 0..ticks {
        let _ = deliver(&mut runtime, RtInput::Tick);
    }
    let view = Rect::new(0, 0, cols, rows);
    let mut buf = sentinel_buf(cols, rows);
    runtime
        .draw_scene(view, &mut buf, |ui, area| {
            if let Some((id, state)) = target {
                ui.reference(Some(ReferenceTarget::new(id, state)), |ui| draw(ui, area));
            } else {
                draw(ui, area);
            }
        })
        .commit_presented();
    buf
}

fn measure_size(draw: impl Fn(&Ui<'_>, Constraints) -> Size) -> Size {
    let mut runtime = Runtime::new(Stub::default(), Theme::junie());
    let mut buffer = Buffer::empty(Rect::new(0, 0, 40, 8));
    let size = Cell::new(Size {
        min: (0, 0),
        preferred: (0, 0),
    });
    runtime
        .draw_scene(Rect::new(0, 0, 40, 8), &mut buffer, |ui, _area| {
            size.set(draw(ui, Constraints::loose(u16::MAX, u16::MAX)));
        })
        .commit_presented();
    size.get()
}

fn convert_color(c: RColor) -> TColor {
    match c {
        RColor::Reset => TColor::Default,
        RColor::Black => TColor::Indexed(0),
        RColor::Red => TColor::Indexed(1),
        RColor::Green => TColor::Indexed(2),
        RColor::Yellow => TColor::Indexed(3),
        RColor::Blue => TColor::Indexed(4),
        RColor::Magenta => TColor::Indexed(5),
        RColor::Cyan => TColor::Indexed(6),
        RColor::Gray => TColor::Indexed(7),
        RColor::DarkGray => TColor::Indexed(8),
        RColor::LightRed => TColor::Indexed(9),
        RColor::LightGreen => TColor::Indexed(10),
        RColor::LightYellow => TColor::Indexed(11),
        RColor::LightBlue => TColor::Indexed(12),
        RColor::LightMagenta => TColor::Indexed(13),
        RColor::LightCyan => TColor::Indexed(14),
        RColor::White => TColor::Indexed(15),
        RColor::Indexed(i) => TColor::Indexed(i),
        RColor::Rgb(r, g, b) => TColor::Rgb(Rgb::new(r, g, b)),
    }
}

fn convert_mods(m: Modifier) -> Mods {
    Mods {
        hidden: m.contains(Modifier::HIDDEN),
        blink: m.intersects(Modifier::SLOW_BLINK | Modifier::RAPID_BLINK),
        bold: m.contains(Modifier::BOLD),
        dim: m.contains(Modifier::DIM),
        italic: m.contains(Modifier::ITALIC),
        underline: m.contains(Modifier::UNDERLINED),
        underline_style: if m.contains(Modifier::UNDERLINED) {
            UnderlineStyle::Single
        } else {
            UnderlineStyle::None
        },
        strikethrough: m.contains(Modifier::CROSSED_OUT),
        reverse: m.contains(Modifier::REVERSED),
    }
}

fn capture_buffer(buf: &Buffer) -> Frame {
    let cols = buf.area.width;
    let rows = buf.area.height;
    let mut frame = Frame::blank(
        cols,
        rows,
        Provenance::now("default", "s2-isolated", vec![]),
    );
    for y in 0..rows {
        let mut x = 0u16;
        while x < cols {
            let Some(rc) = buf.cell((x, y)) else {
                x += 1;
                continue;
            };
            if rc.symbol().is_empty() {
                let mut cont = TCell::blank(x, y);
                cont.fg = convert_color(rc.fg);
                cont.bg = convert_color(rc.bg);
                cont.mods = convert_mods(rc.modifier);
                cont.underline_color = convert_color(rc.underline_color);
                cont.width = 0;
                cont.continuation = true;
                cont.symbol = String::new();
                frame.set(cont);
                x += 1;
                continue;
            }
            let symbol = rc.symbol().to_string();
            let width = u8::try_from(unicode_width::UnicodeWidthStr::width(symbol.as_str()).min(2))
                .unwrap_or(u8::MAX)
                .max(1);
            let lead = TCell {
                x,
                y,
                symbol,
                width,
                continuation: false,
                fg: convert_color(rc.fg),
                bg: convert_color(rc.bg),
                mods: convert_mods(rc.modifier),
                underline_color: convert_color(rc.underline_color),
            };
            if width == 2 && x + 1 >= cols {
                let mut narrow = lead;
                narrow.width = 1;
                frame.set(narrow);
                x += 1;
                continue;
            }
            frame.set(lead);
            if width == 2 && x + 1 < cols {
                let mut cont = TCell::blank(x + 1, y);
                cont.fg = convert_color(rc.fg);
                cont.bg = convert_color(rc.bg);
                cont.mods = convert_mods(rc.modifier);
                cont.underline_color = convert_color(rc.underline_color);
                cont.width = 0;
                cont.continuation = true;
                cont.symbol = String::new();
                frame.set(cont);
                x += 2;
            } else {
                x += 1;
            }
        }
    }
    frame
}

fn capture_isolated(dir: &Path, name: &str, buf: &Buffer) -> Frame {
    let frame = capture_buffer(buf);
    std::fs::write(
        dir.join(format!("isolated-{name}.json")),
        frame.to_json_pretty(),
    )
    .unwrap_or_else(|e| panic!("write isolated-{name}.json: {e}"));
    eprintln!("isolated {name}: digest {:016x}", frame.digest());
    frame
}

fn resolve_rgb(c: TColor, what: &str) -> Rgb {
    match c {
        TColor::Rgb(rgb) => rgb,
        TColor::Indexed(n) => Rgb::from_indexed(n),
        TColor::Default => panic!("{what}: cell color is Default, cannot resolve"),
    }
}

fn lum(c: TColor, what: &str) -> u32 {
    let rgb = resolve_rgb(c, what);
    u32::from(rgb.r) * 2126 + u32::from(rgb.g) * 7152 + u32::from(rgb.b) * 722
}

fn theme_rgb(c: RColor) -> Rgb {
    match c {
        RColor::Rgb(r, g, b) => Rgb { r, g, b },
        other => panic!("theme color is not RGB: {other:?}"),
    }
}

fn char_col(line: &str, needle: &str) -> u16 {
    let byte = line
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not in {line:?}"));
    line[..byte].chars().count() as u16
}

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

const S2_BUSY: Id = Id::root("s2.busy");
const S2_BUSY_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 20,
    height: 1,
};

struct BusyApp {
    fired: Rc<Cell<u32>>,
    status: Status,
    toggle: Option<bool>,
}

impl BusyApp {
    fn button(&self) -> Button<'static> {
        let mut b = Button::new(S2_BUSY, "Run long job").status(self.status);
        if let Some(on) = self.toggle {
            b = b.variant(Variant::TOGGLE).checked(on);
        }
        b
    }
}

impl App for BusyApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let fired = Rc::clone(&self.fired);
        self.button()
            .update(cx)
            .on_activated(|| fired.set(fired.get() + 1))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.button().draw(ui, S2_BUSY_AREA);
    }
}

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_busy_frames() {
    let dir = s2_dir("s2_busy_frames");
    let t = Theme::junie();

    let mut first: Option<Frame> = None;
    for (tick, want) in [(0u64, "⠋"), (1u64, "⠙")] {
        assert_eq!(SPINNER[tick as usize], want, "row pins SPINNER[tick]");
        let buf = render_isolated(
            24,
            3,
            tick,
            Some((S2_BUSY, ReferenceState::FOCUSED)),
            |ui, _| {
                Button::new(S2_BUSY, "Run long job")
                    .status(Status::Busy)
                    .draw(ui, Rect::new(4, 1, 20, 1));
            },
        );

        assert_eq!(buf[(5, 1)].symbol(), want, "V1: tick {tick} marker");

        assert_eq!(
            buf[(7, 1)].fg,
            t.color.fg[FgStep::Secondary.index()],
            "V2: label tone"
        );
        assert!(!buf_is_bold(&buf, 7, 1), "V2: label non-bold");
        assert_eq!(buf[(5, 1)].fg, t.color.accent, "V2: spinner accent");

        let fired = Rc::new(Cell::new(0));
        let mut app = Harness::new(
            BusyApp {
                fired: Rc::clone(&fired),
                status: Status::Busy,
                toggle: None,
            },
            Theme::junie(),
            40,
            8,
        );
        assert!(app.tab_to(S2_BUSY), "S1: busy stays reachable");

        let _ = app.key(KeyCode::Enter);
        assert_eq!(fired.get(), 0, "S2/A1: Enter while busy never fires");
        let _ = app.key(KeyCode::Char(' '));
        assert_eq!(fired.get(), 0, "S2/A1: Space while busy never fires");
        let _ = app.click(2, 0);
        assert_eq!(fired.get(), 0, "A2: click while busy never fires");

        let row: String = (0..20).map(|x| buf[(x, 1)].symbol().to_owned()).collect();
        assert!(
            !row.contains('●') && !row.contains('○'),
            "N2: no toggle marker slot in {row:?}"
        );
        let w = measure_size(|ui, c| {
            Button::new(S2_BUSY, "Run long job")
                .status(Status::Busy)
                .measure(ui, c)
        });
        assert_eq!(w.preferred.0, 12 + 2 + 2, "N2: one marker slot wide");
        assert_surround_intact(&buf, Rect::new(4, 1, 16, 1), "busy tick {tick}");
        let frame = capture_isolated(&dir, &format!("busy-tick-{tick}"), &buf);
        if let Some(prev) = first.as_ref() {
            assert_ne!(frame.digest(), prev.digest(), "N1: tick frames differ");
        } else {
            first = Some(frame);
        }
    }

    let case = s2_case("busy_frames", &["--page", "buttons"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    assert!(
        live_text(&mut s).contains("Start long job"),
        "boot has the busy demo"
    );
    click_at(&mut s, "Start long job", case_timeout(&case));

    let mut seen = std::collections::BTreeSet::new();
    let deadline = std::time::Instant::now() + case_timeout(&case);
    while std::time::Instant::now() < deadline && seen.len() < 2 {
        let snap = s
            .inner
            .snapshot()
            .unwrap_or_else(|e| panic!("busy sample failed: {e:#}"));
        let frame = frame_from_screen(&snap, "default");
        if let Some((row, col)) = support::screen_find(&snap, "Start long job") {
            for c in col.saturating_sub(3)..col {
                if let Some(cell) = frame.get(c, row)
                    && SPINNER.contains(&cell.symbol.as_str())
                {
                    seen.insert(cell.symbol.clone());
                }
            }
        }
        if support::screen_text(&snap).contains("Long job finished") {
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(seen.len() >= 2, "V1/N1 live: two spinner frames {seen:?}");

    waits::wait_state(&mut s, "Long job finished", "busy window closes");
    checkpoint(&mut s, &dir, "finished");
    let done = live_text(&mut s);
    assert!(done.contains("Start long job"), "label restored after busy");
    eprintln!("s2 busy_frames: isolated ticks 0/1 + live cycle {seen:?}");
}

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_busy_checked_geom() {
    let dir = s2_dir("s2_busy_checked_geom");
    let t = Theme::junie();

    let buf = render_isolated(
        30,
        3,
        0,
        Some((S2_BUSY, ReferenceState::FOCUSED)),
        |ui, _| {
            Button::new(S2_BUSY, "Verbose")
                .variant(Variant::TOGGLE)
                .checked(true)
                .status(Status::Busy)
                .draw(ui, Rect::new(4, 1, 22, 1));
        },
    );

    assert_eq!(buf[(5, 1)].symbol(), "⠋", "V1: spinner wins the marker");
    assert_eq!(buf[(5, 1)].fg, t.color.accent, "V1: spinner accent");

    let w = measure_size(|ui, c| {
        Button::new(S2_BUSY, "Verbose")
            .variant(Variant::TOGGLE)
            .checked(true)
            .status(Status::Busy)
            .measure(ui, c)
    });
    assert_eq!(w.preferred.0, 7 + 2 + 2, "V2/N2: one marker slot");

    let fired = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        BusyApp {
            fired: Rc::clone(&fired),
            status: Status::Busy,
            toggle: Some(true),
        },
        Theme::junie(),
        40,
        8,
    );
    assert!(app.tab_to(S2_BUSY), "S1: busy toggle stays reachable");

    let _ = app.key(KeyCode::Enter);
    assert_eq!(fired.get(), 0, "S2/A1: Enter while busy never fires");
    let idle = render_isolated(30, 3, 0, None, |ui, _| {
        Button::new(S2_BUSY, "Verbose")
            .variant(Variant::TOGGLE)
            .checked(true)
            .draw(ui, Rect::new(4, 1, 22, 1));
    });
    assert_eq!(idle[(5, 1)].symbol(), "●", "S1/A1: toggle preserved");

    let row = buf_row_text(&buf, 1);
    assert!(
        !row.contains('●') && !row.contains('○'),
        "N1: no marker in row {row:?}"
    );
    assert_surround_intact(&buf, Rect::new(4, 1, 11, 1), "busy+checked");
    capture_isolated(&dir, "busy-checked", &buf);

    let case = s2_case("busy_checked_geom", &["--page", "buttons"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    let boot = live_text(&mut s);
    let boot_row = boot
        .lines()
        .find(|l| l.contains("Start long job"))
        .expect("boot busy row")
        .to_string();
    click_at(&mut s, "Start long job", case_timeout(&case));
    let busy_text = support::wait_screen(
        &mut s,
        case_timeout(&case),
        "busy marker never appeared",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("Start long job") && SPINNER.iter().any(|g| l.contains(g)))
        },
    );
    let busy_screen = support::screen_text(&busy_text.screen);
    let busy_row = busy_screen
        .lines()
        .find(|l| l.contains("Start long job"))
        .expect("busy row")
        .to_string();

    assert_eq!(
        boot_row.chars().count(),
        busy_row.chars().count(),
        "V2 live: row width stable"
    );

    assert!(
        !busy_row.contains('●') && !busy_row.contains('○'),
        "N1 live: {busy_row:?}"
    );
    assert!(
        busy_screen.contains('●'),
        "N1 presence: ● renders elsewhere"
    );
    waits::wait_state(&mut s, "Long job finished", "busy window closes");
    checkpoint(&mut s, &dir, "finished");
    eprintln!("s2 busy_checked_geom: isolated + live width-stable");
}

const S2_BRAND: Id = Id::root("s2.brand");
const S2_BRAND_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 8,
    height: 1,
};

struct BrandApp {
    fired: Rc<Cell<u32>>,
    clickable: bool,
}

impl App for BrandApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let fired = Rc::clone(&self.fired);
        Brand::new(S2_BRAND, "holla❯")
            .clickable(self.clickable)
            .update(cx)
            .on_activated(|| fired.set(fired.get() + 1))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Brand::new(S2_BRAND, "holla❯")
            .clickable(self.clickable)
            .draw(ui, S2_BRAND_AREA);
    }
}

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_brand_hover() {
    let dir = s2_dir("s2_brand_hover");
    let t = Theme::junie();

    let w = measure_size(|ui, c| {
        Brand::new(S2_BRAND, "holla❯")
            .clickable(true)
            .measure(ui, c)
    });
    assert_eq!(w.preferred.0, 8, "V1: 6 text + 2 pad");

    let idle_buf = render_isolated(16, 3, 0, None, |ui, _| {
        Brand::new(S2_BRAND, "holla❯")
            .clickable(true)
            .draw(ui, Rect::new(4, 1, 8, 1));
    });
    assert_eq!(idle_buf[(5, 1)].bg, t.color.accent, "V1: idle fill accent");
    assert_eq!(idle_buf[(5, 1)].fg, t.color.on_accent, "V1: on-accent text");
    assert!(buf_is_bold(&idle_buf, 5, 1), "V1: bold text");
    assert_eq!(idle_buf[(4, 1)].symbol(), " ", "V1: outer pad cell");
    let fired = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        BrandApp {
            fired: Rc::clone(&fired),
            clickable: true,
        },
        Theme::junie(),
        40,
        8,
    );
    let _ = app.mouse(MouseKind::Move, 2, 0);
    assert_eq!(
        app.hover(),
        Some(S2_BRAND),
        "S1/A1: hit region reports the brand click"
    );
    assert!(!app.tab_to(S2_BRAND), "S1: never a focus stop");
    assert_surround_intact(&idle_buf, Rect::new(4, 1, 8, 1), "brand idle");
    let idle = capture_isolated(&dir, "brand-idle", &idle_buf);

    let hov_buf = render_isolated(
        16,
        3,
        0,
        Some((S2_BRAND, ReferenceState::HOVERED)),
        |ui, _| {
            Brand::new(S2_BRAND, "holla❯")
                .clickable(true)
                .draw(ui, Rect::new(4, 1, 8, 1));
        },
    );
    assert_eq!(
        hov_buf[(5, 1)].bg,
        t.color.accent_hover,
        "V2: hover lifts the fill"
    );
    assert_eq!(
        hov_buf[(5, 1)].fg,
        t.color.on_accent,
        "V2: hover keeps the text"
    );
    let hovered = capture_isolated(&dir, "brand-hover", &hov_buf);
    assert_ne!(
        hovered.digest(),
        idle.digest(),
        "V2: hover changes the frame"
    );

    for name in ["press", "flash"] {
        let buf = render_isolated(
            16,
            3,
            0,
            Some((
                S2_BRAND,
                ReferenceState::HOVERED.union(ReferenceState::PRESSED),
            )),
            |ui, _| {
                Brand::new(S2_BRAND, "holla❯")
                    .clickable(true)
                    .draw(ui, Rect::new(4, 1, 8, 1));
            },
        );
        assert_eq!(buf[(5, 1)].bg, t.color.accent_pressed, "V2: {name} fill");
        capture_isolated(&dir, &format!("brand-{name}"), &buf);
    }

    let press_no_hover_buf = render_isolated(
        16,
        3,
        0,
        Some((S2_BRAND, ReferenceState::PRESSED)),
        |ui, _| {
            Brand::new(S2_BRAND, "holla❯")
                .clickable(true)
                .draw(ui, Rect::new(4, 1, 8, 1));
        },
    );
    let press_no_hover = capture_isolated(&dir, "brand-press-nohover", &press_no_hover_buf);
    assert_eq!(
        press_no_hover.digest(),
        idle.digest(),
        "V2: press w/o hover is idle"
    );
    let buf_a = render_isolated(16, 3, 0, None, |ui, _| {
        Brand::new(S2_BRAND, "holla❯").draw(ui, Rect::new(4, 1, 8, 1));
    });
    let buf_b = render_isolated(16, 3, 0, None, |ui, _| {
        Brand::new(S2_BRAND, "holla❯").draw(ui, Rect::new(4, 1, 8, 1));
    });
    let plain_a = capture_isolated(&dir, "brand-plain-a", &buf_a);
    let plain_b = capture_isolated(&dir, "brand-plain-b", &buf_b);
    assert_eq!(
        plain_a.digest(),
        plain_b.digest(),
        "N1: plain render stateless"
    );

    assert!(!app.tab_to(S2_BRAND), "N2: no focus stop ever");

    let case = s2_case("brand_hover", &["--page", "chrome"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(boot.contains("app❯"), "boot shows the brand lockup");
    let idle_frame = live_frame(&mut s);
    let (brow, bcol) = support::screen_find(
        &s.inner
            .snapshot()
            .unwrap_or_else(|e| panic!("brand pos failed: {e:#}")),
        "app❯",
    )
    .expect("lockup on screen");
    let idle_bg = idle_frame
        .get(bcol + 1, brow)
        .unwrap_or_else(|| panic!("lockup cell missing"))
        .bg;
    hover_over(&mut s, "app❯", case_timeout(&case));
    let hov_frame = live_frame(&mut s);
    let hov_bg = hov_frame
        .get(bcol + 1, brow)
        .unwrap_or_else(|| panic!("hovered lockup cell missing"))
        .bg;
    assert_ne!(idle_bg, hov_bg, "V2 live: hover lifts the lockup fill");

    let hov_text = live_text(&mut s);
    assert!(hov_text.contains("app❯"), "V2 live: glyphs stable");

    Input::down(MouseButton::Left, bcol + 1, brow).send(&mut s);
    std::thread::sleep(Duration::from_millis(150));
    let press_frame = live_frame(&mut s);
    let press_bg = press_frame
        .get(bcol + 1, brow)
        .unwrap_or_else(|| panic!("pressed lockup cell missing"))
        .bg;
    Input::up(MouseButton::Left, bcol + 1, brow).send(&mut s);
    assert_ne!(hov_bg, press_bg, "V2 live: press moves the fill");
    checkpoint(&mut s, &dir, "pressed");
    eprintln!("s2 brand_hover: isolated matrix + live lift/press");
}

const S2_BTNFOCUS: Id = Id::root("s2.btnfocus");
const S2_BTNFOCUS_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 16,
    height: 1,
};

struct FocusButtonApp {
    fired: Rc<Cell<u32>>,
    disabled: bool,
    status: Status,
}

impl App for FocusButtonApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let fired = Rc::clone(&self.fired);
        Button::new(S2_BTNFOCUS, "Run task")
            .variant(Variant::SECONDARY)
            .disabled(self.disabled)
            .status(self.status)
            .update(cx)
            .on_activated(|| fired.set(fired.get() + 1))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(S2_BTNFOCUS, "Run task")
            .variant(Variant::SECONDARY)
            .disabled(self.disabled)
            .status(self.status)
            .draw(ui, S2_BTNFOCUS_AREA);
    }
}

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_button_focus() {
    let dir = s2_dir("s2_button_focus");
    let t = Theme::junie();

    let idle_buf = render_isolated(24, 3, 0, None, |ui, _| {
        Button::new(S2_BTNFOCUS, "Run task")
            .variant(Variant::SECONDARY)
            .draw(ui, Rect::new(4, 1, 16, 1));
    });
    assert_eq!(idle_buf[(4, 1)].symbol(), " ", "idle gutter blank");
    let idle_bg = idle_buf[(5, 1)].bg;
    let idle = capture_isolated(&dir, "btn-idle", &idle_buf);

    let kbd_buf = render_isolated(
        24,
        3,
        0,
        Some((S2_BTNFOCUS, ReferenceState::FOCUSED)),
        |ui, _| {
            Button::new(S2_BTNFOCUS, "Run task")
                .variant(Variant::SECONDARY)
                .draw(ui, Rect::new(4, 1, 16, 1));
        },
    );
    assert_eq!(kbd_buf[(4, 1)].symbol(), "▎", "V1: kbd-focus bar");
    assert_eq!(kbd_buf[(4, 1)].fg, t.color.focus, "V1: focus tone");
    assert!(buf_is_bold(&kbd_buf, 5, 1), "V1: kbd-focus bold");
    assert_eq!(kbd_buf[(5, 1)].bg, idle_bg, "V1: no hover lift");
    let kbd = capture_isolated(&dir, "btn-kbdfocus", &kbd_buf);
    assert_ne!(kbd.digest(), idle.digest(), "V1: focus changes the frame");

    let ptr_buf = render_isolated(
        24,
        3,
        0,
        Some((
            S2_BTNFOCUS,
            ReferenceState::FOCUSED.union(ReferenceState::HOVERED),
        )),
        |ui, _| {
            Button::new(S2_BTNFOCUS, "Run task")
                .variant(Variant::SECONDARY)
                .draw(ui, Rect::new(4, 1, 16, 1));
        },
    );
    assert_eq!(ptr_buf[(4, 1)].symbol(), "▎", "V1: pointer-focus bar");
    assert!(buf_is_bold(&ptr_buf, 5, 1), "V1: pointer-focus bold");
    assert_ne!(ptr_buf[(5, 1)].bg, idle_bg, "V1: hover lift present");
    let ptr = capture_isolated(&dir, "btn-pointerfocus", &ptr_buf);

    let pressed_buf = render_isolated(
        24,
        3,
        0,
        Some((
            S2_BTNFOCUS,
            ReferenceState::FOCUSED
                .union(ReferenceState::HOVERED)
                .union(ReferenceState::PRESSED),
        )),
        |ui, _| {
            Button::new(S2_BTNFOCUS, "Run task")
                .variant(Variant::SECONDARY)
                .draw(ui, Rect::new(4, 1, 16, 1));
        },
    );
    let pressed = capture_isolated(&dir, "btn-pressed", &pressed_buf);
    assert_ne!(pressed.digest(), ptr.digest(), "V2: press inverts");

    let hidden_buf = render_isolated(
        24,
        3,
        0,
        Some((S2_BTNFOCUS, ReferenceState::FOCUSED)),
        |ui, _| {
            Button::new(S2_BTNFOCUS, "Run task")
                .variant(Variant::SECONDARY)
                .draw(ui, Rect::new(4, 1, 16, 1));
        },
    );
    let hidden = capture_isolated(&dir, "btn-hidden", &hidden_buf);
    assert_eq!(hidden.digest(), idle.digest(), "V1: hidden focus is idle");

    let pnh_buf = render_isolated(
        24,
        3,
        0,
        Some((S2_BTNFOCUS, ReferenceState::PRESSED)),
        |ui, _| {
            Button::new(S2_BTNFOCUS, "Run task")
                .variant(Variant::SECONDARY)
                .draw(ui, Rect::new(4, 1, 16, 1));
        },
    );
    let press_no_hover = capture_isolated(&dir, "btn-press-nohover", &pnh_buf);
    assert_eq!(
        press_no_hover.digest(),
        idle.digest(),
        "N1: press w/o hover is idle"
    );

    let danger_buf = render_isolated(24, 3, 0, None, |ui, _| {
        Button::new(S2_BTNFOCUS, "Run task")
            .variant(Variant::DANGER)
            .draw(ui, Rect::new(4, 1, 16, 1));
    });
    let danger = capture_isolated(&dir, "btn-danger", &danger_buf);
    assert_ne!(
        danger.digest(),
        idle.digest(),
        "V2: danger differs from secondary"
    );

    for on in [true, false] {
        let w = measure_size(|ui, c| {
            Button::new(S2_BTNFOCUS, "Verbose")
                .variant(Variant::TOGGLE)
                .checked(on)
                .measure(ui, c)
        });
        assert_eq!(w.preferred.0, 7 + 2 + 2, "S1: on={on} width");
    }
    let on_buf = render_isolated(24, 3, 0, None, |ui, _| {
        Button::new(S2_BTNFOCUS, "Verbose")
            .variant(Variant::TOGGLE)
            .checked(true)
            .draw(ui, Rect::new(4, 1, 16, 1));
    });
    assert_eq!(on_buf[(5, 1)].symbol(), "●", "S1: on marker");
    assert_eq!(on_buf[(5, 1)].fg, t.color.accent, "S1: on accent");

    let fired = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        FocusButtonApp {
            fired: Rc::clone(&fired),
            disabled: false,
            status: Status::Ready,
        },
        Theme::junie(),
        40,
        8,
    );
    let _ = app.mouse(MouseKind::Move, 2, 0);
    assert_eq!(app.hover(), Some(S2_BTNFOCUS), "S2: hit region");
    assert!(app.tab_to(S2_BTNFOCUS), "S2: ring stop");

    let _ = app.key(KeyCode::Enter);
    assert_eq!(fired.get(), 1, "A1: Enter");
    let _ = app.key(KeyCode::Char(' '));
    assert_eq!(fired.get(), 2, "A1: Space");
    let r = app.key(KeyCode::Char('x'));
    assert_eq!(r.flow(), Flow::Ignored, "A1: other keys ignore");
    assert_eq!(fired.get(), 2, "A1: other keys never fire");
    let _ = app.click(2, 0);
    assert_eq!(fired.get(), 3, "A1: click");

    for (disabled, busy) in [(false, false), (true, false), (false, true), (true, true)] {
        let fired = Rc::new(Cell::new(0));
        let mut app = Harness::new(
            FocusButtonApp {
                fired: Rc::clone(&fired),
                disabled,
                status: if busy { Status::Busy } else { Status::Ready },
            },
            Theme::junie(),
            40,
            8,
        );
        assert_eq!(app.tab_to(S2_BTNFOCUS), !disabled, "N2: ring reach");
        let _ = app.key(KeyCode::Enter);
        let _ = app.click(2, 0);
        assert_eq!(
            fired.get() > 0,
            !disabled && !busy,
            "N2: gates are disabled/busy only (disabled={disabled} busy={busy})"
        );
    }

    let case = s2_case("button_focus", &["--page", "buttons"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "Enter / Space Activate", "A1 footer");
    let focused = live_text(&mut s);
    assert_line_has(&focused, "▎", "Run task", "V1 live: focus bar on Run task");
    checkpoint(&mut s, &dir, "focused");
    hover_over(&mut s, "Run task", case_timeout(&case));
    let hov = live_frame(&mut s);
    let hov_text = live_text(&mut s);
    assert_line_has(&hov_text, "▎", "Run task", "V1 live: bar survives hover");

    let (row, col) = support::screen_find(
        &s.inner
            .snapshot()
            .unwrap_or_else(|e| panic!("label pos failed: {e:#}")),
        "Run task",
    )
    .expect("label on screen");
    let cell = hov
        .get(col, row)
        .unwrap_or_else(|| panic!("label cell missing"));
    assert_ne!(cell.bg, TColor::Default, "V1 live: label bg painted");
    let _ = lum(cell.bg, "V1 live label bg");
    eprintln!("s2 button_focus: isolated matrix + live tab/hover");
}

const S2_BTNDIS: Id = Id::root("s2.btndis");
const S2_BTNDIS_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 16,
    height: 1,
};

struct DisabledButtonApp {
    fired: Rc<Cell<u32>>,
    disabled: bool,
    status: Status,
}

impl App for DisabledButtonApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let fired = Rc::clone(&self.fired);
        Button::new(S2_BTNDIS, "Verbose")
            .variant(Variant::TOGGLE)
            .checked(true)
            .disabled(self.disabled)
            .status(self.status)
            .update(cx)
            .on_activated(|| fired.set(fired.get() + 1))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(S2_BTNDIS, "Verbose")
            .variant(Variant::TOGGLE)
            .checked(true)
            .disabled(self.disabled)
            .status(self.status)
            .draw(ui, S2_BTNDIS_AREA);
    }
}

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_button_disabled() {
    let dir = s2_dir("s2_button_disabled");
    let t = Theme::junie();

    let idle_buf = render_isolated(24, 3, 0, None, |ui, _| {
        Button::new(S2_BTNDIS, "Verbose")
            .variant(Variant::TOGGLE)
            .checked(true)
            .disabled(true)
            .draw(ui, Rect::new(4, 1, 16, 1));
    });
    assert_eq!(idle_buf[(4, 1)].symbol(), " ", "V1: blank gutter");
    assert!(!buf_is_bold(&idle_buf, 5, 1), "V1: never bold");

    assert_eq!(idle_buf[(5, 1)].symbol(), "●", "V2: marker kept");
    assert_ne!(idle_buf[(5, 1)].fg, t.color.accent, "V2: accent lost");
    let idle = capture_isolated(&dir, "btndis-idle", &idle_buf);

    let allset_buf = render_isolated(
        24,
        3,
        0,
        Some((
            S2_BTNDIS,
            ReferenceState::FOCUSED
                .union(ReferenceState::HOVERED)
                .union(ReferenceState::PRESSED),
        )),
        |ui, _| {
            Button::new(S2_BTNDIS, "Verbose")
                .variant(Variant::TOGGLE)
                .checked(true)
                .disabled(true)
                .draw(ui, Rect::new(4, 1, 16, 1));
        },
    );
    let allset = capture_isolated(&dir, "btndis-allset", &allset_buf);
    assert_eq!(
        allset.digest(),
        idle.digest(),
        "V1/N1: no lift/bar/inversion"
    );

    let fired = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        DisabledButtonApp {
            fired: Rc::clone(&fired),
            disabled: true,
            status: Status::Ready,
        },
        Theme::junie(),
        40,
        8,
    );
    let _ = app.mouse(MouseKind::Move, 2, 0);
    assert_eq!(app.hover(), Some(S2_BTNDIS), "S1: hit area kept");
    assert!(!app.tab_to(S2_BTNDIS), "S1: no ring stop");

    let _ = app.key(KeyCode::Enter);
    let _ = app.click(2, 0);
    assert_eq!(fired.get(), 0, "S2/A1: disabled never fires");
    let still = render_isolated(24, 3, 0, None, |ui, _| {
        Button::new(S2_BTNDIS, "Verbose")
            .variant(Variant::TOGGLE)
            .checked(true)
            .disabled(true)
            .draw(ui, Rect::new(4, 1, 16, 1));
    });
    assert_eq!(still[(5, 1)].symbol(), "●", "A1: toggle never flips");
    let fired = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        DisabledButtonApp {
            fired: Rc::clone(&fired),
            disabled: false,
            status: Status::Busy,
        },
        Theme::junie(),
        40,
        8,
    );
    assert!(app.tab_to(S2_BTNDIS), "S2: busy stays reachable");
    let _ = app.key(KeyCode::Enter);
    let _ = app.click(2, 0);
    assert_eq!(fired.get(), 0, "S2: busy cannot activate");

    let busy_buf = render_isolated(
        24,
        3,
        3,
        Some((
            S2_BTNDIS,
            ReferenceState::HOVERED.union(ReferenceState::PRESSED),
        )),
        |ui, _| {
            Button::new(S2_BTNDIS, "Verbose")
                .variant(Variant::TOGGLE)
                .checked(true)
                .status(Status::Busy)
                .draw(ui, Rect::new(4, 1, 16, 1));
        },
    );
    assert_eq!(
        busy_buf[(5, 1)].symbol(),
        SPINNER[3],
        "V2: busy spinner kept"
    );
    capture_isolated(&dir, "btndis-busy", &busy_buf);

    let case = s2_case("button_disabled", &["--page", "buttons"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(
        boot.contains("Disabled primary"),
        "boot shows disabled buttons"
    );

    let mut saw_enabled_bar = false;
    for _ in 0..8 {
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(150));
        let text = live_text(&mut s);
        assert_line_lacks(&text, "▎", "Disabled", "N1 live: bar never on disabled");
        if text
            .lines()
            .any(|l| l.contains('▎') && l.contains("Run task"))
        {
            saw_enabled_bar = true;
        }
    }
    assert!(saw_enabled_bar, "N1 presence: bar lands on enabled rows");

    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "Run task", "refocus");
    click_at(&mut s, "Disabled primary", case_timeout(&case));
    let after = live_text(&mut s);
    assert_line_lacks(&after, "▎", "Disabled", "A1 live: click moves no focus");
    assert_line_has(&after, "▎", "Run task", "A1 live: focus stays put");
    checkpoint(&mut s, &dir, "clicked-disabled");
    eprintln!("s2 button_disabled: isolated precedence + live no-focus/no-fire");
}

const S2_CHK: Id = Id::root("s2.chk");
const S2_CHK_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 20,
    height: 1,
};

struct CheckboxApp {
    value: Rc<Cell<bool>>,
    fired: Rc<Cell<u32>>,
    disabled: bool,
}

impl App for CheckboxApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut v = self.value.get();
        let fired = Rc::clone(&self.fired);
        let r = Checkbox::new(S2_CHK, "Run tests")
            .checked(v)
            .disabled(self.disabled)
            .update(cx, &mut v)
            .on_activated(|| fired.set(fired.get() + 1));
        self.value.set(v);
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Checkbox::new(S2_CHK, "Run tests")
            .checked(self.value.get())
            .disabled(self.disabled)
            .draw(ui, S2_CHK_AREA);
    }
}

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_checkbox_matrix() {
    let dir = s2_dir("s2_checkbox_matrix");
    let t = Theme::junie();

    let idle_buf = render_isolated(30, 3, 0, None, |ui, _| {
        Checkbox::new(S2_CHK, "Run tests")
            .checked(false)
            .draw(ui, Rect::new(4, 1, 20, 1));
    });
    let row = buf_row_text(&idle_buf, 1);
    assert!(row.contains("[ ]"), "V1: unchecked mark in {row:?}");
    assert_eq!(
        idle_buf[(6, 1)].fg,
        t.color.fg[FgStep::Muted.index()],
        "V1: unchecked muted"
    );
    let idle = capture_isolated(&dir, "chk-idle", &idle_buf);

    let hot_buf = render_isolated(
        30,
        3,
        0,
        Some((
            S2_CHK,
            ReferenceState::FOCUSED.union(ReferenceState::HOVERED),
        )),
        |ui, _| {
            Checkbox::new(S2_CHK, "Run tests")
                .checked(true)
                .draw(ui, Rect::new(4, 1, 20, 1));
        },
    );
    let row = buf_row_text(&hot_buf, 1);
    assert!(row.contains("[✓]"), "V1: checked mark in {row:?}");
    assert_eq!(hot_buf[(4, 1)].symbol(), "▎", "V1: focus bar");
    assert!(buf_is_bold(&hot_buf, 6, 1), "V1: focus bold");
    assert_eq!(hot_buf[(6, 1)].fg, t.color.accent, "V1: checked accent");
    let hot = capture_isolated(&dir, "chk-hot", &hot_buf);
    assert_ne!(hot.digest(), idle.digest(), "V1: state changes the frame");

    let hidden_buf = render_isolated(
        30,
        3,
        0,
        Some((S2_CHK, ReferenceState::FOCUSED)),
        |ui, _| {
            Checkbox::new(S2_CHK, "Run tests")
                .checked(false)
                .draw(ui, Rect::new(4, 1, 20, 1));
        },
    );
    let hidden = capture_isolated(&dir, "chk-hidden", &hidden_buf);
    assert_eq!(hidden.digest(), idle.digest(), "V1: hidden focus is idle");

    let narrow_buf = render_isolated(12, 3, 0, None, |ui, _| {
        Checkbox::new(S2_CHK, "Run tests")
            .checked(true)
            .draw(ui, Rect::new(4, 1, 3, 1));
    });
    let narrow = buf_row_text(&narrow_buf, 1);
    assert!(
        narrow.contains('✓') && !narrow.contains('['),
        "V2: compact mark in {narrow:?}"
    );
    capture_isolated(&dir, "chk-narrow3", &narrow_buf);
    let one_buf = render_isolated(
        12,
        3,
        0,
        Some((S2_CHK, ReferenceState::FOCUSED)),
        |ui, _| {
            Checkbox::new(S2_CHK, "Run tests")
                .checked(true)
                .draw(ui, Rect::new(4, 1, 1, 1));
        },
    );
    assert_eq!(one_buf[(4, 1)].symbol(), "▎", "V2: 1-cell bar only");

    let dis_buf = render_isolated(30, 3, 0, None, |ui, _| {
        Checkbox::new(S2_CHK, "Run tests")
            .checked(true)
            .disabled(true)
            .draw(ui, Rect::new(4, 1, 20, 1));
    });
    let drow = buf_row_text(&dis_buf, 1);
    assert!(drow.contains('✓'), "S1: disabled keeps ✓ in {drow:?}");
    let value = Rc::new(Cell::new(true));
    let fired = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        CheckboxApp {
            value: Rc::clone(&value),
            fired: Rc::clone(&fired),
            disabled: true,
        },
        Theme::junie(),
        40,
        8,
    );
    assert!(!app.tab_to(S2_CHK), "S1: no focus stop");
    let dis_idle = capture_isolated(&dir, "chk-dis-idle", &dis_buf);
    let dis_hot_buf = render_isolated(
        30,
        3,
        0,
        Some((
            S2_CHK,
            ReferenceState::FOCUSED
                .union(ReferenceState::HOVERED)
                .union(ReferenceState::PRESSED),
        )),
        |ui, _| {
            Checkbox::new(S2_CHK, "Run tests")
                .checked(true)
                .disabled(true)
                .draw(ui, Rect::new(4, 1, 20, 1));
        },
    );
    let dis_hot = capture_isolated(&dir, "chk-dis-hot", &dis_hot_buf);
    assert_eq!(
        dis_hot.digest(),
        dis_idle.digest(),
        "S1: disabled all-set is idle"
    );

    let value = Rc::new(Cell::new(false));
    let fired = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        CheckboxApp {
            value: Rc::clone(&value),
            fired: Rc::clone(&fired),
            disabled: false,
        },
        Theme::junie(),
        40,
        8,
    );
    assert!(app.tab_to(S2_CHK), "A1: ring stop");
    let r = app.key(KeyCode::Char(' '));
    assert!(r.flow() == Flow::Consumed && fired.get() == 1, "A1: Space");
    assert!(value.get(), "A1: flipped on");
    let r = app.key(KeyCode::Enter);
    assert!(r.flow() == Flow::Consumed && fired.get() == 2, "A1: Enter");
    assert!(!value.get(), "A1: flipped off");
    let r = app.click(2, 0);
    assert!(r.flow() == Flow::Consumed && fired.get() == 3, "A1: click");
    assert!(value.get(), "A1: click flips");
    let value = Rc::new(Cell::new(true));
    let fired = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        CheckboxApp {
            value: Rc::clone(&value),
            fired: Rc::clone(&fired),
            disabled: true,
        },
        Theme::junie(),
        40,
        8,
    );
    let r = app.key(KeyCode::Char(' '));
    assert_eq!(r.flow(), Flow::Ignored, "A1: disabled Space ignores");
    assert_eq!(fired.get(), 0, "A1: disabled Space never fires");
    let r = app.click(2, 0);
    assert_eq!(r.flow(), Flow::Consumed, "A1: disabled click consumes");
    assert_eq!(fired.get(), 0, "A1: disabled click never fires");
    assert!(value.get(), "A1: disabled never flips");

    let case = s2_case("checkbox_matrix", &["--page", "forms"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(boot.contains("[✓]"), "boot shows a checked box");
    assert!(boot.contains("[ ]"), "boot shows an unchecked box");
    click_at(&mut s, "[✓]", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1: checkbox never unticked",
        |screen| {
            !support::screen_text(screen)
                .lines()
                .any(|l| l.contains("[✓]") && l.contains("Run tests"))
        },
    );
    let unticked = live_text(&mut s);
    assert_line_has(&unticked, "[ ]", "Run tests", "A1 live: click unticks");
    click_at(&mut s, "Run tests", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1: checkbox never reticked",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("[✓]") && l.contains("Run tests"))
        },
    );
    checkpoint(&mut s, &dir, "reticked");
    eprintln!("s2 checkbox_matrix: isolated matrix + live toggle roundtrip");
}

const S2_TGL: Id = Id::root("s2.tgl");
const S2_TGL_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 26,
    height: 1,
};

struct ToggleApp {
    value: Rc<Cell<bool>>,
    fired: Rc<Cell<u32>>,
    disabled: bool,
}

impl App for ToggleApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut v = self.value.get();
        let fired = Rc::clone(&self.fired);
        let r = Toggle::new(S2_TGL, "Auto-approve")
            .on(v)
            .disabled(self.disabled)
            .update(cx, &mut v)
            .on_activated(|| fired.set(fired.get() + 1));
        self.value.set(v);
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Toggle::new(S2_TGL, "Auto-approve")
            .on(self.value.get())
            .disabled(self.disabled)
            .draw(ui, S2_TGL_AREA);
    }
}

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_toggle_matrix() {
    let dir = s2_dir("s2_toggle_matrix");
    let t = Theme::junie();

    let idle_buf = render_isolated(34, 3, 0, None, |ui, _| {
        Toggle::new(S2_TGL, "Auto-approve")
            .on(false)
            .draw(ui, Rect::new(4, 1, 26, 1));
    });
    let row = buf_row_text(&idle_buf, 1);
    assert!(row.contains("○──"), "V1: off switch in {row:?}");
    assert!(row.contains("off"), "V1: off word in {row:?}");
    assert_eq!(
        idle_buf[(5, 1)].fg,
        t.color.fg[FgStep::Muted.index()],
        "V1: off muted"
    );
    let idle = capture_isolated(&dir, "tgl-off", &idle_buf);

    let hot_buf = render_isolated(
        34,
        3,
        0,
        Some((
            S2_TGL,
            ReferenceState::FOCUSED.union(ReferenceState::HOVERED),
        )),
        |ui, _| {
            Toggle::new(S2_TGL, "Auto-approve")
                .on(true)
                .draw(ui, Rect::new(4, 1, 26, 1));
        },
    );
    let row = buf_row_text(&hot_buf, 1);
    assert!(row.contains("──●"), "V1: on switch in {row:?}");
    assert!(row.contains(" on"), "V1: on word in {row:?}");
    assert_eq!(hot_buf[(4, 1)].symbol(), "▎", "V1: focus bar");
    assert!(buf_is_bold(&hot_buf, 6, 1), "V1: focus bold");
    assert_eq!(hot_buf[(7, 1)].fg, t.color.accent, "V1: on accent");
    let hot = capture_isolated(&dir, "tgl-hot", &hot_buf);
    assert_ne!(hot.digest(), idle.digest(), "V1: state changes the frame");

    let narrow_buf = render_isolated(12, 3, 0, None, |ui, _| {
        Toggle::new(S2_TGL, "Auto-approve")
            .on(true)
            .draw(ui, Rect::new(4, 1, 3, 1));
    });
    let narrow = buf_row_text(&narrow_buf, 1);
    assert!(
        narrow.contains('●') && !narrow.contains("──"),
        "V2: compact ● in {narrow:?}"
    );
    capture_isolated(&dir, "tgl-narrow3", &narrow_buf);

    let dis_buf = render_isolated(34, 3, 0, None, |ui, _| {
        Toggle::new(S2_TGL, "Auto-approve")
            .on(true)
            .disabled(true)
            .draw(ui, Rect::new(4, 1, 26, 1));
    });
    let drow = buf_row_text(&dis_buf, 1);
    assert!(
        drow.contains("──●"),
        "S1: disabled keeps glyphs in {drow:?}"
    );
    let value = Rc::new(Cell::new(true));
    let fired = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        ToggleApp {
            value: Rc::clone(&value),
            fired: Rc::clone(&fired),
            disabled: true,
        },
        Theme::junie(),
        40,
        8,
    );
    assert!(!app.tab_to(S2_TGL), "S1: no focus stop");
    let dis_idle = capture_isolated(&dir, "tgl-dis-idle", &dis_buf);
    let dis_hot_buf = render_isolated(
        34,
        3,
        0,
        Some((
            S2_TGL,
            ReferenceState::FOCUSED
                .union(ReferenceState::HOVERED)
                .union(ReferenceState::PRESSED),
        )),
        |ui, _| {
            Toggle::new(S2_TGL, "Auto-approve")
                .on(true)
                .disabled(true)
                .draw(ui, Rect::new(4, 1, 26, 1));
        },
    );
    let dis_hot = capture_isolated(&dir, "tgl-dis-hot", &dis_hot_buf);
    assert_eq!(
        dis_hot.digest(),
        dis_idle.digest(),
        "S1: disabled all-set is idle"
    );

    let value = Rc::new(Cell::new(false));
    let fired = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        ToggleApp {
            value: Rc::clone(&value),
            fired: Rc::clone(&fired),
            disabled: false,
        },
        Theme::junie(),
        40,
        8,
    );
    assert!(app.tab_to(S2_TGL), "A1: ring stop");
    let r = app.key(KeyCode::Char(' '));
    assert!(r.flow() == Flow::Consumed && fired.get() == 1, "A1: Space");
    assert!(value.get(), "A1: flipped on");
    let r = app.key(KeyCode::Enter);
    assert!(r.flow() == Flow::Consumed && fired.get() == 2, "A1: Enter");
    assert!(!value.get(), "A1: flipped off");
    let r = app.click(2, 0);
    assert!(r.flow() == Flow::Consumed && fired.get() == 3, "A1: click");
    assert!(value.get(), "A1: click flips");
    let value = Rc::new(Cell::new(true));
    let fired = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        ToggleApp {
            value: Rc::clone(&value),
            fired: Rc::clone(&fired),
            disabled: true,
        },
        Theme::junie(),
        40,
        8,
    );
    let r = app.key(KeyCode::Char(' '));
    assert_eq!(r.flow(), Flow::Ignored, "A1: disabled keys ignore");
    assert_eq!(fired.get(), 0, "A1: disabled keys never fire");
    let r = app.click(2, 0);
    assert_eq!(r.flow(), Flow::Consumed, "A1: disabled click consumes");
    assert_eq!(fired.get(), 0, "A1: disabled click never fires");
    assert!(value.get(), "A1: disabled never flips");

    let case = s2_case("toggle_matrix", &["--page", "forms"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(boot.contains("○──"), "boot shows an off toggle");
    assert!(boot.contains("──●"), "boot shows an on toggle");
    click_at(&mut s, "Auto-approve", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1: toggle never flipped on",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("──●") && l.contains("Auto-approve"))
        },
    );
    click_at(&mut s, "Auto-approve", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1: toggle never flipped off",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("○──") && l.contains("Auto-approve"))
        },
    );
    checkpoint(&mut s, &dir, "roundtripped");
    eprintln!("s2 toggle_matrix: isolated matrix + live flip roundtrip");
}

const S2_RADIO: Id = Id::root("s2.radio");
const S2_RADIO_SENTINEL: Id = Id::root("s2.radio.sentinel");
const S2_RADIO_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 20,
    height: 3,
};
const S2_RADIO_ITEMS: [&str; 3] = ["Fast", "Balanced", "Thorough"];

struct RadioApp {
    value: Rc<Cell<Option<termrock::ItemKey>>>,
    st: RadioGroupState,
    chose: Rc<RefCell<Vec<termrock::ItemKey>>>,
    disabled: bool,
    empty: bool,
}

impl App for RadioApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let chose = Rc::clone(&self.chose);
        let value = Rc::clone(&self.value);
        let mut group = RadioGroup::new(S2_RADIO).disabled(self.disabled);
        if let Some(v) = value.get() {
            group = group.value(v);
        }
        let items: &[&str] = if self.empty { &[] } else { &S2_RADIO_ITEMS };
        group.update(cx, &mut self.st, items).on_action(|action| {
            let RadioGroupAction::Chose(key) = action;
            value.set(Some(key));
            chose.borrow_mut().push(key);
        })
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let mut group = RadioGroup::new(S2_RADIO).disabled(self.disabled);
        if let Some(v) = self.value.get() {
            group = group.value(v);
        }
        let items: &[&str] = if self.empty { &[] } else { &S2_RADIO_ITEMS };
        group.draw(ui, S2_RADIO_AREA, &self.st, items);
        ui.register_control(
            S2_RADIO_SENTINEL,
            Rect::new(0, 5, 8, 1),
            Focusability::Focusable,
        );
    }
}

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_radio_matrix() {
    use termrock::ItemKey;

    let dir = s2_dir("s2_radio_matrix");
    let t = Theme::junie();

    let mut st = RadioGroupState::default();
    st.set_cursor(1, ItemKey::index(1));
    let idle_buf = render_isolated(30, 6, 0, None, |ui, _| {
        RadioGroup::new(S2_RADIO).value(ItemKey::index(1)).draw(
            ui,
            Rect::new(4, 1, 20, 4),
            &st,
            &S2_RADIO_ITEMS,
        );
    });
    let label = buf_row_text(&idle_buf, 1);
    let opt0 = buf_row_text(&idle_buf, 2);
    let opt1 = buf_row_text(&idle_buf, 3);
    assert!(label.contains("Mode"), "V1: label in {label:?}");
    assert!(opt0.contains("( )"), "V1: unselected mark in {opt0:?}");
    assert!(opt1.contains("(●)"), "V1: selected mark in {opt1:?}");
    assert_eq!(idle_buf[(6, 3)].fg, t.color.accent, "V1: selected accent");
    let idle = capture_isolated(&dir, "radio-idle", &idle_buf);

    let mut st = RadioGroupState::default();
    st.set_cursor(1, ItemKey::index(1));
    let hot_buf = render_isolated(30, 6, 0, None, |ui, _| {
        ui.reference(
            Some(
                ReferenceTarget::new(
                    S2_RADIO,
                    ReferenceState::FOCUSED.union(ReferenceState::HOVERED),
                )
                .part(PartRef::item(Part::ROW, ItemKey::index(0))),
            ),
            |ui| {
                RadioGroup::new(S2_RADIO).value(ItemKey::index(1)).draw(
                    ui,
                    Rect::new(4, 1, 20, 4),
                    &st,
                    &S2_RADIO_ITEMS,
                );
            },
        );
    });
    assert!(buf_is_bold(&hot_buf, 6, 1), "V1: focused label bold");
    assert_eq!(hot_buf[(4, 3)].symbol(), "▎", "V1: bar on cursor row");
    assert_eq!(hot_buf[(4, 2)].symbol(), " ", "V1: no bar elsewhere");
    assert_ne!(
        hot_buf[(6, 2)].bg,
        hot_buf[(6, 4)].bg,
        "V2: exactly one row lifted"
    );
    let hot = capture_isolated(&dir, "radio-hot", &hot_buf);
    assert_ne!(hot.digest(), idle.digest(), "V1: state changes the frame");

    let value = Rc::new(Cell::new(Some(ItemKey::index(1))));
    let chose = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        RadioApp {
            value: Rc::clone(&value),
            st: RadioGroupState::default(),
            chose: Rc::clone(&chose),
            disabled: false,
            empty: false,
        },
        Theme::junie(),
        40,
        8,
    );
    assert!(app.tab_to(S2_RADIO), "S1: one ring stop");
    let _ = app.key(KeyCode::Tab);
    assert!(
        app.state_of(S2_RADIO_SENTINEL)
            .contains(StateFlags::FOCUSED),
        "S1: options are not stops (single group stop)"
    );
    let before = chose.borrow().len();
    let _ = app.click(8, 0);
    assert_eq!(chose.borrow().len(), before + 1, "S1: option click target");
    assert_eq!(
        chose.borrow().last().copied(),
        Some(ItemKey::index(0)),
        "S1: click chooses option 0"
    );

    let mut app = Harness::new(
        RadioApp {
            value: Rc::new(Cell::new(None)),
            st: RadioGroupState::default(),
            chose: Rc::new(RefCell::new(Vec::new())),
            disabled: false,
            empty: true,
        },
        Theme::junie(),
        40,
        8,
    );
    assert!(!app.tab_to(S2_RADIO), "S2: empty group has no stop");

    let value = Rc::new(Cell::new(Some(ItemKey::index(1))));
    let chose = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        RadioApp {
            value: Rc::clone(&value),
            st: RadioGroupState::default(),
            chose: Rc::clone(&chose),
            disabled: false,
            empty: false,
        },
        Theme::junie(),
        40,
        8,
    );
    assert!(app.tab_to(S2_RADIO), "A1: ring stop");
    let r = app.key(KeyCode::Down);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Down moves");
    assert_eq!(
        app.buffer()[(0, 2)].symbol(),
        "▎",
        "A1: cursor moved to option 2"
    );
    assert_eq!(
        value.get(),
        Some(ItemKey::index(1)),
        "A1: arrows move the cursor, not the value"
    );
    assert!(chose.borrow().is_empty(), "A1: move reports no choice");
    let r = app.key(KeyCode::Char(' '));
    assert_eq!(r.flow(), Flow::Consumed, "A1: Space commits");
    assert_eq!(
        chose.borrow().last().copied(),
        Some(ItemKey::index(2)),
        "A1: Space chooses the cursor"
    );
    assert_eq!(value.get(), Some(ItemKey::index(2)));
    let moved_row: String = (0..20)
        .map(|x| app.buffer()[(x, 2)].symbol().to_owned())
        .collect();
    assert!(
        moved_row.contains("(●)"),
        "A1: selection moved in {moved_row:?}"
    );
    let _ = app.click(8, 0);
    assert_eq!(
        chose.borrow().last().copied(),
        Some(ItemKey::index(0)),
        "A1: click selects"
    );

    let case = s2_case("radio_matrix", &["--page", "forms"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert_line_has(&boot, "(●)", "Balanced", "boot selection");
    click_at(&mut s, "Fast", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1: radio never moved to Fast",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("(●)") && l.contains("Fast"))
        },
    );
    let moved = live_text(&mut s);
    assert_line_lacks(&moved, "(●)", "Balanced", "A1 live: old mark cleared");
    checkpoint(&mut s, &dir, "moved");
    eprintln!("s2 radio_matrix: isolated matrix + live move");
}

const S2_PANEL: Id = Id::root("s2.panel");
const S2_SCROLL: Id = Id::root("s2.scroll");
const S2_SCROLL_AREA: Rect = Rect {
    x: 4,
    y: 1,
    width: 22,
    height: 6,
};

struct ScrollApp {
    lines: Vec<String>,
    st: ViewportState,
}

impl App for ScrollApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let lines: Vec<ViewportLine<'_>> =
            self.lines.iter().map(|l| ViewportLine::Plain(l)).collect();
        TextViewport::new(S2_SCROLL)
            .update(cx, &mut self.st, &lines)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let lines: Vec<ViewportLine<'_>> =
            self.lines.iter().map(|l| ViewportLine::Plain(l)).collect();
        TextViewport::new(S2_SCROLL).draw(ui, S2_SCROLL_AREA, &self.st, &lines);
    }
}

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_panel_focus() {
    let dir = s2_dir("s2_panel_focus");

    let seen = Cell::new(Rect::default());
    let card_buf = render_isolated(40, 8, 0, None, |ui, _| {
        Panel::new(S2_PANEL)
            .kind(PanelKind::Card)
            .title("Titled card")
            .meta("surface")
            .draw(ui, Rect::new(4, 1, 30, 6), |_, inner| {
                seen.set(inner);
            });
    });
    assert_eq!(seen.get().y, 3, "S1: inner below title row + padding");
    let title_row = buf_row_text(&card_buf, 1);
    assert!(
        title_row.contains("Titled card"),
        "V1: title in {title_row:?}"
    );
    assert!(
        title_row.contains("surface"),
        "V2: meta kept in {title_row:?}"
    );
    capture_isolated(&dir, "panel-card-idle", &card_buf);

    let focused_buf = render_isolated(40, 8, 0, None, |ui, _| {
        Panel::new(S2_PANEL)
            .kind(PanelKind::Card)
            .title("Titled card")
            .focused(true)
            .meta("surface")
            .draw(ui, Rect::new(4, 1, 30, 6), |_, _| {});
    });
    assert_eq!(
        focused_buf[(5, 1)].symbol(),
        "▎",
        "V1: focus bar in padding col"
    );
    assert!(buf_is_bold(&focused_buf, 6, 1), "V1: focused title bold");
    capture_isolated(&dir, "panel-card-focused", &focused_buf);

    let framed_idle_buf = render_isolated(40, 8, 0, None, |ui, _| {
        Panel::new(S2_PANEL)
            .kind(PanelKind::Framed)
            .title("Framed")
            .draw(ui, Rect::new(4, 1, 30, 6), |_, _| {});
    });
    let idle_corner = framed_idle_buf[(4, 1)].fg;
    capture_isolated(&dir, "panel-framed-idle", &framed_idle_buf);
    let framed_hot_buf = render_isolated(40, 8, 0, None, |ui, _| {
        Panel::new(S2_PANEL)
            .kind(PanelKind::Framed)
            .title("Framed")
            .focused(true)
            .draw(ui, Rect::new(4, 1, 30, 6), |_, _| {});
    });
    assert_ne!(
        framed_hot_buf[(4, 1)].fg,
        idle_corner,
        "V1: focused border strengthens"
    );
    capture_isolated(&dir, "panel-framed-focused", &framed_hot_buf);

    let lines: Vec<String> = (0..20).map(|i| format!("log line {i:02}")).collect();

    let mut scroll_st = ViewportState::default();
    scroll_st.set_follow(false);
    let mut app = Harness::new(
        ScrollApp {
            lines: lines.clone(),
            st: scroll_st,
        },
        Theme::junie(),
        30,
        10,
    );
    assert!(app.tab_to(S2_SCROLL), "S2: scroll stop");
    let top: String = (0..22)
        .map(|x| app.buffer()[(4 + x, 1)].symbol().to_owned())
        .collect();
    assert!(top.contains("log line 00"), "S2: starts at top in {top:?}");
    let r = app.key(KeyCode::Down);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Down scrolls");
    let top: String = (0..22)
        .map(|x| app.buffer()[(4 + x, 1)].symbol().to_owned())
        .collect();
    assert!(top.contains("log line 01"), "S2: offset moved");
    capture_isolated(&dir, "panel-scrolled", app.buffer());
    for code in [KeyCode::Enter, KeyCode::Char('x'), KeyCode::Char(' ')] {
        let r = app.key(code);
        assert_eq!(r.flow(), Flow::Ignored, "A1/S2: {code:?} ignores");
    }

    let case = s2_case("panel_focus", &["--page", "panels"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(boot.contains("Titled card"), "boot shows the titled card");
    assert!(boot.contains("Framed"), "boot shows framed chrome");
    assert!(boot.contains("of 60"), "boot shows the log scroll readout");

    let mut entered = false;
    for _ in 0..4 {
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(200));
        let text = live_text(&mut s);
        if text.lines().any(|l| {
            l.contains('▎')
                && (l.contains("of 60") || l.contains("of 41") || l.contains("scrollable"))
        }) {
            entered = true;
            break;
        }
    }
    assert!(entered, "V1 live: focus never reached a card");
    checkpoint(&mut s, &dir, "focused");

    let (log_row, log_col) = find_pos(&mut s, "Resol", case_timeout(&case));
    let before = live_text(&mut s);
    let start_before = scroll_start(&before, "of 60");
    wheel_at(&mut s, log_col, log_row, Wheel::Down, 2);
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1: wheel never scrolled the log",
        |screen| scroll_start(&support::screen_text(screen), "of 60") != start_before,
    );
    let after_wheel = live_text(&mut s);
    assert!(
        scroll_start(&after_wheel, "of 60") > start_before,
        "A1 live: wheel scrolled down"
    );

    support::press_step(&s, "x");
    std::thread::sleep(Duration::from_millis(400));
    let after_type = live_text(&mut s);
    assert_eq!(after_type, after_wheel, "S2 live: typing mutates nothing");
    checkpoint(&mut s, &dir, "scrolled");
    eprintln!("s2 panel_focus: isolated chrome/scroll + live wheel/read-only");
}

fn scroll_start(text: &str, of: &str) -> u32 {
    for line in text.lines() {
        if let Some(pos) = line.find(of) {
            let head = &line[..pos];
            if let Some(dash) = head.rfind(['–', '-']) {
                let num: String = head[..dash]
                    .chars()
                    .rev()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                if let Ok(n) = num.parse::<u32>() {
                    return n;
                }
            }
        }
    }
    panic!("no {of:?} readout in\n{text}")
}

const S2_SPLIT: Id = Id::root("s2.split");

struct SplitApp {
    st: SplitPaneState,
    log: Rc<RefCell<Vec<termrock::SplitAction>>>,
    resizable: bool,
}

impl SplitApp {
    fn pane(&self) -> SplitPane<'static> {
        SplitPane::new(S2_SPLIT, SplitAxis::Horizontal)
            .gap(1)
            .min_first(5)
            .min_second(5)
            .resizable(self.resizable)
    }
}

impl App for SplitApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        self.pane()
            .update(cx, &mut self.st)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.pane()
            .draw(ui, Rect::new(0, 0, 80, 24), &self.st, |_, _, _| {});
    }
}

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_split_drag() {
    let dir = s2_dir("s2_split_drag");
    let container = Rect::new(0, 0, 80, 24);

    let split = SplitModel::new(SplitAxis::Horizontal, 50, 5, 5);
    let (left, right) = split.layout(container, 1);
    let handle = split.handle(container, 1);
    assert_eq!(left.width + 1 + right.width, 80, "V2: panes + gap fill");
    assert_eq!(handle.width, 1, "V2: 1-cell gap strip");
    assert_eq!(handle.x, left.right(), "V2: strip between panes");

    let paint_seam = |state: ReferenceState| {
        render_isolated(80, 24, 0, Some((S2_SPLIT, state)), |ui, _| {
            SplitPane::new(S2_SPLIT, SplitAxis::Horizontal)
                .gap(1)
                .min_first(5)
                .min_second(5)
                .draw(ui, container, &SplitPaneState::new(50), |_, _, _| {});
        })
    };
    let idle_buf = paint_seam(ReferenceState::default());
    assert_eq!(idle_buf[(handle.x, 12)].symbol(), "│", "V1: idle rule");
    let idle = capture_isolated(&dir, "split-idle", &idle_buf);
    let hov_buf = paint_seam(ReferenceState::HOVERED);
    assert_eq!(
        hov_buf[(handle.x, 12)].symbol(),
        "│",
        "V1: hover keeps the glyph"
    );
    let hovered = capture_isolated(&dir, "split-hover", &hov_buf);
    assert_ne!(hovered.digest(), idle.digest(), "V1: hover strengthens");

    for (name, state, heavy) in [
        ("focus", ReferenceState::FOCUSED, false),
        ("pressed", ReferenceState::PRESSED, true),
    ] {
        let buf = paint_seam(state);
        let want = if heavy { "┃" } else { "│" };
        assert_eq!(buf[(handle.x, 12)].symbol(), want, "N1: {name}");
        capture_isolated(&dir, &format!("split-{name}"), &buf);
    }

    let mut app = Harness::new(
        SplitApp {
            st: SplitPaneState::new(50),
            log: Rc::new(RefCell::new(Vec::new())),
            resizable: false,
        },
        Theme::junie(),
        80,
        24,
    );
    let _ = app.mouse(MouseKind::Move, handle.x, 12);
    assert_eq!(app.hover(), Some(S2_SPLIT), "S2: click hit");
    assert!(!app.tab_to(S2_SPLIT), "S2: never a stop");

    let mut split = SplitModel::new(SplitAxis::Horizontal, 50, 5, 5);
    assert!(
        split.drag_to(container, 1, Position::new(56, 12)),
        "A1: drag moves"
    );
    assert!(split.percent > 50, "S1: seam moved right");
    assert!(
        !split.drag_to(container, 1, Position::new(56, 12)),
        "S1: repeat consumes"
    );
    let (before_left, _) = split.layout(container, 1);
    split.nudge(container, 1, -1);
    let (after_left, _) = split.layout(container, 1);
    assert_eq!(
        after_left.width + 1,
        before_left.width,
        "S1: nudge moves one cell"
    );

    let log = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        SplitApp {
            st: SplitPaneState::new(50),
            log: Rc::clone(&log),
            resizable: true,
        },
        Theme::junie(),
        80,
        24,
    );
    let _ = app.drag((handle.x, 12), (handle.x.saturating_sub(8), 12));
    assert!(
        log.borrow()
            .iter()
            .any(|a| matches!(a, termrock::SplitAction::Resized(_))),
        "A1: live drag reports Resized"
    );

    split.toggle_max(Maximized::First);
    let handle = split.handle(container, 1);
    assert!(handle.is_empty(), "V2: maximized handle empty");
    let max_buf = render_isolated(80, 24, 0, None, |ui, _| {
        let mut st = SplitPaneState::new(50);
        st.toggle_max(Maximized::First);
        SplitPane::new(S2_SPLIT, SplitAxis::Horizontal)
            .gap(1)
            .min_first(5)
            .min_second(5)
            .draw(ui, container, &st, |_, _, _| {});
    });
    assert_surround_intact(
        &max_buf,
        Rect::new(0, 0, 0, 0),
        "V2: maximized draws nothing",
    );

    let case = s2_case(
        "split_drag",
        &["--page", "terminal", "--motion", "paused", "--frame", "60"],
        80,
        24,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");

    let boot = live_text(&mut s);
    let top = boot
        .lines()
        .find(|l| l.contains("Viewp") && l.contains('╮'))
        .expect("viewport top row")
        .to_string();
    let corner = top.chars().position(|c| c == '╮').expect("box corner");
    let seam_col = top
        .chars()
        .enumerate()
        .skip_while(|(i, _)| *i <= corner)
        .find(|(_, c)| *c == '│')
        .map(|(i, _)| i as u16)
        .expect("seam right of the corner");
    let top_row = boot
        .lines()
        .position(|l| l.contains("Viewp") && l.contains('╮'))
        .expect("top row") as u16;
    let grip_row = top_row + 8;
    let idle_frame = live_frame(&mut s);
    let idle_tone = idle_frame.get(seam_col, grip_row).expect("seam cell").fg;
    Input::move_to(seam_col, grip_row).send(&mut s);
    std::thread::sleep(Duration::from_millis(200));
    let hov_frame = live_frame(&mut s);
    let hov_tone = hov_frame
        .get(seam_col, grip_row)
        .expect("hovered seam cell")
        .fg;
    assert_ne!(idle_tone, hov_tone, "V1 live: hover strengthens the seam");

    let (_, rail_before) = find_pos(&mut s, "2 of 7", case_timeout(&case));
    Input::drag((seam_col, grip_row), (seam_col.saturating_sub(8), grip_row)).send(&mut s);
    std::thread::sleep(Duration::from_millis(300));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1: rail never followed the drag",
        |screen| support::screen_find(screen, "2 of 7").is_some_and(|(_, c)| c != rail_before),
    );
    let (_, rail_after) = support::screen_find(
        &s.inner
            .snapshot()
            .unwrap_or_else(|e| panic!("rail pos failed: {e:#}")),
        "2 of 7",
    )
    .expect("rail on screen");
    assert!(
        rail_after < rail_before,
        "A1 live: rail moved left {rail_before} -> {rail_after}"
    );
    checkpoint(&mut s, &dir, "dragged");
    eprintln!("s2 split_drag: isolated geometry + live hover/drag");
}

const S2_PROPS: Id = Id::root("s2.props");
const S2_PROPS_SENTINEL: Id = Id::root("s2.props.sentinel");
const S2_PROPS_AREA: Rect = Rect {
    x: 4,
    y: 1,
    width: 42,
    height: 4,
};

struct PropsApp {
    rows: [PropsRow<'static>; 3],
    st: PropsState,
    log: Rc<RefCell<Vec<PropsAction>>>,
    empty: bool,
}

impl PropsApp {
    fn rows() -> [PropsRow<'static>; 3] {
        use termrock::ItemKey;
        [
            PropsRow::new(ItemKey::index(0), "Engine", "PostgreSQL 16.3"),
            PropsRow::new(ItemKey::index(1), "Host", "prod-db-1:5432").copyable(),
            PropsRow::new(ItemKey::index(2), "Errors", "3 failed").tone(Role::Danger),
        ]
    }
}

impl App for PropsApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        let rows: &[PropsRow<'_>] = if self.empty { &[] } else { &self.rows };
        PropsList::new(S2_PROPS)
            .update(cx, &mut self.st, rows)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let rows: &[PropsRow<'_>] = if self.empty { &[] } else { &self.rows };
        PropsList::new(S2_PROPS).draw(ui, S2_PROPS_AREA, &self.st, rows);
        ui.register_control(
            S2_PROPS_SENTINEL,
            Rect::new(0, 5, 8, 1),
            Focusability::Focusable,
        );
    }
}

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_props_cursor() {
    use termrock::ItemKey;

    let dir = s2_dir("s2_props_cursor");
    let t = Theme::junie();
    let rows = PropsApp::rows();

    let idle_buf = render_isolated(50, 6, 0, None, |ui, _| {
        PropsList::new(S2_PROPS).draw(ui, Rect::new(4, 1, 42, 4), &PropsState::default(), &rows);
    });
    assert_eq!(
        idle_buf[(4, 1)].fg,
        t.color.fg[FgStep::Muted.index()],
        "V1: label muted"
    );
    assert_eq!(idle_buf[(12, 3)].fg, t.color.danger, "V1: error tone kept");
    let idle = capture_isolated(&dir, "props-idle", &idle_buf);

    let log = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        PropsApp {
            rows: PropsApp::rows(),
            st: PropsState::default(),
            log: Rc::clone(&log),
            empty: false,
        },
        Theme::junie(),
        50,
        6,
    );
    assert!(app.tab_to(S2_PROPS), "S1: one focus stop");
    let r = app.key(KeyCode::Down);
    assert_eq!(r.flow(), Flow::Consumed, "cursor moves");

    let r = app.key(KeyCode::Char('y'));
    assert_eq!(r.flow(), Flow::Consumed, "cursor on the copyable row");
    assert!(
        log.borrow()
            .iter()
            .any(|a| matches!(a, PropsAction::Copy(k) if *k == ItemKey::index(1))),
        "cursor proves row 1"
    );
    let cursor_buf = app.buffer().clone();
    assert_eq!(cursor_buf[(4, 2)].symbol(), "▎", "V1: cursor bar");

    assert!(buf_is_bold(&cursor_buf, 12, 2), "V1: cursor bold");
    let cursor_row = buf_row_text(&cursor_buf, 2);
    assert!(
        cursor_row.contains("y copy"),
        "V1: copy hint in {cursor_row:?}"
    );

    assert!(
        !buf_row_text(&cursor_buf, 1).contains("y copy"),
        "N1: hint only on cursor"
    );
    let cursor = capture_isolated(&dir, "props-cursor", &cursor_buf);
    assert_ne!(
        cursor.digest(),
        idle.digest(),
        "V1: cursor changes the frame"
    );

    let _ = app.key(KeyCode::Tab);
    assert!(
        app.state_of(S2_PROPS_SENTINEL)
            .contains(StateFlags::FOCUSED),
        "S1: rows are not stops (single group stop)"
    );
    let mut app = Harness::new(
        PropsApp {
            rows: PropsApp::rows(),
            st: PropsState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
            empty: true,
        },
        Theme::junie(),
        50,
        6,
    );
    assert!(app.tab_to(S2_PROPS_SENTINEL), "S1: empty harness focus");
    let r = app.key(KeyCode::Down);
    assert_eq!(r.flow(), Flow::Ignored, "S1: empty ignores keys");

    let log = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        PropsApp {
            rows: PropsApp::rows(),
            st: PropsState::default(),
            log: Rc::clone(&log),
            empty: false,
        },
        Theme::junie(),
        50,
        6,
    );
    assert!(app.tab_to(S2_PROPS), "A1: ring stop");
    let _ = app.key(KeyCode::Down);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('y'));
    assert_eq!(r.flow(), Flow::Consumed, "A1: y copies");
    assert!(
        log.borrow()
            .iter()
            .any(|a| matches!(a, PropsAction::Copy(k) if *k == ItemKey::index(1))),
        "A1: Copy(row 1)"
    );
    let _ = app.key(KeyCode::Up);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('y'));
    assert_eq!(r.flow(), Flow::Consumed, "A1: y consumes elsewhere");
    assert!(log.borrow().is_empty(), "A1: no copy elsewhere");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Enter on plain row");
    assert!(log.borrow().is_empty(), "A1: Enter copies nothing plain");
    let _ = app.key(KeyCode::Down);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Enter activates");
    assert!(
        log.borrow()
            .iter()
            .any(|a| matches!(a, PropsAction::Copy(k) if *k == ItemKey::index(1))),
        "A1: Enter copies the copyable row"
    );
    log.borrow_mut().clear();
    let r = app.click(6, 3);
    assert_eq!(r.flow(), Flow::Consumed, "A1: click activates");
    assert!(
        log.borrow().is_empty(),
        "A1: click on plain row copies nothing"
    );

    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('y'));
    assert_eq!(r.flow(), Flow::Consumed, "A1: click moves the cursor");
    assert!(
        log.borrow().is_empty(),
        "A1: cursor sits on the plain row 2"
    );

    let after = app.buffer().clone();
    for (y, value) in [
        (1, "PostgreSQL 16.3"),
        (2, "prod-db-1:5432"),
        (3, "3 failed"),
    ] {
        assert!(
            buf_row_text(&after, y).contains(value),
            "A1: read-only values (row {y} keeps {value:?})"
        );
    }

    let m = measure_size(|ui, c| PropsList::new(S2_PROPS).measure(ui, c));
    assert_eq!(m.preferred.1, 3, "S2: measure counts rows");

    let case = s2_case("props_cursor", &["--page", "chipsselects"], 120, 40);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let text = live_text(&mut s);
    assert!(text.contains("PostgreSQL 16.3"), "V1 live: engine value");
    assert!(text.contains("prod-db-1.acme.io"), "V1 live: host value");
    assert_line_has(&text, "Engine", "PostgreSQL", "V1 live: fact row");

    let frame = live_frame(&mut s);
    let text = live_text(&mut s);
    let (row, line) = text
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("Engine") && l.contains("PostgreSQL"))
        .expect("property line");
    let row = row as u16;
    let col = char_col(line, "Engine");
    let vcol = char_col(line, "PostgreSQL");
    let label_fg = frame.get(col, row).expect("label cell").fg;
    let value_fg = frame.get(vcol, row).expect("value cell").fg;
    assert_eq!(
        resolve_rgb(label_fg, "label"),
        theme_rgb(t.color.fg[FgStep::Muted.index()]),
        "V1 live: label muted"
    );
    assert_eq!(
        resolve_rgb(value_fg, "value"),
        theme_rgb(t.color.fg[FgStep::Primary.index()]),
        "V1 live: value in normal tone"
    );
    eprintln!("s2 props_cursor: isolated list + live property block");
}

const S2_EMPTY: Id = Id::root("s2.empty");

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_empty_narrow() {
    let dir = s2_dir("s2_empty_narrow");
    let t = Theme::junie();

    let buf = render_isolated(40, 9, 0, None, |ui, _| {
        Empty::new(
            S2_EMPTY,
            EmptyState::Empty {
                title: "No results yet",
                hint: Some("A title and one hint"),
            },
        )
        .draw(ui, Rect::new(4, 1, 32, 7));
    });
    let rows: Vec<String> = (0..9).map(|y| buf_row_text(&buf, y)).collect();
    let title_y = rows
        .iter()
        .position(|r| r.contains("No results yet"))
        .expect("title row");
    let hint_y = rows
        .iter()
        .position(|r| r.contains("A title and one hint"))
        .expect("hint row");
    assert_eq!(hint_y, title_y + 2, "V1: hint two rows below");
    assert_eq!(
        buf[(4 + 16 - 7, title_y as u16)].fg,
        t.color.fg[FgStep::Muted.index()],
        "V1: title muted"
    );
    capture_isolated(&dir, "empty-plain", &buf);
    assert_surround_intact(&buf, Rect::new(4, 1, 32, 7), "N1: nothing outside");

    let buf = render_isolated(40, 9, 0, None, |ui, _| {
        Empty::new(
            S2_EMPTY,
            EmptyState::Error {
                message: "Load failed",
                detail: None,
            },
        )
        .draw(ui, Rect::new(4, 1, 32, 7));
    });
    let rows: Vec<String> = (0..9).map(|y| buf_row_text(&buf, y)).collect();
    let ey = rows
        .iter()
        .position(|r| r.contains("Load failed"))
        .expect("error row") as u16;
    let bang_x = rows[ey as usize]
        .chars()
        .position(|c| c == '!')
        .expect("bang") as u16;
    assert!(buf_is_bold(&buf, bang_x, ey), "V1: bold bang");
    capture_isolated(&dir, "empty-error", &buf);

    let buf = render_isolated(20, 5, 0, None, |ui, _| {
        Empty::new(
            S2_EMPTY,
            EmptyState::Empty {
                title: "A very long title indeed",
                hint: Some("hint"),
            },
        )
        .draw(ui, Rect::new(4, 1, 10, 3));
    });
    let narrow: Vec<String> = (0..5).map(|y| buf_row_text(&buf, y)).collect();
    assert!(
        narrow.iter().any(|r| r.contains('…')),
        "V2: truncation in {narrow:?}"
    );
    let buf = render_isolated(40, 5, 0, None, |ui, _| {
        Empty::new(
            S2_EMPTY,
            EmptyState::Empty {
                title: "Title",
                hint: Some("hint"),
            },
        )
        .draw(ui, Rect::new(4, 1, 32, 1));
    });
    let one: Vec<String> = (0..5).map(|y| buf_row_text(&buf, y)).collect();
    assert!(one.iter().any(|r| r.contains("Title")), "V2: 1-row title");
    assert!(
        !one.iter().any(|r| r.contains("hint")),
        "V2: 1-row skips hint"
    );
    capture_isolated(&dir, "empty-narrow", &buf);

    assert!(title_y < 4, "S1: block vertically centered");

    let case = s2_case("empty_narrow", &["--page", "chipsselects"], 120, 40);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let text = live_text(&mut s);
    assert!(text.contains("No results yet"), "V1 live: empty title");
    assert!(text.contains("Empty state"), "V1 live: empty card");
    assert!(text.contains("Properties"), "V1 live: properties beside it");
    eprintln!("s2 empty_narrow: isolated variants + live empty card");
}

fn spawn_notice(
    bin: &'static str,
    slug: &str,
    cols: u16,
    rows: u16,
    needle: &'static str,
) -> (Case, Session) {
    let case = Case::dynamic(
        format!("journeys/showcase/s2/{slug}"),
        bin,
        &[],
        cols,
        rows,
        Color::Truecolor,
        needle,
    );
    let s = support::spawn_boot(&case);
    (case, s)
}

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_toosmall_notice() {
    let dir = s2_dir("s2_toosmall_notice");

    for (app, bin) in [
        ("showcase", SHOWCASE),
        ("tablepro", TABLEPRO),
        ("jackin-preview", JACKIN),
        ("holla", HOLLA),
    ] {
        let slug = format!("toosmall-notice-{app}");
        let (case, mut s) = spawn_notice(bin, &slug, 60, 10, "Terminal too small");
        if app == "showcase" {
            write_provenance(&dir, &case);
        }
        checkpoint(&mut s, &dir, &format!("notice-{app}"));
        let text = live_text(&mut s);
        assert!(text.contains("Terminal too small"), "V1: {app} notice");
        assert!(
            text.contains("Need 72×20, have 60×10"),
            "V1/S1: {app} size line"
        );
        assert!(text.contains("q Quit"), "V1: {app} quit line");

        assert!(!text.contains('›'), "N1: {app} no nav cursor");
        assert!(!text.contains("Playground"), "N1: {app} no page body");
    }

    let (_, mut s) = spawn_notice(SHOWCASE, "toosmall-boundary", 72, 20, BOOT);
    checkpoint(&mut s, &dir, "boundary");
    let text = live_text(&mut s);
    assert!(
        !text.contains("Terminal too small"),
        "V2/N1: 72x20 boots normally"
    );

    let (_, s) = spawn_notice(SHOWCASE, "toosmall-quit", 60, 10, "Terminal too small");
    support::press_step(&s, "q");
    let deadline = std::time::Instant::now() + Duration::from_millis(8_000);
    loop {
        match s.inner.observe_now() {
            Err(_) => break,
            Ok(obs) => {
                if support::screen_text(&obs.screen).trim().is_empty() {
                    break;
                }
            }
        }
        assert!(
            std::time::Instant::now() < deadline,
            "A1: q never quit the notice"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    eprintln!("s2 toosmall_notice: 4-app notice + boundary + q-quit");
}

const S2_NARROW_BTN: Id = Id::root("s2.narrow.btn");
const S2_NARROW_CHK: Id = Id::root("s2.narrow.chk");
const S2_NARROW_TGL: Id = Id::root("s2.narrow.tgl");
const S2_NARROW_RADIO: Id = Id::root("s2.narrow.radio");
const S2_NARROW_SPLIT: Id = Id::root("s2.narrow.split");
const S2_NARROW_PROPS: Id = Id::root("s2.narrow.props");
const S2_NARROW_PANEL: Id = Id::root("s2.narrow.panel");

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_narrow_clip() {
    let dir = s2_dir("s2_narrow_clip");

    for width in [6u16, 4, 3, 2, 1] {
        let used = Cell::new(Rect::default());
        let buf = render_isolated(16, 3, 0, None, |ui, _| {
            used.set(
                Button::new(S2_NARROW_BTN, "Run task")
                    .variant(Variant::SECONDARY)
                    .draw(ui, Rect::new(4, 1, width, 1)),
            );
        });
        assert_eq!(
            used.get().width,
            10u16.min(width),
            "S1: button clips at {width}"
        );
        assert_surround_intact(&buf, Rect::new(4, 1, used.get().width, 1), "V1 button");
        let brow = buf_row_text(&buf, 1);
        assert!(
            !brow.contains('●') && !brow.contains('○') && !SPINNER.iter().any(|g| brow.contains(g)),
            "A1: button flags (ready, plain)"
        );
        if (3..=6).contains(&width) {
            assert!(
                buf_row_text(&buf, 1).contains('…'),
                "V2: button truncates at {width}"
            );
        } else {
            assert!(
                !buf_row_text(&buf, 1).contains("Run"),
                "V2: button clips the label at {width}"
            );
        }

        let checked = true;
        let buf = render_isolated(16, 3, 0, None, |ui, _| {
            Checkbox::new(S2_NARROW_CHK, "Run tests")
                .checked(checked)
                .draw(ui, Rect::new(4, 1, width, 1));
        });
        assert_surround_intact(&buf, Rect::new(4, 1, width, 1), "V1 checkbox");
        assert!(checked, "A1: checkbox flag");
        let on = true;
        let buf = render_isolated(16, 3, 0, None, |ui, _| {
            Toggle::new(S2_NARROW_TGL, "Auto-approve")
                .on(on)
                .draw(ui, Rect::new(4, 1, width, 1));
        });
        assert_surround_intact(&buf, Rect::new(4, 1, width, 1), "V1 toggle");
        assert!(on, "A1: toggle flag");
        let items = ["Fast", "Balanced"];
        let buf = render_isolated(16, 5, 0, None, |ui, _| {
            RadioGroup::new(S2_NARROW_RADIO)
                .value(termrock::ItemKey::index(0))
                .draw(
                    ui,
                    Rect::new(4, 1, width, 3),
                    &RadioGroupState::default(),
                    &items,
                );
        });
        assert_surround_intact(&buf, Rect::new(4, 1, width, 3), "V1 radio");

        let buf = render_isolated(16, 3, 0, None, |ui, _| {
            SplitPane::new(S2_NARROW_SPLIT, SplitAxis::Horizontal)
                .gap(1)
                .min_first(1)
                .min_second(1)
                .draw(
                    ui,
                    Rect::new(4, 1, 1, 1),
                    &SplitPaneState::new(50),
                    |_, _, _| {},
                );
        });
        assert_surround_intact(&buf, Rect::new(4, 1, 1, 1), "V1 splitter");
    }

    let buf = render_isolated(16, 3, 0, None, |ui, _| {
        Checkbox::new(S2_NARROW_CHK, "Run tests")
            .checked(false)
            .draw(ui, Rect::new(4, 1, 3, 1));
    });
    let row = buf_row_text(&buf, 1);
    assert!(
        row.contains('□') && !row.contains('['),
        "V2: compact □ in {row:?}"
    );
    capture_isolated(&dir, "narrow-compact", &buf);

    let buf = render_isolated(16, 5, 0, None, |ui, _| {
        Panel::new(S2_NARROW_PANEL)
            .kind(PanelKind::Framed)
            .title("Framed")
            .draw(ui, Rect::new(4, 1, 4, 3), |_, _| {});
    });
    let brow = buf_row_text(&buf, 1);
    assert!(
        brow.contains('╭') && !brow.contains("Framed"),
        "V2: border-only in {brow:?}"
    );
    capture_isolated(&dir, "narrow-framed", &buf);

    use termrock::ItemKey;
    let rows = [PropsRow::new(
        ItemKey::index(0),
        "Environment",
        "production",
    )];
    let buf = render_isolated(30, 5, 0, None, |ui, _| {
        PropsList::new(S2_NARROW_PROPS).draw(
            ui,
            Rect::new(4, 1, 6, 2),
            &PropsState::default(),
            &rows,
        );
    });
    let spilled =
        (1..3).any(|y| (0..30).any(|x| !(4..10).contains(&x) && buf[(x, y)].symbol() != "·"));
    assert!(spilled, "S1/N1: props spill pinned below label width");
    capture_isolated(&dir, "narrow-props-spill", &buf);

    let buf = render_isolated(20, 5, 0, None, |ui, _| {
        Empty::new(
            S2_EMPTY,
            EmptyState::Empty {
                title: "A very long title indeed",
                hint: None,
            },
        )
        .draw(ui, Rect::new(4, 1, 10, 3));
    });
    assert!(
        (0..5).any(|y| buf_row_text(&buf, y).contains('…')),
        "V2: empty truncates"
    );
    eprintln!("s2 narrow_clip: 6/4/3/2/1 sweep + pinned spills");
}

const S2_HOV_BTN: Id = Id::root("s2.hov.btn");
const S2_HOV_CHK: Id = Id::root("s2.hov.chk");
const S2_HOV_TGL: Id = Id::root("s2.hov.tgl");
const S2_HOV_RADIO: Id = Id::root("s2.hov.radio");
const S2_HOV_BRAND: Id = Id::root("s2.hov.brand");
const S2_HOV_SPLIT: Id = Id::root("s2.hov.split");
const S2_HOV_PROPS: Id = Id::root("s2.hov.props");
const S2_HOV_SCROLL: Id = Id::root("s2.hov.scroll");

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_hover_coverage() {
    let dir = s2_dir("s2_hover_coverage");

    let btn_idle_buf = render_isolated(30, 3, 0, None, |ui, _| {
        Button::new(S2_HOV_BTN, "Run task")
            .variant(Variant::SECONDARY)
            .draw(ui, Rect::new(4, 1, 16, 1));
    });
    let btn_idle = capture_isolated(&dir, "hov-btn-idle", &btn_idle_buf).digest();
    let btn_hov_buf = render_isolated(
        30,
        3,
        0,
        Some((S2_HOV_BTN, ReferenceState::HOVERED)),
        |ui, _| {
            Button::new(S2_HOV_BTN, "Run task")
                .variant(Variant::SECONDARY)
                .draw(ui, Rect::new(4, 1, 16, 1));
        },
    );
    assert_eq!(
        btn_hov_buf[(4, 1)].symbol(),
        " ",
        "V2: hover alone never bars"
    );
    let btn_hov = capture_isolated(&dir, "hov-btn-hover", &btn_hov_buf).digest();
    assert_ne!(btn_idle, btn_hov, "V1: button hover delta");

    let chk_idle = capture_isolated(
        &dir,
        "hov-chk-idle",
        &render_isolated(30, 3, 0, None, |ui, _| {
            Checkbox::new(S2_HOV_CHK, "Run tests")
                .checked(false)
                .draw(ui, Rect::new(4, 1, 20, 1));
        }),
    )
    .digest();
    let chk_hov_buf = render_isolated(
        30,
        3,
        0,
        Some((S2_HOV_CHK, ReferenceState::HOVERED)),
        |ui, _| {
            Checkbox::new(S2_HOV_CHK, "Run tests")
                .checked(false)
                .draw(ui, Rect::new(4, 1, 20, 1));
        },
    );
    assert_eq!(
        chk_hov_buf[(4, 1)].symbol(),
        " ",
        "V2: checkbox hover never bars"
    );
    assert_ne!(
        chk_idle,
        capture_isolated(&dir, "hov-chk-hover", &chk_hov_buf).digest(),
        "V1: checkbox delta"
    );

    let tgl_idle = capture_isolated(
        &dir,
        "hov-tgl-idle",
        &render_isolated(30, 3, 0, None, |ui, _| {
            Toggle::new(S2_HOV_TGL, "Auto-approve")
                .on(false)
                .draw(ui, Rect::new(4, 1, 22, 1));
        }),
    )
    .digest();
    let tgl_hov_buf = render_isolated(
        30,
        3,
        0,
        Some((S2_HOV_TGL, ReferenceState::HOVERED)),
        |ui, _| {
            Toggle::new(S2_HOV_TGL, "Auto-approve")
                .on(false)
                .draw(ui, Rect::new(4, 1, 22, 1));
        },
    );
    assert_ne!(
        tgl_idle,
        capture_isolated(&dir, "hov-tgl-hover", &tgl_hov_buf).digest(),
        "V1: toggle delta"
    );

    use termrock::ItemKey;
    let items = ["Fast", "Balanced"];
    let radio_idle = capture_isolated(
        &dir,
        "hov-radio-idle",
        &render_isolated(30, 6, 0, None, |ui, _| {
            RadioGroup::new(S2_HOV_RADIO).draw(
                ui,
                Rect::new(4, 1, 20, 3),
                &RadioGroupState::default(),
                &items,
            );
        }),
    )
    .digest();
    let radio_hov_buf = render_isolated(30, 6, 0, None, |ui, _| {
        ui.reference(
            Some(
                ReferenceTarget::new(S2_HOV_RADIO, ReferenceState::HOVERED)
                    .part(PartRef::item(Part::ROW, ItemKey::index(1))),
            ),
            |ui| {
                RadioGroup::new(S2_HOV_RADIO).draw(
                    ui,
                    Rect::new(4, 1, 20, 3),
                    &RadioGroupState::default(),
                    &items,
                );
            },
        );
    });
    assert_eq!(
        radio_hov_buf[(4, 1)].symbol(),
        " ",
        "V2: radio hover never bars"
    );
    assert_ne!(
        radio_hov_buf[(6, 2)].bg,
        radio_hov_buf[(6, 1)].bg,
        "V1: exactly one option lifts"
    );
    assert_ne!(
        radio_idle,
        capture_isolated(&dir, "hov-radio-hover", &radio_hov_buf).digest(),
        "V1: radio delta"
    );

    let brand_idle = capture_isolated(
        &dir,
        "hov-brand-idle",
        &render_isolated(16, 3, 0, None, |ui, _| {
            Brand::new(S2_HOV_BRAND, "holla❯")
                .clickable(true)
                .draw(ui, Rect::new(4, 1, 8, 1));
        }),
    )
    .digest();
    let brand_hov_buf = render_isolated(
        16,
        3,
        0,
        Some((S2_HOV_BRAND, ReferenceState::HOVERED)),
        |ui, _| {
            Brand::new(S2_HOV_BRAND, "holla❯")
                .clickable(true)
                .draw(ui, Rect::new(4, 1, 8, 1));
        },
    );
    assert_ne!(
        brand_idle,
        capture_isolated(&dir, "hov-brand-hover", &brand_hov_buf).digest(),
        "V1: brand delta"
    );

    let strip = Rect::new(0, 0, 16, 6);
    let split_idle = capture_isolated(
        &dir,
        "hov-split-idle",
        &render_isolated(16, 6, 0, None, |ui, _| {
            SplitPane::new(S2_HOV_SPLIT, SplitAxis::Horizontal)
                .gap(1)
                .draw(ui, strip, &SplitPaneState::new(50), |_, _, _| {});
        }),
    )
    .digest();
    let split_hov_buf = render_isolated(
        16,
        6,
        0,
        Some((S2_HOV_SPLIT, ReferenceState::HOVERED)),
        |ui, _| {
            SplitPane::new(S2_HOV_SPLIT, SplitAxis::Horizontal)
                .gap(1)
                .draw(ui, strip, &SplitPaneState::new(50), |_, _, _| {});
        },
    );
    assert_ne!(
        split_idle,
        capture_isolated(&dir, "hov-split-hover", &split_hov_buf).digest(),
        "V1: splitter delta"
    );

    let prows = [
        PropsRow::new(ItemKey::index(0), "Engine", "pg"),
        PropsRow::new(ItemKey::index(1), "Host", "db"),
    ];
    let props_idle = capture_isolated(
        &dir,
        "hov-props-idle",
        &render_isolated(40, 5, 0, None, |ui, _| {
            PropsList::new(S2_HOV_PROPS).draw(
                ui,
                Rect::new(4, 1, 32, 3),
                &PropsState::default(),
                &prows,
            );
        }),
    )
    .digest();
    let props_hov_buf = render_isolated(40, 5, 0, None, |ui, _| {
        ui.reference(
            Some(
                ReferenceTarget::new(S2_HOV_PROPS, ReferenceState::HOVERED)
                    .part(PartRef::item(Part::ROW, ItemKey::index(0))),
            ),
            |ui| {
                PropsList::new(S2_HOV_PROPS).draw(
                    ui,
                    Rect::new(4, 1, 32, 3),
                    &PropsState::default(),
                    &prows,
                );
            },
        );
    });
    assert_ne!(
        props_hov_buf[(6, 1)].bg,
        props_hov_buf[(6, 2)].bg,
        "V1: exactly one props row lifts"
    );
    assert_ne!(
        props_idle,
        capture_isolated(&dir, "hov-props-hover", &props_hov_buf).digest(),
        "V1: props delta"
    );

    let vlines: Vec<String> = ["a".to_string(), "b".to_string()].to_vec();
    let scroll_idle = capture_isolated(
        &dir,
        "hov-scroll-idle",
        &render_isolated(24, 5, 0, None, |ui, _| {
            let lines: Vec<ViewportLine<'_>> =
                vlines.iter().map(|l| ViewportLine::Plain(l)).collect();
            TextViewport::new(S2_HOV_SCROLL).draw(
                ui,
                Rect::new(4, 1, 16, 3),
                &ViewportState::default(),
                &lines,
            );
        }),
    )
    .digest();
    let scroll_hov_buf = render_isolated(
        24,
        5,
        0,
        Some((S2_HOV_SCROLL, ReferenceState::HOVERED)),
        |ui, _| {
            let lines: Vec<ViewportLine<'_>> =
                vlines.iter().map(|l| ViewportLine::Plain(l)).collect();
            TextViewport::new(S2_HOV_SCROLL).draw(
                ui,
                Rect::new(4, 1, 16, 3),
                &ViewportState::default(),
                &lines,
            );
        },
    );
    assert_eq!(
        scroll_idle,
        capture_isolated(&dir, "hov-scroll-hover", &scroll_hov_buf).digest(),
        "S1: scroll ignores hover"
    );

    let dis_idle = capture_isolated(
        &dir,
        "hov-dis-idle",
        &render_isolated(30, 3, 0, None, |ui, _| {
            Button::new(S2_HOV_BTN, "Run task")
                .variant(Variant::SECONDARY)
                .disabled(true)
                .draw(ui, Rect::new(4, 1, 16, 1));
        }),
    )
    .digest();
    let dis_hov_buf = render_isolated(
        30,
        3,
        0,
        Some((S2_HOV_BTN, ReferenceState::HOVERED)),
        |ui, _| {
            Button::new(S2_HOV_BTN, "Run task")
                .variant(Variant::SECONDARY)
                .disabled(true)
                .draw(ui, Rect::new(4, 1, 16, 1));
        },
    );
    assert_eq!(
        dis_idle,
        capture_isolated(&dir, "hov-dis-hover", &dis_hov_buf).digest(),
        "N1: disabled ignores hover"
    );

    let mut app = Harness::new(HoverButtonApp, Theme::junie(), 40, 8);
    let plain = app.buffer().clone();
    let _ = app.mouse(MouseKind::Move, 6, 1);
    let lifted = app.buffer().clone();
    assert_ne!(lifted, plain, "N1: hover lifts first");
    let r = app.key(KeyCode::Char('x'));
    assert_eq!(r.flow(), Flow::Ignored, "N1: suppression key ignores");
    assert_eq!(app.buffer(), &plain, "N1: suppressed pointer ignores hover");
    capture_isolated(&dir, "hov-suppressed", app.buffer());

    let value = Rc::new(Cell::new(true));
    let fired = Rc::new(Cell::new(0));
    struct HoverToggleApp {
        value: Rc<Cell<bool>>,
        fired: Rc<Cell<u32>>,
    }
    impl App for HoverToggleApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let mut v = self.value.get();
            let fired = Rc::clone(&self.fired);
            let r = Toggle::new(S2_HOV_TGL, "Verbose")
                .on(v)
                .update(cx, &mut v)
                .on_activated(|| fired.set(fired.get() + 1));
            self.value.set(v);
            r
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            Toggle::new(S2_HOV_TGL, "Verbose")
                .on(self.value.get())
                .draw(ui, Rect::new(0, 0, 16, 1));
        }
    }
    let mut app = Harness::new(
        HoverToggleApp {
            value: Rc::clone(&value),
            fired: Rc::clone(&fired),
        },
        Theme::junie(),
        40,
        8,
    );
    assert!(app.tab_to(S2_HOV_TGL), "A1: ring stop");
    let _ = app.mouse(MouseKind::Move, 2, 0);
    let r = app.key(KeyCode::Char('x'));
    assert_eq!(r.flow(), Flow::Ignored, "A1: stray key ignores");
    assert!(value.get(), "A1: flags untouched");
    assert_eq!(fired.get(), 0, "A1: hover never fires");

    let case = s2_case("hover_coverage", &["--page", "buttons"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let (brow, bcol) = find_pos(&mut s, "Run task", case_timeout(&case));
    let before = live_frame(&mut s);
    let before_bg = before.get(bcol, brow).expect("label cell").bg;
    hover_over(&mut s, "Run task", case_timeout(&case));
    let after = live_frame(&mut s);
    let after_bg = after.get(bcol, brow).expect("hovered label cell").bg;
    assert_ne!(before_bg, after_bg, "V1 live: hover lifts the button row");
    let hov_text = live_text(&mut s);
    assert_line_lacks(
        &hov_text,
        "▎",
        "Run task",
        "V2 live: hover alone never bars",
    );

    let (drow, dcol) = find_pos(&mut s, "Disabled primary", case_timeout(&case));
    let row_cells = |f: &Frame| {
        (dcol..dcol + 16)
            .map(|c| {
                let cell = f.get(c, drow).expect("disabled row cell");
                (cell.symbol.clone(), cell.fg, cell.bg)
            })
            .collect::<Vec<_>>()
    };
    let dis_before = row_cells(&live_frame(&mut s));
    hover_over(&mut s, "Disabled primary", case_timeout(&case));
    let dis_after = row_cells(&live_frame(&mut s));
    assert_eq!(dis_before, dis_after, "N1 live: disabled ignores hover");
    checkpoint(&mut s, &dir, "hovered");
    eprintln!("s2 hover_coverage: isolated deltas + live lift/no-lift");
}

const S2_FLD: Id = Id::root("s2.fld");

fn email_ok(s: &str) -> Result<(), FieldError> {
    if s.contains('@') && s.contains('.') {
        Ok(())
    } else {
        Err(FieldError::new("bad"))
    }
}

fn email_full(s: &str) -> Result<(), FieldError> {
    if s.contains('@') && s.contains('.') {
        Ok(())
    } else {
        Err(FieldError::new("Enter a valid email"))
    }
}

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_field_chrome() {
    let dir = s2_dir("s2_field_chrome");
    let t = Theme::junie();

    let buf = render_isolated(40, 5, 0, None, |ui, _| {
        Field::new(
            "Project name",
            TextInput::new(S2_FLD).value("payments-gateway"),
        )
        .required(true)
        .help("Used as the working directory name")
        .draw(ui, Rect::new(4, 1, 32, 3), &TextInputState::default());
    });
    let label_row = buf_row_text(&buf, 1);
    assert!(
        label_row.contains("Project name *"),
        "V1: required label in {label_row:?}"
    );
    assert_eq!(
        buf[(4 + 2 + 12 + 1, 1)].fg,
        t.color.accent,
        "V1: accent star"
    );

    let buf = render_isolated(40, 5, 0, None, |ui, _| {
        Field::new("Branch", TextInput::new(S2_FLD).placeholder("feat/…"))
            .optional_suffix(true)
            .draw(ui, Rect::new(4, 1, 32, 3), &TextInputState::default());
    });
    assert!(
        buf_row_text(&buf, 1).contains("Branch  optional"),
        "V1: optional suffix"
    );
    let buf = render_isolated(20, 5, 0, None, |ui, _| {
        Field::new("Branch", TextInput::new(S2_FLD).placeholder("feat/…"))
            .optional_suffix(true)
            .draw(ui, Rect::new(4, 1, 10, 3), &TextInputState::default());
    });
    assert!(
        !buf_row_text(&buf, 1).contains("optional"),
        "V1: suffix clipped when narrow"
    );

    let buf = render_isolated(
        40,
        5,
        0,
        Some((S2_FLD, ReferenceState::FOCUSED)),
        |ui, _| {
            Field::new("Project name", TextInput::new(S2_FLD).value("x")).draw(
                ui,
                Rect::new(4, 1, 32, 3),
                &TextInputState::default(),
            );
        },
    );
    assert_eq!(buf[(4, 2)].symbol(), "▎", "V2: focused gutter bar");
    let buf = render_isolated(40, 5, 0, None, |ui, _| {
        Field::new("Project name", TextInput::new(S2_FLD).value("x")).draw(
            ui,
            Rect::new(4, 1, 32, 3),
            &TextInputState::default(),
        );
    });
    assert_eq!(buf[(4, 2)].symbol(), " ", "V2/N1: unfocused gutter blank");

    let mut st = TextInputState::default();
    let mut val = String::new();
    st.begin("mira@example");
    assert!(st.commit(&mut val, &email_ok).is_err(), "fixture invalid");
    assert!(st.error().is_some(), "error fixture invalid");
    let buf = render_isolated(40, 5, 0, None, |ui, _| {
        Field::new("Owner email", TextInput::new(S2_FLD).value(&val))
            .required(true)
            .error(st.error().map(|e| e.message.as_ref()))
            .draw(ui, Rect::new(4, 1, 32, 3), &st);
    });
    assert_eq!(buf[(4 + 32 - 2, 2)].symbol(), "!", "V2: error marker");
    assert!(buf_is_bold(&buf, 4 + 32 - 2, 2), "V2: marker bold");
    capture_isolated(&dir, "fld-chrome", &buf);

    let buf = render_isolated(40, 5, 0, None, |ui, _| {
        Field::new("Branch", TextInput::new(S2_FLD).placeholder("feat/…")).draw(
            ui,
            Rect::new(4, 1, 32, 3),
            &TextInputState::default(),
        );
    });
    assert!(
        buf_row_text(&buf, 2).contains("feat/…"),
        "S2: placeholder when empty"
    );
    let mut st = TextInputState::default();
    st.begin("");
    let buf = render_isolated(
        40,
        5,
        0,
        Some((S2_FLD, ReferenceState::FOCUSED)),
        |ui, _| {
            Field::new("Branch", TextInput::new(S2_FLD).placeholder("feat/…")).draw(
                ui,
                Rect::new(4, 1, 32, 3),
                &st,
            );
        },
    );
    assert!(
        !buf_row_text(&buf, 2).contains("feat/…"),
        "S2: no placeholder while editing"
    );

    let log = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        ChromeNavApp {
            value: String::new(),
            st: TextInputState::default(),
            log: Rc::clone(&log),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S2_FLD), "A2: navigating focus");
    let r = app.key(KeyCode::Char('x'));
    assert_eq!(r.flow(), Flow::Ignored, "A2: typing nav ignores");
    assert!(log.borrow().is_empty(), "A2: no event");

    let mut st = TextInputState::default();
    let mut val = String::new();
    st.begin("bad");
    assert!(
        st.commit(&mut val, &email_full).is_err(),
        "N2 fixture invalid"
    );
    let buf = render_isolated(40, 5, 0, None, |ui, _| {
        Field::new("Owner email", TextInput::new(S2_FLD).value(&val))
            .help("some help")
            .error(st.error().map(|e| e.message.as_ref()))
            .draw(ui, Rect::new(4, 1, 32, 3), &st);
    });
    let msg = buf_row_text(&buf, 3);
    assert!(
        msg.contains("Enter a valid email"),
        "N2: error wins in {msg:?}"
    );
    assert!(!msg.contains("some help"), "N2: help hidden in {msg:?}");

    let case = s2_case("field_chrome", &["--page", "inputs"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(boot.contains("Project name *"), "boot required label");
    assert!(boot.contains("Branch  optional"), "boot optional suffix");

    for (i, needle) in [
        "▎ payments-gateway",
        "▎ feat/…",
        "▎ mira@example",
        "▎ Type a path or symbol",
        "▎ ••••",
    ]
    .iter()
    .enumerate()
    {
        support::press_step(&s, "tab");
        support::wait_screen(
            &mut s,
            case_timeout(&case),
            &format!("A1: gutter never reached stop {i} ({needle})"),
            |screen| support::screen_text(screen).contains(needle),
        );
    }

    let tabbed = live_text(&mut s);
    assert!(!tabbed.contains("EDIT"), "A1 live: tabbing never edits");

    support::press_step(&s, "x");
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(live_text(&mut s), tabbed, "A2 live: typing nav is ignored");

    let help_line = tabbed
        .lines()
        .find(|l| l.contains("working di"))
        .expect("help row");
    assert!(!help_line.contains("valid email"), "N2 live: help row pure");
    checkpoint(&mut s, &dir, "tabbed");
    eprintln!("s2 field_chrome: isolated chrome + live tab walk");
}

struct ChromeNavApp {
    value: String,
    st: TextInputState,
    log: Rc<RefCell<Vec<TextAction>>>,
}

impl App for ChromeNavApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        TextInput::new(S2_FLD)
            .update(cx, &mut self.st, &mut self.value)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Field::new("Branch", TextInput::new(S2_FLD).value(&self.value)).draw(
            ui,
            Rect::new(4, 1, 32, 3),
            &self.st,
        );
    }
}

struct HoverButtonApp;

impl App for HoverButtonApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Button::new(S2_HOV_BTN, "Run task")
            .variant(Variant::SECONDARY)
            .update(cx)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(S2_HOV_BTN, "Run task")
            .variant(Variant::SECONDARY)
            .draw(ui, Rect::new(4, 1, 16, 1));
    }
}

const S2_FLDDIS: Id = Id::root("s2.flddis");
const S2_FLDDIS_SENT: Id = Id::root("s2.flddis.sent");
const S2_TRANSCRIPT: Id = Id::root("s2.transcript");

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_field_disabled() {
    let dir = s2_dir("s2_field_disabled");
    let t = Theme::junie();

    let buf = render_isolated(44, 5, 0, None, |ui, _| {
        Field::new(
            "API token",
            TextInput::new(S2_FLDDIS)
                .value("jb_live_••••••••••••")
                .disabled(true),
        )
        .help("Managed by the organization")
        .draw(ui, Rect::new(4, 1, 36, 3), &TextInputState::default());
    });
    assert_eq!(buf[(4, 2)].symbol(), " ", "V2/N1: gutter blank");
    assert_eq!(buf[(6, 1)].fg, t.color.disabled_fg, "V1: faint label");
    capture_isolated(&dir, "fld-disabled", &buf);

    let log = Rc::new(RefCell::new(Vec::new()));
    let fired = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        DisabledInputApp {
            value: "v".to_string(),
            st: TextInputState::default(),
            log: Rc::clone(&log),
            fired: Rc::clone(&fired),
        },
        Theme::junie(),
        44,
        6,
    );
    assert!(!app.tab_to(S2_FLDDIS), "S1: no ring stop");
    assert!(app.tab_to(S2_FLDDIS_SENT), "S1: sentinel reachable");
    let before = app.buffer().clone();
    let r = app.click(8, 2);
    assert_eq!(r.flow(), Flow::Consumed, "S1/A1: click consumes");
    assert_eq!(app.buffer(), &before, "A1: click never edits");
    assert!(log.borrow().is_empty(), "A1: click reports nothing");

    let dis_idle = capture_isolated(
        &dir,
        "fld-dis-idle",
        &render_isolated(44, 5, 0, None, |ui, _| {
            Field::new(
                "API token",
                TextInput::new(S2_FLDDIS).value("v").disabled(true),
            )
            .draw(ui, Rect::new(4, 1, 36, 3), &TextInputState::default());
        }),
    )
    .digest();
    let dis_hov = capture_isolated(
        &dir,
        "fld-dis-hover",
        &render_isolated(
            44,
            5,
            0,
            Some((S2_FLDDIS, ReferenceState::HOVERED)),
            |ui, _| {
                Field::new(
                    "API token",
                    TextInput::new(S2_FLDDIS).value("v").disabled(true),
                )
                .draw(ui, Rect::new(4, 1, 36, 3), &TextInputState::default());
            },
        ),
    )
    .digest();
    assert_eq!(dis_idle, dis_hov, "S2: hover forced off");

    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "S1: Enter reaches the ring");
    assert_eq!(fired.get(), 1, "S1: sentinel proves arrival");
    assert!(log.borrow().is_empty(), "S1: Enter ignores the field");
    let r = app.key(KeyCode::Char('x'));
    assert_eq!(r.flow(), Flow::Ignored, "S1: keys ignore");
    assert!(
        log.borrow().is_empty(),
        "S1: begin_edit no-op + keys ignore"
    );

    let mut app = Harness::new(
        DisabledAreaApp {
            value: "v".to_string(),
            st: TextAreaState::default(),
        },
        Theme::junie(),
        44,
        8,
    );
    assert!(!app.tab_to(S2_TRANSCRIPT), "A2: area has no stop");
    assert!(app.tab_to(S2_FLDDIS_SENT), "A2: sentinel reachable");
    let r = app.key(KeyCode::Char('x'));
    assert_eq!(r.flow(), Flow::Ignored, "A2: disabled area ignores keys");

    let buf = render_isolated(44, 5, 0, None, |ui, _| {
        Field::new(
            "API token",
            TextInput::new(S2_FLDDIS)
                .value("jb_live_••••••••••••")
                .disabled(true),
        )
        .draw(ui, Rect::new(4, 1, 36, 3), &TextInputState::default());
    });
    assert!(
        buf_row_text(&buf, 2).contains("jb_live_••••"),
        "N2: literal bullets"
    );

    let case = s2_case("field_disabled", &["--page", "inputs"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    assert!(
        live_text(&mut s).contains("jb_live_"),
        "boot shows the token"
    );

    let before = live_text(&mut s);
    assert_line_has(&before, "▎", "›", "A1 presence: nav holds focus at boot");
    click_at(&mut s, "jb_live_", case_timeout(&case));
    std::thread::sleep(Duration::from_millis(400));
    let after_click = live_text(&mut s);

    assert!(!after_click.contains("EDIT"), "A1 live: click never edits");
    assert_eq!(after_click, before, "A1 live: click changes nothing");
    assert_line_has(&after_click, "▎", "›", "A1 live: focus stays on nav");
    assert_line_lacks(&after_click, "▎", "jb_live_", "A1 live: no bar on token");

    for _ in 0..5 {
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(150));
        let text = live_text(&mut s);
        assert!(
            !text.contains("▎ jb_live_"),
            "N1 live: bar never on token\n{text}"
        );
    }
    checkpoint(&mut s, &dir, "skipped");
    eprintln!("s2 field_disabled: isolated guards + live refuse/skip");
}

struct DisabledInputApp {
    value: String,
    st: TextInputState,
    log: Rc<RefCell<Vec<TextAction>>>,
    fired: Rc<Cell<u32>>,
}

impl App for DisabledInputApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        let fired = Rc::clone(&self.fired);
        let a = TextInput::new(S2_FLDDIS)
            .disabled(true)
            .update(cx, &mut self.st, &mut self.value)
            .on_action(|action| log.borrow_mut().push(action));
        let b = Button::new(S2_FLDDIS_SENT, "Next")
            .update(cx)
            .on_activated(|| fired.set(fired.get() + 1));
        a | b
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Field::new(
            "API token",
            TextInput::new(S2_FLDDIS).value(&self.value).disabled(true),
        )
        .draw(ui, Rect::new(4, 1, 36, 3), &self.st);
        Button::new(S2_FLDDIS_SENT, "Next").draw(ui, Rect::new(4, 5, 10, 1));
    }
}

struct DisabledAreaApp {
    value: String,
    st: TextAreaState,
}

impl App for DisabledAreaApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let a = TextArea::new(S2_TRANSCRIPT, 4)
            .disabled(true)
            .update(cx, &mut self.st, &mut self.value)
            .erase();
        let b = Button::new(S2_FLDDIS_SENT, "Next").update(cx).erase();
        a | b
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Field::new(
            "Read-only transcript",
            TextArea::new(S2_TRANSCRIPT, 4)
                .value(&self.value)
                .disabled(true),
        )
        .draw(ui, Rect::new(4, 1, 36, 6), &self.st);
        Button::new(S2_FLDDIS_SENT, "Next").draw(ui, Rect::new(4, 7, 10, 1));
    }
}

const S2_FLDLOSS: Id = Id::root("s2.fldloss");
const S2_FLDLOSS_OTHER: Id = Id::root("s2.fldloss.other");

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_field_focusloss() {
    let dir = s2_dir("s2_field_focusloss");

    let log = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        LossApp {
            value: String::new(),
            other_value: String::new(),
            st: TextInputState::default(),
            other_st: TextInputState::default(),
            log: Rc::clone(&log),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S2_FLDLOSS), "navigating focus");
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "Enter begins editing");
    assert!(log.borrow().is_empty(), "begin reports no event");
    let _ = app.type_str("billing");
    let staged = app.buffer().clone();
    assert!(
        buf_row_text(&staged, 1).contains("billing"),
        "typed text staged"
    );
    assert!(buf_is_underlined(&staged, 6, 1), "editing underlined");
    assert!(app.cursor().is_some(), "cursor placed while editing");

    log.borrow_mut().clear();
    assert!(app.tab_to(S2_FLDLOSS_OTHER), "A1: Tab moves focus");
    assert!(
        log.borrow().contains(&TextAction::Committed),
        "A1: Tab commits"
    );

    let committed = app.buffer().clone();
    assert!(
        buf_row_text(&committed, 1).contains("billing"),
        "N1: defocus never cancels"
    );
    assert!(
        !buf_is_underlined(&committed, 6, 1),
        "S1/V1: committed text plain"
    );
    capture_isolated(&dir, "fld-defocused", &committed);

    assert!(app.tab_to(S2_FLDLOSS), "back to the field");
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Enter);
    let _ = app.type_str("!");
    assert!(
        buf_row_text(app.buffer(), 1).contains("billing!"),
        "N1: committed text kept"
    );
    let _ = app.key(KeyCode::Enter);

    assert!(app.tab_to(S2_FLDLOSS_OTHER), "focus the other field");
    assert_eq!(app.buffer()[(4, 3)].symbol(), "▎", "V2: new field barred");

    log.borrow_mut().clear();
    let r = app.click(8, 1);
    assert_eq!(r.flow(), Flow::Consumed, "A2: click begins the field");
    assert!(
        buf_is_underlined(app.buffer(), 6, 1),
        "A2: editing under way"
    );
    assert!(app.cursor().is_some(), "A2: cursor placed at the pointer");

    let case = s2_case("field_focusloss", &["--page", "inputs"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    for _ in 0..4 {
        support::press_step(&s, "tab");
    }
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Search files never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Type a path or symbol"))
        },
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "editing Search files");
    type_text(&mut s, "billing");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "typed text never rendered",
        |screen| support::screen_text(screen).contains("billing"),
    );

    let (branch_row, _) = click_at(&mut s, "feat/…", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "V2: new field never barred",
        |screen| {
            support::screen_text(screen)
                .lines()
                .nth(branch_row as usize)
                .is_some_and(|l| l.contains('▎'))
        },
    );
    let text = live_text(&mut s);
    let billing_line = text
        .lines()
        .find(|l| l.contains("billing"))
        .expect("committed text")
        .to_string();
    assert!(!billing_line.contains('▎'), "V1 live: committed plainly");
    assert!(
        !billing_line.contains("Type a path"),
        "N1 live: not reverted"
    );
    checkpoint(&mut s, &dir, "defocus-committed");
    eprintln!("s2 field_focusloss: isolated commit + live click-away");
}

struct LossApp {
    value: String,
    other_value: String,
    st: TextInputState,
    other_st: TextInputState,
    log: Rc<RefCell<Vec<TextAction>>>,
}

impl App for LossApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        let a = TextInput::new(S2_FLDLOSS)
            .update(cx, &mut self.st, &mut self.value)
            .on_action(|action| log.borrow_mut().push(action));
        let log = Rc::clone(&self.log);
        let b = TextInput::new(S2_FLDLOSS_OTHER)
            .update(cx, &mut self.other_st, &mut self.other_value)
            .on_action(|action| log.borrow_mut().push(action));
        a | b
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(S2_FLDLOSS)
            .value(&self.value)
            .placeholder("Type a path or symbol…")
            .draw(ui, Rect::new(4, 1, 36, 1), &self.st);
        TextInput::new(S2_FLDLOSS_OTHER)
            .value(&self.other_value)
            .placeholder("feat/…")
            .draw(ui, Rect::new(4, 3, 36, 1), &self.other_st);
    }
}

const S2_TICOMMIT: Id = Id::root("s2.ticommit");
const S2_TICOMMIT_SENT: Id = Id::root("s2.ticommit.sent");

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_input_commit() {
    let dir = s2_dir("s2_input_commit");

    let log = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        CommitApp {
            value: String::new(),
            st: TextInputState::default(),
            log: Rc::clone(&log),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S2_TICOMMIT), "navigating focus");
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Enter begins");
    assert!(log.borrow().is_empty(), "A1: begin reports no event");
    assert!(app.cursor().is_some(), "A1: editing under way");
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Consumed, "reset: Esc cancels");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('w'));
    assert_eq!(r.flow(), Flow::Ignored, "N1: typing nav never edits");
    assert!(log.borrow().is_empty(), "N1: no event");
    let r = app.key_mod(KeyCode::Enter, KeyModifiers::CONTROL);
    assert_eq!(r.flow(), Flow::Ignored, "N2: Ctrl+Enter ignores");
    assert!(log.borrow().is_empty(), "N2: no event");

    let _ = app.key(KeyCode::Enter);
    let _ = app.type_str("work");
    assert!(
        buf_row_text(app.buffer(), 1).contains("work"),
        "S1: typed into the snapshot"
    );

    assert!(
        buf_is_underlined(app.buffer(), 6, 1),
        "V1: editing underlined"
    );
    assert!(app.cursor().is_some(), "V1: hardware cursor placed");
    capture_isolated(&dir, "ti-editing", app.buffer());

    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "commit applies");
    assert!(
        log.borrow().contains(&TextAction::Committed),
        "Committed event"
    );
    assert!(
        buf_row_text(app.buffer(), 1).contains("work"),
        "S2: commit keeps text"
    );
    assert!(
        !buf_is_underlined(app.buffer(), 6, 1),
        "S2: commit leaves editing"
    );
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Enter);
    assert!(app.tab_to(S2_TICOMMIT_SENT), "A2: Tab forward");
    assert!(
        log.borrow().contains(&TextAction::Committed),
        "A2: Tab commits"
    );
    let r = app.key(KeyCode::BackTab);
    assert_eq!(r.flow(), Flow::Consumed, "A2: BackTab back");
    assert!(
        app.state_of(S2_TICOMMIT).contains(StateFlags::FOCUSED),
        "A2: focus returns to the field"
    );

    let case = s2_case("input_commit", &["--page", "inputs"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    for _ in 0..4 {
        support::press_step(&s, "tab");
    }
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Search files never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Type a path or symbol"))
        },
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "V1: editing mode");
    checkpoint(&mut s, &dir, "editing");
    type_text(&mut s, "work");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Search files saved", "V2: saved status");
    checkpoint(&mut s, &dir, "committed");
    let done = live_text(&mut s);
    assert!(!done.contains("EDIT"), "V2 live: commit leaves editing");
    assert!(done.contains("work"), "V2 live: committed text plain");
    eprintln!("s2 input_commit: isolated lifecycle + live saved status");
}

struct CommitApp {
    value: String,
    st: TextInputState,
    log: Rc<RefCell<Vec<TextAction>>>,
}

impl App for CommitApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        let a = TextInput::new(S2_TICOMMIT)
            .update(cx, &mut self.st, &mut self.value)
            .on_action(|action| log.borrow_mut().push(action));
        let b = Button::new(S2_TICOMMIT_SENT, "Next").update(cx).erase();
        a | b
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(S2_TICOMMIT)
            .value(&self.value)
            .placeholder("Type a path or symbol…")
            .draw(ui, Rect::new(4, 1, 36, 1), &self.st);
        Button::new(S2_TICOMMIT_SENT, "Next").draw(ui, Rect::new(4, 3, 10, 1));
    }
}

const S2_TICANCEL: Id = Id::root("s2.ticancel");
const S2_TICANCEL_SENT: Id = Id::root("s2.ticancel.sent");

fn no_x(s: &str) -> Result<(), FieldError> {
    if s.contains('x') {
        Err(FieldError::new("no x"))
    } else {
        Ok(())
    }
}

#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_input_cancel() {
    let dir = s2_dir("s2_input_cancel");

    let log = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        CancelApp {
            value: "payments-gateway".to_string(),
            st: TextInputState::default(),
            log: Rc::clone(&log),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S2_TICANCEL), "navigating focus");
    assert_ne!(app.buffer()[(39, 1)].symbol(), "!", "fixture starts valid");
    let _ = app.key(KeyCode::Enter);
    let _ = app.type_str("-x");
    assert!(
        buf_row_text(app.buffer(), 1).contains("payments-gateway-x"),
        "staged edit"
    );

    assert_ne!(
        app.buffer()[(39, 1)].symbol(),
        "!",
        "live typing sets no error"
    );
    let mut st = TextInputState::default();
    st.begin("payments-gateway-x");
    assert!(
        st.commit(&mut String::new(), &no_x).is_err(),
        "staged edit invalid"
    );
    assert!(st.error().is_some(), "staged edit is invalid");
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Esc applies");
    assert!(
        log.borrow().contains(&TextAction::Cancelled),
        "A1: Cancelled event"
    );

    assert!(
        !buf_is_underlined(app.buffer(), 6, 1),
        "A2: editing cleared first"
    );
    assert!(
        buf_row_text(app.buffer(), 1).contains("payments-gateway"),
        "S1/S2: snapshot restored whole"
    );
    assert!(
        !buf_row_text(app.buffer(), 1).contains("gateway-x"),
        "S1/S2: no staged remnant"
    );
    st.cancel();
    assert!(st.error().is_none(), "S2: revalidated after revert");

    let buf = render_isolated(
        44,
        5,
        0,
        Some((S2_TICANCEL, ReferenceState::FOCUSED)),
        |ui, _| {
            Field::new(
                "Project name",
                TextInput::new(S2_TICANCEL).value("payments-gateway"),
            )
            .required(true)
            .help("Used as the working directory name")
            .error(st.error().map(|e| e.message.as_ref()))
            .draw(ui, Rect::new(4, 1, 36, 3), &st);
        },
    );
    assert!(
        buf_row_text(&buf, 3).contains("working directory"),
        "V2: help re-rendered"
    );
    capture_isolated(&dir, "ti-reverted", &buf);

    log.borrow_mut().clear();
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Ignored, "N1: nav Esc ignores");
    assert!(log.borrow().is_empty(), "N1: no event");

    let _ = app.key(KeyCode::Enter);
    for _ in 0..7 {
        let _ = app.key(KeyCode::Left);
    }
    let _ = app.type_str("X");
    assert!(
        buf_row_text(app.buffer(), 1).contains('X'),
        "mid-buffer edit staged"
    );
    let _ = app.key(KeyCode::Esc);
    assert!(
        buf_row_text(app.buffer(), 1).contains("payments-gateway"),
        "N2: whole buffer restored"
    );
    assert!(
        !buf_row_text(app.buffer(), 1).contains('X'),
        "N2: no partial remnant"
    );

    let case = s2_case("input_cancel", &["--page", "inputs"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Project name never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("payments-gateway"))
        },
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "editing Project name");
    type_text(&mut s, "-x");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "staged edit never rendered",
        |screen| support::screen_text(screen).contains("gateway-x"),
    );
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "Reverted", "A1: Reverted status");
    checkpoint(&mut s, &dir, "reverted");
    let done = live_text(&mut s);
    assert!(!done.contains("EDIT"), "V1 live: cancel leaves editing");
    assert!(done.contains("payments-gateway"), "V1 live: value restored");
    assert!(!done.contains("gateway-x"), "N2 live: no partial revert");
    eprintln!("s2 input_cancel: isolated revert + live Reverted status");
}

struct CancelApp {
    value: String,
    st: TextInputState,
    log: Rc<RefCell<Vec<TextAction>>>,
}

impl App for CancelApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        let a = TextInput::new(S2_TICANCEL)
            .update(cx, &mut self.st, &mut self.value)
            .on_action(|action| log.borrow_mut().push(action));
        let b = Button::new(S2_TICANCEL_SENT, "Next").update(cx).erase();
        a | b
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(S2_TICANCEL)
            .value(&self.value)
            .draw(ui, Rect::new(4, 1, 36, 1), &self.st);
        Button::new(S2_TICANCEL_SENT, "Next").draw(ui, Rect::new(4, 3, 10, 1));
    }
}
