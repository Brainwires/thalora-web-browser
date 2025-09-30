#!/bin/bash

# MCP Test Runner Script
# Comprehensive testing script for the Thalora MCP server interface

echo "🧠 Thalora MCP Test Suite"
echo "========================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to run a test category
run_test_category() {
    local category=$1
    local description=$2

    echo -e "${BLUE}Running $description...${NC}"

    if cargo test --test mcp_tests "$category" -- --nocapture; then
        echo -e "${GREEN}✓ $description passed${NC}"
        return 0
    else
        echo -e "${RED}✗ $description failed${NC}"
        return 1
    fi
}

# Function to build the project first
build_project() {
    echo -e "${BLUE}Building Thalora MCP server...${NC}"

    if cargo build --quiet; then
        echo -e "${GREEN}✓ Build successful${NC}"
        return 0
    else
        echo -e "${RED}✗ Build failed${NC}"
        return 1
    fi
}

# Function to build release version for performance tests
build_release() {
    echo -e "${BLUE}Building release version for performance tests...${NC}"

    if cargo build --release --quiet; then
        echo -e "${GREEN}✓ Release build successful${NC}"
        return 0
    else
        echo -e "${RED}✗ Release build failed${NC}"
        return 1
    fi
}

# Main execution
main() {
    local failed_tests=0
    local total_categories=5

    echo "Starting comprehensive MCP test suite..."
    echo ""

    # Build the project
    if ! build_project; then
        echo -e "${RED}Cannot continue - build failed${NC}"
        exit 1
    fi
    echo ""

    # Test 1: Test Harness Verification
    if ! run_test_category "test_harness" "Test Harness Verification"; then
        ((failed_tests++))
    fi
    echo ""

    # Test 2: Protocol Compliance Tests
    if ! run_test_category "test_mcp_initialization\|test_tools_list\|test_expected_tools" "Protocol Compliance Tests"; then
        ((failed_tests++))
    fi
    echo ""

    # Test 3: Core Tool Functionality
    if ! run_test_category "test_ai_memory\|test_scrape_url\|test_cdp_runtime" "Core Tool Functionality Tests"; then
        ((failed_tests++))
    fi
    echo ""

    # Test 4: Integration Workflows
    if ! run_test_category "test_research_workflow\|test_browser_automation\|test_data_persistence" "Integration Workflow Tests"; then
        ((failed_tests++))
    fi
    echo ""

    # Test 5: Performance Tests (build release first)
    echo -e "${YELLOW}Performance tests require release build...${NC}"
    if build_release; then
        if ! run_test_category "test_.*_performance\|test_stress_test" "Performance & Stress Tests"; then
            ((failed_tests++))
        fi
    else
        echo -e "${YELLOW}Skipping performance tests due to release build failure${NC}"
        ((failed_tests++))
    fi
    echo ""

    # Summary
    echo "========================="
    echo "Test Suite Summary"
    echo "========================="

    local passed_tests=$((total_categories - failed_tests))

    if [ $failed_tests -eq 0 ]; then
        echo -e "${GREEN}🎉 All test categories passed! ($passed_tests/$total_categories)${NC}"
        echo ""
        echo -e "${GREEN}The MCP server is ready for AI model integration!${NC}"
        echo ""
        echo "Quick start:"
        echo "  ./target/release/thalora  # Start MCP server"
        echo "  echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}' | ./target/release/thalora"
        exit 0
    else
        echo -e "${RED}❌ $failed_tests/$total_categories test categories failed${NC}"
        echo -e "${YELLOW}Review the test output above for details${NC}"
        echo ""
        echo "Debugging tips:"
        echo "  - Run individual tests: cargo test test_name -- --nocapture"
        echo "  - Check MCP server logs: RUST_LOG=debug cargo run"
        echo "  - Verify dependencies: cargo check"
        exit 1
    fi
}

# Help function
show_help() {
    echo "Thalora MCP Test Runner"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -q, --quick    Run quick smoke tests only"
    echo "  -p, --perf     Run performance tests only"
    echo "  -v, --verbose  Run with verbose output"
    echo ""
    echo "Test Categories:"
    echo "  1. Test Harness Verification"
    echo "  2. Protocol Compliance Tests"
    echo "  3. Core Tool Functionality Tests"
    echo "  4. Integration Workflow Tests"
    echo "  5. Performance & Stress Tests"
    echo ""
    echo "Examples:"
    echo "  $0                 # Run all tests"
    echo "  $0 --quick         # Run smoke tests"
    echo "  $0 --perf          # Run performance tests only"
}

# Quick test function
run_quick_tests() {
    echo -e "${BLUE}Running quick smoke tests...${NC}"

    if ! build_project; then
        echo -e "${RED}Cannot continue - build failed${NC}"
        exit 1
    fi

    if cargo test --test mcp_tests "test_harness_functionality\|test_tool_categories_smoke" -- --nocapture; then
        echo -e "${GREEN}✓ Quick tests passed - MCP server basic functionality works${NC}"
        exit 0
    else
        echo -e "${RED}✗ Quick tests failed${NC}"
        exit 1
    fi
}

# Performance test function
run_performance_tests() {
    echo -e "${BLUE}Running performance tests only...${NC}"

    if ! build_release; then
        echo -e "${RED}Cannot continue - release build failed${NC}"
        exit 1
    fi

    if cargo test --release --test mcp_tests "performance" -- --nocapture; then
        echo -e "${GREEN}✓ Performance tests completed${NC}"
        exit 0
    else
        echo -e "${RED}✗ Performance tests failed${NC}"
        exit 1
    fi
}

# Parse command line arguments
case "${1:-}" in
    -h|--help)
        show_help
        exit 0
        ;;
    -q|--quick)
        run_quick_tests
        ;;
    -p|--perf)
        run_performance_tests
        ;;
    -v|--verbose)
        export RUST_LOG=debug
        main
        ;;
    "")
        main
        ;;
    *)
        echo "Unknown option: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
esac