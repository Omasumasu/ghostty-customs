use crate::types::{ActivityEntry, HookEvent, SessionInfo, SessionState};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct SessionManager {
    pub sessions: Mutex<HashMap<String, SessionInfo>>,
    pub activities: Mutex<HashMap<String, Vec<ActivityEntry>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            activities: Mutex::new(HashMap::new()),
        }
    }

    /// Handle a hook event and update session state.
    /// Returns the updated SessionInfo if a session was affected.
    pub fn handle_hook_event(&self, event: &HookEvent) -> Option<SessionInfo> {
        match event {
            HookEvent::Start { session_id, cwd } => {
                let sid = session_id.as_deref()?;
                let info = SessionInfo {
                    worktree_path: cwd.clone().unwrap_or_default(),
                    branch: String::new(),
                    state: SessionState::Working,
                    last_activity: chrono_now(),
                    question_text: None,
                };
                self.sessions
                    .lock()
                    .unwrap()
                    .insert(sid.to_string(), info.clone());
                Some(info)
            }
            HookEvent::Stop { session_id } => {
                let sid = session_id.as_deref()?;
                let mut sessions = self.sessions.lock().unwrap();
                if let Some(session) = sessions.get_mut(sid) {
                    session.state = SessionState::Idle;
                    session.last_activity = chrono_now();
                    session.question_text = None;
                    Some(session.clone())
                } else {
                    None
                }
            }
            HookEvent::Notification {
                message,
                session_id,
            } => {
                let sid = session_id.as_deref()?;
                let mut sessions = self.sessions.lock().unwrap();
                if let Some(session) = sessions.get_mut(sid) {
                    // Check if it's a question (contains "?" or is asking for input)
                    if message.contains('?')
                        || message.to_lowercase().contains("would you like")
                        || message.to_lowercase().contains("please")
                    {
                        session.state = SessionState::Question;
                        session.question_text = Some(message.clone());
                    }
                    session.last_activity = chrono_now();
                    Some(session.clone())
                } else {
                    None
                }
            }
            HookEvent::ToolUse { session_id, .. } | HookEvent::ToolResult { session_id, .. } => {
                let sid = session_id.as_deref()?;
                let mut sessions = self.sessions.lock().unwrap();
                if let Some(session) = sessions.get_mut(sid) {
                    session.state = SessionState::Working;
                    session.last_activity = chrono_now();
                    session.question_text = None;
                    Some(session.clone())
                } else {
                    None
                }
            }
            HookEvent::Unknown { .. } => None,
        }
    }

    /// Count sessions in Question state.
    pub fn question_count(&self) -> usize {
        self.sessions
            .lock()
            .unwrap()
            .values()
            .filter(|s| s.state == SessionState::Question)
            .count()
    }

    /// Get a specific session by ID.
    pub fn get_session(&self, session_id: &str) -> Option<SessionInfo> {
        self.sessions.lock().unwrap().get(session_id).cloned()
    }

    /// Get all sessions.
    pub fn get_all_sessions(&self) -> HashMap<String, SessionInfo> {
        self.sessions.lock().unwrap().clone()
    }

    /// Add an activity entry for a session.
    pub fn add_activity(&self, session_id: &str, entry: ActivityEntry) {
        let mut activities = self.activities.lock().unwrap();
        activities
            .entry(session_id.to_string())
            .or_insert_with(Vec::new)
            .push(entry);
    }

    /// Get activity log for a session.
    pub fn get_activity_log(&self, session_id: &str) -> Vec<ActivityEntry> {
        self.activities
            .lock()
            .unwrap()
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }
}

fn chrono_now() -> String {
    // Simple ISO-ish timestamp without chrono dependency
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}

// --- Tauri Commands ---

#[tauri::command]
pub fn get_session_status(
    session_id: String,
    manager: tauri::State<'_, SessionManager>,
) -> Result<Option<SessionInfo>, String> {
    Ok(manager.get_session(&session_id))
}

#[tauri::command]
pub fn get_all_sessions(
    manager: tauri::State<'_, SessionManager>,
) -> Result<HashMap<String, SessionInfo>, String> {
    Ok(manager.get_all_sessions())
}

#[tauri::command]
pub fn get_activity_log(
    session_id: String,
    manager: tauri::State<'_, SessionManager>,
) -> Result<Vec<ActivityEntry>, String> {
    Ok(manager.get_activity_log(&session_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_start() {
        let manager = SessionManager::new();
        let event = HookEvent::Start {
            session_id: Some("s1".to_string()),
            cwd: Some("/tmp/project".to_string()),
        };

        let result = manager.handle_hook_event(&event);
        assert!(result.is_some());

        let session = result.unwrap();
        assert_eq!(session.state, SessionState::Working);
        assert_eq!(session.worktree_path, "/tmp/project");
    }

    #[test]
    fn test_session_stop() {
        let manager = SessionManager::new();

        // Start first
        manager.handle_hook_event(&HookEvent::Start {
            session_id: Some("s1".to_string()),
            cwd: Some("/tmp".to_string()),
        });

        // Then stop
        let result = manager.handle_hook_event(&HookEvent::Stop {
            session_id: Some("s1".to_string()),
        });

        assert!(result.is_some());
        assert_eq!(result.unwrap().state, SessionState::Idle);
    }

    #[test]
    fn test_session_question() {
        let manager = SessionManager::new();

        manager.handle_hook_event(&HookEvent::Start {
            session_id: Some("s1".to_string()),
            cwd: None,
        });

        let result = manager.handle_hook_event(&HookEvent::Notification {
            message: "Should I continue?".to_string(),
            session_id: Some("s1".to_string()),
        });

        let session = result.unwrap();
        assert_eq!(session.state, SessionState::Question);
        assert_eq!(session.question_text, Some("Should I continue?".to_string()));
    }

    #[test]
    fn test_session_back_to_working_on_tool_use() {
        let manager = SessionManager::new();

        manager.handle_hook_event(&HookEvent::Start {
            session_id: Some("s1".to_string()),
            cwd: None,
        });

        // Question state
        manager.handle_hook_event(&HookEvent::Notification {
            message: "Should I continue?".to_string(),
            session_id: Some("s1".to_string()),
        });

        // Tool use should reset to Working
        let result = manager.handle_hook_event(&HookEvent::ToolUse {
            tool_name: "Read".to_string(),
            input: None,
            session_id: Some("s1".to_string()),
        });

        assert_eq!(result.unwrap().state, SessionState::Working);
    }

    #[test]
    fn test_question_count() {
        let manager = SessionManager::new();

        // Start two sessions
        manager.handle_hook_event(&HookEvent::Start {
            session_id: Some("s1".to_string()),
            cwd: None,
        });
        manager.handle_hook_event(&HookEvent::Start {
            session_id: Some("s2".to_string()),
            cwd: None,
        });

        assert_eq!(manager.question_count(), 0);

        // One question
        manager.handle_hook_event(&HookEvent::Notification {
            message: "Yes or no?".to_string(),
            session_id: Some("s1".to_string()),
        });

        assert_eq!(manager.question_count(), 1);

        // Another question
        manager.handle_hook_event(&HookEvent::Notification {
            message: "Really?".to_string(),
            session_id: Some("s2".to_string()),
        });

        assert_eq!(manager.question_count(), 2);
    }

    #[test]
    fn test_no_session_id_returns_none() {
        let manager = SessionManager::new();
        let event = HookEvent::Start {
            session_id: None,
            cwd: None,
        };
        assert!(manager.handle_hook_event(&event).is_none());
    }

    #[test]
    fn test_activity_log() {
        let manager = SessionManager::new();

        let entry = ActivityEntry {
            timestamp: "123".to_string(),
            activity_type: crate::types::ActivityType::ToolUse,
            summary: "Tool: Read".to_string(),
            details: None,
        };

        manager.add_activity("s1", entry.clone());
        let log = manager.get_activity_log("s1");
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].summary, "Tool: Read");

        // Empty log for unknown session
        let empty = manager.get_activity_log("unknown");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_stop_unknown_session() {
        let manager = SessionManager::new();
        let result = manager.handle_hook_event(&HookEvent::Stop {
            session_id: Some("nonexistent".to_string()),
        });
        assert!(result.is_none());
    }
}
