use std::process::Command;

/// Open a worktree directory in an IDE (tries Cursor first, falls back to VS Code).
/// Uses `open -a` for macOS compatibility (CLI tools may not be in PATH from GUI apps).
#[tauri::command]
pub async fn open_in_ide(path: String) -> Result<(), String> {
    // Try Cursor first
    if let Ok(output) = Command::new("open").args(["-a", "Cursor", &path]).output() {
        if output.status.success() {
            return Ok(());
        }
    }

    // Fall back to VS Code
    if let Ok(output) = Command::new("open").args(["-a", "Visual Studio Code", &path]).output() {
        if output.status.success() {
            return Ok(());
        }
    }

    // Last resort: try CLI commands (may work if user has them in PATH)
    if let Ok(_) = Command::new("cursor").arg(&path).spawn() {
        return Ok(());
    }

    Command::new("code")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("IDE を開けませんでした (Cursor, VS Code): {e}"))?;

    Ok(())
}

/// Open a diff tool for the worktree (git difftool --dir-diff).
#[tauri::command]
pub async fn open_difftool(worktree_path: String, base_branch: Option<String>) -> Result<(), String> {
    let base = base_branch.unwrap_or_else(|| "main".to_string());

    // Use full path to git to avoid PATH issues
    let git = find_git();

    Command::new(&git)
        .args(["difftool", "--dir-diff", &base])
        .current_dir(&worktree_path)
        .spawn()
        .map_err(|e| format!("差分ツールを開けませんでした: {e}"))?;

    Ok(())
}

/// Open a new Ghostty terminal window at the given path.
#[tauri::command]
pub async fn open_terminal(path: String) -> Result<(), String> {
    // Use 'open -a Ghostty' with the directory
    Command::new("open")
        .args(["-a", "Ghostty", &path])
        .spawn()
        .map_err(|e| format!("Ghostty を開けませんでした: {e}"))?;

    Ok(())
}

/// Find git binary, preferring Xcode/Homebrew paths for GUI app context
fn find_git() -> String {
    for candidate in &["/usr/bin/git", "/opt/homebrew/bin/git", "/usr/local/bin/git"] {
        if std::path::Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }
    "git".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_git() {
        let git = find_git();
        assert!(!git.is_empty());
    }

    #[test]
    fn test_module_loads() {
        assert!(true);
    }
}
