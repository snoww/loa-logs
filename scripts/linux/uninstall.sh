#!/bin/bash
set -euo pipefail

INSTALL_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/loa-logs"
DESKTOP_FILE="${XDG_DATA_HOME:-$HOME/.local/share}/applications/loa-logs.desktop"
ICON_FILE="${XDG_DATA_HOME:-$HOME/.local/share}/icons/loa-logs.png"
APP_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/xyz.snow.loa-logs"
APP_DATA="${XDG_DATA_HOME:-$HOME/.local/share}/xyz.snow.loa-logs"

info() { echo "[loa-logs] $*"; }

if pgrep -x nineveh > /dev/null 2>&1 || pgrep -f "loa-logs.appimage" > /dev/null 2>&1; then
    info "LOA Logs appears to be running. Please close it first."
    exit 1
fi

info "This will remove:"
info "  $INSTALL_DIR (launcher, binaries, scripts)"
info "  $DESKTOP_FILE"
info "  $ICON_FILE"
echo ""

if [ -d "$APP_DATA" ]; then
    info "Your encounter data and settings will NOT be removed:"
    info "  $APP_CONFIG"
    info "  $APP_DATA"
    info "To remove those too, delete them manually."
    echo ""
fi

read -rp "Proceed with uninstall? [y/N] " confirm
if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    info "Cancelled."
    exit 0
fi

rm -rf "$INSTALL_DIR"
rm -f "$DESKTOP_FILE"
rm -f "$ICON_FILE"

info "Uninstall complete."
