import { createApp } from 'vue'
import App from './App.vue'
import Evidence from './Evidence.vue'
import { evidenceGoal } from './evidence.js'
import './style.css'
// `/goals/{id}/evidence` is the evidence view; every other console path is the operations app.
const goalId = evidenceGoal(location.pathname)
createApp(goalId ? Evidence : App, goalId ? { goalId } : {}).mount('#app')
