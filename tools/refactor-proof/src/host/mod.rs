//! Host CLI operations and result schemas (`tc-proof-host-result/v1`).

mod operation;
mod result;

pub use operation::HostOperation;
pub use result::{HostResult, HostStatus};
