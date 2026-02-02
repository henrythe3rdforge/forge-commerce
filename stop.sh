#!/bin/bash
PID_FILE=".forge-commerce.pid"

if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if kill -0 "$PID" 2>/dev/null; then
        echo "🛑 Stopping Forge Commerce (PID: $PID)..."
        kill "$PID"
        sleep 1
        if kill -0 "$PID" 2>/dev/null; then
            kill -9 "$PID"
        fi
        echo "✅ Stopped"
    else
        echo "⚠️  Process not running"
    fi
    rm -f "$PID_FILE"
else
    echo "⚠️  No PID file found"
fi
