pub const TEST_CLOCK_ENV: &str = "JUNIE_TUI_TEST_CLOCK";

pub const TEST_CLOCK_PASTE_PREFIX: &str = "junie-tui:advance-clock:";

#[must_use]
pub fn test_clock_enabled() -> bool {
    std::env::var_os(TEST_CLOCK_ENV).is_some_and(|value| value == "1")
}

#[must_use]
pub fn parse_test_clock_step(body: &str) -> Option<u64> {
    let rest = body.strip_prefix(TEST_CLOCK_PASTE_PREFIX)?;
    if rest.is_empty() || !rest.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    match rest.parse::<u64>() {
        Ok(0) | Err(_) => None,
        Ok(step_ms) => Some(step_ms),
    }
}
