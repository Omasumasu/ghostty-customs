# ===== zellij + git worktree 連携 =====
# zellijペイン名をgit worktree名に自動設定

# zellijコマンドのパスを検出
_zellij_cmd() {
    command -v zellij 2>/dev/null || echo "/opt/homebrew/bin/zellij"
}

if [[ -n "$ZELLIJ" ]]; then
    function _update_zellij_pane_name() {
        local zellij_bin=$(_zellij_cmd)
        if [[ -x "$zellij_bin" ]] && git rev-parse --git-dir > /dev/null 2>&1; then
            local worktree_name=$(basename "$(git rev-parse --show-toplevel)")
            "$zellij_bin" action rename-pane "$worktree_name"
        fi
    }

    # cdするたびに更新
    function cd() {
        builtin cd "$@" && _update_zellij_pane_name
    }

    # 初回実行
    _update_zellij_pane_name
fi

# ===== クイック起動エイリアス =====
# 4ペイン構成で起動
alias zp4='zellij --layout parallel-claude'
# Terminal IDE構成で起動（3 Claude + yazi/lazygit）
alias zide='zellij --layout terminal-ide'

# yazi/lazygit クイック起動
alias y='yazi'
alias lg='lazygit'

# ===== fzf + ripgrep 統合 =====
if command -v fzf &> /dev/null; then
    # fzf設定
    export FZF_DEFAULT_OPTS='
        --height 60%
        --layout=reverse
        --border
        --preview-window=right:50%
        --bind=ctrl-u:preview-half-page-up,ctrl-d:preview-half-page-down
    '

    # fdがあればfzfのデフォルトコマンドに設定
    if command -v fd &> /dev/null; then
        export FZF_DEFAULT_COMMAND='fd --type f --hidden --follow --exclude .git'
        export FZF_CTRL_T_COMMAND="$FZF_DEFAULT_COMMAND"
    fi

    # ファイル検索（プレビュー付き）
    ff() {
        local file
        file=$(fzf --preview 'bat --style=numbers --color=always --line-range :500 {} 2>/dev/null || cat {}')
        [[ -n "$file" ]] && echo "$file"
    }

    # ripgrepで検索してfzfで絞り込み
    if command -v rg &> /dev/null; then
        # コンテンツ検索
        rgs() {
            local query="${1:-}"
            rg --color=always --line-number --no-heading --smart-case "$query" |
                fzf --ansi \
                    --delimiter ':' \
                    --preview 'bat --style=numbers --color=always --highlight-line {2} {1} 2>/dev/null || cat {1}' \
                    --preview-window '+{2}-10'
        }

        # インタラクティブripgrep（入力しながら検索）
        rgi() {
            local initial_query="${1:-}"
            rg --color=always --line-number --no-heading --smart-case "$initial_query" |
                fzf --ansi \
                    --disabled \
                    --query "$initial_query" \
                    --bind "change:reload:rg --color=always --line-number --no-heading --smart-case {q} || true" \
                    --delimiter ':' \
                    --preview 'bat --style=numbers --color=always --highlight-line {2} {1} 2>/dev/null || cat {1}' \
                    --preview-window '+{2}-10'
        }
    fi

    # Gitブランチ選択
    fbr() {
        local branches branch
        branches=$(git branch -a --color=always | grep -v HEAD)
        branch=$(echo "$branches" | fzf --ansi --preview 'git log --oneline --graph --color=always {1}' | sed 's/^[* ]*//' | sed 's#remotes/origin/##')
        [[ -n "$branch" ]] && git checkout "$branch"
    }

    # Git worktree選択
    fwt() {
        local worktrees wt
        worktrees=$(git worktree list)
        wt=$(echo "$worktrees" | fzf --preview 'git -C {1} log --oneline -10')
        [[ -n "$wt" ]] && cd "$(echo "$wt" | awk '{print $1}')"
    }

    # 履歴検索（Ctrl+R代替）
    fh() {
        local cmd
        cmd=$(history | fzf --tac --no-sort | awk '{$1=""; print substr($0,2)}')
        [[ -n "$cmd" ]] && print -z "$cmd"
    }
fi

# ===== yazi ディレクトリ移動統合 =====
# yaziで選択したディレクトリに移動
function yy() {
    local tmp="$(mktemp -t "yazi-cwd.XXXXXX")"
    yazi "$@" --cwd-file="$tmp"
    if cwd="$(cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
        cd -- "$cwd"
    fi
    rm -f -- "$tmp"
}
