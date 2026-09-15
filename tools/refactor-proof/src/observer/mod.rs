//! Observer IPC client for independently supervised worker execution.

mod client;
mod ipc;

pub use client::{ObserverClient, ObserverUnavailable};
pub use ipc::{
    ObserverEnv, ObserverError, ObserverObservation, ObserverRequest, ObserverRequestSchema,
    ObserverResponse, ObserverStep,
};
