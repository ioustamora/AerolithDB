# AerolithDB Network Scripts

This directory contains cross-platform scripts to demonstrate AerolithDB's distributed, multi-node functionality.

## Overview

These scripts showcase AerolithDB's enterprise-grade distributed architecture by launching a local test network that demonstrates:

- **P2P Mesh Networking**: Bootstrap node formation and peer discovery
- **Cross-Platform Compatibility**: Scripts for both Windows (PowerShell) and Unix (Bash)
- **Distributed Document Operations**: CRUD operations across multiple nodes
- **Network Resilience**: Multi-node data replication and consistency
- **Administrative Tools**: Health monitoring and system statistics
- **Real-time Operations**: Live network status monitoring

## Scripts

### `launch-local-network.ps1` (Windows PowerShell)
Cross-platform PowerShell script for Windows and PowerShell Core on Linux/macOS.

**Features:**
- ANSI color output for visual feedback
- Comprehensive error handling and cleanup
- Graceful shutdown on Ctrl+C
- Verbose logging options
- Configurable node count and ports

### `launch-local-network.sh` (Unix Bash)
Bash script for Linux, macOS, and other Unix-like systems.

**Features:**
- POSIX-compliant shell scripting
- Signal handling for clean shutdown  
- Colored terminal output
- Command-line argument parsing
- Health monitoring and diagnostics

## Requirements

### Common Requirements
- **Rust Toolchain**: Cargo build system (1.70+)
- **Available Ports**: 8080-8090 range
- **System Resources**: 2GB+ RAM recommended for multi-node testing
- **Network Access**: Localhost connectivity

### Platform-Specific Requirements

#### Windows
- **PowerShell**: 5.1+ or PowerShell Core 6+
- **Windows Terminal**: Recommended for best ANSI color support

#### Unix/Linux/macOS
- **Bash**: 4.0+ or compatible shell
- **curl**: For HTTP health checks
- **Standard POSIX utilities**: kill, jobs, sleep, etc.

## Usage

### Quick Start

#### Windows (PowerShell)
```powershell
# Launch default 4-node network
.\scripts\launch-local-network.ps1

# Launch 6-node network with verbose logging
.\scripts\launch-local-network.ps1 -NodesCount 6 -Verbose

# Custom data directory and port
.\scripts\launch-local-network.ps1 -DataDir "C:\temp\aerolithdb-test" -StartPort 9000
```

#### Linux/macOS (Bash)
```bash
# Make script executable (first time)
chmod +x scripts/launch-local-network.sh

# Launch default 4-node network
./scripts/launch-local-network.sh

# Launch 6-node network with verbose logging  
./scripts/launch-local-network.sh -n 6 -v

# Custom configuration
./scripts/launch-local-network.sh -n 3 -d /tmp/aerolithdb-test -p 9000
```

### Command-Line Options

#### PowerShell Script
- `-NodesCount <int>`: Number of regular nodes (default: 4)
- `-DataDir <string>`: Base data directory (default: test-network-data)  
- `-StartPort <int>`: Starting port number (default: 8080)
- `-Verbose`: Enable detailed logging

#### Bash Script
- `-n, --nodes <count>`: Number of regular nodes (default: 4)
- `-d, --data-dir <dir>`: Base data directory (default: test-network-data)
- `-p, --port <port>`: Starting port number (default: 8080)
- `-v, --verbose`: Enable detailed logging
- `-h, --help`: Show help message

## Network Architecture

The scripts create a local distributed network with the following topology:

```
┌─────────────────┐     ┌─────────────────┐
│   Bootstrap     │────▶│   Regular       │
│   Node          │     │   Node 1        │  
│   Port: 8080    │     │   Port: 8081    │
└─────────────────┘     └─────────────────┘
         │                        │
         ▼                        │
┌─────────────────┐               │
│   Regular       │               │
│   Node 2        │◀──────────────┘
│   Port: 8082    │
└─────────────────┘
         │
         ▼
┌─────────────────┐     ┌─────────────────┐
│   Regular       │────▶│   Regular       │
│   Node 3        │     │   Node 4        │
│   Port: 8083    │     │   Port: 8084    │
└─────────────────┘     └─────────────────┘
```

### Node Types

1. **Bootstrap Node** (Port 8080)
   - Initial seed node for network formation
   - Provides cluster discovery services
   - Maintains network topology information
   - All other nodes connect to this initially

2. **Regular Nodes** (Ports 8081-8084+)
   - Full database functionality
   - P2P mesh connectivity
   - Data replication and storage
   - Query processing capabilities

## Test Operations

The scripts automatically perform the following operations to demonstrate distributed functionality:

### 1. Network Formation Phase
- Launch bootstrap node with cluster services
- Start regular nodes with P2P mesh networking
- Wait for all nodes to become healthy
- Verify cross-node connectivity

### 2. User Activity Simulation
- **Document Creation**: Store documents across different nodes
- **Cross-Node Reads**: Retrieve documents from nodes other than where they were created
- **Query Operations**: Execute distributed queries across the network
- **Data Consistency**: Verify replication and consistency

### 3. Administrative Operations
- **Health Checks**: Verify all nodes are operational
- **Statistics Collection**: Gather system performance metrics
- **Network Monitoring**: Real-time network status updates

### Sample Test Data

The scripts create the following test documents:

```json
// Users Collection
{
  "collection": "users",
  "documents": [
    {
      "id": "user_001",
      "name": "Alice Johnson",
      "department": "Engineering",
      "role": "Senior Developer"
    },
    {
      "id": "user_002", 
      "name": "Bob Smith",
      "department": "Product",
      "role": "Product Manager"
    }
  ]
}

// Projects Collection  
{
  "collection": "projects",
  "documents": [
    {
      "id": "proj_001",
      "name": "AerolithDB Enhancement",
      "status": "active",
      "team_members": ["user_001", "user_002"]
    }
  ]
}

// Analytics Collection
{
  "collection": "analytics", 
  "documents": [
    {
      "id": "metrics_<timestamp>",
      "event_type": "network_test",
      "nodes_count": 5,
      "test_phase": "user_simulation"
    }
  ]
}
```

## Manual Testing

Once the network is running, you can interact with it manually:

### Health Checks
```bash
# Check bootstrap node health
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 health

# Check regular node health  
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8081 health
```

### Document Operations
```bash
# Create a document on one node
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 put users user_003 --data '{"name":"Charlie Brown","department":"Sales"}'

# Read from a different node (demonstrates replication)
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8082 get users user_003

# Query across the network
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8083 query users --filter '{"department":"Engineering"}'
```

### System Statistics
```bash
# Get comprehensive system stats
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 stats --format table

# Get analytics data
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 query analytics --filter '{"event_type":"network_test"}'
```

## Network Endpoints

When running, the network provides the following endpoints:

### Bootstrap Node (localhost:8080)
- REST API: `http://localhost:8080/api/v1/`
- GraphQL: `http://localhost:8080/graphql` 
- Health Check: `http://localhost:8080/health`
- WebSocket: `ws://localhost:8080/ws`

### Regular Nodes (localhost:8081-8084+)
- Each node provides the same API endpoints on its respective port
- Full distributed functionality on each node
- Cross-node data replication and consistency

## Troubleshooting

### Common Issues

#### Port Already in Use
```
Error: Address already in use (os error 98)
```
**Solution**: Change the starting port with `-p` or `--port` option, or stop existing services on those ports.

#### Build Failures
```
Error: could not compile aerolithdb
```
**Solution**: Ensure you have the latest Rust toolchain:
```bash
rustup update
cargo clean
cargo build --release
```

#### Node Startup Timeout
```
❌ Bootstrap node failed to become healthy within 30 seconds
```
**Solution**: 
- Check system resources (RAM, CPU)
- Increase timeout by modifying the script
- Check firewall settings for localhost access

#### Permission Denied (Unix)
```
Permission denied: ./scripts/launch-local-network.sh
```
**Solution**: Make the script executable:
```bash
chmod +x scripts/launch-local-network.sh
```

### Debug Information

Enable verbose logging for detailed troubleshooting:

#### PowerShell
```powershell
.\scripts\launch-local-network.ps1 -Verbose
```

#### Bash
```bash
./scripts/launch-local-network.sh -v
```

### Log Files

Node logs are stored in:
- `test-network-data/bootstrap-node/node.log`
- `test-network-data/node-1/node.log`
- `test-network-data/node-2/node.log`
- etc.

## Cleanup

### Automatic Cleanup
The scripts handle cleanup automatically:
- **Ctrl+C**: Gracefully stops all nodes and cleans up processes
- **Script Exit**: Removes temporary data and terminates background processes

### Manual Cleanup
If needed, manually clean up:

#### Windows
```powershell
# Stop any remaining cargo processes
Get-Process | Where-Object {$_.ProcessName -eq "cargo"} | Stop-Process

# Remove test data
Remove-Item -Path "test-network-data" -Recurse -Force
```

#### Unix  
```bash
# Kill any remaining processes
pkill -f "cargo run.*aerolithsdb"

# Remove test data
rm -rf test-network-data
```

## Performance Considerations

### System Requirements
- **Minimum**: 2GB RAM, 2 CPU cores
- **Recommended**: 4GB RAM, 4+ CPU cores for optimal performance
- **Storage**: 500MB+ free space for test data

### Network Performance
- **Latency**: Sub-millisecond for localhost
- **Throughput**: Limited by system I/O and CPU
- **Concurrency**: Handles thousands of operations per second

### Scaling
- **Node Count**: Tested up to 10 nodes on local machine
- **Data Volume**: Suitable for datasets up to 1GB for testing
- **Concurrent Operations**: Supports hundreds of concurrent CLI operations

## Production Notes

These scripts are designed for **development and testing purposes**. For production deployment:

1. **Security**: Enable TLS, authentication, and authorization
2. **Networking**: Configure proper firewall rules and network topology
3. **Storage**: Use persistent storage with backup strategies
4. **Monitoring**: Implement comprehensive observability and alerting
5. **High Availability**: Deploy across multiple data centers/availability zones

## Contributing

To extend or modify these scripts:

1. **Test Changes**: Verify on both Windows and Unix platforms
2. **Error Handling**: Maintain robust error handling and cleanup
3. **Documentation**: Update this README with any new features
4. **Platform Compatibility**: Ensure cross-platform functionality

## License

These scripts are part of the AerolithDB project and follow the same licensing terms.
