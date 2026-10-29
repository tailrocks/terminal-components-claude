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
