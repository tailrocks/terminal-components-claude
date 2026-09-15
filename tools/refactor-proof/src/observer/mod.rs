//! Observer IPC client for independently supervised worker execution.

mod client;
mod ipc;

pub use client::{decode_file_payload, ObserverClient, ObserverUnavailable};
pub use ipc::{
    ObserverEnv, ObserverError, ObserverObservation, ObserverRequest, ObserverRequestSchema,
    ObserverResponse, ObserverStep,
};
