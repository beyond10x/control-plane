// generated from controlplane v1
// model digest 3d7e5edad026a769d94fad7e6af37d426a599672c637ccd2389868c2ed11448b
// contract digest 27517aec229e5e98ea64875d55bb11b465af6d3dcfa1804f979163860350db55
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
