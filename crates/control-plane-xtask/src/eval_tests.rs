//! The trusted verifier and `eval init` against disposable roots under the tree's `.scratch`,
//! deleted when each test ends. Seeding runs without the `worktree` binary.
use super::*;

/// A fresh directory under the tree's `.scratch`, deleted when the guard drops.
fn scratch() -> tempfile::TempDir {
    let parent = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.scratch/eval-verify");
    fs::create_dir_all(&parent).unwrap();
    tempfile::tempdir_in(parent).unwrap()
}

fn verified(case: Case, repo: &Path) -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(verify(case, repo))
}

fn write(repo: &Path, file: &str, contents: &str) {
    let path = repo.join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn go_test_file_without_tests_is_not_executed_tests() {
    let scratch = scratch();
    let repo = scratch.path();
    write(repo, "go.mod", "module example.invalid/go-cli\n\ngo 1.23\n");
    write(
        repo,
        "README.md",
        "# greet\n\n`go run ./cmd/greet Ada` greets; `go test ./...` tests.\n",
    );
    write(
        repo,
        "cmd/greet/main.go",
        "package main\n\nimport (\n\t\"fmt\"\n\t\"os\"\n)\n\nfunc main() {\n\tif len(os.Args) != 2 {\n\t\tos.Exit(2)\n\t}\n\tfmt.Printf(\"Hello, %s!\\n\", os.Args[1])\n}\n",
    );
    // A test file with no test function: `go test` prints `ok … [no tests to run]`.
    write(repo, "cmd/greet/main_test.go", "package main\n");
    let error = verified(Case::Cli, repo)
        .expect_err("a seed whose test file holds no test passed the trusted verifier");
    assert!(
        error.to_string().contains("no Go test executed"),
        "failed for another reason: {error:#}"
    );
    // The same seed with one test function passes, so the refusal is the executed-tests check.
    write(
        repo,
        "cmd/greet/main_test.go",
        "package main\n\nimport \"testing\"\n\nfunc TestGreets(t *testing.T) {}\n",
    );
    verified(Case::Cli, repo).unwrap();
}

/// The control-plane repository, as `eval init` reads it.
fn control_plane() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// `eval init` without the `worktree` binary: the scratch root lies inside this repository, so
/// a sibling stands in for it as the repository the root must stay outside.
fn initialized() -> (tempfile::TempDir, PathBuf) {
    let scratch = scratch();
    let root = scratch.path().join("root");
    let source = scratch.path().join("source");
    fs::create_dir(&source).unwrap();
    init_with(&root, &source, |_| Ok(())).unwrap();
    let root = root.canonicalize().unwrap();
    (scratch, root)
}

fn blank_seed_fails(case: Case) {
    let (_scratch, root) = initialized();
    let repo = root.join("repos").join(case.name());
    assert!(
        repo.join("TASK.md").is_file(),
        "{} was not seeded",
        case.name()
    );
    let error = verified(case, &repo).expect_err("a blank seed passed the trusted verifier");
    assert!(
        error.to_string().starts_with("documented-setup"),
        "{error:#}"
    );
}

#[test]
fn blank_go_cli_seed_fails_trusted_verifier() {
    blank_seed_fails(Case::Cli);
}

#[test]
fn blank_go_json_http_seed_fails_trusted_verifier() {
    blank_seed_fails(Case::JsonHttp);
}

#[test]
fn blank_go_auth_web_seed_fails_trusted_verifier() {
    blank_seed_fails(Case::AuthWeb);
}

fn git(repo: &Path, args: &[&str]) -> String {
    command(repo, "git", args).unwrap().trim().to_owned()
}

#[test]
fn eval_init_gives_each_case_a_local_bare_origin() {
    let (_scratch, root) = initialized();
    let runner = control_plane_runtime::process::ProcessRunner {
        environment: vec![],
        timeout: Duration::from_secs(5),
        cancel: Default::default(),
    };
    let config = control_plane_runtime::RuntimeConfig {
        local_eval_root: Some(root.clone()),
        ..Default::default()
    };
    for case in [Case::Cli, Case::JsonHttp, Case::AuthWeb] {
        let repo = root.join("repos").join(case.name());
        let origin = PathBuf::from(git(&repo, &["remote", "get-url", "origin"]));
        assert_eq!(
            origin,
            root.join("remotes").join(format!("{}.git", case.name()))
        );
        assert_eq!(git(&origin, &["rev-parse", "--is-bare-repository"]), "true");
        assert_eq!(
            git(&origin, &["rev-parse", "refs/heads/main"]),
            git(&repo, &["rev-parse", "HEAD"]),
            "{}: origin main is not the seed",
            case.name()
        );
        assert!(
            config.local_eval(&repo, &runner).unwrap(),
            "{}",
            case.name()
        );
    }
}

#[test]
fn eval_init_refuses_root_inside_control_plane_repository() {
    let scratch = scratch();
    let error = init_with(scratch.path(), &control_plane(), |_| {
        panic!("activated a root inside the control-plane repository")
    })
    .expect_err("a root inside the control-plane repository was initialized");
    assert!(
        error
            .to_string()
            .contains("outside the control-plane repository"),
        "{error:#}"
    );
    assert_eq!(fs::read_dir(scratch.path()).unwrap().count(), 0);
}
