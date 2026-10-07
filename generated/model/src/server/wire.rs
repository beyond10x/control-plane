// generated from controlplane v1
// model digest 81c7f5e0230fee0d706b0ec2528b81407c999295df8491e6ebc040ba5bdb6bdf
// contract digest 003b16f133632f8fde01042faa8c7c30286cba0b098ee1c71da99eb0004dbf1a
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

/// Writes `controlplane.host.PlanningPhase` as JSON.
pub fn encode_controlplane_host_planning_phase(value: &crate::host::PlanningPhase, out: &mut String) {
    match value {
        crate::host::PlanningPhase::Idle => json::push_text(out, "Idle"),
        crate::host::PlanningPhase::Provisioning => json::push_text(out, "Provisioning"),
        crate::host::PlanningPhase::Planning => json::push_text(out, "Planning"),
        crate::host::PlanningPhase::Validated => json::push_text(out, "Validated"),
        crate::host::PlanningPhase::Queued => json::push_text(out, "Queued"),
        crate::host::PlanningPhase::Blocked => json::push_text(out, "Blocked"),
    }
}

/// Reads `controlplane.host.PlanningPhase` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_controlplane_host_planning_phase(value: &json::Value, at: &str) -> Result<crate::host::PlanningPhase, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Idle`, `Provisioning`, `Planning`, `Validated`, `Queued`, `Blocked`")? {
        "Idle" => crate::host::PlanningPhase::Idle,
        "Provisioning" => crate::host::PlanningPhase::Provisioning,
        "Planning" => crate::host::PlanningPhase::Planning,
        "Validated" => crate::host::PlanningPhase::Validated,
        "Queued" => crate::host::PlanningPhase::Queued,
        "Blocked" => crate::host::PlanningPhase::Blocked,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Idle`, `Provisioning`, `Planning`, `Validated`, `Queued`, `Blocked`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `controlplane.host.PublicationIntent.State` as JSON.
pub fn encode_controlplane_host_publication_intent_state(value: &crate::host::PublicationIntentState, out: &mut String) {
    match value {
        crate::host::PublicationIntentState::Confirmed => json::push_text(out, "Confirmed"),
        crate::host::PublicationIntentState::NotPublished => json::push_text(out, "NotPublished"),
        crate::host::PublicationIntentState::Prepared => json::push_text(out, "Prepared"),
        crate::host::PublicationIntentState::Uncertain => json::push_text(out, "Uncertain"),
    }
}

/// Reads `controlplane.host.PublicationIntent.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_controlplane_host_publication_intent_state(value: &json::Value, at: &str) -> Result<crate::host::PublicationIntentState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Confirmed`, `NotPublished`, `Prepared`, `Uncertain`")? {
        "Confirmed" => crate::host::PublicationIntentState::Confirmed,
        "NotPublished" => crate::host::PublicationIntentState::NotPublished,
        "Prepared" => crate::host::PublicationIntentState::Prepared,
        "Uncertain" => crate::host::PublicationIntentState::Uncertain,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Confirmed`, `NotPublished`, `Prepared`, `Uncertain`".to_owned(), found: format!("`{other}`") }),
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

/// Writes `controlplane.host.WorkspaceDirectory.State` as JSON.
pub fn encode_controlplane_host_workspace_directory_state(value: &crate::host::WorkspaceDirectoryState, out: &mut String) {
    match value {
        crate::host::WorkspaceDirectoryState::Registered => json::push_text(out, "Registered"),
        crate::host::WorkspaceDirectoryState::Removed => json::push_text(out, "Removed"),
    }
}

/// Reads `controlplane.host.WorkspaceDirectory.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_controlplane_host_workspace_directory_state(value: &json::Value, at: &str) -> Result<crate::host::WorkspaceDirectoryState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Registered`, `Removed`")? {
        "Registered" => crate::host::WorkspaceDirectoryState::Registered,
        "Removed" => crate::host::WorkspaceDirectoryState::Removed,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Registered`, `Removed`".to_owned(), found: format!("`{other}`") }),
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
    json::member(out, "goal_revision");
    json::push_integer(out, value.goal_revision);
    out.push('}');
}

/// Writes the event `controlplane.host.BlockAssignmentApplied` as JSON.
pub fn encode_event_controlplane_host_block_assignment_applied(value: &crate::host::BlockAssignmentApplied, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    json::member(out, "reason");
    json::push_text(out, &value.reason);
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
    json::member(out, "worktree_id");
    json::push_text(out, &value.worktree_id);
    json::member(out, "implementor_run");
    json::push_text(out, &value.implementor_run);
    json::member(out, "base_revision");
    json::push_text(out, &value.base_revision);
    out.push('}');
}

/// Writes the event `controlplane.host.ClosePublicationApplied` as JSON.
pub fn encode_event_controlplane_host_close_publication_applied(value: &crate::host::ClosePublicationApplied, out: &mut String) {
    out.push('{');
    json::member(out, "publication_id");
    json::push_text(out, &value.publication_id.0);
    json::member(out, "reason");
    json::push_text(out, &value.reason);
    out.push('}');
}

/// Writes the event `controlplane.host.CompleteAssignmentApplied` as JSON.
pub fn encode_event_controlplane_host_complete_assignment_applied(value: &crate::host::CompleteAssignmentApplied, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    json::member(out, "merge_receipt");
    json::push_text(out, &value.merge_receipt);
    out.push('}');
}

/// Writes the event `controlplane.host.ConfigureRepositoryApplied` as JSON.
pub fn encode_event_controlplane_host_configure_repository_applied(value: &crate::host::ConfigureRepositoryApplied, out: &mut String) {
    out.push('{');
    json::member(out, "repository_id");
    json::push_text(out, &value.repository_id.0);
    json::member(out, "base_branch");
    json::push_text(out, &value.base_branch);
    json::member(out, "test_command");
    json::push_text(out, &value.test_command);
    json::member(out, "publish_command");
    json::push_text(out, &value.publish_command);
    out.push('}');
}

/// Writes the event `controlplane.host.ConfirmPublicationApplied` as JSON.
pub fn encode_event_controlplane_host_confirm_publication_applied(value: &crate::host::ConfirmPublicationApplied, out: &mut String) {
    out.push('{');
    json::member(out, "publication_id");
    json::push_text(out, &value.publication_id.0);
    json::member(out, "receipt");
    json::push_text(out, &value.receipt);
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
    json::member(out, "planning_revision");
    json::push_integer(out, value.planning_revision);
    json::member(out, "planning_fingerprint");
    json::push_text(out, &value.planning_fingerprint);
    json::member(out, "planning_repository");
    json::push_text(out, &value.planning_repository);
    json::member(out, "planning_worktree_id");
    json::push_text(out, &value.planning_worktree_id);
    json::member(out, "planning_worktree_path");
    json::push_text(out, &value.planning_worktree_path);
    json::member(out, "planning_reason");
    json::push_text(out, &value.planning_reason);
    json::member(out, "planning_receipt");
    json::push_text(out, &value.planning_receipt);
    json::member(out, "planning_phase");
    encode_controlplane_host_planning_phase(&value.planning_phase, out);
    out.push('}');
}

/// Writes the event `controlplane.host.GoalDeleted` as JSON.
pub fn encode_event_controlplane_host_goal_deleted(value: &crate::host::GoalDeleted, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.MarkPublicationUncertainApplied` as JSON.
pub fn encode_event_controlplane_host_mark_publication_uncertain_applied(value: &crate::host::MarkPublicationUncertainApplied, out: &mut String) {
    out.push('{');
    json::member(out, "publication_id");
    json::push_text(out, &value.publication_id.0);
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

/// Writes the event `controlplane.host.PlanningProgressRecorded` as JSON.
pub fn encode_event_controlplane_host_planning_progress_recorded(value: &crate::host::PlanningProgressRecorded, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    json::member(out, "planning_revision");
    json::push_integer(out, value.planning_revision);
    json::member(out, "planning_fingerprint");
    json::push_text(out, &value.planning_fingerprint);
    json::member(out, "planning_repository");
    json::push_text(out, &value.planning_repository);
    json::member(out, "planning_worktree_id");
    json::push_text(out, &value.planning_worktree_id);
    json::member(out, "planning_worktree_path");
    json::push_text(out, &value.planning_worktree_path);
    json::member(out, "planning_reason");
    json::push_text(out, &value.planning_reason);
    json::member(out, "planning_receipt");
    json::push_text(out, &value.planning_receipt);
    json::member(out, "planning_phase");
    encode_controlplane_host_planning_phase(&value.planning_phase, out);
    out.push('}');
}

/// Writes the event `controlplane.host.PublicationIntentCreated` as JSON.
pub fn encode_event_controlplane_host_publication_intent_created(value: &crate::host::PublicationIntentCreated, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    json::member(out, "candidate");
    json::push_text(out, &value.candidate);
    json::member(out, "target");
    json::push_text(out, &value.target);
    json::member(out, "expected_base");
    json::push_text(out, &value.expected_base);
    json::member(out, "publication_id");
    json::push_text(out, &value.publication_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.ReadyAssignmentApplied` as JSON.
pub fn encode_event_controlplane_host_ready_assignment_applied(value: &crate::host::ReadyAssignmentApplied, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    json::member(out, "reviewer_run");
    json::push_text(out, &value.reviewer_run);
    json::member(out, "review_revision");
    json::push_text(out, &value.review_revision);
    out.push('}');
}

/// Writes the event `controlplane.host.ReconcileAssignmentApplied` as JSON.
pub fn encode_event_controlplane_host_reconcile_assignment_applied(value: &crate::host::ReconcileAssignmentApplied, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    json::member(out, "merge_receipt");
    json::push_text(out, &value.merge_receipt);
    out.push('}');
}

/// Writes the event `controlplane.host.RepairAssignmentApplied` as JSON.
pub fn encode_event_controlplane_host_repair_assignment_applied(value: &crate::host::RepairAssignmentApplied, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    json::member(out, "reason");
    json::push_text(out, &value.reason);
    json::member(out, "implementor_run");
    json::push_text(out, &value.implementor_run);
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
    json::member(out, "candidate");
    json::push_text(out, &value.candidate);
    json::member(out, "test_revision");
    json::push_text(out, &value.test_revision);
    out.push('}');
}

/// Writes the event `controlplane.host.SatisfyGoalApplied` as JSON.
pub fn encode_event_controlplane_host_satisfy_goal_applied(value: &crate::host::SatisfyGoalApplied, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    json::member(out, "satisfaction_receipt");
    json::push_text(out, &value.satisfaction_receipt);
    out.push('}');
}

/// Writes the event `controlplane.host.StartGoalApplied` as JSON.
pub fn encode_event_controlplane_host_start_goal_applied(value: &crate::host::StartGoalApplied, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    out.push('}');
}

/// Writes the event `controlplane.host.UpdateGoalApplied` as JSON.
pub fn encode_event_controlplane_host_update_goal_applied(value: &crate::host::UpdateGoalApplied, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
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

/// Writes the event `controlplane.host.WorkspaceDirectoryCreated` as JSON.
pub fn encode_event_controlplane_host_workspace_directory_created(value: &crate::host::WorkspaceDirectoryCreated, out: &mut String) {
    out.push('{');
    json::member(out, "directory_id");
    json::push_text(out, &value.directory_id.0);
    json::member(out, "workspace_id");
    json::push_text(out, &value.workspace_id.0);
    json::member(out, "path");
    json::push_text(out, &value.path);
    json::member(out, "repository_common_dirs");
    out.push('[');
    for (index0, item0) in value.repository_common_dirs.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, &*item0);
    }
    out.push(']');
    json::member(out, "managed_common_dirs");
    out.push('[');
    for (index0, item0) in value.managed_common_dirs.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, &*item0);
    }
    out.push(']');
    out.push('}');
}

/// Writes the event `controlplane.host.WorkspaceDirectoryRemoved` as JSON.
pub fn encode_event_controlplane_host_workspace_directory_removed(value: &crate::host::WorkspaceDirectoryRemoved, out: &mut String) {
    out.push('{');
    json::member(out, "directory_id");
    json::push_text(out, &value.directory_id.0);
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

/// Writes the declared error `controlplane.host.PublicationIntentNotFound` as JSON.
pub fn encode_error_controlplane_host_publication_intent_not_found(_value: &crate::host::PublicationIntentNotFound, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `controlplane.host.PublicationIntentStateConflict` as JSON.
pub fn encode_error_controlplane_host_publication_intent_state_conflict(value: &crate::host::PublicationIntentStateConflict, out: &mut String) {
    out.push('{');
    json::member(out, "state");
    encode_controlplane_host_publication_intent_state(&value.state, out);
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

/// Writes the declared error `controlplane.host.WorkspaceDirectoryNotFound` as JSON.
pub fn encode_error_controlplane_host_workspace_directory_not_found(_value: &crate::host::WorkspaceDirectoryNotFound, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `controlplane.host.WorkspaceDirectoryStateConflict` as JSON.
pub fn encode_error_controlplane_host_workspace_directory_state_conflict(value: &crate::host::WorkspaceDirectoryStateConflict, out: &mut String) {
    out.push('{');
    json::member(out, "state");
    encode_controlplane_host_workspace_directory_state(&value.state, out);
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
    json::member(out, "goal_revision");
    json::push_integer(out, value.goal_revision);
    json::member(out, "base_revision");
    json::push_text(out, &value.base_revision);
    json::member(out, "test_revision");
    json::push_text(out, &value.test_revision);
    json::member(out, "review_revision");
    json::push_text(out, &value.review_revision);
    json::member(out, "merge_receipt");
    json::push_text(out, &value.merge_receipt);
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
    json::member(out, "revision");
    json::push_integer(out, value.revision);
    json::member(out, "satisfaction_receipt");
    json::push_text(out, &value.satisfaction_receipt);
    json::member(out, "state");
    encode_controlplane_host_goal_state(&value.state, out);
    json::member(out, "planning_revision");
    json::push_integer(out, value.planning_revision);
    json::member(out, "planning_fingerprint");
    json::push_text(out, &value.planning_fingerprint);
    json::member(out, "planning_repository");
    json::push_text(out, &value.planning_repository);
    json::member(out, "planning_worktree_id");
    json::push_text(out, &value.planning_worktree_id);
    json::member(out, "planning_worktree_path");
    json::push_text(out, &value.planning_worktree_path);
    json::member(out, "planning_reason");
    json::push_text(out, &value.planning_reason);
    json::member(out, "planning_receipt");
    json::push_text(out, &value.planning_receipt);
    json::member(out, "planning_phase");
    encode_controlplane_host_planning_phase(&value.planning_phase, out);
    out.push('}');
}

/// Writes one row of the view `controlplane.host.PublicationIntentList` as JSON.
pub fn encode_view_controlplane_host_publication_intent_list(value: &crate::host::PublicationIntentList, out: &mut String) {
    out.push('{');
    json::member(out, "publication_id");
    json::push_text(out, &value.publication_id.0);
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    json::member(out, "candidate");
    json::push_text(out, &value.candidate);
    json::member(out, "target");
    json::push_text(out, &value.target);
    json::member(out, "expected_base");
    json::push_text(out, &value.expected_base);
    json::member(out, "receipt");
    json::push_text(out, &value.receipt);
    if let Some(held0) = &value.reason {
        json::member(out, "reason");
        json::push_text(out, &*held0);
    }
    json::member(out, "state");
    encode_controlplane_host_publication_intent_state(&value.state, out);
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

/// Writes one row of the view `controlplane.host.WorkspaceDirectoryList` as JSON.
pub fn encode_view_controlplane_host_workspace_directory_list(value: &crate::host::WorkspaceDirectoryList, out: &mut String) {
    out.push('{');
    json::member(out, "directory_id");
    json::push_text(out, &value.directory_id.0);
    json::member(out, "workspace_id");
    json::push_text(out, &value.workspace_id.0);
    json::member(out, "path");
    json::push_text(out, &value.path);
    json::member(out, "repository_common_dirs");
    out.push('[');
    for (index0, item0) in value.repository_common_dirs.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, &*item0);
    }
    out.push(']');
    json::member(out, "managed_common_dirs");
    out.push('[');
    for (index0, item0) in value.managed_common_dirs.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, &*item0);
    }
    out.push(']');
    json::member(out, "state");
    encode_controlplane_host_workspace_directory_state(&value.state, out);
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

/// Writes the input of `controlplane.host.AddWorkspaceDirectory` as JSON.
pub fn encode_command_controlplane_host_add_workspace_directory(value: &crate::host::AddWorkspaceDirectory, out: &mut String) {
    out.push('{');
    json::member(out, "workspace_id");
    json::push_text(out, &value.workspace_id.0);
    json::member(out, "path");
    json::push_text(out, &value.path);
    json::member(out, "repository_common_dirs");
    out.push('[');
    for (index0, item0) in value.repository_common_dirs.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, &*item0);
    }
    out.push(']');
    json::member(out, "managed_common_dirs");
    out.push('[');
    for (index0, item0) in value.managed_common_dirs.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, &*item0);
    }
    out.push(']');
    out.push('}');
}

/// Reads the input of `controlplane.host.AddWorkspaceDirectory` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_add_workspace_directory(value: &json::Value, at: &str) -> Result<crate::host::AddWorkspaceDirectory, json::DecodeError> {
    Ok(crate::host::AddWorkspaceDirectory {
        workspace_id: {
            let at0 = json::nested(at, "workspace_id");
            let member0 = json::member_at(value, at, "workspace_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
        path: {
            let at1 = json::nested(at, "path");
            let member1 = json::member_at(value, at, "path")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        repository_common_dirs: {
            let at2 = json::nested(at, "repository_common_dirs");
            let member2 = json::member_at(value, at, "repository_common_dirs")?;
            {
                let mut items2 = Vec::new();
                for (index2, element2) in json::items_at(member2, &at2, "an array")?.iter().enumerate() {
                    let nested2 = json::nested(&at2, &index2.to_string());
                    items2.push(json::text_at(element2, &nested2, "a string")?.to_owned());
                }
                items2
            }
        },
        managed_common_dirs: {
            let at3 = json::nested(at, "managed_common_dirs");
            let member3 = json::member_at(value, at, "managed_common_dirs")?;
            {
                let mut items3 = Vec::new();
                for (index3, element3) in json::items_at(member3, &at3, "an array")?.iter().enumerate() {
                    let nested3 = json::nested(&at3, &index3.to_string());
                    items3.push(json::text_at(element3, &nested3, "a string")?.to_owned());
                }
                items3
            }
        },
    })
}

/// Writes the outcome of `controlplane.host.AddWorkspaceDirectory` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_add_workspace_directory(value: &crate::host::AddWorkspaceDirectoryOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::AddWorkspaceDirectoryOutcome::Created { workspace_directory_created } => {
            json::member(out, "outcome");
            json::push_text(out, "created");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.WorkspaceDirectoryCreated");
            json::member(out, "payload");
            encode_event_controlplane_host_workspace_directory_created(workspace_directory_created, out);
            out.push('}');
            out.push(']');
        }
    }
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
    json::member(out, "reason");
    json::push_text(out, &value.reason);
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
        reason: {
            let at1 = json::nested(at, "reason");
            let member1 = json::member_at(value, at, "reason")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
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
    json::member(out, "worktree_id");
    json::push_text(out, &value.worktree_id);
    json::member(out, "implementor_run");
    json::push_text(out, &value.implementor_run);
    json::member(out, "base_revision");
    json::push_text(out, &value.base_revision);
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
        worktree_id: {
            let at1 = json::nested(at, "worktree_id");
            let member1 = json::member_at(value, at, "worktree_id")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        implementor_run: {
            let at2 = json::nested(at, "implementor_run");
            let member2 = json::member_at(value, at, "implementor_run")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
        },
        base_revision: {
            let at3 = json::nested(at, "base_revision");
            let member3 = json::member_at(value, at, "base_revision")?;
            json::text_at(member3, &at3, "a string")?.to_owned()
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

/// Writes the input of `controlplane.host.ClosePublication` as JSON.
pub fn encode_command_controlplane_host_close_publication(value: &crate::host::ClosePublication, out: &mut String) {
    out.push('{');
    json::member(out, "publication_id");
    json::push_text(out, &value.publication_id.0);
    json::member(out, "reason");
    json::push_text(out, &value.reason);
    out.push('}');
}

/// Reads the input of `controlplane.host.ClosePublication` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_close_publication(value: &json::Value, at: &str) -> Result<crate::host::ClosePublication, json::DecodeError> {
    Ok(crate::host::ClosePublication {
        publication_id: {
            let at0 = json::nested(at, "publication_id");
            let member0 = json::member_at(value, at, "publication_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
        reason: {
            let at1 = json::nested(at, "reason");
            let member1 = json::member_at(value, at, "reason")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
    })
}

/// Writes the outcome of `controlplane.host.ClosePublication` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_close_publication(value: &crate::host::ClosePublicationOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::ClosePublicationOutcome::Applied { close_publication_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.ClosePublicationApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_close_publication_applied(close_publication_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::ClosePublicationOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.PublicationIntentNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_publication_intent_not_found(error, out);
            out.push('}');
        }
        crate::host::ClosePublicationOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.PublicationIntentStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_publication_intent_state_conflict(error, out);
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
    json::member(out, "merge_receipt");
    json::push_text(out, &value.merge_receipt);
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
        merge_receipt: {
            let at1 = json::nested(at, "merge_receipt");
            let member1 = json::member_at(value, at, "merge_receipt")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
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

/// Writes the input of `controlplane.host.ConfigureRepository` as JSON.
pub fn encode_command_controlplane_host_configure_repository(value: &crate::host::ConfigureRepository, out: &mut String) {
    out.push('{');
    json::member(out, "repository_id");
    json::push_text(out, &value.repository_id.0);
    json::member(out, "base_branch");
    json::push_text(out, &value.base_branch);
    json::member(out, "test_command");
    json::push_text(out, &value.test_command);
    json::member(out, "publish_command");
    json::push_text(out, &value.publish_command);
    out.push('}');
}

/// Reads the input of `controlplane.host.ConfigureRepository` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_configure_repository(value: &json::Value, at: &str) -> Result<crate::host::ConfigureRepository, json::DecodeError> {
    Ok(crate::host::ConfigureRepository {
        repository_id: {
            let at0 = json::nested(at, "repository_id");
            let member0 = json::member_at(value, at, "repository_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
        base_branch: {
            let at1 = json::nested(at, "base_branch");
            let member1 = json::member_at(value, at, "base_branch")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        test_command: {
            let at2 = json::nested(at, "test_command");
            let member2 = json::member_at(value, at, "test_command")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
        },
        publish_command: {
            let at3 = json::nested(at, "publish_command");
            let member3 = json::member_at(value, at, "publish_command")?;
            json::text_at(member3, &at3, "a string")?.to_owned()
        },
    })
}

/// Writes the outcome of `controlplane.host.ConfigureRepository` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_configure_repository(value: &crate::host::ConfigureRepositoryOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::ConfigureRepositoryOutcome::Applied { configure_repository_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.ConfigureRepositoryApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_configure_repository_applied(configure_repository_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::ConfigureRepositoryOutcome::NotFound { error } => {
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

/// Writes the input of `controlplane.host.ConfirmPublication` as JSON.
pub fn encode_command_controlplane_host_confirm_publication(value: &crate::host::ConfirmPublication, out: &mut String) {
    out.push('{');
    json::member(out, "publication_id");
    json::push_text(out, &value.publication_id.0);
    json::member(out, "receipt");
    json::push_text(out, &value.receipt);
    out.push('}');
}

/// Reads the input of `controlplane.host.ConfirmPublication` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_confirm_publication(value: &json::Value, at: &str) -> Result<crate::host::ConfirmPublication, json::DecodeError> {
    Ok(crate::host::ConfirmPublication {
        publication_id: {
            let at0 = json::nested(at, "publication_id");
            let member0 = json::member_at(value, at, "publication_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
        receipt: {
            let at1 = json::nested(at, "receipt");
            let member1 = json::member_at(value, at, "receipt")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
    })
}

/// Writes the outcome of `controlplane.host.ConfirmPublication` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_confirm_publication(value: &crate::host::ConfirmPublicationOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::ConfirmPublicationOutcome::Applied { confirm_publication_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.ConfirmPublicationApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_confirm_publication_applied(confirm_publication_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::ConfirmPublicationOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.PublicationIntentNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_publication_intent_not_found(error, out);
            out.push('}');
        }
        crate::host::ConfirmPublicationOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.PublicationIntentStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_publication_intent_state_conflict(error, out);
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

/// Writes the input of `controlplane.host.DeleteGoal` as JSON.
pub fn encode_command_controlplane_host_delete_goal(value: &crate::host::DeleteGoal, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.DeleteGoal` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_delete_goal(value: &json::Value, at: &str) -> Result<crate::host::DeleteGoal, json::DecodeError> {
    Ok(crate::host::DeleteGoal {
        goal_id: {
            let at0 = json::nested(at, "goal_id");
            let member0 = json::member_at(value, at, "goal_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.DeleteGoal` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_delete_goal(value: &crate::host::DeleteGoalOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::DeleteGoalOutcome::Applied { goal_deleted } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.GoalDeleted");
            json::member(out, "payload");
            encode_event_controlplane_host_goal_deleted(goal_deleted, out);
            out.push('}');
            out.push(']');
        }
        crate::host::DeleteGoalOutcome::Paused { error } => {
            json::member(out, "outcome");
            json::push_text(out, "paused");
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
        crate::host::DeleteGoalOutcome::Running { error } => {
            json::member(out, "outcome");
            json::push_text(out, "running");
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
        crate::host::DeleteGoalOutcome::Satisfied { error } => {
            json::member(out, "outcome");
            json::push_text(out, "satisfied");
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
        crate::host::DeleteGoalOutcome::NotFound { error } => {
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

/// Writes the input of `controlplane.host.MarkPublicationUncertain` as JSON.
pub fn encode_command_controlplane_host_mark_publication_uncertain(value: &crate::host::MarkPublicationUncertain, out: &mut String) {
    out.push('{');
    json::member(out, "publication_id");
    json::push_text(out, &value.publication_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.MarkPublicationUncertain` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_mark_publication_uncertain(value: &json::Value, at: &str) -> Result<crate::host::MarkPublicationUncertain, json::DecodeError> {
    Ok(crate::host::MarkPublicationUncertain {
        publication_id: {
            let at0 = json::nested(at, "publication_id");
            let member0 = json::member_at(value, at, "publication_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.MarkPublicationUncertain` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_mark_publication_uncertain(value: &crate::host::MarkPublicationUncertainOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::MarkPublicationUncertainOutcome::Applied { mark_publication_uncertain_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.MarkPublicationUncertainApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_mark_publication_uncertain_applied(mark_publication_uncertain_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::MarkPublicationUncertainOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.PublicationIntentNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_publication_intent_not_found(error, out);
            out.push('}');
        }
        crate::host::MarkPublicationUncertainOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.PublicationIntentStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_publication_intent_state_conflict(error, out);
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

/// Writes the input of `controlplane.host.PreparePublication` as JSON.
pub fn encode_command_controlplane_host_prepare_publication(value: &crate::host::PreparePublication, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    json::member(out, "candidate");
    json::push_text(out, &value.candidate);
    json::member(out, "target");
    json::push_text(out, &value.target);
    json::member(out, "expected_base");
    json::push_text(out, &value.expected_base);
    out.push('}');
}

/// Reads the input of `controlplane.host.PreparePublication` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_prepare_publication(value: &json::Value, at: &str) -> Result<crate::host::PreparePublication, json::DecodeError> {
    Ok(crate::host::PreparePublication {
        assignment_id: {
            let at0 = json::nested(at, "assignment_id");
            let member0 = json::member_at(value, at, "assignment_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
        candidate: {
            let at1 = json::nested(at, "candidate");
            let member1 = json::member_at(value, at, "candidate")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        target: {
            let at2 = json::nested(at, "target");
            let member2 = json::member_at(value, at, "target")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
        },
        expected_base: {
            let at3 = json::nested(at, "expected_base");
            let member3 = json::member_at(value, at, "expected_base")?;
            json::text_at(member3, &at3, "a string")?.to_owned()
        },
    })
}

/// Writes the outcome of `controlplane.host.PreparePublication` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_prepare_publication(value: &crate::host::PreparePublicationOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::PreparePublicationOutcome::Created { publication_intent_created } => {
            json::member(out, "outcome");
            json::push_text(out, "created");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.PublicationIntentCreated");
            json::member(out, "payload");
            encode_event_controlplane_host_publication_intent_created(publication_intent_created, out);
            out.push('}');
            out.push(']');
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
    json::member(out, "goal_revision");
    json::push_integer(out, value.goal_revision);
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
        goal_revision: {
            let at10 = json::nested(at, "goal_revision");
            let member10 = json::member_at(value, at, "goal_revision")?;
            json::integer_at(member10, &at10, "an integer")?
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
    json::member(out, "reviewer_run");
    json::push_text(out, &value.reviewer_run);
    json::member(out, "review_revision");
    json::push_text(out, &value.review_revision);
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
        reviewer_run: {
            let at1 = json::nested(at, "reviewer_run");
            let member1 = json::member_at(value, at, "reviewer_run")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        review_revision: {
            let at2 = json::nested(at, "review_revision");
            let member2 = json::member_at(value, at, "review_revision")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
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

/// Writes the input of `controlplane.host.ReconcileAssignment` as JSON.
pub fn encode_command_controlplane_host_reconcile_assignment(value: &crate::host::ReconcileAssignment, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    json::member(out, "merge_receipt");
    json::push_text(out, &value.merge_receipt);
    out.push('}');
}

/// Reads the input of `controlplane.host.ReconcileAssignment` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_reconcile_assignment(value: &json::Value, at: &str) -> Result<crate::host::ReconcileAssignment, json::DecodeError> {
    Ok(crate::host::ReconcileAssignment {
        assignment_id: {
            let at0 = json::nested(at, "assignment_id");
            let member0 = json::member_at(value, at, "assignment_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
        merge_receipt: {
            let at1 = json::nested(at, "merge_receipt");
            let member1 = json::member_at(value, at, "merge_receipt")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
    })
}

/// Writes the outcome of `controlplane.host.ReconcileAssignment` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_reconcile_assignment(value: &crate::host::ReconcileAssignmentOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::ReconcileAssignmentOutcome::Applied { reconcile_assignment_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.ReconcileAssignmentApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_reconcile_assignment_applied(reconcile_assignment_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::ReconcileAssignmentOutcome::NotFound { error } => {
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
        crate::host::ReconcileAssignmentOutcome::WrongState { error } => {
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
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.RecordPlanningProgress` as JSON.
pub fn encode_command_controlplane_host_record_planning_progress(value: &crate::host::RecordPlanningProgress, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
    json::member(out, "planning_revision");
    json::push_integer(out, value.planning_revision);
    json::member(out, "planning_fingerprint");
    json::push_text(out, &value.planning_fingerprint);
    json::member(out, "planning_repository");
    json::push_text(out, &value.planning_repository);
    json::member(out, "planning_worktree_id");
    json::push_text(out, &value.planning_worktree_id);
    json::member(out, "planning_worktree_path");
    json::push_text(out, &value.planning_worktree_path);
    json::member(out, "planning_reason");
    json::push_text(out, &value.planning_reason);
    json::member(out, "planning_receipt");
    json::push_text(out, &value.planning_receipt);
    json::member(out, "planning_phase");
    encode_controlplane_host_planning_phase(&value.planning_phase, out);
    out.push('}');
}

/// Reads the input of `controlplane.host.RecordPlanningProgress` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_record_planning_progress(value: &json::Value, at: &str) -> Result<crate::host::RecordPlanningProgress, json::DecodeError> {
    Ok(crate::host::RecordPlanningProgress {
        goal_id: {
            let at0 = json::nested(at, "goal_id");
            let member0 = json::member_at(value, at, "goal_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
        planning_revision: {
            let at1 = json::nested(at, "planning_revision");
            let member1 = json::member_at(value, at, "planning_revision")?;
            json::integer_at(member1, &at1, "an integer")?
        },
        planning_fingerprint: {
            let at2 = json::nested(at, "planning_fingerprint");
            let member2 = json::member_at(value, at, "planning_fingerprint")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
        },
        planning_repository: {
            let at3 = json::nested(at, "planning_repository");
            let member3 = json::member_at(value, at, "planning_repository")?;
            json::text_at(member3, &at3, "a string")?.to_owned()
        },
        planning_worktree_id: {
            let at4 = json::nested(at, "planning_worktree_id");
            let member4 = json::member_at(value, at, "planning_worktree_id")?;
            json::text_at(member4, &at4, "a string")?.to_owned()
        },
        planning_worktree_path: {
            let at5 = json::nested(at, "planning_worktree_path");
            let member5 = json::member_at(value, at, "planning_worktree_path")?;
            json::text_at(member5, &at5, "a string")?.to_owned()
        },
        planning_reason: {
            let at6 = json::nested(at, "planning_reason");
            let member6 = json::member_at(value, at, "planning_reason")?;
            json::text_at(member6, &at6, "a string")?.to_owned()
        },
        planning_receipt: {
            let at7 = json::nested(at, "planning_receipt");
            let member7 = json::member_at(value, at, "planning_receipt")?;
            json::text_at(member7, &at7, "a string")?.to_owned()
        },
        planning_phase: {
            let at8 = json::nested(at, "planning_phase");
            let member8 = json::member_at(value, at, "planning_phase")?;
            decode_controlplane_host_planning_phase(member8, &at8)?
        },
    })
}

/// Writes the outcome of `controlplane.host.RecordPlanningProgress` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_record_planning_progress(value: &crate::host::RecordPlanningProgressOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::RecordPlanningProgressOutcome::Applied { planning_progress_recorded } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.PlanningProgressRecorded");
            json::member(out, "payload");
            encode_event_controlplane_host_planning_progress_recorded(planning_progress_recorded, out);
            out.push('}');
            out.push(']');
        }
        crate::host::RecordPlanningProgressOutcome::NotFound { error } => {
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

/// Writes the input of `controlplane.host.RemoveWorkspaceDirectory` as JSON.
pub fn encode_command_controlplane_host_remove_workspace_directory(value: &crate::host::RemoveWorkspaceDirectory, out: &mut String) {
    out.push('{');
    json::member(out, "directory_id");
    json::push_text(out, &value.directory_id.0);
    out.push('}');
}

/// Reads the input of `controlplane.host.RemoveWorkspaceDirectory` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_remove_workspace_directory(value: &json::Value, at: &str) -> Result<crate::host::RemoveWorkspaceDirectory, json::DecodeError> {
    Ok(crate::host::RemoveWorkspaceDirectory {
        directory_id: {
            let at0 = json::nested(at, "directory_id");
            let member0 = json::member_at(value, at, "directory_id")?;
            crate::primitives::Uuid(json::uuid_at(member0, &at0, "a UUID")?.to_owned())
        },
    })
}

/// Writes the outcome of `controlplane.host.RemoveWorkspaceDirectory` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_remove_workspace_directory(value: &crate::host::RemoveWorkspaceDirectoryOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::RemoveWorkspaceDirectoryOutcome::Applied { workspace_directory_removed } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.WorkspaceDirectoryRemoved");
            json::member(out, "payload");
            encode_event_controlplane_host_workspace_directory_removed(workspace_directory_removed, out);
            out.push('}');
            out.push(']');
        }
        crate::host::RemoveWorkspaceDirectoryOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.WorkspaceDirectoryStateConflict");
            json::member(out, "payload");
            encode_error_controlplane_host_workspace_directory_state_conflict(error, out);
            out.push('}');
        }
        crate::host::RemoveWorkspaceDirectoryOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "controlplane.host.WorkspaceDirectoryNotFound");
            json::member(out, "payload");
            encode_error_controlplane_host_workspace_directory_not_found(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `controlplane.host.RepairAssignment` as JSON.
pub fn encode_command_controlplane_host_repair_assignment(value: &crate::host::RepairAssignment, out: &mut String) {
    out.push('{');
    json::member(out, "assignment_id");
    json::push_text(out, &value.assignment_id.0);
    json::member(out, "reason");
    json::push_text(out, &value.reason);
    json::member(out, "implementor_run");
    json::push_text(out, &value.implementor_run);
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
        reason: {
            let at1 = json::nested(at, "reason");
            let member1 = json::member_at(value, at, "reason")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        implementor_run: {
            let at2 = json::nested(at, "implementor_run");
            let member2 = json::member_at(value, at, "implementor_run")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
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
    json::member(out, "candidate");
    json::push_text(out, &value.candidate);
    json::member(out, "test_revision");
    json::push_text(out, &value.test_revision);
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
        candidate: {
            let at1 = json::nested(at, "candidate");
            let member1 = json::member_at(value, at, "candidate")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        test_revision: {
            let at2 = json::nested(at, "test_revision");
            let member2 = json::member_at(value, at, "test_revision")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
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
    json::member(out, "satisfaction_receipt");
    json::push_text(out, &value.satisfaction_receipt);
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
        satisfaction_receipt: {
            let at1 = json::nested(at, "satisfaction_receipt");
            let member1 = json::member_at(value, at, "satisfaction_receipt")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
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

/// Writes the input of `controlplane.host.UpdateGoal` as JSON.
pub fn encode_command_controlplane_host_update_goal(value: &crate::host::UpdateGoal, out: &mut String) {
    out.push('{');
    json::member(out, "goal_id");
    json::push_text(out, &value.goal_id.0);
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

/// Reads the input of `controlplane.host.UpdateGoal` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_controlplane_host_update_goal(value: &json::Value, at: &str) -> Result<crate::host::UpdateGoal, json::DecodeError> {
    Ok(crate::host::UpdateGoal {
        goal_id: {
            let at0 = json::nested(at, "goal_id");
            let member0 = json::member_at(value, at, "goal_id")?;
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

/// Writes the outcome of `controlplane.host.UpdateGoal` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_controlplane_host_update_goal(value: &crate::host::UpdateGoalOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::host::UpdateGoalOutcome::Applied { update_goal_applied } => {
            json::member(out, "outcome");
            json::push_text(out, "applied");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "controlplane.host.UpdateGoalApplied");
            json::member(out, "payload");
            encode_event_controlplane_host_update_goal_applied(update_goal_applied, out);
            out.push('}');
            out.push(']');
        }
        crate::host::UpdateGoalOutcome::Satisfied { error } => {
            json::member(out, "outcome");
            json::push_text(out, "satisfied");
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
        crate::host::UpdateGoalOutcome::Cancelled { error } => {
            json::member(out, "outcome");
            json::push_text(out, "cancelled");
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
        crate::host::UpdateGoalOutcome::NotFound { error } => {
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
        crate::system::SystemEvent::ClosePublicationApplied(event) => encode_event_controlplane_host_close_publication_applied(event, &mut out),
        crate::system::SystemEvent::CompleteAssignmentApplied(event) => encode_event_controlplane_host_complete_assignment_applied(event, &mut out),
        crate::system::SystemEvent::ConfigureRepositoryApplied(event) => encode_event_controlplane_host_configure_repository_applied(event, &mut out),
        crate::system::SystemEvent::ConfirmPublicationApplied(event) => encode_event_controlplane_host_confirm_publication_applied(event, &mut out),
        crate::system::SystemEvent::DisableRepositoryRegistrationApplied(event) => encode_event_controlplane_host_disable_repository_registration_applied(event, &mut out),
        crate::system::SystemEvent::EnableRepositoryRegistrationApplied(event) => encode_event_controlplane_host_enable_repository_registration_applied(event, &mut out),
        crate::system::SystemEvent::GoalCreated(event) => encode_event_controlplane_host_goal_created(event, &mut out),
        crate::system::SystemEvent::GoalDeleted(event) => encode_event_controlplane_host_goal_deleted(event, &mut out),
        crate::system::SystemEvent::MarkPublicationUncertainApplied(event) => encode_event_controlplane_host_mark_publication_uncertain_applied(event, &mut out),
        crate::system::SystemEvent::MergeAssignmentApplied(event) => encode_event_controlplane_host_merge_assignment_applied(event, &mut out),
        crate::system::SystemEvent::PauseGoalApplied(event) => encode_event_controlplane_host_pause_goal_applied(event, &mut out),
        crate::system::SystemEvent::PlanningProgressRecorded(event) => encode_event_controlplane_host_planning_progress_recorded(event, &mut out),
        crate::system::SystemEvent::PublicationIntentCreated(event) => encode_event_controlplane_host_publication_intent_created(event, &mut out),
        crate::system::SystemEvent::ReadyAssignmentApplied(event) => encode_event_controlplane_host_ready_assignment_applied(event, &mut out),
        crate::system::SystemEvent::ReconcileAssignmentApplied(event) => encode_event_controlplane_host_reconcile_assignment_applied(event, &mut out),
        crate::system::SystemEvent::RepairAssignmentApplied(event) => encode_event_controlplane_host_repair_assignment_applied(event, &mut out),
        crate::system::SystemEvent::RepositoryRegistrationCreated(event) => encode_event_controlplane_host_repository_registration_created(event, &mut out),
        crate::system::SystemEvent::ReviewAssignmentApplied(event) => encode_event_controlplane_host_review_assignment_applied(event, &mut out),
        crate::system::SystemEvent::SatisfyGoalApplied(event) => encode_event_controlplane_host_satisfy_goal_applied(event, &mut out),
        crate::system::SystemEvent::StartGoalApplied(event) => encode_event_controlplane_host_start_goal_applied(event, &mut out),
        crate::system::SystemEvent::UpdateGoalApplied(event) => encode_event_controlplane_host_update_goal_applied(event, &mut out),
        crate::system::SystemEvent::WorkspaceCreated(event) => encode_event_controlplane_host_workspace_created(event, &mut out),
        crate::system::SystemEvent::WorkspaceDirectoryCreated(event) => encode_event_controlplane_host_workspace_directory_created(event, &mut out),
        crate::system::SystemEvent::WorkspaceDirectoryRemoved(event) => encode_event_controlplane_host_workspace_directory_removed(event, &mut out),
    }
    out.push('}');
    out
}
