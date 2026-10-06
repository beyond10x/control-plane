<script setup>
import { computed, nextTick, onMounted, onUnmounted, reactive, ref } from 'vue'
import GoalForm from './GoalForm.vue'
import RepoForm from './RepoForm.vue'
import Activity from './Activity.vue'
import StatusHeader from './StatusHeader.vue'
import AttentionStrip from './AttentionStrip.vue'
import { deriveGoalState, goalStateBadge, goalStateLabels } from './goalState.js'
import { attentionItems, newestActivity, parseTime, plannerLine, systemStatus } from './status.js'
const empty = () => ({ workspaces: [], directories: [], repositories: [], goals: [], assignments: [], publications: [] })
const data = ref(empty()), catalog = ref([]), workspaceId = ref(location.pathname.match(/^\/workspaces\/([^/]+)/)?.[1] || '')
const connection = ref('Connecting…'), error = ref(''), notice = ref(''), loaded = ref(false), busy = ref(false)
// `link` is the stream's state: `connecting` until a frame arrives, `live` while frames apply,
// `disconnected` after an error. Counts and ages are only shown while live.
const link = ref('connecting'), live = computed(() => link.value === 'live')
// The server clock: the frame's `server_time`, advanced by the browser's elapsed time since.
const clock = ref(Date.now()), offset = ref(null), now = computed(() => offset.value === null ? null : clock.value + offset.value)
const workspaceDraft = reactive({ name: '', path: '' }), directory = ref(''), repository = ref('')
let stream, csrf, ticker, navigation = 0, anchor = location.hash.startsWith('#goal-') ? location.hash.slice(1) : ''
const workspace = computed(() => catalog.value.find(w => w.workspace_id === workspaceId.value))
const goals = computed(() => data.value.goals || [])
const assignments = computed(() => data.value.assignments || [])
const currentAssignments = computed(() => assignments.value.filter(a => goals.value.some(g => g.goal_id === a.goal_id && g.state === 'Running' && g.revision === a.goal_revision)))
const attention = computed(() => attentionItems(data.value))
const status = computed(() => systemStatus(data.value, link.value))
const newest = computed(() => newestActivity(data.value)), planner = computed(() => plannerLine(data.value))
const metrics = computed(() => [[goals.value.filter(g => g.state === 'Running').length, 'Running goals'],[currentAssignments.value.filter(a => ['Implementing','Reviewing','Merging'].includes(a.state)).length, 'Active workers'],[currentAssignments.value.filter(a => ['Queued','ReadyToMerge'].includes(a.state)).length, 'Queued assignments'],[attention.value.length, 'Need attention']])
const timeline = computed(() => goals.value.flatMap(g => (g.activity || []).map(e => ({ ...e, goal: g.objective, id: `${g.goal_id}:${e.at}:${e.action}:${e.role}` }))).sort((a,b) => b.at.localeCompare(a.at)).slice(0,24))
const badge = state => ['Running','Implementing','Reviewing','Merging','Planning','Testing'].includes(state) ? 'active' : state === 'Blocked' ? 'blocked' : ['Satisfied','Merged'].includes(state) ? 'done' : 'neutral'
function apply(view) { data.value = view; if (view.workspaces) catalog.value = view.workspaces; else if (view.workspace) catalog.value = catalog.value.map(w => w.workspace_id === view.workspace.workspace_id ? view.workspace : w); loaded.value = true; const server = parseTime(view.server_time); clock.value = Date.now(); offset.value = server === null ? null : server - clock.value; if (anchor) nextTick(reveal) }
function reveal() { const target = anchor && document.getElementById(anchor); if (target) { target.scrollIntoView?.({ block: 'start' }); anchor = '' } }
async function session() { const response = await fetch('/api/session'); if (!response.ok) throw Error('Cannot establish local console session'); csrf = (await response.json()).csrf_token }
async function connect() {
  const generation = ++navigation
  stream?.close(); connection.value = 'Connecting…'; link.value = 'connecting'; loaded.value = false; data.value = empty()
  try { const response = await fetch('/api/workspaces'); if (!response.ok) throw Error('Workspace list unavailable'); const value = await response.json(); if (generation !== navigation) return; catalog.value = value.workspaces }
  catch (e) { if (generation === navigation) error.value = e.message }
  if (generation !== navigation) return
  stream = new EventSource(workspaceId.value ? `/workspaces/${encodeURIComponent(workspaceId.value)}/events` : '/events')
  stream.addEventListener('operations', event => { if (generation !== navigation) return; try { apply(JSON.parse(event.data)); connection.value = 'Live · committed observations'; link.value = 'live' } catch { connection.value = 'State unavailable · reconnecting'; link.value = 'disconnected'; stream.close(); setTimeout(() => { if (generation === navigation) connect() },1000) } })
  stream.addEventListener('open', () => { if (generation === navigation) { connection.value = 'Connected · waiting for state'; link.value = 'connecting' } })
  stream.addEventListener('error', () => { if (generation === navigation) { connection.value = 'Disconnected · reconnecting; displayed observations may be stale'; link.value = 'disconnected' } })
  stream.addEventListener('unavailable', event => { if (generation === navigation) { connection.value = event.data; link.value = 'disconnected' } })
}
function navigate(id, path) { if (workspaceId.value === id) return; workspaceId.value = id; history.pushState({},'', path || (id ? `/workspaces/${id}` : '/')); error.value = ''; notice.value = ''; connect() }
function openControls(item) { anchor = item.anchor; if (workspaceId.value === item.workspace_id) { history.replaceState({}, '', item.href); reveal() } else navigate(item.workspace_id, item.href) }
function pop() { workspaceId.value = location.pathname.match(/^\/workspaces\/([^/]+)/)?.[1] || ''; connect() }
async function mutate(path, payload, method = 'POST', form = false) {
  error.value = ''; notice.value = ''; busy.value = true
  try {
    if (!csrf) await session()
    const headers = form ? { 'content-type':'application/x-www-form-urlencoded' } : { 'content-type':'application/json', 'x-csrf-token':csrf }
    const body = form ? new URLSearchParams({ ...payload, csrf }) : JSON.stringify(payload)
    const response = await fetch(path, { method, headers, body })
    if (!response.ok) { let detail = response.headers.get('content-type')?.includes('application/json') ? (await response.json()).error : `Request refused (${response.status})`; if (response.status === 403) { await session(); detail += '. Session refreshed; submit again.' } throw Error(detail) }
    notice.value = 'Change saved. Live state updates after its durable commit.'
    return response.headers.get('content-type')?.includes('application/json') ? await response.json() : true
  } catch (e) { error.value = e.message; return false } finally { busy.value = false }
}
const command = (name,payload) => mutate(`/api/commands/${name}`,payload)
async function addWorkspace() { const result = await mutate('/api/workspaces',{ ...workspaceDraft }); if (result) { const id = result.published?.[0]?.payload?.workspace_id; workspaceDraft.name = ''; workspaceDraft.path = ''; if (id) navigate(id) } }
async function addDirectory() { if (await mutate(`/api/workspaces/${workspaceId.value}/directories`,{ path:directory.value })) directory.value = '' }
async function addRepository() { if (await mutate(`/workspaces/${workspaceId.value}/repositories`,{ path:repository.value },'POST',true)) repository.value = '' }
function workers(goal) { return currentAssignments.value.filter(a => a.goal_id === goal.goal_id && goal.fleet?.[a.assignment_id]?.goal_revision === goal.revision) }
onMounted(() => { connect(); session().catch(e => error.value = e.message); addEventListener('popstate',pop); ticker = setInterval(() => { clock.value = Date.now() }, 1000) })
onUnmounted(() => { ++navigation; stream?.close(); removeEventListener('popstate',pop); clearInterval(ticker) })
</script>
<template>
<header class="brand"><a href="/" @click.prevent="navigate('')"><span class="brand-icon">▦</span> control plane</a><span>ENGINEERING OPERATIONS</span></header>
<nav class="sidebar" aria-label="Workspaces"><a class="overview" href="/" @click.prevent="navigate('')">◈ Operations overview</a><p class="nav-label">WORKSPACES</p><a v-for="w in catalog" :key="w.workspace_id" class="workspace-link" :class="{selected:workspaceId===w.workspace_id}" :href="`/workspaces/${w.workspace_id}`" @click.prevent="navigate(w.workspace_id)"><span class="nav-dot"></span>{{ w.name }}</a><div class="sidebar-footer">Local & governed<br>Evidence stays with the work.</div></nav>
<main><StatusHeader :status="status" :connection="connection" :live="live" :now="now" :newest="newest" :planner="planner"/><AttentionStrip :items="attention" :now="now" :live="live" @open="openControls"/><div class="page-heading"><div><p class="eyebrow">{{ workspaceId ? 'WORKSPACE OPERATIONS' : 'ALL WORKSPACES' }}</p><h1>{{ workspace?.name || (workspaceId ? 'Workspace' : 'Operations overview') }}</h1><p class="path">{{ workspace?.path }}</p></div><a v-if="workspaceId" class="button secondary" href="#goal-controls">Goal controls ↓</a></div>
<section v-if="error" class="error" role="alert"><strong>Request needs attention</strong><p>{{ error }}</p></section><p v-if="notice" role="status">{{ notice }}</p>
<section v-if="data.runtime_error" class="error"><h2>Autonomous processing stopped</h2><p>{{ data.runtime_error }}</p></section>
<div v-if="!loaded" class="empty">{{ workspaceId && catalog.length && !workspace ? 'Workspace not found.' : 'Waiting for current committed state…' }}</div>
<div class="metrics"><div class="metric" v-for="[number,label] in metrics" :key="label"><span>{{ label }}</span><strong>{{ live ? number : '—' }}</strong><small>Committed state</small></div></div>
<div class="section-heading"><h2>Planner operations</h2><span class="muted">Live observations</span></div>
<div v-if="loaded && !goals.length" class="empty">No goals recorded. Create a goal below to begin.</div>
<template v-for="goal in goals" :key="goal.goal_id"><article class="operation"><div class="operation-head"><div><p class="eyebrow">PLANNER · {{ goal.planner_model }}</p><h3>{{ goal.objective }}</h3></div><span class="badge" :class="goalStateBadge(deriveGoalState(goal))">{{ goalStateLabels[deriveGoalState(goal)] }}</span></div><div class="operation-meta"><span>Stage <strong>{{ goal.planning_phase }}</strong></span><span>Revision {{ goal.revision }}</span><span>Merge authority {{ goal.merge_authority ? 'Granted' : 'Withheld' }}</span><strong v-if="goal.acceptance_recorded" class="recorded">Acceptance recorded</strong></div><p>{{ goal.planning_reason }}</p><p v-if="goal.state !== 'Running'" class="muted">{{ goal.state }} · Previous observations retained.</p><Activity :event="goal.planner_activity" :current="goal.state==='Running' && goal.planning_phase!=='Blocked' && goal.planner_activity?.goal_revision===goal.revision"/><div class="operation-foot"><span class="path">{{ goal.planning_worktree_path }}</span><a :href="`/goals/${goal.goal_id}/evidence`" target="_blank" rel="noopener">Inspect evidence ↗</a></div></article><article v-for="assignment in workers(goal)" :key="assignment.assignment_id" class="operation worker"><div class="operation-head"><h3>Worker · {{ assignment.story_id }}</h3><span class="badge" :class="badge(assignment.state)">{{ assignment.state }}</span></div><Activity :event="goal.fleet[assignment.assignment_id]" :current="['Implementing','Reviewing','Merging'].includes(assignment.state)"/></article></template>
<div class="section-heading"><h2>Assignment queue</h2></div><div class="panel table-wrap"><table><thead><tr><th>Assignment / repository</th><th>Stage</th><th>Attempt</th><th>Current reason</th></tr></thead><tbody><tr v-for="a in assignments" :key="a.assignment_id"><td><strong>{{ a.story_id }}</strong><small>{{ data.repositories?.find(r=>r.repository_id===a.repository_id)?.name }}</small><details><summary>Execution details</summary><div class="path">Candidate: {{ a.candidate }}<br>Implementation: {{ a.implementor_run }}<br>Review: {{ a.reviewer_run }}</div><p v-for="p in data.publications?.filter(p=>p.assignment_id===a.assignment_id)" :key="p.publication_id">Publication: {{ p.state }}</p></details></td><td><span class="badge" :class="badge(a.state)">{{ a.state }}</span></td><td>{{ a.attempt }}</td><td>{{ a.reason }}</td></tr><tr v-if="loaded && !assignments.length"><td colspan="4">No assignments recorded. An empty queue does not establish acceptance.</td></tr></tbody></table></div>
<div class="section-heading"><h2>Activity history</h2><span class="muted">Latest 24 recorded events</span></div><section class="panel timeline"><p v-if="loaded && !timeline.length" class="muted">No timed observations recorded yet.</p><div class="timeline-item" v-for="event in timeline" :key="event.id"><span class="eyebrow">{{ event.goal }}</span><Activity :event="event" :current="false"/></div></section>
<template v-if="workspaceId && workspace"><details class="settings"><summary>Workspace settings · Directories</summary><article v-for="d in data.directories?.filter(d=>d.state==='Registered')" :key="d.directory_id"><p class="path">{{ d.path }}</p><button class="secondary" :disabled="busy" @click="mutate(`/api/workspaces/${workspaceId}/directories/${d.directory_id}`,{},'DELETE')">Remove directory</button></article><form @submit.prevent="addDirectory"><label>Directory path<input name="directory_path" v-model="directory" required></label><button :disabled="busy">Add directory</button></form></details>
<div id="goal-controls" class="section-heading"><h2>Goal controls</h2><span class="muted">Unsaved edits stay while live state updates</span></div><article v-for="goal in goals" :id="`goal-${goal.goal_id}`" :key="goal.goal_id"><h3>{{ goal.objective }}</h3><p>{{ goal.acceptance }}</p><div class="actions"><button v-if="goal.state==='Paused'" :disabled="busy" @click="command('StartGoal',{goal_id:goal.goal_id})">Start</button><button v-if="goal.state==='Running'" :disabled="busy" @click="command('PauseGoal',{goal_id:goal.goal_id})">Pause</button><button v-if="['Paused','Running'].includes(goal.state)" class="secondary" :disabled="busy" @click="command('CancelGoal',{goal_id:goal.goal_id})">Cancel</button><button v-if="goal.state==='Cancelled'" class="secondary" :disabled="busy" @click="command('DeleteGoal',{goal_id:goal.goal_id})">Delete goal</button></div><details v-if="['Paused','Running'].includes(goal.state)"><summary>Edit goal and limits</summary><GoalForm :goal="goal" :save="draft=>command('UpdateGoal',{...draft,goal_id:goal.goal_id})"/></details></article>
<details class="settings"><summary>Create a new goal</summary><GoalForm :key="workspaceId" :save="draft=>command('CreateGoal',{...draft,workspace_id:workspaceId})"/></details>
<details class="settings"><summary>Repository settings</summary><section v-for="r in data.repositories" :key="r.repository_id"><h3>{{ r.name }}</h3><p class="path">{{ r.path }}</p><span class="badge">{{ r.state }}</span><RepoForm :repository="r" :save="draft=>command('ConfigureRepository',{...draft,repository_id:r.repository_id})"/><button class="secondary" :disabled="busy" @click="command(r.state==='Registered'?'DisableRepositoryRegistration':'EnableRepositoryRegistration',{repository_id:r.repository_id})">{{ r.state==='Registered' ? 'Disable' : 'Enable' }}</button></section><form @submit.prevent="addRepository"><label>Repository directory<input v-model="repository" required></label><button :disabled="busy">Add repository</button></form></details></template>
<details class="settings"><summary>Manage workspaces · Add workspace</summary><div class="grid"><article v-for="w in catalog" :key="w.workspace_id"><a :href="`/workspaces/${w.workspace_id}`" @click.prevent="navigate(w.workspace_id)">{{ w.name }}</a><p class="path">{{ w.path }}</p><span class="badge">{{ w.state }}</span></article></div><form @submit.prevent="addWorkspace"><label>Name<input name="workspace_name" v-model="workspaceDraft.name" required></label><label>Directory<input name="workspace_path" v-model="workspaceDraft.path" required></label><button :disabled="busy">Add workspace</button></form></details>
</main></template>
