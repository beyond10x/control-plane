// generated from controlplane v1
// model digest 067305d07e71dad22be3826b880d520f1f1c41ed0bdd99b54d385d0d95f58dad
// contract digest e1710083be9dac2cf442d1dc38108393f735542c913f94132cab9f770c55f8f4
// do not edit: regenerate with `ess synthesize --layout crate`

//! The HTTP surface of `controlplane` v1, synthesised.
//!
//! One module per component the specification declares is reached over a network, each holding
//! that component's route table, its listener and the two documents it publishes about itself.
//! The routes are the ones the committed `OpenAPI` document declares, from the same mapping, so a
//! path served here and a path published there cannot be two different answers.
//!
//! Generated, not written: the specification is the source of truth, and the door to changing
//! anything here is `ess synthesize`. What is deliberately absent is absent by
//! decision — no framework, no runtime, no second protocol, no concurrency, no authentication —
//! and each absence is argued in the `TARGET.md` beside this workspace.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod entry;
pub mod http;
pub mod json;
pub mod wire;
pub mod control_plane;
pub mod memory;
mod static_assets;
