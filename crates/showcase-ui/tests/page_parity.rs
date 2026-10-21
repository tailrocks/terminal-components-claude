//! The Showcase pages must render exactly the frame they rendered before the
//! page chrome moved into the shared blocks.
//!
//! The digests below were recorded from the page-owned implementation of the
//! frame, the section strip and the card chrome. A page that changes a single
//! cell, a single colour or a single modifier breaks this test.
//!
//! Set `SHOWCASE_PAGE_DIGEST_DUMP=1` to print the current digests instead of
//! comparing them.

use showcase_ui::{App, PageId};
use termrock::Theme;
use termrock_test_support::Harness;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

fn fnv(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn digest(harness: &Harness<App>) -> u64 {
    let buffer = harness.buffer();
    let area = *buffer.area();
    let mut hash = FNV_OFFSET;
    for pos in area.positions() {
        let Some(cell) = buffer.cell(pos) else {
            continue;
        };
        hash = fnv(hash, cell.symbol().as_bytes());
        hash = fnv(
            hash,
            format!("{:?}|{:?}|{}", cell.fg, cell.bg, cell.modifier.bits()).as_bytes(),
        );
    }
    hash
}

/// The sizes every page is checked at.
const SIZES: [(u16, u16); 3] = [(120, 40), (94, 36), (72, 24)];

/// The reference digests, recorded from the page-owned chrome.
const EXPECTED: &[(PageId, u16, u16, u64)] = &[
    (PageId::Overview, 120, 40, 11050295493537092546),
    (PageId::Overview, 94, 36, 265245419492863806),
    (PageId::Overview, 72, 24, 1959414322675795192),
    (PageId::Buttons, 120, 40, 15911245586784452592),
    (PageId::Buttons, 94, 36, 16093913952419205050),
    (PageId::Buttons, 72, 24, 18194805481808223292),
    (PageId::Inputs, 120, 40, 1924922850220947501),
    (PageId::Inputs, 94, 36, 1410374474246806304),
    (PageId::Inputs, 72, 24, 2916762698110039012),
    (PageId::TextAreas, 120, 40, 3578083519580397634),
    (PageId::TextAreas, 94, 36, 17165889350773272033),
    (PageId::TextAreas, 72, 24, 18110424903868707538),
    (PageId::Forms, 120, 40, 12732279426243991487),
    (PageId::Forms, 94, 36, 5132513223885413638),
    (PageId::Forms, 72, 24, 3519864681510780622),
    (PageId::Lists, 120, 40, 12244647140251062687),
    (PageId::Lists, 94, 36, 10396194719429061140),
    (PageId::Lists, 72, 24, 4265928152063205019),
    (PageId::Trees, 120, 40, 11995029465105099520),
    (PageId::Trees, 94, 36, 13671810011005026061),
    (PageId::Trees, 72, 24, 15475169254985332992),
    (PageId::Tables, 120, 40, 9261625077284736931),
    (PageId::Tables, 94, 36, 6858764814541239139),
    (PageId::Tables, 72, 24, 7984287700459793822),
    (PageId::Editable, 120, 40, 8111759234098356192),
    (PageId::Editable, 94, 36, 6948814101111174929),
    (PageId::Editable, 72, 24, 8470714635111690433),
    (PageId::Panels, 120, 40, 17193315239285122088),
    (PageId::Panels, 94, 36, 5048986325858512055),
    (PageId::Panels, 72, 24, 3478044865239675396),
    (PageId::Sidebars, 120, 40, 2593364996604143358),
    (PageId::Sidebars, 94, 36, 16158865202530344792),
    (PageId::Sidebars, 72, 24, 1481599735238533885),
    (PageId::Dialogs, 120, 40, 5836603414739476635),
    (PageId::Dialogs, 94, 36, 11162695361350188113),
    (PageId::Dialogs, 72, 24, 12395031771573809391),
    (PageId::Progress, 120, 40, 13971373949735811971),
    (PageId::Progress, 94, 36, 8170353787875436056),
    (PageId::Progress, 72, 24, 5200886467623158169),
    (PageId::Scrolling, 120, 40, 4614399495368316958),
    (PageId::Scrolling, 94, 36, 17012135215139101156),
    (PageId::Scrolling, 72, 24, 11657862346163018909),
    (PageId::Terminal, 120, 40, 16713486997613533058),
    (PageId::Terminal, 94, 36, 10834531301898289047),
    (PageId::Terminal, 72, 24, 11799924430438265034),
    (PageId::Editor, 120, 40, 1337637796329482202),
    (PageId::Editor, 94, 36, 7146274324227091806),
    (PageId::Editor, 72, 24, 5214310364118716316),
    (PageId::Grid, 120, 40, 3666620698203807582),
    (PageId::Grid, 94, 36, 12334150473228131772),
    (PageId::Grid, 72, 24, 11661855356404301741),
    (PageId::Chips, 120, 40, 10272328058037678908),
    (PageId::Chips, 94, 36, 4872291124770157024),
    (PageId::Chips, 72, 24, 13292848669377404527),
    (PageId::Pickers, 120, 40, 14623648132736859899),
    (PageId::Pickers, 94, 36, 4127470860857909736),
    (PageId::Pickers, 72, 24, 4621388847621274409),
    (PageId::Chrome, 120, 40, 6700263497163196119),
    (PageId::Chrome, 94, 36, 12973512990952065289),
    (PageId::Chrome, 72, 24, 5752365008248186575),
    (PageId::Settings, 120, 40, 15374202353337629242),
    (PageId::Settings, 94, 36, 1249960723064850096),
    (PageId::Settings, 72, 24, 6807115167094579370),
    (PageId::TaskRunner, 120, 40, 10967000064222126980),
    (PageId::TaskRunner, 94, 36, 15039339211887828337),
    (PageId::TaskRunner, 72, 24, 7704215100529900226),
];

fn dump() {
    for page in PageId::ALL {
        for (width, height) in SIZES {
            let harness = Harness::new(App::with_page(page), Theme::junie(), width, height);
            println!("({:?}, {width}, {height}, {}),", page, digest(&harness));
        }
    }
}

#[test]
fn every_page_renders_the_reference_frame() {
    if std::env::var_os("SHOWCASE_PAGE_DIGEST_DUMP").is_some() {
        dump();
        return;
    }
    assert!(
        !EXPECTED.is_empty(),
        "the reference digests are missing: record them with SHOWCASE_PAGE_DIGEST_DUMP=1"
    );
    for (page, width, height, expected) in EXPECTED {
        let harness = Harness::new(App::with_page(*page), Theme::junie(), *width, *height);
        assert_eq!(
            digest(&harness),
            *expected,
            "page {page:?} at {width}x{height} changed"
        );
    }
}
