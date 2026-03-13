use serde_json::{json, Value};
use std::path::PathBuf;

/// Generate the hooks configuration for Claude settings.json
pub fn generate_hooks_config(port: u16) -> Value {
    let base_url = format!("http://127.0.0.1:{port}");
    json!({
        "hooks": {
            "notification": [{
                "type": "http",
                "url": format!("{base_url}/hooks/notification"),
                "transport": "sse"
            }],
            "tool": [{
                "type": "http",
                "url": format!("{base_url}/hooks/tool"),
                "transport": "sse"
            }],
            "lifecycle": [{
                "type": "http",
                "url": format!("{base_url}/hooks/lifecycle"),
                "transport": "sse"
            }]
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

    for (key, new_entries) in new_hooks {
        let new_arr = new_entries.as_array().unwrap();

        if let Some(existing_arr) = result["hooks"].get(key).and_then(|v| v.as_array()) {
            // Keep user's entries that don't look like our worktree-manager hooks
            // (identified by having a /hooks/ path on localhost)
            let mut clean: Vec<Value> = existing_arr
                .iter()
                .filter(|entry| {
                    let url = entry.get("url").and_then(|v| v.as_str()).unwrap_or("");
                    !url.contains("/hooks/")
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

    // Read existing settings or start fresh
    let existing: Value = if settings_path.exists() {
        let content =
            std::fs::read_to_string(&settings_path).map_err(|e| format!("Failed to read settings: {e}"))?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    let merged = merge_hooks_config(&existing, port);

    // Ensure directory exists
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
        let notification = hooks.get("notification").unwrap().as_array().unwrap();
        assert_eq!(notification.len(), 1);
        assert_eq!(
            notification[0].get("url").unwrap().as_str().unwrap(),
            "http://127.0.0.1:8080/hooks/notification"
        );
    }

    #[test]
    fn test_merge_hooks_config_empty() {
        let existing = json!({});
        let merged = merge_hooks_config(&existing, 9090);
        let hooks = merged.get("hooks").unwrap();
        assert!(hooks.get("notification").is_some());
        assert!(hooks.get("tool").is_some());
        assert!(hooks.get("lifecycle").is_some());
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
                "notification": [{
                    "type": "command",
                    "command": "notify-send"
                }]
            }
        });
        let merged = merge_hooks_config(&existing, 9090);
        let notifications = merged["hooks"]["notification"].as_array().unwrap();
        // Should have user's entry + our entry
        assert_eq!(notifications.len(), 2);
        assert_eq!(
            notifications[0]["type"].as_str().unwrap(),
            "command"
        );
        assert_eq!(
            notifications[1]["url"].as_str().unwrap(),
            "http://127.0.0.1:9090/hooks/notification"
        );
    }

    #[test]
    fn test_merge_hooks_config_deduplicates_on_restart() {
        // Simulate: first run set port 8080, second run sets port 9090
        let existing = json!({
            "hooks": {
                "notification": [{
                    "type": "http",
                    "url": "http://127.0.0.1:8080/hooks/notification",
                    "transport": "sse"
                }]
            }
        });
        let merged = merge_hooks_config(&existing, 9090);
        let notifications = merged["hooks"]["notification"].as_array().unwrap();
        // Old entry should be replaced, not duplicated
        assert_eq!(notifications.len(), 1);
        assert_eq!(
            notifications[0]["url"].as_str().unwrap(),
            "http://127.0.0.1:9090/hooks/notification"
        );
    }
}
