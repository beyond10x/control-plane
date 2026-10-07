// generated from controlplane v1
// model digest 897a3414c77f9f4e1cf2364f3618ac52b1f26e3c84b3ff542878e9e78509dbd7
// contract digest b6fee6bf66c237bc1382d9b6b607570b4f096d13d3d91c101d1efb3fde6bb9b7
// do not edit: regenerate with `ess synthesize --layout crate`

//! Ephemeral generated server. Durable storage and production authentication remain ports.
use clap::Parser;

#[derive(Parser)]
struct Options {
#[arg(long, default_value = "127.0.0.1:8080")]
listen: String,
#[arg(long, default_value = "none", value_parser = ["none", "actor-header"])]
callers: String,
#[arg(long = "static")]
static_root: Option<std::path::PathBuf>,
}

fn main() -> std::process::ExitCode {
let options = Options::parse();
let ports = controlplane::server::memory::MemoryPorts::default();
let mut system = controlplane::system::System::new(controlplane::ports::control_plane::ControlPlane::new(controlplane::behaviour::Generated::new(ports.clone())));
let mode = options.callers;
eprintln!("{}", if mode == "actor-header" { r#"{"format":"ess/1","callers":"actor-header","demonstration":true}"# } else { r#"{"format":"ess/1","callers":"none"}"# });
match controlplane::server::control_plane::serve_with_static(&mut system, &options.listen, |request| { if mode != "actor-header" { return None; } let mut authorization = request.headers.iter().filter(|(key, _)| key.eq_ignore_ascii_case("Authorization")); let header = authorization.next()?.1.strip_prefix("Actor ")?; if authorization.next().is_some() { return None; } let actor = match header { "Operator" => "controlplane.host.Operator",
"Supervisor" => "controlplane.host.Supervisor",
 other => other }; controlplane::actor::Actor::ALL.iter().find(|candidate| candidate.name() == actor).map(|actor| controlplane::actor::Caller { actor: *actor }) }, options.static_root.as_deref()) { Ok(()) => std::process::ExitCode::SUCCESS, Err(error) => { eprintln!("{error}"); std::process::ExitCode::FAILURE } }
}
