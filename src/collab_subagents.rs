use codex_protocol::protocol::{AgentStatus, EventMsg};
use serde_json::{Value, json};

pub(crate) fn provider_subagent_updates(event: &EventMsg) -> Vec<Value> {
    match event {
        EventMsg::CollabAgentSpawnEnd(event) => event
            .new_thread_id
            .as_ref()
            .map(|thread_id| {
                json!({
                    "provider": "codex",
                    "id": thread_id.to_string(),
                    "thread_id": thread_id.to_string(),
                    "parent_id": event.sender_thread_id.to_string(),
                    "name": &event.new_agent_nickname,
                    "role": &event.new_agent_role,
                    "status": codex_agent_status(&event.status),
                    "summary": "Spawned",
                    "prompt": &event.prompt,
                    "model": &event.model,
                    "reasoning_effort": format!("{:?}", &event.reasoning_effort),
                    "completed_at_ms": event.completed_at_ms,
                })
            })
            .into_iter()
            .collect(),
        EventMsg::CollabAgentInteractionBegin(event) => vec![json!({
            "provider": "codex",
            "id": event.receiver_thread_id.to_string(),
            "thread_id": event.receiver_thread_id.to_string(),
            "parent_id": event.sender_thread_id.to_string(),
            "status": "running",
            "summary": "Working",
            "prompt": &event.prompt,
            "started_at_ms": event.started_at_ms,
        })],
        EventMsg::CollabAgentInteractionEnd(event) => vec![json!({
            "provider": "codex",
            "id": event.receiver_thread_id.to_string(),
            "thread_id": event.receiver_thread_id.to_string(),
            "parent_id": event.sender_thread_id.to_string(),
            "name": &event.receiver_agent_nickname,
            "role": &event.receiver_agent_role,
            "status": codex_agent_status(&event.status),
            "summary": "Responded",
            "prompt": &event.prompt,
            "completed_at_ms": event.completed_at_ms,
        })],
        EventMsg::CollabWaitingBegin(event) => {
            if event.receiver_agents.is_empty() {
                event
                    .receiver_thread_ids
                    .iter()
                    .map(|thread_id| {
                        json!({
                            "provider": "codex",
                            "id": thread_id.to_string(),
                            "thread_id": thread_id.to_string(),
                            "parent_id": event.sender_thread_id.to_string(),
                            "status": "running",
                            "summary": "Waiting",
                            "started_at_ms": event.started_at_ms,
                        })
                    })
                    .collect()
            } else {
                event
                    .receiver_agents
                    .iter()
                    .map(|agent| {
                        json!({
                            "provider": "codex",
                            "id": agent.thread_id.to_string(),
                            "thread_id": agent.thread_id.to_string(),
                            "parent_id": event.sender_thread_id.to_string(),
                            "name": &agent.agent_nickname,
                            "role": &agent.agent_role,
                            "status": "running",
                            "summary": "Waiting",
                            "started_at_ms": event.started_at_ms,
                        })
                    })
                    .collect()
            }
        }
        EventMsg::CollabWaitingEnd(event) => {
            if event.agent_statuses.is_empty() {
                event
                    .statuses
                    .iter()
                    .map(|(thread_id, status)| {
                        json!({
                            "provider": "codex",
                            "id": thread_id.to_string(),
                            "thread_id": thread_id.to_string(),
                            "parent_id": event.sender_thread_id.to_string(),
                            "status": codex_agent_status(status),
                            "summary": "Wait finished",
                            "completed_at_ms": event.completed_at_ms,
                        })
                    })
                    .collect()
            } else {
                event
                    .agent_statuses
                    .iter()
                    .map(|agent| {
                        json!({
                            "provider": "codex",
                            "id": agent.thread_id.to_string(),
                            "thread_id": agent.thread_id.to_string(),
                            "parent_id": event.sender_thread_id.to_string(),
                            "name": &agent.agent_nickname,
                            "role": &agent.agent_role,
                            "status": codex_agent_status(&agent.status),
                            "summary": "Wait finished",
                            "completed_at_ms": event.completed_at_ms,
                        })
                    })
                    .collect()
            }
        }
        EventMsg::CollabResumeBegin(event) => vec![json!({
            "provider": "codex",
            "id": event.receiver_thread_id.to_string(),
            "thread_id": event.receiver_thread_id.to_string(),
            "parent_id": event.sender_thread_id.to_string(),
            "name": &event.receiver_agent_nickname,
            "role": &event.receiver_agent_role,
            "status": "running",
            "summary": "Resuming",
            "started_at_ms": event.started_at_ms,
        })],
        EventMsg::CollabResumeEnd(event) => vec![json!({
            "provider": "codex",
            "id": event.receiver_thread_id.to_string(),
            "thread_id": event.receiver_thread_id.to_string(),
            "parent_id": event.sender_thread_id.to_string(),
            "name": &event.receiver_agent_nickname,
            "role": &event.receiver_agent_role,
            "status": codex_agent_status(&event.status),
            "summary": "Resumed",
            "completed_at_ms": event.completed_at_ms,
        })],
        EventMsg::CollabCloseBegin(event) => vec![json!({
            "provider": "codex",
            "id": event.receiver_thread_id.to_string(),
            "thread_id": event.receiver_thread_id.to_string(),
            "parent_id": event.sender_thread_id.to_string(),
            "status": "running",
            "summary": "Closing",
            "started_at_ms": event.started_at_ms,
        })],
        EventMsg::CollabCloseEnd(event) => vec![json!({
            "provider": "codex",
            "id": event.receiver_thread_id.to_string(),
            "thread_id": event.receiver_thread_id.to_string(),
            "parent_id": event.sender_thread_id.to_string(),
            "name": &event.receiver_agent_nickname,
            "role": &event.receiver_agent_role,
            "status": "closed",
            "summary": "Closed",
            "completed_at_ms": event.completed_at_ms,
        })],
        _ => Vec::new(),
    }
}

fn codex_agent_status(status: &AgentStatus) -> &'static str {
    match status {
        AgentStatus::PendingInit => "starting",
        AgentStatus::Running => "running",
        AgentStatus::Interrupted => "interrupted",
        AgentStatus::Completed(_) => "completed",
        AgentStatus::Errored(_) => "failed",
        AgentStatus::Shutdown => "closed",
        AgentStatus::NotFound => "not_found",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::{
        ThreadId,
        openai_models::ReasoningEffort,
        protocol::{CollabAgentRef, CollabAgentSpawnEndEvent, CollabWaitingBeginEvent},
    };

    fn thread_id(value: &str) -> ThreadId {
        ThreadId::from_string(value).unwrap()
    }

    #[test]
    fn maps_spawn_end_to_provider_subagent() {
        let parent = thread_id("018f6c22-7b0a-7000-8000-000000000001");
        let child = thread_id("018f6c22-7b0a-7000-8000-000000000002");
        let event = EventMsg::CollabAgentSpawnEnd(CollabAgentSpawnEndEvent {
            call_id: "call-1".to_string(),
            completed_at_ms: 42,
            sender_thread_id: parent,
            new_thread_id: Some(child),
            new_agent_nickname: Some("worker".to_string()),
            new_agent_role: Some("reviewer".to_string()),
            prompt: "inspect the leak".to_string(),
            model: "gpt-5.5".to_string(),
            reasoning_effort: ReasoningEffort::High,
            status: AgentStatus::Running,
        });

        let updates = provider_subagent_updates(&event);

        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0]["provider"], "codex");
        assert_eq!(updates[0]["id"], child.to_string());
        assert_eq!(updates[0]["parent_id"], parent.to_string());
        assert_eq!(updates[0]["name"], "worker");
        assert_eq!(updates[0]["role"], "reviewer");
        assert_eq!(updates[0]["status"], "running");
        assert_eq!(updates[0]["summary"], "Spawned");
        assert_eq!(updates[0]["prompt"], "inspect the leak");
        assert_eq!(updates[0]["model"], "gpt-5.5");
        assert_eq!(updates[0]["completed_at_ms"], 42);
    }

    #[test]
    fn maps_waiting_begin_receiver_agents() {
        let parent = thread_id("018f6c22-7b0a-7000-8000-000000000003");
        let child = thread_id("018f6c22-7b0a-7000-8000-000000000004");
        let event = EventMsg::CollabWaitingBegin(CollabWaitingBeginEvent {
            started_at_ms: 7,
            sender_thread_id: parent,
            receiver_thread_ids: Vec::new(),
            receiver_agents: vec![CollabAgentRef {
                thread_id: child,
                agent_nickname: Some("solver".to_string()),
                agent_role: Some("implementation".to_string()),
            }],
            call_id: "wait-1".to_string(),
        });

        let updates = provider_subagent_updates(&event);

        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0]["id"], child.to_string());
        assert_eq!(updates[0]["parent_id"], parent.to_string());
        assert_eq!(updates[0]["name"], "solver");
        assert_eq!(updates[0]["role"], "implementation");
        assert_eq!(updates[0]["status"], "running");
        assert_eq!(updates[0]["summary"], "Waiting");
        assert_eq!(updates[0]["started_at_ms"], 7);
    }
}
