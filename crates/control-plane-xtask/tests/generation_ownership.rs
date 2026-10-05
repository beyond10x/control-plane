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
        ("components.yaml", include_str!("../../../ess/components.yaml")),
        ("domains/host.yaml", include_str!("../../../ess/domains/host.yaml")),
    ] {
        std::fs::write(root.join("ess").join(name), content)?;
    }
    generation::run(root, true)?;
    let user_file = root.join("generated/model/operator-notes.txt");
    std::fs::write(&user_file, "retained operator evidence")?;
    let result = generation::run(root, true);
    assert!(result.is_err(), "unowned extra files must require an explicit ownership decision");
    assert_eq!(std::fs::read_to_string(user_file)?, "retained operator evidence");
    Ok(())
}
