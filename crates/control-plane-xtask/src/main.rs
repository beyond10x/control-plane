mod acceptance;
mod codec;
mod eval;
mod foundation;
mod frontend;
mod generation;
mod history;
mod mutation;
mod spec_history;
mod target;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

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
    /// Refuse unacknowledged specification changes that could break stored history.
    SpecHistoryCheck,
    /// Deliberately re-record the host history fixture that `Store::open` must replay.
    RecordHistory {
        /// A new directory outside every home directory and Git work tree; recorded paths are
        /// committed with the fixture.
        #[arg(long)]
        work_dir: PathBuf,
    },
    /// Run the ESS mutation audit against the durable generated store: emit, run, collect.
    Mutate {
        /// A mutation class to audit; repeat for several. Defaults to the guard classes.
        #[arg(long = "class")]
        classes: Vec<String>,
        /// Keep the emitted suites, reports and the `--collect` report under `.scratch`.
        #[arg(long)]
        keep: bool,
    },
    /// Refuse scenario names in active and implemented stories that name no test.
    AcceptanceCheck {
        /// The repository to check; defaults to this one.
        #[arg(long)]
        root: Option<PathBuf>,
    },
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
        Action::SpecHistoryCheck => spec_history::run(root),
        Action::RecordHistory { work_dir } => history::run(root, &work_dir),
        Action::Mutate { classes, keep } => mutation::run_verb(root, classes, keep),
        Action::AcceptanceCheck { root: checked } => {
            acceptance::run(checked.as_deref().unwrap_or(root))
        }
    }
}
