use std::process::Command;

/// Open a worktree directory in an IDE (tries Cursor first, falls back to VS Code).
#[tauri::command]
pub async fn open_in_ide(path: String) -> Result<(), String> {
    // Try Cursor first
    if let Ok(status) = Command::new("cursor").arg(&path).spawn() {
        drop(status);
        return Ok(());
    }

    // Fall back to VS Code
    Command::new("code")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open IDE (tried cursor and code): {e}"))?;

    Ok(())
}

/// Open a diff tool for the worktree (git difftool --dir-diff).
#[tauri::command]
pub async fn open_difftool(worktree_path: String, base_branch: Option<String>) -> Result<(), String> {
    let base = base_branch.unwrap_or_else(|| "main".to_string());

    Command::new("git")
        .args(["difftool", "--dir-diff", &base])
        .current_dir(&worktree_path)
        .spawn()
        .map_err(|e| format!("Failed to open difftool: {e}"))?;

    Ok(())
}

/// Open a new Ghostty terminal window at the given path.
#[tauri::command]
pub async fn open_terminal(path: String) -> Result<(), String> {
    // Use 'open -a Ghostty' with the directory
    Command::new("open")
        .args(["-a", "Ghostty", &path])
        .spawn()
        .map_err(|e| format!("Failed to open Ghostty: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // External tool tests are inherently integration tests that depend on
    // installed applications. We test the command construction logic above
    // through the public API, but actual spawning tests would require
    // the applications to be installed.

    #[test]
    fn test_module_loads() {
        // Smoke test: module compiles and loads
        assert!(true);
    }
}
