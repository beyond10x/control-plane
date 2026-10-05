// generated from controlplane v1
// model digest 528a7c48088b8ebb67277ee677106218efacfce1938bb9582406ba6a479b6902
// contract digest cb2cdc58d77ebfe102a784bb691765368fb444d938e6de9301d0443f5039af83
// do not edit: regenerate with `ess synthesize --layout crate`

//! The `control-plane` component of `controlplane` v1, on the wire.
//!
//! The specification says this component's callers are not deployed with it, so its surface
//! exists on a wire. Which wire is derived rather than chosen: the one contract this model
//! projects for a command surface is the `OpenAPI` document, and an `OpenAPI` document is an
//! HTTP contract. The document is beside this file, served verbatim at `/openapi.json`.

use crate::server::{entry, http, json, wire};

/// The contract this surface answers, byte for byte as `generated/` commits it.
///
/// Embedded rather than rebuilt at run time: a server that regenerated its own contract could
/// publish one the repository never reviewed.
pub const OPENAPI: &str = include_str!("control-plane.openapi.json");

/// The prose the same model produced, byte for byte as the documentation projection wrote it.
pub const DOCS: &str = include_str!("control-plane.docs.md");

/// Every route this surface answers, in path order.
///
/// The same set the `OpenAPI` document declares, plus the two documents about the surface
/// itself, which no specification construct names and nothing can therefore derive. A path
/// absent from this table is answered with `404`, including one the document declares and this
/// table forgot — which is the failure a table computed twice would hide.
pub const ROUTES: &[(&str, &str)] = &[
    ("GET", "/docs"),
    ("POST", "/host/commands/AddWorkspaceDirectory"),
    ("POST", "/host/commands/ArchiveWorkspace"),
    ("POST", "/host/commands/BlockAssignment"),
    ("POST", "/host/commands/CancelAssignment"),
    ("POST", "/host/commands/CancelGoal"),
    ("POST", "/host/commands/ClaimAssignment"),
    ("POST", "/host/commands/CompleteAssignment"),
    ("POST", "/host/commands/ConfigureRepository"),
    ("POST", "/host/commands/ConfirmPublication"),
    ("POST", "/host/commands/CreateGoal"),
    ("POST", "/host/commands/DisableRepositoryRegistration"),
    ("POST", "/host/commands/EnableRepositoryRegistration"),
    ("POST", "/host/commands/MarkPublicationUncertain"),
    ("POST", "/host/commands/MergeAssignment"),
    ("POST", "/host/commands/PauseGoal"),
    ("POST", "/host/commands/PreparePublication"),
    ("POST", "/host/commands/QueueAssignment"),
    ("POST", "/host/commands/ReadyAssignment"),
    ("POST", "/host/commands/ReconcileAssignment"),
    ("POST", "/host/commands/RecordPlanningProgress"),
    ("POST", "/host/commands/RegisterRepository"),
    ("POST", "/host/commands/RegisterWorkspace"),
    ("POST", "/host/commands/RemoveWorkspaceDirectory"),
    ("POST", "/host/commands/RepairAssignment"),
    ("POST", "/host/commands/ReviewAssignment"),
    ("POST", "/host/commands/SatisfyGoal"),
    ("POST", "/host/commands/StartGoal"),
    ("POST", "/host/commands/UpdateGoal"),
    ("GET", "/host/views/AssignmentList"),
    ("GET", "/host/views/GoalList"),
    ("GET", "/host/views/PublicationIntentList"),
    ("GET", "/host/views/RepositoryRegistrationList"),
    ("GET", "/host/views/WorkspaceDirectoryList"),
    ("GET", "/host/views/WorkspaceList"),
    ("GET", "/openapi.json"),
];

/// What this process says about itself as it starts, before it answers anything.
///
/// Three lines of JSON on standard output, in this order, every member of them derived from the
/// specification — except `runtime`, which is appended by the emitted code below and holds what
/// is true of *this process*: the language it was synthesised into, and the address it bound.
/// Everything outside `runtime` is the same in every language this plan is emitted into, and
/// `cargo xtask synth --check` starts both and compares them.
pub const STARTUP: &[&str] = &[
    "{\"log\":\"ess/1\",\"event\":\"system.starting\",\"system\":\"controlplane\",\"version\":\"v1\",\"model_digest\":\"528a7c48088b8ebb67277ee677106218efacfce1938bb9582406ba6a479b6902\",\"contract_digest\":\"cb2cdc58d77ebfe102a784bb691765368fb444d938e6de9301d0443f5039af83\",\"components\":[\"control-plane\"],\"capabilities\":{\"generated\":125,\"obligations\":0,\"refused\":0}",
    "{\"log\":\"ess/1\",\"event\":\"surface.serving\",\"component\":\"control-plane\",\"reached_by\":\"network\",\"transport\":\"http/1.1\",\"routes\":36,\"paths\":[{\"method\":\"GET\",\"path\":\"/docs\",\"serves\":\"documentation\",\"name\":\"docs\"},{\"method\":\"POST\",\"path\":\"/host/commands/AddWorkspaceDirectory\",\"serves\":\"command\",\"name\":\"controlplane.host.AddWorkspaceDirectory\"},{\"method\":\"POST\",\"path\":\"/host/commands/ArchiveWorkspace\",\"serves\":\"command\",\"name\":\"controlplane.host.ArchiveWorkspace\"},{\"method\":\"POST\",\"path\":\"/host/commands/BlockAssignment\",\"serves\":\"command\",\"name\":\"controlplane.host.BlockAssignment\"},{\"method\":\"POST\",\"path\":\"/host/commands/CancelAssignment\",\"serves\":\"command\",\"name\":\"controlplane.host.CancelAssignment\"},{\"method\":\"POST\",\"path\":\"/host/commands/CancelGoal\",\"serves\":\"command\",\"name\":\"controlplane.host.CancelGoal\"},{\"method\":\"POST\",\"path\":\"/host/commands/ClaimAssignment\",\"serves\":\"command\",\"name\":\"controlplane.host.ClaimAssignment\"},{\"method\":\"POST\",\"path\":\"/host/commands/CompleteAssignment\",\"serves\":\"command\",\"name\":\"controlplane.host.CompleteAssignment\"},{\"method\":\"POST\",\"path\":\"/host/commands/ConfigureRepository\",\"serves\":\"command\",\"name\":\"controlplane.host.ConfigureRepository\"},{\"method\":\"POST\",\"path\":\"/host/commands/ConfirmPublication\",\"serves\":\"command\",\"name\":\"controlplane.host.ConfirmPublication\"},{\"method\":\"POST\",\"path\":\"/host/commands/CreateGoal\",\"serves\":\"command\",\"name\":\"controlplane.host.CreateGoal\"},{\"method\":\"POST\",\"path\":\"/host/commands/DisableRepositoryRegistration\",\"serves\":\"command\",\"name\":\"controlplane.host.DisableRepositoryRegistration\"},{\"method\":\"POST\",\"path\":\"/host/commands/EnableRepositoryRegistration\",\"serves\":\"command\",\"name\":\"controlplane.host.EnableRepositoryRegistration\"},{\"method\":\"POST\",\"path\":\"/host/commands/MarkPublicationUncertain\",\"serves\":\"command\",\"name\":\"controlplane.host.MarkPublicationUncertain\"},{\"method\":\"POST\",\"path\":\"/host/commands/MergeAssignment\",\"serves\":\"command\",\"name\":\"controlplane.host.MergeAssignment\"},{\"method\":\"POST\",\"path\":\"/host/commands/PauseGoal\",\"serves\":\"command\",\"name\":\"controlplane.host.PauseGoal\"},{\"method\":\"POST\",\"path\":\"/host/commands/PreparePublication\",\"serves\":\"command\",\"name\":\"controlplane.host.PreparePublication\"},{\"method\":\"POST\",\"path\":\"/host/commands/QueueAssignment\",\"serves\":\"command\",\"name\":\"controlplane.host.QueueAssignment\"},{\"method\":\"POST\",\"path\":\"/host/commands/ReadyAssignment\",\"serves\":\"command\",\"name\":\"controlplane.host.ReadyAssignment\"},{\"method\":\"POST\",\"path\":\"/host/commands/ReconcileAssignment\",\"serves\":\"command\",\"name\":\"controlplane.host.ReconcileAssignment\"},{\"method\":\"POST\",\"path\":\"/host/commands/RecordPlanningProgress\",\"serves\":\"command\",\"name\":\"controlplane.host.RecordPlanningProgress\"},{\"method\":\"POST\",\"path\":\"/host/commands/RegisterRepository\",\"serves\":\"command\",\"name\":\"controlplane.host.RegisterRepository\"},{\"method\":\"POST\",\"path\":\"/host/commands/RegisterWorkspace\",\"serves\":\"command\",\"name\":\"controlplane.host.RegisterWorkspace\"},{\"method\":\"POST\",\"path\":\"/host/commands/RemoveWorkspaceDirectory\",\"serves\":\"command\",\"name\":\"controlplane.host.RemoveWorkspaceDirectory\"},{\"method\":\"POST\",\"path\":\"/host/commands/RepairAssignment\",\"serves\":\"command\",\"name\":\"controlplane.host.RepairAssignment\"},{\"method\":\"POST\",\"path\":\"/host/commands/ReviewAssignment\",\"serves\":\"command\",\"name\":\"controlplane.host.ReviewAssignment\"},{\"method\":\"POST\",\"path\":\"/host/commands/SatisfyGoal\",\"serves\":\"command\",\"name\":\"controlplane.host.SatisfyGoal\"},{\"method\":\"POST\",\"path\":\"/host/commands/StartGoal\",\"serves\":\"command\",\"name\":\"controlplane.host.StartGoal\"},{\"method\":\"POST\",\"path\":\"/host/commands/UpdateGoal\",\"serves\":\"command\",\"name\":\"controlplane.host.UpdateGoal\"},{\"method\":\"GET\",\"path\":\"/host/views/AssignmentList\",\"serves\":\"view\",\"name\":\"controlplane.host.AssignmentList\"},{\"method\":\"GET\",\"path\":\"/host/views/GoalList\",\"serves\":\"view\",\"name\":\"controlplane.host.GoalList\"},{\"method\":\"GET\",\"path\":\"/host/views/PublicationIntentList\",\"serves\":\"view\",\"name\":\"controlplane.host.PublicationIntentList\"},{\"method\":\"GET\",\"path\":\"/host/views/RepositoryRegistrationList\",\"serves\":\"view\",\"name\":\"controlplane.host.RepositoryRegistrationList\"},{\"method\":\"GET\",\"path\":\"/host/views/WorkspaceDirectoryList\",\"serves\":\"view\",\"name\":\"controlplane.host.WorkspaceDirectoryList\"},{\"method\":\"GET\",\"path\":\"/host/views/WorkspaceList\",\"serves\":\"view\",\"name\":\"controlplane.host.WorkspaceList\"},{\"method\":\"GET\",\"path\":\"/openapi.json\",\"serves\":\"contract\",\"name\":\"openapi\"}]",
    "{\"log\":\"ess/1\",\"event\":\"system.ready\",\"system\":\"controlplane\",\"surfaces\":1",
];

/// Writes the startup record, with this process's own facts closing each line.
fn announce(address: &std::net::SocketAddr) {
    for facts in STARTUP {
        let mut line = String::from(*facts);
        line.push_str(",\"runtime\":{\"address\":");
        json::push_text(&mut line, &address.to_string());
        line.push_str(",\"language\":\"rust\",\"port\":");
        json::push_integer(&mut line, i64::from(address.port()));
        line.push_str("}}");
        println!("{line}");
    }
}

/// Serves `control-plane` at `address`, and does not return while it can answer.
///
/// `address` may name port `0`, which binds an ephemeral port; the startup record says which one
/// was taken, because a caller that cannot learn the port cannot make a request.
///
/// It chooses no realization. Every command reaches the port, and a port over unimplemented
/// obligations answers the typed refusal this surface reports as `501` — the honest empty
/// state rather than a server that pretends.
///
/// `authenticate` is the realization's: it says who each request was sent by, or `None`, and
/// [`dispatch`] checks that caller's grant before the command runs. Nothing here reads an
/// actor from the request itself.
///
/// One connection at a time: each is dropped after [`http::READ_TIMEOUT`] without a byte, or
/// [`http::WRITE_TIMEOUT`] of a stalled write, and whatever fails on one connection — a caller
/// that hung up before reading its answer, a reset, a failed accept — ends that connection only.
///
/// # Errors
///
/// What binding the address refuses: the address is taken, or the port is privileged.
pub fn serve<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, address: &str, authenticate: impl Fn(&http::Request) -> Option<crate::actor::Caller>) -> std::io::Result<()>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{ serve_with_static(system, address, authenticate, None) }

/// Serves the declared surface, with files only for paths outside its route table.
///
/// # Errors
/// Returns a listener or static-root error before announcing readiness.
pub fn serve_with_static<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, address: &str, authenticate: impl Fn(&http::Request) -> Option<crate::actor::Caller>, static_root: Option<&std::path::Path>) -> std::io::Result<()>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let static_root = static_root.map(std::fs::canonicalize).transpose()?;
    if static_root.as_ref().is_some_and(|root| !root.is_dir()) { return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "static root is not a directory")); }
    let listener = std::net::TcpListener::bind(address)?;
    announce(&listener.local_addr()?);
    for connection in listener.incoming() {
        // What fails on one connection ends that connection, never this loop: a caller that
        // gave up before it was accepted, or before it read its answer, is that caller's affair.
        let Ok(connection) = connection else {
            // An accept the listener itself failed (no descriptor left) would fail again at
            // once: pause before the next.
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        };
        let _ = connection.set_read_timeout(Some(http::READ_TIMEOUT));
        let _ = connection.set_write_timeout(Some(http::WRITE_TIMEOUT));
        let mut reader = std::io::BufReader::new(connection);
        let (answer, refused) = match http::read(&mut reader) {
            Ok(request) => {
                if !ROUTES.iter().any(|(_, path)| *path == request.path) {
                    if let Some(root) = &static_root {
                        let _ = crate::server::static_assets::answer(reader.get_mut(), root, &request);
                        continue;
                    }
                }
                (dispatch(system, authenticate(&request).as_ref(), &request), false)
            },
            Err(refusal) => (refusal, true),
        };
        let mut stream = reader.into_inner();
        if http::write(&mut stream, &answer).is_ok() && refused {
            http::linger(&mut stream);
        }
    }
    Ok(())
}

/// Answers one request.
///
/// A path this table does not hold is a `404` naming where the whole table is published; a
/// path it holds under a different method is a `405` naming the one it answers. Neither is a
/// status the contract declares, and neither should be: both are facts about a transport rather
/// than about any command.
///
/// Public so a caller can hand it a request it built itself: [`serve`] is this function behind a
/// socket, and nothing else.
///
/// `caller` is who the realization authenticated the request as, or `None`. Every command
/// checks its grant before it runs, and answers the standard refusal when the caller is none
/// or is an actor the specification does not grant the command.
pub fn dispatch<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, caller: Option<&crate::actor::Caller>, request: &http::Request) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    match request.path.as_str() {
        "/docs" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::Response::new(200, http::MARKDOWN, DOCS)
        }
        "/host/commands/AddWorkspaceDirectory" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.AddWorkspaceDirectory") {
                return not_granted(actor);
            }
            serve_controlplane_host_add_workspace_directory(system, &request.body)
        }
        "/host/commands/ArchiveWorkspace" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.ArchiveWorkspace") {
                return not_granted(actor);
            }
            serve_controlplane_host_archive_workspace(system, &request.body)
        }
        "/host/commands/BlockAssignment" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.BlockAssignment") {
                return not_granted(actor);
            }
            serve_controlplane_host_block_assignment(system, &request.body)
        }
        "/host/commands/CancelAssignment" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.CancelAssignment") {
                return not_granted(actor);
            }
            serve_controlplane_host_cancel_assignment(system, &request.body)
        }
        "/host/commands/CancelGoal" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.CancelGoal") {
                return not_granted(actor);
            }
            serve_controlplane_host_cancel_goal(system, &request.body)
        }
        "/host/commands/ClaimAssignment" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.ClaimAssignment") {
                return not_granted(actor);
            }
            serve_controlplane_host_claim_assignment(system, &request.body)
        }
        "/host/commands/CompleteAssignment" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.CompleteAssignment") {
                return not_granted(actor);
            }
            serve_controlplane_host_complete_assignment(system, &request.body)
        }
        "/host/commands/ConfigureRepository" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.ConfigureRepository") {
                return not_granted(actor);
            }
            serve_controlplane_host_configure_repository(system, &request.body)
        }
        "/host/commands/ConfirmPublication" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.ConfirmPublication") {
                return not_granted(actor);
            }
            serve_controlplane_host_confirm_publication(system, &request.body)
        }
        "/host/commands/CreateGoal" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.CreateGoal") {
                return not_granted(actor);
            }
            serve_controlplane_host_create_goal(system, &request.body)
        }
        "/host/commands/DisableRepositoryRegistration" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.DisableRepositoryRegistration") {
                return not_granted(actor);
            }
            serve_controlplane_host_disable_repository_registration(system, &request.body)
        }
        "/host/commands/EnableRepositoryRegistration" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.EnableRepositoryRegistration") {
                return not_granted(actor);
            }
            serve_controlplane_host_enable_repository_registration(system, &request.body)
        }
        "/host/commands/MarkPublicationUncertain" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.MarkPublicationUncertain") {
                return not_granted(actor);
            }
            serve_controlplane_host_mark_publication_uncertain(system, &request.body)
        }
        "/host/commands/MergeAssignment" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.MergeAssignment") {
                return not_granted(actor);
            }
            serve_controlplane_host_merge_assignment(system, &request.body)
        }
        "/host/commands/PauseGoal" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.PauseGoal") {
                return not_granted(actor);
            }
            serve_controlplane_host_pause_goal(system, &request.body)
        }
        "/host/commands/PreparePublication" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.PreparePublication") {
                return not_granted(actor);
            }
            serve_controlplane_host_prepare_publication(system, &request.body)
        }
        "/host/commands/QueueAssignment" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.QueueAssignment") {
                return not_granted(actor);
            }
            serve_controlplane_host_queue_assignment(system, &request.body)
        }
        "/host/commands/ReadyAssignment" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.ReadyAssignment") {
                return not_granted(actor);
            }
            serve_controlplane_host_ready_assignment(system, &request.body)
        }
        "/host/commands/ReconcileAssignment" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.ReconcileAssignment") {
                return not_granted(actor);
            }
            serve_controlplane_host_reconcile_assignment(system, &request.body)
        }
        "/host/commands/RecordPlanningProgress" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.RecordPlanningProgress") {
                return not_granted(actor);
            }
            serve_controlplane_host_record_planning_progress(system, &request.body)
        }
        "/host/commands/RegisterRepository" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.RegisterRepository") {
                return not_granted(actor);
            }
            serve_controlplane_host_register_repository(system, &request.body)
        }
        "/host/commands/RegisterWorkspace" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.RegisterWorkspace") {
                return not_granted(actor);
            }
            serve_controlplane_host_register_workspace(system, &request.body)
        }
        "/host/commands/RemoveWorkspaceDirectory" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.RemoveWorkspaceDirectory") {
                return not_granted(actor);
            }
            serve_controlplane_host_remove_workspace_directory(system, &request.body)
        }
        "/host/commands/RepairAssignment" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.RepairAssignment") {
                return not_granted(actor);
            }
            serve_controlplane_host_repair_assignment(system, &request.body)
        }
        "/host/commands/ReviewAssignment" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.ReviewAssignment") {
                return not_granted(actor);
            }
            serve_controlplane_host_review_assignment(system, &request.body)
        }
        "/host/commands/SatisfyGoal" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.SatisfyGoal") {
                return not_granted(actor);
            }
            serve_controlplane_host_satisfy_goal(system, &request.body)
        }
        "/host/commands/StartGoal" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.StartGoal") {
                return not_granted(actor);
            }
            serve_controlplane_host_start_goal(system, &request.body)
        }
        "/host/commands/UpdateGoal" => {
            if request.method != "POST" {
                return http::method_not_allowed("POST");
            }
            if let Err(actor) = admit(caller, "controlplane.host.UpdateGoal") {
                return not_granted(actor);
            }
            serve_controlplane_host_update_goal(system, &request.body)
        }
        "/host/views/AssignmentList" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_controlplane_host_assignment_list(system))
        }
        "/host/views/GoalList" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_controlplane_host_goal_list(system))
        }
        "/host/views/PublicationIntentList" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_controlplane_host_publication_intent_list(system))
        }
        "/host/views/RepositoryRegistrationList" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_controlplane_host_repository_registration_list(system))
        }
        "/host/views/WorkspaceDirectoryList" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_controlplane_host_workspace_directory_list(system))
        }
        "/host/views/WorkspaceList" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::answer(run_controlplane_host_workspace_list(system))
        }
        "/openapi.json" => {
            if request.method != "GET" {
                return http::method_not_allowed("GET");
            }
            http::Response::new(200, http::JSON, OPENAPI)
        }
        other => http::Response::refusal(
            404,
            &format!("`{other}` is not a path this surface declares; `GET /openapi.json` publishes every one that is"),
        ),
    }
}

/// Runs one command or view of `control-plane` by its qualified name, with no transport.
///
/// The same decoding, refusals and rendering the HTTP routes use — each route and this function call
/// one `run_*` function — so a conformance runner or an in-process caller drives the system
/// without a socket and without a dispatch table of its own. `Ok` is the declared outcome, as the
/// route's body renders it; a view ignores `input`, as its `GET` route ignores a body.
///
/// # Errors
///
/// [`entry::Refused::Unknown`] naming `name` when this surface declares no command or view
/// by it; [`entry::Refused::Input`] when `input` is not the command's declared input (the
/// route's `400`); [`entry::Refused::Unmet`] when the port reports an unmet obligation, and
/// [`entry::Refused::Undelivered`] when the command took effect and delivering what it published
/// failed (the route's `501`, with `committed` `false` and `true`).
/// [`entry::Refused::NotGranted`] when `caller` — who the realization authenticated the call
/// as — is none, or is an actor the specification does not grant the command; checked before
/// the command runs (the route's `403`).
pub fn handle<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, caller: Option<&crate::actor::Caller>, name: &str, input: json::Value) -> Result<json::Value, entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let answered = match name {
        "controlplane.host.AddWorkspaceDirectory" => match admit(caller, "controlplane.host.AddWorkspaceDirectory") {
            Ok(()) => run_controlplane_host_add_workspace_directory(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.ArchiveWorkspace" => match admit(caller, "controlplane.host.ArchiveWorkspace") {
            Ok(()) => run_controlplane_host_archive_workspace(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.AssignmentList" => run_controlplane_host_assignment_list(system),
        "controlplane.host.BlockAssignment" => match admit(caller, "controlplane.host.BlockAssignment") {
            Ok(()) => run_controlplane_host_block_assignment(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.CancelAssignment" => match admit(caller, "controlplane.host.CancelAssignment") {
            Ok(()) => run_controlplane_host_cancel_assignment(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.CancelGoal" => match admit(caller, "controlplane.host.CancelGoal") {
            Ok(()) => run_controlplane_host_cancel_goal(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.ClaimAssignment" => match admit(caller, "controlplane.host.ClaimAssignment") {
            Ok(()) => run_controlplane_host_claim_assignment(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.CompleteAssignment" => match admit(caller, "controlplane.host.CompleteAssignment") {
            Ok(()) => run_controlplane_host_complete_assignment(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.ConfigureRepository" => match admit(caller, "controlplane.host.ConfigureRepository") {
            Ok(()) => run_controlplane_host_configure_repository(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.ConfirmPublication" => match admit(caller, "controlplane.host.ConfirmPublication") {
            Ok(()) => run_controlplane_host_confirm_publication(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.CreateGoal" => match admit(caller, "controlplane.host.CreateGoal") {
            Ok(()) => run_controlplane_host_create_goal(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.DisableRepositoryRegistration" => match admit(caller, "controlplane.host.DisableRepositoryRegistration") {
            Ok(()) => run_controlplane_host_disable_repository_registration(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.EnableRepositoryRegistration" => match admit(caller, "controlplane.host.EnableRepositoryRegistration") {
            Ok(()) => run_controlplane_host_enable_repository_registration(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.GoalList" => run_controlplane_host_goal_list(system),
        "controlplane.host.MarkPublicationUncertain" => match admit(caller, "controlplane.host.MarkPublicationUncertain") {
            Ok(()) => run_controlplane_host_mark_publication_uncertain(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.MergeAssignment" => match admit(caller, "controlplane.host.MergeAssignment") {
            Ok(()) => run_controlplane_host_merge_assignment(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.PauseGoal" => match admit(caller, "controlplane.host.PauseGoal") {
            Ok(()) => run_controlplane_host_pause_goal(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.PreparePublication" => match admit(caller, "controlplane.host.PreparePublication") {
            Ok(()) => run_controlplane_host_prepare_publication(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.PublicationIntentList" => run_controlplane_host_publication_intent_list(system),
        "controlplane.host.QueueAssignment" => match admit(caller, "controlplane.host.QueueAssignment") {
            Ok(()) => run_controlplane_host_queue_assignment(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.ReadyAssignment" => match admit(caller, "controlplane.host.ReadyAssignment") {
            Ok(()) => run_controlplane_host_ready_assignment(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.ReconcileAssignment" => match admit(caller, "controlplane.host.ReconcileAssignment") {
            Ok(()) => run_controlplane_host_reconcile_assignment(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.RecordPlanningProgress" => match admit(caller, "controlplane.host.RecordPlanningProgress") {
            Ok(()) => run_controlplane_host_record_planning_progress(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.RegisterRepository" => match admit(caller, "controlplane.host.RegisterRepository") {
            Ok(()) => run_controlplane_host_register_repository(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.RegisterWorkspace" => match admit(caller, "controlplane.host.RegisterWorkspace") {
            Ok(()) => run_controlplane_host_register_workspace(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.RemoveWorkspaceDirectory" => match admit(caller, "controlplane.host.RemoveWorkspaceDirectory") {
            Ok(()) => run_controlplane_host_remove_workspace_directory(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.RepairAssignment" => match admit(caller, "controlplane.host.RepairAssignment") {
            Ok(()) => run_controlplane_host_repair_assignment(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.RepositoryRegistrationList" => run_controlplane_host_repository_registration_list(system),
        "controlplane.host.ReviewAssignment" => match admit(caller, "controlplane.host.ReviewAssignment") {
            Ok(()) => run_controlplane_host_review_assignment(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.SatisfyGoal" => match admit(caller, "controlplane.host.SatisfyGoal") {
            Ok(()) => run_controlplane_host_satisfy_goal(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.StartGoal" => match admit(caller, "controlplane.host.StartGoal") {
            Ok(()) => run_controlplane_host_start_goal(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.UpdateGoal" => match admit(caller, "controlplane.host.UpdateGoal") {
            Ok(()) => run_controlplane_host_update_goal(system, &input),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "controlplane.host.WorkspaceDirectoryList" => run_controlplane_host_workspace_directory_list(system),
        "controlplane.host.WorkspaceList" => run_controlplane_host_workspace_list(system),
        other => return Err(entry::Refused::Unknown(other.to_owned())),
    };
    let (_, body) = answered?;
    Ok(entry::read(&body))
}

/// Nothing, where `caller` may invoke `command`; otherwise the actor the standard refusal names,
/// `None` where the request was authenticated as no actor.
///
/// Checked before the command runs, on the caller the realization authenticated the request
/// as and never on anything the request says about itself. Public, so code that drives the system
/// in process checks the grant exactly as every route does.
pub fn admit(caller: Option<&crate::actor::Caller>, command: &str) -> Result<(), Option<&'static str>> {
    match caller {
        Some(caller) if caller.may(command) => Ok(()),
        Some(caller) => Err(Some(caller.actor.name())),
        None => Err(None),
    }
}

/// The standard refusal for an actor no grant admits, as the contract declares it: `403`,
/// `{"refused": "not granted", "actor": <name or null>}`.
fn not_granted(actor: Option<&str>) -> http::Response {
    let mut body = String::from("{");
    json::member(&mut body, "refused");
    json::push_text(&mut body, "not granted");
    json::member(&mut body, "actor");
    match actor {
        Some(actor) => json::push_text(&mut body, actor),
        None => body.push_str("null"),
    }
    body.push('}');
    http::Response::new(403, http::JSON, body)
}

/// `POST` `controlplane.host.AddWorkspaceDirectory`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_add_workspace_directory<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_add_workspace_directory(system, &value))
}

/// `controlplane.host.AddWorkspaceDirectory` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_add_workspace_directory<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_add_workspace_directory(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.add_workspace_directory(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_add_workspace_directory(&outcome))
}

/// One declared outcome of `controlplane.host.AddWorkspaceDirectory`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_add_workspace_directory(outcome: &crate::host::AddWorkspaceDirectoryOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::AddWorkspaceDirectoryOutcome::Created { workspace_directory_created, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "created");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.WorkspaceDirectoryCreated");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_workspace_directory_created(workspace_directory_created, &mut body);
            body.push('}');
            body.push(']');
            202
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.ArchiveWorkspace`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_archive_workspace<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_archive_workspace(system, &value))
}

/// `controlplane.host.ArchiveWorkspace` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_archive_workspace<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_archive_workspace(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.archive_workspace(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_archive_workspace(&outcome))
}

/// One declared outcome of `controlplane.host.ArchiveWorkspace`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_archive_workspace(outcome: &crate::host::ArchiveWorkspaceOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::ArchiveWorkspaceOutcome::Applied { archive_workspace_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.ArchiveWorkspaceApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_archive_workspace_applied(archive_workspace_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::ArchiveWorkspaceOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.WorkspaceStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_workspace_state_conflict(error, &mut body);
            409
        }
        crate::host::ArchiveWorkspaceOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.WorkspaceNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.BlockAssignment`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_block_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_block_assignment(system, &value))
}

/// `controlplane.host.BlockAssignment` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_block_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_block_assignment(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.block_assignment(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_block_assignment(&outcome))
}

/// One declared outcome of `controlplane.host.BlockAssignment`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_block_assignment(outcome: &crate::host::BlockAssignmentOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::BlockAssignmentOutcome::Applied { block_assignment_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.BlockAssignmentApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_block_assignment_applied(block_assignment_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::BlockAssignmentOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_assignment_state_conflict(error, &mut body);
            409
        }
        crate::host::BlockAssignmentOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.CancelAssignment`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_cancel_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_cancel_assignment(system, &value))
}

/// `controlplane.host.CancelAssignment` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_cancel_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_cancel_assignment(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.cancel_assignment(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_cancel_assignment(&outcome))
}

/// One declared outcome of `controlplane.host.CancelAssignment`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_cancel_assignment(outcome: &crate::host::CancelAssignmentOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::CancelAssignmentOutcome::Applied { cancel_assignment_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.CancelAssignmentApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_cancel_assignment_applied(cancel_assignment_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::CancelAssignmentOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_assignment_state_conflict(error, &mut body);
            409
        }
        crate::host::CancelAssignmentOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.CancelGoal`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_cancel_goal<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_cancel_goal(system, &value))
}

/// `controlplane.host.CancelGoal` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_cancel_goal<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_cancel_goal(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.cancel_goal(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_cancel_goal(&outcome))
}

/// One declared outcome of `controlplane.host.CancelGoal`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_cancel_goal(outcome: &crate::host::CancelGoalOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::CancelGoalOutcome::Applied { cancel_goal_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.CancelGoalApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_cancel_goal_applied(cancel_goal_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::CancelGoalOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.GoalStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_goal_state_conflict(error, &mut body);
            409
        }
        crate::host::CancelGoalOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.GoalNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.ClaimAssignment`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_claim_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_claim_assignment(system, &value))
}

/// `controlplane.host.ClaimAssignment` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_claim_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_claim_assignment(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.claim_assignment(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_claim_assignment(&outcome))
}

/// One declared outcome of `controlplane.host.ClaimAssignment`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_claim_assignment(outcome: &crate::host::ClaimAssignmentOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::ClaimAssignmentOutcome::Applied { claim_assignment_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.ClaimAssignmentApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_claim_assignment_applied(claim_assignment_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::ClaimAssignmentOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_assignment_state_conflict(error, &mut body);
            409
        }
        crate::host::ClaimAssignmentOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.CompleteAssignment`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_complete_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_complete_assignment(system, &value))
}

/// `controlplane.host.CompleteAssignment` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_complete_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_complete_assignment(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.complete_assignment(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_complete_assignment(&outcome))
}

/// One declared outcome of `controlplane.host.CompleteAssignment`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_complete_assignment(outcome: &crate::host::CompleteAssignmentOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::CompleteAssignmentOutcome::Applied { complete_assignment_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.CompleteAssignmentApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_complete_assignment_applied(complete_assignment_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::CompleteAssignmentOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_assignment_state_conflict(error, &mut body);
            409
        }
        crate::host::CompleteAssignmentOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.ConfigureRepository`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_configure_repository<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_configure_repository(system, &value))
}

/// `controlplane.host.ConfigureRepository` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_configure_repository<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_configure_repository(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.configure_repository(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_configure_repository(&outcome))
}

/// One declared outcome of `controlplane.host.ConfigureRepository`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_configure_repository(outcome: &crate::host::ConfigureRepositoryOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::ConfigureRepositoryOutcome::Applied { configure_repository_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.ConfigureRepositoryApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_configure_repository_applied(configure_repository_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::ConfigureRepositoryOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.RepositoryRegistrationNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.ConfirmPublication`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_confirm_publication<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_confirm_publication(system, &value))
}

/// `controlplane.host.ConfirmPublication` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_confirm_publication<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_confirm_publication(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.confirm_publication(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_confirm_publication(&outcome))
}

/// One declared outcome of `controlplane.host.ConfirmPublication`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_confirm_publication(outcome: &crate::host::ConfirmPublicationOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::ConfirmPublicationOutcome::Applied { confirm_publication_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.ConfirmPublicationApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_confirm_publication_applied(confirm_publication_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::ConfirmPublicationOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.PublicationIntentNotFound");
            404
        }
        crate::host::ConfirmPublicationOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.PublicationIntentStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_publication_intent_state_conflict(error, &mut body);
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.CreateGoal`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_create_goal<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_create_goal(system, &value))
}

/// `controlplane.host.CreateGoal` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_create_goal<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_create_goal(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.create_goal(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_create_goal(&outcome))
}

/// One declared outcome of `controlplane.host.CreateGoal`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_create_goal(outcome: &crate::host::CreateGoalOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::CreateGoalOutcome::Created { goal_created, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "created");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.GoalCreated");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_goal_created(goal_created, &mut body);
            body.push('}');
            body.push(']');
            202
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.DisableRepositoryRegistration`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_disable_repository_registration<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_disable_repository_registration(system, &value))
}

/// `controlplane.host.DisableRepositoryRegistration` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_disable_repository_registration<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_disable_repository_registration(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.disable_repository_registration(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_disable_repository_registration(&outcome))
}

/// One declared outcome of `controlplane.host.DisableRepositoryRegistration`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_disable_repository_registration(outcome: &crate::host::DisableRepositoryRegistrationOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::DisableRepositoryRegistrationOutcome::Applied { disable_repository_registration_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.DisableRepositoryRegistrationApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_disable_repository_registration_applied(disable_repository_registration_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::DisableRepositoryRegistrationOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.RepositoryRegistrationStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_repository_registration_state_conflict(error, &mut body);
            409
        }
        crate::host::DisableRepositoryRegistrationOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.RepositoryRegistrationNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.EnableRepositoryRegistration`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_enable_repository_registration<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_enable_repository_registration(system, &value))
}

/// `controlplane.host.EnableRepositoryRegistration` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_enable_repository_registration<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_enable_repository_registration(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.enable_repository_registration(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_enable_repository_registration(&outcome))
}

/// One declared outcome of `controlplane.host.EnableRepositoryRegistration`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_enable_repository_registration(outcome: &crate::host::EnableRepositoryRegistrationOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::EnableRepositoryRegistrationOutcome::Applied { enable_repository_registration_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.EnableRepositoryRegistrationApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_enable_repository_registration_applied(enable_repository_registration_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::EnableRepositoryRegistrationOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.RepositoryRegistrationStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_repository_registration_state_conflict(error, &mut body);
            409
        }
        crate::host::EnableRepositoryRegistrationOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.RepositoryRegistrationNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.MarkPublicationUncertain`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_mark_publication_uncertain<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_mark_publication_uncertain(system, &value))
}

/// `controlplane.host.MarkPublicationUncertain` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_mark_publication_uncertain<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_mark_publication_uncertain(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.mark_publication_uncertain(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_mark_publication_uncertain(&outcome))
}

/// One declared outcome of `controlplane.host.MarkPublicationUncertain`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_mark_publication_uncertain(outcome: &crate::host::MarkPublicationUncertainOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::MarkPublicationUncertainOutcome::Applied { mark_publication_uncertain_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.MarkPublicationUncertainApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_mark_publication_uncertain_applied(mark_publication_uncertain_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::MarkPublicationUncertainOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.PublicationIntentNotFound");
            404
        }
        crate::host::MarkPublicationUncertainOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.PublicationIntentStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_publication_intent_state_conflict(error, &mut body);
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.MergeAssignment`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_merge_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_merge_assignment(system, &value))
}

/// `controlplane.host.MergeAssignment` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_merge_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_merge_assignment(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.merge_assignment(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_merge_assignment(&outcome))
}

/// One declared outcome of `controlplane.host.MergeAssignment`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_merge_assignment(outcome: &crate::host::MergeAssignmentOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::MergeAssignmentOutcome::Applied { merge_assignment_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.MergeAssignmentApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_merge_assignment_applied(merge_assignment_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::MergeAssignmentOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_assignment_state_conflict(error, &mut body);
            409
        }
        crate::host::MergeAssignmentOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.PauseGoal`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_pause_goal<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_pause_goal(system, &value))
}

/// `controlplane.host.PauseGoal` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_pause_goal<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_pause_goal(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.pause_goal(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_pause_goal(&outcome))
}

/// One declared outcome of `controlplane.host.PauseGoal`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_pause_goal(outcome: &crate::host::PauseGoalOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::PauseGoalOutcome::Applied { pause_goal_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.PauseGoalApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_pause_goal_applied(pause_goal_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::PauseGoalOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.GoalStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_goal_state_conflict(error, &mut body);
            409
        }
        crate::host::PauseGoalOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.GoalNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.PreparePublication`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_prepare_publication<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_prepare_publication(system, &value))
}

/// `controlplane.host.PreparePublication` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_prepare_publication<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_prepare_publication(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.prepare_publication(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_prepare_publication(&outcome))
}

/// One declared outcome of `controlplane.host.PreparePublication`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_prepare_publication(outcome: &crate::host::PreparePublicationOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::PreparePublicationOutcome::Created { publication_intent_created, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "created");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.PublicationIntentCreated");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_publication_intent_created(publication_intent_created, &mut body);
            body.push('}');
            body.push(']');
            202
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.QueueAssignment`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_queue_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_queue_assignment(system, &value))
}

/// `controlplane.host.QueueAssignment` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_queue_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_queue_assignment(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.queue_assignment(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_queue_assignment(&outcome))
}

/// One declared outcome of `controlplane.host.QueueAssignment`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_queue_assignment(outcome: &crate::host::QueueAssignmentOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::QueueAssignmentOutcome::Created { assignment_created, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "created");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.AssignmentCreated");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_assignment_created(assignment_created, &mut body);
            body.push('}');
            body.push(']');
            202
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.ReadyAssignment`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_ready_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_ready_assignment(system, &value))
}

/// `controlplane.host.ReadyAssignment` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_ready_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_ready_assignment(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.ready_assignment(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_ready_assignment(&outcome))
}

/// One declared outcome of `controlplane.host.ReadyAssignment`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_ready_assignment(outcome: &crate::host::ReadyAssignmentOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::ReadyAssignmentOutcome::Applied { ready_assignment_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.ReadyAssignmentApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_ready_assignment_applied(ready_assignment_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::ReadyAssignmentOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_assignment_state_conflict(error, &mut body);
            409
        }
        crate::host::ReadyAssignmentOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.ReconcileAssignment`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_reconcile_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_reconcile_assignment(system, &value))
}

/// `controlplane.host.ReconcileAssignment` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_reconcile_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_reconcile_assignment(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.reconcile_assignment(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_reconcile_assignment(&outcome))
}

/// One declared outcome of `controlplane.host.ReconcileAssignment`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_reconcile_assignment(outcome: &crate::host::ReconcileAssignmentOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::ReconcileAssignmentOutcome::Applied { reconcile_assignment_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.ReconcileAssignmentApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_reconcile_assignment_applied(reconcile_assignment_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::ReconcileAssignmentOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentNotFound");
            404
        }
        crate::host::ReconcileAssignmentOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_assignment_state_conflict(error, &mut body);
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.RecordPlanningProgress`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_record_planning_progress<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_record_planning_progress(system, &value))
}

/// `controlplane.host.RecordPlanningProgress` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_record_planning_progress<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_record_planning_progress(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.record_planning_progress(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_record_planning_progress(&outcome))
}

/// One declared outcome of `controlplane.host.RecordPlanningProgress`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_record_planning_progress(outcome: &crate::host::RecordPlanningProgressOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::RecordPlanningProgressOutcome::Applied { planning_progress_recorded, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.PlanningProgressRecorded");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_planning_progress_recorded(planning_progress_recorded, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::RecordPlanningProgressOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.GoalNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.RegisterRepository`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_register_repository<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_register_repository(system, &value))
}

/// `controlplane.host.RegisterRepository` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_register_repository<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_register_repository(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.register_repository(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_register_repository(&outcome))
}

/// One declared outcome of `controlplane.host.RegisterRepository`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_register_repository(outcome: &crate::host::RegisterRepositoryOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::RegisterRepositoryOutcome::Created { repository_registration_created, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "created");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.RepositoryRegistrationCreated");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_repository_registration_created(repository_registration_created, &mut body);
            body.push('}');
            body.push(']');
            202
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.RegisterWorkspace`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_register_workspace<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_register_workspace(system, &value))
}

/// `controlplane.host.RegisterWorkspace` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_register_workspace<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_register_workspace(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.register_workspace(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_register_workspace(&outcome))
}

/// One declared outcome of `controlplane.host.RegisterWorkspace`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_register_workspace(outcome: &crate::host::RegisterWorkspaceOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::RegisterWorkspaceOutcome::Created { workspace_created, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "created");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.WorkspaceCreated");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_workspace_created(workspace_created, &mut body);
            body.push('}');
            body.push(']');
            202
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.RemoveWorkspaceDirectory`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_remove_workspace_directory<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_remove_workspace_directory(system, &value))
}

/// `controlplane.host.RemoveWorkspaceDirectory` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_remove_workspace_directory<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_remove_workspace_directory(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.remove_workspace_directory(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_remove_workspace_directory(&outcome))
}

/// One declared outcome of `controlplane.host.RemoveWorkspaceDirectory`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_remove_workspace_directory(outcome: &crate::host::RemoveWorkspaceDirectoryOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::RemoveWorkspaceDirectoryOutcome::Applied { workspace_directory_removed, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.WorkspaceDirectoryRemoved");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_workspace_directory_removed(workspace_directory_removed, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::RemoveWorkspaceDirectoryOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.WorkspaceDirectoryStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_workspace_directory_state_conflict(error, &mut body);
            409
        }
        crate::host::RemoveWorkspaceDirectoryOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.WorkspaceDirectoryNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.RepairAssignment`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_repair_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_repair_assignment(system, &value))
}

/// `controlplane.host.RepairAssignment` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_repair_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_repair_assignment(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.repair_assignment(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_repair_assignment(&outcome))
}

/// One declared outcome of `controlplane.host.RepairAssignment`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_repair_assignment(outcome: &crate::host::RepairAssignmentOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::RepairAssignmentOutcome::Applied { repair_assignment_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.RepairAssignmentApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_repair_assignment_applied(repair_assignment_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::RepairAssignmentOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_assignment_state_conflict(error, &mut body);
            409
        }
        crate::host::RepairAssignmentOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.ReviewAssignment`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_review_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_review_assignment(system, &value))
}

/// `controlplane.host.ReviewAssignment` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_review_assignment<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_review_assignment(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.review_assignment(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_review_assignment(&outcome))
}

/// One declared outcome of `controlplane.host.ReviewAssignment`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_review_assignment(outcome: &crate::host::ReviewAssignmentOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::ReviewAssignmentOutcome::Applied { review_assignment_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.ReviewAssignmentApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_review_assignment_applied(review_assignment_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::ReviewAssignmentOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_assignment_state_conflict(error, &mut body);
            409
        }
        crate::host::ReviewAssignmentOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.AssignmentNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.SatisfyGoal`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_satisfy_goal<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_satisfy_goal(system, &value))
}

/// `controlplane.host.SatisfyGoal` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_satisfy_goal<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_satisfy_goal(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.satisfy_goal(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_satisfy_goal(&outcome))
}

/// One declared outcome of `controlplane.host.SatisfyGoal`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_satisfy_goal(outcome: &crate::host::SatisfyGoalOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::SatisfyGoalOutcome::Applied { satisfy_goal_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.SatisfyGoalApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_satisfy_goal_applied(satisfy_goal_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::SatisfyGoalOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.GoalStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_goal_state_conflict(error, &mut body);
            409
        }
        crate::host::SatisfyGoalOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.GoalNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.StartGoal`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_start_goal<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_start_goal(system, &value))
}

/// `controlplane.host.StartGoal` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_start_goal<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_start_goal(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.start_goal(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_start_goal(&outcome))
}

/// One declared outcome of `controlplane.host.StartGoal`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_start_goal(outcome: &crate::host::StartGoalOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::StartGoalOutcome::Applied { start_goal_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.StartGoalApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_start_goal_applied(start_goal_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::StartGoalOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.GoalStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_controlplane_host_goal_state_conflict(error, &mut body);
            409
        }
        crate::host::StartGoalOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.GoalNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `controlplane.host.UpdateGoal`: reads the declared input, runs the port, answers the declared outcome.
fn serve_controlplane_host_update_goal<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, body: &[u8]) -> http::Response
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_controlplane_host_update_goal(system, &value))
}

/// `controlplane.host.UpdateGoal` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it.
fn run_controlplane_host_update_goal<ControlPlaneBehaviors>(system: &mut crate::system::System<ControlPlaneBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    let input = match wire::decode_command_controlplane_host_update_goal(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.control_plane.update_goal(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    let _ = system.take_published();
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_controlplane_host_update_goal(&outcome))
}

/// One declared outcome of `controlplane.host.UpdateGoal`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_controlplane_host_update_goal(outcome: &crate::host::UpdateGoalOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::host::UpdateGoalOutcome::Applied { update_goal_applied, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "applied");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "controlplane.host.UpdateGoalApplied");
            json::member(&mut body, "payload");
            wire::encode_event_controlplane_host_update_goal_applied(update_goal_applied, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::host::UpdateGoalOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "controlplane.host.GoalNotFound");
            404
        }
    };
    body.push('}');
    (status, body)
}

/// `GET` `controlplane.host.AssignmentList` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_controlplane_host_assignment_list<ControlPlaneBehaviors>(system: &crate::system::System<ControlPlaneBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    match system.control_plane.assignment_list() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_controlplane_host_assignment_list(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `controlplane.host.GoalList` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_controlplane_host_goal_list<ControlPlaneBehaviors>(system: &crate::system::System<ControlPlaneBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    match system.control_plane.goal_list() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_controlplane_host_goal_list(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `controlplane.host.PublicationIntentList` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_controlplane_host_publication_intent_list<ControlPlaneBehaviors>(system: &crate::system::System<ControlPlaneBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    match system.control_plane.publication_intent_list() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_controlplane_host_publication_intent_list(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `controlplane.host.RepositoryRegistrationList` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_controlplane_host_repository_registration_list<ControlPlaneBehaviors>(system: &crate::system::System<ControlPlaneBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    match system.control_plane.repository_registration_list() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_controlplane_host_repository_registration_list(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `controlplane.host.WorkspaceDirectoryList` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_controlplane_host_workspace_directory_list<ControlPlaneBehaviors>(system: &crate::system::System<ControlPlaneBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    match system.control_plane.workspace_directory_list() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_controlplane_host_workspace_directory_list(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `controlplane.host.WorkspaceList` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_controlplane_host_workspace_list<ControlPlaneBehaviors>(system: &crate::system::System<ControlPlaneBehaviors>) -> Result<(u16, String), entry::Refused>
where
    ControlPlaneBehaviors: crate::host::obligations::AddWorkspaceDirectoryBehavior + crate::host::obligations::ArchiveWorkspaceBehavior + crate::host::obligations::BlockAssignmentBehavior + crate::host::obligations::CancelAssignmentBehavior + crate::host::obligations::CancelGoalBehavior + crate::host::obligations::ClaimAssignmentBehavior + crate::host::obligations::CompleteAssignmentBehavior + crate::host::obligations::ConfigureRepositoryBehavior + crate::host::obligations::ConfirmPublicationBehavior + crate::host::obligations::CreateGoalBehavior + crate::host::obligations::DisableRepositoryRegistrationBehavior + crate::host::obligations::EnableRepositoryRegistrationBehavior + crate::host::obligations::MarkPublicationUncertainBehavior + crate::host::obligations::MergeAssignmentBehavior + crate::host::obligations::PauseGoalBehavior + crate::host::obligations::PreparePublicationBehavior + crate::host::obligations::QueueAssignmentBehavior + crate::host::obligations::ReadyAssignmentBehavior + crate::host::obligations::ReconcileAssignmentBehavior + crate::host::obligations::RecordPlanningProgressBehavior + crate::host::obligations::RegisterRepositoryBehavior + crate::host::obligations::RegisterWorkspaceBehavior + crate::host::obligations::RemoveWorkspaceDirectoryBehavior + crate::host::obligations::RepairAssignmentBehavior + crate::host::obligations::ReviewAssignmentBehavior + crate::host::obligations::SatisfyGoalBehavior + crate::host::obligations::StartGoalBehavior + crate::host::obligations::UpdateGoalBehavior + crate::host::obligations::AssignmentListQuery + crate::host::obligations::GoalListQuery + crate::host::obligations::PublicationIntentListQuery + crate::host::obligations::RepositoryRegistrationListQuery + crate::host::obligations::WorkspaceDirectoryListQuery + crate::host::obligations::WorkspaceListQuery,
{
    match system.control_plane.workspace_list() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_controlplane_host_workspace_list(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}
