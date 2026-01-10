#!/bin/bash
# Test execution and coverage script for TSI Rust backend
# Usage: ./run_tests.sh [--coverage] [--module MODULE_NAME]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse arguments
RUN_COVERAGE=false
MODULE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --coverage)
            RUN_COVERAGE=true
            shift
            ;;
        --module)
            MODULE="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--coverage] [--module MODULE_NAME]"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}=== TSI Rust Backend Test Suite ===${NC}\n"

# Run tests
if [ -n "$MODULE" ]; then
    echo -e "${YELLOW}Running tests for module: $MODULE${NC}"
    cargo test --lib "$MODULE" -- --nocapture
else
    echo -e "${YELLOW}Running all library tests${NC}"
    cargo test --lib -- --nocapture
fi

TEST_EXIT_CODE=$?

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "\n${GREEN}✓ All tests passed!${NC}\n"
else
    echo -e "\n${RED}✗ Some tests failed${NC}\n"
    exit $TEST_EXIT_CODE
fi

# Run coverage if requested
if [ "$RUN_COVERAGE" = true ]; then
    echo -e "${YELLOW}=== Generating Coverage Report ===${NC}\n"
    
    # Check if cargo-tarpaulin is installed
    if ! command -v cargo-tarpaulin &> /dev/null; then
        echo -e "${YELLOW}Installing cargo-tarpaulin...${NC}"
        cargo install cargo-tarpaulin
    fi
    
    # Generate coverage
    echo -e "${YELLOW}Running coverage analysis...${NC}"
    cargo tarpaulin \
        --ignore-tests \
        --out Html \
        --out Xml \
        --output-dir coverage/ \
        --exclude-files "target/*" "siderust/*" "tests/*"
    
    COVERAGE_EXIT_CODE=$?
    
    if [ $COVERAGE_EXIT_CODE -eq 0 ]; then
        echo -e "\n${GREEN}✓ Coverage report generated${NC}"
        echo -e "HTML report: ${YELLOW}coverage/index.html${NC}"
        echo -e "XML report: ${YELLOW}coverage/cobertura.xml${NC}\n"
        
        # Extract coverage percentage
        COVERAGE_PERCENT=$(grep -oP '\d+\.\d+%' coverage/index.html | head -1)
        echo -e "Current coverage: ${GREEN}$COVERAGE_PERCENT${NC}"
        
        # Check threshold
        COVERAGE_NUM=$(echo "$COVERAGE_PERCENT" | tr -d '%')
        if (( $(echo "$COVERAGE_NUM >= 80" | bc -l) )); then
            echo -e "${GREEN}✓ Coverage meets 80% threshold${NC}\n"
        else
            echo -e "${YELLOW}⚠ Coverage below 80% threshold${NC}\n"
        fi
    else
        echo -e "\n${RED}✗ Coverage generation failed${NC}\n"
        exit $COVERAGE_EXIT_CODE
    fi
fi

echo -e "${GREEN}=== Test Summary ===${NC}"
echo "Test files implemented:"
echo "  ✓ parsing/json_parser_tests.rs (15 tests)"
echo "  ✓ parsing/csv_parser_tests.rs (18 tests)"
echo "  ✓ parsing/dark_periods_parser_tests.rs (31 tests)"
echo "  ✓ io/loaders_tests.rs (21 tests)"
echo ""
echo "Total: 85 tests implemented"
echo ""
echo "See TEST_COVERAGE_REPORT.md for detailed documentation"
echo "See TEST_IMPLEMENTATION_GUIDE.md for implementing remaining tests"
