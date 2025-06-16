//! Core types and data structures for the aerolithsDB consensus system.
//!
//! This module defines all the fundamental types used throughout the consensus
//! engine, including configurations, proposals, votes, and messages.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::conflict_resolution::ConflictResolution;

/// Configuration for the consensus engine behavior and algorithms.
///
/// This structure defines all the parameters that control how the consensus
/// system operates, including algorithm selection, fault tolerance settings,
/// and performance tuning parameters.
#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    /// The consensus algorithm to use for distributed agreement
    pub algorithm: ConsensusAlgorithm,
    
    /// Maximum fraction of Byzantine (malicious) nodes the system can tolerate
    /// Typically set to 1/3 for Byzantine algorithms (e.g., 0.33)
    pub byzantine_tolerance: f32,
    
    /// Maximum time to wait for consensus on a proposal before timeout
    /// Should account for network latency and processing time
    pub timeout: std::time::Duration,
    
    /// Maximum number of operations to batch in a single consensus round
    /// Higher values improve throughput but increase latency
    pub max_batch_size: usize,
    
    /// Strategy for resolving conflicts between concurrent operations
    pub conflict_resolution: ConflictResolution,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        use std::time::Duration;
        Self {
            algorithm: ConsensusAlgorithm::ByzantinePBFT,
            byzantine_tolerance: 0.33,
            timeout: Duration::from_secs(30),
            max_batch_size: 100,
            conflict_resolution: ConflictResolution::LastWriterWins,
        }
    }
}

/// Available consensus algorithms with different trade-offs.
/// 
/// Each algorithm provides different guarantees regarding:
/// - Fault tolerance (fail-stop vs Byzantine failures)
/// - Performance characteristics (latency, throughput)
/// - Network requirements (partition tolerance)
/// - Implementation complexity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusAlgorithm {
    /// Practical Byzantine Fault Tolerance - tolerates malicious nodes
    /// - Handles up to 1/3 Byzantine failures
    /// - Higher overhead but maximum security
    /// - Best for: Untrusted environments, financial applications
    ByzantinePBFT,
    
    /// Raft consensus algorithm - simpler, handles fail-stop failures only
    /// - Efficient leader-based approach
    /// - Lower overhead, faster in trusted environments  
    /// - Best for: Internal clusters, development environments
    Raft,
    
    /// HoneyBadger BFT - asynchronous Byzantine consensus
    /// - No timing assumptions, true async operation
    /// - Optimal resilience with threshold cryptography
    /// - Best for: Highly adversarial networks, blockchain applications
    HoneyBadgerBFT,
}

/// Unique identifier for peers in the consensus network.
/// 
/// This is typically the node ID or public key hash that uniquely
/// identifies each participant in the consensus protocol.
pub type PeerId = String;

/// Unique identifier for consensus proposals.
/// 
/// Each operation proposed for consensus gets a unique UUID to
/// distinguish it from other concurrent proposals.
pub type ProposalId = Uuid;

/// A proposal for a database operation that requires distributed consensus.
/// 
/// Each proposal represents a single atomic operation that all nodes
/// must agree upon before it can be committed to the database.
/// Proposals include cryptographic signatures to ensure authenticity
/// and prevent tampering.
#[derive(Debug, Clone)]
pub struct Proposal {
    /// Unique identifier for this proposal
    pub id: ProposalId,
    
    /// Consensus round number for ordering and conflict resolution
    pub round: u64,
    
    /// Peer that originated this proposal
    pub proposer: PeerId,
    
    /// The database operation being proposed
    pub operation: Operation,
    
    /// When this proposal was created (for timeout and ordering)
    pub timestamp: DateTime<Utc>,
    
    /// Cryptographic signature from the proposer for authenticity
    pub signature: String,
}

/// Database operations that can be proposed for consensus.
/// 
/// These represent all possible state changes that can be made
/// to the distributed database. Each operation is atomic and
/// either succeeds completely or fails without side effects.
#[derive(Debug, Clone)]
pub enum Operation {
    /// Insert a new document into a collection
    Insert {
        /// Target collection name
        collection: String,
        /// Unique document identifier  
        document_id: String,
        /// Document data as JSON
        data: serde_json::Value,
    },
    
    /// Update an existing document with optimistic concurrency control
    Update {
        /// Target collection name
        collection: String,
        /// Document identifier to update
        document_id: String,
        /// New document data as JSON
        data: serde_json::Value,
        /// Expected version for optimistic locking
        version: u64,
    },
    
    /// Delete an existing document with optimistic concurrency control
    Delete {
        /// Target collection name
        collection: String,
        /// Document identifier to delete
        document_id: String,
        /// Expected version for optimistic locking
        version: u64,
    },
    
    /// Create a new collection with optional schema valiaerolithon
    CreateCollection {
        /// Name of the new collection
        name: String,
        /// Optional JSON schema for document valiaerolithon
        schema: Option<serde_json::Value>,
    },
    
    /// Remove an entire collection and all its documents
    DropCollection {
        /// Name of the collection to remove
        name: String,
    },
}

/// Vote on a proposal from a participating node.
///
/// Each peer in the consensus network can vote on proposals to indicate
/// whether they accept, reject, or abstain from the proposed operation.
#[derive(Debug, Clone)]
pub struct Vote {
    /// The proposal being voted on
    pub proposal_id: ProposalId,
    /// The peer casting this vote
    pub voter: PeerId,
    /// The vote decision (accept/reject/abstain)
    pub decision: VoteDecision,
    /// When this vote was cast
    pub timestamp: DateTime<Utc>,
    /// Cryptographic signature for vote authenticity
    pub signature: String,
}

/// Possible decisions a node can make when voting on a proposal.
#[derive(Debug, Clone)]
pub enum VoteDecision {
    /// Vote to accept the proposal
    Accept,
    /// Vote to reject the proposal
    Reject,
    /// Abstain from voting (neutral)
    Abstain,
}

/// Collection of votes for a specific proposal.
///
/// Tracks all votes received for a proposal and determines when
/// the consensus threshold has been reached.
#[derive(Debug, Clone)]
pub struct VoteCollection {
    /// The proposal these votes are for
    pub proposal_id: ProposalId,
    /// All votes received, indexed by voter peer ID
    pub votes: HashMap<PeerId, Vote>,
    /// Whether the consensus threshold has been reached
    pub threshold_reached: bool,
}

/// A committed entry in the consensus log.
///
/// Represents a proposal that has achieved consensus and been committed
/// to the distributed log. This is the permanent record of agreed-upon
/// operations.
#[derive(Debug, Clone)]
pub struct CommittedEntry {
    /// The proposal that was committed
    pub proposal: Proposal,
    /// The votes that achieved consensus
    pub votes: VoteCollection,
    /// When consensus was reached and the entry was committed
    pub committed_at: DateTime<Utc>,
    /// The consensus round number
    pub consensus_round: u64,
}

/// Messages exchanged between nodes during consensus protocol.
///
/// These messages coordinate the consensus process and ensure all
/// nodes stay synchronized with the distributed state machine.
#[derive(Debug, Clone)]
pub enum ConsensusMessage {
    /// Propose a new operation for consensus
    Propose(Proposal),
    /// Vote on an existing proposal
    Vote(Vote),
    /// Commit a proposal that achieved consensus
    Commit(CommitMessage),
    /// Abort a proposal that failed to achieve consensus
    Abort(AbortMessage),
    /// Heartbeat to maintain cluster connectivity
    Heartbeat(HeartbeatMessage),
    /// Request to change view/leader in the consensus protocol
    ViewChange(ViewChangeMessage),
}

/// Message indicating a proposal has been committed.
#[derive(Debug, Clone)]
pub struct CommitMessage {
    /// The proposal that was committed
    pub proposal_id: ProposalId,
    /// The consensus round number
    pub round: u64,
    /// When the commit occurred
    pub committed_at: DateTime<Utc>,
}

/// Message indicating a proposal has been aborted.
#[derive(Debug, Clone)]
pub struct AbortMessage {
    /// The proposal that was aborted
    pub proposal_id: ProposalId,
    /// The consensus round number
    pub round: u64,
    /// Reason for the abort
    pub reason: String,
}

/// Heartbeat message to maintain cluster connectivity.
#[derive(Debug, Clone)]
pub struct HeartbeatMessage {
    /// The peer sending the heartbeat
    pub peer_id: PeerId,
    /// When the heartbeat was sent
    pub timestamp: DateTime<Utc>,
    /// The last committed round this peer knows about
    pub last_committed_round: u64,
}

/// Message requesting a view change in the consensus protocol.
#[derive(Debug, Clone)]
pub struct ViewChangeMessage {
    /// The new view number being proposed
    pub new_view: u64,
    /// The peer requesting the view change
    pub peer_id: PeerId,
    /// The last committed round this peer knows about
    pub last_committed: u64,
}
