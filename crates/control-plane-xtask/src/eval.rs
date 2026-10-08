//! Trusted black-box checks. Generated application source stays outside this repository.
use anyhow::{Context, Result, bail, ensure};
use clap::{Subcommand, ValueEnum};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

#[path = "eval_report.rs"]
mod report;

#[derive(Subcommand)]
pub enum Action {
    /// Seed three empty Go tasks and activate their independent managed-worktree profile.
    Init {
        #[arg(long)]
        root: PathBuf,
    },
    /// Check a candidate without accepting its own claims or changing its source.
    Verify {
        #[arg(long, value_enum)]
        case: Case,
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
    /// Report whether one recorded goal ran unattended, from a stopped run's state store.
    Report {
        /// The state store the run recorded; read from a copy, never written.
        #[arg(long)]
        state: PathBuf,
        /// The goal to report.
        #[arg(long)]
        goal: String,
        /// The repository the goal merged into; its origin holds the target branch.
        #[arg(long)]
        repo: PathBuf,
    },
}
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Case {
    #[value(name = "go-cli")]
    Cli,
    #[value(name = "go-json-http")]
    JsonHttp,
    #[value(name = "go-auth-web")]
    AuthWeb,
}
impl Case {
    fn name(self) -> &'static str {
        match self {
            Self::Cli => "go-cli",
            Self::JsonHttp => "go-json-http",
            Self::AuthWeb => "go-auth-web",
        }
    }
    fn brief(self) -> &'static str {
        match self {
            Self::Cli => {
                "Build a Go standard-library greeting CLI at cmd/greet. `go run ./cmd/greet Ada` prints exactly Hello, Ada! followed by a newline. Missing arguments exit nonzero. Add unit tests and README.md with run and test instructions. Named acceptance scenarios: greet-name, reject-missing-name, documented-setup."
            }
            Self::JsonHttp => {
                "Build a Go standard-library HTTP JSON backend at cmd/server, listening on ADDR. GET /healthz returns 200. POST /api/echo accepts JSON {\"message\":\"hello\"} and returns exactly that JSON object with status 200 and application/json. Malformed JSON returns 400. Add meaningful Go tests and README.md with ADDR, go run ./cmd/server and go test ./... instructions. Named acceptance scenarios: healthy-server, echo-json, malformed-json, documented-setup."
            }
            Self::AuthWeb => {
                "Build a Go standard-library HTTP example with authentication, frontend, backend, tests and README.md. Entry point cmd/server listens on ADDR. Credentials come from APP_USERNAME and APP_PASSWORD; do not embed credentials. GET /healthz is public 200. GET / presents an HTML login form with username/password inputs. POST /login accepts URL-encoded username/password, rejects incorrect credentials with 401, and redirects successful login with 303. Use an unpredictable server-side session and a cookie with HttpOnly, SameSite=Lax or Strict, and Path=/. GET /api/me without a valid session returns 401; authenticated requests return JSON {\"username\":\"eval-user\"} for that configured user. Authenticated GET / shows a welcome page with logout. POST /logout invalidates the server-side session and redirects with 303; replaying its old cookie must return 401 at /api/me. Include frontend HTML/CSS (server-rendered is sufficient), backend Go tests, and README.md documenting ADDR, APP_USERNAME, APP_PASSWORD, go run ./cmd/server, go test ./..., and that this is a local example. Named acceptance scenarios: healthy-server, login-frontend, unauthenticated-denied, invalid-login, authenticated-identity, authenticated-frontend, invalid-session, logout-revokes-session, documented-setup. Implement this as one small cohesive story with scope cmd/, internal/, web/, README.md, go.mod as needed."
            }
        }
    }
}
fn command(repo: &Path, program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(repo)
        .output()?;
    ensure!(
        output.status.success(),
        "{program} {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?)
}
pub fn run(action: Action) -> Result<()> {
    match action {
        Action::Init { root } => init(&root),
        Action::Verify { case, repo } => {
            tokio::runtime::Runtime::new()?.block_on(verify(case, &repo))?;
            println!("PASS {}", case.name());
            Ok(())
        }
        Action::Report { state, goal, repo } => report::run(&state, &goal, &repo),
    }
}
fn init(root: &Path) -> Result<()> {
    fs::create_dir_all(root)?;
    let root = root.canonicalize()?;
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .canonicalize()?;
    ensure!(
        !root.starts_with(&source),
        "eval root must be outside the control-plane repository"
    );
    ensure!(
        !root.join("repos").exists(),
        "eval root already initialized; refusing to overwrite repositories"
    );
    fs::create_dir(root.join("repos"))?;
    fs::create_dir(root.join("remotes"))?;
    fs::write(
        root.join("worktree.toml"),
        "version = 1\nname = 'control-plane-evals'\nexpire_after_seconds = 604800\nprotect_workspace_root = true\n",
    )?;
    command(
        &root,
        "worktree",
        &[
            "activate",
            "--profile",
            "worktree.toml",
            "--workspace",
            root.join("repos").to_str().context("UTF-8 root")?,
        ],
    )?;
    for case in [Case::Cli, Case::JsonHttp, Case::AuthWeb] {
        let repo = root.join("repos").join(case.name());
        fs::create_dir(&repo)?;
        fs::write(
            repo.join("go.mod"),
            format!("module example.invalid/{}\n\ngo 1.23\n", case.name()),
        )?;
        fs::write(repo.join(".gitignore"), "/bin/\n")?;
        fs::write(
            repo.join("AGENTS.md"),
            "# External evaluation repository\n\nThis is a standalone local Go example, outside the beyond10x organization source tree. Use Go standard library and HTML/CSS. No external dependencies. Keep the implementation small. Follow TASK.md; do not edit it, this file, or the external verifier. The host owns AEP lifecycle, testing, review, commits and publication. Include automated Go tests. The planner must establish a valid minimal ESS specification before creating AEP stories, as required by the host. Named application acceptance scenarios in TASK.md are checked by the external verifier.\n",
        )?;
        fs::write(
            repo.join("TASK.md"),
            format!("# {}\n\n{}\n", case.name(), case.brief()),
        )?;
        command(&repo, "git", &["init", "--initial-branch=main"])?;
        command(&repo, "git", &["config", "user.name", "Control Plane Eval"])?;
        command(
            &repo,
            "git",
            &["config", "user.email", "eval@example.invalid"],
        )?;
        command(&repo, "git", &["add", "."])?;
        command(
            &repo,
            "git",
            &["commit", "-m", "Seed isolated evaluation task"],
        )?;
        let remote = root.join("remotes").join(format!("{}.git", case.name()));
        command(
            &root,
            "git",
            &[
                "init",
                "--bare",
                "--initial-branch=main",
                remote.to_str().context("UTF-8 origin")?,
            ],
        )?;
        command(
            &repo,
            "git",
            &["remote", "add", "origin", remote.to_str().unwrap()],
        )?;
        command(&repo, "git", &["push", "-u", "origin", "main"])?;
        println!("{}: {}", case.name(), repo.display());
    }
    fs::write(
        root.join("README.md"),
        "# Control-plane evaluations\n\nThree independent local repositories: go-cli, go-json-http, go-auth-web. TASK.md holds each fixed brief. Register each as a separate workspace. Run sequentially with one worker, one attempt and a ten-minute goal budget. Repository acceptance runs the trusted control-plane-xtask eval verify command, not a check supplied by the generated app. Keep failed runs as evidence; do not silently retry. Local origins are under remotes/. No GitHub publication is involved.\n",
    )?;
    Ok(())
}
struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
async fn verify(case: Case, repo: &Path) -> Result<()> {
    let readme = fs::read_to_string(repo.join("README.md"))
        .context("documented-setup: missing README.md")?;
    for required in ["go run", "go test"] {
        ensure!(
            readme.contains(required),
            "documented-setup: README lacks {required}"
        );
    }
    let tests = command(repo, "go", &["test", "-count=1", "./..."])?;
    ensure!(
        tests
            .lines()
            .any(|line| line.starts_with("ok\t") || line.starts_with("ok ")),
        "no Go test package executed: {tests}"
    );
    command(repo, "go", &["vet", "./..."])?;
    let scratch = tempfile::tempdir()?;
    let binary = scratch.path().join("app");
    let entry = if matches!(case, Case::Cli) {
        "./cmd/greet"
    } else {
        "./cmd/server"
    };
    command(
        repo,
        "go",
        &["build", "-o", binary.to_str().unwrap(), entry],
    )?;
    if matches!(case, Case::Cli) {
        ensure!(
            command(repo, binary.to_str().unwrap(), &["Ada"])? == "Hello, Ada!\n",
            "greet-name: incorrect output"
        );
        ensure!(
            !Command::new(&binary)
                .current_dir(repo)
                .output()?
                .status
                .success(),
            "reject-missing-name: accepted missing argument"
        );
        return Ok(());
    }
    ensure!(
        readme.contains("ADDR"),
        "documented-setup: README lacks ADDR"
    );
    let socket = std::net::TcpListener::bind("127.0.0.1:0")?;
    let address = socket.local_addr()?;
    drop(socket);
    let password = format!("eval-{}-{}", std::process::id(), address.port());
    let mut server = Server(
        Command::new(&binary)
            .current_dir(repo)
            .env("ADDR", address.to_string())
            .env("APP_USERNAME", "eval-user")
            .env("APP_PASSWORD", &password)
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()?,
    );
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(3))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let base = format!("http://{address}");
    let mut ready = false;
    for _ in 0..50 {
        if client
            .get(format!("{base}/healthz"))
            .send()
            .await
            .is_ok_and(|r| r.status() == 200)
        {
            ready = true;
            break;
        }
        if let Some(status) = server.0.try_wait()? {
            bail!("healthy-server: server exited {status}");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    ensure!(ready, "healthy-server: did not become ready");
    if matches!(case, Case::JsonHttp) {
        let response = client
            .post(format!("{base}/api/echo"))
            .json(&serde_json::json!({"message":"hello"}))
            .send()
            .await?;
        ensure!(
            response.status() == 200
                && response
                    .headers()
                    .get("content-type")
                    .is_some_and(|h| h.to_str().unwrap_or("").starts_with("application/json")),
            "echo-json: status/content type"
        );
        ensure!(
            response.json::<serde_json::Value>().await? == serde_json::json!({"message":"hello"}),
            "echo-json: body"
        );
        ensure!(
            client
                .post(format!("{base}/api/echo"))
                .header("content-type", "application/json")
                .body("{")
                .send()
                .await?
                .status()
                == 400,
            "malformed-json: not rejected"
        );
        return Ok(());
    }
    for required in ["APP_USERNAME", "APP_PASSWORD"] {
        ensure!(
            readme.contains(required),
            "documented-setup: README lacks {required}"
        );
    }
    let response = client.get(format!("{base}/")).send().await?;
    ensure!(response.status() == 200, "login-frontend: status");
    let html = response.text().await?.to_lowercase();
    ensure!(
        html.contains("<form") && html.contains("username") && html.contains("password"),
        "login-frontend: missing form"
    );
    ensure!(
        client.get(format!("{base}/api/me")).send().await?.status() == 401,
        "unauthenticated-denied"
    );
    ensure!(
        client
            .post(format!("{base}/login"))
            .form(&[("username", "eval-user"), ("password", "wrong")])
            .send()
            .await?
            .status()
            == 401,
        "invalid-login"
    );
    let login = client
        .post(format!("{base}/login"))
        .form(&[("username", "eval-user"), ("password", &password)])
        .send()
        .await?;
    ensure!(login.status() == 303, "login: expected 303");
    let cookie = login
        .headers()
        .get("set-cookie")
        .context("login: missing cookie")?
        .to_str()?
        .to_owned();
    let lower = cookie.to_lowercase();
    ensure!(
        lower.contains("httponly")
            && (lower.contains("samesite=lax") || lower.contains("samesite=strict"))
            && lower.contains("path=/"),
        "login: cookie flags"
    );
    let cookie = cookie.split(';').next().unwrap();
    let me = client
        .get(format!("{base}/api/me"))
        .header("cookie", cookie)
        .send()
        .await?;
    ensure!(
        me.status() == 200
            && me.json::<serde_json::Value>().await? == serde_json::json!({"username":"eval-user"}),
        "authenticated-identity"
    );
    let page = client
        .get(format!("{base}/"))
        .header("cookie", cookie)
        .send()
        .await?;
    ensure!(
        page.status() == 200 && page.text().await?.to_lowercase().contains("logout"),
        "authenticated-frontend"
    );
    let forged = format!("{}=invalid", cookie.split('=').next().unwrap());
    ensure!(
        client
            .get(format!("{base}/api/me"))
            .header("cookie", forged)
            .send()
            .await?
            .status()
            == 401,
        "invalid-session"
    );
    ensure!(
        client
            .post(format!("{base}/logout"))
            .header("cookie", cookie)
            .send()
            .await?
            .status()
            == 303,
        "logout: expected 303"
    );
    ensure!(
        client
            .get(format!("{base}/api/me"))
            .header("cookie", cookie)
            .send()
            .await?
            .status()
            == 401,
        "logout-revokes-session"
    );
    Ok(())
}
