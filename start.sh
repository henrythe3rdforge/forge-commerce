#!/bin/bash
set -e
cd "$(dirname "$0")"

# Build
echo "Building forge-commerce..."
cargo build --release

# Download HTMX if missing
if [ ! -f static/js/htmx.min.js ]; then
    echo "Downloading htmx.min.js..."
    mkdir -p static/js
    curl -sL https://unpkg.com/htmx.org@2.0.4/dist/htmx.min.js -o static/js/htmx.min.js
fi

# Find port
PORT=${PORT:-8000}
while lsof -ti:$PORT >/dev/null 2>&1; do PORT=$((PORT+1)); done

# Start
echo "Starting Forge Market on http://localhost:$PORT"
PORT=$PORT nohup ./target/release/forge-commerce > /tmp/forge-market.log 2>&1 &
echo $! > .forge-commerce.pid
echo "PID: $(cat .forge-commerce.pid)"
