import { createApp } from 'vue'
import App from './App.vue'
import Evidence from './Evidence.vue'
import { evidenceGoal } from './evidence.js'
import { readSnapshot } from './snapshot.js'
import './style.css'
// `/goals/{id}/evidence` is the evidence view; every other console path is the operations app,
// which mounts from the snapshot embedded in its page.
const goalId = evidenceGoal(location.pathname)
createApp(goalId ? Evidence : App, goalId ? { goalId } : { snapshot: readSnapshot(document) }).mount('#app')
