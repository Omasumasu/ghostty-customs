#!/bin/bash
#
# Ghostty + zellij + Terminal IDE Integration Installer
# ターミナルで完結する開発環境セットアップ
# 含まれるツール: zellij, yazi, lazygit, fzf, ripgrep, fd, bat
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
ZELLIJ_SRC="$REPO_DIR/zellij"
ZELLIJ_CONFIG_DIR="$HOME/.config/zellij"
ZELLIJ_LAYOUTS_DIR="$ZELLIJ_CONFIG_DIR/layouts"
GHOSTTY_CONFIG_DIR="$HOME/.config/ghostty"
GHOSTTY_CONFIG="$GHOSTTY_CONFIG_DIR/config"
ZSHRC="$HOME/.zshrc"
BASHRC="$HOME/.bashrc"

# Marker for our additions
MARKER_START="# ===== zellij + git worktree 連携 ====="
MARKER_END="# ===== zellij + git worktree 連携 END ====="

# Marker for zellij config.kdl
ZELLIJ_MARKER_START="// ===== ghostty-customs zellij config ====="
ZELLIJ_MARKER_END="// ===== ghostty-customs zellij config END ====="

# zellij executable path (detected dynamically)
ZELLIJ_BIN=""

print_header() {
    echo -e "${CYAN}"
    echo "╔══════════════════════════════════════════╗"
    echo "║   TERMINAL IDE INTEGRATION               ║"
    echo "║   ターミナル完結開発環境セットアップ     ║"
    echo "╚══════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_step() {
    echo -e "${GREEN}▶${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✖${NC} $1"
}

print_success() {
    echo -e "${GREEN}✔${NC} $1"
}

# Check if zellij is installed and set ZELLIJ_BIN
check_zellij() {
    ZELLIJ_BIN=$(command -v zellij 2>/dev/null || true)
    if [[ -n "$ZELLIJ_BIN" ]]; then
        local version=$("$ZELLIJ_BIN" --version 2>/dev/null || echo "unknown")
        print_success "zellij found: $version ($ZELLIJ_BIN)"
        return 0
    else
        print_warn "zellij not found"
        return 1
    fi
}

# Install zellij via Homebrew
install_zellij() {
    echo ""
    echo -e "${MAGENTA}=== ZELLIJ INSTALLATION ===${NC}"
    echo ""

    if check_zellij; then
        read -p "zellij is already installed. Reinstall? (y/N): " reinstall
        if [[ ! "$reinstall" =~ ^[Yy]$ ]]; then
            return 0
        fi
    fi

    if ! command -v brew &> /dev/null; then
        print_error "Homebrew not found. Please install zellij manually:"
        echo "  brew install zellij"
        echo "  or visit: https://zellij.dev/documentation/installation"
        return 1
    fi

    print_step "Installing zellij via Homebrew..."
    if brew install zellij; then
        print_success "zellij installed successfully"
        # Re-check to get the installed path
        check_zellij
    else
        print_error "Failed to install zellij"
        return 1
    fi
}

# Check and install terminal IDE tools
check_tool() {
    local tool="$1"
    if command -v "$tool" &> /dev/null; then
        local version=$("$tool" --version 2>/dev/null | head -1 || echo "installed")
        print_success "$tool: $version"
        return 0
    else
        print_warn "$tool not found"
        return 1
    fi
}

install_terminal_tools() {
    echo ""
    echo -e "${MAGENTA}=== TERMINAL IDE TOOLS INSTALLATION ===${NC}"
    echo ""

    if ! command -v brew &> /dev/null; then
        print_error "Homebrew not found. Please install tools manually."
        return 1
    fi

    local tools_to_install=()

    # Check each tool
    echo -e "${YELLOW}Checking installed tools...${NC}"
    echo ""

    if ! check_tool yazi; then
        tools_to_install+=("yazi")
    fi

    if ! check_tool lazygit; then
        tools_to_install+=("lazygit")
    fi

    if ! check_tool fzf; then
        tools_to_install+=("fzf")
    fi

    if ! check_tool rg; then
        tools_to_install+=("ripgrep")
    fi

    if ! check_tool fd; then
        tools_to_install+=("fd")
    fi

    if ! check_tool bat; then
        tools_to_install+=("bat")
    fi

    echo ""

    if [[ ${#tools_to_install[@]} -eq 0 ]]; then
        print_success "All terminal IDE tools are already installed!"
        return 0
    fi

    echo -e "${YELLOW}The following tools will be installed:${NC}"
    for tool in "${tools_to_install[@]}"; do
        echo "  - $tool"
    done
    echo ""

    read -p "Install these tools? (Y/n): " install_choice
    if [[ "$install_choice" =~ ^[Nn]$ ]]; then
        print_warn "Skipping tool installation"
        return 0
    fi

    print_step "Installing tools via Homebrew..."
    if brew install "${tools_to_install[@]}"; then
        print_success "Tools installed successfully"
    else
        print_error "Some tools failed to install"
        return 1
    fi
}

# Create zellij directories and copy layout
setup_zellij_layout() {
    echo ""
    echo -e "${MAGENTA}=== ZELLIJ LAYOUT SETUP ===${NC}"
    echo ""

    # Create directories
    if [[ ! -d "$ZELLIJ_CONFIG_DIR" ]]; then
        print_step "Creating zellij config directory..."
        mkdir -p "$ZELLIJ_CONFIG_DIR"
    fi

    if [[ ! -d "$ZELLIJ_LAYOUTS_DIR" ]]; then
        print_step "Creating zellij layouts directory..."
        mkdir -p "$ZELLIJ_LAYOUTS_DIR"
    fi

    # Copy layout files
    local layouts=("parallel-claude.kdl" "terminal-ide.kdl")

    for layout_name in "${layouts[@]}"; do
        local layout_src="$ZELLIJ_SRC/layouts/$layout_name"
        local layout_dst="$ZELLIJ_LAYOUTS_DIR/$layout_name"

        if [[ -f "$layout_src" ]]; then
            print_step "Installing $layout_name layout..."
            cp "$layout_src" "$layout_dst"
            print_success "Layout installed: $layout_dst"
        else
            print_warn "Layout file not found: $layout_src (skipping)"
        fi
    done
}

# Check if zellij config integration already exists
check_zellij_config_integration() {
    local config_file="$ZELLIJ_CONFIG_DIR/config.kdl"
    if [[ -f "$config_file" ]] && grep -q "$ZELLIJ_MARKER_START" "$config_file"; then
        return 0  # Already installed
    fi
    return 1  # Not installed
}

# Setup zellij config (default_shell as login shell)
setup_zellij_config() {
    echo ""
    echo -e "${MAGENTA}=== ZELLIJ CONFIG (LOGIN SHELL) ===${NC}"
    echo ""

    local config_file="$ZELLIJ_CONFIG_DIR/config.kdl"
    local snippet_file="$ZELLIJ_SRC/config-snippet.kdl"

    if [[ ! -f "$snippet_file" ]]; then
        print_warn "Config snippet not found: $snippet_file"
        print_step "Creating default config snippet..."
        # Create default snippet inline
        mkdir -p "$ZELLIJ_SRC"
        cat > "$snippet_file" << 'EOF'
// デフォルトシェル
// Zellijはインタラクティブシェルとして起動するため.zshrcは自動的に読み込まれます
default_shell "zsh"
EOF
    fi

    # Check if config.kdl exists
    if [[ ! -f "$config_file" ]]; then
        print_step "Creating zellij config.kdl..."
        {
            echo "$ZELLIJ_MARKER_START"
            cat "$snippet_file"
            echo "$ZELLIJ_MARKER_END"
        } > "$config_file"
        print_success "Zellij config created: $config_file"
        return 0
    fi

    # Check if already installed
    if check_zellij_config_integration; then
        print_warn "Zellij login shell config already installed"
        read -p "Reinstall? (y/N): " reinstall
        if [[ ! "$reinstall" =~ ^[Yy]$ ]]; then
            return 0
        fi
        # Remove existing integration
        print_step "Removing existing config integration..."
        local temp_file=$(mktemp)
        sed "/$ZELLIJ_MARKER_START/,/$ZELLIJ_MARKER_END/d" "$config_file" > "$temp_file"
        mv "$temp_file" "$config_file"
    fi

    # Check if default_shell is already set (not commented out)
    if grep -q "^default_shell" "$config_file"; then
        print_warn "default_shell already configured in config.kdl"
        print_step "Skipping to avoid duplicate configuration"
        return 0
    fi

    # Add integration at the end of file
    print_step "Adding login shell configuration..."
    {
        echo ""
        echo "$ZELLIJ_MARKER_START"
        cat "$snippet_file"
        echo "$ZELLIJ_MARKER_END"
    } >> "$config_file"

    print_success "Login shell configuration added to $config_file"
}

# Check if shell integration already exists
check_shell_integration() {
    local rc_file="$1"
    if [[ -f "$rc_file" ]] && grep -q "$MARKER_START" "$rc_file"; then
        return 0  # Already installed
    fi
    return 1  # Not installed
}

# Add shell integration
setup_shell_integration() {
    echo ""
    echo -e "${MAGENTA}=== SHELL INTEGRATION ===${NC}"
    echo ""

    local snippet_file="$ZELLIJ_SRC/zshrc-snippet.sh"

    if [[ ! -f "$snippet_file" ]]; then
        print_error "Shell snippet not found: $snippet_file"
        return 1
    fi

    # Determine which shell config to use
    local shell_rc=""
    local shell_name=""

    if [[ -f "$ZSHRC" ]] || [[ "$SHELL" == *"zsh"* ]]; then
        shell_rc="$ZSHRC"
        shell_name="zsh"
    elif [[ -f "$BASHRC" ]]; then
        shell_rc="$BASHRC"
        shell_name="bash"
    else
        print_warn "No shell config found. Creating .zshrc"
        shell_rc="$ZSHRC"
        shell_name="zsh"
    fi

    print_step "Target shell config: $shell_rc"

    # Check if already installed
    if check_shell_integration "$shell_rc"; then
        print_warn "Shell integration already installed"
        read -p "Reinstall? (y/N): " reinstall
        if [[ ! "$reinstall" =~ ^[Yy]$ ]]; then
            return 0
        fi
        # Remove existing integration
        print_step "Removing existing integration..."
        local temp_file=$(mktemp)
        sed "/$MARKER_START/,/$MARKER_END/d" "$shell_rc" > "$temp_file"
        mv "$temp_file" "$shell_rc"
    fi

    # Add integration
    print_step "Adding shell integration..."
    {
        echo ""
        echo "$MARKER_START"
        cat "$snippet_file"
        echo "$MARKER_END"
    } >> "$shell_rc"

    print_success "Shell integration added to $shell_rc"
}

# Setup Ghostty config
setup_ghostty_config() {
    echo ""
    echo -e "${MAGENTA}=== GHOSTTY CONFIG ===${NC}"
    echo ""

    # Create Ghostty config directory if needed
    if [[ ! -d "$GHOSTTY_CONFIG_DIR" ]]; then
        print_step "Creating Ghostty config directory..."
        mkdir -p "$GHOSTTY_CONFIG_DIR"
    fi

    # Check if config exists
    if [[ -f "$GHOSTTY_CONFIG" ]]; then
        print_step "Ghostty config exists: $GHOSTTY_CONFIG"

        # Check if zellij keybinds already set
        if grep -q "# zellij integration" "$GHOSTTY_CONFIG"; then
            print_warn "zellij keybinds already configured"
        else
            read -p "Add zellij-friendly keybinds? (Y/n): " add_keybinds
            if [[ ! "$add_keybinds" =~ ^[Nn]$ ]]; then
                print_step "Adding zellij-friendly keybinds..."
                {
                    echo ""
                    echo "# zellij integration"
                    echo "keybind = ctrl+shift+t=new_tab"
                    echo "keybind = ctrl+shift+n=new_window"
                } >> "$GHOSTTY_CONFIG"
                print_success "Keybinds added"
            fi
        fi

        # Check if zellij auto-start is configured
        if grep -q "^command = .*zellij" "$GHOSTTY_CONFIG"; then
            print_warn "zellij自動起動は既に設定済み"
        else
            echo ""
            echo -e "${YELLOW}Ghostty起動時にzellijを自動起動しますか？${NC}"
            echo "  1) terminal-ide（推奨: Claude 3ペイン + チートシート）"
            echo "  2) parallel-claude（4ペイン）"
            echo "  3) 設定しない"
            read -p "選択 [1-3]: " layout_choice
            case "$layout_choice" in
                1)
                    print_step "Terminal IDEレイアウトで自動起動を設定..."
                    {
                        echo ""
                        echo "# Terminal IDEレイアウトでzellij起動"
                        echo "command = $ZELLIJ_BIN --layout terminal-ide"
                    } >> "$GHOSTTY_CONFIG"
                    print_success "自動起動設定完了（terminal-ide）"
                    ;;
                2)
                    print_step "4ペインレイアウトで自動起動を設定..."
                    {
                        echo ""
                        echo "# 4ペインレイアウトでzellij起動"
                        echo "command = $ZELLIJ_BIN --layout parallel-claude"
                    } >> "$GHOSTTY_CONFIG"
                    print_success "自動起動設定完了（parallel-claude）"
                    ;;
                *)
                    print_warn "自動起動はスキップ"
                    ;;
            esac
        fi
    else
        print_step "Creating Ghostty config..."
        cat > "$GHOSTTY_CONFIG" << EOF
# Ghostty config

# Font
font-size = 14

# Window
window-padding-x = 4
window-padding-y = 4

# zellij integration
keybind = ctrl+shift+t=new_tab
keybind = ctrl+shift+n=new_window

# Terminal IDEレイアウトでzellij起動
command = $ZELLIJ_BIN --layout terminal-ide
EOF
        print_success "Ghostty config created: $GHOSTTY_CONFIG"
    fi
}

# Show usage instructions
show_usage() {
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║   SETUP COMPLETE!                        ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${MAGENTA}=== 使い方 ===${NC}"
    echo ""
    echo -e "  ${GREEN}1.${NC} 新しいターミナルを開くか、設定を再読み込み:"
    echo -e "     ${CYAN}source ~/.zshrc${NC}"
    echo ""
    echo -e "  ${GREEN}2.${NC} Ghosttyを起動すると自動的にTerminal IDEが起動します"
    echo -e "     （右側にチートシート付き）"
    echo ""
    echo -e "  ${GREEN}3.${NC} 手動起動:"
    echo -e "     ${CYAN}zide${NC}  # Terminal IDE（Claude 3ペイン + チートシート）"
    echo -e "     ${CYAN}zp4${NC}   # 4ペイン構成"
    echo ""
    echo -e "${MAGENTA}=== フローティングツール（zellij内） ===${NC}"
    echo ""
    echo -e "  ${YELLOW}Alt+y${NC}  yazi（ファイラー）"
    echo -e "  ${YELLOW}Alt+g${NC}  lazygit（Git操作）"
    echo -e "  ${YELLOW}Alt+t${NC}  汎用ターミナル"
    echo -e "  ${YELLOW}Esc${NC}    閉じて戻る"
    echo ""
    echo -e "${MAGENTA}=== fzf検索コマンド ===${NC}"
    echo ""
    echo -e "  ${YELLOW}ff${NC}   ファイル検索（プレビュー付き）"
    echo -e "  ${YELLOW}rgs${NC}  コンテンツ検索（ripgrep + fzf）"
    echo -e "  ${YELLOW}rgi${NC}  インタラクティブ検索（入力しながら検索）"
    echo -e "  ${YELLOW}fbr${NC}  Gitブランチ選択"
    echo -e "  ${YELLOW}fwt${NC}  Git worktree選択"
    echo -e "  ${YELLOW}fh${NC}   履歴検索"
    echo ""
    echo -e "${MAGENTA}=== zellijの基本操作 ===${NC}"
    echo ""
    echo -e "  ${YELLOW}Ctrl+p → h/j/k/l${NC}  ペイン間移動"
    echo -e "  ${YELLOW}Ctrl+p → n${NC}        新規ペイン"
    echo -e "  ${YELLOW}Ctrl+p → x${NC}        ペイン閉じる"
    echo -e "  ${YELLOW}Ctrl+p → f${NC}        フルスクリーン"
    echo -e "  ${YELLOW}Ctrl+p → w${NC}        フローティング切り替え"
    echo ""
}

# Uninstall
uninstall() {
    echo ""
    echo -e "${YELLOW}=== UNINSTALL ===${NC}"
    echo ""

    # Remove layouts
    local layouts=("parallel-claude.kdl" "terminal-ide.kdl")
    for layout_name in "${layouts[@]}"; do
        local layout_dst="$ZELLIJ_LAYOUTS_DIR/$layout_name"
        if [[ -f "$layout_dst" ]]; then
            print_step "Removing $layout_name layout..."
            rm "$layout_dst"
            print_success "Layout removed: $layout_name"
        fi
    done

    # Remove zellij config integration
    local config_file="$ZELLIJ_CONFIG_DIR/config.kdl"
    if [[ -f "$config_file" ]] && grep -q "$ZELLIJ_MARKER_START" "$config_file"; then
        print_step "Removing zellij config integration..."
        local temp_file=$(mktemp)
        sed "/$ZELLIJ_MARKER_START/,/$ZELLIJ_MARKER_END/d" "$config_file" > "$temp_file"
        mv "$temp_file" "$config_file"
        print_success "Zellij config integration removed"
    fi

    # Remove shell integration
    for rc_file in "$ZSHRC" "$BASHRC"; do
        if [[ -f "$rc_file" ]] && grep -q "$MARKER_START" "$rc_file"; then
            print_step "Removing shell integration from $rc_file..."
            local temp_file=$(mktemp)
            sed "/$MARKER_START/,/$MARKER_END/d" "$rc_file" > "$temp_file"
            mv "$temp_file" "$rc_file"
            print_success "Shell integration removed"
        fi
    done

    echo ""
    print_success "Uninstall complete"
    echo ""
}

# Main
main() {
    print_header

    # Parse arguments
    if [[ "$1" == "--uninstall" ]] || [[ "$1" == "-u" ]]; then
        uninstall
        exit 0
    fi

    # Install zellij if needed
    if ! check_zellij; then
        read -p "Install zellij? (Y/n): " install_choice
        if [[ ! "$install_choice" =~ ^[Nn]$ ]]; then
            install_zellij || exit 1
        else
            print_error "zellij is required. Exiting."
            exit 1
        fi
    fi

    # Install terminal IDE tools
    install_terminal_tools

    # Setup steps
    setup_zellij_layout
    setup_zellij_config
    setup_shell_integration
    setup_ghostty_config

    # Show usage
    show_usage
}

main "$@"
