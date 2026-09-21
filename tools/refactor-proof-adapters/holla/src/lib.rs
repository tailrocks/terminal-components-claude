//! Holla oracle reference observation adapter (`oracle-holla`).
//!
//! TASK-003 product: expands the 135 frozen HO scenario rows and the 34
//! oracle worlds into sealed-source, repeatable action/frame/semantic
//! evidence. Observation goes through production `holla_app`
//! (`App::for_scenario`) inside the deterministic `junie-tui-testing`
//! harness. Oracle `--frame N` is an 80 ms tick ordinal; the adapter converts
//! it to the millisecond seek production expects and never substitutes a
//! missing world.
//!
//! Frozen registers under `data/` are byte copies of the normative task
//! inputs (`docs/refactoring-plan/holla-scenarios.tsv` and the TASK-003
//! `trusted/*.tsv` files); see each module for its source.

pub mod branches;
pub mod clock;
pub mod construct;
pub mod expansion;
pub mod observe;
pub mod resize_map;
pub mod stages;
pub mod tsv;
pub mod worlds;

pub use clock::{ORACLE_EPOCH_SECS, TICK_MS, tick_to_ms};
pub use construct::{Construction, construct, construct_ho_base};
pub use expansion::{ScenarioProgram, expand_scenarios};
pub use observe::{ORACLE_PALETTES, ORACLE_SIZES, Session};
pub use worlds::{ORACLE_WORLD_COUNT, ORACLE_WORLDS};
