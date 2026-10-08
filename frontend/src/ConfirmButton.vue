<script setup>
import { ref } from 'vue'
// A control that asks before it acts. `message` names the subject and states the consequence;
// nothing is emitted until the operator confirms. With `needed` false it acts at once.
const props = defineProps({ label: String, confirm: String, message: String, needed: { type: Boolean, default: true }, disabled: Boolean, secondary: Boolean })
const emit = defineEmits(['confirm'])
const asking = ref(false)
function ask() { if (props.needed) asking.value = true; else emit('confirm') }
function accept() { asking.value = false; emit('confirm') }
</script>
<template><span class="confirm-control"><button type="button" :class="{ secondary }" :disabled="disabled" :aria-expanded="needed ? asking : null" @click="ask">{{ label }}</button><div v-if="asking" class="confirmation" role="alertdialog" :aria-label="confirm"><p>{{ message }}</p><div class="actions"><button type="button" class="danger" :disabled="disabled" @click="accept">{{ confirm }}</button><button type="button" class="secondary" @click="asking = false">Keep</button></div></div></span></template>
