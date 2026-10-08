// Recorded text read without its raw identifiers. Runtime reasons and activity details name UUIDs
// (assignments, run contexts such as `reviewer-<uuid>`), commit hashes, plan artifact ids such as
// `story:status-endpoint` (queued by supervisor.rs as `<repository_id>::story:<slug>`), workspace
// file references (`workspace:<directory_id>/<file>`) and absolute worktree paths; the console
// shows them only inside a details disclosure, beside text that reads without them.

const UUID = String.raw`[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}`
// A plan artifact id: a kind and a slug that starts with a letter (`story:status-endpoint`,
// `review-result:x`). Not a location (`lib.rs:12:5`), a port (`localhost:11434`) or a URL
// (`https://…`): the kind follows no word character, dot or slash, and the slug is followed by
// neither `:` nor `/` nor a dotted continuation.
const KIND_SLUG = String.raw`[a-z][a-z-]*:[a-z][a-z0-9._-]*[a-z0-9](?![\w-])(?![:\/]|\.\w)`

/** A plan artifact id as words: `story:status-endpoint` and `<repository_id>::story:status-endpoint` → `Status endpoint`. */
export function storyName(id) {
  const words = String(id ?? '').replace(/^.*::/, '').replace(/^[a-z][a-z-]*:/, '').replace(/[-_]+/g, ' ').trim()
  return words ? words[0].toUpperCase() + words.slice(1) : 'Unnamed story'
}
const named = id => `“${storyName(id).toLowerCase()}”`

// A runtime story reference: a repository UUID, `::`, and the artifact id.
const reference = new RegExp(String.raw`[\w-]*\b${UUID}::(${KIND_SLUG})`, 'gi')
// A workspace file reference's root (`workspace:<directory_id>/`): the file name stays.
const root = new RegExp(String.raw`\b[a-z][a-z-]*:${UUID}\/?`, 'gi')
// A UUID, with any word it is glued to (`reviewer-<uuid>`, `entry-<uuid>`).
const uuid = new RegExp(String.raw`[\w-]*\b${UUID}\b`, 'gi')
// A hexadecimal object name of seven or more digits holding both a digit and a letter.
const hash = /\b(?=[0-9a-f]*[0-9])(?=[0-9a-f]*[a-f])[0-9a-f]{7,64}\b/gi
// An absolute path of two or more segments (`/srv/demo/alpha`); `/status` alone is prose.
const path = /(?<![\w./:-])\/[^\s/,;:()"'`]+(?:\/[^\s,;:()"'`]*)+/g
const artifact = new RegExp(String.raw`(?<![\w.\/:@-])` + KIND_SLUG, 'g')

// A preposition directly before an identifier goes with it (`main at <hash>` → `main`).
const governed = id => new RegExp(String.raw`(?:\s+(?:at|of|from|to|in|on)(?=\s))?\s*` + id.source, id.flags)

/** `text` with raw identifiers removed and plan artifact ids named in words; '' for none. */
export function withoutIds(text) {
  return String(text ?? '')
    .replace(reference, (_, id) => named(id))
    .replace(root, '')
    .replace(governed(uuid), ' ')
    .replace(governed(path), ' ')
    .replace(governed(hash), ' ')
    .replace(artifact, named)
    .replace(/\(\s*\)/g, '')
    .replace(/[ \t]{2,}/g, ' ')
    // No space before closing punctuation; a dot that starts a relative path (`./x`, `.git`) keeps it.
    .replace(/\s+([;,:)]|\.(?![\w\/.]))/g, '$1')
    .replace(/\(\s+/g, '(')
    .trim()
}

/** Whether `text` carries an identifier that {@link withoutIds} removes. */
export const carriesIds = text => withoutIds(text) !== String(text ?? '').trim()
