<script setup>
import { reactive, ref } from 'vue'
// `stopped` is how many assignments a save of this repository's settings stops (goalCard.js
// `stoppedBySettings`), `planning` how many running planners it stops (`planningIn`); `refusal`
// is the server's answer to this form's last refused save.
const props = defineProps({ repository: Object, save: Function, stopped: { type: Number, default: 0 }, planning: { type: Number, default: 0 }, refusal: String })
const draft = reactive(Object.fromEntries(['base_branch','test_command','publish_command'].map(key => [key, props.repository[key]])))
const saving = ref(false)
async function submit() { saving.value = true; try { await props.save({ ...draft }) } finally { saving.value = false } }
</script>
<template><form class="repository-form" @submit.prevent="submit"><p v-if="stopped > 0 || planning > 0" class="attention repository-warning" role="note">Saving changes the settings of {{ repository.name }}.<template v-if="stopped > 0"> Its {{ stopped }} in-flight {{ stopped === 1 ? 'assignment is' : 'assignments are' }} stopped and blocked, because the repository configuration changed after {{ stopped === 1 ? 'it was' : 'they were' }} claimed, and can then only be cancelled or re-planned.</template><template v-if="planning > 0"> A running planner for this workspace in {{ repository.name }} is stopped.</template></p><label v-for="[key,label] in [['base_branch','Base branch'],['test_command','Test command'],['publish_command','Publish command']]" :key="key">{{ label }}<input :name="key" v-model="draft[key]" :required="key !== 'publish_command'"></label><button :disabled="saving">Save repository settings</button><p v-if="refusal" class="refusal" role="alert">{{ refusal }}</p></form></template>
