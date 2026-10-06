<script setup>
import { computed } from 'vue'
import { ageSince, formatAge } from './status.js'
// `status` comes from systemStatus; `now` is the server clock (null while unknown); `newest` the
// newest recorded activity time; `planner` from plannerLine.
const props = defineProps({ status: Object, connection: String, live: Boolean, now: Number, newest: Number, planner: Object })
const detail = computed(() => { const since = ageSince(props.status.since, props.now); return [props.status.detail, since && `asked ${since}`, props.status.also].filter(Boolean).join(' · ') })
const activity = computed(() => props.newest === null || props.newest === undefined ? 'No activity recorded' : `Last activity ${formatAge(props.now - props.newest)}`)
</script>
<template><header class="status-header" :class="status.state" aria-label="System status">
<p class="status-line"><span class="status-dot" aria-hidden="true"></span><strong id="system-status">{{ status.label }}</strong><span id="system-detail">{{ detail }}</span><span v-if="live && typeof now === 'number'" id="last-activity" title="Newest recorded work entry, measured on the server clock. Connection keepalives never count as activity.">{{ activity }}</span><span id="connection-status" role="status" :class="{attention:!live}">{{ connection }}</span></p>
<p v-if="live && planner" id="planner-status" class="planner-line"><span class="eyebrow">Planner</span> {{ planner.goal }} · {{ planner.detail }}<template v-if="ageSince(planner.at, now)"> · {{ ageSince(planner.at, now) }}</template></p>
</header></template>
