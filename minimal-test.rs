// Simple minimal test to check storage initialization
// This can be run with: `cargo run --bin minimal-test`

use aerolithdb_storage::{StorageHierarchy, StorageConfig, ShardingStrategy, CompressionConfig, CompressionAlgorithm};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {    println!("Starting minimal storage test");
      // Create minimal configuration
    let storage_config = StorageConfig {
        sharding_strategy: ShardingStrategy::ConsistentHash,
        replication_factor: 1,
        compression: CompressionConfig {
            algorithm: CompressionAlgorithm::None,
            level: 1,
            adaptive: false,
        },
        encryption_at_rest: false,
        data_dir: PathBuf::from("./minimal_test_data"),
        max_storage_size: Some(1024 * 1024), // 1MB
        datacenter_replication: None,
    };println!("Creating storage hierarchy...");
    
    // Create each component step by step to isolate issues
    println!("Step 1: Creating data directory...");
    std::fs::create_dir_all(&storage_config.data_dir)?;
    println!("Step 1 completed.");
    
    println!("Step 2: Calling StorageHierarchy::new...");
    let storage = StorageHierarchy::new(&storage_config).await?;
    println!("Step 2 completed - Storage hierarchy created successfully!");

    println!("Starting storage...");
    storage.start().await?;
    println!("Storage started successfully!");

    println!("✅ Minimal test completed successfully");
    Ok(())
}
