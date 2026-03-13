use std::process::Command;

/// Escape a string for safe use in shell commands.
pub fn shell_escape(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    // If the string contains no special characters, return as-is
    if s.chars()
        .all(|c| c.is_alphanumeric() || c == '/' || c == '.' || c == '-' || c == '_')
    {
        return s.to_string();
    }
    // Single-quote the string, escaping existing single quotes
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Build the claude command string for launching in a terminal.
pub fn build_claude_command(worktree_path: &str, prompt: Option<&str>) -> String {
    let escaped_path = shell_escape(worktree_path);
    match prompt {
        Some(p) => {
            let escaped_prompt = shell_escape(p);
            format!("cd {escaped_path} && claude {escaped_prompt}")
        }
        None => format!("cd {escaped_path} && claude"),
    }
}

/// Launch Claude Code in a new Ghostty terminal window.
#[tauri::command]
pub async fn launch_claude(
    worktree_path: String,
    prompt: Option<String>,
) -> Result<(), String> {
    let cmd = build_claude_command(&worktree_path, prompt.as_deref());

    // Use AppleScript to open a new Ghostty window with the command
    let applescript = format!(
        r#"tell application "Ghostty"
    activate
    tell application "System Events"
        tell process "Ghostty"
            keystroke "n" using command down
            delay 0.5
            keystroke "{}"
            key code 36
        end tell
    end tell
end tell"#,
        cmd.replace('\\', "\\\\").replace('"', "\\\"")
    );

    Command::new("osascript")
        .arg("-e")
        .arg(&applescript)
        .spawn()
        .map_err(|e| format!("Failed to launch Ghostty: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escape_simple() {
        assert_eq!(shell_escape("/tmp/project"), "/tmp/project");
        assert_eq!(shell_escape("hello-world"), "hello-world");
        assert_eq!(shell_escape("file.txt"), "file.txt");
    }

    #[test]
    fn test_shell_escape_spaces() {
        assert_eq!(shell_escape("hello world"), "'hello world'");
    }

    #[test]
    fn test_shell_escape_quotes() {
        assert_eq!(shell_escape("it's"), "'it'\\''s'");
    }

    #[test]
    fn test_shell_escape_empty() {
        assert_eq!(shell_escape(""), "''");
    }

    #[test]
    fn test_shell_escape_special_chars() {
        assert_eq!(shell_escape("$(rm -rf /)"), "'$(rm -rf /)'");
    }

    #[test]
    fn test_build_claude_command_no_prompt() {
        let cmd = build_claude_command("/tmp/project", None);
        assert_eq!(cmd, "cd /tmp/project && claude");
    }

    #[test]
    fn test_build_claude_command_with_prompt() {
        let cmd = build_claude_command("/tmp/project", Some("fix the bug"));
        assert_eq!(cmd, "cd /tmp/project && claude 'fix the bug'");
    }

    #[test]
    fn test_build_claude_command_with_special_path() {
        let cmd = build_claude_command("/tmp/my project", None);
        assert_eq!(cmd, "cd '/tmp/my project' && claude");
    }
}
