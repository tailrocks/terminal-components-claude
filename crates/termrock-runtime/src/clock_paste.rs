//! Visual-baseline clock steps carried in a paste body.
//!
//! The tag is `junie-tui:advance-clock:` plus a positive millisecond count.
//! It is live only when `JUNIE_TUI_TEST_CLOCK` is exactly `1`.

/// Process environment variable that arms the paste protocol. The value must be `1`.
pub const TEST_CLOCK_ENV: &str = "JUNIE_TUI_TEST_CLOCK";

/// Paste-body prefix. A positive decimal millisecond count must follow it.
pub const TEST_CLOCK_PASTE_PREFIX: &str = "junie-tui:advance-clock:";

/// `true` only when [`TEST_CLOCK_ENV`] is exactly `1`.
#[must_use]
pub fn test_clock_enabled() -> bool {
    std::env::var_os(TEST_CLOCK_ENV).is_some_and(|value| value == "1")
}

/// Milliseconds from a clock-step paste, or `None` when the body is not a positive integer step.
///
/// `0`, an empty count, a sign, and any suffix do not match. Leading zeros are accepted only
/// when the value is greater than zero (`01` is 1; `00` is not).
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

#[cfg(test)]
mod tests {
    use super::{TEST_CLOCK_PASTE_PREFIX, parse_test_clock_step};

    fn body(rest: &str) -> String {
        let mut text = String::from(TEST_CLOCK_PASTE_PREFIX);
        text.push_str(rest);
        text
    }

    #[test]
    fn positive_decimal_milliseconds_only() {
        assert_eq!(parse_test_clock_step(&body("1100")), Some(1100));
        assert_eq!(parse_test_clock_step(&body("1")), Some(1));
        assert_eq!(parse_test_clock_step(&body("01")), Some(1));
        assert_eq!(
            parse_test_clock_step(&body("18446744073709551615")),
            Some(u64::MAX)
        );

        for rest in [
            "0", "00", "", "+1", "-1", "100ms", "100 ", " 100", "1_100", "1.5", "1\n",
        ] {
            assert_eq!(parse_test_clock_step(&body(rest)), None, "{rest:?}");
        }
        assert_eq!(parse_test_clock_step("1100"), None);
        assert_eq!(
            parse_test_clock_step("junie-tui:advance-clock:1100 extra"),
            None
        );
        assert_eq!(parse_test_clock_step(&body("18446744073709551616")), None);
    }
}
