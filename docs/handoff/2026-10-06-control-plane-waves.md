# Hand-over: control-plane waves 3 to 5, 2026-10-06

Written at the operator's wrap-up of the 2026-10-06 session. The operator's approval for waves 3 to 5 is approval-record:waves-3-to-5-2026-10-06; wave 3 is done, wave 4 stopped mid-wave, wave 5 not started.

## Where things are

| what | state |
|---|---|
| origin/control-plane/bootstrap | e77e533: waves 1 to 3. CI runs 37526316203 and 37526310767 on e77e533 completed success |
| pull request #1 (bootstrap into main) | open, not merged |
| control-plane/wave-4 (local only, never pushed) | e88ee4f opening, 4342ba7 acceptance rephrase, then the wave-4 store-record commit and this hand-over. No unit merged into it yet |
| control-plane/impl/publication-exit (local only) | 17cce0b, 2c810ff. Adversary pass 1 done and fixed; pass 2 not run |
| control-plane/impl/console-status-and-attention (local only) | 40b04a2, 343561b. Adversary passes 1 and 2 done; pass 2's 6 findings are open |
| control-plane/wave-3 (local) | merged into origin/control-plane/bootstrap; safe to delete with `git branch -d` |

## Worktrees (`$HOME/.local/state/worktree/trees/b10x/control-plane/`)

| id | state |
|---|---|
| cp-plan-waves-3-5 | wave-4 integration tree on control-plane/wave-4; finished with archive |
| cp-wave4-publication-exit | finished with archive (unit branch is local only) |
| cp-wave4-console-status-and-attention | kept: holds the untracked adversary file `frontend/src/status.pass2.attack.test.js` (6 red cases) |

Archives are under `$HOME/.local/state/worktree/archives/control-plane/<id>/`. The unit and wave branches stay in the repository's refs.

## Next steps, in order

1. story:publication-exit: dispatch adversary pass 2 (the last) against 2c810ff, base e88ee4f; brief at `$HOME/.cache/cp-wave4/publication-exit/brief.md`. Then the coordinator verifies the correction and merges with `git merge --no-ff --no-commit` plus a bot commit (the bot route refuses `merge`). The adversary file `candidate_landing_after_its_close_still_reconciles` now lands the candidate inside the grace window; its name and doc still say "after its close" and should be renamed.
2. story:console-status-and-attention: correction round 2 for review-result:adversary-console-status-and-attention-pass-2 (6 findings, blocker: every completing goal shows "Stalled" while its acceptance checks run). Same implementor brief `$HOME/.cache/cp-wave4/console-status-and-attention/brief.md`; the coordinator verifies, no third attack. Then merge.
3. Wave-4 gate: `task check` steps one by one on control-plane/wave-4 with ESS 0.53.0 first on PATH (`$HOME/.cache/ess/toolchains/0.53.0`; the machine's `ess` is 0.54.0 and the repository and CI pin 0.53.0). Check that pass 1's F4 (attention ages reset by a blocker re-recorded every tick) is resolved by publication-exit's block() change. Close the stories, push control-plane/wave-4 to control-plane/bootstrap through the bot.
4. Wave 5 (approved): story:spec-owned-admission and story:console-goal-cards. spec-owned-admission still carries an UNMAPPED modelling question (one active change per common Git directory).
5. story:ess-054-upgrade (draft) after wave 5; it needs its own approval.

## Things a next session will trip over

- `restart_open_is_bounded` (5 s bound on Store::open) fails at load average above 40; it passed alone at 2.1 to 2.5 s.
- `<tree>/.scratch` must be a real directory: a symlink there turns the xtask generation_ownership tests red. Runtime fleet tests keep up to 5 GB there.
- `/dev/shm` has a per-user quota that failed at about 26 GB used.
- acceptance-check reads a hyphenated lower-case prose word as a scenario name.

No open dispatches remain from this session. No pull request was opened by it.
