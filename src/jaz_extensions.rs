use std::{collections::BTreeMap, sync::Arc};

use agent_client_protocol::schema::{AgentNotification, ExtNotification, Meta, SessionId};
use codex_protocol::protocol::{
    AdditionalContextEntry, AdditionalContextKind, Op, ThreadGoalStatus, ThreadGoalUpdatedEvent,
};
use codex_protocol::user_input::UserInput;
use serde_json::{json, value::RawValue};
use tracing::error;

pub(crate) const CONTEXT_COMPACTED_METHOD: &str = "_jaz/context_compacted";
pub(crate) const GOAL_UPDATE_METHOD: &str = "_jaz/session_goal_update";

const JAZ_META_KEY: &str = "jaz";
const GOAL_REQUESTED_META_KEY: &str = "goalRequested";
const GOAL_REQUEST_CONTEXT_KEY: &str = "jaz.goal_request";
const GOAL_REQUEST_CONTEXT: &str = "Use native Codex goal support for this prompt. If no goal is active, create one with the user's request as the objective. Continue according to native Codex goal rules until the goal is complete, blocked, budget-limited, usage-limited, or requires user input.";

pub(crate) fn goal_requested(meta: Option<&Meta>) -> bool {
    meta.and_then(|meta| meta.get(JAZ_META_KEY))
        .and_then(serde_json::Value::as_object)
        .and_then(|jaz| jaz.get(GOAL_REQUESTED_META_KEY))
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
        environments: None,
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
