mod common;

use std::error::Error;

type TestResult = Result<(), Box<dyn Error>>;

macro_rules! functional_case_tests {
    ($($test_name:ident => $case_id:literal),* $(,)?) => {
        const FUNCTIONAL_CASE_TESTS: &[(&str, &str)] = &[
            $( (stringify!($test_name), $case_id), )*
        ];

        $(
            #[test]
            fn $test_name() -> TestResult {
                common::run_case_id($case_id)
            }
        )*
    };
}

functional_case_tests! {
    first30_showcase_buttons => "first30-v1/showcase.buttons",
    first30_holla_first_use => "first30-v1/holla.first_use",
    first30_jackin_settings => "first30-v1/jackin.settings",
    first30_tablepro_connections => "first30-v1/tablepro.connections",
    holla_first_use_frame_one_variant => "first30-v1/holla.first_use.frame1",
    showcase_help_paging => "showcase-help/80x24-v1",
    holla_first_use_action => "holla-first-use/action-identity-v1",
}

const AUXILIARY_TEST_NAMES: &[&str] = &["required_inventory_ready"];

#[test]
fn required_inventory_ready() -> TestResult {
    Err(std::io::Error::other(
        "NOT VERIFIED: the full required inventory is absent; Showcase 54, Holla 10, Jackin 12, and TablePro 21 routes remain unverified",
    )
    .into())
}
