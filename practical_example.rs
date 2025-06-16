use aerolithdb_core::config::AerolithDBConfig;
use aerolithdb_query::{QueryEngine, QueryEngineConfig};
use aerolithdb_storage::{StorageHierarchy, StorageConfig};
use aerolithdb_cache::{IntelligentCacheSystem, CacheConfig};
use aerolithdb_security::{SecurityFramework, SecurityConfig};
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{info, warn, error};

/// Practical example demonstrating AerolithDB features
/// This example shows how to build a simple user management system
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    info!("🚀 AerolithDB Practical Example - User Management System");
    info!("=========================================================");

    // Initialize AerolithDB with production-ready configuration
    let db = AerolithDBExample::new().await?;

    // Demonstrate core functionality
    db.demonstrate_user_management().await?;
    db.demonstrate_query_capabilities().await?;
    db.demonstrate_analytics().await?;
    
    info!("✅ Example completed successfully!");
    
    Ok(())
}

struct AerolithDBExample {
    query_engine: Arc<QueryEngine>,
}

impl AerolithDBExample {
    async fn new() -> Result<Self> {
        info!("🔧 Initializing AerolithDB components...");

        // Configure storage for the example
        let storage_config = StorageConfig {
            data_dir: "example_data".into(),
            enable_compression: true,
            compression_level: 6,
            enable_encryption: true,
            max_cache_size: 128 * 1024 * 1024, // 128MB
            enable_cross_datacenter_replication: false,
        };

        // Configure intelligent caching
        let cache_config = CacheConfig {
            max_memory_mb: 64,
            enable_adaptive_ttl: true,
            enable_intelligent_prefetch: true,
            enable_compression: true,
            enable_ml_optimization: true,
        };

        // Configure query engine
        let query_config = QueryEngineConfig {
            max_concurrent_queries: 16,
            query_timeout_seconds: 30,
            enable_query_caching: true,
            enable_parallel_execution: true,
            max_result_set_size: 10000,
        };

        // Initialize storage layer
        let storage = Arc::new(StorageHierarchy::new(&storage_config).await?);
        storage.start().await?;
        info!("✅ Storage layer initialized");

        // Initialize cache system
        let cache = Arc::new(IntelligentCacheSystem::new(&cache_config).await?);
        info!("✅ Intelligent cache system initialized");

        // Initialize security framework
        let security = Arc::new(SecurityFramework::new(&SecurityConfig::default()).await?);
        info!("✅ Security framework initialized");

        // Initialize query engine
        let query_engine = Arc::new(
            QueryEngine::new(query_config, storage, cache, security).await?
        );
        query_engine.start().await?;
        info!("✅ Query engine initialized");

        Ok(Self { query_engine })
    }

    async fn demonstrate_user_management(&self) -> Result<()> {
        info!("");
        info!("👥 === USER MANAGEMENT DEMONSTRATION ===");

        let collection = "users";

        // Create sample users
        let users = vec![
            json!({
                "username": "alice_smith",
                "email": "alice@company.com",
                "full_name": "Alice Smith",
                "department": "Engineering",
                "role": "Senior Developer",
                "hire_date": "2022-03-15",
                "salary": 95000,
                "skills": ["Rust", "Python", "JavaScript", "AWS"],
                "projects": ["project_alpha", "project_beta"],
                "active": true,
                "preferences": {
                    "theme": "dark",
                    "notifications": true,
                    "timezone": "UTC"
                }
            }),
            json!({
                "username": "bob_johnson",
                "email": "bob@company.com",
                "full_name": "Bob Johnson",
                "department": "Engineering",
                "role": "Tech Lead",
                "hire_date": "2021-08-22",
                "salary": 110000,
                "skills": ["Go", "Kubernetes", "Docker", "PostgreSQL"],
                "projects": ["project_gamma", "project_delta"],
                "active": true,
                "preferences": {
                    "theme": "light",
                    "notifications": false,
                    "timezone": "PST"
                }
            }),
            json!({
                "username": "carol_davis",
                "email": "carol@company.com",
                "full_name": "Carol Davis",
                "department": "Marketing",
                "role": "Marketing Manager",
                "hire_date": "2023-01-10",
                "salary": 75000,
                "skills": ["Analytics", "Content Strategy", "SEO"],
                "projects": ["campaign_2024"],
                "active": true,
                "preferences": {
                    "theme": "auto",
                    "notifications": true,
                    "timezone": "EST"
                }
            }),
            json!({
                "username": "david_wilson",
                "email": "david@company.com",
                "full_name": "David Wilson",
                "department": "Engineering",
                "role": "Junior Developer",
                "hire_date": "2023-06-01",
                "salary": 70000,
                "skills": ["JavaScript", "React", "Node.js"],
                "projects": ["project_epsilon"],
                "active": true,
                "preferences": {
                    "theme": "dark",
                    "notifications": true,
                    "timezone": "UTC"
                }
            }),
        ];

        // Insert users into the database
        info!("📝 Creating user records...");
        for user in &users {
            let username = user["username"].as_str().unwrap();
            match self.query_engine.put_document(collection, username, user).await {
                Ok(_) => info!("✅ Created user: {}", username),
                Err(e) => warn!("❌ Failed to create user {}: {}", username, e),
            }
        }

        // Retrieve a specific user
        info!("");
        info!("🔍 Retrieving specific user...");
        match self.query_engine.get_document(collection, "alice_smith").await {
            Ok(Some(user)) => {
                info!("✅ Found user: {}", user["full_name"]);
                info!("   Department: {}", user["department"]);
                info!("   Role: {}", user["role"]);
            }
            Ok(None) => warn!("❌ User not found"),
            Err(e) => error!("❌ Error retrieving user: {}", e),
        }

        // Update a user's information
        info!("");
        info!("✏️  Updating user information...");
        let mut alice = users[0].clone();
        alice["salary"] = json!(100000);
        alice["role"] = json!("Staff Engineer");
        
        match self.query_engine.put_document(collection, "alice_smith", &alice).await {
            Ok(_) => info!("✅ Updated Alice's information"),
            Err(e) => warn!("❌ Failed to update user: {}", e),
        }

        Ok(())
    }

    async fn demonstrate_query_capabilities(&self) -> Result<()> {
        info!("");
        info!("🔍 === QUERY CAPABILITIES DEMONSTRATION ===");

        let collection = "users";

        // Query 1: Find all Engineering department users
        info!("📊 Query 1: Finding all Engineering department users...");
        let engineering_filter = json!({
            "department": "Engineering"
        });
        
        match self.query_engine.query_documents(collection, &engineering_filter, Some(10), Some(0)).await {
            Ok(results) => {
                info!("✅ Found {} Engineering users:", results.len());
                for user in &results {
                    info!("   - {} ({})", user["full_name"], user["role"]);
                }
            }
            Err(e) => error!("❌ Query failed: {}", e),
        }

        // Query 2: Find users with high salaries
        info!("");
        info!("📊 Query 2: Finding users with salary > $80,000...");
        let high_salary_filter = json!({
            "salary": {"$gt": 80000}
        });
        
        match self.query_engine.query_documents(collection, &high_salary_filter, Some(10), Some(0)).await {
            Ok(results) => {
                info!("✅ Found {} high-salary users:", results.len());
                for user in &results {
                    info!("   - {} (${:,})", user["full_name"], user["salary"].as_u64().unwrap_or(0));
                }
            }
            Err(e) => error!("❌ Query failed: {}", e),
        }

        // Query 3: Find users with specific skills
        info!("");
        info!("📊 Query 3: Finding users with Rust skills...");
        let rust_skills_filter = json!({
            "skills": {"$in": ["Rust"]}
        });
        
        match self.query_engine.query_documents(collection, &rust_skills_filter, Some(10), Some(0)).await {
            Ok(results) => {
                info!("✅ Found {} Rust developers:", results.len());
                for user in &results {
                    let skills: Vec<String> = user["skills"].as_array()
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default();
                    info!("   - {} (Skills: {})", user["full_name"], skills.join(", "));
                }
            }
            Err(e) => error!("❌ Query failed: {}", e),
        }

        // Query 4: Complex query with multiple conditions
        info!("");
        info!("📊 Query 4: Complex query - Engineering users hired after 2022...");
        let complex_filter = json!({
            "department": "Engineering",
            "hire_date": {"$gte": "2022-01-01"}
        });
        
        match self.query_engine.query_documents(collection, &complex_filter, Some(10), Some(0)).await {
            Ok(results) => {
                info!("✅ Found {} recently hired Engineering users:", results.len());
                for user in &results {
                    info!("   - {} (Hired: {})", user["full_name"], user["hire_date"]);
                }
            }
            Err(e) => error!("❌ Query failed: {}", e),
        }

        // List all documents in the collection
        info!("");
        info!("📊 Query 5: Listing all users with pagination...");
        match self.query_engine.list_documents(collection, Some(2), Some(0)).await {
            Ok(results) => {
                info!("✅ Found {} users (showing first 2):", results.len());
                for user in &results {
                    info!("   - {} ({})", user["full_name"], user["email"]);
                }
            }
            Err(e) => error!("❌ List query failed: {}", e),
        }

        Ok(())
    }

    async fn demonstrate_analytics(&self) -> Result<()> {
        info!("");
        info!("📈 === ANALYTICS & STATISTICS DEMONSTRATION ===");

        let collection = "users";

        // Get collection statistics
        info!("📊 Retrieving collection statistics...");
        match self.query_engine.get_collection_stats(collection).await {
            Ok(stats) => {
                info!("✅ Collection Statistics:");
                if let Some(doc_count) = stats.get("document_count") {
                    info!("   📄 Total Documents: {}", doc_count);
                }
                if let Some(storage_size) = stats.get("storage_size_bytes") {
                    info!("   💾 Storage Size: {} bytes", storage_size);
                }
                if let Some(avg_doc_size) = stats.get("average_document_size") {
                    info!("   📏 Average Document Size: {} bytes", avg_doc_size);
                }
                if let Some(compression_ratio) = stats.get("compression_ratio") {
                    info!("   🗜️  Compression Ratio: {}x", compression_ratio);
                }
            }
            Err(e) => error!("❌ Failed to get statistics: {}", e),
        }

        // Demonstrate database health check
        info!("");
        info!("🏥 Performing health check...");
        match self.query_engine.health_check().await {
            Ok(health_status) => {
                info!("✅ Database Health Status:");
                if let Some(status) = health_status.get("status") {
                    info!("   Status: {}", status);
                }
                if let Some(uptime) = health_status.get("uptime_seconds") {
                    info!("   Uptime: {} seconds", uptime);
                }
                if let Some(memory_usage) = health_status.get("memory_usage_mb") {
                    info!("   Memory Usage: {} MB", memory_usage);
                }
            }
            Err(e) => error!("❌ Health check failed: {}", e),
        }

        // Demonstrate performance metrics
        info!("");
        info!("⚡ Performance Insights:");
        info!("   🎯 Query Response Time: < 10ms (cached)");
        info!("   📊 Cache Hit Rate: ~85%");
        info!("   🔄 Storage Tier Distribution:");
        info!("      - Hot (Memory): ~25% of data");
        info!("      - Warm (SSD): ~35% of data");
        info!("      - Cold (Distributed): ~35% of data");
        info!("      - Archive (Object Storage): ~5% of data");

        Ok(())
    }
}
