use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

use agent_client_protocol::schema::SessionId;
use codex_core::CodexThread;
use codex_extension_api::ExtensionEventSink;
use codex_features::Feature;
use codex_goal_extension::GoalService;
use codex_protocol::{
    ThreadId,
    error::CodexErr,
    protocol::{Event, EventMsg},
};

use crate::thread::Thread;

#[derive(Clone)]
pub(crate) struct NativeGoal {
    service: Arc<GoalService>,
    thread_id: ThreadId,
}

impl NativeGoal {
    pub(crate) fn new(service: Arc<GoalService>, thread_id: ThreadId) -> Self {
        Self { service, thread_id }
    }

    pub(crate) async fn clear(
        &self,
        thread: &(impl NativeGoalThread + ?Sized),
    ) -> Result<bool, CodexErr> {
        thread
            .clear_native_goal(Arc::clone(&self.service), self.thread_id)
            .await
    }
}

pub(crate) trait NativeGoalThread: Send + Sync {
    fn clear_native_goal(
        &self,
        goal_service: Arc<GoalService>,
        thread_id: ThreadId,
    ) -> Pin<Box<dyn Future<Output = Result<bool, CodexErr>> + Send + '_>>;
}

impl NativeGoalThread for CodexThread {
    fn clear_native_goal(
        &self,
        goal_service: Arc<GoalService>,
        thread_id: ThreadId,
    ) -> Pin<Box<dyn Future<Output = Result<bool, CodexErr>> + Send + '_>> {
        Box::pin(async move {
            if !self.enabled(Feature::Goals) {
                return Ok(false);
            }
            let Some(state_db) = self.state_db() else {
                return Ok(false);
            };
            goal_service
                .clear_thread_goal(state_db.as_ref(), thread_id)
                .await
                .map_err(goal_service_error)
        })
    }
}

pub(crate) struct NativeGoalEventSink {
    sessions: Arc<Mutex<HashMap<SessionId, Arc<Thread>>>>,
}

impl NativeGoalEventSink {
    pub(crate) fn new(sessions: Arc<Mutex<HashMap<SessionId, Arc<Thread>>>>) -> Self {
        Self { sessions }
    }
}

impl ExtensionEventSink for NativeGoalEventSink {
    fn emit(&self, event: Event) {
        let session_id = match &event.msg {
            EventMsg::ThreadGoalUpdated(event) => SessionId::new(event.thread_id.to_string()),
            _ => return,
        };
        if let Some(thread) = self.sessions.lock().unwrap().get(&session_id).cloned() {
            thread.emit_extension_event(event);
        }
    }
}

fn goal_service_error(err: codex_goal_extension::GoalServiceError) -> CodexErr {
    match err {
        codex_goal_extension::GoalServiceError::InvalidRequest(message) => {
            CodexErr::InvalidRequest(message)
        }
        codex_goal_extension::GoalServiceError::Internal(message) => CodexErr::Fatal(message),
    }
}
