// generated from controlplane v1
// model digest c4dda5ccc49fd738a60e886dbf7d9aa1b3aa7b406ec8f453127c63b9f6583548
// contract digest e2cc170afad569e615d682e77d7a36681653dc02f5870ba785e69e9b36846f10
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
