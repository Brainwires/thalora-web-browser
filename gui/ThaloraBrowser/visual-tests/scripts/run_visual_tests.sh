#!/usr/bin/env bash
#
# run_visual_tests.sh — Visual regression test runner for Thalora browser
#
# Orchestrates: build → launch GUI → for each test case: navigate → screenshot →
# compare → verify elements → report.
#
# Usage:
#   run_visual_tests.sh [--skip-build] [--test <name>] [--threshold 0.85] [--port 9223]
#
# Exit codes:
#   0 = all tests passed
#   1 = one or more tests failed
#   2 = infrastructure error (build failure, GUI won't start, etc.)
#
set -euo pipefail

# --- Defaults ---
SKIP_BUILD=false
TEST_FILTER=""
THRESHOLD=""
PORT=9223
WIDTH=1280
HEIGHT=800

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$VT_DIR/../../../.." && pwd)"
GUI_PROJECT="$PROJECT_ROOT/gui/ThaloraBrowser"
TESTS_DIR="$VT_DIR/tests"
VENV_DIR="$VT_DIR/.venv"
COMPARE_SCRIPT="$SCRIPT_DIR/compare_images.py"
GUI_PID=""

# --- Parse arguments ---
while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-build)  SKIP_BUILD=true; shift ;;
        --test)        TEST_FILTER="$2"; shift 2 ;;
        --threshold)   THRESHOLD="$2"; shift 2 ;;
        --port)        PORT="$2"; shift 2 ;;
        *)             echo "Unknown option: $1" >&2; exit 2 ;;
    esac
done

# --- Counters ---
TOTAL=0
PASSED=0
FAILED=0
FAILURES=()

# --- Colors ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'  # No Color

# --- Cleanup on exit ---
cleanup() {
    if [ -n "$GUI_PID" ] && kill -0 "$GUI_PID" 2>/dev/null; then
        echo -e "${CYAN}[runner]${NC} Stopping GUI (PID $GUI_PID)..."
        # Try graceful shutdown first
        curl -sf -X POST "http://localhost:$PORT/shutdown" >/dev/null 2>&1 || true
        sleep 1
        if kill -0 "$GUI_PID" 2>/dev/null; then
            kill "$GUI_PID" 2>/dev/null || true
            wait "$GUI_PID" 2>/dev/null || true
        fi
    fi
}
trap cleanup EXIT INT TERM

# --- Step 1: Set up Python venv for image comparison ---
echo -e "${CYAN}[runner]${NC} Setting up Python environment..."
if [ ! -d "$VENV_DIR" ]; then
    python3 -m venv "$VENV_DIR"
fi
# Activate and install deps (quietly)
source "$VENV_DIR/bin/activate"
pip install --quiet Pillow numpy 2>&1 | grep -v "already satisfied" || true

# --- Step 2: Build (unless --skip-build) ---
if [ "$SKIP_BUILD" = false ]; then
    echo -e "${CYAN}[runner]${NC} Building Rust library..."
    (cd "$PROJECT_ROOT" && cargo build 2>&1) || {
        echo -e "${RED}[runner] ERROR: cargo build failed${NC}" >&2
        exit 2
    }
    echo -e "${CYAN}[runner]${NC} Building C# GUI..."
    (cd "$PROJECT_ROOT" && dotnet build gui/ThaloraBrowser 2>&1) || {
        echo -e "${RED}[runner] ERROR: dotnet build failed${NC}" >&2
        exit 2
    }
    dotnet build-server shutdown >/dev/null 2>&1 || true
else
    echo -e "${CYAN}[runner]${NC} Skipping build (--skip-build)"
fi

# --- Step 3: Kill any existing instance on the port ---
EXISTING_PID=$(lsof -ti "tcp:$PORT" 2>/dev/null || true)
if [ -n "$EXISTING_PID" ]; then
    echo -e "${YELLOW}[runner]${NC} Killing existing process on port $PORT (PID $EXISTING_PID)..."
    kill "$EXISTING_PID" 2>/dev/null || true
    sleep 1
fi

# --- Step 4: Find test cases ---
if [ ! -d "$TESTS_DIR" ]; then
    echo -e "${RED}[runner] ERROR: Tests directory not found: $TESTS_DIR${NC}" >&2
    exit 2
fi

TEST_DIRS=()
for dir in "$TESTS_DIR"/*/; do
    [ -f "$dir/config.json" ] || continue
    TEST_NAME="$(basename "$dir")"
    if [ -n "$TEST_FILTER" ] && [ "$TEST_NAME" != "$TEST_FILTER" ]; then
        continue
    fi
    TEST_DIRS+=("$dir")
done

if [ ${#TEST_DIRS[@]} -eq 0 ]; then
    echo -e "${YELLOW}[runner]${NC} No test cases found."
    exit 0
fi

echo -e "${CYAN}[runner]${NC} Found ${#TEST_DIRS[@]} test case(s)"

# --- Step 5: Launch GUI (once for all tests) ---
# We navigate to a blank page first, then navigate per-test
echo -e "${CYAN}[runner]${NC} Launching GUI on port $PORT..."
(cd "$GUI_PROJECT" && dotnet run -- \
    --url "about:blank" \
    --control-port "$PORT" \
    --width "$WIDTH" \
    --height "$HEIGHT" \
    2>&1 | while IFS= read -r line; do echo -e "${CYAN}[gui]${NC} $line"; done) &
GUI_PID=$!

# Wait for health
echo -e "${CYAN}[runner]${NC} Waiting for GUI to start..."
HEALTH_TIMEOUT=60
HEALTH_ELAPSED=0
while [ $HEALTH_ELAPSED -lt $HEALTH_TIMEOUT ]; do
    if curl -sf "http://localhost:$PORT/health" >/dev/null 2>&1; then
        echo -e "${GREEN}[runner]${NC} GUI is ready."
        break
    fi
    sleep 1
    HEALTH_ELAPSED=$((HEALTH_ELAPSED + 1))
done

if [ $HEALTH_ELAPSED -ge $HEALTH_TIMEOUT ]; then
    echo -e "${RED}[runner] ERROR: GUI did not start within ${HEALTH_TIMEOUT}s${NC}" >&2
    exit 2
fi

# --- Step 6: Run each test case ---
for test_dir in "${TEST_DIRS[@]}"; do
    TEST_NAME="$(basename "$test_dir")"
    CONFIG="$test_dir/config.json"
    CAPTURES_DIR="$test_dir/captures"
    mkdir -p "$CAPTURES_DIR"

    echo ""
    echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}  Test: $TEST_NAME${NC}"
    echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    # Parse config
    URL=$(python3 -c "import json; c=json.load(open('$CONFIG')); print(c['url'])")
    WAIT_MS=$(python3 -c "import json; c=json.load(open('$CONFIG')); print(c.get('wait_after_load_ms', 3000))")
    TEST_THRESHOLD=$(python3 -c "import json; c=json.load(open('$CONFIG')); print(c.get('screenshot_comparison',{}).get('threshold', 0.85))")

    # Use CLI threshold override if provided
    if [ -n "$THRESHOLD" ]; then
        TEST_THRESHOLD="$THRESHOLD"
    fi

    TEST_PASSED=true

    # 6a: Navigate to URL
    echo -e "${CYAN}[test]${NC} Navigating to: $URL"
    NAV_RESULT=$(curl -sf -X POST "http://localhost:$PORT/navigate" \
        -H "Content-Type: application/json" \
        -d "{\"url\": \"$URL\", \"timeout_ms\": 60000}" 2>&1) || {
        echo -e "${RED}[test]${NC} Navigation failed"
        TEST_PASSED=false
    }

    if [ "$TEST_PASSED" = true ]; then
        # 6b: Wait for images
        echo -e "${CYAN}[test]${NC} Waiting ${WAIT_MS}ms for async content..."
        curl -sf -X POST "http://localhost:$PORT/wait-for-images" \
            -H "Content-Type: application/json" \
            -d "{\"wait_ms\": $WAIT_MS}" >/dev/null 2>&1 || true

        # 6c: Verify expected elements
        ELEMENTS_JSON=$(python3 -c "import json; c=json.load(open('$CONFIG')); print(json.dumps(c.get('expected_elements', [])))")
        ELEMENT_COUNT=$(python3 -c "import json; print(len(json.loads('$ELEMENTS_JSON')))")

        if [ "$ELEMENT_COUNT" -gt 0 ]; then
            echo -e "${CYAN}[test]${NC} Verifying $ELEMENT_COUNT expected elements..."

            # Iterate over expected elements
            for i in $(seq 0 $((ELEMENT_COUNT - 1))); do
                TOTAL=$((TOTAL + 1))
                ELEM_DESC=$(python3 -c "import json; elems=json.loads('$ELEMENTS_JSON'); print(elems[$i]['description'])")
                CRITERIA=$(python3 -c "import json; elems=json.loads('$ELEMENTS_JSON'); print(json.dumps(elems[$i]['criteria']))")

                FIND_RESULT=$(curl -sf -X POST "http://localhost:$PORT/find-element" \
                    -H "Content-Type: application/json" \
                    -d "$CRITERIA" 2>&1) || FIND_RESULT='{"elements":[],"total":0}'

                FOUND=$(python3 -c "import json; r=json.loads('''$FIND_RESULT'''); print(r.get('total', 0))")

                if [ "$FOUND" -gt 0 ]; then
                    echo -e "  ${GREEN}PASS${NC} Element found: $ELEM_DESC (${FOUND} match(es))"
                    PASSED=$((PASSED + 1))
                else
                    echo -e "  ${RED}FAIL${NC} Element NOT found: $ELEM_DESC"
                    FAILED=$((FAILED + 1))
                    FAILURES+=("$TEST_NAME: Element not found — $ELEM_DESC")
                    TEST_PASSED=false
                fi
            done
        fi

        # 6d: Screenshot comparison (if enabled)
        SCREENSHOT_ENABLED=$(python3 -c "import json; c=json.load(open('$CONFIG')); print(c.get('screenshot_comparison',{}).get('enabled', False))")
        if [ "$SCREENSHOT_ENABLED" = "True" ]; then
            TOTAL=$((TOTAL + 1))

            REF_DIR_REL=$(python3 -c "import json; c=json.load(open('$CONFIG')); print(c.get('screenshot_comparison',{}).get('reference_dir', ''))")
            REF_FILE=$(python3 -c "import json; c=json.load(open('$CONFIG')); print(c.get('screenshot_comparison',{}).get('reference_file', 'page-01.png'))")

            # Resolve reference path relative to config file
            REF_PATH="$(cd "$test_dir" && cd "$REF_DIR_REL" && pwd)/$REF_FILE"

            if [ ! -f "$REF_PATH" ]; then
                echo -e "  ${YELLOW}SKIP${NC} Screenshot comparison: reference not found ($REF_PATH)"
                # Don't count as failure — reference might not be captured yet
                TOTAL=$((TOTAL - 1))
            else
                # Capture screenshot
                CAPTURE_PATH="$CAPTURES_DIR/viewport-01.png"
                curl -sf "http://localhost:$PORT/screenshot?delay=500" -o "$CAPTURE_PATH" 2>/dev/null

                if [ -f "$CAPTURE_PATH" ]; then
                    DIFF_PATH="$CAPTURES_DIR/diff-01.png"
                    echo -e "${CYAN}[test]${NC} Comparing screenshot (threshold: $TEST_THRESHOLD)..."

                    COMPARE_OUTPUT=$(python3 "$COMPARE_SCRIPT" "$CAPTURE_PATH" "$REF_PATH" \
                        --threshold "$TEST_THRESHOLD" \
                        --diff-output "$DIFF_PATH" 2>&1) || true

                    echo "  $COMPARE_OUTPUT"

                    if echo "$COMPARE_OUTPUT" | grep -q "\[PASS\]"; then
                        PASSED=$((PASSED + 1))
                    else
                        FAILED=$((FAILED + 1))
                        FAILURES+=("$TEST_NAME: Screenshot comparison failed")
                        TEST_PASSED=false
                    fi
                else
                    echo -e "  ${RED}FAIL${NC} Screenshot capture failed"
                    FAILED=$((FAILED + 1))
                    FAILURES+=("$TEST_NAME: Screenshot capture failed")
                    TEST_PASSED=false
                fi
            fi
        fi
    else
        # Navigation failed — count all expected elements as failures
        ELEMENT_COUNT=$(python3 -c "import json; c=json.load(open('$CONFIG')); print(len(c.get('expected_elements', [])))")
        TOTAL=$((TOTAL + ELEMENT_COUNT))
        FAILED=$((FAILED + ELEMENT_COUNT))
        for i in $(seq 0 $((ELEMENT_COUNT - 1))); do
            ELEM_DESC=$(python3 -c "import json; c=json.load(open('$CONFIG')); print(c['expected_elements'][$i]['description'])")
            FAILURES+=("$TEST_NAME: Skipped (navigation failed) — $ELEM_DESC")
        done
    fi

    # Test case summary
    if [ "$TEST_PASSED" = true ]; then
        echo -e "${GREEN}[test]${NC} $TEST_NAME: ${GREEN}ALL CHECKS PASSED${NC}"
    else
        echo -e "${RED}[test]${NC} $TEST_NAME: ${RED}SOME CHECKS FAILED${NC}"
    fi
done

# --- Step 7: Final report ---
echo ""
echo -e "${BOLD}════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}  Visual Regression Test Results${NC}"
echo -e "${BOLD}════════════════════════════════════════════════════${NC}"
echo -e "  Total checks:  $TOTAL"
echo -e "  ${GREEN}Passed:${NC}        $PASSED"
echo -e "  ${RED}Failed:${NC}        $FAILED"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}  Failures:${NC}"
    for f in "${FAILURES[@]}"; do
        echo -e "    - $f"
    done
    echo ""
    echo -e "${RED}${BOLD}  RESULT: FAIL${NC}"
    exit 1
else
    echo -e "${GREEN}${BOLD}  RESULT: PASS${NC}"
    exit 0
fi
