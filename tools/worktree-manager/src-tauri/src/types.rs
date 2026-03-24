use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: String,
    pub commit: String,
    pub is_bare: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    Working,
    Question,
    Idle,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionInfo {
    pub worktree_path: String,
    pub branch: String,
    pub state: SessionState,
    pub last_activity: String,
    pub question_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActivityType {
    ToolUse,
    Message,
    Error,
    Notification,
    Start,
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivityEntry {
    pub timestamp: String,
    pub activity_type: ActivityType,
    pub summary: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitStatus {
    pub modified: Vec<String>,
    pub added: Vec<String>,
    pub deleted: Vec<String>,
    pub untracked: Vec<String>,
}

impl GitStatus {
    pub fn new() -> Self {
        Self {
            modified: Vec::new(),
            added: Vec::new(),
            deleted: Vec::new(),
            untracked: Vec::new(),
        }
    }

    pub fn is_clean(&self) -> bool {
        self.modified.is_empty()
            && self.added.is_empty()
            && self.deleted.is_empty()
            && self.untracked.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HookEvent {
    Notification {
        message: String,
        session_id: Option<String>,
    },
    ToolUse {
        tool_name: String,
        input: Option<serde_json::Value>,
        session_id: Option<String>,
        cwd: Option<String>,
    },
    ToolResult {
        tool_name: String,
        output: Option<String>,
        session_id: Option<String>,
        cwd: Option<String>,
    },
    Start {
        session_id: Option<String>,
        cwd: Option<String>,
    },
    Stop {
        session_id: Option<String>,
    },
    Unknown {
        raw: serde_json::Value,
    },
}

impl HookEvent {
    /// Parse a HookEvent from a serde_json::Value.
    /// Claude Code hooks send: hook_event_name, session_id, cwd, tool_name, etc.
    /// Real payload example:
    /// {
    ///   "hook_event_name": "PostToolUse",
    ///   "session_id": "...",
    ///   "cwd": "/path/to/project",
    ///   "tool_name": "Bash",
    ///   "tool_input": {...},
    ///   "tool_response": {...}
    /// }
    pub fn from_value(value: serde_json::Value) -> Self {
        let obj = match value.as_object() {
            Some(o) => o,
            None => return HookEvent::Unknown { raw: value },
        };

        let session_id = obj
            .get("session_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let cwd = obj
            .get("cwd")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Claude Code uses "hook_event_name" field, fallback to "type" for manual/test payloads
        let event_name = obj
            .get("hook_event_name")
            .or_else(|| obj.get("type"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match event_name {
            // Notification hooks (idle_prompt, permission_prompt, etc.)
            "Notification" | "notification" => {
                // Claude Code Notification hook doesn't have a "message" field directly.
                // The notification content varies. We use tool_name or a summary.
                let message = obj
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("通知")
                    .to_string();
                HookEvent::Notification {
                    message,
                    session_id,
                }
            }
            // Tool use hooks
            "PreToolUse" | "tool_use" => {
                let tool_name = obj
                    .get("tool_name")
                    .or_else(|| obj.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let input = obj.get("tool_input").or_else(|| obj.get("input")).cloned();
                HookEvent::ToolUse {
                    tool_name,
                    input,
                    session_id,
                    cwd,
                }
            }
            "PostToolUse" | "tool_result" => {
                let tool_name = obj
                    .get("tool_name")
                    .or_else(|| obj.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let output = obj
                    .get("tool_response")
                    .and_then(|v| v.get("stdout"))
                    .and_then(|v| v.as_str())
                    .or_else(|| obj.get("output").and_then(|v| v.as_str()))
                    .map(|s| s.to_string());
                HookEvent::ToolResult {
                    tool_name,
                    output,
                    session_id,
                    cwd,
                }
            }
            // Lifecycle hooks
            "Stop" | "stop" => HookEvent::Stop { session_id },
            "start" => HookEvent::Start { session_id, cwd },
            _ => {
                // Any unknown event with session_id + cwd = treat as "session is active"
                // This ensures we at least register the session
                if session_id.is_some() && cwd.is_some() {
                    // First time seeing this session? Treat as start
                    HookEvent::Start { session_id, cwd }
                } else {
                    HookEvent::Unknown { raw: value.clone() }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_hook_event_notification() {
        let val = json!({
            "type": "notification",
            "message": "Task completed",
            "session_id": "abc123"
        });
        let event = HookEvent::from_value(val);
        assert_eq!(
            event,
            HookEvent::Notification {
                message: "Task completed".to_string(),
                session_id: Some("abc123".to_string()),
            }
        );
    }

    #[test]
    fn test_hook_event_tool_use() {
        let val = json!({
            "type": "tool_use",
            "tool_name": "Read",
            "input": {"path": "/tmp/file.txt"},
            "session_id": "abc123"
        });
        let event = HookEvent::from_value(val);
        match event {
            HookEvent::ToolUse {
                tool_name,
                input,
                session_id,
                ..
            } => {
                assert_eq!(tool_name, "Read");
                assert!(input.is_some());
                assert_eq!(session_id, Some("abc123".to_string()));
            }
            _ => panic!("Expected ToolUse"),
        }
    }

    #[test]
    fn test_hook_event_start_stop() {
        let start = json!({"type": "start", "session_id": "s1", "cwd": "/tmp"});
        let stop = json!({"type": "stop", "session_id": "s1"});

        assert_eq!(
            HookEvent::from_value(start),
            HookEvent::Start {
                session_id: Some("s1".to_string()),
                cwd: Some("/tmp".to_string()),
            }
        );
        assert_eq!(
            HookEvent::from_value(stop),
            HookEvent::Stop {
                session_id: Some("s1".to_string()),
            }
        );
    }

    #[test]
    fn test_hook_event_unknown() {
        let val = json!({"type": "something_new", "data": 42});
        let event = HookEvent::from_value(val);
        assert!(matches!(event, HookEvent::Unknown { .. }));
    }

    #[test]
    fn test_hook_event_non_object() {
        let val = json!("just a string");
        let event = HookEvent::from_value(val);
        assert!(matches!(event, HookEvent::Unknown { .. }));
    }

    #[test]
    fn test_git_status_empty() {
        let status = GitStatus::new();
        assert!(status.is_clean());
    }

    #[test]
    fn test_git_status_not_clean() {
        let mut status = GitStatus::new();
        status.modified.push("file.txt".to_string());
        assert!(!status.is_clean());
    }
}
