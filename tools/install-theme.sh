#!/bin/bash
#
# Ghostty Theme Installer
# 8bit/Retro game themes for Ghostty terminal
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

# Install theme files
install_theme() {
    local theme="$1"
    local src="$THEMES_SRC/$theme"
    local dst="$GHOSTTY_THEMES_DIR/$theme"

    if [[ ! -f "$src" ]]; then
        print_error "Theme file not found: $src"
        return 1
    fi

    print_step "Installing theme: $theme"
    cp "$src" "$dst"
    print_success "Theme installed to $dst"
}

# Update Ghostty config
update_config() {
    local theme="$1"
    local font="$2"

    print_step "Updating Ghostty config..."

    # Create config if it doesn't exist
    if [[ ! -f "$GHOSTTY_CONFIG" ]]; then
        touch "$GHOSTTY_CONFIG"
    fi

    # Remove existing theme and font-family lines
    local temp_config=$(mktemp)
    grep -v "^theme = " "$GHOSTTY_CONFIG" 2>/dev/null | grep -v "^font-family = " > "$temp_config" || true

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
    } > "$GHOSTTY_CONFIG"

    rm "$temp_config"
    print_success "Config updated: $GHOSTTY_CONFIG"
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

# Main installation flow
main() {
    print_header

    # Check Ghostty setup
    check_ghostty

    # Select and install theme
    if ! select_theme; then
        exit 1
    fi

    install_theme "$SELECTED_THEME"

    # Font installation
    SELECTED_FONT=""
    echo ""
    read -p "Install pixel font for 8bit look? (y/N): " install_font_choice

    if [[ "$install_font_choice" =~ ^[Yy]$ ]]; then
        install_font
    fi

    # Update config
    update_config "$SELECTED_THEME" "$SELECTED_FONT"

    # Done!
    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║   INSTALLATION COMPLETE!                 ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "Theme: ${CYAN}$SELECTED_THEME${NC}"
    if [[ -n "$SELECTED_FONT" ]]; then
        echo -e "Font:  ${CYAN}$SELECTED_FONT${NC}"
    fi
    echo ""
    echo -e "${YELLOW}Restart Ghostty to apply changes.${NC}"
    echo ""
}

main "$@"
