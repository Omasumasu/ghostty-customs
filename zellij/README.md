# Terminal IDE 環境

Ghostty + zellij + yazi + lazygit + fzf を組み合わせた「エディタを捨ててターミナルに寄せる」開発環境。

## コンセプト

- **ターミナル**: Ghostty（起動が爆速）
- **分割**: zellij（設定が楽）
- **ファイラー**: yazi（プレビュー付きで爆速）
- **Git**: lazygit（CLIなのにGUI並みに見やすい）
- **検索**: fzf + ripgrep（Cmd+Pより速い）
- **AIコーディング**: Claude Code

メモリ節約 + ターミナル完結の開発体験。

## レイアウト

### Terminal IDE（推奨・27インチモニター向け）

```
┌───────────────┬───────────────┬──────────┐
│  Claude 1     │  Claude 2     │ チート   │
│               │               │ シート   │
├───────────────┴───────────────┤          │
│         Claude 3              │          │
│                               │          │
└───────────────────────────────┴──────────┘

+ Alt+y → yazi（フローティング）
+ Alt+g → lazygit（フローティング）
+ Alt+t → ターミナル（フローティング）
```

- Claude Code 3つを十分な高さで並列実行
- 右側にチートシート常時表示（ショートカット一覧）
- yazi/lazygitは必要な時にフローティングで呼び出し
- 使い終わったらEscで閉じて戻る
- **Ghostty起動時にデフォルトで自動起動**

### 4ペイン（Claude Code 4並列）

```
┌──────────────────┬──────────────────┐
│  Claude Code 1   │  Claude Code 2   │
├──────────────────┼──────────────────┤
│  Claude Code 3   │  Claude Code 4   │
└──────────────────┴──────────────────┘
```

## インストール

```bash
./tools/install-zellij-integration.sh
```

インストールされるもの:
- zellij（未インストールの場合）
- yazi, lazygit, fzf, ripgrep, fd, bat
- レイアウトファイル
- シェル統合（エイリアス、fzf設定）
- zellijキーバインド設定

## 使い方

### 起動

Ghosttyを起動すると自動的にTerminal IDEが起動します（チートシート付き）。

```bash
# 手動で起動する場合
zide  # Terminal IDE構成（推奨）
zp4   # 4ペイン構成
```

### フローティングツール呼び出し

| キー | ツール | 説明 |
|------|--------|------|
| `Alt+y` | yazi | ファイラー（フローティング） |
| `Alt+g` | lazygit | Git操作（フローティング） |
| `Alt+t` | terminal | 汎用ターミナル（フローティング） |
| `Esc` | - | フローティングペインを閉じる |

### クイックコマンド

| コマンド | 説明 |
|---------|------|
| `y` | yazi起動 |
| `yy` | yazi + 終了時にディレクトリ移動 |
| `lg` | lazygit起動 |

### fzf検索コマンド

| コマンド | 説明 |
|---------|------|
| `ff` | ファイル検索（プレビュー付き） |
| `rgs <query>` | コンテンツ検索（ripgrep + fzf） |
| `rgi` | インタラクティブ検索（入力しながら検索） |
| `fbr` | Gitブランチ選択・切り替え |
| `fwt` | Git worktree選択・移動 |
| `fh` | コマンド履歴検索 |

### zellijの基本操作

| キー | 動作 |
|-----|------|
| `Ctrl+p` → `h/j/k/l` | ペイン間移動 |
| `Ctrl+p` → `n` | 新規ペイン |
| `Ctrl+p` → `x` | ペイン閉じる |
| `Ctrl+p` → `f` | フルスクリーン切り替え |
| `Ctrl+p` → `w` | フローティングペイン切り替え |
| `Alt+h/j/k/l` | ペイン間移動（ショートカット） |
| `Ctrl+p` → `d` | デタッチ（バックグラウンド化） |

## ワークフロー例

### 1. 新機能開発

```bash
# Terminal IDE起動
zide

# Alt+y でyazi起動、ファイル構造を確認
# Escで閉じる

# Claude Code ペインで開発
claude

# Alt+g でlazygit起動、コミット
# Escで閉じる
```

### 2. コード検索 → 編集

```bash
# インタラクティブ検索
rgi "function"

# 結果からファイルを特定
# Claude Code で編集依頼
```

### 3. 複数worktreeでの並列作業

```bash
# worktree選択・移動
fwt

# 各ペインで異なるworktreeのClaude Codeを起動
```

## アンインストール

```bash
./tools/install-zellij-integration.sh --uninstall
```

## ファイル構成

```
zellij/
├── README.md                    # このファイル
├── layouts/
│   ├── parallel-claude.kdl      # 4ペインレイアウト
│   └── terminal-ide.kdl         # Terminal IDEレイアウト
├── config-snippet.kdl           # zellij設定（キーバインド含む）
└── zshrc-snippet.sh             # シェル設定スニペット
```

## カスタマイズ

### レイアウトの変更

`layouts/terminal-ide.kdl` を編集して、ペイン構成を変更できます。

### キーバインドの変更

`config-snippet.kdl` を編集:

```kdl
keybinds {
    shared {
        // 別のキーに変更する例
        bind "Alt f" {
            Run "yazi" {
                floating true
            }
        }
    }
}
```

### fzf設定のカスタマイズ

`zshrc-snippet.sh` の `FZF_DEFAULT_OPTS` を編集:

```bash
export FZF_DEFAULT_OPTS='
    --height 80%           # 高さを変更
    --layout=reverse
    --border=rounded       # 枠線スタイル
    --preview-window=right:60%  # プレビュー幅
'
```

### yaziの設定

yaziの設定は `~/.config/yazi/` に配置:
- `yazi.toml` - 基本設定
- `keymap.toml` - キーバインド
- `theme.toml` - テーマ

詳細: https://yazi-rs.github.io/docs/configuration/overview
