//! Host CLI operations and result schemas (`tc-proof-host-result/v1`).

mod authority;
mod context;
mod context_check;
mod dispatch;
mod freeze;
mod git;
mod install;
mod integrate;
mod operation;
mod prepare;
mod result;
mod scope;
mod seal;
mod verify;

pub use authority::AUTHORITY_ENV;
pub use dispatch::{
    dispatch_freeze, dispatch_install, dispatch_integrate, dispatch_prepare, dispatch_seal,
    dispatch_unimplemented, dispatch_verify, emit_result, finish,
};
pub use operation::HostOperation;
pub use result::{HostResult, HostStatus};
