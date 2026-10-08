// A goal's evidence (`GET /api/goals/{id}/evidence`, dashboard.rs `evidence`) as a summary.
// Every value below is read from that JSON; nothing is inferred beyond it.

/** The JSON path a goal's evidence is read from and downloaded at. */
export const evidencePath = id => `/api/goals/${encodeURIComponent(id)}/evidence`

/** The goal id an evidence console path names, or ''. */
export function evidenceGoal(path) {
  const match = /^\/goals\/([^/]+)\/evidence\/?$/.exec(path || '')
  if (!match) return ''
  // A malformed escape (a bare `%`) is kept as the raw segment rather than blanking the page.
  try { return decodeURIComponent(match[1]) } catch { return match[1] }
}

/** A receipt stored as a JSON string, parsed; an empty or unreadable receipt is null. */
export function receipt(text) {
  if (typeof text !== 'string' || !text.trim()) return null
  try { const value = JSON.parse(text); return value && typeof value === 'object' ? value : null } catch { return null }
}

// Activity the runtime records when it runs repository checks (fleet.rs `progress`): `checks.run`
// carries the candidate and command, `goal.checks` the observed head and command.
const checkActions = new Set(['checks.run', 'goal.checks'])

/** The recorded activity, oldest first as recorded. */
export const activity = evidence => Array.isArray(evidence?.history?.activity) ? evidence.history.activity : []

/** The latest recorded check: its command, the candidate it ran on, and when; or null. */
export function latestCheck(evidence) {
  const checks = activity(evidence).filter(event => checkActions.has(event?.action) && typeof event?.detail?.command === 'string')
  if (!checks.length) return null
  const event = checks.reduce((latest, next) => String(next.at || '') >= String(latest.at || '') ? next : latest)
  return { command: event.detail.command, candidate: event.detail.candidate || event.detail.observed_head || '', at: event.at || '' }
}

/** The text shown when a tested candidate's command has left the bounded history. */
export const commandNotRetained = 'command outside the retained activity (last 64 entries)'

/**
 * Each assignment whose checks passed, from durable assignment state: the candidate is its
 * `test_revision` (ReviewAssignment records it after the checks); the command is the retained
 * `checks.run` for that assignment and candidate, or null once the bounded history (the latest
 * 64 activities) no longer holds it. The evidence JSON carries no repository settings.
 */
export function testedCandidates(evidence) {
  const checks = activity(evidence).filter(event => event?.action === 'checks.run' && typeof event?.detail?.command === 'string')
  return (Array.isArray(evidence?.assignments) ? evidence.assignments : []).filter(assignment => assignment.test_revision).map(assignment => {
    const ran = checks.filter(event => event.assignment_id === assignment.assignment_id && event.detail.candidate === assignment.test_revision).at(-1)
    return { id: assignment.assignment_id, story: assignment.story_id || '', candidate: assignment.test_revision, command: ran ? ran.detail.command : null, at: ran?.at || '' }
  })
}

/** The latest retained goal acceptance check (`goal.checks`), or null. */
export function goalCheck(evidence) {
  const event = activity(evidence).filter(event => event?.action === 'goal.checks' && typeof event?.detail?.command === 'string').at(-1)
  return event ? { command: event.detail.command, candidate: event.detail.observed_head || '', at: event.at || '' } : null
}

/** Each assignment with the reviewer run that approved it and the commit its merge receipt published. */
export function assignments(evidence) {
  return (Array.isArray(evidence?.assignments) ? evidence.assignments : []).map(assignment => {
    // ReadyAssignment records reviewer_run with review_revision; a repair clears the reviewer.
    const approvedBy = assignment.reviewer_run && assignment.review_revision ? assignment.reviewer_run : ''
    const merge = receipt(assignment.merge_receipt)
    return {
      id: assignment.assignment_id,
      story: assignment.story_id || '',
      state: assignment.state || '',
      candidate: assignment.candidate || '',
      approvedBy,
      merge: merge ? { commit: merge.observed_head || '', candidate: merge.candidate || '', target: merge.target || '', at: merge.observed_at || '' } : null,
    }
  })
}

/** The goal's state and whether its acceptance is recorded. */
export function goalSummary(evidence) {
  const goal = evidence?.goal || {}
  return { objective: goal.objective || '', state: goal.state || '', acceptanceRecorded: Boolean(goal.satisfaction_receipt) }
}
