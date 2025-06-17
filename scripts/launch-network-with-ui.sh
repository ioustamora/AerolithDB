#!/bin/bash

# AerolithDB Full-Stack Development Environment Launcher
# Cross-platform Bash script for launching distributed backend + React web UI

set -euo pipefail

# Configuration
NODES_COUNT=${1:-4}
START_PORT=${2:-8080}
WEB_PORT=${3:-3000}
SKIP_BUILD=${4:-false}
OPEN_BROWSER=${5:-true}
VERBOSE=${6:-false}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# Global state
NODE_PIDS=()
WEB_PID=""
NODE_URLS=()
DATA_DIR="fullstack-demo-data"

# Helper functions
print_color() {
    local color=$1
    local message=$2
    echo -e "${!color}${message}${NC}"
}

print_header() {
    echo ""
    print_color "WHITE" "================================================================================"
    print_color "CYAN" "  $1"
    print_color "WHITE" "================================================================================"
    echo ""
}

print_subheader() {
    echo ""
    print_color "YELLOW" ">>> $1"
    echo ""
}

wait_for_healthy() {
    local url=$1
    local node_name=$2
    local timeout=${3:-30}
    
    print_color "YELLOW" "⏳ Waiting for $node_name to become healthy..."
    local start_time=$(date +%s)
    
    while true; do
        if curl -s -f "$url/health" >/dev/null 2>&1; then
            print_color "GREEN" "✅ $node_name is healthy"
            return 0
        fi
        
        local current_time=$(date +%s)
        if (( current_time - start_time > timeout )); then
            print_color "RED" "❌ $node_name failed to become healthy within $timeout seconds"
            return 1
        fi
        
        sleep 2
    done
}

wait_for_web_ui() {
    local url=$1
    local timeout=${2:-45}
    
    print_color "YELLOW" "⏳ Waiting for Web UI to become available..."
    local start_time=$(date +%s)
    
    while true; do
        if curl -s -I "$url" | grep -q "200 OK"; then
            print_color "GREEN" "✅ Web UI is available"
            return 0
        fi
        
        local current_time=$(date +%s)
        if (( current_time - start_time > timeout )); then
            print_color "RED" "❌ Web UI failed to become available within $timeout seconds"
            return 1
        fi
        
        sleep 2
    done
}

cleanup() {
    print_color "YELLOW" "🛑 Stopping full-stack environment..."
    
    # Stop web UI
    if [[ -n "$WEB_PID" ]]; then
        print_color "YELLOW" "Stopping Web UI (PID: $WEB_PID)..."
        if kill -0 "$WEB_PID" 2>/dev/null; then
            kill "$WEB_PID" 2>/dev/null || true
            sleep 2
            kill -9 "$WEB_PID" 2>/dev/null || true
        fi
    fi
    
    # Stop all backend nodes
    if [[ ${#NODE_PIDS[@]} -gt 0 ]]; then
        print_color "YELLOW" "Stopping backend nodes..."
        for pid in "${NODE_PIDS[@]}"; do
            if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
                kill "$pid" 2>/dev/null || true
            fi
        done
        
        # Wait for graceful shutdown
        sleep 3
        
        # Force kill if necessary
        for pid in "${NODE_PIDS[@]}"; do
            if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
                kill -9 "$pid" 2>/dev/null || true
            fi
        done
    fi
    
    # Clean up data directory
    if [[ -d "$DATA_DIR" ]]; then
        print_color "YELLOW" "Cleaning up data directory..."
        rm -rf "$DATA_DIR"
    fi
    
    print_color "GREEN" "🏁 Full-stack environment stopped successfully"
}

# Set up signal handlers
trap cleanup EXIT INT TERM

check_dependencies() {
    print_header "Checking Dependencies"
    
    # Check Rust/Cargo
    if ! command -v cargo &> /dev/null; then
        print_color "RED" "❌ Cargo not found. Please install Rust toolchain."
        exit 1
    fi
    print_color "GREEN" "✅ Cargo found: $(cargo --version)"
    
    # Check Node.js
    if ! command -v node &> /dev/null; then
        print_color "RED" "❌ Node.js not found. Please install Node.js 18+"
        exit 1
    fi
    print_color "GREEN" "✅ Node.js found: $(node --version)"
    
    # Check npm
    if ! command -v npm &> /dev/null; then
        print_color "RED" "❌ npm not found. Please install npm 9+"
        exit 1
    fi
    print_color "GREEN" "✅ npm found: $(npm --version)"
    
    # Check curl
    if ! command -v curl &> /dev/null; then
        print_color "RED" "❌ curl not found. Please install curl for health checks."
        exit 1
    fi
    print_color "GREEN" "✅ curl found: $(curl --version | head -1)"
}

check_ports() {
    print_subheader "Checking Port Availability"
    
    local ports_to_check=()
    
    # Backend ports
    for ((i=0; i<=NODES_COUNT; i++)); do
        ports_to_check+=($((START_PORT + i)))
    done
    
    # Web UI port
    ports_to_check+=($WEB_PORT)
    
    for port in "${ports_to_check[@]}"; do
        if netstat -ln 2>/dev/null | grep -q ":$port " || lsof -i ":$port" &>/dev/null; then
            print_color "RED" "❌ Port $port is already in use"
            exit 1
        fi
    done
    
    print_color "GREEN" "✅ All required ports are available"
}

build_backend() {
    if [[ "$SKIP_BUILD" == "true" ]]; then
        print_color "BLUE" "⏭️ Skipping backend build (--skip-build specified)"
        return
    fi
    
    print_header "Building AerolithDB Backend"
    
    if [[ "$VERBOSE" == "true" ]]; then
        cargo build --release
    else
        print_color "BLUE" "🔨 Building Rust binaries (this may take a few minutes)..."
        cargo build --release --quiet
    fi
    
    print_color "GREEN" "✅ Backend build completed"
}

setup_web_client() {
    print_header "Setting Up Web Client"
    
    if [[ ! -d "web-client" ]]; then
        print_color "RED" "❌ web-client directory not found"
        exit 1
    fi
    
    cd web-client
    
    # Install dependencies if needed
    if [[ ! -d "node_modules" ]]; then
        print_color "BLUE" "📦 Installing web client dependencies..."
        npm install --silent
    else
        print_color "GREEN" "✅ Web client dependencies already installed"
    fi
    
    cd ..
}

start_backend() {
    print_header "Starting AerolithDB Distributed Network"
    
    # Clean up any existing data
    if [[ -d "$DATA_DIR" ]]; then
        rm -rf "$DATA_DIR"
    fi
    
    # Start bootstrap node
    local bootstrap_port=$START_PORT
    local bootstrap_url="http://localhost:$bootstrap_port"
    NODE_URLS+=("$bootstrap_url")
    
    print_color "BLUE" "🚀 Starting bootstrap node on port $bootstrap_port..."
    
    RUST_LOG=${VERBOSE:+debug} cargo run --release --bin datisdb -- \
        --port $bootstrap_port \
        --data-dir "$DATA_DIR/bootstrap" \
        --node-id "bootstrap" \
        --mode bootstrap &
    
    NODE_PIDS+=($!)
    
    # Wait for bootstrap node
    if ! wait_for_healthy "$bootstrap_url" "Bootstrap Node"; then
        print_color "RED" "❌ Failed to start bootstrap node"
        exit 1
    fi
    
    # Start regular nodes
    for ((i=1; i<=NODES_COUNT; i++)); do
        local node_port=$((START_PORT + i))
        local node_url="http://localhost:$node_port"
        NODE_URLS+=("$node_url")
        
        print_color "BLUE" "🔗 Starting regular node $i on port $node_port..."
        
        RUST_LOG=${VERBOSE:+debug} cargo run --release --bin datisdb -- \
            --port $node_port \
            --data-dir "$DATA_DIR/node_$i" \
            --node-id "node_$i" \
            --mode regular \
            --bootstrap-peers "localhost:$bootstrap_port" &
        
        NODE_PIDS+=($!)
        
        # Brief delay between node startups
        sleep 1
    done
    
    # Wait for all regular nodes
    for ((i=1; i<=NODES_COUNT; i++)); do
        local node_port=$((START_PORT + i))
        local node_url="http://localhost:$node_port"
        
        if ! wait_for_healthy "$node_url" "Regular Node $i"; then
            print_color "RED" "❌ Failed to start regular node $i"
            exit 1
        fi
    done
    
    print_color "GREEN" "✅ All backend nodes are healthy and running"
}

create_demo_data() {
    print_header "Creating Demo Data"
    
    local bootstrap_url="${NODE_URLS[0]}"
    
    print_color "BLUE" "📝 Creating sample collections and documents..."
    
    # Create users collection
    for i in {1..5}; do
        local user_data="{\"name\":\"Demo User $i\",\"email\":\"user$i@aerolithdb.com\",\"role\":\"developer\",\"created\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}"
        
        if [[ "$VERBOSE" == "true" ]]; then
            print_color "BLUE" "Creating user $i..."
        fi
        
        cargo run --release --bin aerolithsdb-cli -- \
            --url "$bootstrap_url" \
            put users "user_$i" \
            --data "$user_data" >/dev/null 2>&1 || true
    done
    
    # Create projects collection
    for i in {1..3}; do
        local project_data="{\"name\":\"Project $i\",\"description\":\"Demo project for testing\",\"status\":\"active\",\"created\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}"
        
        if [[ "$VERBOSE" == "true" ]]; then
            print_color "BLUE" "Creating project $i..."
        fi
        
        cargo run --release --bin aerolithsdb-cli -- \
            --url "$bootstrap_url" \
            put projects "project_$i" \
            --data "$project_data" >/dev/null 2>&1 || true
    done
    
    print_color "GREEN" "✅ Demo data created successfully"
}

start_web_ui() {
    print_header "Starting Web Client"
    
    cd web-client
    
    print_color "BLUE" "🌐 Starting web development server on port $WEB_PORT..."
    
    # Set environment variables for the web client
    export VITE_API_BASE_URL="http://localhost:$START_PORT"
    export VITE_WS_BASE_URL="ws://localhost:$START_PORT"
    
    # Start the web development server
    if [[ "$VERBOSE" == "true" ]]; then
        npm run dev -- --port $WEB_PORT --host &
    else
        npm run dev -- --port $WEB_PORT --host >/dev/null 2>&1 &
    fi
    
    WEB_PID=$!
    cd ..
    
    # Wait for web UI to be available
    local web_url="http://localhost:$WEB_PORT"
    if ! wait_for_web_ui "$web_url"; then
        print_color "RED" "❌ Failed to start web UI"
        exit 1
    fi
    
    print_color "GREEN" "✅ Web UI started successfully"
    
    # Open browser if requested
    if [[ "$OPEN_BROWSER" == "true" ]]; then
        print_color "BLUE" "🌍 Opening browser..."
        
        # Cross-platform browser opening
        if command -v xdg-open &> /dev/null; then
            xdg-open "$web_url" &>/dev/null &
        elif command -v open &> /dev/null; then
            open "$web_url" &>/dev/null &
        elif command -v start &> /dev/null; then
            start "$web_url" &>/dev/null &
        else
            print_color "YELLOW" "⚠️ Could not auto-open browser. Please navigate to: $web_url"
        fi
    fi
}

show_environment_info() {
    print_header "Full-Stack Environment Ready"
    
    print_color "GREEN" "🎉 AerolithDB Full-Stack Environment is now running!"
    echo ""
    
    print_color "CYAN" "📊 Backend Network:"
    print_color "WHITE" "  Bootstrap Node:    http://localhost:$START_PORT"
    for ((i=1; i<=NODES_COUNT; i++)); do
        print_color "WHITE" "  Regular Node $i:    http://localhost:$((START_PORT + i))"
    done
    echo ""
    
    print_color "CYAN" "🌐 Web Interface:"
    print_color "WHITE" "  Dashboard:         http://localhost:$WEB_PORT"
    print_color "WHITE" "  Network Explorer:  http://localhost:$WEB_PORT/network"
    print_color "WHITE" "  Data Browser:      http://localhost:$WEB_PORT/data"
    print_color "WHITE" "  Query Interface:   http://localhost:$WEB_PORT/query"
    print_color "WHITE" "  Real-time Monitor: http://localhost:$WEB_PORT/realtime"
    echo ""
    
    print_color "CYAN" "🔧 API Endpoints:"
    print_color "WHITE" "  REST API:          http://localhost:$START_PORT/api/v1/"
    print_color "WHITE" "  Health Check:      http://localhost:$START_PORT/health"
    print_color "WHITE" "  Statistics:        http://localhost:$START_PORT/stats"
    echo ""
    
    print_color "YELLOW" "📝 Demo Data Created:"
    print_color "WHITE" "  - 5 users in 'users' collection"
    print_color "WHITE" "  - 3 projects in 'projects' collection"
    echo ""
    
    print_color "BLUE" "💡 Tips:"
    print_color "WHITE" "  - Use Ctrl+C to stop the entire environment"
    print_color "WHITE" "  - All data is stored in '$DATA_DIR/' (will be cleaned up on exit)"
    print_color "WHITE" "  - Web UI automatically connects to the local network"
    print_color "WHITE" "  - Monitor real-time network activity in the NetworkExplorer"
    echo ""
}

monitor_environment() {
    print_color "BLUE" "🔍 Monitoring full-stack environment (Ctrl+C to stop)..."
    echo ""
    
    local check_count=0
    while true; do
        check_count=$((check_count + 1))
        
        # Check backend health every 30 seconds
        if (( check_count % 15 == 0 )); then
            print_color "BLUE" "⏰ Periodic health check..."
            
            local healthy_nodes=0
            for url in "${NODE_URLS[@]}"; do
                if curl -s -f "$url/health" >/dev/null 2>&1; then
                    healthy_nodes=$((healthy_nodes + 1))
                fi
            done
            
            print_color "GREEN" "💚 Backend: $healthy_nodes/${#NODE_URLS[@]} nodes healthy"
            
            # Check web UI
            if curl -s -I "http://localhost:$WEB_PORT" | grep -q "200 OK"; then
                print_color "GREEN" "🌐 Web UI: Responsive"
            else
                print_color "YELLOW" "⚠️ Web UI: Not responding"
            fi
            echo ""
        fi
        
        sleep 2
    done
}

# Main execution
main() {
    print_header "AerolithDB Full-Stack Development Environment"
    
    print_color "BLUE" "🚀 Launching distributed network with web UI..."
    print_color "WHITE" "  Nodes: $NODES_COUNT regular + 1 bootstrap"
    print_color "WHITE" "  Backend Port Range: $START_PORT-$((START_PORT + NODES_COUNT))"
    print_color "WHITE" "  Web UI Port: $WEB_PORT"
    echo ""
    
    check_dependencies
    check_ports
    build_backend
    setup_web_client
    start_backend
    create_demo_data
    start_web_ui
    show_environment_info
    monitor_environment
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "AerolithDB Full-Stack Development Environment"
        echo ""
        echo "Usage: $0 [NODES_COUNT] [START_PORT] [WEB_PORT] [SKIP_BUILD] [OPEN_BROWSER] [VERBOSE]"
        echo ""
        echo "Arguments:"
        echo "  NODES_COUNT   Number of regular nodes (default: 4)"
        echo "  START_PORT    Starting port for backend (default: 8080)"
        echo "  WEB_PORT      Port for web UI (default: 3000)"
        echo "  SKIP_BUILD    Skip building binaries (default: false)"
        echo "  OPEN_BROWSER  Auto-open browser (default: true)"
        echo "  VERBOSE       Enable verbose output (default: false)"
        echo ""
        echo "Examples:"
        echo "  $0                    # Default: 4 nodes, ports 8080-8084, web on 3000"
        echo "  $0 6 9000 3001        # 6 nodes, ports 9000-9006, web on 3001"
        echo "  $0 2 8080 3000 true   # 2 nodes, skip build"
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac
