#!/bin/bash
set -euo pipefail

INSTALL_DIR="$(cd "$(dirname "$0")" && pwd)"
LOG_FILE="$INSTALL_DIR/start-loa.log"
VERSION_FILE="$INSTALL_DIR/.version"
STOP_FILE="$INSTALL_DIR/.nineveh-stop"
APPIMAGE="$INSTALL_DIR/loa-logs.appimage"
NINEVEH_BIN="$INSTALL_DIR/nineveh"
HELPER="$INSTALL_DIR/nineveh-helper.sh"

REPO="snoww/loa-logs"

LOA_PID=""
HELPER_PID=""
EXIT_CODE=0

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

echo "===== LOA Logs launcher started at $(date) =====" > "$LOG_FILE"

cleanup() {
    log "Shutting down..."
    if [ -n "$LOA_PID" ] && kill -0 "$LOA_PID" 2>/dev/null; then
        kill "$LOA_PID" 2>/dev/null
        wait "$LOA_PID" 2>/dev/null
        log "LOA Logs stopped."
    fi
    if pgrep -x nineveh > /dev/null 2>&1; then
        log "Signaling nineveh to stop..."
        touch "$STOP_FILE"
        for _ in $(seq 1 5); do
            pgrep -x nineveh > /dev/null 2>&1 || break
            sleep 1
        done
        if pgrep -x nineveh > /dev/null 2>&1; then
            log "WARNING: nineveh did not stop in time."
        else
            log "nineveh stopped."
        fi
    fi
    rm -f "$STOP_FILE"
    log "Cleanup complete."
    exit "$EXIT_CODE"
}

trap cleanup SIGINT SIGTERM SIGHUP EXIT
rm -f "$STOP_FILE"

check_for_updates() {
    local current_version=""
    if [ -f "$VERSION_FILE" ]; then
        current_version=$(cat "$VERSION_FILE")
    fi

    log "Checking for updates (current: ${current_version:-none})..."

    local release_json
    release_json=$(curl -sf --max-time 10 \
        "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null) || {
        log "WARNING: Could not reach GitHub API, skipping update check."
        return 1
    }

    local latest_tag
    latest_tag=$(echo "$release_json" | python3 -c \
        "import sys,json; print(json.load(sys.stdin)['tag_name'])" 2>/dev/null) || {
        log "WARNING: Could not parse release info."
        return 1
    }

    if [ "$latest_tag" = "$current_version" ]; then
        log "Already on latest version ($current_version)."
        return 1
    fi

    log "New version available: $latest_tag (current: ${current_version:-none})"

    local appimage_url nineveh_url
    appimage_url=$(echo "$release_json" | python3 -c "
import sys, json
assets = json.load(sys.stdin)['assets']
for a in assets:
    if a['name'].lower().endswith('.appimage'):
        print(a['browser_download_url'])
        break
" 2>/dev/null) || {
        log "ERROR: Could not find AppImage in release assets."
        return 1
    }

    nineveh_url=$(echo "$release_json" | python3 -c "
import sys, json
assets = json.load(sys.stdin)['assets']
for a in assets:
    if a['name'] == 'nineveh':
        print(a['browser_download_url'])
        break
" 2>/dev/null) || {
        log "ERROR: Could not find nineveh in release assets."
        return 1
    }

    if [ -z "$appimage_url" ] || [ -z "$nineveh_url" ]; then
        log "ERROR: Missing download URLs in release."
        return 1
    fi

    log "Downloading AppImage..."
    if ! curl -fL --progress-bar -o "$APPIMAGE.tmp" "$appimage_url" 2>>"$LOG_FILE"; then
        log "ERROR: AppImage download failed."
        rm -f "$APPIMAGE.tmp"
        return 1
    fi

    log "Downloading nineveh..."
    if ! curl -fL --progress-bar -o "$NINEVEH_BIN.tmp" "$nineveh_url" 2>>"$LOG_FILE"; then
        log "ERROR: nineveh download failed."
        rm -f "$APPIMAGE.tmp" "$NINEVEH_BIN.tmp"
        return 1
    fi

    mv "$APPIMAGE.tmp" "$APPIMAGE"
    chmod +x "$APPIMAGE"
    mv "$NINEVEH_BIN.tmp" "$NINEVEH_BIN"
    chmod +x "$NINEVEH_BIN"

    echo "$latest_tag" > "$VERSION_FILE"
    log "Updated to $latest_tag."
    return 0
}

if [ ! -f "$APPIMAGE" ] || [ ! -f "$NINEVEH_BIN" ]; then
    log "Missing binaries, downloading latest release..."
    rm -f "$VERSION_FILE"
    if ! check_for_updates; then
        log "ERROR: Could not download binaries. Cannot start."
        EXIT_CODE=1; exit 1
    fi
else
    check_for_updates || true
fi

log "Starting nineveh (requires authentication)..."
pkexec "$HELPER" "$NINEVEH_BIN" "$STOP_FILE" --stop-after-timeout 0 --proxy-without-ipc >> "$LOG_FILE" 2>&1 &
HELPER_PID=$!
log "nineveh helper launched (PID $HELPER_PID)"

log "Waiting for nineveh to initialize..."
TIMEOUT=30
ELAPSED=0
while [ "$ELAPSED" -lt "$TIMEOUT" ]; do
    if [ "$ELAPSED" -gt 3 ] && ! pgrep -x nineveh > /dev/null 2>&1; then
        log "ERROR: nineveh process died. Check log for details."
        EXIT_CODE=1; exit 1
    fi
    if grep -q "Spawned new proxy server" "$LOG_FILE" 2>/dev/null; then
        log "nineveh is ready (proxy server spawned)."
        break
    fi
    sleep 1
    ELAPSED=$((ELAPSED + 1))
done

if [ "$ELAPSED" -ge "$TIMEOUT" ]; then
    log "WARNING: timed out waiting for nineveh, launching LOA Logs anyway..."
fi

log "Starting LOA Logs..."
"$APPIMAGE" >> "$LOG_FILE" 2>&1 &
LOA_PID=$!
log "LOA Logs launched with PID $LOA_PID"

wait "$LOA_PID" 2>/dev/null
LOA_EXIT=$?
log "LOA Logs exited with code $LOA_EXIT"
cleanup
