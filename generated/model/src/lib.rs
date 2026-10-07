// generated from controlplane v1
// model digest 81c7f5e0230fee0d706b0ec2528b81407c999295df8491e6ebc040ba5bdb6bdf
// contract digest 003b16f133632f8fde01042faa8c7c30286cba0b098ee1c71da99eb0004dbf1a
// do not edit: regenerate with `ess synthesize --layout crate`

//! Semantic types synthesised from the `controlplane` specification, v1.
//!
//! Generated, not written: the specification is the source of truth, and the door to changing
//! anything here is `ess synthesize`. What is deliberately absent — behaviour, queries,
//! escalations — is listed with reasons in the `PLAN.md` beside this workspace, and every entry
//! there is owed through a typed seam in an `obligations` module here.

// `deny`, not the source workspace's lint set: this crate must hold on its own, and an undocumented
// public item here is an emitter defect worth failing the gate over.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod actor;
pub mod behaviour;
pub mod host;
pub mod obligation;
pub mod primitives;

pub mod ports;
pub mod system;
#[cfg(feature = "server")]
pub mod server;
