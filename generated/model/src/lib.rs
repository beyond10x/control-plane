// generated from controlplane v1
// model digest 9c38829b718fc37a19c83d986d606b1bf71db0ac9d8c649a266a54fea251ea2b
// contract digest 1a9232380ffaa1aa950639a9b2ceec80e5466fb5787df27a171b3e721835dead
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
