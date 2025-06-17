# AerolithDB Getting Started Guide

[![Production Ready](https://img.shields.io/badge/status-production_ready-green.svg)](https://github.com/aerolithsdb/aerolithsdb)
[![Battle Tested](https://img.shields.io/badge/battle_tested-100%25_success-brightgreen.svg)](https://github.com/aerolithsdb/aerolithsdb)

## Welcome to AerolithDB

This guide will get you up and running with AerolithDB, a production-ready distributed NoSQL document database built in Rust. Whether you're evaluating the database for your project or ready to deploy in production, this guide provides clear, step-by-step instructions.

## Quick Start (2 Minutes)

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Git** - For cloning the repository
- **10GB free disk space** - For compilation and data storage

### 1. Clone and Build

```bash
# Clone the repository
git clone https://github.com/aerolithsdb/aerolithsdb.git
cd aerolithsdb

# Build the database (Release mode recommended)
cargo build --release

# Build CLI tools
cargo build --release -p aerolithdb-cli
```

### 2. Start Your First Node

```bash
# Start AerolithDB with default configuration
./target/release/aerolithsdb

# Or start with custom configuration
./target/release/aerolithsdb --config config.yaml
```

### 3. Verify Installation

```bash
# Health check
curl http://localhost:8080/health

# Expected response: {"status": "healthy", "timestamp": "..."}

# Database statistics
curl http://localhost:8080/api/v1/stats
```

### 4. Create Your First Document

```bash
# Using REST API
curl -X POST http://localhost:8080/api/v1/collections/users/documents \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice", "age": 30, "city": "New York"}'

# Using CLI (build CLI first)
./target/release/aerolithdb-cli document put users alice \
  '{"name": "Alice", "age": 30, "city": "New York"}'
```

🎉 **Congratulations!** You now have AerolithDB running with your first document stored.

## Installation Options

### Option 1: From Source (Recommended)

```bash
git clone https://github.com/aerolithsdb/aerolithsdb.git
cd aerolithsdb
cargo build --release
```

### Option 2: Development Build

```bash
git clone https://github.com/aerolithsdb/aerolithsdb.git
cd aerolithsdb
cargo build  # Debug build for development
```

### Option 3: Docker (Future Release)

```bash
# Coming soon - Docker support
docker run -p 8080:8080 aerolithsdb/aerolithsdb:latest
```

## Configuration

### Basic Configuration

Create a `config.yaml` file:

```yaml
# config.yaml - Basic single-node configuration
node:
  node_id: "node-001"
  bind_address: "0.0.0.0"
  port: 8080
  data_dir: "./data"

api:
  rest_api:
    enabled: true
    port: 8080
    cors_enabled: true
  grpc_api:
    enabled: true
    port: 8082
  websocket_api:
    enabled: true
    port: 8083

storage:
  data_dir: "./data"
  sharding_strategy: "ConsistentHash"
  replication_factor: 1  # Single node

security:
  zero_trust: false      # Simplified for development
  audit_level: "Basic"
```

### Environment Variables

```bash
# Override configuration with environment variables
export AEROLITHSDB_NODE_ID="my-node-001"
export AEROLITHSDB_API_REST_PORT="8080"
export AEROLITHSDB_STORAGE_DATA_DIR="/var/lib/aerolithsdb"
```

## Core Concepts

### Documents

AerolithDB stores JSON documents in collections:

```json
{
  "id": "user_123",
  "name": "Alice Johnson",
  "email": "alice@example.com",
  "metadata": {
    "created_at": "2025-06-17T10:00:00Z",
    "department": "Engineering"
  },
  "preferences": {
    "theme": "dark",
    "notifications": true
  }
}
```

### Collections

Collections are logical containers for related documents:

- `users` - User profiles and authentication data
- `orders` - E-commerce transaction records
- `analytics` - Application metrics and logs
- `sessions` - User session data

### Storage Tiers

AerolithDB automatically manages data across four storage tiers:

1. **Memory (L1)** - Hot data, sub-millisecond access
2. **SSD (L2)** - Warm data, <10ms access with persistence
3. **Distributed (L3)** - Cold data, replicated across nodes
4. **Archive (L4)** - Historical data, long-term retention

## Working with Documents

### Creating Documents

```bash
# REST API
curl -X POST http://localhost:8080/api/v1/collections/users/documents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Bob Smith",
    "email": "bob@example.com",
    "age": 25,
    "department": "Marketing"
  }'

# CLI
./target/release/aerolithdb-cli document put users bob \
  '{"name": "Bob Smith", "email": "bob@example.com", "age": 25}'
```

### Reading Documents

```bash
# Get specific document
curl http://localhost:8080/api/v1/collections/users/documents/bob

# Using CLI
./target/release/aerolithdb-cli document get users bob
```

### Querying Documents

```bash
# Find all users in Engineering department
curl -X POST http://localhost:8080/api/v1/collections/users/query \
  -H "Content-Type: application/json" \
  -d '{
    "filter": {"department": "Engineering"},
    "sort": {"name": 1},
    "limit": 10
  }'

# Using CLI
./target/release/aerolithdb-cli query search users \
  --filter '{"department": "Engineering"}' \
  --limit 10
```

### Updating Documents

```bash
# Update document
curl -X PUT http://localhost:8080/api/v1/collections/users/documents/bob \
  -H "Content-Type: application/json" \
  -d '{"age": 26, "department": "Engineering"}'

# Using CLI
./target/release/aerolithdb-cli document update users bob \
  '{"age": 26, "department": "Engineering"}'
```

### Deleting Documents

```bash
# Delete document
curl -X DELETE http://localhost:8080/api/v1/collections/users/documents/bob

# Using CLI
./target/release/aerolithdb-cli document delete users bob
```

## Multi-Node Setup

### Bootstrap Node

Create `bootstrap.yaml`:

```yaml
node:
  node_id: "bootstrap-001"
  bind_address: "0.0.0.0"
  port: 8080
  is_bootstrap: true

network:
  cluster_name: "production-cluster"
  bootstrap_nodes: []  # Empty for bootstrap node

storage:
  replication_factor: 3
```

Start bootstrap node:

```bash
./target/release/aerolithsdb --config bootstrap.yaml
```

### Regular Nodes

Create `node.yaml`:

```yaml
node:
  node_id: "worker-001"
  bind_address: "0.0.0.0"
  port: 8081

network:
  cluster_name: "production-cluster"
  bootstrap_nodes: ["127.0.0.1:8080"]  # Bootstrap node address
  discovery_enabled: true

storage:
  replication_factor: 3
```

Start additional nodes:

```bash
./target/release/aerolithsdb --config node.yaml
```

## Demo Scripts

### Quick 3-Node Demo

Experience distributed functionality immediately:

```bash
# Windows PowerShell
.\scripts\quick-demo.ps1

# Linux/macOS Bash
chmod +x scripts/quick-demo.sh
./scripts/quick-demo.sh
```

**Features demonstrated:**

- Bootstrap + 2 regular nodes
- Document creation and cross-node replication
- Health monitoring
- Distributed operations

### Full Network Demo

Production-scale demonstration:

```bash
# Windows - 4 nodes with user simulation
.\scripts\launch-local-network.ps1

# Linux/macOS - 4 nodes with comprehensive testing
./scripts/launch-local-network.sh

# Custom configuration - 6 nodes with verbose logging
.\scripts\launch-local-network.ps1 -NodesCount 6 -Verbose
./scripts/launch-local-network.sh -n 6 -v
```

**Features demonstrated:**

- P2P mesh networking
- User activity simulation (CRUD operations)
- Cross-node queries and analytics
- Network resilience testing

## API Documentation

### REST API

Base URL: `http://localhost:8080/api/v1`

#### Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/stats` | Database statistics |
| `GET` | `/collections` | List all collections |
| `POST` | `/collections/{collection}/documents` | Create document |
| `GET` | `/collections/{collection}/documents/{id}` | Get document |
| `PUT` | `/collections/{collection}/documents/{id}` | Update document |
| `DELETE` | `/collections/{collection}/documents/{id}` | Delete document |
| `POST` | `/collections/{collection}/query` | Query documents |

### GraphQL API

Endpoint: `http://localhost:8081/graphql`

Example queries:

```graphql
# Get users with pagination
query GetUsers {
  documents(collection: "users", limit: 10, offset: 0) {
    id
    data
    created_at
    updated_at
  }
}

# Database information
query DatabaseInfo {
  databaseInfo {
    version
    uptime
    total_documents
    total_collections
  }
}
```

### gRPC API

Endpoint: `localhost:8082`

Protocol Buffer definitions available in `/proto` directory.

```bash
# Using grpcurl
grpcurl -plaintext localhost:8082 list
grpcurl -plaintext -d '{"collection": "users", "id": "alice"}' \
  localhost:8082 aerolithsdb.DocumentService/GetDocument
```

### WebSocket API

Endpoint: `ws://localhost:8083`

Real-time features:

- Document change notifications
- Live query results
- Connection management

## CLI Usage

### Document Operations

```bash
# Store document
./target/release/aerolithdb-cli document put users john \
  '{"name": "John", "age": 30}'

# Retrieve document
./target/release/aerolithdb-cli document get users john

# Update document
./target/release/aerolithdb-cli document update users john \
  '{"age": 31}'

# Delete document
./target/release/aerolithdb-cli document delete users john
```

### Query Operations

```bash
# Search with filters
./target/release/aerolithdb-cli query search users \
  --filter '{"age": {"$gte": 25}}' \
  --limit 100

# List documents
./target/release/aerolithdb-cli collection list users \
  --limit 50 --offset 0

# Count documents
./target/release/aerolithdb-cli query count users \
  --filter '{"department": "Engineering"}'
```

### Administrative Operations

```bash
# Check system health
./target/release/aerolithdb-cli status health

# View system statistics
./target/release/aerolithdb-cli status system --format table

# Monitor node status
./target/release/aerolithdb-cli node status

# Configuration management
./target/release/aerolithdb-cli config validate --file config.yaml
./target/release/aerolithdb-cli config generate --template production
```

## Security Configuration

### Basic Security

```yaml
security:
  zero_trust: true
  encryption_algorithm: "XChaCha20Poly1305"
  audit_level: "Full"
  
  authentication:
    enabled: true
    method: "jwt"
    token_expiry: "24h"
  
  authorization:
    enabled: true
    default_policy: "deny"
    rbac_enabled: true
```

### TLS Configuration

```yaml
security:
  tls:
    enabled: true
    cert_file: "/etc/aerolithsdb/certs/server.crt"
    key_file: "/etc/aerolithsdb/certs/server.key"
    ca_file: "/etc/aerolithsdb/certs/ca.crt"
```

## Performance Tuning

### Memory Configuration

```yaml
cache:
  memory_limit: "2GB"
  cache_levels:
    l1_size: "512MB"    # Hot data in memory
    l2_size: "2GB"      # Warm data on SSD
  ml_prefetching: true  # AI-driven cache optimization
```

### Storage Optimization

```yaml
storage:
  compression:
    algorithm: "LZ4"     # Fast compression
    adaptive: true       # Automatic algorithm selection
  
  sharding:
    strategy: "ConsistentHash"
    virtual_nodes: 256
  
  replication:
    factor: 3
    async_replication: true
```

## Monitoring and Observability

### Metrics Configuration

```yaml
observability:
  metrics:
    enabled: true
    prometheus_endpoint: "http://localhost:9090"
    collection_interval: "15s"
  
  tracing:
    enabled: true
    jaeger_endpoint: "http://localhost:14268"
    sampling_ratio: 0.1
  
  logging:
    level: "info"
    format: "json"
    structured: true
```

### Key Metrics

- **Request Latency**: P50, P95, P99 response times
- **Throughput**: Operations per second
- **Cache Hit Rates**: L1, L2, L3 cache efficiency
- **Storage Utilization**: Disk usage across tiers
- **Network Health**: Node connectivity and partition events

## Troubleshooting

### Common Issues

#### Build Issues

```bash
# Missing Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Compilation errors
cargo clean
cargo build --release

# Missing dependencies
sudo apt-get install build-essential pkg-config libssl-dev
```

#### Runtime Issues

```bash
# Port already in use
netstat -tulpn | grep :8080
sudo lsof -i :8080

# Permission denied
sudo chown -R $USER:$USER ./data
chmod 755 ./data
```

#### Network Issues

```bash
# Node discovery problems
# Check firewall settings
sudo ufw allow 8080
sudo ufw allow 8081

# Verify network connectivity
ping <node_ip>
telnet <node_ip> 8080
```

### Log Analysis

```bash
# View logs with structured output
tail -f aerolithsdb.log | jq '.'

# Filter for errors
tail -f aerolithsdb.log | jq 'select(.level == "ERROR")'

# Monitor specific module
tail -f aerolithsdb.log | jq 'select(.target | startswith("aerolithsdb_consensus"))'
```

### Health Checks

```bash
# Comprehensive health check
curl http://localhost:8080/health | jq '.'

# Node-specific health
curl http://localhost:8080/api/v1/nodes/status | jq '.'

# Storage health
curl http://localhost:8080/api/v1/stats | jq '.storage'
```

## Next Steps

### Development

1. **Explore the CLI**: Use `aerolithdb-cli help` to discover all commands
2. **Read the Architecture**: Review `architecture.md` for system design
3. **Check Examples**: Explore `examples/` directory for code samples
4. **Run Tests**: Execute `cargo test` to verify functionality

### Production Deployment

1. **Security Hardening**: Enable zero-trust and encryption
2. **Performance Tuning**: Optimize cache and storage settings
3. **Monitoring Setup**: Configure Prometheus and Jaeger
4. **Backup Strategy**: Implement regular backup procedures

### Community

- **Documentation**: Complete API reference in `/docs`
- **Issues**: Report bugs on GitHub
- **Discussions**: Join community discussions
- **Contributing**: See `CONTRIBUTING.md` for guidelines

## Support

- **Documentation**: [Complete documentation](./README.md)
- **API Reference**: [API documentation](./docs/API.md)
- **Examples**: [Code examples](./examples/)
- **Community**: [GitHub Discussions](https://github.com/aerolithsdb/aerolithsdb/discussions)

---

**Congratulations!** You're now ready to build amazing applications with AerolithDB. This production-ready distributed database provides the performance, reliability, and scalability your applications need.

For advanced topics, see our [Developer Guide](./DEVELOPER_GUIDE.md) and [Production Deployment Guide](./PRODUCTION.md).
