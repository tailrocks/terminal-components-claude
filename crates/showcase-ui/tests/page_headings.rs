use showcase_ui::{App, PageId};
use termrock::{ColorLevel, Theme};
use termrock_test_support::Harness;

#[test]
fn all_page_headings_match_the_reference_capture_matrix() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = include_str!("fixtures/holla-page-headings.tsv");
    assert_eq!(fixture.lines().count(), 352);
    compare_reference_rows(fixture, 2)
}

#[test]
fn all_shell_headers_match_reference_captures_and_visibility_thresholds()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = include_str!("fixtures/holla-shell-headers.tsv");
    assert_eq!(fixture.lines().count(), 355);
    compare_reference_rows(fixture, 0)
}

/// The shell header is the frozen `visual-baseline` capture, verbatim.
///
/// Reference: `visual-baseline:snapshots/showcase/pages/chrome/100x30/16.ansi`,
/// first line of the frame:
///
/// ```text
///  ▪ Junie Design system  App shell › Chrome               16 colors · 100×30   i Inspector   ? Help
/// ```
///
/// The brand lockup, the `group › page` crumb and the right-hand capability,
/// inspector and help affordances all come from that capture.
#[test]
fn the_shell_header_matches_the_visual_baseline_capture() -> Result<(), Box<dyn std::error::Error>>
{
    let h = Harness::new(App::with_page(PageId::Chrome), Theme::junie(), 100, 30)
        .with_color(ColorLevel::Ansi16);
    assert_eq!(
        h.text().lines().next().unwrap_or_default().trim_end(),
        " ▪ Junie Design system  App shell › Chrome               16 colors · 100×30   i Inspector   ? Help"
    );
    Ok(())
}

/// The capability hint clears the crumb at the width the reference guard sets.
///
/// Reference: `visual-baseline:src/bin/showcase/app.rs:1809` draws the
/// capability only when `rx > x + crumb_w + cw + 2`. With the Overview crumb
/// (`Overview`, no group) that guard opens at 76 columns, so 75 columns hides
/// the hint and 76 columns shows it. The frozen captures agree:
/// `visual-baseline:snapshots/showcase/pages/overview/72x20/truecolor.ansi`
/// has no capability and
/// `visual-baseline:snapshots/showcase/pages/overview/80x24/truecolor.ansi`
/// has `truecolor · 80×24`.
#[test]
fn the_capability_hint_clears_the_crumb_at_the_reference_threshold()
-> Result<(), Box<dyn std::error::Error>> {
    let header = |width: u16| -> String {
        let h = Harness::new(App::with_page(PageId::Overview), Theme::junie(), width, 24)
            .with_color(ColorLevel::TrueColor);
        h.text()
            .lines()
            .next()
            .unwrap_or_default()
            .trim_end()
            .to_owned()
    };
    assert!(
        !header(75).contains("truecolor"),
        "the hint stays hidden below the clearance width: {:?}",
        header(75)
    );
    assert_eq!(
        header(76),
        " ▪ Junie Design system  Overview  truecolor · 76×24   i Inspector   ? Help"
    );
    Ok(())
}

fn compare_reference_rows(fixture: &str, row: usize) -> Result<(), Box<dyn std::error::Error>> {
    for line in fixture.lines() {
        let mut fields = line.splitn(5, '\t');
        let name = fields.next().ok_or("missing page")?;
        let width = fields.next().ok_or("missing width")?.parse()?;
        let height = fields.next().ok_or("missing height")?.parse()?;
        let color = fields.next().ok_or("missing capability")?;
        let expected = fields.next().ok_or("missing reference row")?;
        let level = match color {
            "truecolor" => ColorLevel::TrueColor,
            "256" => ColorLevel::Ansi256,
            "16" => ColorLevel::Ansi16,
            "mono" => ColorLevel::Mono,
            _ => return Err("unknown reference capability".into()),
        };
        let page = PageId::from_name(name).ok_or("unknown reference page")?;
        let h = Harness::new(App::with_page(page), Theme::junie(), width, height).with_color(level);
        assert_eq!(
            h.text().lines().nth(row).unwrap_or_default().trim_end(),
            expected,
            "{name} {width}x{height} {color} row {row}"
        );
    }
    Ok(())
}
