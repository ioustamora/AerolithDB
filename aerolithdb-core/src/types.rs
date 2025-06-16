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
#[derive(Debug, Clone)]
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
    }    pub fn concurrent(&self, other: &VectorClock<T>) -> bool {
        // Two clocks are concurrent if neither happens before the other AND they are not equal
        if self.clocks == other.clocks {
            return false; // Equal clocks are not concurrent
        }
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

impl<T> serde::Serialize for VectorClock<T>
where
    T: Clone + Eq + std::hash::Hash + serde::Serialize + serde::de::DeserializeOwned,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.clocks.serialize(serializer)
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid;

    #[test]
    fn test_vector_clock_new() {
        let clock: VectorClock<String> = VectorClock::new();
        assert!(clock.clocks.is_empty());
    }

    #[test]
    fn test_vector_clock_increment() {
        let mut clock = VectorClock::new();
        
        clock.increment("node1".to_string());
        assert_eq!(clock.get(&"node1".to_string()), 1);
        
        clock.increment("node1".to_string());
        assert_eq!(clock.get(&"node1".to_string()), 2);
        
        clock.increment("node2".to_string());
        assert_eq!(clock.get(&"node2".to_string()), 1);
        assert_eq!(clock.get(&"node1".to_string()), 2);
    }

    #[test]
    fn test_vector_clock_get_nonexistent() {
        let clock: VectorClock<String> = VectorClock::new();
        assert_eq!(clock.get(&"nonexistent".to_string()), 0);
    }

    #[test]
    fn test_vector_clock_merge() {
        let mut clock1 = VectorClock::new();
        clock1.increment("node1".to_string());
        clock1.increment("node1".to_string());
        clock1.increment("node2".to_string());

        let mut clock2 = VectorClock::new();
        clock2.increment("node1".to_string());
        clock2.increment("node2".to_string());
        clock2.increment("node2".to_string());
        clock2.increment("node3".to_string());

        clock1.merge(&clock2);

        assert_eq!(clock1.get(&"node1".to_string()), 2); // max(2, 1) = 2
        assert_eq!(clock1.get(&"node2".to_string()), 2); // max(1, 2) = 2
        assert_eq!(clock1.get(&"node3".to_string()), 1); // max(0, 1) = 1
    }

    #[test]
    fn test_vector_clock_happens_before() {
        let mut clock1 = VectorClock::new();
        clock1.increment("node1".to_string());
        
        let mut clock2 = VectorClock::new();
        clock2.increment("node1".to_string());
        clock2.increment("node1".to_string());

        assert!(clock1.happens_before(&clock2));
        assert!(!clock2.happens_before(&clock1));
    }

    #[test]
    fn test_vector_clock_happens_before_with_multiple_nodes() {
        let mut clock1 = VectorClock::new();
        clock1.increment("node1".to_string());
        clock1.increment("node2".to_string());
        
        let mut clock2 = VectorClock::new();
        clock2.increment("node1".to_string());
        clock2.increment("node1".to_string());
        clock2.increment("node2".to_string());
        clock2.increment("node3".to_string());

        assert!(clock1.happens_before(&clock2));
        assert!(!clock2.happens_before(&clock1));
    }

    #[test]
    fn test_vector_clock_concurrent() {
        let mut clock1 = VectorClock::new();
        clock1.increment("node1".to_string());
        clock1.increment("node1".to_string());
        
        let mut clock2 = VectorClock::new();
        clock2.increment("node2".to_string());
        clock2.increment("node2".to_string());

        assert!(clock1.concurrent(&clock2));
        assert!(clock2.concurrent(&clock1));
    }

    #[test]
    fn test_vector_clock_not_concurrent_when_ordered() {
        let mut clock1 = VectorClock::new();
        clock1.increment("node1".to_string());
        
        let mut clock2 = VectorClock::new();
        clock2.increment("node1".to_string());
        clock2.increment("node1".to_string());

        assert!(!clock1.concurrent(&clock2));
        assert!(!clock2.concurrent(&clock1));
    }

    #[test]
    fn test_vector_clock_equal_not_concurrent() {
        let mut clock1 = VectorClock::new();
        clock1.increment("node1".to_string());
        clock1.increment("node2".to_string());
        
        let mut clock2 = VectorClock::new();
        clock2.increment("node1".to_string());
        clock2.increment("node2".to_string());

        assert!(!clock1.concurrent(&clock2));
        assert!(!clock2.concurrent(&clock1));
        assert!(!clock1.happens_before(&clock2));
        assert!(!clock2.happens_before(&clock1));
    }    #[test]
    fn test_document_creation() {
        let data = json!({"name": "test", "value": 42});
        let owner = NodeId(uuid::Uuid::new_v4());

        let doc = Document::new(
            DocumentId("test_doc".to_string()),
            CollectionId("test_collection".to_string()),
            data.clone(),
            owner.clone(),
        );

        assert_eq!(doc.id.0, "test_doc");
        assert_eq!(doc.collection.0, "test_collection");
        assert_eq!(doc.data, data);
        assert_eq!(doc.metadata.access_control.owner, owner);
        assert_eq!(doc.metadata.version, 1);
        assert!(!doc.is_encrypted());
    }    #[test]
    fn test_document_update() {
        let initial_data = json!({"name": "test", "value": 42});
        let updated_data = json!({"name": "updated", "value": 100});
        let owner = NodeId(uuid::Uuid::new_v4());
        let node = NodeId(uuid::Uuid::new_v4());

        let mut doc = Document::new(
            DocumentId("test_doc".to_string()),
            CollectionId("test_collection".to_string()),
            initial_data,
            owner,
        );

        let initial_timestamp = doc.metadata.updated_at;
        let initial_checksum = doc.metadata.checksum.clone();

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        doc.update(updated_data.clone(), node.clone());

        assert_eq!(doc.data, updated_data);
        assert_eq!(doc.metadata.version, 2);
        assert_eq!(doc.metadata.vector_clock.get(&node), 1);
        assert!(doc.metadata.updated_at > initial_timestamp);
        assert_ne!(doc.metadata.checksum, initial_checksum);
    }

    #[test]
    fn test_document_id_display() {
        let doc_id = DocumentId("test_document_123".to_string());
        assert_eq!(format!("{}", doc_id), "test_document_123");
    }

    #[test]
    fn test_collection_id_display() {
        let col_id = CollectionId("users".to_string());
        assert_eq!(format!("{}", col_id), "users");
    }

    #[test]
    fn test_document_id_from_string() {
        let doc_id: DocumentId = "test_doc".to_string().into();
        assert_eq!(doc_id.0, "test_doc");

        let doc_id: DocumentId = "test_doc".into();
        assert_eq!(doc_id.0, "test_doc");
    }

    #[test]
    fn test_collection_id_from_string() {
        let col_id: CollectionId = "test_collection".to_string().into();
        assert_eq!(col_id.0, "test_collection");

        let col_id: CollectionId = "test_collection".into();
        assert_eq!(col_id.0, "test_collection");
    }

    #[test]
    fn test_vector_clock_serialization() {
        let mut clock = VectorClock::new();
        clock.increment("node1".to_string());
        clock.increment("node2".to_string());
        clock.increment("node1".to_string());

        // Test serialization
        let serialized = serde_json::to_string(&clock).unwrap();
        
        // Test deserialization
        let deserialized: VectorClock<String> = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.get(&"node1".to_string()), 2);
        assert_eq!(deserialized.get(&"node2".to_string()), 1);
    }

    #[test]
    fn test_operation_result() {
        let success_result = OperationResult {
            success: true,
            data: Some("test data".to_string()),
            error: None,
            operation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        };

        assert!(success_result.success);
        assert!(success_result.data.is_some());
        assert!(success_result.error.is_none());

        let error_result: OperationResult<String> = OperationResult {
            success: false,
            data: None,
            error: Some("Operation failed".to_string()),
            operation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        };

        assert!(!error_result.success);
        assert!(error_result.data.is_none());
        assert!(error_result.error.is_some());
    }

    #[test]
    fn test_query_options_defaults() {
        let options = QueryOptions {
            limit: None,
            offset: None,
            sort: None,
            projection: None,
        };

        assert!(options.limit.is_none());
        assert!(options.offset.is_none());
        assert!(options.sort.is_none());
        assert!(options.projection.is_none());
    }

    #[test]
    fn test_sort_field() {
        let sort_field = SortField {
            field: "age".to_string(),
            direction: SortDirection::Ascending,
        };

        assert_eq!(sort_field.field, "age");
        matches!(sort_field.direction, SortDirection::Ascending);

        let sort_field_desc = SortField {
            field: "name".to_string(),
            direction: SortDirection::Descending,
        };

        assert_eq!(sort_field_desc.field, "name");
        matches!(sort_field_desc.direction, SortDirection::Descending);
    }
}
