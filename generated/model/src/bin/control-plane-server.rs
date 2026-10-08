// generated from controlplane v1
// model digest c4dda5ccc49fd738a60e886dbf7d9aa1b3aa7b406ec8f453127c63b9f6583548
// contract digest e2cc170afad569e615d682e77d7a36681653dc02f5870ba785e69e9b36846f10
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
