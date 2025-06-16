use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Common types used throughout aerolithsDB
pub use crate::node::NodeId;

/// Document identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(pub String);

/// Collection identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CollectionId(pub String);

/// Transaction identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(pub Uuid);

/// Peer identifier for consensus
pub type PeerId = NodeId;

/// Vector clock for distributed consensus
#[derive(Debug, Clone, Serialize)]
pub struct VectorClock<T>
where
    T: Clone + Eq + std::hash::Hash + serde::Serialize + serde::de::DeserializeOwned,
{
    clocks: HashMap<T, u64>,
}

/// Document with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub collection: CollectionId,
    pub data: serde_json::Value,
    pub metadata: DocumentMetadata,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u64,
    pub vector_clock: VectorClock<PeerId>,
    pub encryption_info: Option<EncryptionInfo>,
    pub access_control: AccessControl,
    pub size: usize,
    pub checksum: String,
}

/// Encryption information for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionInfo {
    pub algorithm: String,
    pub key_id: String,
    pub nonce: String,
    pub encrypted: bool,
}

/// Access control information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControl {
    pub owner: PeerId,
    pub permissions: HashMap<PeerId, Vec<Permission>>,
    pub public_read: bool,
}

/// Permission types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Write,
    Delete,
    Admin,
}

/// Query filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilter {
    pub conditions: Vec<Condition>,
    pub operator: LogicalOperator,
}

/// Query condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub field: String,
    pub operator: ComparisonOperator,
    pub value: serde_json::Value,
}

/// Logical operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// Comparison operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    In,
    NotIn,
    Contains,
    StartsWith,
    EndsWith,
}

/// Query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub documents: Vec<Document>,
    pub total_count: usize,
    pub execution_time: std::time::Duration,
    pub query_id: Uuid,
}

/// Operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub operation_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    Heartbeat(HeartbeatMessage),
    Consensus(ConsensusMessage),
    Replication(ReplicationMessage),
    Query(QueryMessage),
    Response(ResponseMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub node_id: NodeId,
    pub timestamp: DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusMessage {
    pub proposal_id: Uuid,
    pub round: u64,
    pub message_type: ConsensusMessageType,
    pub payload: Vec<u8>,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusMessageType {
    Propose,
    Vote,
    Commit,
    Abort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationMessage {
    pub collection: CollectionId,
    pub document: Document,
    pub operation: ReplicationOperation,
    pub vector_clock: VectorClock<PeerId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicationOperation {
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMessage {
    pub query_id: Uuid,
    pub collection: CollectionId,
    pub filter: QueryFilter,
    pub options: QueryOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryOptions {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort: Option<Vec<SortField>>,
    pub projection: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortField {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub request_id: Uuid,
    pub success: bool,
    pub payload: Vec<u8>,
    pub error: Option<String>,
}

// Implementation blocks
impl<T> VectorClock<T>
where
    T: Clone + Eq + std::hash::Hash + serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    pub fn increment(&mut self, node: T) {
        let current = self.clocks.get(&node).unwrap_or(&0);
        self.clocks.insert(node, current + 1);
    }

    pub fn get(&self, node: &T) -> u64 {
        self.clocks.get(node).copied().unwrap_or(0)
    }

    pub fn merge(&mut self, other: &VectorClock<T>) {
        for (node, clock) in &other.clocks {
            let current = self.clocks.get(node).unwrap_or(&0);
            self.clocks.insert(node.clone(), (*current).max(*clock));
        }
    }

    pub fn happens_before(&self, other: &VectorClock<T>) -> bool {
        let mut strictly_less = false;
        
        for (node, our_clock) in &self.clocks {
            let their_clock = other.clocks.get(node).unwrap_or(&0);
            if our_clock > their_clock {
                return false;
            }
            if our_clock < their_clock {
                strictly_less = true;
            }
        }

        for (node, their_clock) in &other.clocks {
            if !self.clocks.contains_key(node) && *their_clock > 0 {
                strictly_less = true;
            }
        }

        strictly_less
    }

    pub fn concurrent(&self, other: &VectorClock<T>) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
}

impl Document {
    pub fn new(
        id: DocumentId,
        collection: CollectionId,
        data: serde_json::Value,
        owner: PeerId,
    ) -> Self {
        let now = Utc::now();
        let size = data.to_string().len();
        let checksum = blake3::hash(data.to_string().as_bytes()).to_hex().to_string();

        Self {
            id,
            collection,
            data,
            metadata: DocumentMetadata {
                created_at: now,
                updated_at: now,
                version: 1,
                vector_clock: VectorClock::new(),
                encryption_info: None,
                access_control: AccessControl {
                    owner,
                    permissions: HashMap::new(),
                    public_read: false,
                },
                size,
                checksum,
            },
        }
    }

    pub fn update(&mut self, data: serde_json::Value, node: PeerId) {
        self.data = data;
        self.metadata.updated_at = Utc::now();
        self.metadata.version += 1;
        self.metadata.vector_clock.increment(node);
        self.metadata.size = self.data.to_string().len();
        self.metadata.checksum = blake3::hash(self.data.to_string().as_bytes()).to_hex().to_string();
    }

    pub fn is_encrypted(&self) -> bool {
        self.metadata.encryption_info
            .as_ref()
            .map(|info| info.encrypted)
            .unwrap_or(false)
    }
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for CollectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for DocumentId {
    fn from(s: String) -> Self {
        DocumentId(s)
    }
}

impl From<String> for CollectionId {
    fn from(s: String) -> Self {
        CollectionId(s)
    }
}

impl From<&str> for DocumentId {
    fn from(s: &str) -> Self {
        DocumentId(s.to_string())
    }
}

impl From<&str> for CollectionId {
    fn from(s: &str) -> Self {
        CollectionId(s.to_string())
    }
}

impl<'de, T> serde::Deserialize<'de> for VectorClock<T>
where
    T: Clone + Eq + std::hash::Hash + serde::Serialize + serde::de::DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let clocks = HashMap::<T, u64>::deserialize(deserializer)?;
        Ok(VectorClock { clocks })
    }
}
