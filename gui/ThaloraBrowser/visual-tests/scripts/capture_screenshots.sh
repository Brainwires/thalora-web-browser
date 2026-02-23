#!/usr/bin/env bash
#
# capture_screenshots.sh — Capture scrolling viewport screenshots from Thalora browser
#
# Usage: capture_screenshots.sh <url> [output_dir] [port] [width] [height]
#
# Builds the Rust library, launches the GUI, navigates to the URL,
# scrolls through the entire page, and saves a screenshot at each viewport offset.
#
set -euo pipefail

# --- Arguments ---
URL="${1:?Usage: capture_screenshots.sh <url> [output_dir] [port] [width] [height]}"
OUTPUT_DIR="${2:-$(dirname "$0")/../captures/cloudflare-blog}"
PORT="${3:-9222}"
WIDTH="${4:-1280}"
HEIGHT="${5:-800}"

PROJECT_ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
GUI_PROJECT="$PROJECT_ROOT/gui/ThaloraBrowser"
GUI_PID=""

# --- Cleanup on exit ---
cleanup() {
    if [ -n "$GUI_PID" ] && kill -0 "$GUI_PID" 2>/dev/null; then
        echo "[capture] Stopping GUI (PID $GUI_PID)..."
        kill "$GUI_PID" 2>/dev/null || true
        wait "$GUI_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT INT TERM

# --- Step 1: Build Rust library ---
echo "[capture] Building Rust library..."
(cd "$PROJECT_ROOT" && cargo build 2>&1) || {
    echo "[capture] ERROR: cargo build failed" >&2
    exit 1
}

# --- Step 2: Kill any existing instance on the control port ---
echo "[capture] Checking for existing instances on port $PORT..."
EXISTING_PID=$(lsof -ti "tcp:$PORT" 2>/dev/null || true)
if [ -n "$EXISTING_PID" ]; then
    echo "[capture] Killing existing process on port $PORT (PID $EXISTING_PID)..."
    kill "$EXISTING_PID" 2>/dev/null || true
    sleep 1
fi

# --- Step 3: Launch GUI ---
echo "[capture] Launching GUI: --url $URL --control-port $PORT --width $WIDTH --height $HEIGHT"
(cd "$GUI_PROJECT" && dotnet run -- \
    --url "$URL" \
    --control-port "$PORT" \
    --width "$WIDTH" \
    --height "$HEIGHT" \
    2>&1 | while IFS= read -r line; do echo "[gui] $line"; done) &
GUI_PID=$!
echo "[capture] GUI launched (PID $GUI_PID)"

# --- Step 4: Wait for /health ---
echo "[capture] Waiting for control server on port $PORT..."
HEALTH_TIMEOUT=60
HEALTH_ELAPSED=0
while [ $HEALTH_ELAPSED -lt $HEALTH_TIMEOUT ]; do
    if curl -sf "http://localhost:$PORT/health" >/dev/null 2>&1; then
        echo "[capture] Control server is ready."
        break
    fi
    sleep 1
    HEALTH_ELAPSED=$((HEALTH_ELAPSED + 1))
done

if [ $HEALTH_ELAPSED -ge $HEALTH_TIMEOUT ]; then
    echo "[capture] ERROR: Timed out waiting for control server" >&2
    exit 1
fi

# --- Step 5: Wait for page load ---
echo "[capture] Waiting for page to finish loading..."
LOAD_TIMEOUT=60
LOAD_ELAPSED=0
while [ $LOAD_ELAPSED -lt $LOAD_TIMEOUT ]; do
    IS_LOADING=$(curl -sf "http://localhost:$PORT/state" 2>/dev/null | python3 -c "import sys,json; print(json.load(sys.stdin).get('is_loading', True))" 2>/dev/null || echo "True")
    if [ "$IS_LOADING" = "False" ]; then
        echo "[capture] Page loaded."
        break
    fi
    sleep 1
    LOAD_ELAPSED=$((LOAD_ELAPSED + 1))
done

if [ $LOAD_ELAPSED -ge $LOAD_TIMEOUT ]; then
    echo "[capture] WARNING: Page did not finish loading within ${LOAD_TIMEOUT}s, continuing anyway..." >&2
fi

# --- Step 6: Wait for images ---
echo "[capture] Waiting 5s for async image loads..."
curl -sf -X POST "http://localhost:$PORT/wait-for-images" \
    -H "Content-Type: application/json" \
    -d '{"wait_ms": 5000}' >/dev/null 2>&1 || echo "[capture] WARNING: wait-for-images failed"

# --- Step 7: Get content dimensions ---
echo "[capture] Getting content dimensions..."
DIMS_JSON=$(curl -sf "http://localhost:$PORT/content-height" 2>/dev/null)
if [ -z "$DIMS_JSON" ]; then
    echo "[capture] ERROR: Failed to get content dimensions" >&2
    exit 1
fi

CONTENT_HEIGHT=$(echo "$DIMS_JSON" | python3 -c "import sys,json; print(json.load(sys.stdin)['content_height'])")
VIEWPORT_HEIGHT=$(echo "$DIMS_JSON" | python3 -c "import sys,json; print(json.load(sys.stdin)['viewport_height'])")
VIEWPORT_WIDTH=$(echo "$DIMS_JSON" | python3 -c "import sys,json; print(json.load(sys.stdin)['viewport_width'])")

echo "[capture] Content: ${CONTENT_HEIGHT}px tall, Viewport: ${VIEWPORT_WIDTH}x${VIEWPORT_HEIGHT}"

# --- Step 8: Prepare output directory ---
mkdir -p "$OUTPUT_DIR"
rm -f "$OUTPUT_DIR"/viewport-*.png "$OUTPUT_DIR"/metadata.json

# --- Step 9: Scroll + screenshot loop ---
SCROLL_Y=0
VIEWPORT_NUM=1
SCROLL_STEP=$(python3 -c "print(int($VIEWPORT_HEIGHT))")

echo "[capture] Starting capture loop (step=${SCROLL_STEP}px)..."

while true; do
    # Scroll to position
    curl -sf -X POST "http://localhost:$PORT/scroll" \
        -H "Content-Type: application/json" \
        -d "{\"y\": $SCROLL_Y}" >/dev/null 2>&1

    # Small settle delay
    sleep 0.3

    # Capture screenshot
    FILENAME=$(printf "viewport-%02d.png" "$VIEWPORT_NUM")
    FILEPATH="$OUTPUT_DIR/$FILENAME"

    curl -sf "http://localhost:$PORT/screenshot?delay=200" -o "$FILEPATH" 2>/dev/null
    echo "[capture]   $FILENAME (scroll_y=$SCROLL_Y)"

    # Check if we've reached the bottom
    NEXT_Y=$((SCROLL_Y + SCROLL_STEP))
    # Use python3 for float comparison
    AT_BOTTOM=$(python3 -c "print('yes' if $SCROLL_Y >= $CONTENT_HEIGHT - $VIEWPORT_HEIGHT else 'no')")
    if [ "$AT_BOTTOM" = "yes" ]; then
        break
    fi

    SCROLL_Y=$NEXT_Y
    VIEWPORT_NUM=$((VIEWPORT_NUM + 1))
done

# --- Step 10: Save metadata ---
cat > "$OUTPUT_DIR/metadata.json" <<METAEOF
{
    "url": "$URL",
    "content_height": $CONTENT_HEIGHT,
    "viewport_width": $VIEWPORT_WIDTH,
    "viewport_height": $VIEWPORT_HEIGHT,
    "num_viewports": $VIEWPORT_NUM,
    "scroll_step": $SCROLL_STEP,
    "captured_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
METAEOF

echo ""
echo "[capture] Done! $VIEWPORT_NUM screenshots saved to: $OUTPUT_DIR"
echo "[capture] Metadata: $OUTPUT_DIR/metadata.json"
