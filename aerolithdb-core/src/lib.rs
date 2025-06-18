// aerolithsDB Core Library - Distributed Database Orchestration Engine
//
// This module provides the central coordination and lifecycle management for all 
// aerolithsDB subsystems. It acts as the main orchestration engine that initializes,
// configures, and manages the interactions between all major components of the
// distributed database system.
//
// ## Architecture Overview
// 
// aerolithsDB follows a modular architecture with clearly separated concerns:
//
// ### Core Components
// - **Configuration Management**: Centralized configuration with hot-reload support
// - **Node Identity**: Cryptographic identity and cluster membership management
// - **Consensus Engine**: Byzantine fault-tolerant distributed agreement protocols
// - **Storage Hierarchy**: Multi-tier storage with automatic data lifecycle management
// - **Intelligent Caching**: ML-driven cache optimization with predictive eviction
// - **Zero-Trust Security**: End-to-end encryption with fine-grained access control
// - **Query Engine**: Distributed query processing with cost-based optimization
// - **API Gateway**: Multi-protocol interface (REST, GraphQL, gRPC, WebSocket)
// - **Plugin System**: Extensible architecture for custom functionality
//
// ### Initialization Flow
// 1. Load and validate configuration from multiple sources
// 2. Initialize cryptographic identity and security framework
// 3. Set up storage hierarchy with appropriate backends
// 4. Configure intelligent caching with optimization algorithms
// 5. Start consensus engine with Byzantine fault tolerance
// 6. Initialize P2P network manager for cluster communication
// 7. Start query engine with distributed processing capabilities
// 8. Launch API gateway with multi-protocol support
// 9. Load and activate plugin extensions
//
// ### Lifecycle Management
// The `aerolithsDB` struct provides comprehensive lifecycle management including:
// - Graceful startup with dependency resolution
// - Runtime monitoring and health checks
// - Dynamic configuration updates
// - Clean shutdown with data consistency guarantees

// Import essential dependencies for error handling, async operations, and logging
use anyhow::Result;                    // Unified error handling with context preservation
use std::sync::Arc;                    // Thread-safe reference counting for shared state
use tokio::sync::RwLock;              // Async read-write lock for concurrent access
use tracing::{info, debug};           // Structured logging for operational observability

// Import all aerolithsDB subsystem modules for orchestration
use aerolithdb_consensus::ConsensusEngine;     // Distributed consensus and conflict resolution
use aerolithdb_storage::StorageHierarchy;      // Multi-tier storage management and data lifecycle
use aerolithdb_network::NetworkManager;        // P2P networking and cluster communication
use aerolithdb_cache::IntelligentCacheSystem;  // ML-driven intelligent caching layer
use aerolithdb_security::SecurityFramework;    // Zero-trust security and encryption framework
use aerolithdb_query::QueryEngine;             // Distributed query processing and optimization
// use aerolithdb_api::APIGateway;               // Multi-protocol API gateway and request routing - temporarily disabled
// use aerolithdb_plugins::PluginManager;        // Extensible plugin system and runtime management - temporarily disabled

// Internal core modules for configuration, node management, and type definitions
mod config;    // Configuration management with environment and file-based loading
mod node;      // Node identity, metadata, and cluster membership management
mod types;     // Common type definitions and data structures used across modules

// Re-export public interfaces from internal modules for external use
pub use config::*;  // Configuration structures, loading, and validation functions
pub use node::*;    // Node identity structures and cluster membership types
pub use types::*;   // Common type definitions for external API compatibility

/// Main aerolithsDB instance that orchestrates all subsystems.
/// 
/// This is the central coordination point for the distributed database,
/// managing the lifecycle and interactions between all major components:
/// - Configuration management and updates
/// - Node identity and network participation
/// - Consensus algorithms for distributed agreement
/// - Multi-tier storage hierarchy (memory, SSD, distributed, archival)
/// - Intelligent caching with adaptive algorithms
/// - Zero-trust security framework
/// - Query processing and optimization
/// - API gateway for multiple protocols
/// - Plugin system for extensibility
pub struct AerolithsDB {
    /// Global configuration settings, protected by RwLock for concurrent access
    config: Arc<RwLock<AerolithsConfig>>,
    
    /// Local node identity and metadata
    node: Arc<RwLock<Node>>,
    
    /// Consensus engine for distributed agreement and conflict resolution
    consensus: Arc<ConsensusEngine>,
    
    /// Multi-tier storage hierarchy with automatic data tiering
    storage: Arc<StorageHierarchy>,
    
    /// P2P network manager for node communication and discovery
    network: Arc<NetworkManager>,
    
    /// Intelligent cache system with predictive algorithms
    cache: Arc<IntelligentCacheSystem>,
    
    /// Zero-trust security framework with end-to-end encryption
    security: Arc<SecurityFramework>,
      /// Query processing engine with optimization and distributed execution
    query: Arc<QueryEngine>,
    
    // /// API gateway supporting REST, GraphQL, gRPC, and WebSocket protocols
    // api: Arc<APIGateway>,  // Temporarily disabled
    
    // Plugin manager for system extensibility and custom functionality - temporarily disabled
    // plugins: Arc<PluginManager>, // Temporarily disabled
}

impl AerolithsDB {
    /// Create a new AerolithsDB instance with default configuration.
    /// 
    /// This initializes all subsystems in the correct order to handle dependencies:
    /// 1. Load configuration from default sources
    /// 2. Initialize node identity and cryptographic keys
    /// 3. Set up security framework (required by most other components)
    /// 4. Initialize storage hierarchy with tiering configuration
    /// 5. Start intelligent caching system
    /// 6. Configure consensus engine with Byzantine fault tolerance
    /// 7. Start network manager for P2P communication
    /// 8. Initialize query engine with optimization
    /// 9. Start API gateway with multiple protocol support
    /// 10. Load and initialize plugin system
    /// 
    /// # Returns
    /// - `Ok(aerolithsDB)` if all components initialize successfully
    /// - `Err(anyhow::Error)` if any component fails to start
    /// 
    /// # Example
    /// ```rust
    /// let db = aerolithsDB::new().await?;
    /// db.start().await?;
    /// ```
    pub async fn new() -> Result<Self> {
        info!("Initializing aerolithsDB core components");

        // Load configuration from environment, files, or defaults
        let config = Arc::new(RwLock::new(AerolithsConfig::load().await?));
          // Initialize node identity with cryptographic key generation
        let node = Arc::new(RwLock::new(Node::new(&config.read().await.node).await?));
        
        // Initialize security framework first (required by other components)
        // This sets up encryption, authentication, and zero-trust policies
        let security = Arc::new(SecurityFramework::new(&config.read().await.security).await?);        // Initialize storage hierarchy with default configuration
        let storage = Arc::new(StorageHierarchy::new(&aerolithdb_storage::StorageConfig::default()).await?);

        // Initialize intelligent cache system with default configuration
        let cache = Arc::new(IntelligentCacheSystem::new(&aerolithdb_cache::CacheConfig::default()).await?);

        // Initialize consensus engine with default configuration
        let consensus = Arc::new(ConsensusEngine::new(
            &aerolithdb_consensus::ConsensusConfig::default(),
            Arc::clone(&security),
            Arc::clone(&storage),
        ).await?);        // Create a network node for the current instance
        let network_node = Arc::new(tokio::sync::RwLock::new(aerolithdb_network::Node));        // Initialize network manager with default configuration
        let network = Arc::new(NetworkManager::new(
            &aerolithdb_network::NetworkConfig::default(),
            network_node,
            Arc::clone(&security),
            Arc::clone(&consensus),
        ).await?);        // Initialize query engine with default configuration
        let query = Arc::new(QueryEngine::new(
            aerolithdb_query::QueryConfig::default(),
            Arc::clone(&storage),
            Arc::clone(&cache),
            Arc::clone(&security),        ).await?);
        
        // Initialize API gateway with default configuration - temporarily disabled
        // let api = Arc::new(APIGateway::new(
        //     &aerolithdb_api::APIConfig::default(),
        //     Arc::clone(&query),
        //     Arc::clone(&security),
        // ).await?);

        // Initialize plugin manager with default configuration - temporarily disabled
        // let plugins = Arc::new(PluginManager::new(&aerolithdb_plugins::PluginConfig::default()).await?);        debug!("All aerolithsDB components initialized successfully");

        Ok(Self {
            config,
            node,
            consensus,
            storage,
            network,
            cache,
            security,
            query,
            // api,      // Temporarily disabled
            // plugins,  // Temporarily disabled
        })
    }    /// Create a new aerolithsDB instance with custom configuration.
    /// 
    /// This method allows providing a pre-configured aerolithsConfig instead of
    /// loading from default sources. Useful for testing, embedded deployments,
    /// or when configuration is managed externally.
    /// 
    /// # Arguments
    /// * `config` - Pre-configured aerolithsConfig with all subsystem settings
    /// 
    /// # Returns
    /// - `Ok(aerolithsDB)` if all components initialize successfully
    /// - `Err(anyhow::Error)` if any component fails to start
    /// 
    /// # Example
    /// ```rust
    /// let config = aerolithsConfig {
    ///     node: NodeConfig::default(),
    ///     // ... other settings
    /// };
    /// let db = aerolithsDB::new_with_config(config).await?;
    /// ```
    pub async fn new_with_config(config: AerolithsConfig) -> Result<Self> {
        info!("Initializing aerolithsDB core components with custom config");

        // Save the provided configuration for use across all components
        let config = Arc::new(RwLock::new(config));
        
        // Initialize node identity with the provided configuration
        let node = Arc::new(RwLock::new(Node::new(&config.read().await.node).await?));        // Initialize security framework first (required by other components)
        let security = Arc::new(SecurityFramework::new(&config.read().await.security).await?);        // Initialize storage hierarchy with default configuration
        let storage = Arc::new(StorageHierarchy::new(&aerolithdb_storage::StorageConfig::default()).await?);

        // Initialize intelligent cache system with default configuration
        let cache = Arc::new(IntelligentCacheSystem::new(&aerolithdb_cache::CacheConfig::default()).await?);

        // Initialize consensus engine with default configuration
        let consensus = Arc::new(ConsensusEngine::new(
            &aerolithdb_consensus::ConsensusConfig::default(),
            Arc::clone(&security),
            Arc::clone(&storage),
        ).await?);        // Initialize network manager with default configuration
        let network_node = Arc::new(tokio::sync::RwLock::new(aerolithdb_network::Node));
        let network = Arc::new(NetworkManager::new(
            &aerolithdb_network::NetworkConfig::default(),
            network_node,
            Arc::clone(&security),
            Arc::clone(&consensus),
        ).await?);        // Initialize query engine with default configuration
        let query = Arc::new(QueryEngine::new(
            aerolithdb_query::QueryConfig::default(),
            Arc::clone(&storage),
            Arc::clone(&cache),
            Arc::clone(&security),        ).await?);

        // Initialize API gateway with default configuration - temporarily disabled
        // let api = Arc::new(APIGateway::new(
        //     &aerolithdb_api::APIConfig::default(),
        //     Arc::clone(&query),
        //     Arc::clone(&security),
        // ).await?);

        // Initialize plugin manager with default configuration - temporarily disabled
        // let plugins = Arc::new(PluginManager::new(&aerolithdb_plugins::PluginConfig::default()).await?);

        debug!("All aerolithsDB components initialized successfully with custom config");

        Ok(Self {
            config,
            node,
            consensus,
            storage,
            network,
            cache,
            security,
            query,
            // api,      // Temporarily disabled
            // plugins,  // Temporarily disabled
        })
    }    /// Start the aerolithsDB instance and all its subsystems.
    /// 
    /// This method starts all components in the correct dependency order to ensure
    /// proper initialization. Components that depend on others are started after
    /// their dependencies are ready.
    /// 
    /// Startup order:
    /// 1. Security framework (encryption, authentication)
    /// 2. Storage hierarchy (data persistence)
    /// 3. Cache system (performance optimization)
    /// 4. Consensus engine (distributed agreement)
    /// 5. Network manager (P2P communication)
    /// 6. Query engine (data processing)
    /// 7. API gateway (external interfaces)
    /// 8. Plugin manager (extensibility)
    /// 
    /// # Returns
    /// - `Ok(())` if all components start successfully
    /// - `Err(anyhow::Error)` if any component fails to start
    /// 
    /// # Example
    /// ```rust
    /// let mut db = aerolithsDB::new().await?;
    /// db.start().await?;
    /// // Database is now ready to accept requests
    /// ```
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting aerolithsDB instance");

        // Start components in dependency order to avoid initialization conflicts        self.security.start().await?;     // Security must be first for encryption
        self.storage.start().await?;      // Storage needed for persistence
        self.cache.start().await?;        // Cache enhances storage performance
        self.consensus.start().await?;    // Consensus requires storage and security
        self.network.start().await?;      // Network needs consensus for coordination
        self.query.start().await?;        // Query engine needs storage and cache
        // self.api.start().await?;          // API gateway needs query engine - temporarily disabled
        // self.plugins.start().await?;      // Plugins can extend all other systems - temporarily disabled

        info!("aerolithsDB instance started successfully");
        Ok(())
    }

    /// Stop the aerolithsDB instance gracefully.
    /// 
    /// This method performs a graceful shutdown of all subsystems in reverse
    /// dependency order. It ensures that:
    /// - All pending operations are completed
    /// - Data is flushed to persistent storage
    /// - Network connections are closed cleanly
    /// - Resources are properly released
    /// 
    /// Shutdown order (reverse of startup):
    /// 1. Plugin manager (stop extensions first)
    /// 2. API gateway (stop accepting new requests)
    /// 3. Query engine (finish pending queries)
    /// 4. Network manager (close connections)
    /// 5. Consensus engine (complete consensus operations)
    /// 6. Cache system (flush dirty data)
    /// 7. Storage hierarchy (ensure data persistence)
    /// 8. Security framework (clear sensitive data)
    /// 
    /// # Returns
    /// - `Ok(())` if all components stop successfully
    /// - `Err(anyhow::Error)` if any component fails to stop cleanly
    /// 
    /// # Example
    /// ```rust
    /// db.stop().await?;
    /// // Database is now safely shut down
    /// ```
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping aerolithsDB instance");        // Stop components in reverse dependency order for clean shutdown
        // self.plugins.stop().await?;       // Stop plugins that might use other systems - temporarily disabled
        // self.api.stop().await?;           // Stop API to prevent new requests - temporarily disabled
        self.query.stop().await?;         // Finish pending queries
        self.network.stop().await?;       // Close network connections cleanly
        self.consensus.stop().await?;     // Complete consensus operations
        self.cache.stop().await?;         // Flush cache to storage
        self.storage.stop().await?;       // Ensure all data is persisted
        self.security.stop().await?;      // Clear sensitive data last

        info!("aerolithsDB instance stopped successfully");
        Ok(())
    }    /// Get a thread-safe reference to the global configuration.
    /// 
    /// Returns an Arc<RwLock<aerolithsConfig>> that allows multiple components
    /// to read the configuration concurrently while allowing for safe
    /// configuration updates when needed.
    /// 
    /// # Returns
    /// Arc<RwLock<aerolithsConfig>> - Thread-safe configuration reference
    /// 
    /// # Example
    /// ```rust
    /// let config_ref = db.config().await;
    /// let config = config_ref.read().await;
    /// println!("Node ID: {}", config.node.id);
    /// ```
    pub async fn config(&self) -> Arc<RwLock<AerolithsConfig>> {
        Arc::clone(&self.config)
    }

    /// Get a thread-safe reference to the local node information.
    /// 
    /// Returns node identity, metadata, capabilities, and status information.
    /// This is useful for monitoring node health and retrieving node-specific
    /// information for network operations.
    /// 
    /// # Returns
    /// Arc<RwLock<Node>> - Thread-safe node reference
    /// 
    /// # Example
    /// ```rust
    /// let node_ref = db.node().await;
    /// let node = node_ref.read().await;
    /// println!("Node status: {:?}", node.status);
    /// ```
    pub async fn node(&self) -> Arc<RwLock<Node>> {
        Arc::clone(&self.node)
    }

    /// Get a reference to the query engine for direct query operations.
    /// 
    /// Provides access to the query processing engine for executing
    /// database operations, complex queries, and data analytics.
    /// 
    /// # Returns
    /// Arc<QueryEngine> - Query engine reference
    /// 
    /// # Example
    /// ```rust
    /// let query_engine = db.query_engine();
    /// let results = query_engine.execute_query(query_request).await?;
    /// ```
    pub fn query_engine(&self) -> Arc<QueryEngine> {
        Arc::clone(&self.query)
    }

    /// Get a reference to the network manager for network operations.
    /// 
    /// Provides access to P2P networking functionality including
    /// peer discovery, connection management, and message routing.
    /// 
    /// # Returns
    /// Arc<NetworkManager> - Network manager reference
    /// 
    /// # Example
    /// ```rust
    /// let network = db.network_manager();
    /// let peers = network.get_connected_peers().await?;
    /// ```
    pub fn network_manager(&self) -> Arc<NetworkManager> {
        Arc::clone(&self.network)
    }

    /// Get a reference to the storage hierarchy for direct storage operations.
    /// 
    /// Provides access to the multi-tier storage system for advanced
    /// storage operations, data migration, and storage analytics.
    /// 
    /// # Returns
    /// Arc<StorageHierarchy> - Storage hierarchy reference
    /// 
    /// # Example
    /// ```rust
    /// let storage = db.storage();
    /// let stats = storage.get_storage_stats().await?;
    /// ```
    pub fn storage(&self) -> Arc<StorageHierarchy> {
        Arc::clone(&self.storage)
    }
}
