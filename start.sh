#!/bin/bash
set -e

HTMX_VERSION="2.0.4"
HTMX_PATH="static/js/htmx.min.js"
PID_FILE=".forge-commerce.pid"

# Download HTMX if missing
if [ ! -f "$HTMX_PATH" ]; then
    echo "📥 Downloading HTMX ${HTMX_VERSION}..."
    mkdir -p static/js
    curl -sL "https://unpkg.com/htmx.org@${HTMX_VERSION}/dist/htmx.min.js" -o "$HTMX_PATH"
    echo "✅ HTMX downloaded"
fi

# Build release
echo "🔨 Building release..."
cargo build --release

# Find free port
PORT=${PORT:-3000}
while lsof -i :"$PORT" >/dev/null 2>&1; do
    PORT=$((PORT + 1))
done

# Start
echo "🚀 Starting on port $PORT..."
PORT=$PORT ./target/release/forge-commerce &
PID=$!
echo "$PID" > "$PID_FILE"
echo "✅ Running (PID: $PID, Port: $PORT)"
