# Control plane

Control plane is the operator's live view of autonomous engineering. One instance manages multiple workspaces, their directories and repositories, and the agents turning goals into verified changes.

The operator should be able to open the application and immediately answer:

- Is the system connected and processing work?
- What is the planner doing now, and when did it last act?
- Which agents are working, on which repository and assignment?
- What is queued, under review, blocked, or complete?
- What changed, what evidence supports it, and what needs my attention?
- Where can I pause, cancel, revise a goal, or change merge authority?

## The operating experience

The main screen is an operational dashboard. Persistent workspace navigation, a system-health bar, current activity, planner and worker states, and a recent activity feed form its primary surface. Goal creation, directory management and configuration remain accessible without obscuring current work.

Activity updates automatically while the operator watches. Editing a form must not interrupt updates, discard input, or reset focus. A server heartbeat shows connection freshness; executor activity shows actual model calls, file operations, checks, reviews and publication observations. These are separate facts. An animation or a refreshed clock must never imply that an agent made progress.

Every active operation shows a human-readable action, role, target, start or last-activity time, and elapsed time where observed. Waiting for a model response is a visible state. Failures and blockers explain what stopped and what action can resolve it. A quiet system explicitly says whether it is idle, paused, waiting, disconnected or stopped.

Progress uses real counts and named stages. An empty assignment queue cannot establish goal completion. Completion requires verified results and a recorded acceptance decision. Technical receipts remain available for inspection, while the primary view explains their meaning without requiring the operator to read JSON.

## Autonomous execution

Control-plane owns workspace and goal management, scheduling between runs, repository contention, operator controls and the live view. Loom owns execution within a run: model turns, context, sessions, tool feedback, streaming and budgets. Commission revalidates and invokes effects; the Canon governor and engineering protocols decide admissibility and earned completion. Planner, implementor and reviewer are roles using that same runtime.

The [runtime ownership audit](../.engineering/planning/architecture-design/runtime-ownership.md) maps each responsibility to its owner and distinguishes available APIs from foundation gaps. The [boundary correction](../.engineering/planning/story/runtime-boundary.md) replaces the current private fleet loop and duplicated runtime behavior. The first external auth eval failed in planning; scripted tests do not establish successful autonomous delivery.

A goal drives ESS-first planning in the repository's AEP store. The planner produces executable assignments. A bounded fleet implements them in isolated worktrees, runs real checks, obtains independent review, and observes authorized publication before recording success. Changes to a goal, repository configuration or authority invalidate stale evidence. Pausing or cancelling stops further effects; restart reconciles durable state.

Workspaces may include several Git repositories and context directories without Git. One instance owns the fleet across these workspaces and makes shared repository contention visible. Models, worker limits, attempt limits, per-attempt time budgets and merge authority are explicit operator controls.

## Product boundaries

The product lives in the standalone control-plane repository. Runtime code is Rust and uses beyond10x foundation libraries. ESS owns domain contracts; AEP owns engineering plans; the durable host owns observed execution records. Model output cannot grant authority or manufacture test and merge evidence. Release and deployment remain outside autonomous source-change delivery.

## Delivery standard

A working backend and configuration forms are insufficient for this product. The live operator experience is part of acceptance: meaningful updates during model waits and tool execution, visible blockers, retained activity across restart, and responsive controls while workers run. Browser checks must observe these behaviours against the running application.

The implementation agenda is the [AEP bootstrap epic](../.engineering/planning/epic/bootstrap.md), including [live operator visibility](../.engineering/planning/story/operator-observability.md). This document states product intent; the AEP store records delivery status and evidence.
