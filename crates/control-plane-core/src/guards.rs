//! Host facts ESS cannot observe: canonical filesystem identity and cross-aggregate admission.
use crate::{Store, discovery};
use anyhow::{Context, Result, ensure};
use controlplane_model::host::{
    AssignmentSnapshot, AssignmentState as A, GoalState as G, PublicationIntentState as P,
    RepositoryRegistrationData, RepositoryRegistrationState as R, WorkspaceState as W,
};
use serde_json::{Value, json};
use std::path::Path;

fn text<'a>(body: &'a Value, field: &str) -> Result<&'a str> {
    body[field]
        .as_str()
        .with_context(|| format!("{field} must be a string"))
}
pub(crate) fn registration_key(command: &str, body: &Value) -> Option<String> {
    match command {
        "RegisterWorkspace" => Some(format!("workspace:{}", body["path"])),
        "RegisterRepository" => Some(format!(
            "repository:{}:{}",
            body["workspace_id"], body["common_dir"]
        )),
        "AddWorkspaceDirectory" => Some(format!(
            "directory:{}:{}",
            body["workspace_id"], body["path"]
        )),
        _ => None,
    }
}
fn active(state: A) -> bool {
    matches!(
        state,
        A::Implementing | A::Reviewing | A::ReadyToMerge | A::Merging | A::Blocked
    )
}

/// Whether a publication intent still holds its assignment: every intent but one closed as not
/// published. An open intent waits for its outcome to be observed, and a confirmed one for its
/// assignment to be reconciled.
fn holds(state: P) -> bool {
    state != P::NotPublished
}

fn occupies_worker(state: A) -> bool {
    matches!(
        state,
        A::Implementing | A::Reviewing | A::ReadyToMerge | A::Merging
    )
}

/// Whether the repository configuration `admitted` with an assignment's claim or repair is still
/// the repository's `current` one: the same base branch, test command and publication command.
fn same_configuration(
    admitted: &RepositoryRegistrationData,
    current: &RepositoryRegistrationData,
) -> bool {
    admitted.base_branch == current.base_branch
        && admitted.test_command == current.test_command
        && admitted.publish_command == current.publish_command
}

impl Store {
    /// Whether the repository configuration admitted with the assignment's newest claim or repair
    /// is no longer its repository's. The store then refuses the assignment's repair, review,
    /// readiness, merge and publication, so only cancelling or re-planning it ends it. False for
    /// an unknown assignment and for one never claimed.
    pub fn assignment_configuration_changed(&self, assignment_id: &str) -> bool {
        let Some(assignment) = self.memory.assignments.get(assignment_id) else {
            return false;
        };
        let admitted = self.memory.assignment_configs.get(assignment_id);
        let repo = self
            .memory
            .repositories
            .get(&assignment.data.repository_id.0);
        admitted
            .zip(repo)
            .is_some_and(|(admitted, repo)| !same_configuration(admitted, &repo.data))
    }

    pub(crate) fn prepare(&self, command: &str, body: &mut Value) -> Result<Option<Value>> {
        match command {
            "RegisterWorkspace" => {
                let path = discovery::discover(Path::new(text(body, "path")?))?.path;
                let path = path.to_str().context("workspace path is not UTF-8")?;
                body["path"] = json!(path);
                if let Some(row) = self
                    .memory
                    .workspaces
                    .values()
                    .find(|row| row.data.path == path)
                {
                    ensure!(row.state == W::Registered, "workspace was archived");
                    return Ok(self
                        .memory
                        .registration_receipts
                        .get(&registration_key(command, body).expect("registration key"))
                        .cloned());
                }
            }
            "RegisterRepository" => {
                let repo = discovery::repository(Path::new(text(body, "path")?))?;
                body["path"] = json!(repo.path);
                body["common_dir"] = json!(repo.common_dir);
                if self.memory.repositories.values().any(|row| {
                    row.data.workspace_id.0 == body["workspace_id"]
                        && row.data.common_dir == body["common_dir"]
                }) {
                    return Ok(self
                        .memory
                        .registration_receipts
                        .get(&registration_key(command, body).expect("registration key"))
                        .cloned());
                }
            }
            _ => {}
        }
        Ok(None)
    }

    pub(crate) fn guard(&self, command: &str, body: &Value) -> Result<()> {
        if command == "DeleteGoal" {
            let goal = text(body, "goal_id")?;
            ensure!(
                self.memory
                    .assignments
                    .values()
                    .all(|assignment| assignment.data.goal_id.0 != goal),
                "goal has assignment history; remove assignments before deleting the goal"
            );
        }
        if matches!(command, "RegisterRepository" | "CreateGoal") {
            let workspace = self
                .memory
                .workspaces
                .get(text(body, "workspace_id")?)
                .context("workspace not registered")?;
            ensure!(workspace.state == W::Registered, "workspace is archived");
        }
        // CreateGoal declares its limits; UpdateGoal's stay here, because an input guard in ESS
        // would answer before a finished goal's declared `satisfied` or `cancelled` refusal.
        if command == "UpdateGoal" {
            for field in ["max_workers", "max_attempts", "max_minutes"] {
                ensure!(
                    body[field].as_i64().is_some_and(|n| n > 0),
                    "{field} must be positive"
                );
            }
        }
        if matches!(command, "CreateGoal" | "UpdateGoal") {
            ensure!(
                !text(body, "objective")?.trim().is_empty(),
                "goal objective is empty"
            );
            ensure!(
                !text(body, "acceptance")?.trim().is_empty(),
                "goal acceptance is empty"
            );
        }
        if command == "StartGoal"
            && let Some(goal) = self.memory.goals.get(text(body, "goal_id")?)
        {
            self.require_active_workspace(&goal.data.workspace_id.0)?;
            ensure!(
                !self
                    .memory
                    .goals
                    .values()
                    .any(|other| other.state == G::Running
                        && other.data.workspace_id == goal.data.workspace_id
                        && other.data.goal_id != goal.data.goal_id),
                "workspace already has an active goal"
            );
        }
        if command == "QueueAssignment" {
            let goal = self
                .memory
                .goals
                .get(text(body, "goal_id")?)
                .context("goal not found")?;
            let repo = self
                .memory
                .repositories
                .get(text(body, "repository_id")?)
                .context("repository not found")?;
            self.require_active_workspace(&goal.data.workspace_id.0)?;
            ensure!(
                repo.state == R::Registered && repo.data.workspace_id == goal.data.workspace_id,
                "repository is disabled or belongs to another workspace"
            );
            ensure!(
                !self
                    .memory
                    .assignments
                    .values()
                    .any(|row| row.data.goal_id == goal.data.goal_id
                        && row.data.story_id == body["story_id"]
                        && row.state != A::Cancelled),
                "story already has an assignment"
            );
        }
        if let Some(id) = body["assignment_id"].as_str() {
            if let Some(assignment) = self.memory.assignments.get(id) {
                self.guard_assignment(command, body, assignment)?;
            } else if command == "PreparePublication" {
                anyhow::bail!("assignment not found");
            }
        }
        // A receipt or reason is checked only for an open intent: an unknown or settled one keeps
        // its declared `not-found` or `wrong-state` answer.
        let open = body["publication_id"]
            .as_str()
            .and_then(|id| self.memory.publications.get(id))
            .is_some_and(|p| matches!(p.state, P::Prepared | P::Uncertain));
        if command == "ConfirmPublication" && open {
            ensure!(
                !text(body, "receipt")?.is_empty(),
                "publication receipt is empty"
            );
        }
        if command == "ClosePublication" && open {
            ensure!(
                !text(body, "reason")?.trim().is_empty(),
                "publication close reason is empty"
            );
        }
        if command == "SatisfyGoal" {
            ensure!(
                !text(body, "satisfaction_receipt")?.is_empty(),
                "goal satisfaction requires a receipt"
            );
            ensure!(
                self.memory
                    .assignments
                    .values()
                    .filter(|row| row.data.goal_id.0 == body["goal_id"])
                    .all(|row| matches!(row.state, A::Merged | A::Cancelled)),
                "goal has unfinished assignments"
            );
            // Acceptance checks a goal, then releases the store before it records satisfaction,
            // so it names the revision it checked in `receipt_revision`, and the generated
            // `stale-revision` refusal compares that with the goal's current revision. The input
            // is optional only so that decisions recorded before it existed replay; a new
            // satisfaction of a Running goal must name it, and the receipt must name the same
            // revision. Any other goal keeps its declared refusal.
            if let Some(goal) = body["goal_id"]
                .as_str()
                .and_then(|id| self.memory.goals.get(id))
                && goal.state == G::Running
            {
                let current = goal.data.revision;
                let checked = body["receipt_revision"].as_i64().with_context(|| {
                    format!(
                        "goal satisfaction names no integer receipt_revision; the goal is at \
                         revision {current}"
                    )
                })?;
                let receipt = serde_json::from_str::<Value>(text(body, "satisfaction_receipt")?)
                    .ok()
                    .filter(Value::is_object)
                    .with_context(|| {
                        format!(
                            "goal satisfaction receipt is not a JSON object naming its \
                             goal_revision; receipt_revision is {checked}"
                        )
                    })?;
                let named = receipt.get("goal_revision").with_context(|| {
                    format!(
                        "goal satisfaction receipt names no goal_revision; receipt_revision is \
                         {checked}"
                    )
                })?;
                ensure!(
                    named.as_i64() == Some(checked),
                    "goal satisfaction receipt names goal revision {named} but receipt_revision \
                     is {checked}"
                );
            }
        }
        if command == "ArchiveWorkspace" {
            ensure!(
                !self
                    .memory
                    .goals
                    .values()
                    .any(|goal| goal.data.workspace_id.0 == body["workspace_id"]
                        && goal.state == G::Running),
                "workspace has an active goal"
            );
        }
        Ok(())
    }

    fn guard_assignment(
        &self,
        command: &str,
        body: &Value,
        assignment: &AssignmentSnapshot,
    ) -> Result<()> {
        let data = &assignment.data;
        let goal = self
            .memory
            .goals
            .get(&data.goal_id.0)
            .context("assignment goal missing")?;
        let repo = self
            .memory
            .repositories
            .get(&data.repository_id.0)
            .context("assignment repository missing")?;
        if matches!(
            command,
            "ClaimAssignment"
                | "RepairAssignment"
                | "ReviewAssignment"
                | "ReadyAssignment"
                | "MergeAssignment"
                | "PreparePublication"
        ) {
            self.require_active_workspace(&goal.data.workspace_id.0)?;
            ensure!(
                goal.state == G::Running && data.goal_revision == goal.data.revision,
                "goal is paused, terminal, or changed"
            );
            ensure!(repo.state == R::Registered, "repository is disabled");
            if command != "ClaimAssignment" {
                let admitted = self
                    .memory
                    .assignment_configs
                    .get(&data.assignment_id.0)
                    .context("assignment has no admitted repository configuration")?;
                ensure!(
                    same_configuration(admitted, &repo.data),
                    "repository configuration changed; assignment evidence is stale"
                );
            }
        }
        if matches!(command, "ClaimAssignment" | "RepairAssignment") {
            ensure!(
                data.attempt < goal.data.max_attempts,
                "assignment attempt limit reached"
            );
            ensure!(
                !self.memory.assignments.values().any(|other| {
                    other.data.assignment_id != data.assignment_id
                        && active(other.state)
                        && self
                            .memory
                            .repositories
                            .get(&other.data.repository_id.0)
                            .is_some_and(|r| r.data.common_dir == repo.data.common_dir)
                }),
                "repository already has an active change"
            );
            if command == "ClaimAssignment"
                || (command == "RepairAssignment" && assignment.state == A::Blocked)
            {
                let count = self
                    .memory
                    .assignments
                    .values()
                    .filter(|a| a.data.goal_id == data.goal_id && occupies_worker(a.state))
                    .count();
                ensure!(
                    count < goal.data.max_workers as usize,
                    "goal worker limit reached"
                );
            }
        }
        if matches!(command, "RepairAssignment" | "CancelAssignment") {
            ensure!(
                !self
                    .memory
                    .publications
                    .values()
                    .any(|p| p.data.assignment_id == data.assignment_id && holds(p.state)),
                "publication must be reconciled or closed before repair or cancellation"
            );
        }
        if matches!(command, "MergeAssignment" | "PreparePublication") {
            ensure!(goal.data.merge_authority, "goal does not authorize merging");
            ensure!(
                !data.candidate.is_empty()
                    && data.test_revision == data.candidate
                    && data.review_revision == data.candidate
                    && !data.reviewer_run.is_empty()
                    && data.reviewer_run != data.implementor_run,
                "current independent review and tests are required"
            );
        }
        if command == "PreparePublication" {
            ensure!(
                assignment.state == A::ReadyToMerge,
                "assignment is not ready to publish"
            );
            ensure!(
                text(body, "candidate")? == data.candidate
                    && text(body, "expected_base")? == data.base_revision
                    && text(body, "target")? == repo.data.base_branch,
                "publication candidate, base, or target does not match assignment"
            );
            ensure!(
                !self
                    .memory
                    .publications
                    .values()
                    .any(|p| p.data.assignment_id == data.assignment_id && holds(p.state)),
                "publication already exists; reconcile it"
            );
        }
        if command == "MergeAssignment" {
            ensure!(
                self.memory
                    .publications
                    .values()
                    .any(|p| p.data.assignment_id == data.assignment_id
                        && p.data.candidate == data.candidate
                        && p.data.target == repo.data.base_branch
                        && p.data.expected_base == data.base_revision
                        && p.state == P::Prepared),
                "durable publication intent is required"
            );
        }
        if matches!(command, "CompleteAssignment" | "ReconcileAssignment") {
            let receipt = text(body, "merge_receipt")?;
            ensure!(
                !receipt.is_empty()
                    && self
                        .memory
                        .publications
                        .values()
                        .any(|p| p.data.assignment_id == data.assignment_id
                            && p.state == P::Confirmed
                            && p.data.candidate == data.candidate
                            && p.data.receipt == receipt),
                "confirmed publication receipt is required"
            );
        }
        Ok(())
    }

    fn require_active_workspace(&self, id: &str) -> Result<()> {
        ensure!(
            self.memory
                .workspaces
                .get(id)
                .is_some_and(|workspace| workspace.state == W::Registered),
            "workspace is archived or missing"
        );
        Ok(())
    }
}
