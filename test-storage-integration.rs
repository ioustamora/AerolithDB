// Storage integration demonstration
// This demonstrates the production storage integration capabilities
// Run with: `cargo run --bin test-storage-integration`

use aerolithdb_storage::{StorageHierarchy, StorageConfig, ShardingStrategy, CompressionConfig, CompressionAlgorithm};
use aerolithdb_query::{QueryEngine, QueryConfig, OptimizerConfig, QueryRequest};
use aerolithdb_cache::IntelligentCacheSystem;
use aerolithdb_security::SecurityFramework;
use std::sync::Arc;
use std::path::PathBuf;
use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting storage integration test");    // Create configurations
    let storage_config = StorageConfig {
        sharding_strategy: ShardingStrategy::ConsistentHash,
        replication_factor: 2,        compression: CompressionConfig {
            algorithm: CompressionAlgorithm::None,
            level: 1,
            adaptive: false,
        },
        encryption_at_rest: false,
        data_dir: PathBuf::from("./test_data"),
        max_storage_size: Some(1024 * 1024 * 1024), // 1GB
        datacenter_replication: None,
    };

    let query_config = QueryConfig {
        optimizer: OptimizerConfig {
            cost_based: true,
            statistics_enabled: true,
            max_optimization_time: std::time::Duration::from_millis(100),
        },
        execution_timeout: std::time::Duration::from_secs(30),
        max_concurrent_queries: 100,
        index_advisor: true,
    };

    // Initialize storage
    info!("Initializing storage hierarchy");
    let storage = match StorageHierarchy::new(&storage_config).await {
        Ok(storage) => Arc::new(storage),
        Err(e) => {
            error!("Failed to initialize storage: {}", e);
            return Err(e);
        }
    };    // Start storage
    if let Err(e) = storage.start().await {
        error!("Failed to start storage: {}", e);
        return Err(e);
    }

    // Initialize cache and security systems
    let cache = Arc::new(IntelligentCacheSystem::new(&Default::default()).await?);
    let security = Arc::new(SecurityFramework::new(&Default::default()).await?);    // Initialize query engine
    info!("Initializing query engine");
    let query_engine = QueryEngine::new(query_config, storage.clone(), cache, security).await?;
    query_engine.start().await?;

    // Test document operations
    info!("Testing document operations");

    // 1. Store a document
    let test_collection = "test_collection";
    let test_doc_id = "doc_1";
    let test_data = serde_json::json!({
        "id": test_doc_id,
        "name": "Test Document",
        "value": 42,
        "tags": ["production", "demo"],
        "metadata": {
            "created_by": "test_user",
            "timestamp": "2025-06-13T10:00:00Z"
        }
    });

    info!("Storing document: {}", test_doc_id);
    match query_engine.store_document(test_collection, test_doc_id, &test_data).await {
        Ok(_) => info!("✅ Document stored successfully"),
        Err(e) => {
            error!("❌ Failed to store document: {}", e);
            return Err(e);
        }
    }    // 2. Retrieve the document
    info!("Retrieving document: {}", test_doc_id);
    match query_engine.get_document(test_collection, test_doc_id).await {
        Ok(doc) => {
            info!("✅ Document retrieved successfully: {}", serde_json::to_string_pretty(&doc)?);
        }
        Err(e) => {
            error!("❌ Failed to retrieve document: {}", e);
            return Err(e);
        }
    }

    // 3. Store additional documents
    for i in 2..=5 {
        let doc_id = format!("doc_{}", i);
        let doc_data = serde_json::json!({
            "id": &doc_id,
            "name": format!("Test Document {}", i),
            "value": i * 10,
            "tags": ["test", "batch"],
            "priority": if i % 2 == 0 { "high" } else { "low" }
        });

        info!("Storing document: {}", doc_id);
        query_engine.store_document(test_collection, &doc_id, &doc_data).await?;
    }

    // 4. List documents
    info!("Listing documents in collection: {}", test_collection);
    match query_engine.list_documents(test_collection, Some(10), None).await {
        Ok(result) => {
            info!("✅ Found {} documents:", result.total);
            for (i, doc) in result.documents.iter().enumerate() {
                info!("  {}. {}", i + 1, serde_json::to_string(doc)?);
            }
        }
        Err(e) => {
            error!("❌ Failed to list documents: {}", e);
            return Err(e);
        }
    }

    // 5. Query documents with filter
    info!("Querying documents with filter");
    let query_request = QueryRequest {
        filter: Some(serde_json::json!({"priority": "high"})),
        limit: Some(5),
        offset: None,
        sort: Some(serde_json::json!({"value": 1})), // ascending
    };

    match query_engine.query_documents(test_collection, &query_request).await {
        Ok(result) => {
            info!("✅ Query returned {} documents:", result.documents.len());
            for doc in &result.documents {
                info!("  - {}", serde_json::to_string(doc)?);
            }
        }
        Err(e) => {
            error!("❌ Failed to query documents: {}", e);
            return Err(e);
        }
    }

    // 6. Get statistics
    info!("Getting database statistics");
    match query_engine.get_stats().await {
        Ok(stats) => {
            info!("✅ Database statistics:");
            info!("{}", serde_json::to_string_pretty(&stats)?);
        }
        Err(e) => {
            error!("❌ Failed to get statistics: {}", e);
            return Err(e);
        }
    }

    // 7. Delete a document
    let delete_doc_id = "doc_3";
    info!("Deleting document: {}", delete_doc_id);
    match query_engine.delete_document(test_collection, delete_doc_id).await {        Ok(_) => {
            info!("✅ Document deleted successfully");
        }
        Err(e) => {
            error!("❌ Failed to delete document: {}", e);
            return Err(e);
        }
    }

    // 8. Verify deletion
    info!("Verifying document deletion");    match query_engine.get_document(test_collection, delete_doc_id).await {
        Ok(_doc) => {
            error!("❌ Document should have been deleted but was found");
            return Err(anyhow::anyhow!("Document deletion verification failed"));
        }
        Err(_) => {
            info!("✅ Document deletion verified - document not found");
        }
    }

    // Clean up
    info!("Cleaning up");
    query_engine.stop().await?;
    storage.stop().await?;

    info!("🎉 Storage integration test completed successfully!");
    Ok(())
}
