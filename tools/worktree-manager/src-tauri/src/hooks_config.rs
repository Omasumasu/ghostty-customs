use serde_json::{json, Value};
use std::path::PathBuf;

/// Generate the hooks configuration for Claude settings.json
/// Uses the correct Claude Code hooks format: event name → [{ matcher, hooks: [{ type, command }] }]
pub fn generate_hooks_config(port: u16) -> Value {
    let base_url = format!("http://127.0.0.1:{port}");

    // Claude Code pipes hook data as JSON to stdin of command hooks
    // We read stdin and POST it to our local server
    let curl_cmd = |endpoint: &str| -> String {
        format!(
            "curl -s -X POST -H 'Content-Type: application/json' -d @- {base_url}/hooks/{endpoint}"
        )
    };

    json!({
        "hooks": {
            "Notification": [
                {
                    "matcher": "",
                    "hooks": [{
                        "type": "command",
                        "command": curl_cmd("notification")
                    }]
                }
            ],
            "PreToolUse": [
                {
                    "matcher": "",
                    "hooks": [{
                        "type": "command",
                        "command": curl_cmd("tool")
                    }]
                }
            ],
            "PostToolUse": [
                {
                    "matcher": "",
                    "hooks": [{
                        "type": "command",
                        "command": curl_cmd("tool")
                    }]
                }
            ],
            "Stop": [
                {
                    "matcher": "",
                    "hooks": [{
                        "type": "command",
                        "command": curl_cmd("lifecycle")
                    }]
                }
            ]
        }
    })
}

/// Merge hooks config into existing settings, appending to arrays rather than overwriting.
pub fn merge_hooks_config(existing: &Value, port: u16) -> Value {
    let new_config = generate_hooks_config(port);
    let mut result = existing.clone();

    let new_hooks = new_config.get("hooks").unwrap().as_object().unwrap();

    // Ensure result has a "hooks" object
    if !result.get("hooks").map_or(false, |v| v.is_object()) {
        result["hooks"] = json!({});
    }

    // Remove old-style http hooks (from previous version of this app)
    for old_key in &["notification", "tool", "lifecycle"] {
        if let Some(arr) = result["hooks"].get(*old_key).and_then(|v| v.as_array()) {
            let clean: Vec<Value> = arr
                .iter()
                .filter(|entry| {
                    let url = entry.get("url").and_then(|v| v.as_str()).unwrap_or("");
                    !(url.contains("127.0.0.1") && url.contains("/hooks/"))
                })
                .cloned()
                .collect();
            if clean.is_empty() {
                if let Some(obj) = result["hooks"].as_object_mut() {
                    obj.remove(*old_key);
                }
            } else {
                result["hooks"][*old_key] = Value::Array(clean);
            }
        }
    }

    // Merge new hooks (correct format with matcher + hooks array)
    for (key, new_entries) in new_hooks {
        let new_arr = new_entries.as_array().unwrap();

        if let Some(existing_arr) = result["hooks"].get(key).and_then(|v| v.as_array()) {
            // Keep user's entries that don't contain our curl commands
            let mut clean: Vec<Value> = existing_arr
                .iter()
                .filter(|entry| {
                    let hooks_arr = entry.get("hooks").and_then(|v| v.as_array());
                    if let Some(hooks) = hooks_arr {
                        !hooks.iter().any(|h| {
                            let cmd = h.get("command").and_then(|v| v.as_str()).unwrap_or("");
                            cmd.contains("127.0.0.1") && cmd.contains("/hooks/")
                        })
                    } else {
                        true
                    }
                })
                .cloned()
                .collect();

            clean.extend(new_arr.iter().cloned());
            result["hooks"][key] = Value::Array(clean);
        } else {
            result["hooks"][key] = Value::Array(new_arr.clone());
        }
    }

    result
}

/// Get the path to Claude's settings.json
pub fn get_settings_path() -> PathBuf {
    let home = dirs_next_home();
    PathBuf::from(home).join(".claude").join("settings.json")
}

fn dirs_next_home() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "~".to_string())
}

/// Update Claude's settings.json with hooks configuration
pub fn update_settings(port: u16) -> Result<(), String> {
    let settings_path = get_settings_path();

    let existing: Value = if settings_path.exists() {
        let content =
            std::fs::read_to_string(&settings_path).map_err(|e| format!("Failed to read settings: {e}"))?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    let merged = merge_hooks_config(&existing, port);

    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {e}"))?;
    }

    let content =
        serde_json::to_string_pretty(&merged).map_err(|e| format!("Failed to serialize: {e}"))?;
    std::fs::write(&settings_path, content).map_err(|e| format!("Failed to write settings: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hooks_config() {
        let config = generate_hooks_config(8080);
        let hooks = config.get("hooks").unwrap();
        let notification = hooks.get("Notification").unwrap().as_array().unwrap();
        assert_eq!(notification.len(), 1);
        let inner = notification[0]["hooks"].as_array().unwrap();
        assert_eq!(inner[0]["type"].as_str().unwrap(), "command");
        let cmd = inner[0]["command"].as_str().unwrap();
        assert!(cmd.contains("127.0.0.1:8080"));
        assert!(cmd.contains("/hooks/notification"));
    }

    #[test]
    fn test_merge_hooks_config_empty() {
        let existing = json!({});
        let merged = merge_hooks_config(&existing, 9090);
        let hooks = merged.get("hooks").unwrap();
        assert!(hooks.get("Notification").is_some());
        assert!(hooks.get("PreToolUse").is_some());
        assert!(hooks.get("PostToolUse").is_some());
        assert!(hooks.get("Stop").is_some());
    }

    #[test]
    fn test_merge_hooks_config_preserves_other_settings() {
        let existing = json!({
            "theme": "dark",
            "someOtherSetting": true
        });
        let merged = merge_hooks_config(&existing, 9090);
        assert_eq!(merged.get("theme").unwrap().as_str().unwrap(), "dark");
        assert_eq!(merged.get("someOtherSetting").unwrap().as_bool().unwrap(), true);
        assert!(merged.get("hooks").is_some());
    }

    #[test]
    fn test_merge_hooks_config_preserves_user_hooks() {
        let existing = json!({
            "hooks": {
                "Notification": [{
                    "matcher": "idle_prompt",
                    "hooks": [{
                        "type": "command",
                        "command": "terminal-notifier -title 'Claude' -message 'Question'"
                    }]
                }]
            }
        });
        let merged = merge_hooks_config(&existing, 9090);
        let notifications = merged["hooks"]["Notification"].as_array().unwrap();
        assert_eq!(notifications.len(), 2);
        assert_eq!(notifications[0]["matcher"].as_str().unwrap(), "idle_prompt");
    }

    #[test]
    fn test_merge_hooks_config_deduplicates_on_restart() {
        let existing = json!({
            "hooks": {
                "Notification": [{
                    "matcher": "",
                    "hooks": [{
                        "type": "command",
                        "command": "curl -s -X POST -H 'Content-Type: application/json' -d @- http://127.0.0.1:8080/hooks/notification"
                    }]
                }]
            }
        });
        let merged = merge_hooks_config(&existing, 9090);
        let notifications = merged["hooks"]["Notification"].as_array().unwrap();
        assert_eq!(notifications.len(), 1);
        let cmd = notifications[0]["hooks"][0]["command"].as_str().unwrap();
        assert!(cmd.contains("9090"));
    }

    #[test]
    fn test_merge_removes_old_style_http_hooks() {
        let existing = json!({
            "hooks": {
                "notification": [{
                    "type": "http",
                    "url": "http://127.0.0.1:8080/hooks/notification",
                    "transport": "sse"
                }],
                "Notification": [{
                    "matcher": "idle_prompt",
                    "hooks": [{
                        "type": "command",
                        "command": "terminal-notifier -title 'Claude' -message 'Question'"
                    }]
                }]
            }
        });
        let merged = merge_hooks_config(&existing, 9090);
        // Old-style "notification" (lowercase) should be removed
        assert!(merged["hooks"].get("notification").is_none());
        // User's Notification hook + our new one
        let notifications = merged["hooks"]["Notification"].as_array().unwrap();
        assert_eq!(notifications.len(), 2);
    }
}
