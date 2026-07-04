#!/bin/bash
# Runs as root via pkexec.
# Kills any existing nineveh, starts a new one, watches for stop signal.

NINEVEH_BIN="$1"
STOP_FILE="$2"
shift 2
NINEVEH_ARGS="$@"
NINEVEH_PID=""

cleanup() {
    if [ -n "$NINEVEH_PID" ] && kill -0 "$NINEVEH_PID" 2>/dev/null; then
        kill "$NINEVEH_PID" 2>/dev/null
        wait "$NINEVEH_PID" 2>/dev/null
    fi
    rm -f "$STOP_FILE"
    exit 0
}

trap cleanup SIGINT SIGTERM SIGHUP

EXISTING=$(pgrep -x "$(basename "$NINEVEH_BIN")")
if [ -n "$EXISTING" ]; then
    kill $EXISTING 2>/dev/null
    sleep 1
fi

$NINEVEH_BIN $NINEVEH_ARGS &
NINEVEH_PID=$!

while kill -0 "$NINEVEH_PID" 2>/dev/null; do
    if [ -f "$STOP_FILE" ]; then
        cleanup
    fi
    sleep 1
done
