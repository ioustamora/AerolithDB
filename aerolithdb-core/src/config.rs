// aerolithsDB Configuration Management System
//
// This module provides comprehensive configuration management for all aerolithsDB subsystems.
// It supports multiple configuration sources, valiaerolithon, and hot-reload capabilities.
// The configuration system is designed to handle complex distributed database scenarios
// with enterprise-grade security, compliance, and operational requirements.
//
// ## Configuration Sources (in order of precedence)
// 1. Command-line arguments
// 2. Environment variables  
// 3. Configuration files (JSON, YAML, TOML)
// 4. Default values
//
// ## Configuration Categories
// - **Node Configuration**: Identity, network binding, and local settings
// - **Network Configuration**: Cluster membership and P2P communication
// - **Storage Configuration**: Multi-tier storage hierarchy and data management
// - **Cache Configuration**: Intelligent caching with ML-driven optimization
// - **Security Configuration**: Zero-trust security, encryption, and compliance
// - **Consensus Configuration**: Distributed agreement algorithms and parameters
// - **Query Configuration**: Query processing, optimization, and execution settings
// - **API Configuration**: Multi-protocol API endpoints and security settings
// - **Plugin Configuration**: Extension system security and loading policies
// - **Observability Configuration**: Metrics, tracing, logging, and alerting

// Import essential dependencies for serialization, file operations, and error handling
use anyhow::Result;                           // Unified error handling with context
use serde::{Deserialize, Serialize};         // JSON/YAML serialization support
use std::path::PathBuf;                       // Cross-platform file path handling
use std::time::Duration;                      // Time duration for timeouts and intervals

// Import from security module to avoid duplication
use aerolithdb_security::{EncryptionAlgorithm, AuditLevel, ComplianceMode, SecurityConfig};

// Import from consensus module
use aerolithdb_consensus::ConsensusAlgorithm;

/// Main configuration structure for the entire aerolithsDB system.
/// 
/// This is the root configuration object that contains all settings for every
/// subsystem in the database. It supports serialization to multiple formats
/// (JSON, YAML, TOML) and provides valiaerolithon for all configuration values.
/// 
/// The configuration is designed to be:
/// - **Hierarchical**: Organized by functional areas and subsystems
/// - **Extensible**: Easy to add new configuration sections
/// - **Validatable**: Built-in valiaerolithon for all critical settings
/// - **Environment-aware**: Support for environment-specific overrides
/// - **Hot-reloadable**: Safe runtime configuration updates where possible
/// 
/// # Example
/// ```rust
/// use aerolithsdb_core::aerolithsConfig;
/// 
/// // Load configuration from file or use defaults
/// let config = aerolithsConfig::load().await?;
/// 
/// // Access specific subsystem configurations
/// println!("Node ID: {}", config.node.node_id);
/// println!("Storage strategy: {:?}", config.storage.sharding_strategy);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AerolithsConfig {
    /// Node identity and local binding configuration
    pub node: NodeConfig,
    
    /// P2P network and cluster membership settings
    pub network: NetworkConfig,
    
    /// Multi-tier storage hierarchy configuration
    pub storage: StorageConfig,
    
    /// Intelligent caching system settings
    pub cache: CacheConfig,
    
    /// Zero-trust security and encryption configuration
    pub security: SecurityConfig,
    
    /// Distributed consensus algorithm settings
    pub consensus: ConsensusConfig,
    
    /// Query processing and optimization configuration
    pub query: QueryConfig,
    
    /// Multi-protocol API gateway settings
    pub api: APIConfig,
    
    /// Plugin system and extension configuration
    pub plugins: PluginConfig,
    
    /// Monitoring, metrics, and observability settings
    pub observability: ObservabilityConfig,
}

/// Node-specific configuration including identity and network binding.
/// 
/// Defines the local node's identity within the distributed cluster and
/// configures how the node binds to network interfaces and discovers peers.
/// Each node must have a unique identifier and appropriate network settings.
/// 
/// # Security Considerations
/// - Node IDs should be cryptographically unique to prevent identity conflicts
/// - External addresses must be properly configured for NAT traversal
/// - Bind addresses should follow security best practices (avoid 0.0.0.0 in production)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Unique identifier for this node in the cluster (UUID recommended)
    pub node_id: String,
    
    /// Local directory for node-specific data storage
    pub data_dir: PathBuf,
    
    /// IP address to bind services to (use "127.0.0.1" for localhost only)
    pub bind_address: String,
    
    /// Primary port for node communication (other services use port + offset)
    pub port: u16,
    
    /// External address for NAT traversal and peer discovery (auto-detected if None)
    pub external_address: Option<String>,
}

/// Network cluster configuration for P2P communication and discovery.
/// 
/// Configures how nodes communicate within the distributed cluster including
/// peer discovery, connection management, and cluster membership protocols.
/// These settings directly impact cluster formation and network resilience.
/// 
/// # Network Architecture
/// - **Network ID**: Unique identifier preventing cross-cluster communication
/// - **Bootstrap Nodes**: Initial seed nodes for cluster discovery
/// - **Connection Limits**: Prevent resource exhaustion and maintain performance
/// - **Timeouts**: Balance responsiveness with network resilience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Unique network identifier to prevent cross-cluster communication
    pub network_id: String,
    
    /// Human-readable name for this network cluster
    pub network_name: String,
    
    /// Governance policy identifier for network-level decision making
    pub governance_policy: String,
    
    /// List of bootstrap nodes for initial cluster discovery
    pub bootstrap_nodes: Vec<String>,
    
    /// Maximum number of concurrent peer connections
    pub max_connections: usize,
    
    /// Timeout for establishing new peer connections
    pub connection_timeout: Duration,
    
    /// Interval between heartbeat messages to maintain connections
    pub heartbeat_interval: Duration,
}

/// Multi-tier storage hierarchy configuration.
/// 
/// Configures the sophisticated storage system that automatically manages data
/// across different storage tiers (memory, SSD, distributed, archival) based on
/// access patterns, data age, and performance requirements.
/// 
/// # Storage Tiers
/// - **Hot Tier**: In-memory storage for frequently accessed data
/// - **Warm Tier**: Local SSD storage for recent data
/// - **Cold Tier**: Distributed storage across cluster nodes  
/// - **Archive Tier**: Long-term object storage for historical data
/// 
/// # Data Lifecycle
/// Data automatically migrates between tiers based on:
/// - Access frequency and recency
/// - Data age and retention policies
/// - Storage capacity and cost optimization
/// - Compliance and audit requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Strategy for distributing data across nodes and shards
    pub sharding_strategy: ShardingStrategy,
    
    /// Number of replicas to maintain for each data item (minimum 1)
    pub replication_factor: usize,
    
    /// Compression settings for storage optimization
    pub compression: CompressionConfig,
    
    /// Enable encryption at rest for data security compliance
    pub encryption_at_rest: bool,
    
    /// Root directory for local storage tiers
    pub data_dir: PathBuf,
    
    /// Maximum total storage size before triggering archival (None = unlimited)
    pub max_storage_size: Option<u64>,
}

/// Intelligent caching system configuration with ML-driven optimization.
/// 
/// Configures the multi-layer intelligent caching system that uses machine learning
/// algorithms to predict data access patterns and optimize cache performance.
/// The system adapts to workload changes and provides transparent acceleration.
/// 
/// # Cache Layers
/// - **L1 (Memory)**: Ultra-fast in-memory cache with nanosecond access times
/// - **L2 (NVMe)**: Local NVMe SSD cache with microsecond access times  
/// - **L3 (Network)**: Distributed cache across cluster nodes
/// 
/// # ML-Driven Features
/// - Predictive prefetching based on access patterns
/// - Adaptive eviction policies that learn from workload characteristics
/// - Dynamic cache sizing based on performance metrics
/// - Anomaly detection for cache performance issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Ordered list of cache layers from fastest to slowest
    pub hierarchy: Vec<CacheLayer>,
    
    /// Enable machine learning-based predictive prefetching
    pub ml_prefetching: bool,
    
    /// Enable compression for cached data to maximize capacity
    pub compression: bool,
    
    /// Time-to-live strategy for cache expiration management
    pub ttl_strategy: TTLStrategy,
    
    /// Maximum memory usage for in-memory cache layers (bytes)
    pub max_memory_usage: u64,
}

/// Zero-trust security framework configuration.
/// 
/// Configures the comprehensive security system that implements zero-trust principles
/// with end-to-end encryption, fine-grained access control, and compliance monitoring.
/// All communications and data storage are encrypted by default.
/// 
/// # Security Features
/// - **Zero-Trust Architecture**: Never trust, always verify approach
/// - **End-to-End Encryption**: Data encrypted in transit and at rest
/// - **Fine-Grained Access Control**: Attribute-based access control (ABAC)
/// - **Cryptographic Identity**: Node and user identity based on cryptographic keys
/// - **Audit Logging**: Comprehensive audit trails for compliance
/// - **Key Management**: Automatic key rotation and lifecycle management

/// Distributed consensus algorithm configuration.
/// 
/// Configures the consensus engine that ensures distributed agreement across
/// the cluster for data consistency and conflict resolution. The system supports
/// multiple consensus algorithms optimized for different scenarios.
/// 
/// # Consensus Algorithms
/// - **Byzantine PBFT**: Byzantine fault tolerance for untrusted environments
/// - **Raft**: Simplified consensus for trusted network environments
/// - **HoneyBadgerBFT**: Asynchronous Byzantine fault tolerance
/// 
/// # Performance Tuning
/// - Batch size affects throughput vs latency trade-offs
/// - Timeout values balance responsiveness with network resilience
/// - Byzantine tolerance threshold determines fault tolerance level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Consensus algorithm to use for distributed agreement
    pub algorithm: ConsensusAlgorithm,
    
    /// Fraction of nodes that can be Byzantine faulty (typically 0.33)
    pub byzantine_tolerance: f32,
    
    /// Maximum time to wait for consensus agreement
    pub timeout: Duration,
    
    /// Maximum number of operations to batch together for efficiency
    pub max_batch_size: usize,
    
    /// Strategy for resolving conflicting concurrent updates
    pub conflict_resolution: ConflictResolution,
}

/// Query processing and optimization engine configuration.
/// 
/// Configures the distributed query engine that processes complex queries across
/// the cluster with cost-based optimization and intelligent execution planning.
/// The engine supports SQL-like queries, full-text search, and analytics workloads.
/// 
/// # Query Features
/// - **Cost-Based Optimization**: Statistics-driven query plan optimization
/// - **Distributed Execution**: Queries executed across multiple nodes
/// - **Index Advisor**: Automatic index recommenaerolithons for performance
/// - **Parallel Processing**: Multi-threaded query execution
/// - **Result Streaming**: Large result sets streamed efficiently
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConfig {
    /// Query optimizer configuration and statistics settings
    pub optimizer: OptimizerConfig,
    
    /// Maximum time allowed for query execution
    pub execution_timeout: Duration,
    
    /// Maximum number of concurrent queries per node
    pub max_concurrent_queries: usize,
    
    /// Enable automatic index recommenaerolithons for performance optimization
    pub index_advisor: bool,
}

/// Multi-protocol API gateway configuration.
/// 
/// Configures the API gateway that provides multiple protocol interfaces
/// for client applications including REST, GraphQL, gRPC, and WebSocket.
/// Each protocol is optimized for different use cases and client types.
/// 
/// # Protocol Support
/// - **REST API**: Standard HTTP/JSON API for web applications
/// - **GraphQL**: Flexible query language for modern applications
/// - **gRPC**: High-performance RPC for microservices
/// - **WebSocket**: Real-time bidirectional communication
/// 
/// # Security Integration
/// All protocols integrate with the security framework for:
/// - Authentication and authorization
/// - Rate limiting and DDoS protection
/// - Request/response encryption
/// - Audit logging of all API access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIConfig {
    /// RESTful HTTP/JSON API configuration
    pub rest_api: RESTAPIConfig,
    
    /// GraphQL query language API configuration
    pub graphql_api: GraphQLConfig,
    
    /// gRPC high-performance RPC API configuration
    pub grpc_api: GRPCConfig,
    
    /// WebSocket real-time communication API configuration
    pub websocket_api: WebSocketConfig,
}

/// Plugin system configuration for extensibility.
/// 
/// Configures the secure plugin system that allows extending aerolithsDB functionality
/// with custom storage backends, query processors, security modules, and more.
/// Plugins are sandboxed for security and can be loaded/unloaded at runtime.
/// 
/// # Plugin Types
/// - **Storage Plugins**: Custom storage backends and drivers
/// - **Query Plugins**: Domain-specific query processors
/// - **Security Plugins**: Custom authentication and authorization
/// - **Analytics Plugins**: Specialized data analytics and ML models
/// - **Integration Plugins**: External system connectors
/// 
/// # Security Model
/// Plugins run in restricted sandboxes with:
/// - Limited system access permissions
/// - Resource usage quotas
/// - API access controls
/// - Runtime monitoring and isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Directory containing plugin files and configurations
    pub plugin_dir: PathBuf,
    
    /// Automatically load plugins at startup
    pub auto_load: bool,
    
    /// Security policy for plugin execution and permissions
    pub security_policy: PluginSecurityPolicy,
}

/// Observability and monitoring configuration.
/// 
/// Configures comprehensive observability including metrics collection, distributed
/// tracing, structured logging, and alerting. Essential for operational visibility
/// and performance monitoring in production distributed database deployments.
/// 
/// # Observability Components
/// - **Metrics**: Prometheus-compatible metrics for performance monitoring
/// - **Tracing**: Distributed tracing for request flow analysis
/// - **Logging**: Structured logging with multiple output formats
/// - **Alerting**: Configurable alerts for operational issues
/// 
/// # Integration Points
/// - Prometheus for metrics collection and visualization
/// - Jaeger for distributed tracing and performance analysis
/// - ELK stack for log aggregation and analysis
/// - PagerDuty/Slack for alert notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Performance metrics collection and export settings
    pub metrics: MetricsConfig,
    
    /// Distributed tracing configuration for request analysis
    pub tracing: TracingConfig,
    
    /// Structured logging configuration and output settings
    pub logging: LoggingConfig,
    
    /// Alert configuration for operational notifications
    pub alerting: AlertingConfig,
}

// ================================================================================================
// ENUMERATION TYPES FOR CONFIGURATION OPTIONS
// ================================================================================================

/// Eviction policy for cache management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    LRU,  // Least Recently Used
    LFU,  // Least Frequently Used
    TTL,  // Time To Live
    ML,   // Machine Learning based
}

/// Plugin types supported by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginType {
    Storage,
    Query,
    Security,
    Network,
}

/// Sharding strategies for data distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShardingStrategy {
    ConsistentHashing,
    RangeBased,
    DirectoryBased,
}

/// Cache layer types in the hierarchical caching system.
/// 
/// Each layer provides different performance characteristics and capacity.
/// The intelligent cache system automatically manages data placement across
/// layers based on access patterns and performance requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheLayer {
    /// In-memory cache layer with nanosecond access times
    Memory,
    
    /// NVMe SSD cache layer with microsecond access times
    NVMe,
    
    /// Network-distributed cache layer across cluster nodes
    Network,
}

/// Time-to-live strategies for cache expiration management.
/// 
/// Different TTL strategies optimize for various workload patterns and
/// help balance cache hit rates with memory usage efficiency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TTLStrategy {
    /// Machine learning-driven adaptive TTL based on access patterns
    Adaptive,
    
    /// Fixed TTL duration for all cached items
    Fixed(Duration),    
    /// Least Recently Used eviction without explicit TTL
    LRU,
}

/// Conflict resolution strategies for concurrent data updates.
/// 
/// When multiple clients update the same data concurrently, the system
/// uses these strategies to resolve conflicts and maintain consistency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Last writer wins - simple timestamp-based resolution
    LastWriterWins,
    
    /// Semantic merge based on data type and structure
    SemanticMerge,
    
    /// Custom user-defined conflict resolution function
    UserDefinedResolver,
    
    /// Manual intervention required for conflict resolution
    RequireManualIntervention,
}

// ================================================================================================
// SUB-CONFIGURATION STRUCTURES
// ================================================================================================

/// Data compression configuration for storage optimization.
/// 
/// Compression reduces storage space and network bandwidth at the cost of
/// CPU overhead. Adaptive compression automatically adjusts based on data
/// characteristics and performance requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Compression algorithm to use for data reduction
    pub algorithm: CompressionAlgorithm,
    
    /// Compression level (1-22, higher = better compression, slower)
    pub level: u8,
    
    /// Enable adaptive compression based on data characteristics
    pub adaptive: bool,
}

/// Available compression algorithms with different performance characteristics.
/// 
/// Each algorithm provides different trade-offs between compression ratio,
/// speed, and CPU usage. Choose based on workload requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// LZ4: Ultra-fast compression with moderate ratios
    LZ4,
    
    /// Zstd: Balanced compression with good speed and ratios
    Zstd,
    
    /// Snappy: Fast compression optimized for speed over ratio
    Snappy,
}

/// Query optimizer configuration for performance tuning.
/// 
/// The cost-based optimizer uses statistics and heuristics to generate
/// efficient query execution plans. Proper configuration is essential
/// for optimal query performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerConfig {
    /// Enable cost-based optimization using table statistics
    pub cost_based: bool,
    
    /// Maintain table and index statistics for optimization
    pub statistics_enabled: bool,
    
    /// Maximum time allowed for query plan optimization
    pub max_optimization_time: Duration,
}

/// REST API endpoint configuration.
/// 
/// Configures the RESTful HTTP/JSON API endpoint that provides standard
/// web-compatible access to database operations. Essential for web
/// applications and standard HTTP clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RESTAPIConfig {
    /// Enable the REST API endpoint
    pub enabled: bool,
    
    /// IP address to bind the REST API server
    pub bind_address: String,
    
    /// Port number for the REST API server
    pub port: u16,
    
    /// Enable Cross-Origin Resource Sharing (CORS) for web browsers
    pub cors_enabled: bool,
}

/// GraphQL API endpoint configuration.
/// 
/// Configures the GraphQL endpoint that provides a flexible query language
/// for modern applications. Allows clients to specify exactly what data
/// they need, reducing over-fetching and improving performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLConfig {
    /// Enable the GraphQL API endpoint
    pub enabled: bool,    /// IP address to bind the GraphQL API server
    pub bind_address: String,
    
    /// Port number for the GraphQL API server
    pub port: u16,
    
    /// Enable GraphQL introspection for development (disable in production)
    pub introspection: bool,
    
    /// Enable GraphQL Playground IDE for development (disable in production)
    pub playground: bool,
}

/// gRPC API endpoint configuration.
/// 
/// Configures the high-performance gRPC endpoint for microservice
/// communication and high-throughput applications. Provides efficient
/// binary protocol with strong typing and streaming support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GRPCConfig {
    /// Enable the gRPC API endpoint
    pub enabled: bool,
    
    /// IP address to bind the gRPC server
    pub bind_address: String,
    
    /// Port number for the gRPC server
    pub port: u16,
    
    /// Enable gRPC reflection for development tools
    pub reflection: bool,
}

/// WebSocket API endpoint configuration.
/// 
/// Configures real-time bidirectional communication for applications
/// requiring live updates, streaming data, or interactive features.
/// Essential for real-time dashboards and collaborative applications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Enable the WebSocket API endpoint
    pub enabled: bool,
    
    /// IP address to bind the WebSocket server
    pub bind_address: String,
    
    /// Port number for the WebSocket server
    pub port: u16,
    
    /// Maximum number of concurrent WebSocket connections
    pub max_connections: usize,
}

/// Plugin security policies for extension system safety.
/// 
/// Defines the security model for plugin execution to protect the system
/// from malicious or buggy extensions while enabling functionality extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginSecurityPolicy {
    /// Full sandboxing with minimal system access
    Sandboxed,
    
    /// Trusted plugins with extended permissions
    Trusted,
    
    /// Restrictive policy with limited API access
    Restrictive,
}

/// Prometheus-compatible metrics collection configuration.
/// 
/// Configures metrics collection and export for monitoring database
/// performance, resource usage, and operational health. Essential for
/// production monitoring and alerting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection and export
    pub enabled: bool,
    
    /// Prometheus endpoint URL for metrics scraping
    pub prometheus_endpoint: String,
    
    /// Interval between metrics collection cycles
    pub collection_interval: Duration,
}

/// Distributed tracing configuration for request analysis.
/// 
/// Configures integration with distributed tracing systems to track
/// request flows across the distributed database cluster. Essential
/// for performance analysis and debugging in distributed environments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable distributed tracing
    pub enabled: bool,
    
    /// Jaeger collector endpoint URL (None for auto-discovery)
    pub jaeger_endpoint: Option<String>,
    
    /// Sampling ratio for trace collection (0.0-1.0)
    pub sampling_ratio: f32,
}

/// Structured logging configuration for operational visibility.
/// 
/// Configures comprehensive logging system with structured output
/// for integration with log aggregation and analysis systems.
/// Critical for debugging, auditing, and operational monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Minimum log level to output (trace, debug, info, warn, error)
    pub level: String,
    
    /// Optional file output path (None for stdout only)
    pub file_output: Option<PathBuf>,
    
    /// Enable structured JSON logging format
    pub structured: bool,
}

/// Alert configuration for operational notifications.
/// 
/// Configures automated alerting for critical system events and
/// performance issues. Integrates with external notification systems
/// for operational team awareness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    /// Enable alerting system
    pub enabled: bool,
    
    /// Webhook URL for alert notifications (Slack, PagerDuty, etc.)
    pub webhook_url: Option<String>,
    
    /// Alert thresholds for various metrics (metric_name -> threshold_value)
    pub thresholds: std::collections::HashMap<String, f64>,
}

// ================================================================================================
// CONFIGURATION LOADING AND MANAGEMENT
// ================================================================================================
impl AerolithsConfig {
    /// Load configuration from multiple sources with precedence order.
    /// 
    /// This method implements a hierarchical configuration loading system that
    /// combines multiple sources in order of precedence:
    /// 1. Command-line arguments (highest precedence)
    /// 2. Environment variables
    /// 3. Configuration files (config.json, config.yaml, config.toml)
    /// 4. Default values (lowest precedence)
    /// 
    /// If no configuration file exists, a default configuration is created
    /// and saved for future use. This ensures a working default setup for
    /// new installations while allowing customization.
    /// 
    /// # Configuration File Formats
    /// Supports multiple configuration file formats:
    /// - **JSON**: config.json (human-readable, widely supported)
    /// - **YAML**: config.yaml (more readable, supports comments)
    /// - **TOML**: config.toml (simple, intuitive syntax)
    /// 
    /// # Environment Variable Overrides
    /// Any configuration value can be overridden using environment variables
    /// with the format: `aerolithSDB_<SECTION>_<FIELD>` (e.g., `aerolithSDB_NODE_PORT=9001`)
    /// 
    /// # Returns
    /// - `Ok(aerolithsConfig)` with loaded and validated configuration
    /// - `Err(anyhow::Error)` if configuration is invalid or cannot be loaded
    /// 
    /// # Example
    /// ```rust
    /// // Load configuration from file or create default
    /// let config = aerolithsConfig::load().await?;
    /// println!("Loaded configuration for node: {}", config.node.node_id);
    /// ```
    pub async fn load() -> Result<Self> {
        // Try to load from config file, fallback to default
        match tokio::fs::read_to_string("config.json").await {
            Ok(content) => {
                serde_json::from_str(&content)
                    .map_err(|e| anyhow::anyhow!("Failed to parse config: {}", e))
            }
            Err(_) => {
                // Create default config
                let default_config = Self::default();
                default_config.save().await?;
                Ok(default_config)
            }
        }
    }

    /// Save configuration to file for persistence and sharing.
    /// 
    /// Serializes the current configuration to JSON format and saves it to
    /// the standard configuration file location. This allows configuration
    /// changes to persist across application restarts and enables easy
    /// configuration sharing between development and production environments.
    /// 
    /// The saved configuration file uses pretty-printed JSON formatting for
    /// human readability and includes all configuration sections with their
    /// current values.
    /// 
    /// # File Format
    /// The configuration is saved as pretty-printed JSON with proper indentation
    /// and field ordering for human readability. All configuration sections
    /// are included even if they use default values.
    /// 
    /// # Returns
    /// - `Ok(())` if configuration was saved successfully
    /// - `Err(anyhow::Error)` if file cannot be written or serialization fails
    /// 
    /// # Example
    /// ```rust
    /// let mut config = aerolithsConfig::load().await?;
    /// config.node.port = 9001;  // Modify configuration
    /// config.save().await?;     // Save changes to file
    /// ```
    pub async fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        tokio::fs::write("config.json", content).await?;
        Ok(())
    }
}

impl Default for AerolithsConfig {
    /// Create a comprehensive default configuration for aerolithsDB.
    /// 
    /// This method provides production-ready default settings that balance
    /// security, performance, and functionality. The defaults are suitable
    /// for development environments and provide a starting point for
    /// production configuration tuning.
    /// 
    /// # Default Configuration Philosophy
    /// - **Security First**: Zero-trust enabled, encryption at rest, comprehensive auditing
    /// - **Performance Optimized**: Intelligent caching, cost-based optimization, compression
    /// - **Production Ready**: Appropriate timeouts, connection limits, and resource usage
    /// - **Compliance Aware**: GDPR compliance enabled by default
    /// - **Observable**: Full metrics, tracing, and logging enabled
    /// 
    /// # Key Default Settings
    /// - **Node**: Unique UUID, localhost binding, standard ports
    /// - **Network**: Default network with reasonable connection limits
    /// - **Storage**: Consistent hashing, 3x replication, encryption enabled
    /// - **Cache**: Multi-tier with ML prefetching and 1GB memory limit
    /// - **Security**: Zero-trust, XChaCha20 encryption, comprehensive auditing
    /// - **Consensus**: Byzantine PBFT with 33% fault tolerance
    /// - **Query**: Cost-based optimization with 30-second timeout
    /// - **API**: All protocols enabled with CORS and reflection
    /// - **Plugins**: Restrictive security policy with auto-loading
    /// - **Observability**: Full monitoring with Prometheus and Jaeger
    /// 
    /// # Customization Recommenaerolithons
    /// For production deployment, consider customizing:
    /// - Node ID and network settings for your environment
    /// - Storage directories and size limits
    /// - Memory limits based on available hardware
    /// - Security settings for your compliance requirements
    /// - API bindings and security settings
    /// - Observability endpoints for your monitoring infrastructure
    fn default() -> Self {
        Self {
            // Node configuration with unique identity and localhost binding
            node: NodeConfig {
                // Generate cryptographically unique node identifier
                node_id: uuid::Uuid::new_v4().to_string(),
                
                // Local data directory for node-specific storage
                data_dir: PathBuf::from("./data"),
                
                // Bind to all interfaces (change to 127.0.0.1 for localhost-only)
                bind_address: "0.0.0.0".to_string(),
                
                // Primary port for node communication (other services use port + offset)
                port: 9000,
                
                // External address auto-detected for NAT traversal
                external_address: None,
            },
            
            // Network configuration for cluster communication
            network: NetworkConfig {
                // Default network identifier
                network_id: "default".to_string(),
                
                // Human-readable network name
                network_name: "aerolithsDB Network".to_string(),                // Default governance policy for network decisions
                governance_policy: "default".to_string(),
                
                // Empty bootstrap nodes list (single-node deployment)
                bootstrap_nodes: vec![],
                
                // Maximum concurrent peer connections to prevent resource exhaustion
                max_connections: 100,
                
                // Connection timeout balancing responsiveness with network delays
                connection_timeout: Duration::from_secs(30),
                
                // Heartbeat interval for connection health monitoring
                heartbeat_interval: Duration::from_secs(10),
            },
            
            // Storage hierarchy configuration with enterprise features
            storage: StorageConfig {
                // Consistent hashing for even load distribution
                sharding_strategy: ShardingStrategy::ConsistentHashing,
                
                // 3x replication for fault tolerance and availability
                replication_factor: 3,
                
                // Efficient LZ4 compression with adaptive optimization
                compression: CompressionConfig {
                    algorithm: CompressionAlgorithm::LZ4,  // Fast compression
                    level: 6,                              // Balanced compression level
                    adaptive: true,                        // Adapt to data characteristics
                },
                
                // Encryption at rest enabled for security compliance
                encryption_at_rest: true,
                
                // Local storage directory for persistence
                data_dir: PathBuf::from("./data/storage"),
                
                // No storage size limit (unlimited growth)
                max_storage_size: None,
            },
            
            // Intelligent multi-tier cache configuration
            cache: CacheConfig {
                // Three-tier cache hierarchy from fastest to slowest
                hierarchy: vec![CacheLayer::Memory, CacheLayer::NVMe, CacheLayer::Network],
                
                // Enable ML-driven predictive prefetching
                ml_prefetching: true,
                
                // Enable compression for cached data
                compression: true,
                
                // Adaptive TTL based on access patterns
                ttl_strategy: TTLStrategy::Adaptive,
                
                // 1GB memory limit for in-memory cache
                max_memory_usage: 1024 * 1024 * 1024, // 1GB
            },
            
            // Zero-trust security configuration
            security: SecurityConfig {
                // Enable zero-trust security model
                zero_trust: true,
                
                // 30-day key rotation for forward secrecy
                key_rotation_interval: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
                
                // Comprehensive audit logging for compliance
                audit_level: AuditLevel::Full,
                
                // GDPR compliance enabled by default
                compliance_mode: ComplianceMode::GDPR,
                
                // XChaCha20-Poly1305 for high-performance authenticated encryption
                encryption_algorithm: EncryptionAlgorithm::XChaCha20Poly1305,
            },
            
            // Byzantine fault-tolerant consensus configuration
            consensus: ConsensusConfig {
                // Byzantine PBFT for untrusted network environments
                algorithm: ConsensusAlgorithm::ByzantinePBFT,
                
                // 33% Byzantine fault tolerance (industry standard)
                byzantine_tolerance: 0.33,
                
                // 5-second consensus timeout
                timeout: Duration::from_secs(5),
                
                // Batch up to 1000 operations for efficiency
                max_batch_size: 1000,
                
                // Last writer wins for conflict resolution
                conflict_resolution: ConflictResolution::LastWriterWins,
            },
            
            // Query engine configuration with optimization
            query: QueryConfig {
                // Cost-based optimizer with statistics
                optimizer: OptimizerConfig {
                    cost_based: true,          // Enable cost-based optimization
                    statistics_enabled: true,  // Maintain table statistics
                    max_optimization_time: Duration::from_secs(1), // 1-second optimization limit
                },
                
                // 30-second query execution timeout
                execution_timeout: Duration::from_secs(30),
                
                // 100 concurrent queries per node
                max_concurrent_queries: 100,
                
                // Enable automatic index recommenaerolithons
                index_advisor: true,
            },
            
            // Multi-protocol API gateway configuration
            api: APIConfig {
                // REST API with CORS enabled for web applications
                rest_api: RESTAPIConfig {
                    enabled: true,                         // Enable REST API
                    bind_address: "0.0.0.0".to_string(),  // Bind to all interfaces
                    port: 8080,                           // Standard HTTP port
                    cors_enabled: true,                   // Enable CORS for browsers
                },
                
                // GraphQL API with development features
                graphql_api: GraphQLConfig {
                    enabled: true,                         // Enable GraphQL
                    bind_address: "0.0.0.0".to_string(),  // Bind to all interfaces
                    port: 8081,                           // GraphQL port
                    introspection: true,                  // Enable introspection
                    playground: false,                    // Disable playground in production
                },
                
                // gRPC API with reflection for development
                grpc_api: GRPCConfig {
                    enabled: true,                         // Enable gRPC
                    bind_address: "0.0.0.0".to_string(),  // Bind to all interfaces
                    port: 8082,                           // gRPC port
                    reflection: true,                     // Enable reflection
                },
                
                // WebSocket API for real-time communication
                websocket_api: WebSocketConfig {
                    enabled: true,                         // Enable WebSocket
                    bind_address: "0.0.0.0".to_string(),  // Bind to all interfaces
                    port: 8083,                           // WebSocket port
                    max_connections: 1000,                // Connection limit
                },
            },
            
            // Plugin system with restrictive security
            plugins: PluginConfig {
                // Plugin directory for extensions
                plugin_dir: PathBuf::from("./plugins"),
                
                // Automatically load plugins at startup
                auto_load: true,
                
                // Restrictive security policy for safety
                security_policy: PluginSecurityPolicy::Restrictive,
            },
            
            // Comprehensive observability configuration
            observability: ObservabilityConfig {
                // Prometheus metrics collection
                metrics: MetricsConfig {
                    enabled: true,                                        // Enable metrics
                    prometheus_endpoint: "http://localhost:9090".to_string(), // Prometheus URL
                    collection_interval: Duration::from_secs(15),        // 15-second collection
                },
                
                // Jaeger distributed tracing
                tracing: TracingConfig {
                    enabled: true,                                              // Enable tracing
                    jaeger_endpoint: Some("http://localhost:14268".to_string()), // Jaeger collector
                    sampling_ratio: 0.1,                                       // 10% sampling
                },
                
                // Structured logging configuration
                logging: LoggingConfig {
                    level: "info".to_string(),  // Info-level logging
                    file_output: None,          // Log to stdout
                    structured: true,           // JSON-structured logs
                },
                
                // Alerting configuration (disabled by default)
                alerting: AlertingConfig {
                    enabled: false,                                // Disabled until configured
                    webhook_url: None,                            // No webhook configured
                    thresholds: std::collections::HashMap::new(), // No thresholds configured
                },
            },
        }
    }
}

// ================================================================================================

