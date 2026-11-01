//! Functional tests — one standard, the latest approach, for every test:
//! real executables in PTYs driven by `tui_test` (`tuiscotti-tui-test`
//! 0.2.1, the live half of latest Tuiscotti), the shared `support::App`
//! harness, and a frozen visual snapshot per test via latest Tuiscotti
//! (`tuiscotti-latest`, main `d8d5415`) checked with `App::visual`.
//! No test in this target uses the legacy Tuiscotti 0.2 live API.

#![cfg(any(target_os = "macos", target_os = "linux"))]

mod holla_jackin;
mod lifecycle;
mod ported;
mod showcase_tablepro;
mod support;
mod visual;
