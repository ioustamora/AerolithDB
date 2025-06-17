# AerolithDB Comprehensive Multinode Test Network Guide

## 🎯 Overview

This guide provides complete instructions for setting up, running, and validating a persistent AerolithDB multinode test network that exercises all current distributed database features including user roles, security, replication, encryption, admin operations, and performance testing.

## 🚀 Quick Start

### Option 1: Quick Demo (3 nodes, fast)
```bash
# Windows (PowerShell)
.\scripts\quick-demo.ps1

# Linux/macOS (Bash)
./scripts/quick-demo.sh
```

### Option 2: Full Network Demo (4+ nodes, comprehensive)
```bash
# Windows (PowerShell) - Default 4 nodes
.\scripts\launch-local-network.ps1

# Windows (PowerShell) - Custom 6 nodes with verbose logging
.\scripts\launch-local-network.ps1 -NodesCount 6 -Verbose

# Linux/macOS (Bash) - Default 4 nodes
./scripts/launch-local-network.sh

# Linux/macOS (Bash) - Custom 8 nodes with verbose logging  
./scripts/launch-local-network.sh -n 8 -v
```

### Option 3: Advanced Network Testing (Byzantine, partitions, load testing)
```bash
# Windows (PowerShell) - Advanced scenarios
.\scripts\demo-advanced-test.ps1 -NodesCount 6 -TestDuration 300

# Linux/macOS (Bash) - Advanced scenarios
./scripts/advanced-network-test.sh 6 "advanced-test" "debug" 300 true
```

## 🏗️ Network Architecture

The scripts create a persistent distributed network with the following topology:

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Bootstrap     │────▶│   Regular       │────▶│   Regular       │
│   Node          │     │   Node 1        │     │   Node 2        │
│   Port: 8080    │     │   Port: 8081    │     │   Port: 8082    │
└─────────────────┘     └─────────────────┘     └─────────────────┘
         │                        │                        │
         ▼                        ▼                        ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Regular       │────▶│   Regular       │     │   Web UI /      │
│   Node 3        │     │   Node 4+       │     │   Network       │
│   Port: 8083    │     │   Port: 8084+   │     │   Explorer      │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

### Node Types

1. **Bootstrap Node** (Port 8080)
   - Initial seed node for network formation
   - Provides cluster discovery services
   - Maintains network topology information
   - Web UI and Network Explorer access point

2. **Regular Nodes** (Ports 8081-8084+)
   - Full database functionality
   - P2P mesh connectivity
   - Data replication and storage
   - Query processing capabilities

## 🧪 Automated Test Coverage

The scripts automatically test the following distributed features:

### 1. Network Formation & Discovery
- ✅ Bootstrap node initialization
- ✅ P2P peer discovery and mesh formation
- ✅ Health verification across all nodes
- ✅ Cross-node connectivity validation

### 2. Document Operations (CRUD)
- ✅ **CREATE**: Documents stored across different nodes
- ✅ **READ**: Cross-node data retrieval demonstrating replication
- ✅ **UPDATE**: Distributed consistency validation
- ✅ **DELETE**: Network-wide data removal

### 3. Query & Analytics
- ✅ Distributed query execution
- ✅ Cross-node search operations
- ✅ Real-time analytics collection
- ✅ Performance metrics gathering

### 4. Security & Encryption
- ✅ Data encryption at rest and in transit
- ✅ User authentication testing
- ✅ Authorization validation
- ✅ Secure document storage/retrieval

### 5. Administrative Operations
- ✅ Health checks across all nodes
- ✅ System statistics collection
- ✅ Network status monitoring
- ✅ Performance metrics reporting

### 6. Advanced Scenarios (Advanced Tests Only)
- ✅ Byzantine fault tolerance simulation
- ✅ Network partition and recovery testing
- ✅ Cross-datacenter replication
- ✅ Load testing with detailed metrics

## 🔑 User Roles & Security Testing

### Built-in Test Users
The scripts create the following test users with different roles:

```json
{
  "admin": {
    "username": "admin",
    "password": "password123",
    "role": "administrator",
    "permissions": ["read", "write", "admin", "delete"]
  },
  "user_001": {
    "username": "alice",
    "department": "Engineering", 
    "role": "developer",
    "permissions": ["read", "write"]
  },
  "user_002": {
    "username": "bob",
    "department": "Marketing",
    "role": "analyst", 
    "permissions": ["read"]
  }
}
```

### Security Validation Tests
- **Authentication**: Valid/invalid login attempts
- **Authorization**: Role-based access control
- **Encryption**: Sensitive data protection
- **Audit Logging**: Complete operation tracking

## 🔧 Configuration Options

### Network Size Configuration
```bash
# Small test (3 nodes)
.\scripts\quick-demo.ps1

# Medium test (6 nodes) 
.\scripts\launch-local-network.ps1 -NodesCount 6

# Large test (10+ nodes)
.\scripts\launch-local-network.ps1 -NodesCount 12
```

### Port Configuration
The scripts automatically allocate ports starting from 8080:
- Bootstrap: 8080
- Node 1: 8081  
- Node 2: 8082
- Node N: 8080 + N

### Data Directory Configuration
```bash
# Custom data directory
.\scripts\launch-local-network.ps1 -DataDir "my-test-network"

# Linux/macOS with custom directory
./scripts/launch-local-network.sh -d "my-test-network"
```

### Logging Configuration
```bash
# Verbose logging
.\scripts\launch-local-network.ps1 -Verbose

# Debug level logging (advanced tests)
.\scripts\demo-advanced-test.ps1 -LogLevel "debug"
```

## 🎮 Manual Testing & Validation

After the automated tests complete, the network remains running for manual operations:

### Health Checks
```bash
# Check bootstrap node health
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 health

# Check all nodes
for port in 8080 8081 8082 8083; do
  cargo run --release --bin aerolithsdb-cli -- --url http://localhost:$port health
done
```

### Document Operations
```bash
# Create a document on one node
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 put users user_003 --data '{"name":"Charlie Brown","department":"Sales","role":"manager"}'

# Read from a different node (demonstrates replication)
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8082 get users user_003

# Update document with role change
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8081 put users user_003 --data '{"name":"Charlie Brown","department":"Sales","role":"senior_manager"}'

# Delete document
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8083 delete users user_003
```

### Query Operations
```bash
# Query by department
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 query users --filter '{"department":"Engineering"}'

# Query by role
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8081 query users --filter '{"role":"developer"}'

# Analytics queries
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8082 query analytics --filter '{"event_type":"network_test"}'

# Active projects
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8083 query projects --filter '{"status":"active"}'
```

### Administrative Operations
```bash
# System statistics from each node
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 stats
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8081 stats

# Network topology information
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 network-info

# Performance metrics
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 metrics
```

### Security & Encryption Testing
```bash
# Create encrypted document
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 put secure_docs sensitive_001 --data '{"ssn":"123-45-6789","credit_card":"4111-1111-1111-1111"}' --encrypt

# Retrieve and decrypt
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8081 get secure_docs sensitive_001 --decrypt

# Test authentication (when auth is enabled)
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 --auth admin:password123 stats
```

## 🌐 Web UI & Network Explorer Access

Once the network is running, you can access the web interfaces:

### Web UI Access
```bash
# Open in browser
http://localhost:8080        # Bootstrap node web interface
http://localhost:8081        # Node 1 web interface
http://localhost:8082        # Node 2 web interface
```

### Network Explorer
The bootstrap node provides a Network Explorer interface:
```bash
http://localhost:8080/explorer
```

**Network Explorer Features:**
- Real-time network topology visualization
- Node health and status monitoring
- Document distribution across nodes
- Query execution interface
- Performance metrics dashboard

## 📊 Test Data & Scenarios

### Pre-loaded Test Data
The scripts automatically create:

1. **Users Collection** (25+ users)
   - Engineering department users
   - Marketing department users  
   - Sales department users
   - Admin users

2. **Projects Collection** (10+ projects)
   - Active development projects
   - Completed projects
   - Research initiatives

3. **Analytics Collection** (50+ events)
   - Network test events
   - User activity metrics
   - Performance measurements

4. **Secure Documents** (Advanced tests)
   - Encrypted financial data
   - PII with encryption
   - Compliance audit trails

### Custom Test Data
You can add your own test data:

```bash
# Custom user
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 put users custom_user --data '{
  "name": "Your Name",
  "department": "Custom Department", 
  "role": "custom_role",
  "permissions": ["read", "write"],
  "created_at": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'"
}'

# Custom project
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8081 put projects custom_project --data '{
  "name": "My Test Project",
  "description": "Testing AerolithDB features",
  "status": "active",
  "team_members": ["custom_user"],
  "start_date": "'$(date -u +%Y-%m-%d)'"
}'
```

## 🧩 Comprehensive Test Scenarios

### Scenario 1: User Management Workflow
```bash
# 1. Create new user
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 put users new_employee --data '{"name":"Jane Doe","department":"HR","role":"specialist"}'

# 2. Verify user exists on different node
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8082 get users new_employee

# 3. Update user role
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8081 put users new_employee --data '{"name":"Jane Doe","department":"HR","role":"manager"}'

# 4. Query users by department
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8083 query users --filter '{"department":"HR"}'

# 5. Verify replication across all nodes
for port in 8080 8081 8082 8083; do
  echo "Checking node $port:"
  cargo run --release --bin aerolithsdb-cli -- --url http://localhost:$port get users new_employee
done
```

### Scenario 2: Project Collaboration Workflow
```bash
# 1. Create project
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 put projects team_project --data '{
  "name":"Distributed Database Enhancement",
  "description":"Improving consensus algorithms",
  "status":"active",
  "team_members":["user_001","user_002","new_employee"],
  "start_date":"'$(date -u +%Y-%m-%d)'"
}'

# 2. Add project updates from different nodes
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8081 put project_updates update_001 --data '{
  "project_id":"team_project",
  "author":"user_001", 
  "update":"Initial architecture design completed",
  "timestamp":"'$(date -u +%Y-%m-%dT%H:%M:%SZ)'"
}'

cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8082 put project_updates update_002 --data '{
  "project_id":"team_project",
  "author":"user_002",
  "update":"Consensus algorithm implementation started", 
  "timestamp":"'$(date -u +%Y-%m-%dT%H:%M:%SZ)'"
}'

# 3. Query project status across nodes
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8083 query projects --filter '{"status":"active"}'
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 query project_updates --filter '{"project_id":"team_project"}'
```

### Scenario 3: Analytics & Reporting Workflow
```bash
# 1. Generate analytics events
for i in {1..10}; do
  port=$((8080 + (i % 4)))
  cargo run --release --bin aerolithsdb-cli -- --url http://localhost:$port put analytics event_$i --data '{
    "event_type":"user_activity",
    "user_id":"user_'$(printf "%03d" $i)'",
    "action":"document_view",
    "timestamp":"'$(date -u +%Y-%m-%dT%H:%M:%SZ)'",
    "node_id":"node_'$((port - 8080))'"
  }'
done

# 2. Query analytics from different nodes
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8081 query analytics --filter '{"event_type":"user_activity"}'

# 3. Generate performance metrics
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8082 put metrics performance_$(date +%s) --data '{
  "metric_type":"query_performance",
  "avg_latency_ms":15,
  "throughput_ops_sec":1200,
  "timestamp":"'$(date -u +%Y-%m-%dT%H:%M:%SZ)'",
  "node_cluster_size":4
}'
```

### Scenario 4: Security & Compliance Workflow
```bash
# 1. Store sensitive data (encrypted)
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 put confidential audit_001 --data '{
  "document_type":"financial_audit",
  "customer_id":"CUST_12345",
  "amount":50000,
  "transaction_id":"TXN_789",
  "compliance_officer":"jane.doe@company.com",
  "timestamp":"'$(date -u +%Y-%m-%dT%H:%M:%SZ)'"
}' --encrypt

# 2. Verify encryption across nodes
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8081 get confidential audit_001 --decrypt

# 3. Create audit trail
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8082 put audit_log access_$(date +%s) --data '{
  "accessed_document":"audit_001",
  "accessed_by":"compliance_officer",
  "access_type":"read",
  "timestamp":"'$(date -u +%Y-%m-%dT%H:%M:%SZ)'",
  "ip_address":"127.0.0.1",
  "authorized":true
}'

# 4. Query compliance data
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8083 query audit_log --filter '{"authorized":true}'
```

## 🔍 Validation & Verification

### Network Health Verification
```bash
# Create a comprehensive health check script
cat > health_check.sh << 'EOF'
#!/bin/bash
echo "=== AerolithDB Network Health Check ==="
for port in 8080 8081 8082 8083 8084; do
  echo "Checking node on port $port..."
  if cargo run --release --bin aerolithsdb-cli -- --url http://localhost:$port health &>/dev/null; then
    echo "✅ Node $port: HEALTHY"
  else
    echo "❌ Node $port: UNAVAILABLE" 
  fi
done
EOF
chmod +x health_check.sh
./health_check.sh
```

### Data Consistency Verification
```bash
# Create consistency check script
cat > consistency_check.sh << 'EOF'
#!/bin/bash
echo "=== Data Consistency Check ==="
test_doc='{"test_id":"consistency_check","timestamp":"'$(date -u +%Y-%m-%dT%H:%M:%SZ)'"}'

# Write to one node
echo "Writing test document to node 8080..."
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 put test_collection consistency_test --data "$test_doc"

sleep 2

# Read from all other nodes
for port in 8081 8082 8083; do
  echo "Reading from node $port..."
  if cargo run --release --bin aerolithsdb-cli -- --url http://localhost:$port get test_collection consistency_test &>/dev/null; then
    echo "✅ Node $port: Data replicated successfully"
  else
    echo "❌ Node $port: Data not yet replicated"
  fi
done
EOF
chmod +x consistency_check.sh
./consistency_check.sh
```

### Performance Verification
```bash
# Simple performance test
cat > performance_test.sh << 'EOF'
#!/bin/bash
echo "=== Performance Test ==="
start_time=$(date +%s%N)

# Perform 10 operations across different nodes
for i in {1..10}; do
  port=$((8080 + (i % 4)))
  cargo run --release --bin aerolithsdb-cli -- --url http://localhost:$port put perf_test doc_$i --data '{"id":'$i',"data":"performance test"}' &>/dev/null
done

end_time=$(date +%s%N)
duration=$(( (end_time - start_time) / 1000000 ))
ops_per_sec=$(( 10000 / duration ))

echo "✅ 10 operations completed in ${duration}ms"
echo "✅ Throughput: ${ops_per_sec} ops/second"
EOF
chmod +x performance_test.sh
./performance_test.sh
```

## 🛠️ Troubleshooting

### Common Issues

1. **Port Already in Use**
   ```bash
   # Check which process is using port 8080
   netstat -tulpn | grep 8080  # Linux
   netstat -ano | findstr 8080 # Windows
   
   # Kill process if needed
   sudo kill -9 <PID>          # Linux
   taskkill /PID <PID> /F      # Windows
   ```

2. **Network Formation Issues**
   ```bash
   # Check node logs
   tail -f test-network-data/bootstrap/aerolithsdb.log
   tail -f test-network-data/node-1/aerolithsdb.log
   
   # Verify network connectivity
   ping localhost
   curl http://localhost:8080/health
   ```

3. **Build Issues**
   ```bash
   # Clean and rebuild
   cargo clean
   cargo build --release
   
   # Update dependencies
   cargo update
   ```

### Recovery Procedures

1. **Restart Single Node**
   ```bash
   # Stop specific node (find PID from logs)
   kill <node_pid>
   
   # Restart node manually
   cd AerolithDB
   AEROLITHSDB_NODE_ID="node-2" \
   AEROLITHSDB_STORAGE_DATA_DIR="./test-network-data/node-2" \
   AEROLITHSDB_API_REST_PORT="8082" \
   AEROLITHSDB_NETWORK_BOOTSTRAP_NODES="http://localhost:8080" \
   cargo run --release --bin aerolithsdb &
   ```

2. **Full Network Restart**
   ```bash
   # Stop all nodes (scripts handle this on Ctrl+C)
   # Then restart
   .\scripts\launch-local-network.ps1  # Windows
   ./scripts/launch-local-network.sh   # Linux/macOS
   ```

## 📚 Additional Resources

### Documentation
- `scripts/README.md` - Detailed script documentation
- `NETWORK_SCRIPTS_SUMMARY.md` - Network orchestration features
- `ADVANCED_TESTING_DEMONSTRATION_SUMMARY.md` - Advanced test scenarios
- `BATTLE_TEST_RESULTS.md` - Comprehensive test results

### Example Commands
- `scripts/launch-local-network.ps1 -Help` - PowerShell help
- `scripts/launch-local-network.sh -h` - Bash help

### Test Scripts
- `tests/simple_network_test.rs` - Basic network functionality tests
- `tests/network_battle_test.rs` - Comprehensive distributed tests
- `scripts/advanced-network-test.sh` - Advanced scenario testing

## 🎉 Conclusion

This guide provides a complete framework for testing all AerolithDB distributed features in a persistent multinode environment. The network supports:

- ✅ **Automated Testing**: Comprehensive validation of all distributed features
- ✅ **Manual Testing**: CLI and web UI access for manual operations  
- ✅ **Persistent Network**: Remains running for extended testing and validation
- ✅ **Configurable Scale**: From 3-node demos to 12+ node stress tests
- ✅ **Security Testing**: User roles, encryption, authentication, and authorization
- ✅ **Performance Testing**: Load testing and throughput validation
- ✅ **Enterprise Features**: Replication, consensus, partition recovery, and more

The scripts and tests are production-ready and provide a solid foundation for validating AerolithDB's distributed capabilities in a controlled environment.
