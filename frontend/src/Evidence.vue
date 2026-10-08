<script setup>
import { computed, onMounted, ref } from 'vue'
import { activitySentence, openDetail } from './activity.js'
import { activity, assignments, evidencePath, goalSummary, latestCheck } from './evidence.js'
// A goal's evidence, summarised from `GET /api/goals/{id}/evidence`; the same JSON is the
// download at the foot of the page.
const props = defineProps({ goalId: String })
const evidence = ref(null), error = ref('')
const path = computed(() => evidencePath(props.goalId))
const goal = computed(() => goalSummary(evidence.value))
const check = computed(() => latestCheck(evidence.value))
const work = computed(() => assignments(evidence.value))
const merged = computed(() => work.value.filter(item => item.merge))
const timeline = computed(() => [...activity(evidence.value)].reverse())
const badge = state => ['Running','Implementing','Reviewing','Merging'].includes(state) ? 'active' : state === 'Blocked' ? 'blocked' : ['Satisfied','Merged'].includes(state) ? 'done' : 'neutral'
onMounted(async () => {
  try {
    const response = await fetch(path.value)
    const body = response.headers.get('content-type')?.includes('application/json') ? await response.json() : null
    if (!response.ok) throw Error(body?.error || `Evidence unavailable (${response.status})`)
    evidence.value = body
  } catch (e) { error.value = e.message }
})
</script>
<template>
<header class="brand"><a href="/"><span class="brand-icon">▦</span> control plane</a><span>ENGINEERING OPERATIONS</span></header>
<main class="evidence">
<div class="page-heading"><div><p class="eyebrow">GOAL EVIDENCE</p><h1>{{ goal.objective || 'Goal evidence' }}</h1><p class="path">Goal {{ goalId }}</p></div><a class="button secondary" href="/">Operations overview</a></div>
<section v-if="error" class="error" role="alert"><strong>Evidence unavailable</strong><p>{{ error }}</p></section>
<div v-else-if="!evidence" class="empty">Loading recorded evidence…</div>
<template v-else>
<section class="panel" data-test="summary"><h2>Goal</h2><p><span class="badge" :class="badge(goal.state)">{{ goal.state }}</span> {{ goal.acceptanceRecorded ? 'Acceptance recorded' : 'No acceptance recorded' }}</p></section>
<section class="panel" data-test="latest-check"><h2>Latest check</h2><template v-if="check"><p><code>{{ check.command }}</code></p><p class="path">Candidate {{ check.candidate || 'not recorded' }}<template v-if="check.at"> · <time :datetime="check.at">{{ check.at }}</time></template></p></template><p v-else class="muted">No check recorded.</p></section>
<section class="panel" data-test="merges"><h2>Assignments</h2>
<p v-if="!merged.length" class="muted">Nothing was merged.</p>
<p v-if="!work.length" class="muted">No assignments recorded.</p>
<article v-for="item in work" :key="item.id" class="operation" data-test="assignment"><div class="operation-head"><h3>{{ item.story }}</h3><span class="badge" :class="badge(item.state)">{{ item.state }}</span></div>
<p class="path">Candidate {{ item.candidate || 'not recorded' }}</p>
<p>{{ item.approvedBy ? 'Approved by reviewer run' : 'No approving review recorded' }} <code v-if="item.approvedBy">{{ item.approvedBy }}</code></p>
<p v-if="item.merge">Published commit <code>{{ item.merge.commit || 'not recorded' }}</code><template v-if="item.merge.target"> on {{ item.merge.target }}</template><template v-if="item.merge.candidate"> · candidate <code>{{ item.merge.candidate }}</code></template></p>
<p v-else class="muted">Not merged.</p></article>
</section>
<section class="panel timeline" data-test="timeline"><h2>Recorded activity</h2><p v-if="!timeline.length" class="muted">No activity recorded.</p><ol v-else><li v-for="(event, index) in timeline" :key="event.id || index" class="timeline-item"><strong>{{ activitySentence(event.action) }}</strong><span v-if="openDetail(event)"> · {{ openDetail(event) }}</span><div class="meta">{{ event.role }}<template v-if="event.at"> · <time :datetime="event.at">{{ event.at }}</time></template></div></li></ol></section>
</template>
<p><a class="download" :href="path" :download="`evidence-${goalId}.json`">Download raw evidence (JSON)</a></p>
</main>
</template>
