// generated from controlplane v1
// model digest 5ee011354cbdde7e1cd5aaec6e606c149d9e94ef7ac62c9bf1fc21621ff9b8ba
// contract digest c0aa7ecbbdf9304cda7cddf46352bf7a404ef748f1fd5a39222ff2628fadf6ca
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
