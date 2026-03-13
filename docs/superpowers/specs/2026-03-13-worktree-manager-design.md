# Worktree Manager - Design Spec

Git worktree の管理と Claude Code セッション監視を行う Tauri デスクトップアプリケーション。

## Problem

- TUI に馴染めない。GUI で直感的に操作したい
- モニタ1枚でペイン分割は窮屈。1つのコントロールパネルで全体を俯瞰したい
- 並列作業中の Claude Code の進捗・質問をリアルタイムで把握したい
- worktree のライフサイクル管理が面倒で不要データが溜まる
- diff 確認は diffit / IDE に委譲したい（特に markdown）

## Solution

macOS メニューバー常駐 + フルウィンドウのハイブリッド型コントロールパネル。

- **メニューバー Tray**: ステータスアイコン + バッジ通知。クリックでドロップダウン（worktree 一覧 + 状態）。
- **メインウィンドウ**: 左に worktree リスト、右に選択 worktree の詳細（Claude 状態、アクティビティ、Git 情報、外部ツール起動）。

普段は Tray でチラ見、操作時にフルウィンドウを開く。

## Architecture

```
┌─────────────────────────────────────┐
│           Tauri App                 │
│                                     │
│  ┌──────────┐    ┌───────────────┐  │
│  │  Rust    │◄──►│  WebView      │  │
│  │  Backend │    │  (SvelteKit)  │  │
│  └────┬─────┘    └───────────────┘  │
│       │                             │
└───────┼─────────────────────────────┘
        │
        ▼
┌───────────────────┐
│  System Layer     │
├───────────────────┤
│ • git CLI         │  worktree CRUD
│ • Claude hooks    │  HTTP 受信 (localhost)
│ • JSONL watcher   │  セッションファイル監視
│ • Process spawn   │  Claude Code 起動
│ • open コマンド   │  diffit / IDE 連携
└───────────────────┘
```

### Communication Flow

1. Claude Code hooks → HTTP POST → Rust バックエンド → Tauri event → フロントエンド更新
2. フロントエンド操作 → Tauri command → Rust → git / claude CLI 実行

### Rust Backend Modules

| Module | Responsibility |
|--------|---------------|
| `main.rs` | エントリポイント、Tauri app builder |
| `tray.rs` | メニューバー Tray 制御、バッジ更新 |
| `worktree.rs` | `git worktree add/remove/list` ラッパー、ブランチ状態取得 |
| `claude.rs` | Claude Code CLI 起動、プロセス管理 |
| `hooks_server.rs` | hooks HTTP 受信サーバー（axum） |
| `watcher.rs` | `~/.claude/projects/` 下の JSONL ファイル監視 |

### Claude Code Monitoring

Ref: https://docs.anthropic.com/en/docs/claude-code/hooks

2つの経路で状態を取得:

**Push (hooks)**: `~/.claude/settings.json` に HTTP hooks を設定。以下のイベントを受信:
- `Notification` (matcher: `idle_prompt`) → 入力待ち
- `Notification` (matcher: `permission_prompt`) → 許可待ち
- `PreToolUse` / `PostToolUse` → ツール実行中
- `Stop` → セッション完了
- `SessionStart` / `SessionEnd` → セッションライフサイクル

**settings.json hooks 設定例:**

```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "idle_prompt",
        "hooks": [{
          "type": "http",
          "url": "http://127.0.0.1:{{PORT}}/hooks/notification",
          "method": "POST"
        }]
      },
      {
        "matcher": "permission_prompt",
        "hooks": [{
          "type": "http",
          "url": "http://127.0.0.1:{{PORT}}/hooks/notification",
          "method": "POST"
        }]
      }
    ],
    "PreToolUse": [{
      "hooks": [{
        "type": "http",
        "url": "http://127.0.0.1:{{PORT}}/hooks/tool",
        "method": "POST"
      }]
    }],
    "PostToolUse": [{
      "hooks": [{
        "type": "http",
        "url": "http://127.0.0.1:{{PORT}}/hooks/tool",
        "method": "POST"
      }]
    }],
    "Stop": [{
      "hooks": [{
        "type": "http",
        "url": "http://127.0.0.1:{{PORT}}/hooks/lifecycle",
        "method": "POST"
      }]
    }]
  }
}
```

`{{PORT}}` はアプリ起動時に決定し、`settings.json` を自動更新する。

**Hooks HTTP Server (axum) エンドポイント:**

| Endpoint | Method | Body | Purpose |
|----------|--------|------|---------|
| `/hooks/notification` | POST | `{ "type": "Notification", "matcher": "idle_prompt\|permission_prompt", "session_id": "...", "cwd": "..." }` | 質問/許可待ち検知 |
| `/hooks/tool` | POST | `{ "type": "PreToolUse\|PostToolUse", "tool_name": "...", "session_id": "...", "cwd": "..." }` | ツール実行状態 |
| `/hooks/lifecycle` | POST | `{ "type": "Stop", "session_id": "...", "cwd": "..." }` | セッション終了 |

サーバーは `127.0.0.1` のみにバインドする（外部からのアクセスを遮断）。

**Fallback**: hooks が利用不可の場合、JSONL watch のみで動作する（リアルタイム性は下がるが機能は維持）。

**Pull (JSONL watch)**: `~/.claude/projects/<encoded-path>/<session-id>.jsonl` を `notify` crate で監視。`<encoded-path>` はプロジェクトパスの `-` 区切りエンコード（例: `/Users/foo/proj` → `-Users-foo-proj`）。各行の `type` フィールドからアクティビティログを構築。

**複数セッション**: 同一 worktree に複数セッションが存在する場合、最新のセッション（最も新しい JSONL ファイル）のみを追跡する。

### Session States

```
            SessionStart
                │
                ▼
  ┌──────── WORKING ◄────────┐
  │             │             │
  │     idle_prompt /         │
  │     permission_prompt     │
  │             │         PreToolUse /
  │             ▼         PostToolUse
  │         QUESTION ─────────┘
  │
  │  Stop / SessionEnd
  │             │
  │             ▼
  └────────► IDLE
                │
        branch merged?
                │
                ▼
             MERGED
```

| State | Color | Trigger | 初期状態 |
|-------|-------|---------|----------|
| WORKING | 緑 `#9ece6a` | PreToolUse / PostToolUse | セッション起動直後 |
| QUESTION | 赤 `#f7768e` | idle_prompt / permission_prompt | — |
| IDLE | グレー `#565f89` | Stop / SessionEnd | セッションなしの worktree |
| MERGED | 紫 `#bb9af7` | ブランチがベースにマージ済み | — |

## Tech Stack

### Rust Backend

| Crate | Purpose |
|-------|---------|
| `tauri` v2 | アプリフレームワーク + Tray |
| `notify` | ファイルシステム監視 |
| `axum` | hooks HTTP サーバー |
| `serde` / `serde_json` | JSON シリアライゼーション |
| `tokio` | 非同期ランタイム |

### Frontend

| Technology | Purpose |
|------------|---------|
| SvelteKit | UI フレームワーク |
| TypeScript | 型安全 |
| Tailwind CSS | スタイリング |
| `@tauri-apps/api` | Tauri バックエンド連携 |

### Design

- ターミナルライク、ダーク基調
- Tokyo Night カラーパレット
- モノスペースフォント

## Project Structure

```
tools/worktree-manager/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── tray.rs
│   │   ├── worktree.rs
│   │   ├── claude.rs
│   │   ├── hooks_server.rs
│   │   └── watcher.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── routes/
│   │   └── +page.svelte
│   ├── lib/
│   │   ├── components/
│   │   │   ├── WorktreeList.svelte
│   │   │   ├── WorktreeDetail.svelte
│   │   │   ├── SessionStatus.svelte
│   │   │   ├── ActivityLog.svelte
│   │   │   └── NewWorktreeDialog.svelte
│   │   └── stores/
│   │       ├── worktrees.ts
│   │       └── sessions.ts
│   └── app.css              # Tokyo Night テーマ
├── svelte.config.js
├── package.json
└── tailwind.config.js
```

## Core Workflows

### 1. New Worktree + Claude Code Launch

```
[+ New] ボタン
  → ブランチ名入力（or 既存ブランチ選択）
  → ベースブランチ選択（default: main）
  → git worktree add 実行
  → Claude Code の初期プロンプト入力（任意）
  → ターミナルで Claude Code セッション起動
  → ステータス監視開始
```

### 2. Question/Permission Alert

```
Claude Code が質問/許可待ち
  → hooks HTTP POST → Rust バックエンド
  → Tray バッジ赤 + macOS 通知
  → Tray クリック → 該当 worktree ハイライト
  → 「Terminal」ボタンで該当ターミナルにフォーカス
  → ユーザーが回答 → ステータス自動更新
```

### 3. External Tool Integration

```
「diffit」 → worktree パスで difft 起動
「IDE」   → worktree パスで VS Code / Cursor 起動
「Terminal」→ worktree パスで Ghostty ターミナル起動
```

### 4. Cleanup

```
マージ済みブランチ検知 → MERGED 表示
「クリーンアップ」→ 該当 worktree 一覧
  → 選択して一括削除（git worktree remove + git branch -d）
```

## Worktree Lifecycle Management

Phase 1 (初期): 半自動
- マージ済みブランチの worktree を検知・ハイライト
- 「クリーンアップしますか？」と提案
- ユーザーが選択して削除

Phase 2 (将来): フル自動
- マージ後の自動削除（設定で有効化）
- 一定期間放置の worktree に警告
- ルールベースの自動管理

## Tauri Commands (IPC)

フロントエンドから呼び出す Tauri command 一覧:

### Worktree 操作

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `list_worktrees` | `repo_path: String` | `Vec<WorktreeInfo>` | worktree 一覧取得 |
| `create_worktree` | `{ repo_path, branch_name, base_branch }` | `WorktreeInfo` | 新規 worktree 作成 |
| `remove_worktree` | `{ repo_path, worktree_path, delete_branch: bool }` | `()` | worktree 削除 |
| `get_worktree_status` | `worktree_path: String` | `GitStatus` | 変更ファイル数等 |
| `cleanup_merged` | `repo_path: String` | `Vec<WorktreeInfo>` | マージ済み worktree 一覧 |

### Claude Code 操作

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `launch_claude` | `{ worktree_path, initial_prompt? }` | `SessionInfo` | Claude Code 起動 |
| `get_session_status` | `session_id: String` | `SessionState` | セッション状態取得 |
| `get_activity_log` | `{ session_id, limit? }` | `Vec<ActivityEntry>` | アクティビティログ取得 |

### 外部ツール

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `open_in_ide` | `worktree_path: String` | `()` | IDE で開く |
| `open_difftool` | `worktree_path: String` | `()` | difft 起動 |
| `open_terminal` | `worktree_path: String` | `()` | Ghostty ターミナル起動 |

### 型定義

```typescript
interface WorktreeInfo {
  path: string;
  branch: string;
  base_branch: string;
  commit_hash: string;
  commits_ahead: number;
  is_merged: boolean;
  session: SessionInfo | null;
}

interface SessionInfo {
  session_id: string;
  state: 'working' | 'question' | 'idle';
  started_at: string;      // ISO 8601
  last_activity: string;   // ISO 8601
  current_tool?: string;
}

interface ActivityEntry {
  timestamp: string;
  type: 'tool_use' | 'message' | 'error';
  tool_name?: string;
  summary: string;
}

interface GitStatus {
  added: number;
  modified: number;
  deleted: number;
}
```

## Tray Badge

バッジの数値は **QUESTION 状態のセッション数**（= ユーザー対応が必要な件数）。
セッションが QUESTION → WORKING / IDLE に遷移すると自動的にカウント減少。
カウントが 0 になるとバッジ非表示。

## Error Handling

- **worktree パス消失**: 起動時に `git worktree list` と照合、不整合は警告表示
- **Claude Code クラッシュ**: プロセス監視で検知 → ステータス「停止」、再起動ボタン表示
- **hooks サーバーポート競合**: 空きポート自動選択、`settings.json` の hooks URL を動的更新
- **複数リポジトリ**: 初期版は単一リポジトリ。将来的にリポジトリ切り替え追加可能

## UI Reference

モックアップは `.superpowers/brainstorm/` に保存:
- `ui-layout.html` — メインウィンドウ + Tray ドロップダウン
