<script setup>
import { reactive, ref } from 'vue'
import { goalFields as fields } from './goalCard.js'
// `inFlight` is how many assignments a save of this goal cancels (goalCard.js `inFlight`);
// `refusal` is the server's answer to this form's last refused save.
const props = defineProps({ goal: { type: Object, default: () => ({}) }, save: Function, inFlight: { type: Number, default: 0 }, refusal: String })
const defaults = { objective: '', acceptance: '', planner_model: 'gpt-5.6-sol', implementor_model: 'gpt-5.6-sol', reviewer_model: 'gpt-5.6-sol', max_workers: 1, max_attempts: 1, max_minutes: 10, merge_authority: false }
const draft = reactive(Object.fromEntries(fields.map(key => [key, props.goal[key] ?? defaults[key]])))
const editingRevision = ref(props.goal.revision)
const saving = ref(false)
async function submit() { saving.value = true; try { if (await props.save({ ...draft })) { editingRevision.value = props.goal.revision; if (!props.goal.goal_id) Object.assign(draft, defaults) } } finally { saving.value = false } }
</script>
<template><form class="goal-form" @submit.prevent="submit">
<p v-if="goal.revision && editingRevision !== goal.revision" class="attention">This goal changed since these fields were opened. Your draft is preserved; saving replaces its editable settings.</p>
<p v-if="goal.goal_id && inFlight > 0" class="attention revision-warning" role="note">Saving makes this goal revision {{ goal.revision + 1 }}: its {{ inFlight }} in-flight {{ inFlight === 1 ? 'assignment' : 'assignments' }} will be cancelled, including a change to merge authority alone.</p>
<label>Objective<input name="objective" v-model="draft.objective" required></label>
<label>Acceptance<textarea name="acceptance" v-model="draft.acceptance" required></textarea></label>
<div class="grid"><label v-for="[key,label] in [['planner_model','Planner model'],['implementor_model','Implementation model'],['reviewer_model','Review model']]" :key="key">{{ label }}<input :name="key" v-model="draft[key]" required></label>
<label v-for="[key,label] in [['max_workers','Workers'],['max_attempts','Attempts per assignment'],['max_minutes','Minutes per phase']]" :key="key">{{ label }}<input type="number" min="1" :name="key" v-model.number="draft[key]" required></label></div>
<label><input type="checkbox" name="merge_authority" v-model="draft.merge_authority">Allow verified merges</label>
<button :disabled="saving">{{ saving ? 'Saving…' : goal.goal_id ? 'Save goal' : 'Create paused goal' }}</button>
<p v-if="refusal" class="refusal" role="alert">{{ refusal }}</p></form></template>
