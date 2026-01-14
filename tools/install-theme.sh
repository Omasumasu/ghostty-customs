#!/bin/bash
#
# Ghostty Theme Installer
# 8bit/Retro game themes for Ghostty terminal
# With optional zellij theme integration
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
THEMES_SRC="$REPO_DIR/themes"
GHOSTTY_CONFIG_DIR="$HOME/.config/ghostty"
GHOSTTY_THEMES_DIR="$GHOSTTY_CONFIG_DIR/themes"
GHOSTTY_CONFIG="$GHOSTTY_CONFIG_DIR/config"
ZELLIJ_CONFIG_DIR="$HOME/.config/zellij"
ZELLIJ_THEMES_DIR="$ZELLIJ_CONFIG_DIR/themes"
ZELLIJ_CONFIG="$ZELLIJ_CONFIG_DIR/config.kdl"

# Flags
USE_ZELLIJ=false
SELECTED_THEME=""
SELECTED_FONT=""

# Available themes
declare -a THEMES=(
    "nes-rpg:NES/Famicom RPG Style (Dragon Quest, FF風)"
    "arcade-neon:80s Arcade Neon (パックマン、ギャラガ風)"
    "avatar-terminal:Avatar UI Style (AIターミナル風)"
    "retro-amber:CRT Amber Monitor (アンバーモニター)"
    "retro-phosphor:CRT Green Phosphor (グリーンモニター)"
)

# Available pixel fonts
declare -a FONTS=(
    "font-press-start-2p:Press Start 2P:最も8bit/NES風"
    "font-cozette:Cozette:Nerd Font互換ピクセルフォント"
)

print_header() {
    echo -e "${CYAN}"
    echo "╔══════════════════════════════════════════╗"
    echo "║   GHOSTTY 8BIT THEME INSTALLER           ║"
    echo "║   ゴーストティ 8ビット テーマ           ║"
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

# Ask if user wants zellij integration
ask_zellij() {
    echo ""
    echo -e "${MAGENTA}=== ZELLIJ INTEGRATION ===${NC}"
    echo ""
    echo "zellijを使用していますか？"
    echo "Ghosttyのテーマに合わせたzellijテーマを生成します。"
    echo ""
    read -p "zellijテーマも生成する? (y/N): " zellij_choice

    if [[ "$zellij_choice" =~ ^[Yy]$ ]]; then
        USE_ZELLIJ=true
        print_success "zellijテーマも生成します"
    else
        USE_ZELLIJ=false
        print_step "zellijテーマはスキップします"
    fi
}

# Display theme selection menu
select_theme() {
    echo ""
    echo -e "${MAGENTA}=== SELECT THEME ===${NC}"
    echo ""

    local i=1
    for theme_info in "${THEMES[@]}"; do
        local name="${theme_info%%:*}"
        local desc="${theme_info#*:}"
        echo -e "  ${CYAN}[$i]${NC} $name"
        echo -e "      ${YELLOW}$desc${NC}"
        ((i++))
    done

    echo ""
    read -p "Select theme number (1-${#THEMES[@]}): " selection

    if [[ "$selection" =~ ^[0-9]+$ ]] && [ "$selection" -ge 1 ] && [ "$selection" -le "${#THEMES[@]}" ]; then
        local idx=$((selection - 1))
        SELECTED_THEME="${THEMES[$idx]%%:*}"
        return 0
    else
        print_error "Invalid selection"
        return 1
    fi
}

# Check if Ghostty config directory exists
check_ghostty() {
    if [[ ! -d "$GHOSTTY_CONFIG_DIR" ]]; then
        print_step "Creating Ghostty config directory..."
        mkdir -p "$GHOSTTY_CONFIG_DIR"
    fi

    if [[ ! -d "$GHOSTTY_THEMES_DIR" ]]; then
        print_step "Creating Ghostty themes directory..."
        mkdir -p "$GHOSTTY_THEMES_DIR"
    fi
}

# Check if zellij config directory exists
check_zellij() {
    if [[ ! -d "$ZELLIJ_CONFIG_DIR" ]]; then
        print_step "Creating zellij config directory..."
        mkdir -p "$ZELLIJ_CONFIG_DIR"
    fi

    if [[ ! -d "$ZELLIJ_THEMES_DIR" ]]; then
        print_step "Creating zellij themes directory..."
        mkdir -p "$ZELLIJ_THEMES_DIR"
    fi
}

# Install Ghostty theme files
install_ghostty_theme() {
    local theme="$1"
    local src="$THEMES_SRC/$theme"
    local dst="$GHOSTTY_THEMES_DIR/$theme"

    if [[ ! -f "$src" ]]; then
        print_error "Theme file not found: $src"
        return 1
    fi

    print_step "Installing Ghostty theme: $theme"
    cp "$src" "$dst"
    print_success "Ghostty theme installed to $dst"
}

# Parse Ghostty theme and extract colors
parse_ghostty_theme() {
    local theme_file="$1"

    # Extract colors from Ghostty theme file
    BG=$(grep "^background = " "$theme_file" | sed 's/background = //')
    FG=$(grep "^foreground = " "$theme_file" | sed 's/foreground = //')

    # Extract palette colors (palette = N=#XXXXXX format)
    COLOR_BLACK=$(grep "^palette = 0=" "$theme_file" | sed 's/palette = 0=//')
    COLOR_RED=$(grep "^palette = 1=" "$theme_file" | sed 's/palette = 1=//')
    COLOR_GREEN=$(grep "^palette = 2=" "$theme_file" | sed 's/palette = 2=//')
    COLOR_YELLOW=$(grep "^palette = 3=" "$theme_file" | sed 's/palette = 3=//')
    COLOR_BLUE=$(grep "^palette = 4=" "$theme_file" | sed 's/palette = 4=//')
    COLOR_MAGENTA=$(grep "^palette = 5=" "$theme_file" | sed 's/palette = 5=//')
    COLOR_CYAN=$(grep "^palette = 6=" "$theme_file" | sed 's/palette = 6=//')
    COLOR_WHITE=$(grep "^palette = 7=" "$theme_file" | sed 's/palette = 7=//')

    # Bright colors (use as orange alternative)
    COLOR_ORANGE=$(grep "^palette = 9=" "$theme_file" | sed 's/palette = 9=//')
}

# Generate zellij theme from Ghostty colors
generate_zellij_theme() {
    local theme_name="$1"
    local ghostty_theme="$THEMES_SRC/$theme_name"
    local zellij_theme="$ZELLIJ_THEMES_DIR/$theme_name.kdl"

    print_step "Generating zellij theme: $theme_name"

    # Parse Ghostty theme
    parse_ghostty_theme "$ghostty_theme"

    # Generate zellij theme file
    cat > "$zellij_theme" << EOF
// Auto-generated zellij theme from Ghostty theme: $theme_name
// Generated by install-theme.sh

themes {
    $theme_name {
        fg "$FG"
        bg "$BG"
        black "$COLOR_BLACK"
        red "$COLOR_RED"
        green "$COLOR_GREEN"
        yellow "$COLOR_YELLOW"
        blue "$COLOR_BLUE"
        magenta "$COLOR_MAGENTA"
        cyan "$COLOR_CYAN"
        white "$COLOR_WHITE"
        orange "$COLOR_ORANGE"
    }
}
EOF

    print_success "Zellij theme generated: $zellij_theme"
}

# Update zellij config to use theme
update_zellij_config() {
    local theme="$1"

    print_step "Updating zellij config..."

    if [[ ! -f "$ZELLIJ_CONFIG" ]]; then
        print_warn "zellij config not found: $ZELLIJ_CONFIG"
        print_warn "Please add the following to your zellij config:"
        echo ""
        echo -e "  ${CYAN}theme \"$theme\"${NC}"
        echo ""
        return 0
    fi

    # Check if theme line exists
    if grep -q "^theme " "$ZELLIJ_CONFIG"; then
        # Replace existing theme line
        sed -i.bak "s/^theme .*/theme \"$theme\"/" "$ZELLIJ_CONFIG"
        rm -f "$ZELLIJ_CONFIG.bak"
        print_success "Updated theme in zellij config"
    else
        # Check if there's a commented theme line
        if grep -q "^// *theme " "$ZELLIJ_CONFIG"; then
            # Add theme line after the commented one
            sed -i.bak "/^\/\/ *theme /a\\
theme \"$theme\"" "$ZELLIJ_CONFIG"
            rm -f "$ZELLIJ_CONFIG.bak"
            print_success "Added theme to zellij config"
        else
            # Append theme to end of file
            echo "" >> "$ZELLIJ_CONFIG"
            echo "// 8bit theme" >> "$ZELLIJ_CONFIG"
            echo "theme \"$theme\"" >> "$ZELLIJ_CONFIG"
            print_success "Appended theme to zellij config"
        fi
    fi
}

# Update Ghostty config
update_ghostty_config() {
    local theme="$1"
    local font="$2"

    print_step "Updating Ghostty config..."

    # Create config if it doesn't exist
    if [[ ! -f "$GHOSTTY_CONFIG" ]]; then
        touch "$GHOSTTY_CONFIG"
    fi

    # Remove existing 8bit theme settings block
    local temp_config=$(mktemp)
    sed '/^# === 8BIT THEME SETTINGS ===/,/^$/d' "$GHOSTTY_CONFIG" > "$temp_config" 2>/dev/null || cat "$GHOSTTY_CONFIG" > "$temp_config"

    # Also remove any stray theme/font-family lines
    grep -v "^theme = " "$temp_config" 2>/dev/null | grep -v "^font-family = " > "${temp_config}.2" || true
    mv "${temp_config}.2" "$temp_config"

    # Add new settings
    {
        cat "$temp_config"
        echo ""
        echo "# === 8BIT THEME SETTINGS ==="
        echo "theme = $theme"
        if [[ -n "$font" ]]; then
            echo "font-family = \"$font\""
        fi
        echo ""
        echo "# 8bit style options"
        echo "cursor-style = block"
        echo "cursor-style-blink = true"
        echo "window-padding-x = 16"
        echo "window-padding-y = 16"
        echo ""
    } > "$GHOSTTY_CONFIG"

    rm "$temp_config"
    print_success "Ghostty config updated: $GHOSTTY_CONFIG"
}

# Check if Homebrew is available
check_homebrew() {
    if command -v brew &> /dev/null; then
        return 0
    else
        return 1
    fi
}

# Install pixel font via Homebrew
install_font() {
    echo ""
    echo -e "${MAGENTA}=== PIXEL FONT INSTALLATION ===${NC}"
    echo ""

    if ! check_homebrew; then
        print_warn "Homebrew not found. Skipping font installation."
        print_warn "Install manually: https://brew.sh"
        return 1
    fi

    echo "Available pixel fonts:"
    echo ""

    local i=1
    for font_info in "${FONTS[@]}"; do
        IFS=':' read -r cask name desc <<< "$font_info"
        echo -e "  ${CYAN}[$i]${NC} $name"
        echo -e "      ${YELLOW}$desc${NC}"
        ((i++))
    done
    echo -e "  ${CYAN}[$i]${NC} Skip font installation"

    echo ""
    read -p "Select font (1-$i): " font_selection

    if [[ "$font_selection" == "$i" ]]; then
        print_step "Skipping font installation"
        SELECTED_FONT=""
        return 0
    fi

    if [[ "$font_selection" =~ ^[0-9]+$ ]] && [ "$font_selection" -ge 1 ] && [ "$font_selection" -lt "$i" ]; then
        local idx=$((font_selection - 1))
        IFS=':' read -r cask name desc <<< "${FONTS[$idx]}"

        print_step "Installing font: $name via Homebrew..."

        # Add tap if needed
        brew tap homebrew/cask-fonts 2>/dev/null || true

        if brew install --cask "$cask" 2>/dev/null; then
            print_success "Font installed: $name"
            SELECTED_FONT="$name"
        else
            print_warn "Font may already be installed or installation failed"
            SELECTED_FONT="$name"
        fi
    else
        print_error "Invalid selection"
        SELECTED_FONT=""
    fi
}

# Print completion summary
print_summary() {
    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║   INSTALLATION COMPLETE!                 ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "Theme: ${CYAN}$SELECTED_THEME${NC}"
    if [[ -n "$SELECTED_FONT" ]]; then
        echo -e "Font:  ${CYAN}$SELECTED_FONT${NC}"
    fi
    if [[ "$USE_ZELLIJ" == true ]]; then
        echo -e "Zellij: ${CYAN}テーマ設定済み${NC}"
    fi
    echo ""
    echo -e "${YELLOW}Restart Ghostty to apply changes.${NC}"
    if [[ "$USE_ZELLIJ" == true ]]; then
        echo -e "${YELLOW}zellijも再起動してください。${NC}"
    fi
    echo ""
}

# Main installation flow
main() {
    print_header

    # Step 1: Ask about zellij
    ask_zellij

    # Step 2: Select theme
    if ! select_theme; then
        exit 1
    fi

    # Step 3: Setup directories
    check_ghostty
    if [[ "$USE_ZELLIJ" == true ]]; then
        check_zellij
    fi

    # Step 4: Install Ghostty theme
    install_ghostty_theme "$SELECTED_THEME"

    # Step 5: Generate and install zellij theme if requested
    if [[ "$USE_ZELLIJ" == true ]]; then
        generate_zellij_theme "$SELECTED_THEME"
        update_zellij_config "$SELECTED_THEME"
    fi

    # Step 6: Font installation (optional)
    SELECTED_FONT=""
    echo ""
    read -p "Install pixel font for 8bit look? (y/N): " install_font_choice

    if [[ "$install_font_choice" =~ ^[Yy]$ ]]; then
        install_font
    fi

    # Step 7: Update Ghostty config
    update_ghostty_config "$SELECTED_THEME" "$SELECTED_FONT"

    # Done!
    print_summary
}

main "$@"
