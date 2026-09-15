//! Host CLI operations and result schemas (`tc-proof-host-result/v1`).

mod authority;
mod dispatch;
mod install;
mod operation;
mod prepare;
mod result;

pub use authority::AUTHORITY_ENV;
pub use dispatch::{
    dispatch_install, dispatch_prepare, dispatch_unimplemented, emit_result, finish,
};
pub use operation::HostOperation;
pub use result::{HostResult, HostStatus};
