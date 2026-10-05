use std::{collections::VecDeque, path::Path, process::Command, sync::{Arc, Mutex}};
use control_plane_core::{Actor, Store};
use control_plane_runtime::{AgentModel, ModelRequest, RuntimeConfig, Supervisor};
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::sync::Notify;

struct Scripted(Mutex<VecDeque<Value>>);
impl AgentModel for Scripted {
    fn respond(&self, request: &ModelRequest) -> anyhow::Result<Value> {
        let next = self.0.lock().unwrap().pop_front().expect("unexpected idle model call");
        if request.role == "critic" { assert!(request.execution_context.starts_with("critic-")); }
        Ok(next)
    }
}

fn run(root: &Path, program: &str, args: &[&str], env: &[(String,String)]) -> String {
    let out = Command::new(program).args(args).envs(env.iter().cloned()).current_dir(root).output().unwrap();
    assert!(out.status.success(), "{program} {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

async fn setup(existing: bool) -> (TempDir, Arc<tokio::sync::Mutex<Store>>, RuntimeConfig, String, String) {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repos/demo");
    std::fs::create_dir_all(&repo).unwrap();
    run(&repo,"git", &["init","--initial-branch=main"], &[]);
    run(&repo,"git", &["config","user.name","Fixture"], &[]);
    run(&repo,"git", &["config","user.email","fixture@example.invalid"], &[]);
    std::fs::write(repo.join("README.md"),"# Demo\n").unwrap();
    std::fs::create_dir_all(repo.join("ess/domains")).unwrap();
    std::fs::write(repo.join("ess/system.yaml"),"format: ess/22\nsystem: demo\nversion: v1\ndomains: [demo.item]\n").unwrap();
    std::fs::write(repo.join("ess/domains/item.yaml"),"domain: demo.item\nentities:\n  - name: demo.item.Item\n    identity: {name: item_id, type: Uuid}\n    fields: []\n    lifecycle:\n      initial: Present\n      states: [Present]\n      terminal: [Present]\n").unwrap();
    let mut config = RuntimeConfig::default();
    config.environment = vec![("XDG_STATE_HOME".into(),temp.path().join("state").display().to_string()),("XDG_CONFIG_HOME".into(),temp.path().join("config").display().to_string())];
    config.commit_command = vec!["git".into(),"commit".into(),"-m".into()];
    let profile=temp.path().join("profile.toml");
    std::fs::write(&profile,"version = 1\nname = 'fixture'\nexpire_after_seconds = 604800\nprotect_workspace_root = false\n").unwrap();
    run(&repo,"worktree", &["activate","--profile",profile.to_str().unwrap(),"--workspace",temp.path().join("repos").to_str().unwrap()], &config.environment);
    if existing {
        run(&repo,"aep", &["plan","reverse","init","--protocols",&config.aep_protocols,"--profile","development.standard"], &config.environment);
        run(&repo,"aep", &["plan","artifact","new","story","deliver","--title","Deliver requested change"], &config.environment);
        run(&repo,"aep", &["plan","artifact","scope","story:deliver","--add","src/","--inferred"], &config.environment);
        run(&repo,"aep", &["plan","artifact","move","story:deliver","--to","proposed"], &config.environment);
        run(&repo,"aep", &["plan","artifact","move","story:deliver","--to","active"], &config.environment);
    }
    run(&repo,"git", &["add","."], &[]);
    run(&repo,"git", &["commit","-m","fixture"], &[]);
    let mut store=Store::open(temp.path().join("host.sqlite3")).await.unwrap();
    let workspace=store.register_workspace(&repo,"demo").await.unwrap()["published"][0]["payload"]["workspace_id"].as_str().unwrap().to_owned();
    let goal=store.execute("CreateGoal",json!({"workspace_id":workspace,"objective":"Deliver requested change","acceptance":"Required checks pass after reviewed merge","max_workers":3,"max_attempts":3,"max_minutes":1,"planner_model":"scripted","implementor_model":"scripted","reviewer_model":"scripted","merge_authority":true}),Actor::Operator).await.unwrap()["published"][0]["payload"]["goal_id"].as_str().unwrap().to_owned();
    store.execute("StartGoal",json!({"goal_id":goal}),Actor::Operator).await.unwrap();
    let repository=store.query("RepositoryRegistrationList").unwrap()[0]["repository_id"].as_str().unwrap().to_owned();
    (temp,Arc::new(tokio::sync::Mutex::new(store)),config,goal,repository)
}

#[tokio::test]
async fn existing_backlog_is_not_duplicated() {
    let (_temp,store,config,_goal,repo)=setup(true).await;
    let model=Arc::new(Scripted(Mutex::new(VecDeque::from([json!({"action":"finish","stories":["story:deliver"],"summary":"Existing story covers goal."}),json!({"approved":true,"reason":"Current acceptance and scope cover the goal."})]))));
    let supervisor=Supervisor::new(store.clone(),Arc::new(Notify::new()),config,model.clone());
    let first=supervisor.tick().await.unwrap();
    assert_eq!(first.queued,1,"{first:?}");
    let rows=store.lock().await.query("AssignmentList").unwrap();
    assert_eq!(rows.as_array().unwrap().len(),1);
    assert_eq!(rows[0]["story_id"],format!("{repo}::story:deliver"));
    assert!(!rows[0]["candidate"].as_str().unwrap().is_empty());
    assert_eq!(supervisor.tick().await.unwrap().queued,0);
    assert!(model.0.lock().unwrap().is_empty());
}

#[tokio::test]
async fn goal_drives_plan() {
    let (_temp,store,config,_goal,_repo)=setup(false).await;
    let model=Arc::new(Scripted(Mutex::new(VecDeque::from([
        json!({"action":"aep","args":["new","story","deliver","--title","Deliver requested change","--from","-"],"body":"## Acceptance\nNamed scenario: delivery_works.\n## Scope\nsrc/\n"}),
        json!({"action":"aep","args":["scope","story:deliver","--add","src/","--inferred"],"body":null}),
        json!({"action":"finish","stories":["story:deliver"],"summary":"New ESS-backed work."}),
        json!({"approved":true,"reason":"Validated specification and named scenario."})
    ]))));
    let supervisor=Supervisor::new(store.clone(),Arc::new(Notify::new()),config,model);
    let report=supervisor.tick().await.unwrap();
    assert_eq!(report.queued,1,"{report:?}");
    assert_eq!(store.lock().await.query("AssignmentList").unwrap()[0]["goal_revision"],1);
}

#[tokio::test]
async fn goal_completion_requires_evidence() {
    let (_temp,store,config,_goal,_repo)=setup(true).await;
    let model=Arc::new(Scripted(Mutex::new(VecDeque::from([json!({"action":"finish","stories":[],"summary":"Nothing else to plan."}),json!({"approved":true,"reason":"No further proposal."})]))));
    let supervisor=Supervisor::new(store.clone(),Arc::new(Notify::new()),config,model);
    let report=supervisor.tick().await.unwrap();
    assert_eq!(report.queued,0);
    assert_eq!(store.lock().await.query("GoalList").unwrap()[0]["state"],"Running");
}
