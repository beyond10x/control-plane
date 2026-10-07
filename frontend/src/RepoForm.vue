<script setup>
import { reactive, ref } from 'vue'
const props = defineProps({ repository: Object, save: Function })
const draft = reactive(Object.fromEntries(['base_branch','test_command','publish_command'].map(key => [key, props.repository[key]])))
const saving = ref(false)
async function submit() { saving.value = true; try { await props.save({ ...draft }) } finally { saving.value = false } }
</script>
<template><form @submit.prevent="submit"><label v-for="[key,label] in [['base_branch','Base branch'],['test_command','Test command'],['publish_command','Publish command']]" :key="key">{{ label }}<input :name="key" v-model="draft[key]" :required="key !== 'publish_command'"></label><button :disabled="saving">Save repository settings</button></form></template>
