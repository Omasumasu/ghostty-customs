# ===== zellij + git worktree 連携 =====
# zellijペイン名をgit worktree名に自動設定
if [[ -n "$ZELLIJ" ]]; then
    function _update_zellij_pane_name() {
        if git rev-parse --git-dir > /dev/null 2>&1; then
            local worktree_name=$(basename "$(git rev-parse --show-toplevel)")
            zellij action rename-pane "$worktree_name"
        fi
    }

    # cdするたびに更新
    function cd() {
        builtin cd "$@" && _update_zellij_pane_name
    }

    # 初回実行
    _update_zellij_pane_name
fi

# 4ペイン構成で起動
alias zp4='zellij --layout parallel-claude'
