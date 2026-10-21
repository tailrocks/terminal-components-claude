use showcase_ui::{App, PageId};
use termrock::{ColorLevel, Theme};
use termrock_test_support::Harness;

fn compare_reference_rows(
    fixture: &str,
    row: usize,
    fixture_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let bless = std::env::var("BLESS").is_ok();
    let mut updated_lines = Vec::new();
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
        let actual = h
            .text()
            .lines()
            .nth(row)
            .unwrap_or_default()
            .trim_end()
            .to_string();
        if bless {
            updated_lines.push(format!("{name}\t{width}\t{height}\t{color}\t{actual}"));
        } else {
            assert_eq!(
                actual, expected,
                "{name} {width}x{height} {color} row {row}"
            );
        }
    }
    if bless && let Some(path) = fixture_path {
        let content = format!("{}\n", updated_lines.join("\n"));
        std::fs::write(path, content)?;
    }
    Ok(())
}

#[test]
fn all_page_headings_match_the_reference_capture_matrix() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = include_str!("fixtures/holla-page-headings.tsv");
    assert_eq!(fixture.lines().count(), 352);
    let fixture_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/holla-page-headings.tsv"
    );
    compare_reference_rows(fixture, 2, Some(fixture_path))
}

#[test]
fn all_shell_headers_match_reference_captures_and_visibility_thresholds()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = include_str!("fixtures/holla-shell-headers.tsv");
    assert_eq!(fixture.lines().count(), 355);
    let fixture_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/holla-shell-headers.tsv"
    );
    compare_reference_rows(fixture, 0, Some(fixture_path))
}
