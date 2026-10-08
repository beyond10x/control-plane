<script setup>
import { computed, ref } from 'vue'
import Activity from './Activity.vue'
import ConfirmButton from './ConfirmButton.vue'
import GoalForm from './GoalForm.vue'
import { deriveGoalState, goalStateBadge, goalStateLabels } from './goalState.js'
import { assignmentStep, currentAssignments, goalFields, inFlight, storyName } from './goalCard.js'
import { ageSince, plainReason, shortGoal } from './status.js'
import { carriesIds, withoutIds } from './identifiers.js'
// One goal: its title (workspace · repository · short goal), its derived state, its controls in
// the header, the step each of its current assignments is on, and its planner's newest entry.
// `run(name, payload, place)` sends a command; a refusal is kept in `refusals[place]`.
const props = defineProps({ goal: Object, assignments: Array, publications: Array, repositories: Array, workspace: String, now: Number, busy: Boolean, refusals: Object, run: Function })
const editing = ref(false)
const derived = computed(() => deriveGoalState(props.goal, props.assignments))
const repository = id => props.repositories?.find(row => row.repository_id === id)?.name || ''
const current = computed(() => currentAssignments(props.goal, props.assignments))
const cancelling = computed(() => inFlight(props.goal, props.assignments, props.publications).length)
const short = computed(() => shortGoal(props.goal))
// The planning repository; before planning has chosen one, the repositories of the current work.
const scope = computed(() => [props.workspace, repository(props.goal.planning_repository) || [...new Set(current.value.map(row => repository(row.repository_id)).filter(Boolean))].join(', ')].filter(Boolean))
const editable = computed(() => ['Paused', 'Running'].includes(props.goal.state))
const place = computed(() => `goal:${props.goal.goal_id}`)
const command = (name, payload = {}) => props.run(name, { goal_id: props.goal.goal_id, ...payload }, place.value)
const toggleMerge = () => command('UpdateGoal', { ...Object.fromEntries(goalFields.map(key => [key, props.goal[key]])), merge_authority: !props.goal.merge_authority })
const cancelled = count => `its ${count} in-flight ${count === 1 ? 'assignment' : 'assignments'} will be cancelled`
const badge = state => ['Implementing', 'Reviewing', 'Merging'].includes(state) ? 'active' : state === 'Blocked' ? 'blocked' : state === 'Merged' ? 'done' : 'neutral'
const active = state => ['Implementing', 'Reviewing', 'Merging'].includes(state)
const step = assignment => assignmentStep(props.goal, assignment)
const words = { Queued: 'Queued · waiting for a worker', ReadyToMerge: 'Ready to merge', Cancelled: 'Cancelled' }
</script>
<template><article class="operation goal-card" :id="`goal-${goal.goal_id}`">
<div class="operation-head card-head"><div><p class="eyebrow">PLANNER · {{ goal.planner_model }}</p><div class="card-title"><span class="card-scope" v-if="scope.length">{{ scope.join(' · ') }} · </span><h3>{{ short }}</h3></div></div><span class="badge" :class="goalStateBadge(derived)">{{ goalStateLabels[derived] }}</span>
<div class="actions card-actions"><button v-if="goal.state==='Paused'" type="button" :disabled="busy" @click="command('StartGoal')">Start</button><button v-if="goal.state==='Running'" type="button" :disabled="busy" @click="command('PauseGoal')">Pause</button><ConfirmButton v-if="editable" label="Cancel" confirm="Cancel goal" secondary :disabled="busy" :message="`Cancel the goal “${short}”? No further work starts for it, and its in-flight assignments are cancelled.`" @confirm="command('CancelGoal')"/><ConfirmButton v-if="goal.state==='Cancelled'" label="Delete goal" confirm="Delete goal" secondary :disabled="busy" :message="`Delete the goal “${short}”? The goal is deleted. This cannot be undone.`" @confirm="command('DeleteGoal')"/><button v-if="editable" type="button" class="secondary" :aria-expanded="editing" @click="editing = !editing">{{ editing ? 'Close editor' : 'Edit' }}</button>
<span class="merge-authority">Merge authority {{ goal.merge_authority ? 'granted' : 'withheld' }}<ConfirmButton v-if="editable" :label="goal.merge_authority ? 'Withhold' : 'Grant'" :confirm="goal.merge_authority ? 'Withhold merge authority' : 'Grant merge authority'" secondary :disabled="busy" :needed="cancelling > 0" :message="`Changing merge authority saves the goal “${short}” as revision ${goal.revision + 1}: ${cancelled(cancelling)}.`" @confirm="toggleMerge"/></span></div></div>
<p v-if="refusals?.[place]" class="refusal" role="alert">{{ refusals[place] }}</p>
<div class="operation-meta"><span>Stage <strong>{{ goal.planning_phase }}</strong></span><span>Revision {{ goal.revision }}</span><strong v-if="goal.acceptance_recorded" class="recorded">Acceptance recorded</strong></div>
<p>{{ withoutIds(goal.planning_reason) }}</p><p v-if="goal.state !== 'Running'" class="muted">{{ goal.state }} · Previous observations retained.</p>
<ul v-if="current.length" class="card-assignments"><li v-for="assignment in current" :key="assignment.assignment_id" :data-assignment="assignment.assignment_id" class="assignment-row"><div class="assignment-head"><strong>{{ storyName(assignment.story_id) }}</strong><span class="muted">{{ repository(assignment.repository_id) }}</span><span class="badge" :class="badge(assignment.state)">{{ assignment.state }}</span></div>
<p class="assignment-step"><template v-if="active(assignment.state)">{{ assignment.state }}<template v-if="step(assignment).label"> · {{ step(assignment).label }}</template><template v-if="ageSince(step(assignment).at, now)"> · started <time :datetime="step(assignment).at" :title="step(assignment).at">{{ ageSince(step(assignment).at, now) }}</time></template></template><template v-else-if="assignment.state==='Merged'">Merged <time v-if="assignment.merged_at" :datetime="assignment.merged_at" :title="assignment.merged_at">{{ ageSince(assignment.merged_at, now) || assignment.merged_at }}</time><template v-else>· merge time not recorded</template></template><template v-else-if="assignment.state==='Blocked'">Blocked · {{ plainReason(withoutIds(assignment.reason)) || 'stopped without a recorded reason' }}</template><template v-else>{{ words[assignment.state] || assignment.state }}</template></p>
<details class="raw"><summary>Identifiers</summary><div class="path">Story {{ assignment.story_id }}<br>Assignment {{ assignment.assignment_id }}<br>Attempt {{ assignment.attempt }}<template v-if="assignment.reason && carriesIds(assignment.reason)"><br>Reason {{ assignment.reason }}</template><template v-if="assignment.candidate"><br>Candidate {{ assignment.candidate }}</template><template v-if="assignment.implementor_run"><br>Implementation run {{ assignment.implementor_run }}</template><template v-if="assignment.reviewer_run"><br>Review run {{ assignment.reviewer_run }}</template><template v-if="assignment.worktree_id"><br>Worktree {{ assignment.worktree_id }}</template></div></details></li></ul>
<Activity :event="goal.planner_activity" :now="now" :current="goal.state==='Running' && goal.planning_phase!=='Blocked' && goal.planner_activity?.goal_revision===goal.revision"/>
<GoalForm v-if="editing && editable" :goal="goal" :in-flight="cancelling" :refusal="refusals?.[`goal-form:${goal.goal_id}`]" :save="draft => run('UpdateGoal', { ...draft, goal_id: goal.goal_id }, `goal-form:${goal.goal_id}`)"/>
<div class="operation-foot"><details class="raw"><summary>Identifiers and paths</summary><div class="path">Goal {{ goal.goal_id }}<template v-if="goal.planning_worktree_path"><br>Planning worktree {{ goal.planning_worktree_path }}</template><template v-if="goal.planning_reason && carriesIds(goal.planning_reason)"><br>Planning reason {{ goal.planning_reason }}</template></div></details><a :href="`/goals/${goal.goal_id}/evidence`" target="_blank" rel="noopener">Inspect evidence ↗</a></div>
</article></template>
