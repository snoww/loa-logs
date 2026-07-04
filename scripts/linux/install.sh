#!/bin/bash
set -euo pipefail

REPO="snoww/loa-logs"
INSTALL_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/loa-logs"
DESKTOP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
ICON_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/icons"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

info() { echo "[loa-logs] $*"; }
error() { echo "[loa-logs] ERROR: $*" >&2; }

check_deps() {
    local missing=()
    for cmd in curl python3 pkexec pgrep; do
        if ! command -v "$cmd" > /dev/null 2>&1; then
            missing+=("$cmd")
        fi
    done
    if [ ${#missing[@]} -gt 0 ]; then
        error "Missing required commands: ${missing[*]}"
        error "Install them with your package manager before running this script."
        exit 1
    fi
}

get_latest_release() {
    curl -sf --max-time 10 "https://api.github.com/repos/$REPO/releases/latest"
}

parse_tag() {
    python3 -c "import sys,json; print(json.load(sys.stdin)['tag_name'])"
}

parse_asset_url() {
    local pattern="$1"
    python3 -c "
import sys, json
assets = json.load(sys.stdin)['assets']
for a in assets:
    if $pattern:
        print(a['browser_download_url'])
        break
"
}

main() {
    check_deps

    info "Fetching latest release info..."
    local release_json
    release_json=$(get_latest_release) || {
        error "Could not reach GitHub API."
        exit 1
    }

    local version
    version=$(echo "$release_json" | parse_tag) || {
        error "Could not parse release info."
        exit 1
    }
    info "Latest version: $version"

    local appimage_url nineveh_url
    appimage_url=$(echo "$release_json" | parse_asset_url \
        "a['name'].lower().endswith('.appimage')") || {
        error "No AppImage found in release."
        exit 1
    }
    nineveh_url=$(echo "$release_json" | parse_asset_url \
        "a['name'] == 'nineveh'") || {
        error "No nineveh binary found in release."
        exit 1
    }

    mkdir -p "$INSTALL_DIR" "$DESKTOP_DIR" "$ICON_DIR"

    info "Downloading AppImage..."
    curl -fL --progress-bar -o "$INSTALL_DIR/loa-logs.appimage" "$appimage_url"
    chmod +x "$INSTALL_DIR/loa-logs.appimage"

    info "Downloading nineveh..."
    curl -fL --progress-bar -o "$INSTALL_DIR/nineveh" "$nineveh_url"
    chmod +x "$INSTALL_DIR/nineveh"

    echo "$version" > "$INSTALL_DIR/.version"

    info "Installing launcher scripts..."
    cp "$SCRIPT_DIR/start-loa.sh" "$INSTALL_DIR/start-loa.sh"
    cp "$SCRIPT_DIR/nineveh-helper.sh" "$INSTALL_DIR/nineveh-helper.sh"
    chmod +x "$INSTALL_DIR/start-loa.sh" "$INSTALL_DIR/nineveh-helper.sh"

    info "Installing desktop entry..."
    local icon_src="$SCRIPT_DIR/../../src-tauri/icons/icon.png"
    if [ -f "$icon_src" ]; then
        cp "$icon_src" "$ICON_DIR/loa-logs.png"
    else
        info "Icon not found in repo, downloading from GitHub..."
        curl -sfL -o "$ICON_DIR/loa-logs.png" \
            "https://raw.githubusercontent.com/$REPO/master/src-tauri/icons/icon.png" || {
            error "Could not download icon."
        }
    fi

    cat > "$DESKTOP_DIR/loa-logs.desktop" <<DESKTOP
[Desktop Entry]
Name=LOA Logs
Comment=Lost Ark DPS meter with auto-updating launcher
Exec=$INSTALL_DIR/start-loa.sh
Terminal=false
Type=Application
Icon=$ICON_DIR/loa-logs.png
Categories=Utility;Game;
DESKTOP

    info "Installation complete!"
    info "  Install directory: $INSTALL_DIR"
    info "  Version: $version"
    info "  Launch from your application menu or run: $INSTALL_DIR/start-loa.sh"
}

main "$@"
