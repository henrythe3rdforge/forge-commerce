#!/bin/bash
PID_FILE=".forge-commerce.pid"

if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if kill -0 "$PID" 2>/dev/null; then
        echo "✅ Forge Commerce is running (PID: $PID)"
        exit 0
    else
        echo "⚠️  PID file exists but process not running"
        rm -f "$PID_FILE"
        exit 1
    fi
else
    echo "⛔ Forge Commerce is not running"
    exit 1
fi
