use crate::types::{ActivityEntry, ActivityType};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

/// Encode a project path for use as a JSONL session directory name.
/// Replaces `/` with `-` and strips leading `-`.
pub fn encode_project_path(path: &str) -> String {
    let encoded = path.replace('/', "-");
    encoded.trim_start_matches('-').to_string()
}

/// Parse a single JSONL line into an ActivityEntry.
pub fn parse_jsonl_line(line: &str) -> Option<ActivityEntry> {
    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    let obj = value.as_object()?;

    let timestamp = obj
        .get("timestamp")
        .or_else(|| obj.get("ts"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let event_type = obj
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let (activity_type, summary, details) = match event_type {
        "tool_use" | "tool" => {
            let tool = obj
                .get("tool")
                .or_else(|| obj.get("tool_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            (
                ActivityType::ToolUse,
                format!("Tool: {tool}"),
                obj.get("input")
                    .map(|v| v.to_string()),
            )
        }
        "message" | "assistant" => {
            let text = obj
                .get("message")
                .or_else(|| obj.get("text"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let summary = if text.len() > 100 {
                format!("{}...", &text[..100])
            } else {
                text.clone()
            };
            (ActivityType::Message, summary, Some(text))
        }
        "error" => {
            let msg = obj
                .get("error")
                .or_else(|| obj.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            (ActivityType::Error, msg.clone(), Some(msg))
        }
        "notification" => {
            let msg = obj
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            (ActivityType::Notification, msg, None)
        }
        "start" => (
            ActivityType::Start,
            "Session started".to_string(),
            None,
        ),
        "stop" => (
            ActivityType::Stop,
            "Session stopped".to_string(),
            None,
        ),
        _ => (
            ActivityType::Message,
            format!("Unknown event: {event_type}"),
            Some(line.to_string()),
        ),
    };

    Some(ActivityEntry {
        timestamp,
        activity_type,
        summary,
        details,
    })
}

/// Find the latest session file in a directory (by modification time).
pub fn find_latest_session_file(session_dir: &Path) -> Option<PathBuf> {
    if !session_dir.is_dir() {
        return None;
    }

    fs::read_dir(session_dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map_or(false, |ext| ext == "jsonl")
        })
        .max_by_key(|entry| entry.metadata().ok().and_then(|m| m.modified().ok()))
        .map(|entry| entry.path())
}

/// Watch a JSONL session file for new lines and send ActivityEntries.
/// This function blocks the current thread until the watcher is dropped.
pub fn watch_session_file(
    path: PathBuf,
    sender: mpsc::UnboundedSender<ActivityEntry>,
) -> Result<RecommendedWatcher, String> {
    let watch_path = path.clone();

    // Read existing content to get current position
    let initial_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let file_offset = std::sync::Arc::new(std::sync::Mutex::new(initial_size));

    let offset = file_offset.clone();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                // Read new lines from file
                if let Ok(file) = fs::File::open(&watch_path) {
                    let mut reader = BufReader::new(file);
                    let mut current_offset = offset.lock().unwrap();
                    if reader.seek(SeekFrom::Start(*current_offset)).is_ok() {
                        let mut line = String::new();
                        while reader.read_line(&mut line).unwrap_or(0) > 0 {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                if let Some(entry) = parse_jsonl_line(trimmed) {
                                    let _ = sender.send(entry);
                                }
                            }
                            *current_offset += line.len() as u64;
                            line.clear();
                        }
                    }
                }
            }
        }
    })
    .map_err(|e| format!("Failed to create watcher: {e}"))?;

    // Watch the parent directory (some OS don't support watching individual files well)
    let watch_dir = path.parent().unwrap_or(&path);
    watcher
        .watch(watch_dir, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Failed to watch: {e}"))?;

    Ok(watcher)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_project_path() {
        assert_eq!(
            encode_project_path("/Users/me/project"),
            "Users-me-project"
        );
        assert_eq!(
            encode_project_path("/tmp/foo/bar"),
            "tmp-foo-bar"
        );
        assert_eq!(encode_project_path("relative/path"), "relative-path");
    }

    #[test]
    fn test_parse_jsonl_line_tool_use() {
        let line = r#"{"type":"tool_use","tool":"Read","input":{"path":"/tmp"},"timestamp":"2024-01-01T00:00:00Z"}"#;
        let entry = parse_jsonl_line(line).unwrap();
        assert_eq!(entry.activity_type, ActivityType::ToolUse);
        assert_eq!(entry.summary, "Tool: Read");
        assert_eq!(entry.timestamp, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_parse_jsonl_line_message() {
        let line = r#"{"type":"message","message":"Hello world","timestamp":"2024-01-01T00:00:00Z"}"#;
        let entry = parse_jsonl_line(line).unwrap();
        assert_eq!(entry.activity_type, ActivityType::Message);
        assert_eq!(entry.summary, "Hello world");
    }

    #[test]
    fn test_parse_jsonl_line_error() {
        let line = r#"{"type":"error","error":"Something failed","timestamp":"2024-01-01T00:00:00Z"}"#;
        let entry = parse_jsonl_line(line).unwrap();
        assert_eq!(entry.activity_type, ActivityType::Error);
        assert!(entry.summary.contains("Something failed"));
    }

    #[test]
    fn test_parse_jsonl_line_invalid() {
        assert!(parse_jsonl_line("not json").is_none());
        assert!(parse_jsonl_line("").is_none());
    }

    #[test]
    fn test_parse_jsonl_line_start_stop() {
        let start = r#"{"type":"start","timestamp":"2024-01-01T00:00:00Z"}"#;
        let stop = r#"{"type":"stop","timestamp":"2024-01-01T00:01:00Z"}"#;

        let s = parse_jsonl_line(start).unwrap();
        assert_eq!(s.activity_type, ActivityType::Start);

        let e = parse_jsonl_line(stop).unwrap();
        assert_eq!(e.activity_type, ActivityType::Stop);
    }

    #[test]
    fn test_find_latest_session_file() {
        let dir = tempfile::tempdir().unwrap();

        // No files
        assert!(find_latest_session_file(dir.path()).is_none());

        // Create some files
        std::fs::write(dir.path().join("old.jsonl"), "{}").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        std::fs::write(dir.path().join("new.jsonl"), "{}").unwrap();
        // Non-jsonl file should be ignored
        std::fs::write(dir.path().join("other.txt"), "{}").unwrap();

        let latest = find_latest_session_file(dir.path()).unwrap();
        assert!(latest.file_name().unwrap().to_str().unwrap() == "new.jsonl");
    }

    #[test]
    fn test_find_latest_session_file_nonexistent() {
        assert!(find_latest_session_file(Path::new("/nonexistent/path")).is_none());
    }
}
