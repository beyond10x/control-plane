//! The console pages, the evidence JSON and the activity history the projections read.
use super::*;

pub async fn live(State(state): State<AppState>) -> Response {
    web::home(State(state)).await
}
pub async fn workspace_live(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    web::workspace(State(state), Path(id)).await
}
pub async fn evidence(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let result = async {
        let view = state.snapshot().await?;
        let goal = rows(&view, "goals")?
            .iter()
            .find(|g| field(g, "goal_id") == id)
            .context("goal not found")?;
        let assignments: Vec<_> = rows(&view, "assignments")?
            .iter()
            .filter(|a| field(a, "goal_id") == id)
            .collect();
        let publications: Vec<_> = rows(&view, "publications")?
            .iter()
            .filter(|p| {
                assignments
                    .iter()
                    .any(|a| a["assignment_id"] == p["assignment_id"])
            })
            .collect();
        // The snapshot attached the history under the same store lock as the goal's receipt,
        // so both describe one committed state.
        let history = &goal["activity_history"];
        Ok(json!({"goal":goal,"history":history,"assignments":assignments,"publications":publications}))
    }
    .await;
    api_answer(result)
}

/// Add each goal's progress history (`Store::activity_history`) to view rows as
/// `activity_history`. Bounded receipts no longer carry it; readers take it from here.
pub(crate) fn attach_history(store: &Store, goals: &mut Value) -> Result<()> {
    for goal in goals.as_array_mut().into_iter().flatten() {
        if !goal["activity_history"].is_object() {
            goal["activity_history"] = store.activity_history(field(goal, "goal_id"))?;
        }
    }
    Ok(())
}
