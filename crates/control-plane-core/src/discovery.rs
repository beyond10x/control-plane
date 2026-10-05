use anyhow::{Context, Result, bail, ensure};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone)]
pub struct DiscoveredRepository {
    pub name: String,
    pub path: PathBuf,
    pub common_dir: PathBuf,
    pub base_branch: String,
}
#[derive(Debug, Clone)]
pub struct DiscoveredWorkspace {
    pub path: PathBuf,
    pub repositories: Vec<DiscoveredRepository>,
}

pub fn discover(path: &Path) -> Result<DiscoveredWorkspace> {
    let path = path
        .canonicalize()
        .context("workspace directory does not exist")?;
    ensure!(path.is_dir(), "workspace must be a directory");
    if git(&path, &["rev-parse", "--show-toplevel"]).is_ok() {
        let repo = repository(&path)?;
        return Ok(DiscoveredWorkspace {
            path: repo.path.clone(),
            repositories: vec![repo],
        });
    }
    let mut repositories = Vec::new();
    for entry in std::fs::read_dir(&path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() && entry.path().join(".git").exists() {
            repositories.push(repository(&entry.path())?);
        }
    }
    repositories.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(DiscoveredWorkspace { path, repositories })
}

pub(crate) fn repository(path: &Path) -> Result<DiscoveredRepository> {
    let root = PathBuf::from(git(path, &["rev-parse", "--show-toplevel"])?).canonicalize()?;
    let common = PathBuf::from(git(
        &root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?)
    .canonicalize()?;
    let branch = match git(&root, &["symbolic-ref", "--short", "HEAD"]) {
        Ok(branch) => branch,
        Err(_) => {
            let branches = git(
                &root,
                &[
                    "for-each-ref",
                    "--points-at=HEAD",
                    "--format=%(refname:short)",
                    "refs/heads",
                ],
            )?;
            let names: Vec<_> = branches.lines().collect();
            if names.len() == 1 {
                names[0].to_owned()
            } else {
                let remote = git(&root, &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])
                    .context("detached Git repository needs an unambiguous base branch; select a branch before registering it")?;
                remote
                    .strip_prefix("origin/")
                    .context("origin HEAD has no branch")?
                    .to_owned()
            }
        }
    };
    let name = root
        .file_name()
        .context("repository has no directory name")?
        .to_string_lossy()
        .into_owned();
    Ok(DiscoveredRepository {
        name,
        path: root,
        common_dir: common,
        base_branch: branch,
    })
}
fn git(path: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()?;
    if !output.status.success() {
        bail!(
            "git discovery failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}
