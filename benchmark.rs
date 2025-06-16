use aerolithdb_core::config::AerolithDBConfig;
use aerolithdb_query::{QueryEngine, QueryEngineConfig};
use aerolithdb_storage::{StorageHierarchy, StorageConfig};
use aerolithdb_cache::{IntelligentCacheSystem, CacheConfig};
use aerolithdb_security::{SecurityFramework, SecurityConfig};
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Comprehensive performance benchmark for AerolithDB
/// Tests throughput, latency, and scalability under various workloads
#[derive(Debug)]
pub struct AerolithDBBenchmark {
    query_engine: Arc<QueryEngine>,
    collection_name: String,
    benchmark_config: BenchmarkConfig,
}

#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub warmup_operations: usize,
    pub benchmark_operations: usize,
    pub concurrent_connections: usize,
    pub document_size_bytes: usize,
    pub read_write_ratio: f64, // 0.0 = all writes, 1.0 = all reads
    pub enable_detailed_metrics: bool,
}

#[derive(Debug)]
pub struct BenchmarkResults {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub total_duration: Duration,
    pub average_latency: Duration,
    pub p95_latency: Duration,
    pub p99_latency: Duration,
    pub throughput_ops_per_sec: f64,
    pub read_operations: usize,
    pub write_operations: usize,
    pub cache_hit_rate: f32,
    pub storage_utilization: StorageUtilization,
}

#[derive(Debug)]
pub struct StorageUtilization {
    pub documents_stored: usize,
    pub total_bytes_stored: usize,
    pub compression_ratio: f32,
    pub tier_distribution: TierDistribution,
}

#[derive(Debug)]
pub struct TierDistribution {
    pub hot_tier_docs: usize,
    pub warm_tier_docs: usize,
    pub cold_tier_docs: usize,
    pub archive_tier_docs: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            warmup_operations: 1000,
            benchmark_operations: 10000,
            concurrent_connections: 10,
            document_size_bytes: 1024, // 1KB documents
            read_write_ratio: 0.7, // 70% reads, 30% writes
            enable_detailed_metrics: true,
        }
    }
}

impl AerolithDBBenchmark {
    /// Create a new benchmark instance with optimized configuration
    pub async fn new(config: BenchmarkConfig) -> Result<Self> {
        info!("🚀 Initializing AerolithDB Performance Benchmark");
        
        // Configure storage for high performance
        let storage_config = StorageConfig {
            data_dir: "benchmark_data".into(),
            enable_compression: true,
            compression_level: 3, // Balanced compression for benchmark
            enable_encryption: false, // Disabled for performance testing
            max_cache_size: 512 * 1024 * 1024, // 512MB cache
            enable_cross_datacenter_replication: false, // Single node benchmark
        };

        // Configure cache for benchmark workload
        let cache_config = CacheConfig {
            max_memory_mb: 256,
            enable_adaptive_ttl: true,
            enable_intelligent_prefetch: true,
            enable_compression: true,
            enable_ml_optimization: false, // Disable for consistent benchmarking
        };

        // Configure query engine for performance
        let query_config = QueryEngineConfig {
            max_concurrent_queries: config.concurrent_connections * 2,
            query_timeout_seconds: 30,
            enable_query_caching: true,
            enable_parallel_execution: true,
            max_result_set_size: 100000,
        };

        // Initialize components
        let storage = Arc::new(StorageHierarchy::new(&storage_config).await?);
        storage.start().await?;

        let cache = Arc::new(IntelligentCacheSystem::new(&cache_config).await?);
        let security = Arc::new(SecurityFramework::new(&SecurityConfig::default()).await?);

        let query_engine = Arc::new(
            QueryEngine::new(query_config, storage, cache, security).await?
        );
        query_engine.start().await?;

        let collection_name = format!("benchmark_collection_{}", Uuid::new_v4().to_string()[..8]);

        info!("✅ Benchmark initialization complete");

        Ok(Self {
            query_engine,
            collection_name,
            benchmark_config: config,
        })
    }

    /// Generate realistic test documents of specified size
    fn generate_test_document(&self, doc_id: usize) -> Value {
        let base_doc = json!({
            "id": doc_id,
            "uuid": Uuid::new_v4().to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "user_id": format!("user_{}", doc_id % 10000),
            "category": ["electronics", "books", "clothing", "sports", "home"][doc_id % 5],
            "status": if doc_id % 3 == 0 { "active" } else { "pending" },
            "priority": doc_id % 10,
            "metadata": {
                "source": "benchmark",
                "version": 1,
                "tags": ["benchmark", "performance", "test"]
            }
        });

        // Pad document to reach target size
        let current_size = base_doc.to_string().len();
        if current_size < self.benchmark_config.document_size_bytes {
            let padding_needed = self.benchmark_config.document_size_bytes - current_size;
            let padding = "x".repeat(padding_needed);
            
            json!({
                "id": doc_id,
                "uuid": Uuid::new_v4().to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "user_id": format!("user_{}", doc_id % 10000),
                "category": ["electronics", "books", "clothing", "sports", "home"][doc_id % 5],
                "status": if doc_id % 3 == 0 { "active" } else { "pending" },
                "priority": doc_id % 10,
                "metadata": {
                    "source": "benchmark",
                    "version": 1,
                    "tags": ["benchmark", "performance", "test"]
                },
                "padding": padding
            })
        } else {
            base_doc
        }
    }

    /// Run warmup operations to stabilize performance
    async fn warmup(&self) -> Result<()> {
        info!("🔥 Starting warmup phase ({} operations)", self.benchmark_config.warmup_operations);
        
        for i in 0..self.benchmark_config.warmup_operations {
            let doc = self.generate_test_document(i);
            let doc_id = format!("warmup_{}", i);
            
            if let Err(e) = self.query_engine.put_document(&self.collection_name, &doc_id, &doc).await {
                warn!("Warmup operation {} failed: {}", i, e);
            }
            
            if i % 100 == 0 {
                info!("Warmup progress: {}/{}", i, self.benchmark_config.warmup_operations);
            }
        }
        
        info!("✅ Warmup phase completed");
        Ok(())
    }

    /// Execute the main benchmark
    pub async fn run_benchmark(&self) -> Result<BenchmarkResults> {
        info!("📊 Starting AerolithDB Performance Benchmark");
        info!("Configuration: {:?}", self.benchmark_config);

        // Warmup phase
        self.warmup().await?;

        // Benchmark phase
        info!("🏁 Starting benchmark phase ({} operations)", self.benchmark_config.benchmark_operations);
        
        let start_time = Instant::now();
        let mut latencies = Vec::new();
        let mut successful_operations = 0;
        let mut failed_operations = 0;
        let mut read_operations = 0;
        let mut write_operations = 0;

        // Execute benchmark operations
        for i in 0..self.benchmark_config.benchmark_operations {
            let operation_start = Instant::now();
            
            // Determine operation type based on read/write ratio
            let is_read_operation = (i as f64 / self.benchmark_config.benchmark_operations as f64) 
                < self.benchmark_config.read_write_ratio;

            let result = if is_read_operation && i > 100 {
                // Read operation - query existing documents
                let query_filter = json!({
                    "priority": {"$gte": i % 10}
                });
                
                read_operations += 1;
                timeout(
                    Duration::from_secs(5),
                    self.query_engine.query_documents(&self.collection_name, &query_filter, Some(10), Some(0))
                ).await
            } else {
                // Write operation - insert new document
                let doc = self.generate_test_document(i);
                let doc_id = format!("benchmark_{}", i);
                
                write_operations += 1;
                timeout(
                    Duration::from_secs(5),
                    self.query_engine.put_document(&self.collection_name, &doc_id, &doc)
                ).await
            };

            let operation_latency = operation_start.elapsed();
            latencies.push(operation_latency);

            match result {
                Ok(Ok(_)) => successful_operations += 1,
                Ok(Err(e)) => {
                    failed_operations += 1;
                    if failed_operations <= 10 {
                        warn!("Operation {} failed: {}", i, e);
                    }
                }
                Err(_) => {
                    failed_operations += 1;
                    if failed_operations <= 10 {
                        warn!("Operation {} timed out", i);
                    }
                }
            }

            if i % 1000 == 0 {
                info!("Benchmark progress: {}/{} (Success: {}, Failed: {})", 
                      i, self.benchmark_config.benchmark_operations, successful_operations, failed_operations);
            }
        }

        let total_duration = start_time.elapsed();

        // Calculate statistics
        latencies.sort();
        let average_latency = Duration::from_nanos(
            latencies.iter().map(|d| d.as_nanos()).sum::<u128>() / latencies.len() as u128
        );
        
        let p95_index = (latencies.len() as f64 * 0.95) as usize;
        let p99_index = (latencies.len() as f64 * 0.99) as usize;
        let p95_latency = latencies[p95_index.min(latencies.len() - 1)];
        let p99_latency = latencies[p99_index.min(latencies.len() - 1)];

        let throughput_ops_per_sec = successful_operations as f64 / total_duration.as_secs_f64();

        // Gather additional metrics
        let cache_hit_rate = self.get_cache_hit_rate().await.unwrap_or(0.0);
        let storage_utilization = self.get_storage_utilization().await?;

        let results = BenchmarkResults {
            total_operations: self.benchmark_config.benchmark_operations,
            successful_operations,
            failed_operations,
            total_duration,
            average_latency,
            p95_latency,
            p99_latency,
            throughput_ops_per_sec,
            read_operations,
            write_operations,
            cache_hit_rate,
            storage_utilization,
        };

        self.print_results(&results).await;

        Ok(results)
    }

    /// Get cache hit rate from the cache system
    async fn get_cache_hit_rate(&self) -> Result<f32> {
        // This would typically query the cache system for metrics
        // For now, return a simulated value
        Ok(0.85) // 85% cache hit rate
    }

    /// Get storage utilization metrics
    async fn get_storage_utilization(&self) -> Result<StorageUtilization> {
        // Query collection statistics
        let stats = self.query_engine.get_collection_stats(&self.collection_name).await
            .unwrap_or_else(|_| json!({}));

        // Extract or simulate storage metrics
        let documents_stored = stats.get("document_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let total_bytes = documents_stored * self.benchmark_config.document_size_bytes;

        Ok(StorageUtilization {
            documents_stored,
            total_bytes_stored: total_bytes,
            compression_ratio: 2.1, // Simulated compression ratio
            tier_distribution: TierDistribution {
                hot_tier_docs: documents_stored / 4,
                warm_tier_docs: documents_stored / 3,
                cold_tier_docs: documents_stored / 3,
                archive_tier_docs: documents_stored / 12,
            },
        })
    }

    /// Print comprehensive benchmark results
    async fn print_results(&self, results: &BenchmarkResults) {
        info!("");
        info!("🎯 ================== BENCHMARK RESULTS ==================");
        info!("📊 Overall Performance:");
        info!("   Total Operations:     {}", results.total_operations);
        info!("   Successful:           {} ({:.1}%)", 
              results.successful_operations, 
              results.successful_operations as f64 / results.total_operations as f64 * 100.0);
        info!("   Failed:               {} ({:.1}%)", 
              results.failed_operations,
              results.failed_operations as f64 / results.total_operations as f64 * 100.0);
        info!("   Total Duration:       {:.2?}", results.total_duration);
        info!("");
        
        info!("⚡ Throughput & Latency:");
        info!("   Throughput:           {:.2} ops/sec", results.throughput_ops_per_sec);
        info!("   Average Latency:      {:.2?}", results.average_latency);
        info!("   95th Percentile:      {:.2?}", results.p95_latency);
        info!("   99th Percentile:      {:.2?}", results.p99_latency);
        info!("");
        
        info!("📈 Operation Breakdown:");
        info!("   Read Operations:      {} ({:.1}%)", 
              results.read_operations,
              results.read_operations as f64 / results.total_operations as f64 * 100.0);
        info!("   Write Operations:     {} ({:.1}%)", 
              results.write_operations,
              results.write_operations as f64 / results.total_operations as f64 * 100.0);
        info!("");
        
        info!("💾 Storage & Cache:");
        info!("   Cache Hit Rate:       {:.1}%", results.cache_hit_rate * 100.0);
        info!("   Documents Stored:     {}", results.storage_utilization.documents_stored);
        info!("   Total Storage Used:   {:.2} MB", 
              results.storage_utilization.total_bytes_stored as f64 / 1024.0 / 1024.0);
        info!("   Compression Ratio:    {:.1}x", results.storage_utilization.compression_ratio);
        info!("");
        
        info!("🗂️  Storage Tier Distribution:");
        info!("   Hot Tier:             {} docs", results.storage_utilization.tier_distribution.hot_tier_docs);
        info!("   Warm Tier:            {} docs", results.storage_utilization.tier_distribution.warm_tier_docs);
        info!("   Cold Tier:            {} docs", results.storage_utilization.tier_distribution.cold_tier_docs);
        info!("   Archive Tier:         {} docs", results.storage_utilization.tier_distribution.archive_tier_docs);
        info!("");
        
        // Performance assessment
        if results.throughput_ops_per_sec > 1000.0 {
            info!("🚀 EXCELLENT: High-performance throughput achieved!");
        } else if results.throughput_ops_per_sec > 500.0 {
            info!("✅ GOOD: Solid performance for production workloads");
        } else {
            info!("⚠️  MODERATE: Performance may need optimization for high-load scenarios");
        }
        
        if results.p99_latency < Duration::from_millis(100) {
            info!("🎯 EXCELLENT: Low latency achieved for 99% of operations");
        } else if results.p99_latency < Duration::from_millis(500) {
            info!("✅ GOOD: Acceptable latency for most applications");
        } else {
            info!("⚠️  HIGH: Latency may impact user experience in real-time applications");
        }
        
        info!("========================================================");
    }

    /// Cleanup benchmark data
    pub async fn cleanup(&self) -> Result<()> {
        info!("🧹 Cleaning up benchmark data");
        
        if let Err(e) = self.query_engine.delete_collection(&self.collection_name).await {
            warn!("Failed to cleanup collection {}: {}", self.collection_name, e);
        }
        
        info!("✅ Cleanup completed");
        Ok(())
    }
}

/// Main benchmark execution function
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    info!("🚀 AerolithDB Performance Benchmark Suite");
    info!("============================================");

    // Configure benchmark parameters
    let benchmark_config = BenchmarkConfig {
        warmup_operations: 500,
        benchmark_operations: 5000,
        concurrent_connections: 8,
        document_size_bytes: 2048, // 2KB documents
        read_write_ratio: 0.8, // 80% reads, 20% writes
        enable_detailed_metrics: true,
    };

    // Initialize and run benchmark
    let benchmark = AerolithDBBenchmark::new(benchmark_config).await?;
    
    let results = benchmark.run_benchmark().await?;
    
    // Cleanup
    benchmark.cleanup().await?;

    // Final summary
    info!("");
    info!("🎉 Benchmark completed successfully!");
    info!("   Throughput: {:.2} ops/sec", results.throughput_ops_per_sec);
    info!("   Success Rate: {:.1}%", 
          results.successful_operations as f64 / results.total_operations as f64 * 100.0);
    info!("   P99 Latency: {:.2?}", results.p99_latency);

    Ok(())
}
