//! # CLI Command Arguments
//!
//! This module defines all command-line argument structures for aerolithsDB CLI operations.
//! Each struct represents the arguments for a specific command category.

use clap::Args;

/// Command-line arguments for document storage operations.
///
/// Supports both simple document creation and advanced scenarios with
/// encryption, replication, and retention policies. Data can be provided
/// inline as JSON or loaded from files for larger documents.
#[derive(Debug, Args)]
pub struct PutArgs {
    /// Name of the collection to store the document in.
    /// 
    /// Collections are automatically created if they don't exist.
    /// Collection names must follow naming conventions:
    /// - Alphanumeric characters and underscores
    /// - Maximum 64 characters
    /// - Cannot start with numbers or system reserved prefixes
    pub collection: String,
    
    /// Unique identifier for the document within the collection.
    /// 
    /// Document IDs must be unique within their collection and can be:
    /// - User-provided meaningful identifiers (e.g., "user123", "order-456")
    /// - UUIDs for guaranteed uniqueness across systems
    /// - Natural keys from existing systems
    /// 
    /// Maximum length: 255 characters
    pub id: String,
    
    /// Document data as JSON string or file path (prefixed with @).
    /// 
    /// Supports multiple input formats:
    /// - Inline JSON: `'{"name": "John", "age": 30}'`
    /// - File reference: `@path/to/document.json`
    /// - Large documents: Files are recommended for documents >1KB
    /// 
    /// JSON must be well-formed and can contain nested objects and arrays.
    pub data: String,
    
    /// Optional encryption policy for document storage.
    /// 
    /// Specifies encryption requirements for the document:
    /// - "none": No encryption (default)
    /// - "aes256": AES-256 encryption with server-managed keys
    /// - "client": Client-side encryption (keys managed externally)
    /// 
    /// Encryption policies affect query capabilities and performance.
    #[arg(long)]
    pub encryption_policy: Option<String>,
    
    /// Replication factor for the document across cluster nodes.
    /// 
    /// Determines how many copies of the document are maintained:
    /// - 1: No replication (fastest, least durable)
    /// - 2-3: Standard replication (balanced performance/durability)
    /// - 4+: High durability (slower writes, higher storage cost)
    /// 
    /// Must not exceed the number of available cluster nodes.
    #[arg(long)]
    pub replication_factor: Option<u8>,
    
    /// Data retention policy in days.
    /// 
    /// Automatic deletion after specified period:
    /// - 0: No automatic deletion (default)
    /// - 1-365: Standard retention periods
    /// - >365: Long-term archival (may incur additional costs)
    /// 
    /// Useful for compliance with data protection regulations.
    #[arg(long)]
    pub retention_days: Option<u32>,
}

/// Command-line arguments for document retrieval operations.
///
/// Provides flexible document access with caching optimization and
/// format control. Supports both single document lookup and batch
/// operations for improved performance.
#[derive(Debug, Args)]
pub struct GetArgs {
    /// Name of the collection containing the document.
    pub collection: String,
    
    /// Unique identifier of the document to retrieve.
    pub id: String,
    
    /// Output format for the retrieved document.
    /// 
    /// Available formats:
    /// - "json": Machine-readable JSON format (default)
    /// - "yaml": Human-readable YAML format
    /// - "pretty": Pretty-printed JSON with indentation
    /// 
    /// Format selection affects readability vs. parsing efficiency.
    #[arg(long, default_value = "json")]
    pub format: String,
}

/// Command-line arguments for document deletion operations.
///
/// Supports both immediate deletion and soft deletion patterns.
/// Provides confirmation mechanisms to prevent accidental data loss.
#[derive(Debug, Args)]
pub struct DeleteArgs {
    /// Name of the collection containing the document.
    pub collection: String,
    
    /// Unique identifier of the document to delete.
    pub id: String,
    
    /// Skip interactive confirmation prompt.
    /// 
    /// When set, deletion proceeds without asking for confirmation.
    /// Use with caution in automated scripts and batch operations.
    /// Interactive confirmation is recommended for manual operations.
    #[arg(long)]
    pub force: bool,
    
    /// Perform soft deletion instead of permanent removal.
    /// 
    /// Soft-deleted documents:
    /// - Are marked as deleted but not physically removed
    /// - Can be recovered within a grace period
    /// - Don't appear in normal queries
    /// - Are eventually purged by background processes
    #[arg(long)]
    pub soft: bool,
}

/// Command-line arguments for document querying operations.
///
/// Provides comprehensive querying capabilities including filtering,
/// sorting, pagination, and aggregation. Supports complex queries
/// with multiple conditions and optimized execution.
#[derive(Debug, Args)]
pub struct QueryArgs {
    /// Name of the collection to query.
    pub collection: String,
    
    /// JSON filter criteria for document selection.
    /// 
    /// Supports MongoDB-style query operators:
    /// - Equality: `{"name": "John"}`
    /// - Comparison: `{"age": {"$gt": 25, "$lt": 65}}`
    /// - Logical: `{"$or": [{"status": "active"}, {"priority": "high"}]}`
    /// - Array operations: `{"tags": {"$in": ["important", "urgent"]}}`
    /// - Text search: `{"description": {"$regex": "pattern"}}`
    /// 
    /// Can be provided inline or via file reference (@file.json).
    #[arg(long)]
    pub filter: Option<String>,
    
    /// Sorting specification for result ordering.
    /// 
    /// Format: `field:direction` where direction is 'asc' or 'desc'
    /// Examples:
    /// - Single field: `created_at:desc`
    /// - Multiple fields: `priority:desc,created_at:asc`
    /// - Nested fields: `user.profile.score:desc`
    /// 
    /// Sorting is optimized when fields have appropriate indexes.
    #[arg(long)]
    pub sort: Option<String>,
    
    /// Maximum number of documents to return.
    /// 
    /// Pagination control for large result sets:
    /// - Default: 10 documents
    /// - Range: 1-1000 documents per query
    /// - Use with offset for pagination
    /// 
    /// Large limits may impact query performance and response times.
    #[arg(long, default_value = "10")]
    pub limit: u32,
    
    /// Number of documents to skip before returning results.
    /// 
    /// Used for pagination with limit parameter:
    /// - Page 1: offset=0, limit=10
    /// - Page 2: offset=10, limit=10
    /// - Page N: offset=(N-1)*limit, limit=10
    /// 
    /// Large offsets may impact performance; consider cursor-based pagination.
    #[arg(long, default_value = "0")]
    pub offset: u32,
    
    /// Include detailed metadata in query results.
    /// 
    /// When enabled, results include:
    /// - Document version information
    /// - Creation and modification timestamps
    /// - Replication status and location
    /// - Cache hit information
    /// - Query execution statistics
    #[arg(long)]
    pub include_metadata: bool,
    
    /// Output format for query results.
    /// 
    /// Available formats:
    /// - "json": Structured JSON array (default)
    /// - "jsonl": JSON Lines format (one document per line)
    /// - "csv": Comma-separated values (flattens nested objects)
    /// - "table": Formatted table display
    /// - "count": Returns only the count of matching documents
    #[arg(long, default_value = "json")]
    pub format: String,
    
    /// Include performance analysis with query results.
    /// 
    /// Provides detailed execution information:
    /// - Query parsing and optimization time
    /// - Index usage and efficiency
    /// - Network and storage latency
    /// - Resource consumption metrics
    /// - Optimization recommenaerolithons
    #[arg(long)]
    pub explain: bool,
}

/// Command-line arguments for collection listing operations.
///
/// Provides comprehensive collection discovery and metadata access.
/// Supports filtering and detailed information retrieval for
/// administrative and analytical purposes.
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Specific collection to list (optional).
    /// 
    /// When provided, shows detailed information about the specific collection
    /// including schema analysis, index status, and performance metrics.
    /// When omitted, lists all accessible collections with summary information.
    pub collection: Option<String>,
    
    /// Include detailed collection information.
    /// 
    /// Extended metadata includes:
    /// - Document count and size statistics
    /// - Index information and usage patterns
    /// - Replication and sharding status
    /// - Performance characteristics
    /// - Storage utilization and optimization suggestions
    #[arg(long)]
    pub detailed: bool,
    
    /// Output format for collection information.
    /// 
    /// Available formats:
    /// - "table": Human-readable formatted table (default)
    /// - "json": Machine-readable JSON structure
    /// - "yaml": Human-readable YAML format
    /// - "csv": Comma-separated values for spreadsheet import
    #[arg(long, default_value = "table")]
    pub format: String,
    
    /// Filter collections by name pattern.
    /// 
    /// Supports glob-style patterns:
    /// - Wildcards: `user_*` matches `user_profiles`, `user_sessions`
    /// - Character classes: `log_[0-9]*` matches `log_2023`, `log_2024`
    /// - Case sensitivity: Patterns are case-sensitive by default
    /// 
    /// Useful for organizing collections in large deployments.
    #[arg(long)]
    pub pattern: Option<String>,
}

/// Command-line arguments for database statistics operations.
///
/// Provides comprehensive system monitoring and analytics capabilities.
/// Supports real-time metrics, historical analysis, and performance
/// insights for database administration and optimization.
#[derive(Debug, Args)]
pub struct StatsArgs {
    /// Include detailed system information.
    /// 
    /// Extended statistics include:
    /// - Memory usage and allocation patterns
    /// - Storage utilization and I/O metrics
    /// - Network activity and latency measurements
    /// - Cache performance and hit rates
    /// - Query execution patterns and bottlenecks
    /// - Background process status and resource usage
    #[arg(long)]
    pub detailed: bool,
    
    /// Output format for statistics.
    /// 
    /// Available formats:
    /// - "table": Human-readable formatted display (default)
    /// - "json": Machine-readable structured data
    /// - "prometheus": Prometheus-compatible metrics format
    /// - "csv": Comma-separated values for analysis tools
    #[arg(long, default_value = "table")]
    pub format: String,
    
    /// Focus on specific metric categories.
    /// 
    /// Available categories:
    /// - "performance": Query latency, throughput, and optimization metrics
    /// - "storage": Disk usage, compression ratios, and I/O statistics
    /// - "memory": RAM allocation, cache usage, and garbage collection
    /// - "network": Connection counts, bandwidth, and latency measurements
    /// - "system": CPU usage, load averages, and operating system metrics
    /// 
    /// Multiple categories can be specified: `--category performance,storage`
    #[arg(long)]
    pub category: Option<String>,
    
    /// Show historical trends over time period.
    /// 
    /// Time period specifications:
    /// - "1h": Last hour with minute-level granularity
    /// - "24h": Last 24 hours with hourly granularity
    /// - "7d": Last 7 days with daily granularity
    /// - "30d": Last 30 days with daily granularity
    /// 
    /// Historical data helps identify trends and patterns.
    #[arg(long)]
    pub history: Option<String>,
}

/// Command-line arguments for analytics operations.
///
/// Provides advanced analytical capabilities including query pattern
/// analysis, performance profiling, and optimization recommenaerolithons.
/// Designed for database administrators and performance engineers.
#[derive(Debug, Args)]
pub struct AnalyticsArgs {
    /// Type of analytics report to generate.
    /// 
    /// Available report types:
    /// - "query-patterns": Analysis of query frequency and performance
    /// - "index-usage": Index effectiveness and optimization opportunities
    /// - "storage-analysis": Storage utilization and compression effectiveness
    /// - "performance-profile": Comprehensive performance characterization
    /// - "capacity-planning": Growth trends and scaling recommenaerolithons
    #[arg(long, default_value = "query-patterns")]
    pub report_type: String,
    
    /// Time range for analytics data collection.
    /// 
    /// Supported ranges:
    /// - "1h": Last hour (real-time analysis)
    /// - "24h": Last 24 hours (daily patterns)
    /// - "7d": Last week (weekly trends)
    /// - "30d": Last month (monthly analysis)
    /// - "custom": Custom range (requires --start-date and --end-date)
    #[arg(long, default_value = "24h")]
    pub time_range: String,
    
    /// Output format for analytics results.
    /// 
    /// Available formats:
    /// - "report": Comprehensive human-readable report (default)
    /// - "json": Machine-readable structured data
    /// - "csv": Tabular data for spreadsheet analysis
    /// - "html": Web-ready formatted report with charts
    #[arg(long, default_value = "report")]
    pub format: String,
    
    /// Save analytics results to file.
    /// 
    /// When specified, results are written to the given file path
    /// in addition to console output. File format is determined by
    /// the file extension or the --format parameter.
    #[arg(long)]
    pub output_file: Option<String>,
    
    /// Include optimization recommenaerolithons.
    /// 
    /// When enabled, the analytics report includes:
    /// - Index creation suggestions
    /// - Query optimization recommenaerolithons
    /// - Storage tier optimization opportunities
    /// - Configuration tuning suggestions
    /// - Capacity planning recommenaerolithons
    #[arg(long)]
    pub include_recommenaerolithons: bool,
}

/// Command-line arguments for optimization operations.
///
/// Provides automated and manual optimization capabilities for
/// database performance tuning. Includes index analysis, query
/// optimization, and storage reorganization features.
#[derive(Debug, Args)]
pub struct OptimizeArgs {
    /// Specific collection to optimize (optional).
    /// 
    /// When provided, optimization focuses on the specified collection.
    /// When omitted, performs system-wide optimization analysis.
    /// Collection-specific optimization is more targeted and faster.
    pub collection: Option<String>,
    
    /// Type of optimization to perform.
    /// 
    /// Available optimization types:
    /// - "analyze": Analysis-only mode (no changes made)
    /// - "indexes": Index optimization and recommenaerolithons
    /// - "queries": Query performance optimization
    /// - "storage": Storage layout and compression optimization
    /// - "full": Comprehensive optimization (all types)
    #[arg(long, default_value = "analyze")]
    pub optimization_type: String,
    
    /// Execute optimization recommenaerolithons automatically.
    /// 
    /// When enabled, optimization recommenaerolithons are automatically
    /// applied without manual confirmation. Use with caution in
    /// production environments. Recommended for development and
    /// testing environments only.
    #[arg(long)]
    pub auto_apply: bool,
    
    /// Dry-run mode (show what would be optimized).
    /// 
    /// When enabled, shows optimization actions that would be taken
    /// without actually performing them. Useful for understanding
    /// the impact of optimization before applying changes.
    #[arg(long)]
    pub dry_run: bool,
    
    /// Include detailed optimization analysis.
    /// 
    /// Provides comprehensive information about:
    /// - Current performance characteristics
    /// - Identified optimization opportunities
    /// - Expected impact of recommenaerolithons
    /// - Implementation complexity and risks
    /// - Rollback procedures if needed
    #[arg(long)]
    pub detailed: bool,
}

// ================================================================================================
// CONFIGURATION MANAGEMENT COMMANDS
// ================================================================================================

/// Command-line arguments for configuration valiaerolithon operations.
///
/// Provides comprehensive configuration valiaerolithon including syntax,
/// schema, and value valiaerolithon. Supports both local file and
/// server configuration valiaerolithon modes.
#[derive(Debug, Args)]
pub struct ConfigValidateArgs {
    /// Path to configuration file to validate.
    /// 
    /// Supports multiple configuration formats:
    /// - JSON: config.json
    /// - YAML: config.yaml, config.yml
    /// - TOML: config.toml
    /// 
    /// File format is determined by extension.
    #[arg(long)]
    pub file_path: Option<String>,
    
    /// Validate server-side configuration.
    /// 
    /// When enabled, validates the configuration currently
    /// running on the connected aerolithsDB server instead of
    /// a local file. Requires active server connection.
    #[arg(long)]
    pub server_config: bool,
    
    /// Enable strict valiaerolithon mode.
    /// 
    /// In strict mode:
    /// - Warnings are treated as errors
    /// - All optional fields are validated
    /// - Deprecated settings cause valiaerolithon failure
    /// - Future compatibility is checked
    #[arg(long)]
    pub strict: bool,
}

/// Command-line arguments for configuration generation operations.
///
/// Provides template generation for various deployment scenarios
/// with customizable components and output formats.
#[derive(Debug, Args)]
pub struct ConfigGenerateArgs {
    /// Configuration template type to generate.
    /// 
    /// Available templates:
    /// - "basic": Minimal configuration for simple deployments
    /// - "development": Local development with relaxed security
    /// - "production": Enterprise-grade security and performance
    /// - "cluster": Multi-node distributed deployment
    /// - "security": Zero-trust security focused configuration
    #[arg(long, default_value = "basic")]
    pub template: String,
    
    /// Output format for generated configuration.
    /// 
    /// Available formats:
    /// - "json": Machine-readable JSON (default)
    /// - "yaml": Human-readable YAML with comments
    /// - "toml": Simple configuration syntax
    /// - "env": Environment variable format
    #[arg(long, default_value = "yaml")]
    pub format: String,
    
    /// Output file path (stdout if not specified).
    #[arg(long)]
    pub output: Option<String>,
    
    /// Include only specific configuration components.
    /// 
    /// Available components:
    /// - node, network, storage, cache, security
    /// - consensus, query, api, plugins, observability
    /// 
    /// Multiple components: --components security,storage,api
    #[arg(long, value_delimiter = ',')]
    pub components: Vec<String>,
}

/// Command-line arguments for configuration display operations.
///
/// Provides comprehensive configuration viewing with security
/// considerations and flexible formatting options.
#[derive(Debug, Args)]
pub struct ConfigShowArgs {
    /// Display server configuration instead of defaults.
    #[arg(long)]
    pub server_config: bool,
    
    /// Show only specific configuration section.
    /// 
    /// Available sections:
    /// - node, network, storage, cache, security
    /// - consensus, query, api, plugins, observability
    #[arg(long)]
    pub section: Option<String>,
    
    /// Output format for configuration display.
    /// 
    /// Available formats:
    /// - "json": Structured JSON format
    /// - "yaml": Human-readable YAML format
    /// - "table": Formatted table display
    #[arg(long, default_value = "yaml")]
    pub format: String,
    
    /// Show sensitive values (passwords, keys).
    /// 
    /// WARNING: Use with caution as this exposes sensitive
    /// configuration values. Only use in secure environments
    /// and avoid logging output.
    #[arg(long)]
    pub show_secrets: bool,
    
    /// Show only changed values from defaults.
    #[arg(long)]
    pub changed_only: bool,
}

// ================================================================================================
// BATCH OPERATIONS COMMANDS
// ================================================================================================

/// Command-line arguments for batch document insertion operations.
///
/// Provides high-performance bulk document insertion with
/// flexible input sources and processing options.
#[derive(Debug, Args)]
pub struct BatchPutArgs {
    /// Target collection for document insertion.
    pub collection: String,
    
    /// Input file path for document data.
    /// 
    /// Supported formats:
    /// - JSON: Single document or array of documents
    /// - JSON Lines: One document per line (recommended for large datasets)
    /// - CSV: Structured data with header row
    #[arg(long)]
    pub file: Option<String>,
    
    /// Read document data from stdin.
    /// 
    /// Useful for pipeline integration:
    /// cat documents.jsonl | aerolithsdb-cli batch put collection --stdin
    #[arg(long)]
    pub stdin: bool,
    
    /// Input data format.
    /// 
    /// Available formats:
    /// - "json": JSON array or single document
    /// - "jsonl": JSON Lines (one document per line)
    /// - "csv": Comma-separated values with headers
    #[arg(long, default_value = "jsonl")]
    pub format: String,
    
    /// Number of documents per batch request.
    /// 
    /// Optimal batch size depends on:
    /// - Document size (smaller docs = larger batches)
    /// - Network latency (higher latency = larger batches)
    /// - Memory constraints (available RAM limits)
    /// 
    /// Typical range: 50-1000 documents per batch
    #[arg(long)]
    pub batch_size: Option<usize>,
    
    /// Number of parallel batch processing threads.
    /// 
    /// Controls concurrency for batch operations:
    /// - Higher values = faster processing (up to server limits)
    /// - Lower values = reduced server load
    /// - Optimal value depends on server capacity and network
    #[arg(long)]
    pub parallel: Option<usize>,
    
    /// Continue processing on individual document errors.
    /// 
    /// When enabled, batch operation continues even if some
    /// documents fail to insert. Error summary is provided
    /// at the end of the operation.
    #[arg(long)]
    pub continue_on_error: bool,
    
    /// Enable verbose progress reporting.
    #[arg(long)]
    pub verbose: bool,
    
    /// Field name to use as document ID.
    /// 
    /// When specified, uses the value of this field as the
    /// document ID instead of auto-generating UUIDs.
    /// Field must exist and be unique across all documents.
    #[arg(long)]
    pub id_field: Option<String>,
}

/// Command-line arguments for batch document deletion operations.
///
/// Provides safe and efficient bulk document deletion with
/// multiple targeting methods and safety features.
#[derive(Debug, Args)]
pub struct BatchDeleteArgs {
    /// Target collection for document deletion.
    pub collection: String,
    
    /// Comma-separated list of document IDs to delete.
    /// 
    /// Example: --ids "doc1,doc2,doc3"
    /// For large ID lists, consider using --file option.
    #[arg(long)]
    pub ids: Option<String>,
    
    /// File containing document IDs to delete (one per line).
    /// 
    /// File format: plain text with one document ID per line.
    /// Empty lines and whitespace are ignored.
    #[arg(long)]
    pub file: Option<String>,
    
    /// JSON filter to select documents for deletion.
    /// 
    /// Uses same query syntax as regular queries.
    /// Example: --filter '{"status": "archived", "age": {"$gt": 365}}'
    /// 
    /// WARNING: This can delete many documents. Use with caution.
    #[arg(long)]
    pub filter: Option<String>,
    
    /// Skip confirmation prompt and delete immediately.
    /// 
    /// Use with extreme caution in production environments.
    /// Recommended only for automated scripts with proper safeguards.
    #[arg(long)]
    pub force: bool,
    
    /// Show what would be deleted without actually deleting.
    /// 
    /// Dry run mode shows:
    /// - Number of documents that would be deleted
    /// - Document IDs (up to a reasonable limit)
    /// - Estimated operation time
    #[arg(long)]
    pub dry_run: bool,
    
    /// Create backup before deletion.
    /// 
    /// Creates a backup file containing all documents that will
    /// be deleted, allowing for recovery if needed.
    #[arg(long)]
    pub backup: bool,
    
    /// Number of documents per batch deletion request.
    #[arg(long)]
    pub batch_size: Option<usize>,
    
    /// Number of parallel deletion threads.
    #[arg(long)]
    pub parallel: Option<usize>,
    
    /// Continue processing on individual document errors.
    #[arg(long)]
    pub continue_on_error: bool,
    
    /// Enable verbose progress reporting.
    #[arg(long)]
    pub verbose: bool,
}

/// Command-line arguments for data import operations.
///
/// Provides comprehensive data import from various external
/// formats with transformation and valiaerolithon capabilities.
#[derive(Debug, Args)]
pub struct BatchImportArgs {
    /// Target collection for imported documents.
    pub collection: String,
    
    /// Source file path for import data.
    #[arg(long)]
    pub file: Option<String>,
    
    /// Source data format.
    /// 
    /// Supported formats:
    /// - "json": JSON documents or arrays
    /// - "csv": Comma-separated values with headers
    /// - "xml": XML documents (with mapping configuration)
    /// - "tsv": Tab-separated values
    /// - "parquet": Columnar data format
    #[arg(long, default_value = "json")]
    pub format: String,
    
    /// Field name to use as document ID.
    #[arg(long)]
    pub id_field: Option<String>,
    
    /// Field mapping transformations.
    /// 
    /// Format: "source_field:target_field"
    /// Example: --map-fields "first_name:name.first,last_name:name.last"
    #[arg(long, value_delimiter = ',')]
    pub map_fields: Vec<String>,
    
    /// JSON schema file for document valiaerolithon.
    /// 
    /// Documents are validated against the schema before import.
    /// Invalid documents are rejected with detailed error messages.
    #[arg(long)]
    pub validate_schema: Option<String>,
    
    /// Number of documents per batch.
    #[arg(long)]
    pub batch_size: Option<usize>,
    
    /// Number of parallel processing threads.
    #[arg(long)]
    pub parallel: Option<usize>,
    
    /// Continue import on individual document errors.
    #[arg(long)]
    pub continue_on_error: bool,
    
    /// Enable verbose progress reporting.
    #[arg(long)]
    pub verbose: bool,
    
    /// Import mode for handling existing documents.
    /// 
    /// Available modes:
    /// - "insert": Insert new documents only, skip existing IDs
    /// - "update": Update existing documents, preserve missing fields
    /// - "replace": Replace existing documents completely
    /// - "upsert": Insert new or update existing documents
    #[arg(long, default_value = "upsert")]
    pub mode: String,
}

/// Command-line arguments for data export operations.
///
/// Provides flexible data export to various formats with
/// filtering and transformation capabilities.
#[derive(Debug, Args)]
pub struct BatchExportArgs {
    /// Source collection for export.
    pub collection: String,
    
    /// Output file path (stdout if not specified).
    #[arg(long)]
    pub output: Option<String>,
    
    /// Export data format.
    /// 
    /// Available formats:
    /// - "json": JSON array of documents
    /// - "jsonl": JSON Lines (one document per line)
    /// - "csv": Comma-separated values with headers
    /// - "xml": XML format with configurable structure
    /// - "tsv": Tab-separated values
    /// - "parquet": Columnar data format
    #[arg(long, default_value = "jsonl")]
    pub format: String,
    
    /// JSON filter for document selection.
    /// 
    /// Only documents matching the filter will be exported.
    /// Uses same query syntax as regular queries.
    #[arg(long)]
    pub filter: Option<String>,
    
    /// Specific fields to include in export.
    /// 
    /// Comma-separated list of field names.
    /// Example: --fields "id,name,email,created_at"
    /// If not specified, all fields are included.
    #[arg(long, value_delimiter = ',')]
    pub fields: Vec<String>,
    
    /// Maximum number of documents to export.
    #[arg(long)]
    pub limit: Option<usize>,
    
    /// Compress output data (for supported formats).
    #[arg(long)]
    pub compress: bool,
    
    /// Pretty-print JSON output (for JSON format).
    #[arg(long)]
    pub pretty: bool,
    
    /// Use streaming export for large datasets.
    /// 
    /// Streaming mode processes documents incrementally,
    /// reducing memory usage for large collections.
    #[arg(long)]
    pub streaming: bool,
}
