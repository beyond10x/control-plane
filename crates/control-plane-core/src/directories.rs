//! Workspace directories retain context independently of discovered Git repositories.
use super::*;
use controlplane_model::host::{
    AssignmentState, PlanningPhase, RepositoryRegistrationState, WorkspaceDirectoryState,
    WorkspaceState,
};
use std::collections::BTreeSet;

impl Store {
    /// Add a canonical directory and newly discovered repositories in one Eventlog append.
    pub async fn add_workspace_directory(
        &mut self,
        workspace_id: &str,
        path: &Path,
    ) -> Result<Value> {
        ensure!(
            self.memory
                .workspaces
                .get(workspace_id)
                .is_some_and(|w| w.state == WorkspaceState::Registered),
            "workspace is archived or missing"
        );
        let path = path.canonicalize().context("directory does not exist")?;
        ensure!(path.is_dir(), "workspace member must be a directory");
        let canonical = path.to_str().context("directory path is not UTF-8")?;
        if self.memory.directories.values().any(|d| {
            d.data.workspace_id.0 == workspace_id
                && d.data.path == canonical
                && d.state == WorkspaceDirectoryState::Registered
        }) {
            let key = guards::registration_key(
                "AddWorkspaceDirectory",
                &json!({"workspace_id":workspace_id,"path":canonical}),
            )
            .expect("directory receipt key");
            return self
                .memory
                .registration_receipts
                .get(&key)
                .cloned()
                .context("directory registration receipt missing");
        }
        let found = discover(&path)?;
        let mut next = self.memory.clone();
        let mut decisions = Vec::new();
        let mut covered = BTreeSet::new();
        let mut managed = BTreeSet::new();
        for repo in found.repositories {
            let common = repo
                .common_dir
                .to_str()
                .context("Git common directory is not UTF-8")?
                .to_owned();
            covered.insert(common.clone());
            if !next
                .repositories
                .values()
                .any(|r| r.data.workspace_id.0 == workspace_id && r.data.common_dir == common)
            {
                stage(
                    &mut next,
                    &mut decisions,
                    "RegisterRepository",
                    json!({"workspace_id":workspace_id,"name":repo.name,"path":repo.path,"common_dir":common,"base_branch":repo.base_branch,"test_command":"task check","publish_command":""}),
                    Actor::Operator,
                )?;
                managed.insert(common);
            }
        }
        let answer = stage(
            &mut next,
            &mut decisions,
            "AddWorkspaceDirectory",
            json!({"workspace_id":workspace_id,"path":canonical,"repository_common_dirs":covered,"managed_common_dirs":managed}),
            Actor::Operator,
        )?;
        self.commit(next, decisions, Actor::Operator).await?;
        Ok(answer)
    }

    /// Remove context and disable only repositories introduced by directory membership and no
    /// longer covered anywhere in this workspace. Manual registrations remain operator-owned.
    pub async fn remove_workspace_directory(&mut self, directory_id: &str) -> Result<Value> {
        let Some(directory) = self.memory.directories.get(directory_id).cloned() else {
            return self
                .apply(
                    "RemoveWorkspaceDirectory",
                    json!({"directory_id":directory_id}),
                    Actor::Operator,
                )
                .await;
        };
        if directory.state != WorkspaceDirectoryState::Registered {
            return self
                .apply(
                    "RemoveWorkspaceDirectory",
                    json!({"directory_id":directory_id}),
                    Actor::Operator,
                )
                .await;
        }
        let workspace = &directory.data.workspace_id.0;
        let covered = &directory.data.repository_common_dirs;
        let repositories: Vec<_> = self
            .memory
            .repositories
            .values()
            .filter(|r| r.data.workspace_id.0 == *workspace && covered.contains(&r.data.common_dir))
            .cloned()
            .collect();
        ensure!(
            !self.memory.assignments.values().any(|a| !matches!(
                a.state,
                AssignmentState::Merged | AssignmentState::Cancelled
            ) && repositories
                .iter()
                .any(|r| r.data.repository_id == a.data.repository_id)),
            "directory has active assignments; finish or cancel them before removal"
        );
        ensure!(
            !self
                .memory
                .goals
                .values()
                .any(|g| g.data.workspace_id.0 == *workspace
                    && !g.data.planning_worktree_id.is_empty()
                    && matches!(
                        g.data.planning_phase,
                        PlanningPhase::Provisioning
                            | PlanningPhase::Planning
                            | PlanningPhase::Validated
                    )),
            "directory has active planning work; reconcile it before removal"
        );
        let mut next = self.memory.clone();
        let mut decisions = Vec::new();
        let answer = stage(
            &mut next,
            &mut decisions,
            "RemoveWorkspaceDirectory",
            json!({"directory_id":directory_id}),
            Actor::Operator,
        )?;
        for repo in repositories {
            let common = &repo.data.common_dir;
            let still_covered = next.directories.values().any(|d| {
                d.data.workspace_id.0 == *workspace
                    && d.state == WorkspaceDirectoryState::Registered
                    && d.data.repository_common_dirs.contains(common)
            });
            let was_managed = next.directories.values().any(|d| {
                d.data.workspace_id.0 == *workspace && d.data.managed_common_dirs.contains(common)
            });
            if !still_covered
                && was_managed
                && repo.state == RepositoryRegistrationState::Registered
            {
                stage(
                    &mut next,
                    &mut decisions,
                    "DisableRepositoryRegistration",
                    json!({"repository_id":repo.data.repository_id.0}),
                    Actor::Operator,
                )?;
            }
        }
        self.commit(next, decisions, Actor::Operator).await?;
        Ok(answer)
    }

    /// Upgrade old workspaces which have no directory history. Explicit removals are preserved.
    pub async fn backfill_workspace_directories(&mut self) -> Result<()> {
        let missing: Vec<_> = self
            .memory
            .workspaces
            .values()
            .filter(|w| {
                w.state == WorkspaceState::Registered
                    && !self
                        .memory
                        .directories
                        .values()
                        .any(|d| d.data.workspace_id == w.data.workspace_id)
            })
            .map(|w| (w.data.workspace_id.0.clone(), w.data.path.clone()))
            .collect();
        for (id, path) in missing {
            // A disconnected or moved legacy root must not prevent healthy workspaces from
            // loading. Leave its history untouched so a later startup can retry the migration.
            if !Path::new(&path).try_exists()? {
                continue;
            }
            self.add_workspace_directory(&id, Path::new(&path)).await?;
        }
        Ok(())
    }
}
