# zellij + git worktree 並列開発環境

Ghostty + zellij + git worktree を組み合わせた並列開発環境のセットアップ。

## 概要

Claude Codeを複数のgit worktreeで同時に実行するための環境。
4ペイン（2x2グリッド）構成で、各ペインにworktree名が自動表示されます。

```
┌──────────────────┬──────────────────┐
│  worktree-1      │  worktree-2      │
│  (feature-xxx)   │  (feature-yyy)   │
├──────────────────┼──────────────────┤
│  worktree-3      │  worktree-4      │
│  (bugfix-zzz)    │  (main)          │
└──────────────────┴──────────────────┘
```

## インストール

```bash
./tools/install-zellij-integration.sh
```

セットアップされる内容:
- zellij（未インストールの場合）
- `~/.config/zellij/layouts/parallel-claude.kdl`
- シェル設定（ペイン名自動更新 + `zp4`エイリアス）
- Ghosttyのzellij対応キーバインド

## 使い方

### 1. 4ペイン構成で起動

```bash
zp4
```

### 2. 各ペインでworktreeに移動

```bash
cd /path/to/your-worktree
# → ペイン名が自動的にworktree名に変わる
```

### 3. Claude Codeを起動

```bash
claude
# または
git gtr new feature-xxx --ai
```

## zellijの基本操作

| キー | 動作 |
|-----|------|
| `Ctrl+p` → `h/j/k/l` | ペイン間移動 |
| `Ctrl+p` → `n` | 新規ペイン |
| `Ctrl+p` → `x` | ペイン閉じる |
| `Ctrl+p` → `f` | フルスクリーン切り替え |
| `Alt+h/j/k/l` | ペイン間移動（ショートカット） |
| `Ctrl+p` → `d` | デタッチ（バックグラウンド化） |

## アンインストール

```bash
./tools/install-zellij-integration.sh --uninstall
```

## ファイル構成

```
zellij/
├── README.md              # このファイル
├── layouts/
│   └── parallel-claude.kdl  # 4ペインレイアウト
└── zshrc-snippet.sh       # シェル設定スニペット
```

## カスタマイズ

### レイアウトの変更

`layouts/parallel-claude.kdl` を編集して、ペイン構成を変更できます。

例: 3ペイン構成（左1大 + 右2小）

```kdl
layout {
    pane split_direction="vertical" {
        pane name="main" size="60%"
        pane split_direction="horizontal" {
            pane name="sub-1"
            pane name="sub-2"
        }
    }
}
```

### シェル設定のカスタマイズ

`zshrc-snippet.sh` を編集して、ペイン名の表示形式などを変更できます。
