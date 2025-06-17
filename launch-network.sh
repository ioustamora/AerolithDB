#!/bin/bash
# AerolithDB Multinode Test Network Launcher (Bash)
# Cross-platform bash script to easily launch multinode test networks

set -euo pipefail

# Default values
TEST_TYPE="full"
NODES_COUNT=0
VERBOSE=false
TEST_DURATION=300
HELP=false

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Usage function
show_help() {
    echo -e "${CYAN}🚀 AerolithDB Multinode Test Network Launcher${NC}"
    echo -e "${CYAN}=============================================${NC}"
    echo ""
    echo -e "${YELLOW}USAGE:${NC}"
    echo "  ./launch-network.sh [OPTIONS]"
    echo ""
    echo -e "${YELLOW}TEST TYPES:${NC}"
    echo "  quick     - 3 nodes, basic functionality demo (2-3 minutes)"
    echo "  full      - 4+ nodes, comprehensive testing (5-10 minutes)"
    echo "  advanced  - 6+ nodes, all features + stress testing (10+ minutes)"
    echo ""
    echo -e "${YELLOW}OPTIONS:${NC}"
    echo "  -t, --type TYPE       Test type [quick|full|advanced] (default: full)"
    echo "  -n, --nodes COUNT     Number of nodes (auto-selected if not specified)"
    echo "  -v, --verbose         Enable detailed logging"
    echo "  -d, --duration SEC    Duration for advanced tests in seconds (default: 300)"
    echo "  -h, --help            Show this help message"
    echo ""
    echo -e "${YELLOW}EXAMPLES:${NC}"
    echo "  ./launch-network.sh                         # Full test with 4 nodes"
    echo "  ./launch-network.sh -t quick                # Quick 3-node demo"
    echo "  ./launch-network.sh -t advanced -v          # Advanced with logging"
    echo "  ./launch-network.sh -n 8 -v                 # Custom 8-node network"
    echo ""
    echo -e "${YELLOW}AFTER LAUNCH:${NC}"
    echo "  • Web UI: http://localhost:8080"
    echo "  • Network Explorer: http://localhost:8080/explorer"
    echo "  • CLI: cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 health"
    echo ""
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--type)
            TEST_TYPE="$2"
            shift 2
            ;;
        -n|--nodes)
            NODES_COUNT="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -d|--duration)
            TEST_DURATION="$2"
            shift 2
            ;;
        -h|--help)
            HELP=true
            shift
            ;;
        *)
            echo -e "${RED}❌ Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# Show help if requested
if [ "$HELP" = true ]; then
    show_help
    exit 0
fi

# Validate test type
if [[ ! "$TEST_TYPE" =~ ^(quick|full|advanced)$ ]]; then
    echo -e "${RED}❌ Invalid test type: $TEST_TYPE${NC}"
    echo -e "${YELLOW}Valid options: quick, full, advanced${NC}"
    exit 1
fi

# Check prerequisites
check_prerequisites() {
    echo -e "${BLUE}🔍 Checking prerequisites...${NC}"
    
    # Check if Cargo is available
    if ! command -v cargo >/dev/null 2>&1; then
        echo -e "${RED}❌ Cargo (Rust) is required but not found${NC}"
        echo -e "${YELLOW}   Please install Rust: https://rustup.rs/${NC}"
        return 1
    fi
    
    local cargo_version=$(cargo --version)
    echo -e "${GREEN}✅ Cargo: $cargo_version${NC}"
    
    # Check if in AerolithDB directory
    if [ ! -f "Cargo.toml" ]; then
        echo -e "${RED}❌ Not in AerolithDB root directory${NC}"
        echo -e "${YELLOW}   Please run this script from the AerolithDB project root${NC}"
        return 1
    fi
    
    # Check if scripts exist
    local required_scripts=(
        "scripts/quick-demo.sh"
        "scripts/launch-local-network.sh"
        "scripts/advanced-network-test.sh"
    )
    
    for script in "${required_scripts[@]}"; do
        if [ ! -f "$script" ]; then
            echo -e "${RED}❌ Required script not found: $script${NC}"
            return 1
        fi
    done
    
    echo -e "${GREEN}✅ All prerequisites met${NC}"
    return 0
}

# Get default node count based on test type
get_default_node_count() {
    case "$1" in
        "quick") echo 3 ;;
        "full") echo 4 ;;
        "advanced") echo 6 ;;
        *) echo 4 ;;
    esac
}

# Show test information
show_test_info() {
    local test_type="$1"
    local node_count="$2"
    local duration="$3"
    
    echo ""
    echo -e "${CYAN}🎯 Test Configuration${NC}"
    echo -e "${CYAN}===================${NC}"
    echo "Test Type: $test_type"
    echo "Node Count: $node_count"
    
    case "$test_type" in
        "quick")
            echo "Duration: ~2-3 minutes"
            echo ""
            echo -e "${YELLOW}Features Tested:${NC}"
            echo "  • Basic network formation"
            echo "  • Simple CRUD operations"
            echo "  • Cross-node replication"
            echo "  • Health monitoring"
            ;;
        "full")
            echo "Duration: ~5-10 minutes"
            echo ""
            echo -e "${YELLOW}Features Tested:${NC}"
            echo "  • Complete network formation (bootstrap + $((node_count-1)) nodes)"
            echo "  • Full CRUD operations across nodes"
            echo "  • Distributed queries and analytics"
            echo "  • User simulation and admin operations"
            echo "  • Cross-node data consistency"
            echo "  • Health monitoring and statistics"
            ;;
        "advanced")
            echo "Duration: ~${duration} seconds + setup"
            echo ""
            echo -e "${YELLOW}Features Tested:${NC}"
            echo "  • All full test features PLUS:"
            echo "  • Byzantine fault tolerance"
            echo "  • Network partition recovery"
            echo "  • Cross-datacenter replication"
            echo "  • Load testing and performance"
            echo "  • Security and encryption"
            echo "  • Compliance and governance"
            ;;
    esac
    
    echo ""
    echo -e "${YELLOW}Network Endpoints (after launch):${NC}"
    for ((i=0; i<node_count; i++)); do
        local port=$((8080 + i))
        local node_type
        if [ $i -eq 0 ]; then
            node_type="Bootstrap"
        else
            node_type="Node $i"
        fi
        echo "  • $node_type : http://localhost:$port"
    done
    echo ""
}

# Launch test
launch_test() {
    local test_type="$1"
    local node_count="$2"
    local verbose_mode="$3"
    local duration="$4"
    
    local script_path
    local script_args=()
    
    case "$test_type" in
        "quick")
            echo -e "${GREEN}🚀 Launching Quick Demo...${NC}"
            script_path="scripts/quick-demo.sh"
            ;;
        "full")
            echo -e "${GREEN}🚀 Launching Full Network Demo...${NC}"
            script_path="scripts/launch-local-network.sh"
            script_args+=("-n" "$node_count")
            if [ "$verbose_mode" = true ]; then
                script_args+=("-v")
            fi
            ;;
        "advanced")
            echo -e "${GREEN}🚀 Launching Advanced Network Test...${NC}"
            script_path="scripts/advanced-network-test.sh"
            script_args+=("$node_count" "advanced-test" "debug" "$duration")
            if [ "$verbose_mode" = true ]; then
                script_args+=("true")
            else
                script_args+=("false")
            fi
            ;;
    esac
    
    echo -e "${BLUE}Executing: $script_path ${script_args[*]}${NC}"
    echo ""
    
    # Make script executable if needed
    chmod +x "$script_path"
    
    # Execute the script
    if [ ${#script_args[@]} -gt 0 ]; then
        "$script_path" "${script_args[@]}"
    else
        "$script_path"
    fi
}

# Main execution
echo -e "${MAGENTA}🏗️  AerolithDB Multinode Test Network Launcher${NC}"
echo -e "${MAGENTA}===============================================${NC}"

# Check prerequisites
if ! check_prerequisites; then
    exit 1
fi

# Set default node count if not specified
if [ "$NODES_COUNT" -eq 0 ]; then
    NODES_COUNT=$(get_default_node_count "$TEST_TYPE")
fi

# Show test information
show_test_info "$TEST_TYPE" "$NODES_COUNT" "$TEST_DURATION"

# Confirm launch
echo -e "${YELLOW}Ready to launch? (Press Enter to continue, Ctrl+C to cancel)${NC}"
read -r

# Launch the test
if launch_test "$TEST_TYPE" "$NODES_COUNT" "$VERBOSE" "$TEST_DURATION"; then
    echo ""
    echo -e "${GREEN}🎉 Test network launched successfully!${NC}"
    echo -e "${GREEN}Network will remain running for manual testing.${NC}"
    echo -e "${YELLOW}Press Ctrl+C in the script window to shutdown gracefully.${NC}"
else
    echo ""
    echo -e "${RED}❌ Failed to launch test network${NC}"
    exit 1
fi
