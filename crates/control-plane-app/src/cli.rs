use super::*;
use clap::{Args, Parser, Subcommand};
use std::path::{Path as FsPath, PathBuf};

#[derive(Debug, Parser)]
#[command(
    name = "control-plane",
    version,
    about = "Local autonomous engineering workspaces"
)]
pub struct Cli {
    #[arg(long, global = true, default_value = "http://127.0.0.1:8787")]
    pub url: String,
    #[command(subcommand)]
    pub command: Command,
}
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start the local service and browser console.
    Serve {
        #[arg(long, default_value_os_t=default_state())]
        state: PathBuf,
        #[arg(long, default_value = "127.0.0.1:8787")]
        listen: SocketAddr,
        /// Register these startup directories instead of the current directory; repeatable.
        #[arg(long)]
        workspace: Vec<PathBuf>,
        /// Private Gates policy for trusted planner commits; otherwise inherit B10X_GATES_POLICY.
        #[arg(long)]
        gates_policy: Option<PathBuf>,
        /// Allow Go and local Git commits only for eval repositories and origins beneath this root.
        #[arg(long)]
        local_eval_root: Option<PathBuf>,
    },
    /// Add and inspect workspaces through the running service.
    Workspace {
        #[command(subcommand)]
        action: WorkspaceAction,
    },
    /// Set goals and control execution through the running service.
    Goal {
        #[command(subcommand)]
        action: GoalAction,
    },
    /// Configure a registered repository's existing test and publishing tools.
    Repository {
        #[command(subcommand)]
        action: RepositoryAction,
    },
    /// Show current workspaces, goals, assignments and publication receipts.
    Status,
}
#[derive(Debug, Subcommand)]
pub enum WorkspaceAction {
    /// Add the current directory or an explicitly selected directory.
    Add {
        path: Option<PathBuf>,
        #[arg(long)]
        name: Option<String>,
    },
    List,
    Directories {
        workspace_id: String,
    },
    AddDirectory {
        workspace_id: String,
        path: PathBuf,
    },
    RemoveDirectory {
        workspace_id: String,
        directory_id: String,
    },
}
#[derive(Debug, Args)]
pub struct GoalSettings {
    #[arg(long)]
    objective: String,
    #[arg(long)]
    acceptance: String,
    #[arg(long, default_value_t = 3)]
    max_workers: i64,
    #[arg(long, default_value_t = 3)]
    max_attempts: i64,
    #[arg(long, default_value_t = 60)]
    max_minutes: i64,
    #[arg(long, default_value = "gpt-5.6-sol")]
    planner_model: String,
    #[arg(long, default_value = "gpt-5.6-sol")]
    implementor_model: String,
    #[arg(long, default_value = "gpt-5.6-sol")]
    reviewer_model: String,
    #[arg(long)]
    allow_merges: bool,
}
impl GoalSettings {
    fn body(self) -> Value {
        json!({"objective":self.objective,"acceptance":self.acceptance,"max_workers":self.max_workers,"max_attempts":self.max_attempts,"max_minutes":self.max_minutes,"planner_model":self.planner_model,"implementor_model":self.implementor_model,"reviewer_model":self.reviewer_model,"merge_authority":self.allow_merges})
    }
}
#[derive(Debug, Args)]
pub struct GoalChanges {
    #[arg(long)]
    objective: Option<String>,
    #[arg(long)]
    acceptance: Option<String>,
    #[arg(long)]
    max_workers: Option<i64>,
    #[arg(long)]
    max_attempts: Option<i64>,
    #[arg(long)]
    max_minutes: Option<i64>,
    #[arg(long)]
    planner_model: Option<String>,
    #[arg(long)]
    implementor_model: Option<String>,
    #[arg(long)]
    reviewer_model: Option<String>,
    /// Explicit true or false; omitted settings retain their current values.
    #[arg(long,action=clap::ArgAction::Set)]
    merge_authority: Option<bool>,
}
#[derive(Debug, Subcommand)]
pub enum GoalAction {
    Create {
        workspace_id: String,
        #[command(flatten)]
        settings: GoalSettings,
    },
    Edit {
        goal_id: String,
        #[command(flatten)]
        changes: GoalChanges,
    },
    Start {
        goal_id: String,
    },
    Pause {
        goal_id: String,
    },
    Cancel {
        goal_id: String,
    },
    /// Delete a cancelled goal with no assignment history.
    Delete {
        goal_id: String,
    },
    List,
}
#[derive(Debug, Subcommand)]
pub enum RepositoryAction {
    Configure {
        repository_id: String,
        #[arg(long)]
        base_branch: String,
        #[arg(long)]
        test_command: String,
        #[arg(long)]
        publish_command: String,
    },
    Enable {
        repository_id: String,
    },
    Disable {
        repository_id: String,
    },
    List,
}
fn default_state() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".local/state")
        })
        .join("control-plane/state.sqlite")
}

/// Loopback-only client. CSRF session retrieval and mutation use the same admitted handlers as forms.
#[derive(Clone)]
pub struct Client {
    base: String,
    http: reqwest::Client,
}
impl Client {
    pub fn new(base: &str) -> Result<Self> {
        let url = reqwest::Url::parse(base)?;
        let local = url.host_str().is_some_and(|host| {
            host == "localhost"
                || host
                    .trim_matches(['[', ']'])
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
        });
        ensure!(
            url.scheme() == "http"
                && local
                && url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none()
                && url.path() == "/",
            "service URL must name a loopback HTTP origin"
        );
        Ok(Self {
            base: base.trim_end_matches('/').into(),
            http: reqwest::Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
        })
    }
    pub async fn snapshot(&self) -> Result<Value> {
        self.read("/api/state").await
    }
    async fn read(&self, path: &str) -> Result<Value> {
        answer(self.http.get(format!("{}{path}", self.base)).send().await?).await
    }
    async fn post(&self, path: &str, body: Value) -> Result<Value> {
        let session = self.read("/api/session").await?;
        let token = session["csrf_token"]
            .as_str()
            .context("service session did not supply a request token")?;
        answer(
            self.http
                .post(format!("{}{path}", self.base))
                .header("x-csrf-token", token)
                .json(&body)
                .send()
                .await?,
        )
        .await
    }
    pub async fn add_workspace(&self, path: &FsPath, name: &str) -> Result<Value> {
        self.post("/api/workspaces", json!({"path":path,"name":name}))
            .await
    }
    pub async fn directories(&self, workspace: &str) -> Result<Value> {
        self.read(&format!("/api/workspaces/{workspace}/directories"))
            .await
    }
    pub async fn add_directory(&self, workspace: &str, path: &FsPath) -> Result<Value> {
        self.post(
            &format!("/api/workspaces/{workspace}/directories"),
            json!({"path":path}),
        )
        .await
    }
    pub async fn remove_directory(&self, workspace: &str, directory: &str) -> Result<Value> {
        self.post(
            &format!("/api/workspaces/{workspace}/directories/{directory}/remove"),
            json!({}),
        )
        .await
    }
    pub async fn command(&self, name: &str, body: Value) -> Result<Value> {
        ensure!(OPERATOR_COMMANDS.contains(&name), "not an operator command");
        self.post(&format!("/api/commands/{name}"), body).await
    }
}
async fn answer(response: reqwest::Response) -> Result<Value> {
    let status = response.status();
    let body = response.text().await?;
    ensure!(
        status.is_success(),
        "service refused request ({status}): {body}"
    );
    Ok(serde_json::from_str(&body)?)
}

pub async fn run(cli: Cli) -> Result<Option<Value>> {
    if let Command::Serve {
        state,
        listen,
        workspace,
        gates_policy,
        local_eval_root,
    } = cli.command
    {
        ensure!(
            listen.ip().is_loopback(),
            "control-plane serves loopback addresses only"
        );
        let listener = tokio::net::TcpListener::bind(listen).await?;
        let address = listener.local_addr()?;
        let model_sessions = state.with_extension("loom-sessions");
        let mut store = Store::open(state).await?;
        initialize_workspaces(&mut store, &std::env::current_dir()?, &workspace).await?;
        let store = Arc::new(Mutex::new(store));
        let app = AppState::new(store, address, Arc::new(Notify::new()));
        let mut config = control_plane_runtime::RuntimeConfig {
            local_eval_root: local_eval_root
                .map(|root| root.canonicalize())
                .transpose()?,
            ..Default::default()
        };
        if let Some(policy) = gates_policy {
            config.environment.push((
                "B10X_GATES_POLICY".into(),
                policy
                    .canonicalize()?
                    .to_str()
                    .context("Gates policy path must be UTF-8")?
                    .into(),
            ));
        }
        let shutdown = tokio_util::sync::CancellationToken::new();
        let signal_shutdown = shutdown.clone();
        let signals = tokio::spawn(async move {
            shutdown_signal().await;
            signal_shutdown.cancel();
        });
        eprintln!("Control plane: http://{address}");
        let result = serve_with_runtime(
            listener,
            app,
            config,
            Arc::new(control_plane_runtime::CodexAgentModel::new(model_sessions)),
            shutdown,
        )
        .await;
        signals.abort();
        result?;
        return Ok(None);
    }
    let client = Client::new(&cli.url)?;
    let value=match cli.command {
        Command::Serve{..}=>unreachable!(),
        Command::Status=>client.snapshot().await?,
        Command::Workspace{action}=>match action {
            WorkspaceAction::List=>client.read("/api/workspaces").await?["workspaces"].clone(),
            WorkspaceAction::Directories{workspace_id}=>client.directories(&workspace_id).await?,
            WorkspaceAction::AddDirectory{workspace_id,path}=>client.add_directory(&workspace_id,&path.canonicalize()?).await?,
            WorkspaceAction::RemoveDirectory{workspace_id,directory_id}=>client.remove_directory(&workspace_id,&directory_id).await?,
            WorkspaceAction::Add{path,name}=>{
                let path=path.unwrap_or(std::env::current_dir()?).canonicalize()?;
                let name=name.unwrap_or_else(||path.file_name().unwrap_or_default().to_string_lossy().into_owned());
                client.add_workspace(&path,&name).await?
            }
        },
        Command::Goal{action}=>match action{
            GoalAction::Create{workspace_id,settings}=>{let mut body=settings.body();body["workspace_id"]=json!(workspace_id);client.command("CreateGoal",body).await?},
            GoalAction::Edit{goal_id,changes}=>{
                let view=client.snapshot().await?;
                let current=rows(&view,"goals")?.iter().find(|goal|field(goal,"goal_id")==goal_id).context("goal not found")?;
                let mut body=json!({"goal_id":goal_id});
                for key in ["objective","acceptance","max_workers","max_attempts","max_minutes","planner_model","implementor_model","reviewer_model","merge_authority"]{body[key]=current[key].clone();}
                for (key,value) in [("objective",changes.objective),("acceptance",changes.acceptance),("planner_model",changes.planner_model),("implementor_model",changes.implementor_model),("reviewer_model",changes.reviewer_model)]{if let Some(value)=value{body[key]=json!(value);}}
                for (key,value) in [("max_workers",changes.max_workers),("max_attempts",changes.max_attempts),("max_minutes",changes.max_minutes)]{if let Some(value)=value{body[key]=json!(value);}}
                if let Some(value)=changes.merge_authority{body["merge_authority"]=json!(value);}
                client.command("UpdateGoal",body).await?
            },
            GoalAction::Start{goal_id}=>client.command("StartGoal",json!({"goal_id":goal_id})).await?,
            GoalAction::Pause{goal_id}=>client.command("PauseGoal",json!({"goal_id":goal_id})).await?,
            GoalAction::Cancel{goal_id}=>client.command("CancelGoal",json!({"goal_id":goal_id})).await?,
            GoalAction::Delete{goal_id}=>client.command("DeleteGoal",json!({"goal_id":goal_id})).await?,
            GoalAction::List=>client.snapshot().await?["goals"].clone(),
        },
        Command::Repository{action}=>match action{
            RepositoryAction::Configure{repository_id,base_branch,test_command,publish_command}=>client.command("ConfigureRepository",json!({"repository_id":repository_id,"base_branch":base_branch,"test_command":test_command,"publish_command":publish_command})).await?,
            RepositoryAction::Enable{repository_id}=>client.command("EnableRepositoryRegistration",json!({"repository_id":repository_id})).await?,
            RepositoryAction::Disable{repository_id}=>client.command("DisableRepositoryRegistration",json!({"repository_id":repository_id})).await?,
            RepositoryAction::List=>client.snapshot().await?["repositories"].clone(),
        }
    };
    Ok(Some(value))
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut terminate) => {
                tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
            }
            Err(_) => {
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
