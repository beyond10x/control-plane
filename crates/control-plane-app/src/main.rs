use clap::Parser;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Some(value) = control_plane_app::run(control_plane_app::Cli::parse()).await? {
        println!("{}", serde_json::to_string_pretty(&value)?);
    }
    Ok(())
}
