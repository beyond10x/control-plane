// generated from controlplane v1
// model digest c4dda5ccc49fd738a60e886dbf7d9aa1b3aa7b406ec8f453127c63b9f6583548
// contract digest e2cc170afad569e615d682e77d7a36681653dc02f5870ba785e69e9b36846f10
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
