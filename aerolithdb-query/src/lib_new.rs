//! # aerolithsDB Distributed Query Processing Engine
//!
//! This module implements the sophisticated query processing engine that handles complex
//! queries across aerolithsDB's distributed architecture with cost-based optimization,
//! intelligent execution planning, and seamless integration with storage and cache layers.
//!
//! ## Modular Architecture
//!
//! The query engine is organized into focused modules:
//! - **Engine**: Core query processing orchestration [`engine`]
//! - **Configuration**: Query engine settings and optimization [`config`] 
//! - **Types**: Request/response structures and data types [`types`]
//! - **Processing**: Document filtering, sorting, and pagination [`processing`]
//! - **Statistics**: Performance analytics and metrics collection [`stats`]
//!
//! ## Key Features
//! - **Cost-Based Optimization**: Statistics-driven query plan optimization
//! - **Distributed Execution**: Queries executed across multiple cluster nodes
//! - **Cache Integration**: Intelligent cache utilization for performance
//! - **Index Management**: Automatic index recommenaerolithon and utilization
//! - **Security Integration**: Fine-grained access control and audit logging
//! - **Real-time Analytics**: Support for both OLTP and OLAP workloads
//!
//! ## Example Usage
//! ```rust
//! use aerolithsdb_query::{QueryEngine, QueryConfig, QueryRequest};
//! use serde_json::json;
//! 
//! // Initialize query engine
//! let config = QueryConfig::default();
//! let engine = QueryEngine::new(config, storage, cache, security).await?;
//! 
//! // Execute a query
//! let query = QueryRequest::with_filter(json!({"status": "active"}))
//!     .with_sort(json!({"created_at": -1}))
//!     .with_pagination(50, 0);
//! 
//! let results = engine.query_documents("users", &query).await?;
//! println!("Found {} documents", results.total);
//! ```

// Module declarations
pub mod config;
pub mod types;
pub mod processing; 
pub mod stats;
pub mod engine;

// Re-export main types for convenience
pub use config::{QueryConfig, OptimizerConfig};
pub use types::{QueryRequest, QueryResult};
pub use engine::QueryEngine;
pub use processing::{DocumentFilter, DocumentSorter, DocumentPaginator};
pub use stats::QueryStats;

// External dependencies used by the query engine
use anyhow::Result;
use log::debug;
use std::sync::Arc;

// Crate dependencies
use aerolithsdb_storage::StorageHierarchy;
use aerolithsdb_cache::IntelligentCacheSystem;  
use aerolithsdb_security::SecurityFramework;
