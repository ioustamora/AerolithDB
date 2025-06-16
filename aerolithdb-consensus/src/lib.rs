//! # aerolithsDB Distributed Consensus Engine
//!
//! This module provides the core distributed consensus infrastructure for aerolithsDB,
//! ensuring strong consistency and fault tolerance across the entire cluster.
//! The consensus system is responsible for maintaining agreement among all nodes
//! regarding the order and content of operations, even in the presence of
//! network partitions, node failures, and Byzantine (malicious) actors.
//!
//! ## Architecture Overview
//!
//! The aerolithsDB consensus engine implements a modular architecture that supports
//! multiple consensus algorithms and provides comprehensive fault tolerance mechanisms.
//! It serves as the founaerolithon for all distributed operations in aerolithsDB, ensuring
//! data consistency, operation ordering, and cluster coordination.
//!
//! ### Core Design Principles
//!
//! - **Strong Consistency**: All nodes agree on operation order and system state
//! - **Byzantine Fault Tolerance**: Handles malicious actors and arbitrary failures
//! - **Network Partition Resilience**: Continues operation during network splits
//! - **High Performance**: Optimized for low latency and high throughput
//! - **Adaptive Algorithms**: Automatically adjusts to network conditions and load
//!
//! ## Key Components
//!
//! ### Consensus Algorithms
//! Multiple algorithm implementations providing different trade-offs:
//! - **Byzantine PBFT**: Maximum security for untrusted environments
//! - **Raft**: High performance for trusted network environments
//! - **HoneyBadger BFT**: Asynchronous operation for adversarial networks
//!
//! ### Fault Tolerance Systems
//! Comprehensive protection against various failure modes:
//! - **Byzantine Tolerance**: Detects and isolates malicious nodes
//! - **Network Partition Recovery**: Heals split networks automatically
//! - **Conflict Resolution**: Resolves concurrent update conflicts intelligently
//! - **Vector Clocks**: Maintains causal ordering across distributed events
//!
//! ### Performance Optimization
//! Advanced optimizations for production deployments:
//! - **Batching**: Groups operations for improved throughput
//! - **Pipelining**: Overlaps consensus rounds for reduced latency
//! - **Adaptive Timeouts**: Adjusts to network conditions automatically
//! - **Load Balancing**: Distributes consensus load across cluster nodes
//!
//! ## Consensus Algorithms
//!
//! ### Practical Byzantine Fault Tolerance (PBFT)
//! Handles up to 1/3 Byzantine failures with strong consistency guarantees:
//! - **Security**: Maximum protection against malicious actors
//! - **Performance**: Optimized for high throughput in untrusted environments
//! - **Robustness**: Continues operation even with sophisticated attacks
//! - **Use Cases**: Financial systems, blockchain applications, critical infrastructure
//!
//! ### Raft Consensus Algorithm  
//! Leader-based consensus optimized for trusted network environments:
//! - **Simplicity**: Easier to understand and implement than Byzantine algorithms
//! - **Performance**: Lower overhead and faster convergence in trusted networks
//! - **Fault Tolerance**: Handles fail-stop failures (up to 1/2 node failures)
//! - **Use Cases**: Internal clusters, development environments, trusted deployments
//!
//! ### HoneyBadger BFT
//! Asynchronous Byzantine consensus for adversarial network conditions:
//! - **No Timing Assumptions**: True asynchronous operation without timeouts
//! - **Optimal Resilience**: Handles worst-case Byzantine adversaries
//! - **Threshold Cryptography**: Uses advanced cryptographic techniques
//! - **Use Cases**: Public networks, blockchain applications, untrusted environments
//!
//! ## Fault Tolerance and Recovery
//!
//! ### Byzantine Fault Tolerance
//! Comprehensive protection against malicious nodes and arbitrary failures:
//! - **Malicious Actor Detection**: Identifies nodes exhibiting Byzantine behavior
//! - **Isolation Mechanisms**: Quarantines malicious nodes to prevent damage
//! - **Recovery Procedures**: Restores system integrity after attacks
//! - **Cryptographic Verification**: Ensures message authenticity and integrity
//!
//! ### Network Partition Recovery
//! Automatic healing of network splits and communication failures:
//! - **Partition Detection**: Monitors network connectivity and latency
//! - **Split-Brain Prevention**: Prevents conflicting operations during partitions
//! - **Automatic Reconciliation**: Merges state when partitions are healed
//! - **Consistency Preservation**: Maintains data consistency throughout recovery
//!
//! ### Conflict Resolution
//! Intelligent handling of concurrent operations and data conflicts:
//! - **Vector Clock Ordering**: Maintains causal ordering of distributed events
//! - **Conflict Detection**: Identifies conflicting concurrent operations
//! - **Resolution Strategies**: Multiple strategies for resolving conflicts
//! - **Consistency Guarantees**: Ensures deterministic conflict resolution
//!
//! ## Performance and Scalability
//!
//! ### Optimization Techniques
//! Advanced optimizations for high-performance consensus:
//! - **Operation Batching**: Groups multiple operations for efficient processing
//! - **Pipeline Processing**: Overlaps consensus rounds to reduce latency
//! - **Adaptive Timeouts**: Automatically adjusts to network conditions
//! - **Load Balancing**: Distributes consensus workload across nodes
//!
//! ### Monitoring and Metrics
//! Comprehensive observability for consensus operations:
//! - **Performance Metrics**: Latency, throughput, and resource utilization
//! - **Health Monitoring**: Node status, network connectivity, and fault detection
//! - **Audit Logging**: Complete audit trail of all consensus operations
//! - **Real-time Analytics**: Live monitoring of consensus performance
//!
//! ## Integration and Usage
//!
//! ### Basic Setup
//! ```rust
//! use aerolithsdb_consensus::{ConsensusEngine, ConsensusConfig, ConsensusAlgorithm};
//! 
//! let config = ConsensusConfig {
//!     algorithm: ConsensusAlgorithm::ByzantinePBFT,
//!     byzantine_tolerance: 0.33,
//!     timeout: Duration::from_secs(30),
//!     max_batch_size: 100,
//!     conflict_resolution: ConflictResolution::LastWriterWins,
//! };
//! 
//! let engine = ConsensusEngine::new(&config, security, storage).await?;
//! engine.start().await?;
//! ```
//!
//! ### Operation Proposal
//! ```rust
//! use aerolithsdb_consensus::Operation;
//! 
//! let operation = Operation::Insert {
//!     collection: "users".to_string(),
//!     document_id: "user_123".to_string(),
//!     data: serde_json::json!({"name": "Alice", "age": 30}),
//! };
//! 
//! let proposal_id = engine.propose_operation(operation).await?;
//! ```
//!
//! ## Security Considerations
//!
//! The consensus engine implements multiple layers of security:
//! - **Cryptographic Signatures**: All messages are cryptographically signed
//! - **Identity Verification**: Peer identities are verified before participation
//! - **Message Authentication**: Prevents message tampering and replay attacks
//! - **Byzantine Protection**: Handles malicious nodes and arbitrary failures
//!
//! ## Error Handling and Recovery
//!
//! Robust error handling and recovery mechanisms:
//! - **Graceful Degraaerolithon**: Continues operation with reduced capacity
//! - **Automatic Recovery**: Self-healing from transient failures
//! - **Error Propagation**: Clear error reporting and diagnostics
//! - **State Consistency**: Maintains consistency even during failures

pub mod byzantine_tolerance;
pub mod conflict_resolution;
pub mod engine;
pub mod partition_recovery;
pub mod types;
pub mod vector_clock;

// Re-export main types and functionality for public API
pub use engine::ConsensusEngine;
pub use types::{
    ConsensusConfig, ConsensusAlgorithm, ConsensusMessage, Proposal, Vote, VoteDecision,
    VoteCollection, CommittedEntry, Operation, PeerId, ProposalId, CommitMessage,
    AbortMessage, HeartbeatMessage, ViewChangeMessage,
};

// Re-export supporting systems
pub use byzantine_tolerance::ByzantineFaultTolerance;
pub use conflict_resolution::{ConflictResolution, ConflictResolutionEngine};
pub use partition_recovery::NetworkPartitionRecovery;
pub use vector_clock::VectorClock;
