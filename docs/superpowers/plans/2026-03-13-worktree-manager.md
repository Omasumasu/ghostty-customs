# Worktree Manager Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Git worktree 管理と Claude Code セッション監視を行う Tauri デスクトップアプリを構築する。

**Architecture:** Tauri v2 (Rust backend + SvelteKit frontend) のハイブリッド型アプリ。メニューバー Tray に常駐し、フルウィンドウのコントロールパネルを提供。Rust バックエンドが git CLI ラッパー、Claude Code hooks HTTP サーバー、JSONL ファイル監視を担い、フロントエンドにリアルタイムイベントを push する。

**Tech Stack:** Tauri v2, Rust, SvelteKit, TypeScript, Tailwind CSS, axum, notify, tokio, serde

**Spec:** `docs/superpowers/specs/2026-03-13-worktree-manager-design.md`

---

## File Structure

```
tools/worktree-manager/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # Tauri app builder, plugin registration, state management
│   │   ├── worktree.rs          # git worktree CRUD: list, add, remove, status, merge check
│   │   ├── claude.rs            # Claude Code CLI spawning, process tracking
│   │   ├── hooks_server.rs      # axum HTTP server for Claude Code hooks
│   │   ├── hooks_config.rs      # ~/.claude/settings.json auto-update
│   │   ├── watcher.rs           # JSONL file watcher using notify crate
│   │   ├── session_manager.rs   # Session state machine, watcher→frontend bridge
│   │   ├── tray.rs              # macOS menu bar tray icon, badge, dropdown menu
│   │   ├── external.rs          # External tool launchers (IDE, difft, terminal)
│   │   └── types.rs             # Shared types: WorktreeInfo, SessionInfo, SessionState, etc.
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── icons/
├── src/
│   ├── routes/
│   │   ├── +layout.svelte       # Global layout, CSS import
│   │   ├── +layout.ts           # SSR disable (export const ssr = false)
│   │   └── +page.svelte         # Main page: WorktreeList + WorktreeDetail
│   ├── lib/
│   │   ├── components/
│   │   │   ├── WorktreeList.svelte
│   │   │   ├── WorktreeDetail.svelte
│   │   │   ├── SessionStatus.svelte
│   │   │   ├── ActivityLog.svelte
│   │   │   ├── NewWorktreeDialog.svelte
│   │   │   └── CleanupDialog.svelte
│   │   ├── stores/
│   │   │   ├── worktrees.ts
│   │   │   └── sessions.ts
│   │   └── types.ts
│   ├── app.css
│   └── app.html
├── svelte.config.js             # adapter-static for Tauri
├── vite.config.ts
├── package.json
├── tsconfig.json
└── tailwind.config.js
```

---

## Chunk 1: プロジェクト基盤 + Git Worktree バックエンド

### Task 1: Tauri + SvelteKit プロジェクトスキャフォールド

**Files:**
- Create: `tools/worktree-manager/` (Tauri scaffold)

- [ ] **Step 1: Tauri + SvelteKit プロジェクト作成**

```bash
cd /Users/hikaru_masuta/Documents/GitHub/ghostty-customs/tools
npm create tauri-app@latest worktree-manager -- --template sveltekit-ts --manager npm
```

対話プロンプトの回答:
- Package manager: npm
- Frontend: SvelteKit (TypeScript)

- [ ] **Step 2: 依存関係インストール**

```bash
cd /Users/hikaru_masuta/Documents/GitHub/ghostty-customs/tools/worktree-manager
npm install
```

- [ ] **Step 3: Tailwind CSS セットアップ**

```bash
cd /Users/hikaru_masuta/Documents/GitHub/ghostty-customs/tools/worktree-manager
npm install -D tailwindcss @tailwindcss/vite
```

`vite.config.ts` に Tailwind プラグインを追加:

```typescript
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
});
```

`src/app.css`:

```css
@import "tailwindcss";
```

`src/routes/+layout.svelte`:

```svelte
<script>
  import "../app.css";
  let { children } = $props();
</script>

{@render children()}
```

- [ ] **Step 4: SSR 無効化 + adapter-static**

```bash
cd /Users/hikaru_masuta/Documents/GitHub/ghostty-customs/tools/worktree-manager
npm install -D @sveltejs/adapter-static
```

`svelte.config.js`:

```javascript
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

export default {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({ fallback: "index.html" }),
  },
};
```

`src/routes/+layout.ts`:

```typescript
export const ssr = false;
export const prerender = false;
```

- [ ] **Step 5: Rust バックエンド依存関係追加**

`src-tauri/Cargo.toml` の `[dependencies]` に追加:

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
axum = "0.8"
notify = "7"
```

`[dev-dependencies]` に追加:

```toml
tempfile = "3"
```

- [ ] **Step 6: ビルド確認**

```bash
cd /Users/hikaru_masuta/Documents/GitHub/ghostty-customs/tools/worktree-manager
cargo check --manifest-path src-tauri/Cargo.toml
npm run build
```

Expected: ビルド成功

- [ ] **Step 7: コミット**

```bash
git add tools/worktree-manager/
git commit -m "feat: Tauri + SvelteKit プロジェクトスキャフォールド"
```

---

### Task 2: 共有型定義 (types.rs + types.ts)

**Files:**
- Create: `tools/worktree-manager/src-tauri/src/types.rs`
- Create: `tools/worktree-manager/src/lib/types.ts`

- [ ] **Step 1: Rust 型定義を作成**

`src-tauri/src/types.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: String,
    pub base_branch: String,
    pub commit_hash: String,
    pub commits_ahead: i32,
    pub is_merged: bool,
    pub session: Option<SessionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub state: SessionState,
    pub started_at: String,
    pub last_activity: String,
    pub current_tool: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    Working,
    Question,
    Idle,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub timestamp: String,
    #[serde(rename = "type")]
    pub entry_type: ActivityType,
    pub tool_name: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    ToolUse,
    Message,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub added: i32,
    pub modified: i32,
    pub deleted: i32,
}

/// Claude Code hooks から受信するイベント。
/// serde(tag) ではなく手動デシリアライズを使用（Claude hooks のペイロードが
/// 同じ "type" フィールドで異なる構造を持つため）。
#[derive(Debug, Clone, Serialize)]
pub enum HookEvent {
    Notification {
        matcher: String,
        session_id: String,
        cwd: String,
    },
    ToolUse {
        event_type: String, // "PreToolUse" | "PostToolUse"
        tool_name: String,
        session_id: String,
        cwd: String,
    },
    Stop {
        session_id: String,
        cwd: String,
    },
}

impl HookEvent {
    /// serde_json::Value から手動でデシリアライズ
    pub fn from_value(value: &serde_json::Value) -> Option<Self> {
        let event_type = value.get("type")?.as_str()?;
        let session_id = value
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let cwd = value
            .get("cwd")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        match event_type {
            "Notification" => {
                let matcher = value
                    .get("matcher")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Some(HookEvent::Notification {
                    matcher,
                    session_id,
                    cwd,
                })
            }
            "PreToolUse" | "PostToolUse" => {
                let tool_name = value
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Some(HookEvent::ToolUse {
                    event_type: event_type.to_string(),
                    tool_name,
                    session_id,
                    cwd,
                })
            }
            "Stop" => Some(HookEvent::Stop { session_id, cwd }),
            _ => None,
        }
    }
}
```

- [ ] **Step 2: TypeScript 型定義を作成**

`src/lib/types.ts`:

```typescript
export interface WorktreeInfo {
  path: string;
  branch: string;
  base_branch: string;
  commit_hash: string;
  commits_ahead: number;
  is_merged: boolean;
  session: SessionInfo | null;
}

export interface SessionInfo {
  session_id: string;
  state: "working" | "question" | "idle" | "merged";
  started_at: string;
  last_activity: string;
  current_tool?: string;
}

export interface ActivityEntry {
  timestamp: string;
  type: "tool_use" | "message" | "error";
  tool_name?: string;
  summary: string;
}

export interface GitStatus {
  added: number;
  modified: number;
  deleted: number;
}
```

- [ ] **Step 3: `main.rs` に types モジュールを登録**

`src-tauri/src/main.rs` の先頭に:

```rust
mod types;
```

- [ ] **Step 4: ビルド確認**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: 成功

- [ ] **Step 5: コミット**

```bash
git add tools/worktree-manager/src-tauri/src/types.rs tools/worktree-manager/src/lib/types.ts tools/worktree-manager/src-tauri/src/main.rs
git commit -m "feat: 共有型定義 (WorktreeInfo, SessionInfo, HookEvent)"
```

---

### Task 3a: Git Worktree パーサー

**Files:**
- Create: `tools/worktree-manager/src-tauri/src/worktree.rs`

- [ ] **Step 1: パーサーのテストを作成**

`src-tauri/src/worktree.rs`:

```rust
use crate::types::{GitStatus, WorktreeInfo};
use std::process::Command;

pub fn parse_worktree_list(output: &str) -> Vec<WorktreeInfo> {
    todo!()
}

pub fn parse_git_status(output: &str) -> GitStatus {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_worktree_list_output() {
        let output = "/Users/test/repo abc1234 [main]\n/Users/test/repo/.worktrees/feat abc5678 [feat]\n";
        let result = parse_worktree_list(output);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].branch, "main");
        assert_eq!(result[0].commit_hash, "abc1234");
        assert_eq!(result[1].branch, "feat");
        assert_eq!(result[1].path, "/Users/test/repo/.worktrees/feat");
    }

    #[test]
    fn test_parse_worktree_list_bare() {
        let output = "/Users/test/repo (bare)\n";
        let result = parse_worktree_list(output);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_git_status_short() {
        let output = " M src/main.rs\nA  src/new.rs\nD  src/old.rs\n?? untracked.txt\n";
        let status = parse_git_status(output);
        assert_eq!(status.added, 1);
        assert_eq!(status.modified, 1);
        assert_eq!(status.deleted, 1);
    }
}
```

- [ ] **Step 2: テスト実行 — 失敗確認**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- worktree::tests
```

Expected: FAIL (todo! panics)

- [ ] **Step 3: パーサーを実装**

`parse_worktree_list` と `parse_git_status` の実装:

```rust
pub fn parse_worktree_list(output: &str) -> Vec<WorktreeInfo> {
    output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 || parts[1] == "(bare)" {
                return None;
            }
            let branch = parts[2].trim_matches(|c| c == '[' || c == ']').to_string();
            Some(WorktreeInfo {
                path: parts[0].to_string(),
                branch,
                base_branch: String::new(),
                commit_hash: parts[1].to_string(),
                commits_ahead: 0,
                is_merged: false,
                session: None,
            })
        })
        .collect()
}

pub fn parse_git_status(output: &str) -> GitStatus {
    let mut added = 0;
    let mut modified = 0;
    let mut deleted = 0;
    for line in output.lines() {
        if line.len() < 2 { continue; }
        let chars: Vec<char> = line.chars().collect();
        match (chars[0], chars[1]) {
            ('A', _) | (_, 'A') => added += 1,
            ('D', _) | (_, 'D') => deleted += 1,
            ('M', _) | (_, 'M') => modified += 1,
            _ => {}
        }
    }
    GitStatus { added, modified, deleted }
}
```

- [ ] **Step 4: テスト実行 — 成功確認**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- worktree::tests
```

Expected: 3 tests passed

- [ ] **Step 5: main.rs にモジュール追加**

```rust
mod worktree;
```

- [ ] **Step 6: コミット**

```bash
git add tools/worktree-manager/src-tauri/src/worktree.rs tools/worktree-manager/src-tauri/src/main.rs
git commit -m "feat: git worktree/status パーサー"
```

---

### Task 3b: Worktree Tauri commands

**Files:**
- Modify: `tools/worktree-manager/src-tauri/src/worktree.rs`

- [ ] **Step 1: list_worktrees コマンドを実装**

```rust
use tauri::command;

#[command]
pub fn list_worktrees(repo_path: String) -> Result<Vec<WorktreeInfo>, String> {
    let output = Command::new("git")
        .args(["worktree", "list"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| format!("git worktree list failed: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = parse_worktree_list(&stdout);
    for wt in &mut worktrees {
        wt.commits_ahead = get_commits_ahead(&repo_path, &wt.branch).unwrap_or(0);
        wt.is_merged = check_is_merged(&repo_path, &wt.branch);
        wt.base_branch = "main".to_string();
    }
    Ok(worktrees)
}

fn get_commits_ahead(repo_path: &str, branch: &str) -> Result<i32, String> {
    let output = Command::new("git")
        .args(["rev-list", "--count", &format!("main..{}", branch)])
        .current_dir(repo_path)
        .output()
        .map_err(|e| e.to_string())?;
    String::from_utf8_lossy(&output.stdout).trim().parse::<i32>().map_err(|e| e.to_string())
}

fn check_is_merged(repo_path: &str, branch: &str) -> bool {
    Command::new("git")
        .args(["branch", "--merged", "main"])
        .current_dir(repo_path)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().any(|l| l.trim() == branch))
        .unwrap_or(false)
}
```

- [ ] **Step 2: create_worktree コマンドを実装**

```rust
#[command]
pub fn create_worktree(
    repo_path: String, branch_name: String, base_branch: String,
) -> Result<WorktreeInfo, String> {
    let worktree_path = format!("{}/.worktrees/{}", repo_path, branch_name);
    let output = Command::new("git")
        .args(["worktree", "add", "-b", &branch_name, &worktree_path, &base_branch])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| format!("git worktree add failed: {}", e))?;
    if !output.status.success() {
        return Err(format!("git worktree add failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    let hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(&worktree_path)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    Ok(WorktreeInfo {
        path: worktree_path, branch: branch_name, base_branch,
        commit_hash: hash, commits_ahead: 0, is_merged: false, session: None,
    })
}
```

- [ ] **Step 3: remove_worktree, get_worktree_status, cleanup_merged を実装**

```rust
#[command]
pub fn remove_worktree(repo_path: String, worktree_path: String, delete_branch: bool) -> Result<(), String> {
    let output = Command::new("git")
        .args(["worktree", "remove", "--force", &worktree_path])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| format!("git worktree remove failed: {}", e))?;
    if !output.status.success() {
        return Err(format!("{}", String::from_utf8_lossy(&output.stderr)));
    }
    if delete_branch {
        if let Some(branch) = worktree_path.split('/').last() {
            let _ = Command::new("git").args(["branch", "-d", branch]).current_dir(&repo_path).output();
        }
    }
    Ok(())
}

#[command]
pub fn get_worktree_status(worktree_path: String) -> Result<GitStatus, String> {
    let output = Command::new("git")
        .args(["status", "--short"])
        .current_dir(&worktree_path)
        .output()
        .map_err(|e| format!("git status failed: {}", e))?;
    Ok(parse_git_status(&String::from_utf8_lossy(&output.stdout)))
}

#[command]
pub fn cleanup_merged(repo_path: String) -> Result<Vec<WorktreeInfo>, String> {
    let worktrees = list_worktrees(repo_path)?;
    Ok(worktrees.into_iter().filter(|wt| wt.is_merged).collect())
}
```

- [ ] **Step 4: 統合テストを追加**

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        Command::new("git").args(["init"]).current_dir(dir.path()).output().unwrap();
        Command::new("git").args(["commit", "--allow-empty", "-m", "initial"]).current_dir(dir.path()).output().unwrap();
        dir
    }

    #[test]
    fn test_list_worktrees_real_repo() {
        let dir = setup_test_repo();
        let result = list_worktrees(dir.path().to_string_lossy().to_string());
        assert!(result.is_ok());
        assert!(result.unwrap().len() >= 1);
    }
}
```

- [ ] **Step 5: テスト実行**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- worktree
```

Expected: 全テスト通過

- [ ] **Step 6: main.rs に Tauri command を登録**

```rust
.invoke_handler(tauri::generate_handler![
    worktree::list_worktrees,
    worktree::create_worktree,
    worktree::remove_worktree,
    worktree::get_worktree_status,
    worktree::cleanup_merged,
])
```

- [ ] **Step 7: コミット**

```bash
git add tools/worktree-manager/src-tauri/src/
git commit -m "feat: git worktree Tauri commands (CRUD + cleanup)"
```

---

## Chunk 2: Claude Code 連携

### Task 4: HookEvent テスト + Hooks HTTP サーバー

**Files:**
- Create: `tools/worktree-manager/src-tauri/src/hooks_server.rs`

- [ ] **Step 1: HookEvent パースのテストを作成**

`src-tauri/src/hooks_server.rs`:

```rust
use crate::types::HookEvent;
use axum::{extract::State, routing::post, Json, Router};
use std::sync::Arc;
use tokio::sync::broadcast;

pub type HookSender = broadcast::Sender<HookEvent>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_notification_event() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"type":"Notification","matcher":"idle_prompt","session_id":"abc123","cwd":"/tmp/repo"}"#
        ).unwrap();
        let event = HookEvent::from_value(&json).unwrap();
        match event {
            HookEvent::Notification { matcher, session_id, .. } => {
                assert_eq!(matcher, "idle_prompt");
                assert_eq!(session_id, "abc123");
            }
            _ => panic!("Expected Notification"),
        }
    }

    #[test]
    fn test_parse_pre_tool_use() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"type":"PreToolUse","tool_name":"Edit","session_id":"abc123","cwd":"/tmp"}"#
        ).unwrap();
        let event = HookEvent::from_value(&json).unwrap();
        match event {
            HookEvent::ToolUse { event_type, tool_name, .. } => {
                assert_eq!(event_type, "PreToolUse");
                assert_eq!(tool_name, "Edit");
            }
            _ => panic!("Expected ToolUse"),
        }
    }

    #[test]
    fn test_parse_post_tool_use() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"type":"PostToolUse","tool_name":"Bash","session_id":"def","cwd":"/tmp"}"#
        ).unwrap();
        let event = HookEvent::from_value(&json).unwrap();
        match event {
            HookEvent::ToolUse { event_type, .. } => assert_eq!(event_type, "PostToolUse"),
            _ => panic!("Expected ToolUse"),
        }
    }

    #[test]
    fn test_parse_stop() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"type":"Stop","session_id":"abc","cwd":"/tmp"}"#
        ).unwrap();
        let event = HookEvent::from_value(&json).unwrap();
        assert!(matches!(event, HookEvent::Stop { .. }));
    }

    #[test]
    fn test_parse_unknown_type_returns_none() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"type":"SomethingElse"}"#
        ).unwrap();
        assert!(HookEvent::from_value(&json).is_none());
    }
}
```

- [ ] **Step 2: テスト実行 — 成功確認**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- hooks_server
```

Expected: 5 tests passed（HookEvent::from_value は types.rs に既に実装済み）

- [ ] **Step 3: axum サーバーを実装**

`hooks_server.rs` に追加:

```rust
pub struct HooksServer {
    pub port: u16,
    pub sender: HookSender,
}

impl HooksServer {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { port: 0, sender }
    }

    pub async fn start(&mut self) -> Result<(), String> {
        let sender = self.sender.clone();
        let app = Router::new()
            .route("/hooks/notification", post(handle_hook))
            .route("/hooks/tool", post(handle_hook))
            .route("/hooks/lifecycle", post(handle_hook))
            .with_state(Arc::new(sender));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| format!("Failed to bind: {}", e))?;
        self.port = listener.local_addr().map_err(|e| e.to_string())?.port();

        tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });
        Ok(())
    }
}

async fn handle_hook(
    State(sender): State<Arc<HookSender>>,
    Json(body): Json<serde_json::Value>,
) -> &'static str {
    if let Some(event) = HookEvent::from_value(&body) {
        let _ = sender.send(event);
    }
    "ok"
}
```

- [ ] **Step 4: ビルド確認**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: 成功

- [ ] **Step 5: コミット**

```bash
git add tools/worktree-manager/src-tauri/src/hooks_server.rs
git commit -m "feat: Claude Code hooks HTTP サーバー (axum)"
```

---

### Task 5: settings.json 自動設定 (hooks_config.rs)

**Files:**
- Create: `tools/worktree-manager/src-tauri/src/hooks_config.rs`

- [ ] **Step 1: テストを作成**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hooks_config() {
        let config = generate_hooks_config(12345);
        let hooks = config.get("hooks").unwrap();
        let notification = hooks.get("Notification").unwrap();
        assert!(notification.is_array());
        // URL にポート番号が含まれることを確認
        let url = notification[0]["hooks"][0]["url"].as_str().unwrap();
        assert!(url.contains("12345"));
    }

    #[test]
    fn test_merge_hooks_into_existing_settings() {
        let existing = serde_json::json!({
            "some_setting": true,
            "hooks": {
                "ExistingHook": [{"hooks": [{"type": "command", "command": "echo hi"}]}]
            }
        });
        let result = merge_hooks_config(existing, 9999);
        // 既存設定が保持されていること
        assert_eq!(result["some_setting"], true);
        // 既存 hooks が保持されていること
        assert!(result["hooks"]["ExistingHook"].is_array());
        // 新しい hooks が追加されていること
        assert!(result["hooks"]["Notification"].is_array());
    }
}
```

- [ ] **Step 2: テスト実行 — 失敗確認**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- hooks_config
```

Expected: FAIL

- [ ] **Step 3: hooks_config.rs を実装**

```rust
use serde_json::{json, Value};
use std::path::PathBuf;

pub fn get_settings_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".claude").join("settings.json")
}

pub fn generate_hooks_config(port: u16) -> Value {
    let base_url = format!("http://127.0.0.1:{}", port);
    json!({
        "hooks": {
            "Notification": [
                {
                    "matcher": "idle_prompt",
                    "hooks": [{"type": "http", "url": format!("{}/hooks/notification", base_url), "method": "POST"}]
                },
                {
                    "matcher": "permission_prompt",
                    "hooks": [{"type": "http", "url": format!("{}/hooks/notification", base_url), "method": "POST"}]
                }
            ],
            "PreToolUse": [{"hooks": [{"type": "http", "url": format!("{}/hooks/tool", base_url), "method": "POST"}]}],
            "PostToolUse": [{"hooks": [{"type": "http", "url": format!("{}/hooks/tool", base_url), "method": "POST"}]}],
            "Stop": [{"hooks": [{"type": "http", "url": format!("{}/hooks/lifecycle", base_url), "method": "POST"}]}]
        }
    })
}

pub fn merge_hooks_config(mut existing: Value, port: u16) -> Value {
    let new_hooks = generate_hooks_config(port);
    let hooks_obj = existing
        .as_object_mut()
        .unwrap()
        .entry("hooks")
        .or_insert_with(|| json!({}));

    if let (Some(existing_hooks), Some(new)) = (hooks_obj.as_object_mut(), new_hooks["hooks"].as_object()) {
        for (key, value) in new {
            if let Some(existing_array) = existing_hooks.get_mut(key).and_then(|v| v.as_array_mut()) {
                // 既存エントリを保持しつつ、新しいエントリを追加（重複URL除去）
                if let Some(new_array) = value.as_array() {
                    for new_entry in new_array {
                        let new_url = new_entry.pointer("/hooks/0/url");
                        let already_exists = existing_array.iter().any(|e| e.pointer("/hooks/0/url") == new_url);
                        if !already_exists {
                            existing_array.push(new_entry.clone());
                        }
                    }
                }
            } else {
                existing_hooks.insert(key.clone(), value.clone());
            }
        }
    }
    existing
}

/// ~/.claude/settings.json に hooks 設定を書き込む
pub fn update_settings(port: u16) -> Result<(), String> {
    let path = get_settings_path();
    let existing: Value = if path.exists() {
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or(json!({}))
    } else {
        json!({})
    };

    let updated = merge_hooks_config(existing, port);
    let content = serde_json::to_string_pretty(&updated).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}
```

- [ ] **Step 4: テスト実行 — 成功確認**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- hooks_config
```

Expected: 2 tests passed

- [ ] **Step 5: コミット**

```bash
git add tools/worktree-manager/src-tauri/src/hooks_config.rs
git commit -m "feat: settings.json hooks 自動設定"
```

---

### Task 6: JSONL ファイルウォッチャー (watcher.rs)

**Files:**
- Create: `tools/worktree-manager/src-tauri/src/watcher.rs`

- [ ] **Step 1: パース関数のテストを作成**

```rust
use crate::types::{ActivityEntry, ActivityType};
use std::path::{Path, PathBuf};

pub fn encode_project_path(path: &str) -> String { todo!() }
pub fn parse_jsonl_line(line: &str) -> Option<ActivityEntry> { todo!() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_project_path() {
        assert_eq!(encode_project_path("/Users/foo/proj"), "-Users-foo-proj");
    }

    #[test]
    fn test_parse_jsonl_line_tool_use() {
        let line = r#"{"type":"tool_use","tool_name":"Edit","timestamp":"2026-03-13T12:00:00Z"}"#;
        let entry = parse_jsonl_line(line).unwrap();
        assert_eq!(entry.tool_name.as_deref(), Some("Edit"));
    }

    #[test]
    fn test_parse_jsonl_line_unknown_returns_none() {
        assert!(parse_jsonl_line(r#"{"type":"unknown"}"#).is_none());
    }
}
```

- [ ] **Step 2: テスト実行 — 失敗確認**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- watcher::tests
```

Expected: FAIL

- [ ] **Step 3: パース関数を実装**

```rust
pub fn encode_project_path(path: &str) -> String {
    path.replace('/', "-")
}

pub fn parse_jsonl_line(line: &str) -> Option<ActivityEntry> {
    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    let entry_type = value.get("type")?.as_str()?;
    match entry_type {
        "tool_use" | "tool_result" => Some(ActivityEntry {
            timestamp: value.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            entry_type: ActivityType::ToolUse,
            tool_name: value.get("tool_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
            summary: value.get("tool_name").and_then(|v| v.as_str()).unwrap_or("tool").to_string(),
        }),
        "assistant" | "user" => Some(ActivityEntry {
            timestamp: value.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            entry_type: ActivityType::Message,
            tool_name: None,
            summary: "message".to_string(),
        }),
        _ => None,
    }
}
```

- [ ] **Step 4: テスト実行 — 成功確認**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- watcher::tests
```

Expected: 3 tests passed

- [ ] **Step 5: ファイル監視関数を追加**

```rust
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::broadcast;

pub fn get_claude_projects_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".claude").join("projects")
}

pub fn find_latest_session_file(session_dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(session_dir).ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "jsonl").unwrap_or(false))
        .max_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()))
        .map(|e| e.path())
}

pub fn watch_session_file(
    path: PathBuf, sender: broadcast::Sender<ActivityEntry>,
) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| format!("Watcher creation failed: {}", e))?;
    watcher.watch(&path, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Watch failed: {}", e))?;

    std::thread::spawn(move || {
        let _watcher = watcher;
        let mut last_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        for res in rx {
            if let Ok(Event { kind: notify::EventKind::Modify(_), .. }) = res {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let current_size = content.len() as u64;
                    if current_size > last_size {
                        for line in content[last_size as usize..].lines() {
                            if let Some(entry) = parse_jsonl_line(line) {
                                let _ = sender.send(entry);
                            }
                        }
                        last_size = current_size;
                    }
                }
            }
        }
    });
    Ok(())
}
```

- [ ] **Step 6: コミット**

```bash
git add tools/worktree-manager/src-tauri/src/watcher.rs
git commit -m "feat: JSONL セッションファイルウォッチャー"
```

---

### Task 7: セッションマネージャー (session_manager.rs)

hooks/watcher からのイベントをセッション状態に変換し、フロントエンドに emit する中核モジュール。

**Files:**
- Create: `tools/worktree-manager/src-tauri/src/session_manager.rs`

- [ ] **Step 1: 状態遷移のテストを作成**

```rust
use crate::types::{HookEvent, SessionInfo, SessionState, ActivityEntry};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct SessionManager {
    pub sessions: Arc<Mutex<HashMap<String, SessionInfo>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self { sessions: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn handle_hook_event(&self, event: &HookEvent) -> Option<SessionInfo> {
        todo!()
    }

    pub fn question_count(&self) -> usize {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_sets_question_state() {
        let mgr = SessionManager::new();
        let event = HookEvent::Notification {
            matcher: "idle_prompt".into(), session_id: "s1".into(), cwd: "/tmp".into(),
        };
        let info = mgr.handle_hook_event(&event).unwrap();
        assert_eq!(info.state, SessionState::Question);
    }

    #[test]
    fn test_tool_use_sets_working_state() {
        let mgr = SessionManager::new();
        let event = HookEvent::ToolUse {
            event_type: "PreToolUse".into(), tool_name: "Edit".into(),
            session_id: "s1".into(), cwd: "/tmp".into(),
        };
        let info = mgr.handle_hook_event(&event).unwrap();
        assert_eq!(info.state, SessionState::Working);
        assert_eq!(info.current_tool.as_deref(), Some("Edit"));
    }

    #[test]
    fn test_stop_sets_idle_state() {
        let mgr = SessionManager::new();
        // まず working にする
        mgr.handle_hook_event(&HookEvent::ToolUse {
            event_type: "PreToolUse".into(), tool_name: "Edit".into(),
            session_id: "s1".into(), cwd: "/tmp".into(),
        });
        let event = HookEvent::Stop { session_id: "s1".into(), cwd: "/tmp".into() };
        let info = mgr.handle_hook_event(&event).unwrap();
        assert_eq!(info.state, SessionState::Idle);
    }

    #[test]
    fn test_question_count() {
        let mgr = SessionManager::new();
        mgr.handle_hook_event(&HookEvent::Notification {
            matcher: "idle_prompt".into(), session_id: "s1".into(), cwd: "/a".into(),
        });
        mgr.handle_hook_event(&HookEvent::Notification {
            matcher: "permission_prompt".into(), session_id: "s2".into(), cwd: "/b".into(),
        });
        assert_eq!(mgr.question_count(), 2);
    }
}
```

- [ ] **Step 2: テスト実行 — 失敗確認**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- session_manager
```

Expected: FAIL

- [ ] **Step 3: SessionManager を実装**

```rust
impl SessionManager {
    pub fn new() -> Self {
        Self { sessions: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn handle_hook_event(&self, event: &HookEvent) -> Option<SessionInfo> {
        let mut sessions = self.sessions.lock().ok()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();

        let (session_id, new_state, tool) = match event {
            HookEvent::Notification { session_id, matcher, .. } => {
                (session_id.clone(), SessionState::Question, None)
            }
            HookEvent::ToolUse { session_id, tool_name, .. } => {
                (session_id.clone(), SessionState::Working, Some(tool_name.clone()))
            }
            HookEvent::Stop { session_id, .. } => {
                (session_id.clone(), SessionState::Idle, None)
            }
        };

        let entry = sessions.entry(session_id.clone()).or_insert_with(|| SessionInfo {
            session_id: session_id.clone(),
            state: SessionState::Idle,
            started_at: now.clone(),
            last_activity: now.clone(),
            current_tool: None,
        });

        entry.state = new_state;
        entry.last_activity = now;
        entry.current_tool = tool;
        Some(entry.clone())
    }

    pub fn question_count(&self) -> usize {
        self.sessions.lock()
            .map(|s| s.values().filter(|s| s.state == SessionState::Question).count())
            .unwrap_or(0)
    }

    pub fn get_session(&self, session_id: &str) -> Option<SessionInfo> {
        self.sessions.lock().ok()?.get(session_id).cloned()
    }

    /// セッションのアクティビティログを追加
    pub fn add_activity(&self, session_id: &str, entry: ActivityEntry) {
        let mut activities = self.activities.lock().unwrap_or_else(|e| e.into_inner());
        activities.entry(session_id.to_string()).or_default().push(entry);
    }

    /// セッションのアクティビティログを取得
    pub fn get_activity_log(&self, session_id: &str, limit: Option<usize>) -> Vec<ActivityEntry> {
        let activities = self.activities.lock().unwrap_or_else(|e| e.into_inner());
        let entries = activities.get(session_id).cloned().unwrap_or_default();
        let limit = limit.unwrap_or(50);
        entries.into_iter().rev().take(limit).collect()
    }
}
```

注意: `SessionManager` の struct に `activities` フィールドを追加:

```rust
pub struct SessionManager {
    pub sessions: Arc<Mutex<HashMap<String, SessionInfo>>>,
    pub activities: Arc<Mutex<HashMap<String, Vec<ActivityEntry>>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            activities: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    // ... 以下同じ
}
```

- [ ] **Step 4: Tauri commands を追加**

```rust
use tauri::command;

#[command]
pub fn get_session_status(
    session_id: String,
    state: tauri::State<'_, Arc<SessionManager>>,
) -> Result<SessionInfo, String> {
    state.get_session(&session_id).ok_or("Session not found".into())
}

#[command]
pub fn get_activity_log(
    session_id: String,
    limit: Option<usize>,
    state: tauri::State<'_, Arc<SessionManager>>,
) -> Result<Vec<ActivityEntry>, String> {
    Ok(state.get_activity_log(&session_id, limit))
}
```

- [ ] **Step 5: テスト実行 — 成功確認**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- session_manager
```

Expected: 4 tests passed

- [ ] **Step 6: コミット**

```bash
git add tools/worktree-manager/src-tauri/src/session_manager.rs
git commit -m "feat: セッションマネージャー (状態遷移 + question count)"
```

---

### Task 8: Claude Code ランチャー (claude.rs)

**Files:**
- Create: `tools/worktree-manager/src-tauri/src/claude.rs`

- [ ] **Step 1: ヘルパー関数のテストを作成**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escape() {
        assert_eq!(shell_escape("hello"), "hello");
        assert_eq!(shell_escape("it's"), "it'\\''s");
    }

    #[test]
    fn test_build_claude_command_no_prompt() {
        let cmd = build_claude_command("/tmp/repo", None);
        assert_eq!(cmd, "cd '/tmp/repo' && claude");
    }

    #[test]
    fn test_build_claude_command_with_prompt() {
        let cmd = build_claude_command("/tmp/repo", Some("fix bug"));
        assert_eq!(cmd, "cd '/tmp/repo' && claude -p 'fix bug'");
    }
}
```

- [ ] **Step 2: claude.rs を実装**

```rust
use crate::types::{SessionInfo, SessionState};
use std::process::Command;
use tauri::command;

fn shell_escape(s: &str) -> String {
    s.replace('\'', "'\\''")
}

fn build_claude_command(worktree_path: &str, initial_prompt: Option<&str>) -> String {
    let escaped_path = shell_escape(worktree_path);
    match initial_prompt {
        Some(prompt) => format!("cd '{}' && claude -p '{}'", escaped_path, shell_escape(prompt)),
        None => format!("cd '{}' && claude", escaped_path),
    }
}

fn now_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

#[command]
pub fn launch_claude(
    worktree_path: String, initial_prompt: Option<String>,
) -> Result<SessionInfo, String> {
    let shell_cmd = build_claude_command(&worktree_path, initial_prompt.as_deref());

    // Ghostty で新しいウィンドウを開いて claude を実行
    let applescript = format!(
        r#"tell application "Ghostty"
            activate
            tell application "System Events"
                keystroke "t" using command down
                delay 0.5
                keystroke "{}"
                key code 36
            end tell
        end tell"#,
        shell_cmd.replace('"', "\\\"")
    );

    Command::new("osascript")
        .args(["-e", &applescript])
        .spawn()
        .map_err(|e| format!("Failed to launch claude: {}", e))?;

    Ok(SessionInfo {
        session_id: format!("pending-{}", now_timestamp()),
        state: SessionState::Working,
        started_at: now_timestamp(),
        last_activity: now_timestamp(),
        current_tool: None,
    })
}
```

- [ ] **Step 3: テスト実行**

```bash
cargo test --manifest-path src-tauri/Cargo.toml -- claude::tests
```

Expected: 3 tests passed

- [ ] **Step 4: コミット**

```bash
git add tools/worktree-manager/src-tauri/src/claude.rs
git commit -m "feat: Claude Code ランチャー (Ghostty 連携)"
```

---

### Task 9: 外部ツールランチャー (external.rs)

**Files:**
- Create: `tools/worktree-manager/src-tauri/src/external.rs`

- [ ] **Step 1: external.rs を実装**

```rust
use std::process::Command;
use tauri::command;

#[command]
pub fn open_in_ide(worktree_path: String) -> Result<(), String> {
    for editor in &["cursor", "code"] {
        if Command::new("which").arg(editor).output().map(|o| o.status.success()).unwrap_or(false) {
            Command::new(editor).arg(&worktree_path).spawn()
                .map_err(|e| format!("Failed to open {}: {}", editor, e))?;
            return Ok(());
        }
    }
    Err("No IDE found (tried cursor, code)".to_string())
}

#[command]
pub fn open_difftool(worktree_path: String) -> Result<(), String> {
    Command::new("git").args(["difftool", "--dir-diff"]).current_dir(&worktree_path).spawn()
        .map_err(|e| format!("Failed to open difftool: {}", e))?;
    Ok(())
}

#[command]
pub fn open_terminal(worktree_path: String) -> Result<(), String> {
    Command::new("open").args(["-a", "Ghostty", &worktree_path]).spawn()
        .map_err(|e| format!("Failed to open Ghostty: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_editor_preference_order() {
        let editors = ["cursor", "code"];
        assert_eq!(editors[0], "cursor");
        assert_eq!(editors[1], "code");
    }
}
```

- [ ] **Step 3: コミット**

```bash
git add tools/worktree-manager/src-tauri/src/external.rs
git commit -m "feat: 外部ツールランチャー (IDE, difftool, terminal)"
```

---

### Task 10: main.rs 統合 — 全モジュール + Tray 空ファイル

**Files:**
- Modify: `tools/worktree-manager/src-tauri/src/main.rs`
- Create: `tools/worktree-manager/src-tauri/src/tray.rs` (空)

- [ ] **Step 1: 空の tray.rs を作成**

```rust
// Tray implementation — Task 15 で実装
```

- [ ] **Step 2: main.rs を更新**

```rust
mod claude;
mod external;
mod hooks_config;
mod hooks_server;
mod session_manager;
mod tray;
mod types;
mod watcher;
mod worktree;

use hooks_server::HooksServer;
use session_manager::SessionManager;
use std::sync::Arc;

fn main() {
    let session_mgr = Arc::new(SessionManager::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(session_mgr.clone())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let mgr = session_mgr.clone();

            tauri::async_runtime::spawn(async move {
                // hooks サーバー起動
                let mut server = HooksServer::new();
                if let Err(e) = server.start().await {
                    eprintln!("Failed to start hooks server: {}", e);
                    return;
                }
                println!("Hooks server started on port {}", server.port);

                // settings.json にポートを書き込む
                if let Err(e) = hooks_config::update_settings(server.port) {
                    eprintln!("Failed to update settings.json: {}", e);
                }

                // hooks イベントをセッション状態に変換 → フロントエンドに emit
                let mut rx = server.sender.subscribe();
                while let Ok(event) = rx.recv().await {
                    if let Some(session_info) = mgr.handle_hook_event(&event) {
                        let _ = app_handle.emit("session-update", &session_info);
                    }
                    // バッジ更新
                    let count = mgr.question_count();
                    let _ = app_handle.emit("badge-update", count);
                }
            });

            // JSONL watcher: セッションファイルを監視してアクティビティログに追加
            // (hooks の補完として機能。hooks が利用不可でもログは取得可能)
            let mgr_watcher = session_mgr.clone();
            let app_watcher = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // リポジトリパスはフロントエンドから設定された後に開始
                // listen for "start-watching" event from frontend
                // 初期版では hooks 経由のみで十分。
                // watcher モジュールの関数は将来的にここから呼び出す:
                // watcher::watch_session_file(path, sender) で ActivityEntry を受信
                // → mgr_watcher.add_activity(session_id, entry)
                // → app_watcher.emit("activity-update", entry)
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            worktree::list_worktrees,
            worktree::create_worktree,
            worktree::remove_worktree,
            worktree::get_worktree_status,
            worktree::cleanup_merged,
            claude::launch_claude,
            session_manager::get_session_status,
            session_manager::get_activity_log,
            external::open_in_ide,
            external::open_difftool,
            external::open_terminal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: dialog プラグイン追加**

```bash
cd /Users/hikaru_masuta/Documents/GitHub/ghostty-customs/tools/worktree-manager
npm install @tauri-apps/plugin-dialog
cargo add tauri-plugin-dialog --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 4: ビルド確認**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: 成功

- [ ] **Step 5: 全テスト実行**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: 全テスト通過

- [ ] **Step 6: コミット**

```bash
git add tools/worktree-manager/src-tauri/
git commit -m "feat: main.rs 統合 — 全モジュール + hooks→session→frontend パイプライン"
```

---

## Chunk 3: フロントエンド (SvelteKit UI)

### Task 11: Tokyo Night テーマ

**Files:**
- Modify: `tools/worktree-manager/src/app.css`

- [ ] **Step 1: app.css に Tokyo Night テーマを定義**

```css
@import "tailwindcss";

@theme {
  --color-tn-bg: #1a1b26;
  --color-tn-bg-dark: #16161e;
  --color-tn-bg-highlight: #24283b;
  --color-tn-bg-alt: #1e2030;
  --color-tn-fg: #c0caf5;
  --color-tn-fg-dim: #a9b1d6;
  --color-tn-fg-muted: #565f89;
  --color-tn-fg-dark: #787c99;
  --color-tn-border: #2a2b3d;
  --color-tn-blue: #7aa2f7;
  --color-tn-green: #9ece6a;
  --color-tn-yellow: #e0af68;
  --color-tn-red: #f7768e;
  --color-tn-purple: #bb9af7;
  --font-family-mono: "JetBrains Mono", "Fira Code", "SF Mono", ui-monospace, monospace;
}

body {
  background-color: var(--color-tn-bg);
  color: var(--color-tn-fg);
  font-family: var(--font-family-mono);
  font-size: 13px;
  margin: 0;
  overflow: hidden;
  user-select: none;
}

::-webkit-scrollbar { width: 6px; }
::-webkit-scrollbar-track { background: var(--color-tn-bg); }
::-webkit-scrollbar-thumb { background: var(--color-tn-fg-muted); border-radius: 3px; }
```

- [ ] **Step 2: コミット**

```bash
git add tools/worktree-manager/src/app.css
git commit -m "feat: Tokyo Night テーマ定義"
```

---

### Task 12: Svelte Stores

**Files:**
- Create: `tools/worktree-manager/src/lib/stores/worktrees.ts`
- Create: `tools/worktree-manager/src/lib/stores/sessions.ts`

- [ ] **Step 1: worktrees store を作成**

`src/lib/stores/worktrees.ts`:

```typescript
import { writable, derived, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { WorktreeInfo } from "$lib/types";

export const worktrees = writable<WorktreeInfo[]>([]);
export const selectedWorktree = writable<WorktreeInfo | null>(null);
export const repoPath = writable<string>("");

export const mergedWorktrees = derived(worktrees, ($wt) => $wt.filter((w) => w.is_merged));

export async function loadWorktrees(path: string) {
  repoPath.set(path);
  const result = await invoke<WorktreeInfo[]>("list_worktrees", { repoPath: path });
  worktrees.set(result);
}

export async function createWorktree(branchName: string, baseBranch: string): Promise<WorktreeInfo> {
  const path = get(repoPath);
  const result = await invoke<WorktreeInfo>("create_worktree", { repoPath: path, branchName, baseBranch });
  await loadWorktrees(path);
  return result;
}

export async function removeWorktree(worktreePath: string, deleteBranch: boolean) {
  const path = get(repoPath);
  await invoke("remove_worktree", { repoPath: path, worktreePath, deleteBranch });
  await loadWorktrees(path);
}
```

- [ ] **Step 2: sessions store を作成**

`src/lib/stores/sessions.ts`:

```typescript
import { writable, derived } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import type { SessionInfo } from "$lib/types";

export const sessions = writable<Map<string, SessionInfo>>(new Map());

export const questionCount = derived(sessions, ($s) => {
  let count = 0;
  for (const session of $s.values()) {
    if (session.state === "question") count++;
  }
  return count;
});

export function initSessionListener() {
  listen<SessionInfo>("session-update", (event) => {
    sessions.update((map) => {
      map.set(event.payload.session_id, event.payload);
      return new Map(map);
    });
  });
}
```

- [ ] **Step 3: コミット**

```bash
git add tools/worktree-manager/src/lib/stores/
git commit -m "feat: Svelte stores (worktrees + sessions)"
```

---

### Task 13a: WorktreeList コンポーネント

**Files:**
- Create: `tools/worktree-manager/src/lib/components/WorktreeList.svelte`

- [ ] **Step 1: WorktreeList.svelte を作成**

```svelte
<script lang="ts">
  import type { WorktreeInfo } from "$lib/types";
  import { worktrees, selectedWorktree } from "$lib/stores/worktrees";

  interface Props {
    onnewclick: () => void;
    oncleanupclick: () => void;
  }
  let { onnewclick, oncleanupclick }: Props = $props();

  function stateColor(wt: WorktreeInfo): string {
    if (wt.is_merged) return "text-tn-purple";
    if (!wt.session) return "text-tn-fg-muted";
    switch (wt.session.state) {
      case "working": return "text-tn-green";
      case "question": return "text-tn-red";
      case "idle": return "text-tn-fg-muted";
      default: return "text-tn-fg-muted";
    }
  }

  function stateLabel(wt: WorktreeInfo): string {
    if (wt.is_merged) return "✓ MERGED";
    if (!wt.session) return "○ IDLE";
    switch (wt.session.state) {
      case "working": return "● WORKING";
      case "question": return "⚡ QUESTION";
      case "idle": return "○ IDLE";
      default: return "○ IDLE";
    }
  }

  function borderColor(wt: WorktreeInfo): string {
    if (wt.is_merged) return "border-l-tn-purple";
    if (!wt.session) return "border-l-tn-fg-muted";
    switch (wt.session.state) {
      case "working": return "border-l-tn-green";
      case "question": return "border-l-tn-red";
      default: return "border-l-tn-fg-muted";
    }
  }

  function claudeStatus(wt: WorktreeInfo): string {
    if (!wt.session) return "";
    switch (wt.session.state) {
      case "working": return `Claude: 作業中${wt.session.current_tool ? ` (${wt.session.current_tool})` : ""}`;
      case "question": return "Claude: 質問待ち";
      case "idle": return "Claude: 完了";
      default: return "";
    }
  }
</script>

<div class="w-[280px] border-r border-tn-border p-3 flex-shrink-0 overflow-y-auto h-full">
  <div class="flex justify-between items-center mb-3">
    <span class="text-tn-blue font-bold text-[13px]">WORKTREES</span>
    <button class="bg-tn-blue text-tn-bg px-2 py-0.5 rounded text-[11px] cursor-pointer hover:opacity-80" onclick={onnewclick}>+ New</button>
  </div>

  {#each $worktrees as wt}
    <button
      class="w-full text-left bg-tn-bg-highlight border-l-[3px] {borderColor(wt)} p-2 mb-1.5 rounded-r cursor-pointer hover:bg-tn-bg-alt transition-colors"
      class:opacity-70={wt.is_merged}
      onclick={() => selectedWorktree.set(wt)}
    >
      <div class="flex justify-between">
        <span class="text-tn-fg text-[12px] font-bold" class:line-through={wt.is_merged}>{wt.branch}</span>
        <span class="{stateColor(wt)} text-[10px]">{stateLabel(wt)}</span>
      </div>
      <div class="text-tn-fg-muted text-[10px] mt-0.5">{wt.path}</div>
      {#if wt.session}
        <div class="{stateColor(wt)} text-[10px] mt-0.5">{claudeStatus(wt)}</div>
      {/if}
    </button>
  {/each}

  {#if $worktrees.some(wt => wt.is_merged)}
    <div class="border-t border-tn-border mt-3 pt-2">
      <button class="text-tn-red text-[11px] cursor-pointer hover:underline" onclick={oncleanupclick}>
        🧹 クリーンアップ ({$worktrees.filter(wt => wt.is_merged).length})
      </button>
    </div>
  {/if}
</div>
```

- [ ] **Step 2: コミット**

```bash
git add tools/worktree-manager/src/lib/components/WorktreeList.svelte
git commit -m "feat: WorktreeList コンポーネント"
```

---

### Task 13b: SessionStatus コンポーネント

**Files:**
- Create: `tools/worktree-manager/src/lib/components/SessionStatus.svelte`

- [ ] **Step 1: SessionStatus.svelte を作成**

```svelte
<script lang="ts">
  import type { SessionInfo } from "$lib/types";

  interface Props { session: SessionInfo | null; }
  let { session }: Props = $props();

  function stateColor(state: string): string {
    switch (state) {
      case "working": return "text-tn-green";
      case "question": return "text-tn-red";
      default: return "text-tn-fg-muted";
    }
  }

  function stateText(state: string): string {
    switch (state) {
      case "working": return "● 作業中";
      case "question": return "⚡ 質問待ち";
      case "idle": return "○ 完了";
      default: return "—";
    }
  }

  function elapsed(startedAt: string): string {
    const start = new Date(startedAt).getTime();
    const mins = Math.floor((Date.now() - start) / 60000);
    if (mins < 60) return `${mins}分経過`;
    return `${Math.floor(mins / 60)}時間${mins % 60}分経過`;
  }
</script>

{#if session}
  <div class="bg-tn-bg-highlight rounded p-2.5 mb-3">
    <div class="flex justify-between items-center">
      <span class="{stateColor(session.state)} text-[12px]">{stateText(session.state)} — Claude Code</span>
      <span class="text-tn-fg-muted text-[10px]">セッション: {elapsed(session.started_at)}</span>
    </div>
    {#if session.current_tool}
      <div class="text-tn-fg-dim text-[11px] mt-1.5 p-1.5 bg-tn-bg rounded">&gt; {session.current_tool} 実行中...</div>
    {/if}
  </div>
{:else}
  <div class="bg-tn-bg-highlight rounded p-2.5 mb-3">
    <span class="text-tn-fg-muted text-[12px]">Claude Code セッションなし</span>
  </div>
{/if}
```

- [ ] **Step 2: コミット**

```bash
git add tools/worktree-manager/src/lib/components/SessionStatus.svelte
git commit -m "feat: SessionStatus コンポーネント"
```

---

### Task 13c: ActivityLog コンポーネント

**Files:**
- Create: `tools/worktree-manager/src/lib/components/ActivityLog.svelte`

- [ ] **Step 1: ActivityLog.svelte を作成**

```svelte
<script lang="ts">
  import type { ActivityEntry } from "$lib/types";

  interface Props { entries: ActivityEntry[]; }
  let { entries }: Props = $props();

  function typeColor(type: string): string {
    switch (type) {
      case "tool_use": return "text-tn-green";
      case "message": return "text-tn-blue";
      case "error": return "text-tn-red";
      default: return "text-tn-fg-muted";
    }
  }

  function typeLabel(type: string): string {
    switch (type) { case "tool_use": return "Tool"; case "message": return "Msg"; case "error": return "Err"; default: return "—"; }
  }

  function formatTime(ts: string): string {
    try { return new Date(ts).toLocaleTimeString("ja-JP", { hour: "2-digit", minute: "2-digit" }); }
    catch { return "—"; }
  }
</script>

<div class="mb-3">
  <span class="text-tn-fg-muted text-[11px] font-bold">RECENT ACTIVITY</span>
  <div class="mt-1.5 text-[11px]">
    {#each entries.slice(0, 20) as entry}
      <div class="py-1 border-b border-tn-bg-alt flex gap-1.5">
        <span class="text-tn-fg-muted w-10 flex-shrink-0">{formatTime(entry.timestamp)}</span>
        <span class="{typeColor(entry.type)} w-8 flex-shrink-0">{typeLabel(entry.type)}</span>
        <span class="text-tn-fg-dim truncate">{#if entry.tool_name}{entry.tool_name} — {/if}{entry.summary}</span>
      </div>
    {/each}
    {#if entries.length === 0}
      <div class="text-tn-fg-muted py-2">アクティビティなし</div>
    {/if}
  </div>
</div>
```

- [ ] **Step 2: コミット**

```bash
git add tools/worktree-manager/src/lib/components/ActivityLog.svelte
git commit -m "feat: ActivityLog コンポーネント"
```

---

### Task 13d: WorktreeDetail コンポーネント

**Files:**
- Create: `tools/worktree-manager/src/lib/components/WorktreeDetail.svelte`

- [ ] **Step 1: WorktreeDetail.svelte を作成**

```svelte
<script lang="ts">
  import { selectedWorktree } from "$lib/stores/worktrees";
  import { invoke } from "@tauri-apps/api/core";
  import SessionStatus from "./SessionStatus.svelte";
  import ActivityLog from "./ActivityLog.svelte";
  import type { ActivityEntry, GitStatus } from "$lib/types";

  let gitStatus = $state<GitStatus | null>(null);
  let activityLog = $state<ActivityEntry[]>([]);

  $effect(() => {
    if ($selectedWorktree) {
      invoke<GitStatus>("get_worktree_status", { worktreePath: $selectedWorktree.path })
        .then((s) => (gitStatus = s)).catch(() => (gitStatus = null));

      // アクティビティログ取得
      if ($selectedWorktree.session) {
        invoke<ActivityEntry[]>("get_activity_log", {
          sessionId: $selectedWorktree.session.session_id,
          limit: 20,
        }).then((entries) => (activityLog = entries)).catch(() => (activityLog = []));
      } else {
        activityLog = [];
      }
    }
  });

  async function openIde() {
    if ($selectedWorktree) await invoke("open_in_ide", { worktreePath: $selectedWorktree.path });
  }
  async function openDiff() {
    if ($selectedWorktree) await invoke("open_difftool", { worktreePath: $selectedWorktree.path });
  }
  async function openTerminal() {
    if ($selectedWorktree) await invoke("open_terminal", { worktreePath: $selectedWorktree.path });
  }
</script>

{#if $selectedWorktree}
  <div class="flex-1 p-3 overflow-y-auto">
    <div class="flex justify-between items-center mb-4 pb-2 border-b border-tn-border">
      <div>
        <span class="text-tn-blue text-[15px] font-bold">{$selectedWorktree.branch}</span>
        <span class="text-tn-fg-muted text-[11px] ml-2">{$selectedWorktree.commits_ahead} commits ahead</span>
      </div>
      <div class="flex gap-1.5">
        <button class="bg-tn-bg-highlight text-tn-blue px-2.5 py-1 rounded text-[11px] border border-tn-border hover:bg-tn-bg-alt cursor-pointer" onclick={openDiff}>diffit</button>
        <button class="bg-tn-bg-highlight text-tn-blue px-2.5 py-1 rounded text-[11px] border border-tn-border hover:bg-tn-bg-alt cursor-pointer" onclick={openIde}>IDE</button>
        <button class="bg-tn-bg-highlight text-tn-blue px-2.5 py-1 rounded text-[11px] border border-tn-border hover:bg-tn-bg-alt cursor-pointer" onclick={openTerminal}>Terminal</button>
      </div>
    </div>
    <SessionStatus session={$selectedWorktree.session} />
    <ActivityLog entries={activityLog} />
    <div>
      <span class="text-tn-fg-muted text-[11px] font-bold">GIT</span>
      <div class="mt-1.5 text-[11px]">
        <div class="py-0.5"><span class="text-tn-fg-muted">branch:</span> <span class="text-tn-fg">{$selectedWorktree.branch}</span></div>
        <div class="py-0.5"><span class="text-tn-fg-muted">base:</span> <span class="text-tn-fg">{$selectedWorktree.base_branch}</span></div>
        {#if gitStatus}
          <div class="py-0.5">
            <span class="text-tn-fg-muted">files:</span>
            <span class="text-tn-green">+{gitStatus.added}</span>
            <span class="text-tn-red">-{gitStatus.deleted}</span>
            <span class="text-tn-yellow">~{gitStatus.modified}</span>
          </div>
        {/if}
      </div>
    </div>
  </div>
{:else}
  <div class="flex-1 flex items-center justify-center">
    <span class="text-tn-fg-muted text-[13px]">Worktree を選択してください</span>
  </div>
{/if}
```

- [ ] **Step 2: コミット**

```bash
git add tools/worktree-manager/src/lib/components/WorktreeDetail.svelte
git commit -m "feat: WorktreeDetail コンポーネント"
```

---

### Task 14: ダイアログコンポーネント + メインページ

**Files:**
- Create: `tools/worktree-manager/src/lib/components/NewWorktreeDialog.svelte`
- Create: `tools/worktree-manager/src/lib/components/CleanupDialog.svelte`
- Modify: `tools/worktree-manager/src/routes/+page.svelte`

- [ ] **Step 1: NewWorktreeDialog.svelte を作成**

```svelte
<script lang="ts">
  import { createWorktree } from "$lib/stores/worktrees";
  import { invoke } from "@tauri-apps/api/core";

  interface Props { open: boolean; onclose: () => void; }
  let { open, onclose }: Props = $props();

  let branchName = $state("");
  let baseBranch = $state("main");
  let initialPrompt = $state("");
  let launchClaude = $state(true);
  let loading = $state(false);
  let error = $state("");

  async function submit() {
    if (!branchName.trim()) { error = "ブランチ名を入力してください"; return; }
    loading = true; error = "";
    try {
      const wt = await createWorktree(branchName.trim(), baseBranch);
      if (launchClaude) {
        await invoke("launch_claude", { worktreePath: wt.path, initialPrompt: initialPrompt.trim() || null });
      }
      branchName = ""; initialPrompt = ""; onclose();
    } catch (e: any) { error = e.toString(); }
    finally { loading = false; }
  }
</script>

{#if open}
  <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" onclick={onclose} role="dialog">
    <div class="bg-tn-bg-highlight border border-tn-border rounded-lg p-5 w-[420px]" onclick={(e) => e.stopPropagation()} role="document">
      <h2 class="text-tn-blue text-[14px] font-bold mb-4">New Worktree</h2>
      <label class="block mb-3">
        <span class="text-tn-fg-muted text-[11px] block mb-1">ブランチ名</span>
        <input type="text" bind:value={branchName} placeholder="feature/my-feature"
          class="w-full bg-tn-bg border border-tn-border rounded px-2 py-1.5 text-tn-fg text-[12px] outline-none focus:border-tn-blue" />
      </label>
      <label class="block mb-3">
        <span class="text-tn-fg-muted text-[11px] block mb-1">ベースブランチ</span>
        <input type="text" bind:value={baseBranch}
          class="w-full bg-tn-bg border border-tn-border rounded px-2 py-1.5 text-tn-fg text-[12px] outline-none focus:border-tn-blue" />
      </label>
      <label class="flex items-center gap-2 mb-3">
        <input type="checkbox" bind:checked={launchClaude} class="accent-tn-blue" />
        <span class="text-tn-fg-dim text-[11px]">Claude Code を起動する</span>
      </label>
      {#if launchClaude}
        <label class="block mb-3">
          <span class="text-tn-fg-muted text-[11px] block mb-1">初期プロンプト（任意）</span>
          <textarea bind:value={initialPrompt} placeholder="タスクの説明..." rows="3"
            class="w-full bg-tn-bg border border-tn-border rounded px-2 py-1.5 text-tn-fg text-[12px] outline-none focus:border-tn-blue resize-none"></textarea>
        </label>
      {/if}
      {#if error}<div class="text-tn-red text-[11px] mb-3">{error}</div>{/if}
      <div class="flex justify-end gap-2">
        <button class="text-tn-fg-muted text-[11px] px-3 py-1.5 hover:text-tn-fg cursor-pointer" onclick={onclose}>キャンセル</button>
        <button class="bg-tn-blue text-tn-bg text-[11px] px-3 py-1.5 rounded hover:opacity-80 cursor-pointer disabled:opacity-50"
          disabled={loading} onclick={submit}>{loading ? "作成中..." : "作成"}</button>
      </div>
    </div>
  </div>
{/if}
```

- [ ] **Step 2: CleanupDialog.svelte を作成**

```svelte
<script lang="ts">
  import { mergedWorktrees, removeWorktree } from "$lib/stores/worktrees";

  interface Props { open: boolean; onclose: () => void; }
  let { open, onclose }: Props = $props();

  let selected = $state<Set<string>>(new Set());
  let loading = $state(false);

  function toggle(path: string) {
    selected.has(path) ? selected.delete(path) : selected.add(path);
    selected = new Set(selected);
  }

  async function cleanup() {
    loading = true;
    for (const path of selected) {
      try { await removeWorktree(path, true); } catch (e) { console.error(e); }
    }
    selected = new Set(); loading = false; onclose();
  }
</script>

{#if open}
  <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" onclick={onclose} role="dialog">
    <div class="bg-tn-bg-highlight border border-tn-border rounded-lg p-5 w-[420px]" onclick={(e) => e.stopPropagation()} role="document">
      <h2 class="text-tn-red text-[14px] font-bold mb-4">クリーンアップ</h2>
      <p class="text-tn-fg-dim text-[11px] mb-3">マージ済みの worktree を削除します。ブランチも削除されます。</p>
      {#each $mergedWorktrees as wt}
        <button class="w-full text-left flex items-center gap-2 p-2 rounded hover:bg-tn-bg-alt cursor-pointer mb-1" onclick={() => toggle(wt.path)}>
          <input type="checkbox" checked={selected.has(wt.path)} class="accent-tn-red pointer-events-none" />
          <div>
            <div class="text-tn-fg text-[12px]">{wt.branch}</div>
            <div class="text-tn-fg-muted text-[10px]">{wt.path}</div>
          </div>
        </button>
      {/each}
      <div class="flex justify-end gap-2 mt-4">
        <button class="text-tn-fg-muted text-[11px] px-3 py-1.5 hover:text-tn-fg cursor-pointer" onclick={onclose}>キャンセル</button>
        <button class="bg-tn-red text-tn-bg text-[11px] px-3 py-1.5 rounded hover:opacity-80 cursor-pointer disabled:opacity-50"
          disabled={loading || selected.size === 0} onclick={cleanup}>{loading ? "削除中..." : `${selected.size} 件削除`}</button>
      </div>
    </div>
  </div>
{/if}
```

- [ ] **Step 3: +page.svelte を実装**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import WorktreeList from "$lib/components/WorktreeList.svelte";
  import WorktreeDetail from "$lib/components/WorktreeDetail.svelte";
  import NewWorktreeDialog from "$lib/components/NewWorktreeDialog.svelte";
  import CleanupDialog from "$lib/components/CleanupDialog.svelte";
  import { loadWorktrees } from "$lib/stores/worktrees";
  import { initSessionListener } from "$lib/stores/sessions";

  let showNewDialog = $state(false);
  let showCleanupDialog = $state(false);

  onMount(async () => {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({ directory: true, title: "リポジトリを選択" });
    const repoPath = (selected as string) || "";
    if (!repoPath) return;

    await loadWorktrees(repoPath);
    initSessionListener();

    const interval = setInterval(() => loadWorktrees(repoPath), 30000);
    return () => clearInterval(interval);
  });
</script>

<div class="flex h-screen bg-tn-bg">
  <WorktreeList onnewclick={() => showNewDialog = true} oncleanupclick={() => showCleanupDialog = true} />
  <WorktreeDetail />
</div>

<NewWorktreeDialog open={showNewDialog} onclose={() => showNewDialog = false} />
<CleanupDialog open={showCleanupDialog} onclose={() => showCleanupDialog = false} />
```

- [ ] **Step 4: ビルド確認**

```bash
cd /Users/hikaru_masuta/Documents/GitHub/ghostty-customs/tools/worktree-manager
npm run build
```

Expected: 成功

- [ ] **Step 5: コミット**

```bash
git add tools/worktree-manager/src/
git commit -m "feat: ダイアログ + メインページ統合"
```

---

## Chunk 4: Tray + 仕上げ

### Task 15: macOS Tray アイコン + 通知

**Files:**
- Modify: `tools/worktree-manager/src-tauri/src/tray.rs`

- [ ] **Step 1: notification プラグイン追加**

```bash
cd /Users/hikaru_masuta/Documents/GitHub/ghostty-customs/tools/worktree-manager
npm install @tauri-apps/plugin-notification
cargo add tauri-plugin-notification --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 2: tray.rs を実装**

```rust
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Worktree Manager")
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;
    Ok(())
}

/// QUESTION 状態のセッション数が変わったときに macOS 通知を送信
pub fn send_question_notification(app: &AppHandle, count: usize) {
    if count > 0 {
        use tauri_plugin_notification::NotificationExt;
        let _ = app
            .notification()
            .builder()
            .title("Worktree Manager")
            .body(format!("{} 件の質問が待っています", count))
            .show();
    }
}
```

- [ ] **Step 3: main.rs に tray + notification プラグイン追加**

`.setup()` 内に追加:

```rust
tray::setup_tray(app)?;
```

`tauri::Builder` に追加:

```rust
.plugin(tauri_plugin_notification::init())
```

hooks イベントループ内で通知送信:

```rust
let prev_count = mgr.question_count();
// ... handle_hook_event ...
let new_count = mgr.question_count();
if new_count > prev_count {
    tray::send_question_notification(&app_handle, new_count);
}
```

- [ ] **Step 4: ビルド確認**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: 成功

- [ ] **Step 5: コミット**

```bash
git add tools/worktree-manager/src-tauri/
git commit -m "feat: macOS Tray + 通知 (QUESTION 状態アラート)"
```

---

### Task 16: tauri.conf.json + ウィンドウ設定

**Files:**
- Modify: `tools/worktree-manager/src-tauri/tauri.conf.json`

- [ ] **Step 1: ウィンドウ設定を調整**

`tauri.conf.json` の `windows` セクション:

```json
{
  "label": "main",
  "title": "Worktree Manager",
  "width": 900,
  "height": 600,
  "minWidth": 700,
  "minHeight": 400,
  "decorations": true,
  "transparent": false
}
```

`"app"` セクションに `"trayIcon"` が必要なら追加。`"security"` の CSP で `connect-src` に `http://127.0.0.1:*` を許可。

- [ ] **Step 2: コミット**

```bash
git add tools/worktree-manager/src-tauri/tauri.conf.json
git commit -m "chore: ウィンドウ設定 (900x600, Worktree Manager)"
```

---

### Task 17: .gitignore + 最終ビルド

**Files:**
- Modify: `.gitignore`

- [ ] **Step 1: .gitignore に追加**

```
.superpowers/
tools/worktree-manager/src-tauri/target/
tools/worktree-manager/node_modules/
```

- [ ] **Step 2: 全テスト実行**

```bash
cd /Users/hikaru_masuta/Documents/GitHub/ghostty-customs/tools/worktree-manager
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: 全テスト通過

- [ ] **Step 3: 開発モード起動確認**

```bash
npm run tauri dev
```

Expected: アプリ起動、Tray 表示、リポジトリ選択ダイアログ表示

- [ ] **Step 4: コミット**

```bash
git add .gitignore
git commit -m "chore: .gitignore 更新 (.superpowers, Tauri build artifacts)"
```

---

## Implementation Notes

- **Claude Code hooks ペイロード**: 実際のペイロードは spec より多くのフィールドを含む（`transcript_path`, `permission_mode` 等）。`HookEvent::from_value` で必要なフィールドのみ抽出しているので互換性は維持される。
- **Ghostty セッション起動**: AppleScript はバージョン依存。動作しない場合は `open -a Terminal` にフォールバック。
- **Tauri v2 互換性**: scaffold 後に公式ドキュメントで API 変更を確認。
- **settings.json の競合**: 既存の hooks 設定がある場合、`merge_hooks_config` で既存エントリを保持しつつ上書きする。アプリ終了時に元に戻す処理は Phase 2 で検討。
