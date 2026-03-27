#!/bin/bash
# Test runner that produces structured JSON reports for AI consumption
# Usage: ./run_tests.sh [--all] [--test <binary>] [--filter <pattern>] [--retest-failures] [--build-only]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
REPORT_FILE="$SCRIPT_DIR/test-report.json"
RAW_OUTPUT="$SCRIPT_DIR/.raw-test-output.txt"
KNOWN_FAILURES="$SCRIPT_DIR/known-failures.json"

# Parse arguments
MODE="all"
TEST_BINARY=""
FILTER=""
BUILD_ONLY=false
EXTRA_ARGS=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --all)
      MODE="all"
      shift
      ;;
    --test)
      MODE="binary"
      TEST_BINARY="$2"
      shift 2
      ;;
    --filter)
      MODE="filter"
      FILTER="$2"
      shift 2
      ;;
    --retest-failures)
      MODE="retest"
      shift
      ;;
    --build-only)
      BUILD_ONLY=true
      shift
      ;;
    --)
      shift
      EXTRA_ARGS="$*"
      break
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--all] [--test <binary>] [--filter <pattern>] [--retest-failures] [--build-only]"
      exit 1
      ;;
  esac
done

cd "$PROJECT_DIR"

# Helper: write JSON report
write_report() {
  local build_success="$1"
  local build_error="$2"
  local total="$3"
  local passed="$4"
  local failed="$5"
  local ignored="$6"
  local failures_json="$7"
  local duration="$8"
  local binaries_run="$9"

  cat > "$REPORT_FILE" <<JSONEOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "build_success": $build_success,
  "build_error": $(echo "$build_error" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read().strip()))' 2>/dev/null || echo '""'),
  "total": $total,
  "passed": $passed,
  "failed": $failed,
  "ignored": $ignored,
  "failures": $failures_json,
  "binaries_run": $binaries_run,
  "duration_secs": $duration
}
JSONEOF
  echo "Report written to: $REPORT_FILE"
}


# Build tests first
echo "=== Building tests... ==="
BUILD_START=$(date +%s)
BUILD_ERROR=""
BUILD_SUCCESS=true

if ! cargo test --no-run 2>"$SCRIPT_DIR/.build-errors.txt"; then
  BUILD_SUCCESS=false
  BUILD_ERROR=$(cat "$SCRIPT_DIR/.build-errors.txt")
  BUILD_END=$(date +%s)
  DURATION=$((BUILD_END - BUILD_START))
  echo "BUILD FAILED"
  write_report "false" "$BUILD_ERROR" "0" "0" "0" "0" "[]" "$DURATION" "[]"
  cat "$SCRIPT_DIR/.build-errors.txt"
  exit 1
fi

if [ "$BUILD_ONLY" = true ]; then
  echo "Build succeeded (build-only mode)"
  exit 0
fi

# Determine what to run
CARGO_TEST_CMD="cargo test"
case $MODE in
  all)
    echo "=== Running all tests... ==="
    CARGO_TEST_CMD="cargo test"
    ;;
  binary)
    echo "=== Running test binary: $TEST_BINARY ==="
    CARGO_TEST_CMD="cargo test --test $TEST_BINARY"
    ;;
  filter)
    echo "=== Running tests matching: $FILTER ==="
    CARGO_TEST_CMD="cargo test $FILTER"
    ;;
  retest)
    if [ ! -f "$REPORT_FILE" ]; then
      echo "No previous report found. Running all tests instead."
      CARGO_TEST_CMD="cargo test"
    else
      # Extract failed test names from previous report
      FAILED_NAMES=$(python3 -c "
import json
with open('$REPORT_FILE') as f:
    report = json.load(f)
for failure in report.get('failures', []):
    print(failure['name'])
" 2>/dev/null)

      if [ -z "$FAILED_NAMES" ]; then
        echo "No failures in previous report. Nothing to retest."
        exit 0
      fi

      # Run each failed test name as a separate filter
      # Cargo test accepts a single filter string that matches test names
      # We use the full qualified name for exact matching
      NUM_FAILED=$(echo "$FAILED_NAMES" | wc -l)
      echo "=== Retesting $NUM_FAILED previously failed tests ==="

      # Collect unique test function names (last segment) for filter
      FILTER_PARTS=$(python3 -c "
import json
with open('$REPORT_FILE') as f:
    report = json.load(f)
names = set()
for failure in report.get('failures', []):
    # Use full name for more specific matching
    names.add(failure['name'])
# Join with | for regex-like matching
# But cargo test filter is a simple substring match, not regex
# Run multiple cargo test invocations or use the function name
# Best approach: use each unique function name
func_names = set()
for failure in report.get('failures', []):
    parts = failure['name'].split('::')
    func_names.add(parts[-1])
print(' '.join(func_names))
" 2>/dev/null)

      echo "Filters: $FILTER_PARTS"
      # Cargo test doesn't support OR filters, but we can use a broad enough pattern
      # Run all tests and let cargo filter by each name
      # Multiple test runs with individual names
      CARGO_TEST_CMD="cargo test"
      # We'll override the command below to run per-filter
      EXTRA_ARGS=""

      # Run with combined approach: run cargo test for each failed test
      TEST_START=$(date +%s)
      > "$RAW_OUTPUT"  # Clear output file
      for FUNC_NAME in $FILTER_PARTS; do
        echo "  Retesting: $FUNC_NAME"
        cargo test "$FUNC_NAME" 2>&1 | tee -a "$RAW_OUTPUT" || true
      done
      TEST_END=$(date +%s)
      DURATION=$((TEST_END - TEST_START))

      # Skip the normal run section below
      CARGO_TEST_CMD="__SKIP__"
    fi
    ;;
esac

# Run tests (skip if retest mode already ran them)
if [ "$CARGO_TEST_CMD" != "__SKIP__" ]; then
  TEST_START=$(date +%s)
  echo "Running: $CARGO_TEST_CMD $EXTRA_ARGS"
  $CARGO_TEST_CMD $EXTRA_ARGS 2>&1 | tee "$RAW_OUTPUT" || true
  TEST_END=$(date +%s)
  DURATION=$((TEST_END - TEST_START))
fi

# Parse output
echo ""
echo "=== Parsing test results... ==="
PARSED=$(RAW_OUTPUT="$RAW_OUTPUT" python3 << 'PYEOF'
import re
import json
import sys
import os

raw_file = os.environ.get("RAW_OUTPUT", "")
if not raw_file:
    print(json.dumps({"total":0,"passed":0,"failed":0,"ignored":0,"failures":[],"binaries":[]}))
    sys.exit(0)

with open(raw_file, 'r', errors='replace') as f:
    content = f.read()

failures = []
total = 0
passed = 0
failed = 0
ignored = 0
binaries = []
binary_for_test = {}

current_binary = "unknown"
lines = content.split('\n')
i = 0

# Phase 1: Parse structured sections (---- <name> stdout ----)
stdout_sections = {}
# Also capture panic messages: thread '<name>' panicked at <file>:<line>
panic_info = {}

while i < len(lines):
    line = lines[i]

    # Detect test binary: "Running tests/<name> (target/...)"
    m = re.match(r'\s*Running\s+(?:tests/|unittests\s+)?(\S+?)(?:\.rs)?\s+\(', line)
    if m:
        parts = m.group(1).split('/')
        if len(parts) > 0:
            current_binary = parts[-1].replace('.rs', '')
        if current_binary not in binaries:
            binaries.append(current_binary)

    # Parse test results: "test <name> ... ok/FAILED/ignored"
    m = re.match(r'^test\s+(\S+)\s+\.\.\.\s+(ok|FAILED|ignored)', line)
    if m:
        test_name = m.group(1)
        result = m.group(2)
        total += 1
        binary_for_test[test_name] = current_binary
        if result == 'ok':
            passed += 1
        elif result == 'FAILED':
            failed += 1
        elif result == 'ignored':
            ignored += 1

    # Capture structured stdout sections (when tests run without --nocapture)
    m = re.match(r'^---- (\S+) stdout ----$', line)
    if m:
        test_name = m.group(1)
        stdout_lines = []
        i += 1
        while i < len(lines):
            if lines[i].startswith('---- ') and lines[i].endswith(' ----'):
                break
            if lines[i].strip() == '' and i + 1 < len(lines) and (
                lines[i+1].startswith('---- ') or lines[i+1].startswith('failures:')):
                break
            stdout_lines.append(lines[i])
            i += 1
        stdout_sections[test_name] = '\n'.join(stdout_lines[-50:])
        continue

    # Capture panic info: thread '<name>' panicked at <file>:<line>
    m = re.match(r"^thread '(.+?)' \(\d+\) panicked at (.+):(\d+):(\d+)", line)
    if not m:
        m = re.match(r"^thread '(.+?)' panicked at (.+):(\d+):(\d+)", line)
    if m:
        panic_test = m.group(1)
        panic_file = m.group(2)
        panic_line = m.group(3)
        # Capture the next few lines as context (assertion message, etc.)
        context_lines = [f"panicked at {panic_file}:{panic_line}"]
        j = i + 1
        while j < len(lines) and j < i + 10:
            ctx = lines[j].strip()
            if ctx == '' or ctx.startswith('thread ') or ctx.startswith('test '):
                break
            context_lines.append(ctx)
            j += 1
        panic_info[panic_test] = '\n'.join(context_lines)

    i += 1

# Phase 2: Build failure list from the "failures:" summary section
failed_names = set()
in_failures_list = False
in_stdout_section = False
for line in lines:
    stripped = line.strip()
    # Skip the "failures:" that introduces stdout sections
    if stripped == 'failures:' and not in_failures_list:
        in_failures_list = True
        continue
    if in_failures_list:
        if stripped.startswith('----'):
            # This is the stdout section header, skip to end
            in_stdout_section = True
            in_failures_list = False
            continue
        if stripped == '' or stripped.startswith('test result:'):
            if not in_stdout_section:
                in_failures_list = False
            continue
        if stripped and not in_stdout_section:
            failed_names.add(stripped)
    if in_stdout_section and stripped == 'failures:':
        # Second "failures:" section - this is the name list
        in_stdout_section = False
        in_failures_list = True

# Phase 3: Build failure objects with best available error info
for test_name in failed_names:
    # Priority: stdout section > panic info > no info
    error = stdout_sections.get(test_name, "")
    if not error or error == "(no stdout captured)":
        error = panic_info.get(test_name, "")
    if not error:
        # Try matching by the last part of the test name
        short_name = test_name.split('::')[-1] if '::' in test_name else test_name
        for pname, pinfo in panic_info.items():
            if short_name in pname or pname in test_name:
                error = pinfo
                break
    if not error:
        error = "(no error details captured)"

    failures.append({
        "name": test_name,
        "binary": binary_for_test.get(test_name, current_binary),
        "error": error[:2000]
    })

result = {
    "total": total,
    "passed": passed,
    "failed": failed,
    "ignored": ignored,
    "failures": failures,
    "binaries": binaries
}

print(json.dumps(result))
PYEOF
)

# Extract values from parsed output
TOTAL=$(echo "$PARSED" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['total'])")
PASSED=$(echo "$PARSED" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['passed'])")
FAILED=$(echo "$PARSED" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['failed'])")
IGNORED=$(echo "$PARSED" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['ignored'])")
FAILURES_JSON=$(echo "$PARSED" | python3 -c "import sys,json; d=json.load(sys.stdin); print(json.dumps(d['failures'], indent=2))")
BINARIES_JSON=$(echo "$PARSED" | python3 -c "import sys,json; d=json.load(sys.stdin); print(json.dumps(d['binaries']))")

# Write report
write_report "true" "" "$TOTAL" "$PASSED" "$FAILED" "$IGNORED" "$FAILURES_JSON" "$DURATION" "$BINARIES_JSON"

# Print summary
echo ""
echo "=== Test Summary ==="
echo "Total: $TOTAL | Passed: $PASSED | Failed: $FAILED | Ignored: $IGNORED"
echo "Duration: ${DURATION}s"

if [ "$FAILED" -gt 0 ]; then
  echo ""
  echo "Failed tests:"
  echo "$FAILURES_JSON" | python3 -c "
import sys, json
failures = json.load(sys.stdin)
for f in failures:
    print(f'  - {f[\"name\"]}')
"
  echo ""
  echo "See $REPORT_FILE for full details."
  exit 1
else
  echo "All tests passed!"
  exit 0
fi
