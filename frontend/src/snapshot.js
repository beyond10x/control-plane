// The console payload the server embeds in the page (crates/control-plane-app/src/web.rs
// `console_page`): the projection of the page's stream, with its clock, as
// `<script type="application/json" id="console-snapshot">`. The application mounts from it and
// replaces it with the stream's first frame.

/** The embedded snapshot, or null when the page holds none or it does not parse. */
export function readSnapshot(document) {
  const text = document.getElementById('console-snapshot')?.textContent
  if (!text) return null
  try { return JSON.parse(text) } catch { return null }
}
