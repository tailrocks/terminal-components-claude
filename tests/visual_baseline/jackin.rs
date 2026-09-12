//! jackin-preview captures (26): scenarios x {80x24,120x40}, intro phases
//!
//! Ported verbatim from the retired tools/tuisnap_baseline.sh (capture
//! names, argv, needles, sends, CAP_TIMEOUTs). Do not hand-tune: drift
//! against the approved frames means the port or the app changed.

use crate::support::{Case, Color, JACKIN};

crate::baseline_case!(jackin_first_use_default_80x24_truecolor => Case::new("jackin_first-use_default_80x24_truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_first_use_default_120x40_truecolor => Case::new("jackin_first-use_default_120x40_truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_returning_default_80x24_truecolor => Case::new("jackin_returning_default_80x24_truecolor", JACKIN, &["--scenario", "returning", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_returning_default_120x40_truecolor => Case::new("jackin_returning_default_120x40_truecolor", JACKIN, &["--scenario", "returning", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_accounts_mixed_default_80x24_truecolor => Case::new("jackin_accounts-mixed_default_80x24_truecolor", JACKIN, &["--scenario", "accounts-mixed", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_accounts_mixed_default_120x40_truecolor => Case::new("jackin_accounts-mixed_default_120x40_truecolor", JACKIN, &["--scenario", "accounts-mixed", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_launch_running_default_80x24_truecolor => Case::new("jackin_launch-running_default_80x24_truecolor", JACKIN, &["--scenario", "launch-running", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_launch_running_default_120x40_truecolor => Case::new("jackin_launch-running_default_120x40_truecolor", JACKIN, &["--scenario", "launch-running", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_launch_failure_default_80x24_truecolor => Case::new("jackin_launch-failure_default_80x24_truecolor", JACKIN, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_launch_failure_default_120x40_truecolor => Case::new("jackin_launch-failure_default_120x40_truecolor", JACKIN, &["--scenario", "launch-failure", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_capsule_multi_default_80x24_truecolor => Case::new("jackin_capsule-multi_default_80x24_truecolor", JACKIN, &["--scenario", "capsule-multi", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_capsule_multi_default_120x40_truecolor => Case::new("jackin_capsule-multi_default_120x40_truecolor", JACKIN, &["--scenario", "capsule-multi", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_hard_cases_default_80x24_truecolor => Case::new("jackin_hard-cases_default_80x24_truecolor", JACKIN, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_hard_cases_default_120x40_truecolor => Case::new("jackin_hard-cases_default_120x40_truecolor", JACKIN, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_outro_last_default_80x24_truecolor => Case::new("jackin_outro-last_default_80x24_truecolor", JACKIN, &["--scenario", "outro-last", "--motion", "paused", "--frame", "40"], 80, 24, Color::Truecolor, "Enter Skip"));
crate::baseline_case!(jackin_outro_last_default_120x40_truecolor => Case::new("jackin_outro-last_default_120x40_truecolor", JACKIN, &["--scenario", "outro-last", "--motion", "paused", "--frame", "40"], 120, 40, Color::Truecolor, "Enter Skip"));
crate::baseline_case!(jackin_first_use_f300_120x40_truecolor => Case::new("jackin_first-use_f300_120x40_truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "300"], 120, 40, Color::Truecolor, "Enter Skip"));
crate::baseline_case!(jackin_first_use_f400_120x40_truecolor => Case::new("jackin_first-use_f400_120x40_truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "400"], 120, 40, Color::Truecolor, "jackin❯"));
crate::baseline_case!(jackin_first_use_default_120x40_none => Case::new("jackin_first-use_default_120x40_none", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::None, "jackin❯"));
crate::baseline_case!(jackin_capsule_multi_default_120x40_none => Case::new("jackin_capsule-multi_default_120x40_none", JACKIN, &["--scenario", "capsule-multi", "--motion", "paused", "--frame", "40"], 120, 40, Color::None, "jackin❯"));
crate::baseline_case!(jackin_accounts_mixed_default_120x40_none => Case::new("jackin_accounts-mixed_default_120x40_none", JACKIN, &["--scenario", "accounts-mixed", "--motion", "paused", "--frame", "40"], 120, 40, Color::None, "jackin❯"));
crate::baseline_case!(jackin_hard_cases_default_120x40_none => Case::new("jackin_hard-cases_default_120x40_none", JACKIN, &["--scenario", "hard-cases", "--motion", "paused", "--frame", "40"], 120, 40, Color::None, "jackin❯"));
crate::baseline_case!(jackin_capsule_multi_default_120x40_256 => Case::new("jackin_capsule-multi_default_120x40_256", JACKIN, &["--scenario", "capsule-multi", "--motion", "paused", "--frame", "40"], 120, 40, Color::Ansi256, "jackin❯"));
crate::baseline_case!(jackin_accounts_mixed_default_120x40_16 => Case::new("jackin_accounts-mixed_default_120x40_16", JACKIN, &["--scenario", "accounts-mixed", "--motion", "paused", "--frame", "40"], 120, 40, Color::Ansi16, "jackin❯"));
crate::baseline_case!(jackin_first_use_default_120x40_nocolor => Case::new("jackin_first-use_default_120x40_nocolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 120, 40, Color::NoColorEnv, "jackin❯"));
crate::baseline_case!(jackin_first_use_default_72x20_truecolor => Case::new("jackin_first-use_default_72x20_truecolor", JACKIN, &["--scenario", "first-use", "--motion", "paused", "--frame", "40"], 72, 20, Color::Truecolor, "jackin❯"));
