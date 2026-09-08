//! Title-row text from the actual Holla 794b095 production binary.
//!
//! The fixture contains all 22 pages × four sizes × four capabilities. It
//! records reference observations, not candidate snapshots or a blessed
//! baseline. Acquisition hashes live in the external Showcase fidelity
//! evidence manifest. These rows are independent of animated page bodies.

use junie_tui::{ColorLevel, Theme};
use junie_tui_testing::Harness;
use showcase_app::{App, PageId};

#[test]
fn all_page_headings_match_the_reference_capture_matrix() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = include_str!("fixtures/holla-page-headings.tsv");
    assert_eq!(fixture.lines().count(), 352);
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
            h.text().lines().nth(2).unwrap_or_default().trim_end(),
            expected,
            "{name} {width}x{height} {color} title row"
        );
    }
    Ok(())
}
