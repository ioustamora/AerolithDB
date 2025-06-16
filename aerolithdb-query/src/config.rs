//! # Query Engine Configuration
//!
//! Configuration types and defaults for the aerolithsDB query processing engine.
//! Provides comprehensive configuration for optimization, execution, and resource management.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Comprehensive query engine configuration for optimization and execution control.
///
/// This configuration structure provides fine-grained control over query processing
/// behavior, including optimization strategies, resource limits, and performance tuning.
///
/// ## Configuration Categories
/// - **Optimizer Settings**: Cost-based optimization and query planning
/// - **Execution Limits**: Timeouts, concurrency, and resource constraints
/// - **Feature Flags**: Enable/disable advanced query processing features
/// - **Performance Tuning**: Cache utilization and parallel processing controls
///
/// ## Example Usage
/// ```rust
/// use std::time::Duration;
/// 
/// let config = QueryConfig {
///     optimizer: OptimizerConfig {
///         cost_based: true,
///         statistics_enabled: true,
///         max_optimization_time: Duration::from_secs(10),
///     },
///     execution_timeout: Duration::from_secs(600),
///     max_concurrent_queries: 200,
///     index_advisor: true,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConfig {
    /// Query optimizer configuration and strategy settings
    pub optimizer: OptimizerConfig,
    
    /// Maximum time allowed for query execution before timeout
    pub execution_timeout: Duration,
    
    /// Maximum number of concurrent queries allowed per engine instance
    pub max_concurrent_queries: usize,
    
    /// Enable automatic index recommenaerolithon based on query patterns
    pub index_advisor: bool,
}

/// Configuration for the cost-based query optimizer.
///
/// Controls how the query engine optimizes execution plans using statistics
/// and cost models to minimize query execution time and resource usage.
///
/// ## Optimization Features
/// - **Cost-Based Planning**: Uses statistics to estimate execution costs
/// - **Statistics Collection**: Automatic collection of query performance data
/// - **Optimization Time Limits**: Prevents excessive optimization overhead
/// - **Plan Caching**: Caches optimized plans for repeated queries
///
/// ## Performance Impact
/// Enabling cost-based optimization typically improves query performance
/// for complex queries at the cost of additional planning time. The optimizer
/// uses collected statistics to make informed decisions about:
/// - Index selection and usage
/// - Join order optimization
/// - Predicate pushdown strategies
/// - Parallel execution planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerConfig {
    /// Enable cost-based query optimization for improved performance
    pub cost_based: bool,
    
    /// Enable automatic collection of query execution statistics
    pub statistics_enabled: bool,
    
    /// Maximum time to spend on query plan optimization
    pub max_optimization_time: Duration,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            optimizer: OptimizerConfig {
                cost_based: true,
                statistics_enabled: true,
                max_optimization_time: Duration::from_secs(5),
            },
            execution_timeout: Duration::from_secs(300),
            max_concurrent_queries: 100,
            index_advisor: true,
        }
    }
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            cost_based: true,
            statistics_enabled: true,
            max_optimization_time: Duration::from_secs(5),
        }
    }
}
