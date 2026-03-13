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

2つの経路で状態を取得:

**Push (hooks)**: `~/.claude/settings.json` に HTTP hooks を設定。以下のイベントを受信:
- `Notification` (matcher: `idle_prompt`) → 入力待ち
- `Notification` (matcher: `permission_prompt`) → 許可待ち
- `PreToolUse` / `PostToolUse` → ツール実行中
- `Stop` → セッション完了
- `SessionStart` / `SessionEnd` → セッションライフサイクル

**Pull (JSONL watch)**: `~/.claude/projects/<encoded-path>/<session-id>.jsonl` を `notify` crate で監視。各行の `type` フィールドからアクティビティログを構築。

### Session States

| State | Color | Trigger |
|-------|-------|---------|
| WORKING | 緑 `#9ece6a` | PreToolUse / PostToolUse |
| QUESTION | 赤 `#f7768e` | idle_prompt / permission_prompt |
| IDLE | グレー `#565f89` | Stop / SessionEnd |
| MERGED | 紫 `#bb9af7` | ブランチがベースにマージ済み |

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

## Error Handling

- **worktree パス消失**: 起動時に `git worktree list` と照合、不整合は警告表示
- **Claude Code クラッシュ**: プロセス監視で検知 → ステータス「停止」、再起動ボタン表示
- **hooks サーバーポート競合**: 空きポート自動選択、`settings.json` の hooks URL を動的更新
- **複数リポジトリ**: 初期版は単一リポジトリ。将来的にリポジトリ切り替え追加可能

## UI Reference

モックアップは `.superpowers/brainstorm/` に保存:
- `ui-layout.html` — メインウィンドウ + Tray ドロップダウン
