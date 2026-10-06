mod codec;
mod eval;
mod foundation;
mod frontend;
mod generation;
mod target;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;

#[derive(Parser)]
#[command(about = "Control-plane generated contract and durable conformance gates")]
struct Cli {
    #[command(subcommand)]
    command: Action,
}
#[derive(Subcommand)]
enum Action {
    /// Prepare external local repositories or verify an agent's real output.
    Eval {
        #[command(subcommand)]
        action: eval::Action,
    },
    /// Generate Rust, OpenAPI and the conformance suite from ESS.
    Generate,
    /// Fail when checked-in artifacts differ from fresh emitter output.
    GeneratedCheck,
    /// Execute all ESS scenarios against durable production contract storage.
    Conformance,
    /// Reject higher-level libraries in the production dependency graph.
    FoundationCheck,
    /// Build the Vue assets embedded in the service binary.
    FrontendBuild,
    /// Rebuild Vue in scratch and refuse drift in the embedded assets.
    FrontendCheck,
}
fn main() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    match Cli::parse().command {
        Action::Eval { action } => eval::run(action),
        Action::Generate => generation::run(root, true),
        Action::GeneratedCheck => generation::run(root, false),
        Action::Conformance => target::conformance(root),
        Action::FoundationCheck => foundation::run(root),
        Action::FrontendBuild => frontend::run(root, true),
        Action::FrontendCheck => frontend::run(root, false),
    }
}
