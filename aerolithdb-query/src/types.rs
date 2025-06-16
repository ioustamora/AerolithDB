//! # Query Request and Response Types
//!
//! Data structures for query requests, responses, and intermediate results.
//! Provides type-safe interfaces for all query operations in aerolithsDB.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Comprehensive query request structure supporting complex document filtering and sorting.
///
/// This structure represents a complete query request with all necessary parameters
/// for document retrieval, filtering, sorting, and pagination. It provides a
/// flexible interface that supports both simple lookups and complex analytical queries.
///
/// ## Query Capabilities
/// - **Flexible Filtering**: MongoDB-style query operators for complex conditions
/// - **Multi-Field Sorting**: Sort by multiple fields with ascending/descending order
/// - **Efficient Pagination**: Offset/limit support for large result sets
/// - **Index Utilization**: Automatic index selection for optimal performance
///
/// ## Query Filter Examples
/// ```json
/// // Simple equality filter
/// {"status": "active"}
/// 
/// // Range and comparison operators
/// {"age": {"$gte": 18, "$lt": 65}, "score": {"$gt": 75}}
/// 
/// // Array and text operations
/// {"tags": {"$in": ["important", "urgent"]}, "title": {"$regex": "^Project"}}
/// 
/// // Complex boolean logic
/// {"$and": [{"status": "active"}, {"$or": [{"type": "premium"}, {"score": {"$gte": 90}}]}]}
/// ```
///
/// ## Sorting Examples
/// ```json
/// // Single field sort
/// {"created_at": -1}
/// 
/// // Multi-field sorting
/// {"priority": -1, "created_at": 1, "title": 1}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    /// MongoDB-style filter criteria for document selection
    /// 
    /// Supports complex query operators including:
    /// - Equality: `{"field": "value"}`
    /// - Comparison: `{"field": {"$gt": 10, "$lt": 100}}`
    /// - Array operations: `{"field": {"$in": [1, 2, 3]}}`
    /// - Text matching: `{"field": {"$regex": "pattern"}}`
    /// - Boolean logic: `{"$and": [...], "$or": [...], "$not": {...}}`
    pub filter: Option<serde_json::Value>,
    
    /// Sort specification with field names and direction (1 for asc, -1 for desc)
    pub sort: Option<serde_json::Value>,
    
    /// Maximum number of documents to return (0 means no limit)
    pub limit: Option<usize>,
    
    /// Number of documents to skip for pagination
    pub offset: Option<usize>,
}

/// Comprehensive query result containing documents and execution metadata.
///
/// Provides complete information about query execution including performance
/// metrics, cache utilization, and result metadata for debugging and optimization.
///
/// ## Result Metadata
/// - **Performance Metrics**: Execution time and resource utilization
/// - **Cache Information**: Whether results came from cache for performance analysis
/// - **Result Statistics**: Total document count for pagination support
/// - **Execution Context**: Additional metadata for debugging and optimization
///
/// ## Example Usage
/// ```rust
/// let result = engine.query_documents("users", &query).await?;
/// 
/// println!("Found {} documents in {:?}", result.total, result.execution_time);
/// if result.from_cache {
///     println!("Results served from cache for optimal performance");
/// }
/// 
/// for doc in result.documents {
///     println!("Document: {}", serde_json::to_string_pretty(&doc)?);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Array of matching documents with full content and metadata
    pub documents: Vec<serde_json::Value>,
    
    /// Total number of matching documents (may exceed returned documents due to limit)
    pub total: usize,
    
    /// Total time spent executing the query including optimization and retrieval
    pub execution_time: Duration,
    
    /// Indicates whether the result was served from cache for performance tracking
    pub from_cache: bool,
}

impl QueryRequest {
    /// Create a new empty query request with default settings.
    pub fn new() -> Self {
        Self {
            filter: None,
            sort: None,
            limit: None,
            offset: None,
        }
    }

    /// Create a query request with a filter condition.
    pub fn with_filter(filter: serde_json::Value) -> Self {
        Self {
            filter: Some(filter),
            sort: None,
            limit: None,
            offset: None,
        }
    }

    /// Add sorting to the query request.
    pub fn with_sort(mut self, sort: serde_json::Value) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Add pagination to the query request.
    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

impl Default for QueryRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryResult {
    /// Create a new empty query result.
    pub fn empty(execution_time: Duration) -> Self {
        Self {
            documents: vec![],
            total: 0,
            execution_time,
            from_cache: false,
        }
    }

    /// Check if the query result is empty.
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Get the number of returned documents.
    pub fn count(&self) -> usize {
        self.documents.len()
    }
}
