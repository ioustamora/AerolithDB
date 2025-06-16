//! # Query Statistics and Analytics
//!
//! Statistics collection and analysis for query performance optimization.
//! Provides comprehensive metrics for query execution, cache utilization, and system health.

use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;
use chrono::Utc;

use aerolithdb_storage::StorageHierarchy;

/// Statistics collector for query performance analysis and optimization.
///
/// Collects and analyzes query execution metrics to provide insights into
/// system performance, cache effectiveness, and optimization opportunities.
pub struct QueryStats;

impl QueryStats {
    /// Generate comprehensive database statistics including storage and query metrics.
    ///
    /// Collects statistics from all subsystems and formats them into a comprehensive
    /// report suitable for monitoring, debugging, and performance analysis.
    ///
    /// # Arguments
    /// * `storage` - Reference to the storage hierarchy for storage statistics
    ///
    /// # Returns
    /// * `Result<Value>` - JSON object containing comprehensive system statistics
    ///
    /// # Statistics Categories
    /// - **Query Engine**: Optimizer status and query processing metrics
    /// - **Storage System**: Document counts, sizes, and tier utilization
    /// - **Performance**: Cache hit rates and compression ratios
    /// - **System Health**: Timestamps and operational status
    ///
    /// # Example Output
    /// ```json
    /// {
    ///   "query_engine": {
    ///     "optimizer_enabled": true,
    ///     "cost_based_optimization": true,
    ///     "max_concurrent_queries": 100,
    ///     "execution_timeout": "300s"
    ///   },
    ///   "storage": {
    ///     "total_documents": 1000000,
    ///     "total_size_bytes": 2147483648,
    ///     "hot_tier_size": 536870912,
    ///     "cache_hit_rate": 0.85,
    ///     "average_compression_ratio": 0.65
    ///   },
    ///   "metadata": {
    ///     "timestamp": "2024-01-15T10:30:00Z",
    ///     "uptime": "running"
    ///   }
    /// }
    /// ```
    pub async fn collect_database_stats(
        storage: &Arc<StorageHierarchy>,
        optimizer_enabled: bool,
        cost_based: bool,
        max_concurrent: usize,        timeout_secs: u64,
    ) -> Result<Value> {// Collect storage statistics with error handling
        let storage_stats = match storage.get_storage_stats().await {
            Ok(stats) => {
                json!({
                    "total_documents": stats.total_documents,
                    "total_size_bytes": stats.total_size,
                    "hot_tier_size": stats.hot_tier_size,
                    "warm_tier_size": stats.warm_tier_size,
                    "cold_tier_size": stats.cold_tier_size,
                    "archive_tier_size": stats.archive_tier_size,
                    "cache_hit_rate": stats.cache_hit_rate,
                    "average_compression_ratio": stats.compression_ratio
                })
            }
            Err(e) => {
                json!({
                    "error": "Failed to retrieve storage statistics",
                    "error_details": e.to_string()
                })
            }
        };

        // Construct comprehensive statistics report
        let stats = json!({
            "query_engine": {
                "optimizer_enabled": optimizer_enabled,
                "cost_based_optimization": cost_based,
                "max_concurrent_queries": max_concurrent,
                "execution_timeout": format!("{}s", timeout_secs)
            },
            "storage": storage_stats,
            "metadata": {
                "timestamp": Utc::now().to_rfc3339(),
                "uptime": "running",
                "collection_time": Utc::now().to_rfc3339()
            }        });

        Ok(stats)
    }

    /// Analyze query performance and provide optimization recommenaerolithons.
    ///
    /// Examines query execution patterns and provides actionable recommenaerolithons
    /// for improving performance through index creation, query restructuring,
    /// or configuration adjustments.
    ///
    /// # Arguments
    /// * `execution_time_ms` - Query execution time in milliseconds
    /// * `documents_scanned` - Number of documents examined during execution
    /// * `documents_returned` - Number of documents in the result set
    /// * `cache_hit` - Whether the query was served from cache
    ///
    /// # Returns
    /// * `Value` - JSON object containing performance analysis and recommenaerolithons
    pub fn analyze_query_performance(
        execution_time_ms: u64,
        documents_scanned: usize,
        documents_returned: usize,
        cache_hit: bool,
    ) -> Value {
        let mut recommenaerolithons = Vec::new();
        let mut performance_score: f32 = 100.0;

        // Analyze execution time
        if execution_time_ms > 5000 {
            recommenaerolithons.push("Query execution time exceeds 5 seconds. Consider adding indices for filtered fields.".to_string());
            performance_score -= 30.0;
        } else if execution_time_ms > 1000 {
            recommenaerolithons.push("Query execution time is elevated. Review query structure and index utilization.".to_string());
            performance_score -= 15.0;
        }

        // Analyze scan efficiency
        let scan_ratio = if documents_scanned > 0 {
            documents_returned as f64 / documents_scanned as f64
        } else {
            1.0
        };

        if scan_ratio < 0.1 {
            recommenaerolithons.push("Low scan efficiency detected. Consider creating more selective indices.".to_string());
            performance_score -= 25.0;
        } else if scan_ratio < 0.5 {
            recommenaerolithons.push("Moderate scan efficiency. Review filter conditions for optimization opportunities.".to_string());
            performance_score -= 10.0;
        }

        // Analyze cache utilization
        if !cache_hit && documents_scanned > 100 {
            recommenaerolithons.push("Large query not served from cache. Consider query result caching for frequently accessed data.".to_string());
            performance_score -= 10.0;
        }

        json!({
            "performance_score": performance_score.max(0.0),
            "execution_time_ms": execution_time_ms,
            "scan_efficiency": scan_ratio,
            "cache_utilized": cache_hit,
            "recommenaerolithons": recommenaerolithons,
            "metrics": {
                "documents_scanned": documents_scanned,
                "documents_returned": documents_returned,
                "scan_ratio": scan_ratio
            }
        })
    }

    /// Generate query execution summary for monitoring and debugging.
    ///
    /// Creates a detailed summary of query execution including timing,
    /// resource utilization, and result characteristics.
    pub fn generate_execution_summary(
        collection: &str,
        filter: &Option<Value>,
        execution_time_ms: u64,
        documents_found: usize,
        from_cache: bool,
    ) -> Value {
        json!({
            "collection": collection,
            "filter": filter,
            "execution_time_ms": execution_time_ms,
            "documents_found": documents_found,
            "served_from_cache": from_cache,
            "timestamp": Utc::now().to_rfc3339()
        })
    }
}
