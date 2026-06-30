use std::{collections::BTreeMap, sync::Arc};

use agent_client_protocol::schema::{AgentNotification, ExtNotification, Meta, SessionId};
use codex_protocol::protocol::{
    AdditionalContextEntry, AdditionalContextKind, Op, ThreadGoalStatus, ThreadGoalUpdatedEvent,
};
use codex_protocol::user_input::UserInput;
use serde_json::{json, value::RawValue};
use tracing::error;

pub(crate) const CONTEXT_COMPACTED_METHOD: &str = "codex/context_compacted";
pub(crate) const GOAL_UPDATE_METHOD: &str = "thread/goal/updated";
pub(crate) const GOAL_CLEAR_METHOD: &str = "thread/goal/cleared";

const CODEX_META_KEY: &str = "codex";
const GOAL_META_KEY: &str = "goal";
const GOAL_REQUESTED_META_KEY: &str = "requested";
const GOAL_REQUEST_CONTEXT_KEY: &str = "codex.goal_request";
const GOAL_REQUEST_CONTEXT: &str = "Goal mode was explicitly requested for this prompt. If no goal is active, call create_goal with a concise objective derived from the user's request before continuing. Continue according to native Codex goal rules until the goal is complete, blocked, budget-limited, usage-limited, or requires user input.";

pub(crate) fn goal_requested(meta: Option<&Meta>) -> bool {
    meta.and_then(|meta| meta.get(CODEX_META_KEY))
        .and_then(serde_json::Value::as_object)
        .and_then(|codex| codex.get(GOAL_META_KEY))
        .and_then(serde_json::Value::as_object)
        .and_then(|goal| goal.get(GOAL_REQUESTED_META_KEY))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

pub(crate) fn goal_request_context() -> (String, AdditionalContextEntry) {
    (
        GOAL_REQUEST_CONTEXT_KEY.to_string(),
        AdditionalContextEntry {
            value: GOAL_REQUEST_CONTEXT.to_string(),
            kind: AdditionalContextKind::Application,
        },
    )
}

pub(crate) fn user_input_op(items: Vec<UserInput>, goal_requested: bool) -> Op {
    let mut additional_context = BTreeMap::new();
    if goal_requested {
        let (key, entry) = goal_request_context();
        additional_context.insert(key, entry);
    }
    Op::UserInput {
        items,
        final_output_json_schema: None,
        responsesapi_client_metadata: None,
        additional_context,
        thread_settings: Default::default(),
    }
}

pub(crate) fn context_compacted_notification(session_id: &SessionId) -> Option<AgentNotification> {
    ext_notification(
        CONTEXT_COMPACTED_METHOD,
        json!({
            "sessionId": session_id.clone(),
            "source": "codex",
            "status": "completed",
            "trigger": "manual",
        }),
    )
}

pub(crate) fn thread_goal_update_notification(
    session_id: &SessionId,
    event: &ThreadGoalUpdatedEvent,
) -> Option<AgentNotification> {
    ext_notification(
        GOAL_UPDATE_METHOD,
        json!({
            "sessionId": session_id.clone(),
            "goal": thread_goal_update_payload(event),
        }),
    )
}

pub(crate) fn thread_goal_clear_notification(session_id: &SessionId) -> Option<AgentNotification> {
    ext_notification(
        GOAL_CLEAR_METHOD,
        json!({
            "sessionId": session_id.clone(),
        }),
    )
}

pub(crate) fn format_thread_goal_update(event: &ThreadGoalUpdatedEvent) -> Option<String> {
    if matches!(event.goal.status, ThreadGoalStatus::Active) {
        return None;
    }

    let status = match event.goal.status {
        ThreadGoalStatus::Paused => "paused",
        ThreadGoalStatus::BudgetLimited => "budget limited",
        ThreadGoalStatus::Blocked => "blocked",
        ThreadGoalStatus::UsageLimited => "usage limited",
        ThreadGoalStatus::Complete => "complete",
        ThreadGoalStatus::Active => unreachable!(),
    };

    let objective = event.goal.objective.trim();
    Some(if objective.contains('\n') {
        format!("Goal updated ({status}):\n{objective}")
    } else {
        format!("Goal updated ({status}): {objective}")
    })
}

fn ext_notification(method: &str, params: serde_json::Value) -> Option<AgentNotification> {
    let Ok(raw) = RawValue::from_string(params.to_string()) else {
        error!("Failed to encode extension notification {method}");
        return None;
    };
    Some(AgentNotification::ExtNotification(ExtNotification::new(
        method,
        Arc::from(raw),
    )))
}

fn thread_goal_update_payload(event: &ThreadGoalUpdatedEvent) -> serde_json::Value {
    let goal = &event.goal;
    json!({
        "threadId": goal.thread_id,
        "objective": goal.objective.as_str(),
        "status": goal.status,
        "tokenBudget": goal.token_budget,
        "tokensUsed": goal.tokens_used,
        "timeUsedSeconds": goal.time_used_seconds,
        "createdAt": goal.created_at,
        "updatedAt": goal.updated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_codex_goal_request_meta() {
        let meta = Meta::from_iter([(
            "codex".to_string(),
            serde_json::json!({
                "goal": {
                    "requested": true,
                    "objective": "  Finish the goal  "
                }
            }),
        )]);

        assert!(goal_requested(Some(&meta)));
    }

    #[test]
    fn ignores_jaz_goal_request_meta() {
        let meta = Meta::from_iter([(
            "jaz".to_string(),
            serde_json::json!({
                "goalRequested": true,
                "goalObjective": "Finish the goal"
            }),
        )]);

        assert!(!goal_requested(Some(&meta)));
    }
}
