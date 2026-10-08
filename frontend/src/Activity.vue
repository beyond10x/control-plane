<script setup>
import { computed } from 'vue'
import { activitySentence, openDetail } from './activity.js'
import { ageSince } from './status.js'
// One recorded entry as a sentence with its age on the server clock `now` (null while unknown).
// The detail is shown in the open without its identifiers (activity.js `openDetail`); the action
// id, the recorded time, the worktree and the detail as recorded sit behind the details disclosure.
const props = defineProps({ event: Object, current: Boolean, now: Number })
const open = computed(() => openDetail(props.event))
const recorded = computed(() => String(props.event?.detail ?? '').trim())
const age = computed(() => ageSince(props.event?.at, props.now))
</script>
<template><div v-if="event" class="event"><div class="event-top"><strong>{{ activitySentence(event.action) }}</strong><span class="badge" :class="current ? 'active' : 'neutral'">{{ current ? event.status : `Recorded ${event.status}` }}</span></div><p v-if="open">{{ open }}</p><div class="meta">{{ event.role }} · <time :datetime="event.at" :title="event.at">{{ age || (event.at ? 'time unknown' : 'Observation time unavailable') }}</time></div><details class="raw"><summary>Details</summary><div class="path">Action {{ event.action }}<template v-if="event.at"><br>Recorded {{ event.at }}</template><template v-if="recorded && recorded !== open"><br>{{ recorded }}</template><template v-if="event.worktree"><br>Worktree {{ event.worktree }}</template></div></details></div></template>
