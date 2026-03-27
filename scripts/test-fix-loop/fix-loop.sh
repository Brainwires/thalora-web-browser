#!/bin/bash
# Test-Fix Loop — Autonomous AI test fixing
# Usage: ./fix-loop.sh [--tool amp|claude] [max_iterations]
#
# Runs Claude Code (or amp) in a loop to fix test failures.
# Each iteration: run tests → identify failure → fix → retest → commit.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
REPORT_FILE="$SCRIPT_DIR/test-report.json"

# Parse arguments
TOOL="claude"
MAX_ITERATIONS=10

while [[ $# -gt 0 ]]; do
  case $1 in
    --tool)
      TOOL="$2"
      shift 2
      ;;
    --tool=*)
      TOOL="${1#*=}"
      shift
      ;;
    *)
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        MAX_ITERATIONS="$1"
      fi
      shift
      ;;
  esac
done

if [[ "$TOOL" != "amp" && "$TOOL" != "claude" ]]; then
  echo "Error: Invalid tool '$TOOL'. Must be 'amp' or 'claude'."
  exit 1
fi

cd "$PROJECT_DIR"

# Initialize progress file if needed
if [ ! -f "$PROGRESS_FILE" ]; then
  cat > "$PROGRESS_FILE" << 'EOF'
# Test-Fix Loop Progress Log

## Codebase Patterns
- Wasmtime compilation is slow — never use timeouts on cargo commands
- Pre-existing known failure: test_encrypt_different_each_time (crypto test)
- MCP tests use subprocess harness (McpTestHarness) — errors may be server-side
- Feature flags: THALORA_ENABLE_* environment variables control feature gating
- Test files under tests/protocols/ are compiled as part of tests/mcp_tests.rs binary

---

EOF
  echo "Initialized progress.txt"
fi

# Run initial test suite to establish baseline
echo "=== Running initial test suite for baseline ==="
"$SCRIPT_DIR/run_tests.sh" --all || true

if [ -f "$REPORT_FILE" ]; then
  INITIAL_FAILURES=$(python3 -c "
import json
with open('$REPORT_FILE') as f:
    r = json.load(f)
print(r.get('failed', 0))
" 2>/dev/null || echo "?")
  echo "Baseline: $INITIAL_FAILURES failures detected"
else
  echo "Warning: No test report generated. Build may have failed."
fi

echo ""
echo "Starting Test-Fix Loop — Tool: $TOOL — Max iterations: $MAX_ITERATIONS"

for i in $(seq 1 $MAX_ITERATIONS); do
  echo ""
  echo "==============================================================="
  echo "  Test-Fix Iteration $i of $MAX_ITERATIONS ($TOOL)"
  echo "==============================================================="

  if [[ "$TOOL" == "amp" ]]; then
    OUTPUT=$(cat "$SCRIPT_DIR/CLAUDE.md" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
  else
    OUTPUT=$(claude --dangerously-skip-permissions --print < "$SCRIPT_DIR/CLAUDE.md" 2>&1 | tee /dev/stderr) || true
  fi

  # Check for completion
  if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
    echo ""
    echo "=== ALL TESTS FIXED ==="
    echo "Completed at iteration $i of $MAX_ITERATIONS"

    # Print final stats
    if [ -f "$REPORT_FILE" ]; then
      python3 -c "
import json
with open('$REPORT_FILE') as f:
    r = json.load(f)
print(f'Final: {r[\"passed\"]} passed, {r[\"failed\"]} failed, {r[\"ignored\"]} ignored')
" 2>/dev/null || true
    fi
    exit 0
  fi

  echo "Iteration $i complete. Continuing..."
  sleep 2
done

echo ""
echo "Reached max iterations ($MAX_ITERATIONS) without fixing all tests."
echo "Check $PROGRESS_FILE for status and remaining issues."

# Print current state
if [ -f "$REPORT_FILE" ]; then
  echo ""
  python3 -c "
import json
with open('$REPORT_FILE') as f:
    r = json.load(f)
print(f'Current state: {r[\"passed\"]} passed, {r[\"failed\"]} failed, {r[\"ignored\"]} ignored')
if r['failures']:
    print('Remaining failures:')
    for f in r['failures']:
        print(f'  - {f[\"name\"]}')
" 2>/dev/null || true
fi

exit 1
