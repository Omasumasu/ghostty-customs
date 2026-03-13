use crate::types::{GitStatus, WorktreeInfo};
use std::process::Command;

/// Parse the output of `git worktree list --porcelain`
pub fn parse_worktree_list(output: &str) -> Vec<WorktreeInfo> {
    let mut worktrees = Vec::new();
    let mut current_path = None;
    let mut current_commit = None;
    let mut current_branch = None;
    let mut is_bare = false;

    for line in output.lines() {
        if line.starts_with("worktree ") {
            // Save previous worktree if any
            if let Some(path) = current_path.take() {
                worktrees.push(WorktreeInfo {
                    path,
                    branch: current_branch.take().unwrap_or_default(),
                    commit: current_commit.take().unwrap_or_default(),
                    is_bare,
                });
                is_bare = false;
            }
            current_path = Some(line.trim_start_matches("worktree ").to_string());
        } else if line.starts_with("HEAD ") {
            current_commit = Some(line.trim_start_matches("HEAD ").to_string());
        } else if line.starts_with("branch ") {
            let branch = line.trim_start_matches("branch ");
            // Strip refs/heads/ prefix
            let branch = branch
                .strip_prefix("refs/heads/")
                .unwrap_or(branch);
            current_branch = Some(branch.to_string());
        } else if line == "bare" {
            is_bare = true;
        } else if line == "detached" {
            current_branch = Some("(detached)".to_string());
        }
    }

    // Don't forget the last entry
    if let Some(path) = current_path.take() {
        worktrees.push(WorktreeInfo {
            path,
            branch: current_branch.take().unwrap_or_default(),
            commit: current_commit.take().unwrap_or_default(),
            is_bare,
        });
    }

    worktrees
}

/// Parse the output of `git status --short`
pub fn parse_git_status(output: &str) -> GitStatus {
    let mut status = GitStatus::new();

    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        // git status --short format: XY filename
        // X = index status, Y = working tree status
        // The first two characters are always the status codes, then a space, then the filename.
        if line.len() < 4 {
            continue;
        }
        let x = line.as_bytes()[0];
        let y = line.as_bytes()[1];
        // line[2] should be ' '
        let filename = line[3..].to_string();

        match (x, y) {
            (b'?', b'?') => status.untracked.push(filename),
            (b'A', b' ') | (b'A', b'M') => status.added.push(filename),
            (b'D', b' ') | (b' ', b'D') => status.deleted.push(filename),
            (b'M', _) | (_, b'M') => status.modified.push(filename),
            (b'R', _) => status.modified.push(filename),
            _ => status.untracked.push(filename),
        }
    }

    status
}

/// Get number of commits ahead of a reference branch
pub fn get_commits_ahead(worktree_path: &str, base_branch: &str) -> Result<usize, String> {
    let output = Command::new("git")
        .args(["rev-list", "--count", &format!("{base_branch}..HEAD")])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| format!("Failed to run git rev-list: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let count_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    count_str
        .parse::<usize>()
        .map_err(|e| format!("Failed to parse commit count: {e}"))
}

/// Check if a branch has been merged into main/master
pub fn check_is_merged(repo_path: &str, branch: &str) -> Result<bool, String> {
    // Try main first, then master, then fall back to HEAD
    let targets = ["main", "master", "HEAD"];
    let merge_base = targets.iter().find(|&&target| {
        // Check if target ref exists
        let check = Command::new("git")
            .args(["rev-parse", "--verify", target])
            .current_dir(repo_path)
            .output();
        check.map(|o| o.status.success()).unwrap_or(false)
    }).copied().unwrap_or("HEAD");

    let output = Command::new("git")
        .args(["branch", "--merged", merge_base, "--list", branch])
        .current_dir(repo_path)
        .output()
        .map_err(|e| format!("Failed to run git branch --merged: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().any(|line| line.trim() == branch || line.trim().trim_start_matches("* ") == branch))
}

// --- Tauri Commands ---

#[tauri::command]
pub async fn list_worktrees(repo_path: String) -> Result<Vec<WorktreeInfo>, String> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| format!("Failed to run git worktree list: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_worktree_list(&stdout))
}

#[tauri::command]
pub async fn create_worktree(
    repo_path: String,
    branch: String,
    path: String,
    base_branch: Option<String>,
) -> Result<WorktreeInfo, String> {
    let base = base_branch.as_deref().unwrap_or("main");
    let output = Command::new("git")
        .args(["worktree", "add", "-b", &branch, &path, base])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| format!("Failed to create worktree: {e}"))?;

    if !output.status.success() {
        // Try without -b (branch already exists)
        let output2 = Command::new("git")
            .args(["worktree", "add", &path, &branch])
            .current_dir(&repo_path)
            .output()
            .map_err(|e| format!("Failed to create worktree: {e}"))?;

        if !output2.status.success() {
            return Err(String::from_utf8_lossy(&output2.stderr).to_string());
        }
    }

    // Get the HEAD commit of the new worktree
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&path)
        .output()
        .map_err(|e| format!("Failed to get HEAD: {e}"))?;

    let commit = String::from_utf8_lossy(&head.stdout).trim().to_string();

    Ok(WorktreeInfo {
        path,
        branch,
        commit,
        is_bare: false,
    })
}

#[tauri::command]
pub async fn remove_worktree(repo_path: String, path: String) -> Result<(), String> {
    let output = Command::new("git")
        .args(["worktree", "remove", &path])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| format!("Failed to remove worktree: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn get_worktree_status(worktree_path: String) -> Result<GitStatus, String> {
    let output = Command::new("git")
        .args(["status", "--short"])
        .current_dir(&worktree_path)
        .output()
        .map_err(|e| format!("Failed to run git status: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_git_status(&stdout))
}

#[tauri::command]
pub async fn cleanup_merged(repo_path: String) -> Result<Vec<String>, String> {
    // First list all worktrees
    let worktrees = list_worktrees(repo_path.clone()).await?;
    let mut removed = Vec::new();

    for wt in &worktrees {
        if wt.is_bare || wt.branch == "main" || wt.branch == "master" {
            continue;
        }
        if check_is_merged(&repo_path, &wt.branch)? {
            if let Ok(()) = remove_worktree(repo_path.clone(), wt.path.clone()).await {
                removed.push(wt.branch.clone());
            }
        }
    }

    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_worktree_list_basic() {
        let output = "\
worktree /Users/me/project
HEAD abc123def456
branch refs/heads/main

worktree /Users/me/project-feature
HEAD def789abc012
branch refs/heads/feature/login

";
        let result = parse_worktree_list(output);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].path, "/Users/me/project");
        assert_eq!(result[0].branch, "main");
        assert_eq!(result[0].commit, "abc123def456");
        assert!(!result[0].is_bare);

        assert_eq!(result[1].path, "/Users/me/project-feature");
        assert_eq!(result[1].branch, "feature/login");
    }

    #[test]
    fn test_parse_worktree_list_bare() {
        let output = "\
worktree /Users/me/project.git
HEAD abc123
bare

";
        let result = parse_worktree_list(output);
        assert_eq!(result.len(), 1);
        assert!(result[0].is_bare);
    }

    #[test]
    fn test_parse_worktree_list_detached() {
        let output = "\
worktree /Users/me/project-detached
HEAD abc123
detached

";
        let result = parse_worktree_list(output);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].branch, "(detached)");
    }

    #[test]
    fn test_parse_worktree_list_empty() {
        let result = parse_worktree_list("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_git_status() {
        let output = " M src/main.rs\nA  new_file.rs\n D deleted.rs\n?? untracked.txt\n";
        let status = parse_git_status(output);
        assert_eq!(status.modified, vec!["src/main.rs"]);
        assert_eq!(status.added, vec!["new_file.rs"]);
        assert_eq!(status.deleted, vec!["deleted.rs"]);
        assert_eq!(status.untracked, vec!["untracked.txt"]);
    }

    #[test]
    fn test_parse_git_status_empty() {
        let status = parse_git_status("");
        assert!(status.is_clean());
    }

    #[test]
    fn test_parse_git_status_modified_variants() {
        let output = "MM both.rs\n M working.rs\n";
        let status = parse_git_status(output);
        assert_eq!(status.modified.len(), 2);
        assert!(status.modified.contains(&"both.rs".to_string()));
        assert!(status.modified.contains(&"working.rs".to_string()));
    }

    #[test]
    fn test_integration_with_temp_repo() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path();

        // Init a git repo
        let init = Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()
            .unwrap();
        assert!(init.status.success());

        // Configure git user for commits
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(repo_path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Create initial commit
        std::fs::write(repo_path.join("README.md"), "# Test").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(repo_path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        // List worktrees
        let output = Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .current_dir(repo_path)
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let worktrees = parse_worktree_list(&stdout);
        assert_eq!(worktrees.len(), 1);

        // Create a worktree
        let wt_path = repo_path.parent().unwrap().join("test-worktree");
        let add = Command::new("git")
            .args(["worktree", "add", "-b", "test-branch", wt_path.to_str().unwrap()])
            .current_dir(repo_path)
            .output()
            .unwrap();
        assert!(add.status.success(), "Failed: {}", String::from_utf8_lossy(&add.stderr));

        // List again
        let output = Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .current_dir(repo_path)
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let worktrees = parse_worktree_list(&stdout);
        assert_eq!(worktrees.len(), 2);
        assert_eq!(worktrees[1].branch, "test-branch");

        // Cleanup
        Command::new("git")
            .args(["worktree", "remove", wt_path.to_str().unwrap()])
            .current_dir(repo_path)
            .output()
            .unwrap();
    }
}
