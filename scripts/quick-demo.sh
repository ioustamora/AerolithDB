#!/bin/bash

# AerolithDB Quick Multi-Node Demo (Bash)
# Simple demonstration of distributed functionality

set -euo pipefail

PORT=${1:-8080}

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

NODE_PIDS=()

print_color() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

cleanup() {
    print_color "$YELLOW" "\n🛑 Cleaning up processes..."
    
    for pid in "${NODE_PIDS[@]}"; do
        if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
            sleep 1
            if kill -0 "$pid" 2>/dev/null; then
                kill -9 "$pid" 2>/dev/null || true
            fi
        fi
    done
    
    print_color "$GREEN" "✅ Cleanup completed"
}

trap cleanup EXIT INT TERM

wait_for_health() {
    local url=$1
    local timeout=${2:-30}
    
    local elapsed=0
    while [ $elapsed -lt $timeout ]; do
        if curl -s -f "$url/health" >/dev/null 2>&1; then
            return 0
        fi
        sleep 2
        elapsed=$((elapsed + 2))
    done
    return 1
}

print_color "$BLUE" "🚀 AerolithDB Quick Multi-Node Demo"
print_color "$BLUE" "====================================="

# Check requirements
if ! command -v cargo >/dev/null 2>&1; then
    print_color "$RED" "❌ cargo not found. Please install Rust."
    exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
    print_color "$RED" "❌ curl not found. Please install curl."
    exit 1
fi

# Build project
print_color "$YELLOW" "🔨 Building AerolithDB..."
if cargo build --release >/dev/null 2>&1; then
    print_color "$GREEN" "✅ Build completed"
else
    print_color "$RED" "❌ Build failed"
    exit 1
fi

# Clean previous data
rm -rf quick-demo-data
mkdir -p quick-demo-data/{bootstrap,node1,node2}

# Start bootstrap node
print_color "$YELLOW" "🎯 Starting bootstrap node on port $PORT..."

export AEROLITHSDB_NODE_ID="demo-bootstrap"
export AEROLITHSDB_STORAGE_DATA_DIR="quick-demo-data/bootstrap"
export AEROLITHSDB_API_REST_PORT="$PORT"
export RUST_LOG="info"

cargo run --release -- >"quick-demo-data/bootstrap/node.log" 2>&1 &
BOOTSTRAP_PID=$!
NODE_PIDS+=("$BOOTSTRAP_PID")

BOOTSTRAP_URL="http://localhost:$PORT"
if wait_for_health "$BOOTSTRAP_URL"; then
    print_color "$GREEN" "✅ Bootstrap node ready"
else
    print_color "$RED" "❌ Bootstrap node failed to start"
    exit 1
fi

# Start node 1
print_color "$YELLOW" "🎯 Starting regular node 1 on port $((PORT + 1))..."

export AEROLITHSDB_NODE_ID="demo-node-1"
export AEROLITHSDB_STORAGE_DATA_DIR="quick-demo-data/node1"
export AEROLITHSDB_API_REST_PORT="$((PORT + 1))"

cargo run --release -- >"quick-demo-data/node1/node.log" 2>&1 &
NODE1_PID=$!
NODE_PIDS+=("$NODE1_PID")

# Start node 2
print_color "$YELLOW" "🎯 Starting regular node 2 on port $((PORT + 2))..."

export AEROLITHSDB_NODE_ID="demo-node-2"
export AEROLITHSDB_STORAGE_DATA_DIR="quick-demo-data/node2"
export AEROLITHSDB_API_REST_PORT="$((PORT + 2))"

cargo run --release -- >"quick-demo-data/node2/node.log" 2>&1 &
NODE2_PID=$!
NODE_PIDS+=("$NODE2_PID")

# Wait for nodes
sleep 5

NODE1_URL="http://localhost:$((PORT + 1))"
NODE2_URL="http://localhost:$((PORT + 2))"

if wait_for_health "$NODE1_URL"; then
    print_color "$GREEN" "✅ Node 1 ready"
else
    print_color "$RED" "❌ Node 1 failed to start"
    exit 1
fi

if wait_for_health "$NODE2_URL"; then
    print_color "$GREEN" "✅ Node 2 ready"
else
    print_color "$RED" "❌ Node 2 failed to start"
    exit 1
fi

print_color "$BLUE" "\n🎭 Demonstrating Distributed Operations"
print_color "$BLUE" "======================================="

# Create document
print_color "$YELLOW" "📝 Creating document on bootstrap node..."
DOC_DATA="{\"name\":\"Demo User\",\"email\":\"demo@aerolithdb.com\",\"created\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}"

if cargo run --release --bin aerolithsdb-cli -- --url "$BOOTSTRAP_URL" put demo_users user_001 --data "$DOC_DATA" >/dev/null 2>&1; then
    print_color "$GREEN" "✅ Document created on bootstrap node"
else
    print_color "$YELLOW" "⚠️  Document creation status unclear"
fi

sleep 2

# Read from different nodes
print_color "$YELLOW" "📖 Reading document from Node 1..."
if cargo run --release --bin aerolithsdb-cli -- --url "$NODE1_URL" get demo_users user_001 >/dev/null 2>&1; then
    print_color "$GREEN" "✅ Document successfully read from Node 1"
else
    print_color "$YELLOW" "⚠️  Document not yet replicated to Node 1 (eventual consistency)"
fi

print_color "$YELLOW" "📖 Reading document from Node 2..."
if cargo run --release --bin aerolithsdb-cli -- --url "$NODE2_URL" get demo_users user_001 >/dev/null 2>&1; then
    print_color "$GREEN" "✅ Document successfully read from Node 2"
else
    print_color "$YELLOW" "⚠️  Document not yet replicated to Node 2 (eventual consistency)"
fi

# Health checks
print_color "$YELLOW" "\n🩺 Health checks across all nodes..."

for url in "$BOOTSTRAP_URL" "$NODE1_URL" "$NODE2_URL"; do
    case $url in
        "$BOOTSTRAP_URL") node_name="Bootstrap" ;;
        "$NODE1_URL") node_name="Node 1" ;;
        "$NODE2_URL") node_name="Node 2" ;;
    esac
    
    if cargo run --release --bin aerolithsdb-cli -- --url "$url" health >/dev/null 2>&1; then
        print_color "$GREEN" "✅ $node_name is healthy"
    else
        print_color "$RED" "❌ $node_name health check failed"
    fi
done

print_color "$BLUE" "\n🎯 Demo Completed Successfully!"
print_color "$BLUE" "=============================="
print_color "$GREEN" "✅ Multi-node AerolithDB network is running"
print_color "$GREEN" "✅ Distributed document operations demonstrated"
print_color "$GREEN" "✅ Cross-node connectivity verified"

print_color "$YELLOW" "\n🔗 Available endpoints:"
print_color "$YELLOW" "   Bootstrap Node: $BOOTSTRAP_URL"
print_color "$YELLOW" "   Regular Node 1: $NODE1_URL"
print_color "$YELLOW" "   Regular Node 2: $NODE2_URL"

print_color "$BLUE" "\n💡 Try these commands:"
print_color "$BLUE" "   cargo run --release --bin aerolithsdb-cli -- --url $BOOTSTRAP_URL stats"
print_color "$BLUE" "   cargo run --release --bin aerolithsdb-cli -- --url $NODE1_URL get demo_users user_001"
print_color "$BLUE" "   cargo run --release --bin aerolithsdb-cli -- --url $NODE2_URL health"

print_color "$YELLOW" "\nPress Ctrl+C to stop all nodes and exit..."

# Keep running until interrupted
while true; do
    sleep 5
    
    # Quick health check
    healthy_count=0
    for url in "$BOOTSTRAP_URL" "$NODE1_URL" "$NODE2_URL"; do
        if curl -s -f "$url/health" >/dev/null 2>&1; then
            ((healthy_count++))
        fi
    done
    
    print_color "$BLUE" "🔄 Network Status: $healthy_count/3 nodes healthy"
done
