# AerolithDB Multinode Test Network - Quick Reference

## 🚀 Launch Commands

### Quick Start (Pick One)

```bash
# 1. Quick Demo (3 nodes, 2 minutes)
.\scripts\quick-demo.ps1              # Windows
./scripts/quick-demo.sh               # Linux/macOS

# 2. Full Network (4+ nodes, comprehensive)
.\scripts\launch-local-network.ps1    # Windows
./scripts/launch-local-network.sh     # Linux/macOS

# 3. Advanced Testing (All features, 5+ minutes)
.\scripts\demo-advanced-test.ps1      # Windows  
./scripts/advanced-network-test.sh    # Linux/macOS
```

### Custom Configuration

```bash
# Custom node count and verbose logging
.\scripts\launch-local-network.ps1 -NodesCount 8 -Verbose
./scripts/launch-local-network.sh -n 8 -v

# Advanced testing with custom duration
.\scripts\demo-advanced-test.ps1 -NodesCount 6 -TestDuration 300
./scripts/advanced-network-test.sh 6 "test" "debug" 300 true
```

## 🎯 What Gets Tested Automatically

### ✅ Core Distributed Features
- **Network Formation**: Bootstrap + P2P mesh with 4+ nodes
- **Document CRUD**: CREATE, READ, UPDATE, DELETE across nodes
- **Cross-Node Queries**: Distributed search and analytics
- **Data Replication**: Automatic data sync across nodes
- **Health Monitoring**: Real-time node status checking

### ✅ Security & Authentication
- **User Roles**: Admin, developer, analyst permissions
- **Data Encryption**: AES-256 for sensitive documents
- **Authentication**: User login validation
- **Authorization**: Role-based access control
- **Audit Logging**: Complete operation tracking

### ✅ Advanced Scenarios (Advanced Tests)
- **Byzantine Fault Tolerance**: Malicious node simulation
- **Network Partitions**: Split-brain recovery testing
- **Cross-Datacenter Replication**: Multi-region sync
- **Load Testing**: Performance under stress
- **Compliance**: GDPR and financial regulations

## 🌐 After Launch - Manual Access

### Web UI Access
```
http://localhost:8080    # Bootstrap node web interface
http://localhost:8081    # Node 1 web interface  
http://localhost:8082    # Node 2 web interface
http://localhost:8083    # Node 3 web interface
```

### Network Explorer
```
http://localhost:8080/explorer    # Network topology visualization
```

### CLI Operations
```bash
# Health checks
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 health

# Create documents
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 put users test_user --data '{"name":"Test User","role":"tester"}'

# Query data
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8081 query users --filter '{"role":"tester"}'

# System stats
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8082 stats
```

## 📊 Test Data Created

### Collections & Sample Data
- **Users** (25+ entries): Engineering, Marketing, Sales departments
- **Projects** (10+ entries): Active/completed development projects  
- **Analytics** (50+ entries): Network metrics and user activity
- **Secure Documents**: Encrypted financial and PII data

### Test Users Created
```json
{
  "admin": {"role": "administrator", "password": "password123"},
  "user_001": {"name": "Alice", "department": "Engineering"},
  "user_002": {"name": "Bob", "department": "Marketing"}
}
```

## 🔧 Network Endpoints

| Node | Port | Purpose | URL |
|------|------|---------|-----|
| Bootstrap | 8080 | Seed node, Web UI | http://localhost:8080 |
| Node 1 | 8081 | Regular node | http://localhost:8081 |
| Node 2 | 8082 | Regular node | http://localhost:8082 |
| Node 3 | 8083 | Regular node | http://localhost:8083 |
| Node N | 8080+N | Additional nodes | http://localhost:808N |

## ⚡ Quick Test Commands

### Verify Network Health
```bash
# Check all nodes are healthy
for port in 8080 8081 8082 8083; do
  echo "Checking port $port..."
  cargo run --release --bin aerolithsdb-cli -- --url http://localhost:$port health
done
```

### Test Data Replication
```bash
# Create on one node
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8080 put test replication_test --data '{"test":"replication","timestamp":"'$(date)'"}'

# Read from different node (proves replication)
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8082 get test replication_test
```

### Test Cross-Node Queries
```bash
# Query users by department
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8081 query users --filter '{"department":"Engineering"}'

# Query active projects  
cargo run --release --bin aerolithsdb-cli -- --url http://localhost:8083 query projects --filter '{"status":"active"}'
```

## 🛑 Shutdown

The scripts handle graceful shutdown when you press **Ctrl+C**. All nodes will be stopped and data will be preserved.

## 📚 Documentation

- **Comprehensive Guide**: `COMPREHENSIVE_MULTINODE_TEST_GUIDE.md`
- **Script Documentation**: `scripts/README.md`
- **Network Features**: `NETWORK_SCRIPTS_SUMMARY.md`
- **Test Results**: `BATTLE_TEST_RESULTS.md`

## 🎉 Success Indicators

After running any script, you should see:
- ✅ All nodes report "HEALTHY" status
- ✅ Documents created across multiple nodes
- ✅ Cross-node queries return data  
- ✅ Health checks pass on all endpoints
- ✅ Network remains running for manual testing

**The network demonstrates a fully functional distributed database with enterprise-grade features ready for production deployment.**
