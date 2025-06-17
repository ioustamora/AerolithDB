# AerolithDB Developer Guide

[![Production Ready](https://img.shields.io/badge/status-production_ready-green.svg)](https://github.com/aerolithsdb/aerolithsdb)
[![Battle Tested](https://img.shields.io/badge/battle_tested-100%25_success-brightgreen.svg)](https://github.com/aerolithsdb/aerolithsdb)

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Building and Testing](#building-and-testing)
- [Contributing](#contributing)
- [API Development](#api-development)
- [Storage Engine](#storage-engine)
- [Consensus Algorithm](#consensus-algorithm)
- [Network Protocol](#network-protocol)
- [Performance Optimization](#performance-optimization)
- [Security Implementation](#security-implementation)
- [Debugging and Profiling](#debugging-and-profiling)

## Architecture Overview

AerolithDB is a distributed NoSQL document database built in Rust with a modular architecture:

```
┌─────────────────────────────────────────────────────────────┐
│                    AerolithDB Core                         │
├─────────────┬─────────────┬─────────────┬─────────────────┤
│  REST API   │  GraphQL    │   gRPC      │   WebSocket     │
│  (8080)     │  (8080)     │   (8082)    │   (8083)        │
├─────────────┴─────────────┴─────────────┴─────────────────┤
│                 Query Engine                                │
├─────────────────────────────────────────────────────────────┤
│  Consensus Layer (Byzantine Fault Tolerance)               │
├─────────────────────────────────────────────────────────────┤
│  Storage Engine (Multi-tier: Memory → SSD → HDD → S3)      │
├─────────────────────────────────────────────────────────────┤
│  Network Layer (P2P Mesh, Auto-discovery)                  │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

**aerolithdb-core**: Central node management and coordination
**aerolithdb-storage**: Multi-tier storage with automatic data lifecycle
**aerolithdb-consensus**: Byzantine fault-tolerant consensus algorithm
**aerolithdb-query**: Advanced query engine with indexing and optimization
**aerolithdb-api**: Multi-protocol API layer (REST, GraphQL, gRPC, WebSocket)
**aerolithdb-network**: P2P networking with auto-discovery
**aerolithdb-security**: Zero-trust security model with comprehensive auditing
**aerolithdb-cli**: Command-line interface for administration and development

## Development Environment

### Prerequisites

- **Rust 1.70+** with `cargo`
- **Git** for version control
- **Docker** (optional, for integration testing)
- **IDE**: VS Code with Rust-analyzer extension recommended

### Setup Development Environment

```bash
# Clone repository
git clone https://github.com/aerolithsdb/aerolithsdb.git
cd aerolithsdb

# Install development dependencies
cargo install cargo-watch cargo-audit cargo-outdated

# Set up Git hooks (optional)
git config core.hooksPath .githooks

# Run development build
cargo build

# Start development server with auto-reload
cargo watch -x "run"
```

### Environment Configuration

Create `.env` for development:

```bash
# .env - Development environment
RUST_LOG=debug
AEROLITHSDB_ENV=development
AEROLITHSDB_API_REST_PORT=8080
AEROLITHSDB_STORAGE_DATA_DIR=./dev_data
AEROLITHSDB_CONSENSUS_BOOTSTRAP=true
```

## Project Structure

```
aerolithdb/
├── Cargo.toml                 # Workspace configuration
├── Cargo.lock                 # Dependency lock file
├── README.md                  # Project documentation
├── docs/                      # Additional documentation
│   ├── GETTING_STARTED.md
│   ├── DEVELOPER_GUIDE.md
│   └── PRODUCTION_DEPLOYMENT.md
├── src/
│   └── main.rs               # Application entry point
├── aerolithdb-core/          # Core node functionality
│   ├── src/
│   │   ├── lib.rs           # Public API
│   │   ├── node.rs          # Node management
│   │   ├── config.rs        # Configuration handling
│   │   └── types.rs         # Core data types
│   └── Cargo.toml
├── aerolithdb-storage/       # Storage engine
│   ├── src/
│   │   ├── lib.rs           # Storage API
│   │   ├── backends.rs      # Storage backend implementations
│   │   ├── sharding.rs      # Data sharding logic
│   │   ├── replication.rs   # Replication management
│   │   └── compression.rs   # Data compression
│   └── Cargo.toml
├── aerolithdb-consensus/     # Consensus algorithm
│   ├── src/
│   │   ├── lib.rs           # Consensus API
│   │   ├── engine.rs        # Core consensus logic
│   │   ├── vector_clock.rs  # Vector clock implementation
│   │   └── byzantine_tolerance.rs # Byzantine fault tolerance
│   └── Cargo.toml
├── aerolithdb-query/         # Query engine
│   ├── src/
│   │   ├── lib.rs           # Query API
│   │   ├── engine.rs        # Query execution engine
│   │   ├── processing.rs    # Query processing pipeline
│   │   └── stats.rs         # Query statistics
│   └── Cargo.toml
├── aerolithdb-api/           # API layer
│   ├── src/
│   │   ├── lib.rs           # API exports
│   │   ├── rest.rs          # REST API implementation
│   │   ├── graphql.rs       # GraphQL API
│   │   ├── grpc.rs          # gRPC API
│   │   └── websocket.rs     # WebSocket API
│   └── Cargo.toml
├── aerolithdb-network/       # Networking
│   └── src/lib.rs           # P2P networking implementation
├── aerolithdb-security/      # Security
│   └── src/lib.rs           # Security and authentication
├── aerolithdb-cli/           # Command-line interface
│   ├── src/
│   │   ├── main.rs          # CLI entry point
│   │   ├── commands.rs      # Command implementations
│   │   └── client.rs        # API client
│   └── Cargo.toml
├── tests/                    # Integration tests
│   ├── minimal_battle_test.rs
│   └── network_battle_test.rs
└── examples/                 # Usage examples
    └── grpc_protobuf_client.rs
```

## Building and Testing

### Development Build

```bash
# Build all components
cargo build

# Build specific component
cargo build -p aerolithdb-storage

# Build with all features
cargo build --all-features

# Release build
cargo build --release
```

### Testing Strategy

```bash
# Run unit tests
cargo test

# Run unit tests with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_document_storage

# Run integration tests
cargo test --test minimal_battle_test

# Run network battle test (requires multiple terminals)
cargo test --test network_battle_test

# Performance benchmarks
cargo test --release bench
```

### Code Quality Tools

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Security audit
cargo audit

# Check for outdated dependencies
cargo outdated

# Generate documentation
cargo doc --open
```

## Contributing

### Development Workflow

1. **Fork and Clone**:
   ```bash
   git clone https://github.com/yourusername/aerolithsdb.git
   cd aerolithsdb
   ```

2. **Create Feature Branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Develop and Test**:
   ```bash
   cargo test
   cargo fmt
   cargo clippy
   ```

4. **Commit and Push**:
   ```bash
   git commit -m "feat: implement your feature"
   git push origin feature/your-feature-name
   ```

5. **Create Pull Request** with:

   - Clear description of changes
   - Test coverage for new features
   - Documentation updates
   - Performance impact analysis (if applicable)

### Code Style Guidelines

- Follow Rust naming conventions (`snake_case` for functions, `PascalCase` for types)
- Use `cargo fmt` for consistent formatting
- Write comprehensive unit tests for new features
- Document public APIs with rustdoc comments
- Handle errors explicitly (avoid `unwrap()` in production code)
- Use structured logging with appropriate levels

### Testing Requirements

- Unit tests for all new functionality
- Integration tests for API endpoints
- Performance tests for storage operations
- Documentation tests for code examples

## API Development

### Adding New REST Endpoints

1. **Define Route in `aerolithdb-api/src/rest.rs`**:
   ```rust
   pub fn configure_routes(cfg: &mut web::ServiceConfig) {
       cfg.service(
           web::scope("/api/v1")
               .route("/your-endpoint", web::post().to(your_handler))
       );
   }
   ```

2. **Implement Handler**:
   ```rust
   pub async fn your_handler(
       req: web::Json<YourRequest>,
       storage: web::Data<StorageEngine>,
   ) -> Result<impl Responder, ApiError> {
       let result = storage.your_operation(req.into_inner()).await?;
       Ok(web::Json(result))
   }
   ```

3. **Add Tests**:
   ```rust
   #[tokio::test]
   async fn test_your_endpoint() {
       let app = test::init_service(create_app()).await;
       let req = test::TestRequest::post()
           .uri("/api/v1/your-endpoint")
           .set_json(&your_test_data())
           .to_request();
       let resp = test::call_service(&app, req).await;
       assert!(resp.status().is_success());
   }
   ```

### GraphQL Schema Extension

Edit `aerolithdb-api/src/graphql.rs`:

```rust
#[derive(async_graphql::Object)]
impl Query {
    async fn your_query(&self, ctx: &Context<'_>) -> Result<YourType> {
        let storage = ctx.data::<StorageEngine>()?;
        storage.your_operation().await
    }
}
```

## Storage Engine

### Architecture

The storage engine implements a four-tier hierarchy:

1. **L1 (Memory)**: In-memory cache for hot data
2. **L2 (SSD)**: Fast persistent storage for warm data
3. **L3 (HDD)**: Bulk storage for cold data
4. **L4 (Object Storage)**: Archive storage for historical data

### Adding New Storage Backend

1. **Implement `StorageBackend` trait**:
   ```rust
   #[async_trait]
   impl StorageBackend for YourBackend {
       async fn put(&self, key: &str, value: &[u8]) -> Result<()> {
           // Implementation
       }
       
       async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
           // Implementation
       }
       
       async fn delete(&self, key: &str) -> Result<()> {
           // Implementation
       }
   }
   ```

2. **Register Backend**:
   ```rust
   // In aerolithdb-storage/src/backends.rs
   pub fn create_backend(config: &BackendConfig) -> Box<dyn StorageBackend> {
       match config.backend_type {
           BackendType::YourBackend => Box::new(YourBackend::new(config)),
           // ... other backends
       }
   }
   ```

## Consensus Algorithm

AerolithDB implements a Byzantine Fault Tolerant consensus algorithm based on PBFT (Practical Byzantine Fault Tolerance) with optimizations for document databases.

### Key Components

- **Vector Clocks**: For causality tracking
- **Byzantine Tolerance**: Handles up to (n-1)/3 Byzantine failures
- **Conflict Resolution**: Automatic conflict resolution for concurrent operations
- **Partition Recovery**: Automatic recovery from network partitions

### Extending Consensus Logic

Modify `aerolithdb-consensus/src/engine.rs`:

```rust
impl ConsensusEngine {
    pub async fn propose_operation(&self, operation: Operation) -> Result<ConsensusResult> {
        // Phase 1: Pre-prepare
        let proposal = self.create_proposal(operation).await?;
        self.broadcast_prepare(proposal).await?;
        
        // Phase 2: Prepare
        let prepared = self.collect_prepare_votes(proposal.id).await?;
        if prepared {
            self.broadcast_commit(proposal.id).await?;
        }
        
        // Phase 3: Commit
        let committed = self.collect_commit_votes(proposal.id).await?;
        if committed {
            self.apply_operation(proposal.operation).await?;
        }
        
        Ok(ConsensusResult::Success)
    }
}
```

## Network Protocol

### P2P Communication

AerolithDB uses a custom P2P protocol built on top of:

- **Transport**: TCP with TLS encryption
- **Discovery**: mDNS and DHT-based peer discovery
- **Messaging**: Protocol Buffers for message serialization
- **Failure Detection**: Heartbeat-based failure detection

### Message Types

```rust
pub enum NetworkMessage {
    // Discovery
    PeerDiscovery(PeerInfo),
    PeerAnnouncement(NodeMetadata),
    
    // Consensus
    Propose(Proposal),
    Prepare(PrepareMessage),
    Commit(CommitMessage),
    
    // Data
    DataRequest(DataRequest),
    DataResponse(DataResponse),
    
    // Health
    Heartbeat(HeartbeatMessage),
    HealthCheck(HealthRequest),
}
```

## Performance Optimization

### Profiling

```bash
# CPU profiling
cargo build --release
sudo perf record -g ./target/release/aerolithsdb
sudo perf report

# Memory profiling with Valgrind
cargo build
valgrind --tool=massif ./target/debug/aerolithsdb

# Heap profiling
export MALLOC_CONF="prof:true"
./target/release/aerolithsdb
```

### Benchmarking

```bash
# Storage benchmarks
cargo bench --package aerolithdb-storage

# Query engine benchmarks
cargo bench --package aerolithdb-query

# End-to-end benchmarks
cargo test --release bench_full_stack
```

### Common Performance Tuning

1. **Storage Optimization**:
   ```rust
   // Use appropriate batch sizes
   const OPTIMAL_BATCH_SIZE: usize = 1000;
   
   // Enable compression for cold storage
   storage_config.compression_enabled = true;
   storage_config.compression_algorithm = CompressionAlgorithm::LZ4;
   ```

2. **Query Optimization**:
   ```rust
   // Use indices for frequently queried fields
   query_config.auto_index_threshold = 1000; // queries
   query_config.index_fields = vec!["user_id", "timestamp"];
   ```

3. **Network Optimization**:
   ```rust
   // Adjust batch sizes for network operations
   network_config.batch_size = 100;
   network_config.compression_enabled = true;
   ```

## Security Implementation

### Authentication

AerolithDB implements a zero-trust security model:

```rust
pub struct SecurityContext {
    pub user_id: String,
    pub permissions: Vec<Permission>,
    pub session_token: String,
    pub audit_context: AuditContext,
}

impl SecurityContext {
    pub fn check_permission(&self, operation: &Operation) -> Result<()> {
        if !self.permissions.contains(&operation.required_permission()) {
            return Err(SecurityError::InsufficientPermissions);
        }
        Ok(())
    }
}
```

### Encryption

- **At Rest**: AES-256 encryption for stored data
- **In Transit**: TLS 1.3 for all network communication
- **Key Management**: Integration with external key management systems

### Audit Logging

All operations are logged for security auditing:

```rust
pub struct AuditLog {
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub operation: String,
    pub resource: String,
    pub result: OperationResult,
    pub client_ip: IpAddr,
}
```

## Debugging and Profiling

### Logging Configuration

```rust
// Enable debug logging
RUST_LOG=aerolithdb=debug,aerolithdb_storage=trace

// Structured logging
log_config.format = LogFormat::Json;
log_config.structured = true;
log_config.include_module_path = true;
```

### Common Debugging Scenarios

1. **Storage Issues**:
   ```bash
   # Check storage health
   curl http://localhost:8080/api/v1/storage/health
   
   # Inspect storage statistics
   curl http://localhost:8080/api/v1/storage/stats
   ```

2. **Consensus Problems**:
   ```bash
   # Check consensus state
   curl http://localhost:8080/api/v1/consensus/status
   
   # View recent consensus decisions
   curl http://localhost:8080/api/v1/consensus/history
   ```

3. **Network Issues**:
   ```bash
   # Check peer connectivity
   curl http://localhost:8080/api/v1/network/peers
   
   # View network statistics
   curl http://localhost:8080/api/v1/network/stats
   ```

### Performance Debugging

```rust
// Add performance metrics
use std::time::Instant;

let start = Instant::now();
let result = expensive_operation().await?;
let duration = start.elapsed();

log::info!(
    operation = "expensive_operation",
    duration_ms = duration.as_millis(),
    "Operation completed"
);
```

## Advanced Topics

### Custom Index Types

Implement custom indexing strategies:

```rust
#[async_trait]
pub trait Index: Send + Sync {
    async fn insert(&self, key: &str, document: &Document) -> Result<()>;
    async fn query(&self, query: &IndexQuery) -> Result<Vec<DocumentId>>;
    async fn delete(&self, key: &str, document_id: &DocumentId) -> Result<()>;
}
```

### Plugin Development

Create plugins for extended functionality:

```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    async fn initialize(&self, context: &PluginContext) -> Result<()>;
    async fn handle_event(&self, event: &SystemEvent) -> Result<()>;
}
```

### Custom Storage Backends

Implement new storage backends:

```rust
pub struct S3Backend {
    client: S3Client,
    bucket: String,
}

#[async_trait]
impl StorageBackend for S3Backend {
    async fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(value.to_vec().into())
            .send()
            .await?;
        Ok(())
    }
}
```

## Resources

- [Rust Book](https://doc.rust-lang.org/book/) - Rust programming language guide
- [Tokio Documentation](https://tokio.rs/) - Async runtime documentation  
- [Protocol Buffers](https://developers.google.com/protocol-buffers) - Message serialization
- [Byzantine Fault Tolerance](https://en.wikipedia.org/wiki/Byzantine_fault) - Consensus algorithm background
- [Performance Profiling](https://github.com/flamegraph-rs/flamegraph) - Rust profiling tools

## Getting Help

- **Documentation**: Check the `docs/` directory for specific guides
- **Issues**: Report bugs or request features on GitHub
- **Discussions**: Join community discussions for questions and ideas
- **Code Review**: Submit pull requests for community review

---

This developer guide provides comprehensive information for contributing to AerolithDB. For production deployment, see the [Production Deployment Guide](PRODUCTION_DEPLOYMENT.md).
