use codex_protocol::{
    models::{AgentMessageInputContent, ResponseItem, plaintext_agent_message_content},
    protocol::{AgentStatus, EventMsg, SubAgentActivityKind},
};
use serde_json::{Value, json};

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct ProviderSubagentIdentity {
    pub(crate) name: Option<String>,
    pub(crate) role: Option<String>,
}

pub(crate) fn apply_provider_subagent_identity(
    update: &mut Value,
    identity: &ProviderSubagentIdentity,
) {
    if let Some(name) = &identity.name {
        update["name"] = json!(name);
    }
    if let Some(role) = &identity.role {
        update["role"] = json!(role);
    }
}

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
        EventMsg::SubAgentActivity(event) => {
            let (status, summary) = match event.kind {
                SubAgentActivityKind::Started => ("running", "Spawned"),
                SubAgentActivityKind::Interacted => ("running", "Working"),
                SubAgentActivityKind::Interrupted => ("cancelled", "Interrupted"),
            };
            let mut update = json!({
                "provider": "codex",
                "id": event.agent_path.as_str(),
                "thread_id": event.agent_thread_id.to_string(),
                "name": event.agent_path.name(),
                "task": event.agent_path.name(),
                "status": status,
                "summary": summary,
            });
            let timestamp = match event.kind {
                SubAgentActivityKind::Interrupted => "completed_at_ms",
                _ => "started_at_ms",
            };
            update[timestamp] = json!(event.occurred_at_ms);
            vec![update]
        }
        EventMsg::RawResponseItem(event) => match &event.item {
            ResponseItem::AgentMessage {
                author,
                recipient,
                content,
                ..
            } if is_v2_completion_message(author, recipient, content) => vec![json!({
                "provider": "codex",
                "id": author,
                "name": author.rsplit('/').next().unwrap_or(author),
                "status": "completed",
                "summary": "Completed",
            })],
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

fn is_v2_completion_message(
    author: &str,
    recipient: &str,
    content: &[AgentMessageInputContent],
) -> bool {
    let Some((parent, name)) = author.rsplit_once('/') else {
        return false;
    };
    !name.is_empty()
        && parent == recipient
        && plaintext_agent_message_content(content)
            .is_some_and(|text| text.starts_with("Message Type: FINAL_ANSWER\n"))
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
        AgentPath, ThreadId,
        models::{AgentMessageInputContent, ResponseItem},
        openai_models::ReasoningEffort,
        protocol::{
            CollabAgentRef, CollabAgentSpawnEndEvent, CollabWaitingBeginEvent,
            RawResponseItemEvent, SubAgentActivityEvent, SubAgentActivityKind,
        },
    };

    fn thread_id(value: &str) -> ThreadId {
        ThreadId::from_string(value).unwrap()
    }

    fn agent_message(author: &str, recipient: &str, text: &str) -> EventMsg {
        EventMsg::RawResponseItem(RawResponseItemEvent {
            item: ResponseItem::AgentMessage {
                id: None,
                author: author.to_string(),
                recipient: recipient.to_string(),
                content: vec![AgentMessageInputContent::InputText {
                    text: text.to_string(),
                }],
                internal_chat_message_metadata_passthrough: None,
            },
        })
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

    #[test]
    fn maps_v2_activity_using_agent_path_as_stable_id() {
        let child = thread_id("018f6c22-7b0a-7000-8000-000000000005");
        let event = EventMsg::SubAgentActivity(SubAgentActivityEvent {
            event_id: "activity-1".to_string(),
            occurred_at_ms: 11,
            agent_thread_id: child,
            agent_path: AgentPath::from_string("/root/reviewer".to_string()).unwrap(),
            kind: SubAgentActivityKind::Started,
        });

        let updates = provider_subagent_updates(&event);

        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0]["id"], "/root/reviewer");
        assert_eq!(updates[0]["thread_id"], child.to_string());
        assert_eq!(updates[0]["name"], "reviewer");
        assert_eq!(updates[0]["task"], "reviewer");
        assert_eq!(updates[0]["status"], "running");
        assert_eq!(updates[0]["summary"], "Spawned");
        assert_eq!(updates[0]["started_at_ms"], 11);
    }

    #[test]
    fn applies_resolved_v2_identity_without_losing_task() {
        let mut update = json!({"name": "reviewer", "task": "reviewer"});

        apply_provider_subagent_identity(
            &mut update,
            &ProviderSubagentIdentity {
                name: Some("Newton".to_string()),
                role: Some("reviewer".to_string()),
            },
        );

        assert_eq!(update["name"], "Newton");
        assert_eq!(update["role"], "reviewer");
        assert_eq!(update["task"], "reviewer");
    }

    #[test]
    fn maps_v2_interruption_to_cancelled_completion() {
        let event = EventMsg::SubAgentActivity(SubAgentActivityEvent {
            event_id: "activity-2".to_string(),
            occurred_at_ms: 19,
            agent_thread_id: thread_id("018f6c22-7b0a-7000-8000-000000000006"),
            agent_path: AgentPath::from_string("/root/reviewer".to_string()).unwrap(),
            kind: SubAgentActivityKind::Interrupted,
        });

        let updates = provider_subagent_updates(&event);

        assert_eq!(updates[0]["status"], "cancelled");
        assert_eq!(updates[0]["summary"], "Interrupted");
        assert_eq!(updates[0]["completed_at_ms"], 19);
        assert!(updates[0].get("started_at_ms").is_none());
    }

    #[test]
    fn maps_v2_final_answer_to_completion_for_same_path() {
        let updates = provider_subagent_updates(&agent_message(
            "/root/reviewer",
            "/root",
            "Message Type: FINAL_ANSWER\nTask name: /root\nSender: /root/reviewer\nPayload:\nReview complete",
        ));

        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0]["id"], "/root/reviewer");
        assert_eq!(updates[0]["name"], "reviewer");
        assert_eq!(updates[0]["status"], "completed");
        assert_eq!(updates[0]["summary"], "Completed");
    }

    #[test]
    fn maps_nested_v2_final_answer_to_its_direct_parent() {
        let updates = provider_subagent_updates(&agent_message(
            "/root/reviewer/reader",
            "/root/reviewer",
            "Message Type: FINAL_ANSWER\nTask name: /root/reviewer\nSender: /root/reviewer/reader\nPayload:\nRead complete",
        ));

        assert_eq!(updates[0]["id"], "/root/reviewer/reader");
        assert_eq!(updates[0]["name"], "reader");
        assert_eq!(updates[0]["status"], "completed");
    }

    #[test]
    fn ignores_v2_nonterminal_message_to_parent() {
        let updates = provider_subagent_updates(&agent_message(
            "/root/reviewer",
            "/root",
            "Message Type: MESSAGE\nTask name: /root\nSender: /root/reviewer\nPayload:\nStill working",
        ));

        assert!(updates.is_empty());
    }

    #[test]
    fn ignores_v2_final_answer_not_sent_to_direct_parent() {
        let updates = provider_subagent_updates(&agent_message(
            "/root/reviewer",
            "/root/implementer",
            "Message Type: FINAL_ANSWER\nTask name: /root/implementer\nSender: /root/reviewer\nPayload:\nPeer update",
        ));

        assert!(updates.is_empty());
    }
}
