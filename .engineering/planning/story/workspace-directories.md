---
format: aep.planning-md/3
id: story:workspace-directories
kind: story
status: implemented
title: Multiple workspace directories and current-directory startup
relations:
- decomposes: epic:bootstrap
- serves: vision:autonomous-engineering
- depends_on: story:workspace-host
scope:
- confidence: inferred
  path: crates/control-plane-app
- confidence: inferred
  path: crates/control-plane-core
- confidence: inferred
  path: ess
- confidence: inferred
  path: generated
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T21:11:59Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-05T21:11:59Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-05T21:36:00Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome

One running service manages multiple workspaces. Each workspace owns multiple persistent directory memberships, including directories without Git and directories containing repositories. Startup registers the current directory as a workspace by default, without starting a goal or enabling merge authority.

## ESS first

WorkspaceDirectory is declared in ess/domains/host.yaml before this story: directory_id and workspace_id UUIDs, canonical path, observed repository common directories and the subset introduced by membership registration. Workspace owns many directories. AddWorkspaceDirectory and RemoveWorkspaceDirectory are Operator commands; Registered moves to Removed. Strict ESS 0.53 validation passed at source 870330a.

## Acceptance

Named conformance and operational scenarios: multiple_workspaces_keep_directory_membership_isolated, non_git_directory_survives_restart_and_canonical_duplicates_are_idempotent, overlapping_directories_preserve_shared_repositories, active_assignment_prevents_directory_removal, startup_registers_current_directory_once. Run against actual Store, API and CLI handlers, with real temporary directories and Git discovery. Regenerate and execute every generated conformance scenario.

## Implementation contract

Expose GET/POST /api/workspaces, workspace detail, GET/POST workspace directories and directory removal. Keep /api/state and existing commands compatible. The browser lists workspaces and adds/removes directories in the selected workspace; the CLI exposes equivalent operations. Directory registration and derived repository changes commit atomically. Removing a directory must preserve pre-existing manual registrations and repositories covered by another active directory; do not remove directories with active assignments. Canonical paths deduplicate membership within one workspace while allowing the same directory in different workspaces. Default startup adopts the current directory idempotently; explicit repeatable --workspace paths select startup roots. Previously persisted Workspace records gain their initial membership without losing goals or repositories.

## Authorization

Operator explicitly requested multiple workspaces, workspace APIs, multiple directories per workspace and adding the current repository on startup. This extends the approved implementation goal; no further approval is required.

## Scope

ess, generated, crates/control-plane-core and crates/control-plane-app. One implementor owns this extension; runtime changes to observe directories remain coordinated with the existing planner/fleet lane. Root manifests, the AEP store and gate integration remain coordinator-owned.
