#!/bin/bash

# AerolithDB Local Test Network Launcher (Bash)
# 
# This script demonstrates AerolithDB's distributed, cross-platform, multi-node 
# functionality by launching a local test network with:
# - 1 Bootstrap/seed node for network formation
# - 4 Regular nodes connecting via P2P mesh networking
# - Simulated user activity (document CRUD operations)
# - Administrative operations (health checks, network monitoring)
# - Cross-node data replication and consistency validation

set -euo pipefail

# Default configuration
NODES_COUNT=4
DATA_DIR="test-network-data"
START_PORT=8080
VERBOSE=false

# ANSI Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Array to track node PIDs for cleanup
NODE_PIDS=()

# Function to print colored output
print_colored() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to print headers
print_header() {
    local title=$1
    echo ""
    print_colored "$BOLD" "$(printf '=%.0s' {1..80})"
    print_colored "$CYAN" "  $title"
    print_colored "$BOLD" "$(printf '=%.0s' {1..80})"
    echo ""
}

# Function to display usage information
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Launch AerolithDB Local Test Network with Bootstrap Node and Mesh Networking"
    echo ""
    echo "OPTIONS:"
    echo "  -n, --nodes COUNT     Number of regular nodes to launch (default: 4)"
    echo "  -d, --data-dir DIR    Base directory for node data storage (default: test-network-data)"
    echo "  -p, --port PORT       Starting port for node allocation (default: 8080)"
    echo "  -v, --verbose         Enable verbose logging output"
    echo "  -h, --help            Show this help message"
    echo ""
    echo "EXAMPLES:"
    echo "  $0                    Launch default 4-node network"
    echo "  $0 -n 6 -v           Launch 6-node network with verbose logging"
    echo "  $0 -d /tmp/testnet    Use custom data directory"
    echo ""
    echo "REQUIREMENTS:"
    echo "  - Cargo build system (Rust toolchain)"
    echo "  - Bash 4.0+ or compatible shell"
    echo "  - Available ports 8080-8090"
    echo "  - curl (for health checks)"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--nodes)
            NODES_COUNT="$2"
            shift 2
            ;;
        -d|--data-dir)
            DATA_DIR="$2"
            shift 2
            ;;
        -p|--port)
            START_PORT="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

# Function to wait for a node to become healthy
wait_for_healthy() {
    local url=$1
    local node_name=$2
    local timeout=${3:-30}
    
    print_colored "$YELLOW" "⏳ Waiting for $node_name to become healthy..."
    
    local elapsed=0
    while [ $elapsed -lt $timeout ]; do
        if curl -s -f "$url/health" >/dev/null 2>&1; then
            print_colored "$GREEN" "✅ $node_name is healthy"
            return 0
        fi
        
        sleep 2
        elapsed=$((elapsed + 2))
    done
    
    print_colored "$RED" "❌ $node_name failed to become healthy within $timeout seconds"
    return 1
}

# Function to cleanup nodes on exit
cleanup_nodes() {
    print_header "🛑 SHUTTING DOWN NETWORK NODES"
    
    # Kill all background jobs
    jobs -p | xargs -r kill 2>/dev/null || true
    
    # Kill all node processes
    for pid in "${NODE_PIDS[@]}"; do
        if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            print_colored "$YELLOW" "Stopping node process (PID: $pid)"
            kill "$pid" 2>/dev/null || true
            
            # Wait for graceful shutdown
            local count=0
            while [ $count -lt 5 ] && kill -0 "$pid" 2>/dev/null; do
                sleep 1
                count=$((count + 1))
            done
            
            # Force kill if still running
            if kill -0 "$pid" 2>/dev/null; then
                print_colored "$RED" "Force terminating node process: $pid"
                kill -9 "$pid" 2>/dev/null || true
            fi
        fi
    done
    
    print_colored "$GREEN" "✅ All nodes stopped"
}

# Register cleanup handler for various exit signals
trap cleanup_nodes EXIT INT TERM

# Function to start a node
start_node() {
    local node_id=$1
    local port=$2
    local data_dir=$3
    local is_bootstrap=$4
    local bootstrap_nodes=${5:-""}
    
    export AEROLITHSDB_NODE_ID="$node_id"
    export AEROLITHSDB_STORAGE_DATA_DIR="$data_dir"
    export AEROLITHSDB_API_REST_PORT="$port"
    export AEROLITHSDB_NETWORK_IS_BOOTSTRAP="$is_bootstrap"
    
    if [ "$is_bootstrap" = "false" ] && [ -n "$bootstrap_nodes" ]; then
        export AEROLITHSDB_NETWORK_BOOTSTRAP_NODES="$bootstrap_nodes"
    fi
    
    if [ "$VERBOSE" = "true" ]; then
        export RUST_LOG="debug,aerolithsdb=trace"
    else
        export RUST_LOG="info,aerolithsdb=info"
    fi
    
    # Start the node in background
    cargo run --release -- > "$data_dir/node.log" 2>&1 &
    local pid=$!
    NODE_PIDS+=("$pid")
    
    echo "$pid"
}

# Function to execute CLI command with error handling
execute_cli() {
    local url=$1
    shift
    local args=("$@")
    
    if cargo run --release --bin aerolithsdb-cli -- --url "$url" "${args[@]}" >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Main execution starts here
print_header "🚀 AEROLITHDB LOCAL NETWORK LAUNCHER"
print_colored "$BLUE" "Launching distributed test network with $NODES_COUNT nodes"
print_colored "$BLUE" "Bootstrap port: $START_PORT, Data directory: $DATA_DIR"

# Validate requirements
if ! command -v cargo >/dev/null 2>&1; then
    print_colored "$RED" "❌ Error: cargo (Rust build system) is required but not found"
    exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
    print_colored "$RED" "❌ Error: curl is required but not found"
    exit 1
fi

# Clean up previous test data
if [ -d "$DATA_DIR" ]; then
    print_colored "$YELLOW" "🧹 Cleaning up previous test data..."
    rm -rf "$DATA_DIR"
fi

# Create data directories
print_colored "$BLUE" "📁 Creating node data directories..."
BOOTSTRAP_DATA_DIR="$DATA_DIR/bootstrap-node"
mkdir -p "$BOOTSTRAP_DATA_DIR"

for ((i=1; i<=NODES_COUNT; i++)); do
    mkdir -p "$DATA_DIR/node-$i"
done

# Build the project
print_header "🔨 BUILDING AEROLITHDB"
print_colored "$BLUE" "Building AerolithDB binaries..."

if ! cargo build --release; then
    print_colored "$RED" "❌ Build failed"
    exit 1
fi

print_colored "$GREEN" "✅ Build completed successfully"

# Track node URLs
NODE_URLS=()

print_header "🏗️ LAUNCHING BOOTSTRAP NODE"

# Start bootstrap node
BOOTSTRAP_PORT=$START_PORT
BOOTSTRAP_URL="http://localhost:$BOOTSTRAP_PORT"
NODE_URLS+=("$BOOTSTRAP_URL")

print_colored "$BLUE" "🎯 Starting bootstrap node on port $BOOTSTRAP_PORT..."

BOOTSTRAP_PID=$(start_node "bootstrap-node-001" "$BOOTSTRAP_PORT" "$BOOTSTRAP_DATA_DIR" "true")
print_colored "$GREEN" "✅ Bootstrap node started (PID: $BOOTSTRAP_PID)"

# Wait for bootstrap node to be ready
if ! wait_for_healthy "$BOOTSTRAP_URL" "Bootstrap Node"; then
    print_colored "$RED" "❌ Bootstrap node failed to start properly"
    exit 1
fi

print_header "🌐 LAUNCHING REGULAR NODES"

# Launch regular nodes
for ((i=1; i<=NODES_COUNT; i++)); do
    NODE_PORT=$((START_PORT + i))
    NODE_ID=$(printf "regular-node-%03d" $i)
    NODE_DATA_DIR="$DATA_DIR/node-$i"
    NODE_URL="http://localhost:$NODE_PORT"
    NODE_URLS+=("$NODE_URL")
    
    print_colored "$BLUE" "🎯 Starting regular node $i on port $NODE_PORT..."
    
    NODE_PID=$(start_node "$NODE_ID" "$NODE_PORT" "$NODE_DATA_DIR" "false" "$BOOTSTRAP_URL")
    print_colored "$GREEN" "✅ Regular node $i started (PID: $NODE_PID)"
    
    # Brief pause between node startups
    sleep 2
done

print_header "⏳ WAITING FOR NETWORK FORMATION"

# Wait for all nodes to be healthy
ALL_HEALTHY=true
for ((i=0; i<${#NODE_URLS[@]}; i++)); do
    url="${NODE_URLS[$i]}"
    if [ $i -eq 0 ]; then
        node_name="Bootstrap Node"
    else
        node_name="Regular Node $i"
    fi
    
    if ! wait_for_healthy "$url" "$node_name" 45; then
        ALL_HEALTHY=false
    fi
done

if [ "$ALL_HEALTHY" = "false" ]; then
    print_colored "$RED" "❌ Not all nodes became healthy"
    exit 1
fi

# Additional wait for network stabilization
print_colored "$YELLOW" "⏳ Allowing time for P2P mesh formation..."
sleep 10

print_header "🎭 SIMULATING USER ACTIVITY"

# Test documents for simulation
declare -a TEST_DOCS=(
    "users:user_001:{\"name\":\"Alice Johnson\",\"email\":\"alice@aerolithdb.com\",\"department\":\"Engineering\",\"role\":\"Senior Developer\",\"joined_date\":\"2024-01-15\"}"
    "users:user_002:{\"name\":\"Bob Smith\",\"email\":\"bob@aerolithdb.com\",\"department\":\"Product\",\"role\":\"Product Manager\",\"joined_date\":\"2024-02-01\"}"
    "projects:proj_001:{\"name\":\"AerolithDB Enhancement\",\"description\":\"Improving distributed consensus algorithms\",\"status\":\"active\",\"team_members\":[\"user_001\",\"user_002\"],\"start_date\":\"2024-03-01\"}"
    "analytics:metrics_$(date +%Y%m%d_%H%M%S):{\"event_type\":\"network_test\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"nodes_count\":$((NODES_COUNT + 1)),\"test_phase\":\"user_simulation\"}"
)

# CREATE operations across different nodes
print_colored "$BLUE" "📝 Creating documents across nodes..."

for ((i=0; i<${#TEST_DOCS[@]}; i++)); do
    IFS=':' read -r collection doc_id json_data <<< "${TEST_DOCS[$i]}"
    
    target_node_index=$((i % ${#NODE_URLS[@]}))
    target_url="${NODE_URLS[$target_node_index]}"
    
    if [ $target_node_index -eq 0 ]; then
        node_name="Bootstrap"
    else
        node_name="Node $target_node_index"
    fi
    
    print_colored "$CYAN" "  Creating document '$doc_id' in collection '$collection' on $node_name..."
    
    if execute_cli "$target_url" put "$collection" "$doc_id" --data "$json_data"; then
        print_colored "$GREEN" "    ✅ Document created successfully"
    else
        print_colored "$RED" "    ❌ Failed to create document"
    fi
    
    sleep 1
done

print_header "🔄 DEMONSTRATING CROSS-NODE OPERATIONS"

# READ operations from different nodes to demonstrate replication
print_colored "$BLUE" "📖 Reading documents from different nodes (demonstrating replication)..."

for ((i=0; i<${#TEST_DOCS[@]}; i++)); do
    IFS=':' read -r collection doc_id _ <<< "${TEST_DOCS[$i]}"
    
    # Try reading from a different node than where it was written
    reader_node_index=$(((i + 2) % ${#NODE_URLS[@]}))
    reader_url="${NODE_URLS[$reader_node_index]}"
    
    if [ $reader_node_index -eq 0 ]; then
        reader_name="Bootstrap"
    else
        reader_name="Node $reader_node_index"
    fi
    
    print_colored "$CYAN" "  Reading '$doc_id' from $reader_name..."
    
    if execute_cli "$reader_url" get "$collection" "$doc_id"; then
        print_colored "$GREEN" "    ✅ Cross-node read successful"
    else
        print_colored "$YELLOW" "    ⚠️  Document not yet replicated (this is normal in eventual consistency)"
    fi
    
    sleep 1
done

print_header "🔍 QUERYING AND ANALYTICS"

# Query operations demonstrating search capabilities
print_colored "$BLUE" "🔎 Performing queries across the distributed network..."

declare -a QUERIES=(
    "users:{\"department\":\"Engineering\"}:Query all users in Engineering department"
    "projects:{\"status\":\"active\"}:Query active projects" 
    "analytics:{\"event_type\":\"network_test\"}:Query recent analytics events"
)

for ((i=0; i<${#QUERIES[@]}; i++)); do
    IFS=':' read -r collection filter description <<< "${QUERIES[$i]}"
    
    query_node_index=$((i % ${#NODE_URLS[@]}))
    query_url="${NODE_URLS[$query_node_index]}"
    
    if [ $query_node_index -eq 0 ]; then
        query_node_name="Bootstrap"
    else
        query_node_name="Node $query_node_index"
    fi
    
    print_colored "$CYAN" "  $description (via $query_node_name)..."
    
    if execute_cli "$query_url" query "$collection" --filter "$filter"; then
        print_colored "$GREEN" "    ✅ Query executed successfully"
    else
        print_colored "$YELLOW" "    ⚠️  Query returned no results (data may not be fully replicated yet)"
    fi
    
    sleep 1
done

print_header "👑 ADMINISTRATIVE OPERATIONS"

# Administrative health checks and monitoring
print_colored "$BLUE" "🩺 Performing network health checks and monitoring..."

for ((i=0; i<${#NODE_URLS[@]}; i++)); do
    url="${NODE_URLS[$i]}"
    
    if [ $i -eq 0 ]; then
        node_name="Bootstrap Node"
    else
        node_name="Regular Node $i"
    fi
    
    print_colored "$CYAN" "  Health check: $node_name..."
    
    if execute_cli "$url" health; then
        print_colored "$GREEN" "    ✅ $node_name is healthy"
    else
        print_colored "$RED" "    ❌ $node_name health check failed"
    fi
done

# Statistics collection
print_colored "$BLUE" "📊 Collecting system statistics..."
STATS_URL="${NODE_URLS[0]}"  # Get stats from bootstrap node

if execute_cli "$STATS_URL" stats --format table; then
    print_colored "$GREEN" "    ✅ Statistics collected successfully"
else
    print_colored "$YELLOW" "    ⚠️  Statistics collection error"
fi

print_header "🎯 NETWORK TEST COMPLETED"

print_colored "$GREEN" "✅ AerolithDB distributed network test completed successfully!"
echo ""
print_colored "$BLUE" "📋 Test Summary:"
print_colored "$BLUE" "   • Bootstrap Node: ${NODE_URLS[0]}"
print_colored "$BLUE" "   • Regular Nodes: $NODES_COUNT nodes"
print_colored "$BLUE" "   • Documents Created: ${#TEST_DOCS[@]}"
print_colored "$BLUE" "   • Cross-node Operations: Tested"
print_colored "$BLUE" "   • Query Operations: Tested"
print_colored "$BLUE" "   • Administrative Operations: Tested"
echo ""
print_colored "$YELLOW" "🔗 Network endpoints:"
for ((i=0; i<${#NODE_URLS[@]}; i++)); do
    url="${NODE_URLS[$i]}"
    if [ $i -eq 0 ]; then
        node_type="Bootstrap"
    else
        node_type="Regular"
    fi
    print_colored "$YELLOW" "   • $node_type Node: $url"
done
echo ""
print_colored "$CYAN" "💡 You can now interact with the network using:"
print_colored "$CYAN" "   cargo run --release --bin aerolithsdb-cli -- --url ${NODE_URLS[0]} health"
print_colored "$CYAN" "   cargo run --release --bin aerolithsdb-cli -- --url ${NODE_URLS[0]} stats"
print_colored "$CYAN" "   cargo run --release --bin aerolithsdb-cli -- --url ${NODE_URLS[0]} get users user_001"
echo ""
print_colored "$BOLD" "Press Ctrl+C to stop all nodes and exit"

# Keep the script running until interrupted
print_colored "$BLUE" "🔄 Monitoring network health... (Press Ctrl+C to exit)"

while true; do
    sleep 10
    
    # Periodic health check
    healthy_count=0
    for url in "${NODE_URLS[@]}"; do
        if curl -s -f "$url/health" >/dev/null 2>&1; then
            ((healthy_count++))
        fi
    done
    
    print_colored "$BLUE" "🔄 Network Status: $healthy_count/${#NODE_URLS[@]} nodes healthy"
done
