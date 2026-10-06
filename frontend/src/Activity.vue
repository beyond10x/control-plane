<script setup>
defineProps({ event: Object, current: Boolean })
</script>
<template><div v-if="event" class="event"><div class="event-top"><strong>{{ event.action }}</strong><span class="badge" :class="current ? 'active' : 'neutral'">{{ current ? event.status : `Recorded ${event.status}` }}</span></div><p>{{ event.detail }}</p><div class="meta">{{ event.role }} · <time :datetime="event.at">{{ event.at || 'Observation time unavailable' }}</time></div><div v-if="event.worktree" class="path">{{ event.worktree }}</div></div></template>
