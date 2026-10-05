// generated from controlplane v1
// model digest 3d7e5edad026a769d94fad7e6af37d426a599672c637ccd2389868c2ed11448b
// contract digest 27517aec229e5e98ea64875d55bb11b465af6d3dcfa1804f979163860350db55
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
