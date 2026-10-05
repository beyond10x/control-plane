#[path = "../src/generation.rs"]
mod generation;

#[test]
fn regeneration_refuses_unowned_extra_files_without_deleting_them() -> anyhow::Result<()> {
    let scratch = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.scratch/conformance-adversary");
    std::fs::create_dir_all(&scratch)?;
    let fixture = tempfile::tempdir_in(scratch)?;
    let root = fixture.path();
    std::fs::create_dir_all(root.join("ess/domains"))?;
    for (name, content) in [
        ("system.yaml", include_str!("../../../ess/system.yaml")),
        (
            "components.yaml",
            include_str!("../../../ess/components.yaml"),
        ),
        (
            "domains/host.yaml",
            include_str!("../../../ess/domains/host.yaml"),
        ),
    ] {
        std::fs::write(root.join("ess").join(name), content)?;
    }
    generation::run(root, true)?;
    let user_file = root.join("generated/model/operator-notes.txt");
    std::fs::write(&user_file, "retained operator evidence")?;
    let result = generation::run(root, true);
    assert!(
        result.is_err(),
        "unowned extra files must require an explicit ownership decision"
    );
    assert_eq!(
        std::fs::read_to_string(user_file)?,
        "retained operator evidence"
    );
    Ok(())
}

fn fixture() -> anyhow::Result<tempfile::TempDir> {
    let scratch = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.scratch/conformance-adversary");
    std::fs::create_dir_all(&scratch)?;
    let fixture = tempfile::tempdir_in(scratch)?;
    std::fs::create_dir_all(fixture.path().join("ess/domains"))?;
    for (name, content) in [
        ("system.yaml", include_str!("../../../ess/system.yaml")),
        (
            "components.yaml",
            include_str!("../../../ess/components.yaml"),
        ),
        (
            "domains/host.yaml",
            include_str!("../../../ess/domains/host.yaml"),
        ),
    ] {
        std::fs::write(fixture.path().join("ess").join(name), content)?;
    }
    Ok(fixture)
}

#[test]
fn refusal_in_later_output_preserves_all_earlier_output() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let root = fixture.path();
    generation::run(root, true)?;
    let model = root.join("generated/model/Cargo.toml");
    std::fs::write(
        &model,
        "operator edit retained until all destinations pass preflight",
    )?;
    let extra = root.join("generated/api/operator-notes.txt");
    std::fs::write(&extra, "retained API notes")?;
    let suite = std::fs::read(root.join("generated/conformance.json"))?;
    let error = generation::run(root, true).unwrap_err().to_string();
    assert!(error.contains("operator-notes.txt"), "{error}");
    assert_eq!(
        std::fs::read_to_string(model)?,
        "operator edit retained until all destinations pass preflight"
    );
    assert_eq!(std::fs::read_to_string(extra)?, "retained API notes");
    assert_eq!(
        std::fs::read(root.join("generated/conformance.json"))?,
        suite
    );
    Ok(())
}

#[test]
fn generated_root_symlink_is_refused_before_writing_its_target() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let root = fixture.path();
    let target = root.join("operator-owned");
    std::fs::create_dir(&target)?;
    std::os::unix::fs::symlink(&target, root.join("generated"))?;
    let error = generation::run(root, true).unwrap_err().to_string();
    assert!(error.contains("symlink refused"), "{error}");
    assert_eq!(std::fs::read_dir(target)?.count(), 0);
    Ok(())
}
