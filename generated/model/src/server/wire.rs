// generated from controlplane v1
// model digest 8e307f3ce0541f736b4688846bf3bc3617af6ba4bd43e0b156673e614f1f8a57
// contract digest c4a296ae41814f3a2a24c5f55da9b458369ad96cbca829869fd81211af1fd1ed
// do not edit: regenerate with `ess synthesize --layout crate`
//! Every generated declaration, as JSON, in the renderings the published wire contracts fix.
//!
//! Generated from the model beside the types it crosses, so a field renamed in the specification
//! is renamed here in the same regeneration. An absent optional field is omitted rather
//! than sent as `null`, which is what the `required` list of the published schema says.

use crate::server::json;

/// Writes `controlplane.host.Assignment.State` as JSON.
pub fn encode_controlplane_host_assignment_state(value: &crate::host::AssignmentState, out: &mut String) {
    match value {
        crate::host::AssignmentState::Blocked => json::push_text(out, "Blocked"),
        crate::host::AssignmentState::Cancelled => json::push_text(out, "Cancelled"),
        crate::host::AssignmentState::Implementing => json::push_text(out, "Implementing"),
        crate::host::AssignmentState::Merged => json::push_text(out, "Merged"),
        crate::host::AssignmentState::Merging => json::push_text(out, "Merging"),
        crate::host::AssignmentState::Queued => json::push_text(out, "Queued"),
        crate::host::AssignmentState::ReadyToMerge => json::push_text(out, "ReadyToMerge"),
        crate::host::AssignmentState::Reviewing => json::push_text(out, "Reviewing"),
    }
}

/// Reads `controlplane.host.Assignment.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_controlplane_host_assignment_state(value: &json::Value, at: &str) -> Result<crate::host::AssignmentState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Blocked`, `Cancelled`, `Implementing`, `Merged`, `Merging`, `Queued`, `ReadyToMerge`, `Reviewing`")? {
        "Blocked" => crate::host::AssignmentState::Blocked,
        "Cancelled" => crate::host::AssignmentState::Cancelled,
        "Implementing" => crate::host::AssignmentState::Implementing,
        "Merged" => crate::host::AssignmentState::Merged,
        "Merging" => crate::host::AssignmentState::Merging,
        "Queued" => crate::host::AssignmentState::Queued,
        "ReadyToMerge" => crate::host::AssignmentState::ReadyToMerge,
        "Reviewing" => crate::host::AssignmentState::Reviewing,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Blocked`, `Cancelled`, `Implementing`, `Merged`, `Merging`, `Queued`, `ReadyToMerge`, `Reviewing`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `controlplane.host.Goal.State` as JSON.
pub fn encode_controlplane_host_goal_state(value: &crate::host::GoalState, out: &mut String) {
    match value {
        crate::host::GoalState::Cancelled => json::push_text(out, "Cancelled"),
        crate::host::GoalState::Paused => json::push_text(out, "Paused"),
        crate::host::GoalState::Running => json::push_text(out, "Running"),
        crate::host::GoalState::Satisfied => json::push_text(out, "Satisfied"),
    }
}

/// Reads `controlplane.host.Goal.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_controlplane_host_goal_state(value: &json::Value, at: &str) -> Result<crate::host::GoalState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Cancelled`, `Paused`, `Running`, `Satisfied`")? {
        "Cancelled" => crate::host::GoalState::Cancelled,
        "Paused" => crate::host::GoalState::Paused,
        "Running" => crate::host::GoalState::Running,
        "Satisfied" => crate::host::GoalState::Satisfied,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Cancelled`, `Paused`, `Running`, `Satisfied`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `controlplane.host.RepositoryRegistration.State` as JSON.
pub fn encode_controlplane_host_repository_registration_state(value: &crate::host::RepositoryRegistrationState, out: &mut String) {
    match value {
        crate::host::RepositoryRegistrationState::Disabled => json::push_text(out, "Disabled"),
        crate::host::RepositoryRegistrationState::Registered => json::push_text(out, "Registered"),
    }
}

/// Reads `controlplane.host.RepositoryRegistration.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_controlplane_host_repository_registration_state(value: &json::Value, at: &str) -> Result<crate::host::RepositoryRegistrationState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Disabled`, `Registered`")? {
        "Disabled" => crate::host::RepositoryRegistrationState::Disabled,
        "Registered" => crate::host::RepositoryRegistrationState::Registered,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Disabled`, `Registered`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `controlplane.host.Workspace.State` as JSON.
pub fn encode_controlplane_host_workspace_state(value: &crate::host::WorkspaceState, out: &mut String) {
    match value {
        crate::host::WorkspaceState::Archived => json::push_text(out, "Archived"),
        crate::host::WorkspaceState::Registered => json::push_text(out, "Registered"),
    }
}

/// Reads `controlplane.host.Workspace.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_controlplane_host_workspace_state(value: &json::Value, at: &str) -> Result<crate::host::WorkspaceState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Archived`, `Registered`")? {
        "Archived" => crate::host::WorkspaceState::Archived,
        "Registered" => crate::host::WorkspaceState::Registered,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Archived`, `Registered`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes the event `controlplane.host.ArchiveWorkspaceApplied` as JSON.
pub fn encode_event_controlplane_host_archive_workspace_applied(value: &crate::host::ArchiveWorkspaceApplied, out: &mut String) {
    out.push('{');
    json::member(out, "workspace_id");
    json::push_text(out, &value.workspace_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.AssignmentCreated` as JSON.
pub fn encode_event_controlplane_host_assignment_created(value: &crate::host::AssignmentCreated, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    json::member(out, "repository_id");
    json::push_text(out, &value.repository_id.0);
    json::member(out, "story_id");
    json::push_text(out, &value.story_id);
    json::member(out, "case_id");
    json::push_text(out, &value.case_id);
    json::member(out, "worktree_id");
    json::push_text(out, &value.worktree_id);
    json::member(out, "candidate");
    json::push_text(out, &value.candidate);
    json::member(out, "attempt");
    json::push_integer(out, value.attempt);
    json::member(out, "reason");
    json::push_text(out, &value.reason);
    json::member(out, "implementor_run");
    json::push_text(out, &value.implementor_run);
    json::member(out, "reviewer_run");
    json::push_text(out, &value.reviewer_run);
    out.push('}');
}

/// Writes the event `controlplane.host.BlockAssignmentApplied` as JSON.
pub fn encode_event_controlplane_host_block_assignment_applied(value: &crate::host::BlockAssignmentApplied, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.CancelAssignmentApplied` as JSON.
pub fn encode_event_controlplane_host_cancel_assignment_applied(value: &crate::host::CancelAssignmentApplied, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.CancelGoalApplied` as JSON.
pub fn encode_event_controlplane_host_cancel_goal_applied(value: &crate::host::CancelGoalApplied, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.ClaimAssignmentApplied` as JSON.
pub fn encode_event_controlplane_host_claim_assignment_applied(value: &crate::host::ClaimAssignmentApplied, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.CompleteAssignmentApplied` as JSON.
pub fn encode_event_controlplane_host_complete_assignment_applied(value: &crate::host::CompleteAssignmentApplied, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.DisableRepositoryRegistrationApplied` as JSON.
pub fn encode_event_controlplane_host_disable_repository_registration_applied(value: &crate::host::DisableRepositoryRegistrationApplied, out: &mut String) {
    out.push('{');
    json::member(out, "repository_id");
    json::push_text(out, &value.repository_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.EnableRepositoryRegistrationApplied` as JSON.
pub fn encode_event_controlplane_host_enable_repository_registration_applied(value: &crate::host::EnableRepositoryRegistrationApplied, out: &mut String) {
    out.push('{');
    json::member(out, "repository_id");
    json::push_text(out, &value.repository_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.GoalCreated` as JSON.
pub fn encode_event_controlplane_host_goal_created(value: &crate::host::GoalCreated, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    json::member(out, "workspace_id");
    json::push_text(out, &value.workspace_id.0);
    json::member(out, "objective");
    json::push_text(out, &value.objective);
    json::member(out, "acceptance");
    json::push_text(out, &value.acceptance);
    json::member(out, "max_workers");
    json::push_integer(out, value.max_workers);
    json::member(out, "max_attempts");
    json::push_integer(out, value.max_attempts);
    json::member(out, "max_minutes");
    json::push_integer(out, value.max_minutes);
    json::member(out, "planner_model");
    json::push_text(out, &value.planner_model);
    json::member(out, "implementor_model");
    json::push_text(out, &value.implementor_model);
    json::member(out, "reviewer_model");
    json::push_text(out, &value.reviewer_model);
    json::member(out, "merge_authority");
    json::push_bool(out, value.merge_authority);
    out.push('}');
}

/// Writes the event `controlplane.host.MergeAssignmentApplied` as JSON.
pub fn encode_event_controlplane_host_merge_assignment_applied(value: &crate::host::MergeAssignmentApplied, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.PauseGoalApplied` as JSON.
pub fn encode_event_controlplane_host_pause_goal_applied(value: &crate::host::PauseGoalApplied, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.ReadyAssignmentApplied` as JSON.
pub fn encode_event_controlplane_host_ready_assignment_applied(value: &crate::host::ReadyAssignmentApplied, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.RepairAssignmentApplied` as JSON.
pub fn encode_event_controlplane_host_repair_assignment_applied(value: &crate::host::RepairAssignmentApplied, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.RepositoryRegistrationCreated` as JSON.
pub fn encode_event_controlplane_host_repository_registration_created(value: &crate::host::RepositoryRegistrationCreated, out: &mut String) {
    out.push('{');
    json::member(out, "repository_id");
    json::push_text(out, &value.repository_id.0);
    json::member(out, "workspace_id");
    json::push_text(out, &value.workspace_id.0);
    json::member(out, "name");
    json::push_text(out, &value.name);
    json::member(out, "path");
    json::push_text(out, &value.path);
    json::member(out, "common_dir");
    json::push_text(out, &value.common_dir);
    json::member(out, "base_branch");
    json::push_text(out, &value.base_branch);
    json::member(out, "test_command");
    json::push_text(out, &value.test_command);
    json::member(out, "publish_command");
    json::push_text(out, &value.publish_command);
    out.push('}');
}

/// Writes the event `controlplane.host.ReviewAssignmentApplied` as JSON.
pub fn encode_event_controlplane_host_review_assignment_applied(value: &crate::host::ReviewAssignmentApplied, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.SatisfyGoalApplied` as JSON.
pub fn encode_event_controlplane_host_satisfy_goal_applied(value: &crate::host::SatisfyGoalApplied, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.StartGoalApplied` as JSON.
pub fn encode_event_controlplane_host_start_goal_applied(value: &crate::host::StartGoalApplied, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.WorkspaceCreated` as JSON.
pub fn encode_event_controlplane_host_workspace_created(value: &crate::host::WorkspaceCreated, out: &mut String) {
    out.push('{');
    json::member(out, "workspace_id");
    json::push_text(out, &value.workspace_id.0);
    json::member(out, "path");
    json::push_text(out, &value.path);
    json::member(out, "name");
    json::push_text(out, &value.name);
    out.push('}');
}

/// Writes the declared error `controlplane.host.AssignmentNotFound` as JSON.
pub fn encode_error_controlplane_host_assignment_not_found(_value: &crate::host::AssignmentNotFound, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `controlplane.host.AssignmentStateConflict` as JSON.
pub fn encode_error_controlplane_host_assignment_state_conflict(value: &crate::host::AssignmentStateConflict, out: &mut String) {
    out.push('{');
    json::member(out, "state");
    encode_controlplane_host_assignment_state(&value.state, out);
    out.push('}');
}

/// Writes the declared error `controlplane.host.GoalNotFound` as JSON.
pub fn encode_error_controlplane_host_goal_not_found(_value: &crate::host::GoalNotFound, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `controlplane.host.GoalStateConflict` as JSON.
pub fn encode_error_controlplane_host_goal_state_conflict(value: &crate::host::GoalStateConflict, out: &mut String) {
    out.push('{');
    json::member(out, "state");
    encode_controlplane_host_goal_state(&value.state, out);
    out.push('}');
}

/// Writes the declared error `controlplane.host.RepositoryRegistrationNotFound` as JSON.
pub fn encode_error_controlplane_host_repository_registration_not_found(_value: &crate::host::RepositoryRegistrationNotFound, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `controlplane.host.RepositoryRegistrationStateConflict` as JSON.
pub fn encode_error_controlplane_host_repository_registration_state_conflict(value: &crate::host::RepositoryRegistrationStateConflict, out: &mut String) {
    out.push('{');
    json::member(out, "state");
    encode_controlplane_host_repository_registration_state(&value.state, out);
    out.push('}');
}

/// Writes the declared error `controlplane.host.WorkspaceNotFound` as JSON.
pub fn encode_error_controlplane_host_workspace_not_found(_value: &crate::host::WorkspaceNotFound, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `controlplane.host.WorkspaceStateConflict` as JSON.
pub fn encode_error_controlplane_host_workspace_state_conflict(value: &crate::host::WorkspaceStateConflict, out: &mut String) {
    out.push('{');
    json::member(out, "state");
    encode_controlplane_host_workspace_state(&value.state, out);
    out.push('}');
}

/// Writes one row of the view `controlplane.host.AssignmentList` as JSON.
pub fn encode_view_controlplane_host_assignment_list(value: &crate::host::AssignmentList, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    json::member(out, "repository_id");
    json::push_text(out, &value.repository_id.0);
    json::member(out, "story_id");
    json::push_text(out, &value.story_id);
    json::member(out, "case_id");
    json::push_text(out, &value.case_id);
    json::member(out, "worktree_id");
    json::push_text(out, &value.worktree_id);
    json::member(out, "candidate");
    json::push_text(out, &value.candidate);
    json::member(out, "attempt");
    json::push_integer(out, value.attempt);
    json::member(out, "reason");
    json::push_text(out, &value.reason);
    json::member(out, "implementor_run");
    json::push_text(out, &value.implementor_run);
    json::member(out, "reviewer_run");
    json::push_text(out, &value.reviewer_run);
    json::member(out, "state");
    encode_controlplane_host_assignment_state(&value.state, out);
    out.push('}');
}

/// Writes one row of the view `controlplane.host.GoalList` as JSON.
pub fn encode_view_controlplane_host_goal_list(value: &crate::host::GoalList, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    json::member(out, "workspace_id");
    json::push_text(out, &value.workspace_id.0);
    json::member(out, "objective");
    json::push_text(out, &value.objective);
    json::member(out, "acceptance");
    json::push_text(out, &value.acceptance);
    json::member(out, "max_workers");
    json::push_integer(out, value.max_workers);
    json::member(out, "max_attempts");
    json::push_integer(out, value.max_attempts);
    json::member(out, "max_minutes");
    json::push_integer(out, value.max_minutes);
    json::member(out, "planner_model");
    json::push_text(out, &value.planner_model);
    json::member(out, "implementor_model");
    json::push_text(out, &value.implementor_model);
    json::member(out, "reviewer_model");
    json::push_text(out, &value.reviewer_model);
    json::member(out, "merge_authority");
    json::push_bool(out, value.merge_authority);
    json::member(out, "state");
    encode_controlplane_host_goal_state(&value.state, out);
    out.push('}');
}

/// Writes one row of the view `controlplane.host.RepositoryRegistrationList` as JSON.
pub fn encode_view_controlplane_host_repository_registration_list(value: &crate::host::RepositoryRegistrationList, out: &mut String) {
    out.push('{');
    json::member(out, "repository_id");
    json::push_text(out, &value.repository_id.0);
    json::member(out, "workspace_id");
    json::push_text(out, &value.workspace_id.0);
    json::member(out, "name");
    json::push_text(out, &value.name);
    json::member(out, "path");
    json::push_text(out, &value.path);
    json::member(out, "common_dir");
    json::push_text(out, &value.common_dir);
    json::member(out, "base_branch");
    json::push_text(out, &value.base_branch);
    json::member(out, "test_command");
    json::push_text(out, &value.test_command);
    json::member(out, "publish_command");
    json::push_text(out, &value.publish_command);
    json::member(out, "state");
    encode_controlplane_host_repository_registration_state(&value.state, out);
    out.push('}');
}

/// Writes one row of the view `controlplane.host.WorkspaceList` as JSON.
pub fn encode_view_controlplane_host_workspace_list(value: &crate::host::WorkspaceList, out: &mut String) {
    out.push('{');
    json::member(out, "workspace_id");
    json::push_text(out, &value.workspace_id.0);
    json::member(out, "path");
    json::push_text(out, &value.path);
    json::member(out, "name");
    json::push_text(out, &value.name);
    json::member(out, "state");
    encode_controlplane_host_workspace_state(&value.state, out);
    out.push('}');
}

/// Writes the input of `controlplane.host.ArchiveWorkspace` as JSON.
pub fn encode_command_controlplane_host_archive_workspace(value: &crate::host::ArchiveWorkspace, out: &mut String) {
    out.push('{');
    json::member(out, "workspace_id");
    json::push_text(out, &value.workspace_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.ArchiveWorkspace` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_archive_workspace(value: &json::Value, at: &str) -> Result<crate::host::ArchiveWorkspace, json::DecodeError> {
    Ok(crate::host::ArchiveWorkspace {
        workspace_id: {
            let at0 = json::nested(at, "workspace_id");
            let member0 = json::member_at(value, at, "workspace_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.ArchiveWorkspace` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_archive_workspace(value: &crate::host::ArchiveWorkspaceOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::ArchiveWorkspaceOutcome::Applied { archive_workspace_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.ArchiveWorkspaceApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_archive_workspace_applied(archive_workspace_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::ArchiveWorkspaceOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.WorkspaceStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_workspace_state_conflict(error, out);
            out.push('}');
        }
        crate::host::ArchiveWorkspaceOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.WorkspaceNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_workspace_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.BlockAssignment` as JSON.
pub fn encode_command_controlplane_host_block_assignment(value: &crate::host::BlockAssignment, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.BlockAssignment` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_block_assignment(value: &json::Value, at: &str) -> Result<crate::host::BlockAssignment, json::DecodeError> {
    Ok(crate::host::BlockAssignment {
        assignment_id: {
            let at0 = json::nested(at, "assignment_id");
            let member0 = json::member_at(value, at, "assignment_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.BlockAssignment` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_block_assignment(value: &crate::host::BlockAssignmentOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::BlockAssignmentOutcome::Applied { block_assignment_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.BlockAssignmentApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_block_assignment_applied(block_assignment_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::BlockAssignmentOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_state_conflict(error, out);
            out.push('}');
        }
        crate::host::BlockAssignmentOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.CancelAssignment` as JSON.
pub fn encode_command_controlplane_host_cancel_assignment(value: &crate::host::CancelAssignment, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.CancelAssignment` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_cancel_assignment(value: &json::Value, at: &str) -> Result<crate::host::CancelAssignment, json::DecodeError> {
    Ok(crate::host::CancelAssignment {
        assignment_id: {
            let at0 = json::nested(at, "assignment_id");
            let member0 = json::member_at(value, at, "assignment_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.CancelAssignment` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_cancel_assignment(value: &crate::host::CancelAssignmentOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::CancelAssignmentOutcome::Applied { cancel_assignment_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.CancelAssignmentApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_cancel_assignment_applied(cancel_assignment_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::CancelAssignmentOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_state_conflict(error, out);
            out.push('}');
        }
        crate::host::CancelAssignmentOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.CancelGoal` as JSON.
pub fn encode_command_controlplane_host_cancel_goal(value: &crate::host::CancelGoal, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.CancelGoal` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_cancel_goal(value: &json::Value, at: &str) -> Result<crate::host::CancelGoal, json::DecodeError> {
    Ok(crate::host::CancelGoal {
        goal_id: {
            let at0 = json::nested(at, "goal_id");
            let member0 = json::member_at(value, at, "goal_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.CancelGoal` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_cancel_goal(value: &crate::host::CancelGoalOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::CancelGoalOutcome::Applied { cancel_goal_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.CancelGoalApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_cancel_goal_applied(cancel_goal_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::CancelGoalOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.GoalStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_goal_state_conflict(error, out);
            out.push('}');
        }
        crate::host::CancelGoalOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.GoalNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_goal_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.ClaimAssignment` as JSON.
pub fn encode_command_controlplane_host_claim_assignment(value: &crate::host::ClaimAssignment, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.ClaimAssignment` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_claim_assignment(value: &json::Value, at: &str) -> Result<crate::host::ClaimAssignment, json::DecodeError> {
    Ok(crate::host::ClaimAssignment {
        assignment_id: {
            let at0 = json::nested(at, "assignment_id");
            let member0 = json::member_at(value, at, "assignment_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.ClaimAssignment` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_claim_assignment(value: &crate::host::ClaimAssignmentOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::ClaimAssignmentOutcome::Applied { claim_assignment_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.ClaimAssignmentApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_claim_assignment_applied(claim_assignment_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::ClaimAssignmentOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_state_conflict(error, out);
            out.push('}');
        }
        crate::host::ClaimAssignmentOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.CompleteAssignment` as JSON.
pub fn encode_command_controlplane_host_complete_assignment(value: &crate::host::CompleteAssignment, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.CompleteAssignment` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_complete_assignment(value: &json::Value, at: &str) -> Result<crate::host::CompleteAssignment, json::DecodeError> {
    Ok(crate::host::CompleteAssignment {
        assignment_id: {
            let at0 = json::nested(at, "assignment_id");
            let member0 = json::member_at(value, at, "assignment_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.CompleteAssignment` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_complete_assignment(value: &crate::host::CompleteAssignmentOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::CompleteAssignmentOutcome::Applied { complete_assignment_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.CompleteAssignmentApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_complete_assignment_applied(complete_assignment_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::CompleteAssignmentOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_state_conflict(error, out);
            out.push('}');
        }
        crate::host::CompleteAssignmentOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.CreateGoal` as JSON.
pub fn encode_command_controlplane_host_create_goal(value: &crate::host::CreateGoal, out: &mut String) {
    out.push('{');
    json::member(out, "workspace_id");
    json::push_text(out, &value.workspace_id.0);
    json::member(out, "objective");
    json::push_text(out, &value.objective);
    json::member(out, "acceptance");
    json::push_text(out, &value.acceptance);
    json::member(out, "max_workers");
    json::push_integer(out, value.max_workers);
    json::member(out, "max_attempts");
    json::push_integer(out, value.max_attempts);
    json::member(out, "max_minutes");
    json::push_integer(out, value.max_minutes);
    json::member(out, "planner_model");
    json::push_text(out, &value.planner_model);
    json::member(out, "implementor_model");
    json::push_text(out, &value.implementor_model);
    json::member(out, "reviewer_model");
    json::push_text(out, &value.reviewer_model);
    json::member(out, "merge_authority");
    json::push_bool(out, value.merge_authority);
    out.push('}');
}

/// Reads the input of `controlplane.host.CreateGoal` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_create_goal(value: &json::Value, at: &str) -> Result<crate::host::CreateGoal, json::DecodeError> {
    Ok(crate::host::CreateGoal {
        workspace_id: {
            let at0 = json::nested(at, "workspace_id");
            let member0 = json::member_at(value, at, "workspace_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
        objective: {
            let at1 = json::nested(at, "objective");
            let member1 = json::member_at(value, at, "objective")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        acceptance: {
            let at2 = json::nested(at, "acceptance");
            let member2 = json::member_at(value, at, "acceptance")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
        },
        max_workers: {
            let at3 = json::nested(at, "max_workers");
            let member3 = json::member_at(value, at, "max_workers")?;
            json::integer_at(member3, &at3, "an integer")?
        },
        max_attempts: {
            let at4 = json::nested(at, "max_attempts");
            let member4 = json::member_at(value, at, "max_attempts")?;
            json::integer_at(member4, &at4, "an integer")?
        },
        max_minutes: {
            let at5 = json::nested(at, "max_minutes");
            let member5 = json::member_at(value, at, "max_minutes")?;
            json::integer_at(member5, &at5, "an integer")?
        },
        planner_model: {
            let at6 = json::nested(at, "planner_model");
            let member6 = json::member_at(value, at, "planner_model")?;
            json::text_at(member6, &at6, "a string")?.to_owned()
        },
        implementor_model: {
            let at7 = json::nested(at, "implementor_model");
            let member7 = json::member_at(value, at, "implementor_model")?;
            json::text_at(member7, &at7, "a string")?.to_owned()
        },
        reviewer_model: {
            let at8 = json::nested(at, "reviewer_model");
            let member8 = json::member_at(value, at, "reviewer_model")?;
            json::text_at(member8, &at8, "a string")?.to_owned()
        },
        merge_authority: {
            let at9 = json::nested(at, "merge_authority");
            let member9 = json::member_at(value, at, "merge_authority")?;
            json::bool_at(member9, &at9, "a boolean")?
        },
    })
}

/// Writes the outcome of `controlplane.host.CreateGoal` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_create_goal(value: &crate::host::CreateGoalOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::CreateGoalOutcome::Created { goal_created } => {
            json::member(out, "outcome");
            json::push_text(out, "created");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.GoalCreated");
            json::member(out, "payload");
            encode_event_controlplane_host_goal_created(goal_created, out);
            out.push('}');
            out.push(']');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.DisableRepositoryRegistration` as JSON.
pub fn encode_command_controlplane_host_disable_repository_registration(value: &crate::host::DisableRepositoryRegistration, out: &mut String) {
    out.push('{');
    json::member(out, "repository_id");
    json::push_text(out, &value.repository_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.DisableRepositoryRegistration` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_disable_repository_registration(value: &json::Value, at: &str) -> Result<crate::host::DisableRepositoryRegistration, json::DecodeError> {
    Ok(crate::host::DisableRepositoryRegistration {
        repository_id: {
            let at0 = json::nested(at, "repository_id");
            let member0 = json::member_at(value, at, "repository_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.DisableRepositoryRegistration` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_disable_repository_registration(value: &crate::host::DisableRepositoryRegistrationOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::DisableRepositoryRegistrationOutcome::Applied { disable_repository_registration_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.DisableRepositoryRegistrationApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_disable_repository_registration_applied(disable_repository_registration_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::DisableRepositoryRegistrationOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.RepositoryRegistrationStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_repository_registration_state_conflict(error, out);
            out.push('}');
        }
        crate::host::DisableRepositoryRegistrationOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.RepositoryRegistrationNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_repository_registration_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.EnableRepositoryRegistration` as JSON.
pub fn encode_command_controlplane_host_enable_repository_registration(value: &crate::host::EnableRepositoryRegistration, out: &mut String) {
    out.push('{');
    json::member(out, "repository_id");
    json::push_text(out, &value.repository_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.EnableRepositoryRegistration` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_enable_repository_registration(value: &json::Value, at: &str) -> Result<crate::host::EnableRepositoryRegistration, json::DecodeError> {
    Ok(crate::host::EnableRepositoryRegistration {
        repository_id: {
            let at0 = json::nested(at, "repository_id");
            let member0 = json::member_at(value, at, "repository_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.EnableRepositoryRegistration` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_enable_repository_registration(value: &crate::host::EnableRepositoryRegistrationOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::EnableRepositoryRegistrationOutcome::Applied { enable_repository_registration_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.EnableRepositoryRegistrationApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_enable_repository_registration_applied(enable_repository_registration_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::EnableRepositoryRegistrationOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.RepositoryRegistrationStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_repository_registration_state_conflict(error, out);
            out.push('}');
        }
        crate::host::EnableRepositoryRegistrationOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.RepositoryRegistrationNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_repository_registration_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.MergeAssignment` as JSON.
pub fn encode_command_controlplane_host_merge_assignment(value: &crate::host::MergeAssignment, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.MergeAssignment` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_merge_assignment(value: &json::Value, at: &str) -> Result<crate::host::MergeAssignment, json::DecodeError> {
    Ok(crate::host::MergeAssignment {
        assignment_id: {
            let at0 = json::nested(at, "assignment_id");
            let member0 = json::member_at(value, at, "assignment_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.MergeAssignment` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_merge_assignment(value: &crate::host::MergeAssignmentOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::MergeAssignmentOutcome::Applied { merge_assignment_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.MergeAssignmentApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_merge_assignment_applied(merge_assignment_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::MergeAssignmentOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_state_conflict(error, out);
            out.push('}');
        }
        crate::host::MergeAssignmentOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.PauseGoal` as JSON.
pub fn encode_command_controlplane_host_pause_goal(value: &crate::host::PauseGoal, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.PauseGoal` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_pause_goal(value: &json::Value, at: &str) -> Result<crate::host::PauseGoal, json::DecodeError> {
    Ok(crate::host::PauseGoal {
        goal_id: {
            let at0 = json::nested(at, "goal_id");
            let member0 = json::member_at(value, at, "goal_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.PauseGoal` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_pause_goal(value: &crate::host::PauseGoalOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::PauseGoalOutcome::Applied { pause_goal_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.PauseGoalApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_pause_goal_applied(pause_goal_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::PauseGoalOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.GoalStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_goal_state_conflict(error, out);
            out.push('}');
        }
        crate::host::PauseGoalOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.GoalNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_goal_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.QueueAssignment` as JSON.
pub fn encode_command_controlplane_host_queue_assignment(value: &crate::host::QueueAssignment, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    json::member(out, "repository_id");
    json::push_text(out, &value.repository_id.0);
    json::member(out, "story_id");
    json::push_text(out, &value.story_id);
    json::member(out, "case_id");
    json::push_text(out, &value.case_id);
    json::member(out, "worktree_id");
    json::push_text(out, &value.worktree_id);
    json::member(out, "candidate");
    json::push_text(out, &value.candidate);
    json::member(out, "attempt");
    json::push_integer(out, value.attempt);
    json::member(out, "reason");
    json::push_text(out, &value.reason);
    json::member(out, "implementor_run");
    json::push_text(out, &value.implementor_run);
    json::member(out, "reviewer_run");
    json::push_text(out, &value.reviewer_run);
    out.push('}');
}

/// Reads the input of `controlplane.host.QueueAssignment` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_queue_assignment(value: &json::Value, at: &str) -> Result<crate::host::QueueAssignment, json::DecodeError> {
    Ok(crate::host::QueueAssignment {
        goal_id: {
            let at0 = json::nested(at, "goal_id");
            let member0 = json::member_at(value, at, "goal_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
        repository_id: {
            let at1 = json::nested(at, "repository_id");
            let member1 = json::member_at(value, at, "repository_id")?;
            crate::primitives::Uuid(json::uuid_at(member1, &at1, "a UUID")?.to_owned())
        },
        story_id: {
            let at2 = json::nested(at, "story_id");
            let member2 = json::member_at(value, at, "story_id")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
        },
        case_id: {
            let at3 = json::nested(at, "case_id");
            let member3 = json::member_at(value, at, "case_id")?;
            json::text_at(member3, &at3, "a string")?.to_owned()
        },
        worktree_id: {
            let at4 = json::nested(at, "worktree_id");
            let member4 = json::member_at(value, at, "worktree_id")?;
            json::text_at(member4, &at4, "a string")?.to_owned()
        },
        candidate: {
            let at5 = json::nested(at, "candidate");
            let member5 = json::member_at(value, at, "candidate")?;
            json::text_at(member5, &at5, "a string")?.to_owned()
        },
        attempt: {
            let at6 = json::nested(at, "attempt");
            let member6 = json::member_at(value, at, "attempt")?;
            json::integer_at(member6, &at6, "an integer")?
        },
        reason: {
            let at7 = json::nested(at, "reason");
            let member7 = json::member_at(value, at, "reason")?;
            json::text_at(member7, &at7, "a string")?.to_owned()
        },
        implementor_run: {
            let at8 = json::nested(at, "implementor_run");
            let member8 = json::member_at(value, at, "implementor_run")?;
            json::text_at(member8, &at8, "a string")?.to_owned()
        },
        reviewer_run: {
            let at9 = json::nested(at, "reviewer_run");
            let member9 = json::member_at(value, at, "reviewer_run")?;
            json::text_at(member9, &at9, "a string")?.to_owned()
        },
    })
}

/// Writes the outcome of `controlplane.host.QueueAssignment` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_queue_assignment(value: &crate::host::QueueAssignmentOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::QueueAssignmentOutcome::Created { assignment_created } => {
            json::member(out, "outcome");
            json::push_text(out, "created");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.AssignmentCreated");
            json::member(out, "payload");
            encode_event_controlplane_host_assignment_created(assignment_created, out);
            out.push('}');
            out.push(']');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.ReadyAssignment` as JSON.
pub fn encode_command_controlplane_host_ready_assignment(value: &crate::host::ReadyAssignment, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.ReadyAssignment` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_ready_assignment(value: &json::Value, at: &str) -> Result<crate::host::ReadyAssignment, json::DecodeError> {
    Ok(crate::host::ReadyAssignment {
        assignment_id: {
            let at0 = json::nested(at, "assignment_id");
            let member0 = json::member_at(value, at, "assignment_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.ReadyAssignment` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_ready_assignment(value: &crate::host::ReadyAssignmentOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::ReadyAssignmentOutcome::Applied { ready_assignment_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.ReadyAssignmentApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_ready_assignment_applied(ready_assignment_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::ReadyAssignmentOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_state_conflict(error, out);
            out.push('}');
        }
        crate::host::ReadyAssignmentOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.RegisterRepository` as JSON.
pub fn encode_command_controlplane_host_register_repository(value: &crate::host::RegisterRepository, out: &mut String) {
    out.push('{');
    json::member(out, "workspace_id");
    json::push_text(out, &value.workspace_id.0);
    json::member(out, "name");
    json::push_text(out, &value.name);
    json::member(out, "path");
    json::push_text(out, &value.path);
    json::member(out, "common_dir");
    json::push_text(out, &value.common_dir);
    json::member(out, "base_branch");
    json::push_text(out, &value.base_branch);
    json::member(out, "test_command");
    json::push_text(out, &value.test_command);
    json::member(out, "publish_command");
    json::push_text(out, &value.publish_command);
    out.push('}');
}

/// Reads the input of `controlplane.host.RegisterRepository` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_register_repository(value: &json::Value, at: &str) -> Result<crate::host::RegisterRepository, json::DecodeError> {
    Ok(crate::host::RegisterRepository {
        workspace_id: {
            let at0 = json::nested(at, "workspace_id");
            let member0 = json::member_at(value, at, "workspace_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
        name: {
            let at1 = json::nested(at, "name");
            let member1 = json::member_at(value, at, "name")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        path: {
            let at2 = json::nested(at, "path");
            let member2 = json::member_at(value, at, "path")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
        },
        common_dir: {
            let at3 = json::nested(at, "common_dir");
            let member3 = json::member_at(value, at, "common_dir")?;
            json::text_at(member3, &at3, "a string")?.to_owned()
        },
        base_branch: {
            let at4 = json::nested(at, "base_branch");
            let member4 = json::member_at(value, at, "base_branch")?;
            json::text_at(member4, &at4, "a string")?.to_owned()
        },
        test_command: {
            let at5 = json::nested(at, "test_command");
            let member5 = json::member_at(value, at, "test_command")?;
            json::text_at(member5, &at5, "a string")?.to_owned()
        },
        publish_command: {
            let at6 = json::nested(at, "publish_command");
            let member6 = json::member_at(value, at, "publish_command")?;
            json::text_at(member6, &at6, "a string")?.to_owned()
        },
    })
}

/// Writes the outcome of `controlplane.host.RegisterRepository` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_register_repository(value: &crate::host::RegisterRepositoryOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::RegisterRepositoryOutcome::Created { repository_registration_created } => {
            json::member(out, "outcome");
            json::push_text(out, "created");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.RepositoryRegistrationCreated");
            json::member(out, "payload");
            encode_event_controlplane_host_repository_registration_created(repository_registration_created, out);
            out.push('}');
            out.push(']');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.RegisterWorkspace` as JSON.
pub fn encode_command_controlplane_host_register_workspace(value: &crate::host::RegisterWorkspace, out: &mut String) {
    out.push('{');
    json::member(out, "path");
    json::push_text(out, &value.path);
    json::member(out, "name");
    json::push_text(out, &value.name);
    out.push('}');
}

/// Reads the input of `controlplane.host.RegisterWorkspace` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_register_workspace(value: &json::Value, at: &str) -> Result<crate::host::RegisterWorkspace, json::DecodeError> {
    Ok(crate::host::RegisterWorkspace {
        path: {
            let at0 = json::nested(at, "path");
            let member0 = json::member_at(value, at, "path")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        name: {
            let at1 = json::nested(at, "name");
            let member1 = json::member_at(value, at, "name")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
    })
}

/// Writes the outcome of `controlplane.host.RegisterWorkspace` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_register_workspace(value: &crate::host::RegisterWorkspaceOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::RegisterWorkspaceOutcome::Created { workspace_created } => {
            json::member(out, "outcome");
            json::push_text(out, "created");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.WorkspaceCreated");
            json::member(out, "payload");
            encode_event_controlplane_host_workspace_created(workspace_created, out);
            out.push('}');
            out.push(']');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.RepairAssignment` as JSON.
pub fn encode_command_controlplane_host_repair_assignment(value: &crate::host::RepairAssignment, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.RepairAssignment` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_repair_assignment(value: &json::Value, at: &str) -> Result<crate::host::RepairAssignment, json::DecodeError> {
    Ok(crate::host::RepairAssignment {
        assignment_id: {
            let at0 = json::nested(at, "assignment_id");
            let member0 = json::member_at(value, at, "assignment_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.RepairAssignment` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_repair_assignment(value: &crate::host::RepairAssignmentOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::RepairAssignmentOutcome::Applied { repair_assignment_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.RepairAssignmentApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_repair_assignment_applied(repair_assignment_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::RepairAssignmentOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_state_conflict(error, out);
            out.push('}');
        }
        crate::host::RepairAssignmentOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.ReviewAssignment` as JSON.
pub fn encode_command_controlplane_host_review_assignment(value: &crate::host::ReviewAssignment, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.ReviewAssignment` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_review_assignment(value: &json::Value, at: &str) -> Result<crate::host::ReviewAssignment, json::DecodeError> {
    Ok(crate::host::ReviewAssignment {
        assignment_id: {
            let at0 = json::nested(at, "assignment_id");
            let member0 = json::member_at(value, at, "assignment_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.ReviewAssignment` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_review_assignment(value: &crate::host::ReviewAssignmentOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::ReviewAssignmentOutcome::Applied { review_assignment_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.ReviewAssignmentApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_review_assignment_applied(review_assignment_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::ReviewAssignmentOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_state_conflict(error, out);
            out.push('}');
        }
        crate::host::ReviewAssignmentOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.AssignmentNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_assignment_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.SatisfyGoal` as JSON.
pub fn encode_command_controlplane_host_satisfy_goal(value: &crate::host::SatisfyGoal, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.SatisfyGoal` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_satisfy_goal(value: &json::Value, at: &str) -> Result<crate::host::SatisfyGoal, json::DecodeError> {
    Ok(crate::host::SatisfyGoal {
        goal_id: {
            let at0 = json::nested(at, "goal_id");
            let member0 = json::member_at(value, at, "goal_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.SatisfyGoal` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_satisfy_goal(value: &crate::host::SatisfyGoalOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::SatisfyGoalOutcome::Applied { satisfy_goal_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.SatisfyGoalApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_satisfy_goal_applied(satisfy_goal_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::SatisfyGoalOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.GoalStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_goal_state_conflict(error, out);
            out.push('}');
        }
        crate::host::SatisfyGoalOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.GoalNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_goal_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.StartGoal` as JSON.
pub fn encode_command_controlplane_host_start_goal(value: &crate::host::StartGoal, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.StartGoal` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_start_goal(value: &json::Value, at: &str) -> Result<crate::host::StartGoal, json::DecodeError> {
    Ok(crate::host::StartGoal {
        goal_id: {
            let at0 = json::nested(at, "goal_id");
            let member0 = json::member_at(value, at, "goal_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.StartGoal` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_start_goal(value: &crate::host::StartGoalOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::StartGoalOutcome::Applied { start_goal_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.StartGoalApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_start_goal_applied(start_goal_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::StartGoalOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.GoalStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_goal_state_conflict(error, out);
            out.push('}');
        }
        crate::host::StartGoalOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.GoalNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_goal_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes any event on the system's log as JSON: its qualified name and its payload,
/// `{"event": …, "payload": {…}}`, the envelope a command's answer lists it in.
pub fn encode_system_event(value: &crate::system::SystemEvent) -> String {
    let mut out = String::from("{");
    json::member(&mut out, "event");
    json::push_text(&mut out, value.name());
    json::member(&mut out, "payload");
    match value {
        crate::system::SystemEvent::ArchiveWorkspaceApplied(event) => encode_event_controlplane_host_archive_workspace_applied(event, &mut out),
        crate::system::SystemEvent::AssignmentCreated(event) => encode_event_controlplane_host_assignment_created(event, &mut out),
        crate::system::SystemEvent::BlockAssignmentApplied(event) => encode_event_controlplane_host_block_assignment_applied(event, &mut out),
        crate::system::SystemEvent::CancelAssignmentApplied(event) => encode_event_controlplane_host_cancel_assignment_applied(event, &mut out),
        crate::system::SystemEvent::CancelGoalApplied(event) => encode_event_controlplane_host_cancel_goal_applied(event, &mut out),
        crate::system::SystemEvent::ClaimAssignmentApplied(event) => encode_event_controlplane_host_claim_assignment_applied(event, &mut out),
        crate::system::SystemEvent::CompleteAssignmentApplied(event) => encode_event_controlplane_host_complete_assignment_applied(event, &mut out),
        crate::system::SystemEvent::DisableRepositoryRegistrationApplied(event) => encode_event_controlplane_host_disable_repository_registration_applied(event, &mut out),
        crate::system::SystemEvent::EnableRepositoryRegistrationApplied(event) => encode_event_controlplane_host_enable_repository_registration_applied(event, &mut out),
        crate::system::SystemEvent::GoalCreated(event) => encode_event_controlplane_host_goal_created(event, &mut out),
        crate::system::SystemEvent::MergeAssignmentApplied(event) => encode_event_controlplane_host_merge_assignment_applied(event, &mut out),
        crate::system::SystemEvent::PauseGoalApplied(event) => encode_event_controlplane_host_pause_goal_applied(event, &mut out),
        crate::system::SystemEvent::ReadyAssignmentApplied(event) => encode_event_controlplane_host_ready_assignment_applied(event, &mut out),
        crate::system::SystemEvent::RepairAssignmentApplied(event) => encode_event_controlplane_host_repair_assignment_applied(event, &mut out),
        crate::system::SystemEvent::RepositoryRegistrationCreated(event) => encode_event_controlplane_host_repository_registration_created(event, &mut out),
        crate::system::SystemEvent::ReviewAssignmentApplied(event) => encode_event_controlplane_host_review_assignment_applied(event, &mut out),
        crate::system::SystemEvent::SatisfyGoalApplied(event) => encode_event_controlplane_host_satisfy_goal_applied(event, &mut out),
        crate::system::SystemEvent::StartGoalApplied(event) => encode_event_controlplane_host_start_goal_applied(event, &mut out),
        crate::system::SystemEvent::WorkspaceCreated(event) => encode_event_controlplane_host_workspace_created(event, &mut out),
    }
    out.push('}');
    out
}
