mod common;

use std::error::Error;

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn first_showcase() -> TestResult {
    common::run_case_id("first/showcase")
}

#[test]
fn first_holla() -> TestResult {
    common::run_case_id("first/holla")
}

#[test]
fn first_jackin() -> TestResult {
    common::run_case_id("first/jackin")
}

#[test]
fn first_tablepro() -> TestResult {
    common::run_case_id("first/tablepro")
}

#[test]
fn showcase_help_paging() -> TestResult {
    common::run_case_id("showcase-help/80x24-v1")
}

#[test]
fn holla_first_use_action() -> TestResult {
    common::run_case_id("holla-first-use/action-identity-v1")
}

#[test]
fn required_inventory_ready() -> TestResult {
    Err(std::io::Error::other(
        "NOT VERIFIED: the full required inventory is absent; Showcase 54, Holla 10, Jackin 12, and TablePro 21 routes remain unverified",
    )
    .into())
}
