// generated from controlplane v1
// model digest ababf8b26c6dd7e1f8c6cbec3d38a6897ebb4ebb9a1a5296c3980bb81d3f2548
// contract digest 9905ca468d3ef7bea19af021061f1e1629effe3b09efa4c30cf08a552ffd80d2
// do not edit: regenerate with `ess synthesize --layout crate`

//! Static fallback for paths outside a served route table.
use std::io::Write as _;

pub(crate) fn answer(stream: &mut std::net::TcpStream, root: &std::path::Path, request: &crate::server::http::Request) -> std::io::Result<()> {
    if request.method != "GET" && request.method != "HEAD" {
        return crate::server::http::write(stream, &crate::server::http::Response::new(405, "text/plain", "method not allowed"));
    }
    let Some(path) = resolve(root, &request.path) else {
        return crate::server::http::write(stream, &crate::server::http::Response::new(404, "text/plain", "not found"));
    };
    let Ok(bytes) = std::fs::read(&path) else {
        return crate::server::http::write(stream, &crate::server::http::Response::new(404, "text/plain", "not found"));
    };
    let mime = match path.extension().and_then(|value| value.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8", "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8", "json" => "application/json",
        "wasm" => "application/wasm", "svg" => "image/svg+xml", "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg", "ico" => "image/x-icon", _ => "application/octet-stream",
    };
    write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", bytes.len())?;
    if request.method != "HEAD" { stream.write_all(&bytes)?; }
    stream.flush()
}

fn resolve(root: &std::path::Path, raw: &str) -> Option<std::path::PathBuf> {
    let mut bytes = Vec::new();
    let mut input = raw.as_bytes().iter().copied();
    while let Some(byte) = input.next() {
        if byte == b'%' {
            let high = char::from(input.next()?).to_digit(16)?;
            let low = char::from(input.next()?).to_digit(16)?;
            bytes.push(u8::try_from(high * 16 + low).ok()?);
        } else { bytes.push(byte); }
    }
    let decoded = std::str::from_utf8(&bytes).ok()?;
    if decoded.contains('\\') || decoded.contains('\0') { return None; }
    let relative = std::path::Path::new(decoded.strip_prefix('/')?);
    if relative.components().any(|component| !matches!(component, std::path::Component::Normal(_) | std::path::Component::CurDir)) { return None; }
    let mut path = root.join(relative).canonicalize().ok()?;
    if path.is_dir() { path = path.join("index.html").canonicalize().ok()?; }
    (path.starts_with(root) && path.is_file()).then_some(path)
}
