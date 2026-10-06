# Contract verification

Run these from the repository:

```console
cargo run --locked -p control-plane-xtask -- generated-check
cargo run --locked -p control-plane-xtask -- conformance
cargo run --locked -p control-plane-xtask -- foundation-check
cargo run --locked -p control-plane-xtask -- spec-history-check
cargo test --locked -p control-plane-xtask
```

`generate` installs fresh Rust, OpenAPI and conformance emitter bytes. Both generation commands
use new scratch ownership roots and compare every generated file, including file-set changes;
only `.ess-output` machine ownership ledgers are excluded. Generation requires every capability
in the compiler plan to be generated. ESS 0.53.0 is the matching released compiler; the Rust
runner and primitives are pinned to its release commit.

The conformance target invokes `control_plane_core::contract::ContractStore`, the production
persistence boundary beneath operational filesystem admission. It selects the requested declared
actor and invokes the actual dispatcher. Only an actual dispatcher 403 with the `not granted`
body becomes `TargetError::NotGranted`. Other failures remain execution errors. No target code
predicts a domain outcome from an input or supplies synthetic domain state.

Each scenario owns a fresh Eventlog SQLite database. Every command closes its store; each next
command or view reopens it and replays committed history. Consistency tokens are issued only
after successful durable command completion. Reads reject future tokens and replay the complete
history before querying. Observed events contain the actual successful command's committed
outcome envelope. Scenario cleanup clears the observation log and database. No effects are
executed by this contract-level suite; operational guards and planner/fleet behavior have their
own implementation tests.

Integer conversion is exact `i64` throughout, with no intermediate binary64. Tests cover both
numeric extrema and integers beyond binary64's exact range through the codec, actual generated
command decoder, emitted event, durable replay and view.

`conformance` saves an `ess-conformance-report/2` document and detailed diagnostics under
`.scratch/conformance/`. The gate requires at least 152 scenarios, every scenario passed, zero
failed/error/unsupported/skipped, and the report's coverage qualification passed. The report is
constructed by the ESS Rust runner over the original admitted suite bytes.

The ordinary `--suite-format 4` synthesis produced suite/34 without a coverage inventory;
its report could only qualify coverage as unknown. The supported `--suite-format 5` option
produces suite/35 with the compiler's `complete_inventory`: 152 generated obligations,
zero authored, zero outside selection and zero refused. Selection covers the complete system.
No inventory or expectation is hand-edited. This measures those declared generated obligations;
it does not establish completeness of the product specification.

The production dependency gate traverses Cargo's resolved normal/build dependency graph from
all product workspace crates. Dependency renames do not hide package identity. ESS verification
libraries are admitted only under xtask tooling. Runtime ecosystem dependencies are restricted
to embedded Loom SDK and its internal Commission/governor/executor/intake libraries, Canon,
Eventlog, LLM libraries and ESS runtime/primitives. Standalone higher-level repositories and
Loom service entrypoints are rejected. Process adapters to repository-configured CLI tools do
not introduce Cargo runtime dependencies.

## Stored history

`Store::open` re-invokes every recorded command with its recorded body and actor through current
generated behavior. It refuses to start when any answer differs from the recorded one. Two checks
keep a specification change from doing that silently.

`spec-history-check`, a `task check` step, compares the specification at a baseline commit with
`ess/`. It copies the baseline's `ess/` tree into `.scratch/` and runs
`ess verify diff --compatibility --format json`. The baseline is chosen like this:

- Use the merge base of `HEAD` with `origin/main` when that commit holds `ess/ess-inputs.yaml`.
  A clone without `origin/main` uses `main` instead.
- Otherwise use the full commit id recorded as `baseline` in `ess/spec-acknowledgements.json`.
- When both exist, use the one that descends from the other. When neither descends from the
  other, the check fails.

The baseline must be in the clone, so CI checks out full history.

A change fails the check when ESS rates any of `callers`, `readers` or `history` other than
`compatible`. ESS rates a changed outcome, payload, error shape or grant `history: compatible`,
but replay still fails, so the history verdict alone is not enough. Purely additive changes are
exempt, because no recorded call can observe them:

- an enum or union variant added (`variant-added`, relation `expanded`)
- an `Optional<…>` input added to a command
- an `Optional<…>` field added to an entity or a view

New commands, views, events, entities, errors, types, actors and grants are already `compatible`
in every dimension. A required input or field, and any field added to an event or error, still
fails. A command's `outcome-subject-changed` that only restates a reported
`transition-route-changed` of the same entity and transition is decided by that route change.

An acknowledgement admits exactly the change it reviewed: the id, plus the `change` object ESS
printed for it. If a different change later carries the same id, it is not acknowledged. The
refusal prints a ready entry for each unacknowledged change. Removing `Blocked` from
`Assignment.repair.from` needs this one entry:

```json
{
  "format": "control-plane-spec-acknowledgements/1",
  "baseline": "<full commit id>",
  "acknowledged": [
    {
      "id": "entity/controlplane.host.Assignment/transition-route-changed/repair",
      "change": {
        "category": "entity",
        "subject": "controlplane.host.Assignment",
        "changed": {
          "kind": "transition-route-changed",
          "transition": "repair",
          "before": "Blocked, Reviewing -> Implementing",
          "after": "Reviewing -> Implementing"
        }
      },
      "reason": "stored Blocked repairs are migrated by <change>"
    }
  ]
}
```

An acknowledgement that matches no failing change is stale and fails the check. The exception is
an entry that the baseline commit's own acknowledgement file already holds: it was published with
that baseline. Acknowledge a change only after stored history has a migration or cannot contain
the affected decision. When you move the recorded baseline, empty the acknowledgements in the
same change.

`crates/control-plane-core/tests/recorded_history.rs` opens a committed history,
`crates/control-plane-core/tests/fixtures/recorded-history.db`, and compares every view with
`recorded-history.views.json`. The history must apply every generated command at least once. It
must also answer every declared refusal response of every command at least once: each
`not-found`, and each `wrong-state` or `DeleteGoal` `paused`. The test reads those responses
from the generated OpenAPI contract. A second test changes one stored `applied` outcome in a copy
and requires `Store::open` to refuse it. No test or gate writes the fixture. Re-record it only on
purpose:

```console
cargo run --locked -p control-plane-xtask -- record-history --work-dir /dev/shm/control-plane-history
```

`record-history` runs a scripted operator and supervisor through `Store::execute` against real Git
repositories in the new work directory. That directory must be outside every home directory and
Git work tree: recorded paths are committed, and the Security gate rejects personal paths. Before
anything else, the command replays the committed history. It refuses to replace that history
when `Store::open` fails or the replayed views differ from `recorded-history.views.json`. If
replay fails after a specification change, write a migration; do not re-record. Re-record only to
cover a new command or refusal, and review the views diff. A new view field also changes the
recorded views, so delete both fixture files deliberately before you re-record.

## Evidence and negative controls

The initial `generated_gate_detects_changed_bytes` test failed with exit 101 against a stub that
accepted different contract bytes. The completed comparator passes that same assertion and
also rejects stale generated files while ignoring machine ownership ledgers.

The first real durable run reported 38 passed, 44 failed and 70 error: the adapter omitted
read-your-writes consistency tokens, so ESS correctly refused to weaken the requested views.
The adapter now supplies a token only after a durable commit. This red report and diagnostics
are retained locally as `.scratch/conformance/initial-red-*`.

The negative conformance test performs actual commands and reads, then deliberately maps an
observed `Archived` workspace state to `Registered`. It produces 150 passed and two failed,
with zero errors/unsupported/skipped. The failures are:

- `controlplane.host.ArchiveWorkspace/outcome/applied`
- `controlplane.host.Workspace/transition/archive/by/controlplane.host.ArchiveWorkspace/applied`

The test requires the named transition failure and rejects its report through the same gate.
Its report and diagnostics are retained under `.scratch/conformance-tests/negative-*`.
The dependency fixture separately proves that a renamed transitive Metaharness dependency is
rejected, while permitted embedded Loom internals and xtask-only ESS tooling are accepted.

Three consecutive local CLI runs each produced 152 passed, zero failed/error/unsupported/skipped,
and a passed coverage qualification. Their canonical reports were byte-identical and are
retained locally as `.scratch/conformance/report-{1,2,3}.json`. Repository CI and the final
planner/fleet qualification remain separate integration checks.
