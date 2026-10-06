<script setup>
import { ageSince } from './status.js'
// `items` come from attentionItems; `now` is the server clock (null while unknown).
defineProps({ items: Array, now: Number, live: Boolean })
const emit = defineEmits(['open'])
// A plain click on a goal-controls link stays in the console; a modified click opens normally.
function follow(event, item) { if (item.external || event.button || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return; event.preventDefault(); emit('open', item) }
</script>
<template><section v-if="items.length" class="attention-strip" aria-label="Needs your attention">
<h2>Needs your attention · {{ items.length }}<span v-if="!live" class="muted"> as of the last update</span></h2>
<ul><li v-for="item in items" :key="item.key" class="attention-row"><span class="badge blocked">{{ item.kind }}</span><span class="attention-subject">{{ item.goal }}<template v-if="item.repository"> · {{ item.repository }}</template></span><span class="attention-reason">{{ item.reason }}</span><span class="attention-age">{{ ageSince(item.at, now) || 'Age unknown' }}</span><a v-if="item.href" class="resolve" :href="item.href" :target="item.external ? '_blank' : null" :rel="item.external ? 'noopener' : null" @click="follow($event, item)">{{ item.control }}</a></li></ul>
</section><p v-else-if="live" class="attention-clear">Nothing needs your attention.</p></template>
