//! Usage tracking and metering infrastructure
//! 
//! Comprehensive tracking of API calls, storage usage, compute consumption,
//! and network traffic for billing and analytics purposes.

use crate::config::UsageConfig;
use crate::errors::{UsageError, UsageResult};
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use tracing::{info, debug, warn, error};
use uuid::Uuid;

/// Comprehensive usage metrics for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetrics {
    /// Tenant identifier
    pub tenant_id: Uuid,
    
    /// Timestamp when metrics were collected
    pub timestamp: DateTime<Utc>,
    
    /// API call metrics
    pub api_calls: ApiCallMetrics,
    
    /// Storage usage metrics
    pub storage_usage: StorageMetrics,
    
    /// Compute usage metrics
    pub compute_usage: ComputeMetrics,
    
    /// Network usage metrics
    pub network_usage: NetworkMetrics,
    
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

/// API call usage metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiCallMetrics {
    /// Total API calls
    pub total_calls: u64,
    
    /// Read operations (GET, query)
    pub read_operations: u64,
    
    /// Write operations (POST, PUT, DELETE)
    pub write_operations: u64,
    
    /// Query operations
    pub query_operations: u64,
    
    /// Admin operations
    pub admin_operations: u64,
    
    /// WebSocket connections
    pub websocket_connections: u64,
    
    /// GraphQL operations
    pub graphql_operations: u64,
    
    /// gRPC operations
    pub grpc_operations: u64,
    
    /// Failed operations
    pub failed_operations: u64,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
}

/// Storage usage metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageMetrics {
    /// Total storage used in bytes
    pub total_bytes: u64,
    
    /// Number of documents
    pub documents_count: u64,
    
    /// Number of collections
    pub collections_count: u32,
    
    /// Index storage size in bytes
    pub index_size_bytes: u64,
    
    /// Backup storage size in bytes
    pub backup_size_bytes: u64,
    
    /// Hot tier storage in bytes
    pub hot_tier_bytes: u64,
    
    /// Warm tier storage in bytes
    pub warm_tier_bytes: u64,
    
    /// Cold tier storage in bytes
    pub cold_tier_bytes: u64,
    
    /// Archive tier storage in bytes
    pub archive_tier_bytes: u64,
    
    /// Compression ratio
    pub compression_ratio: f64,
}

/// Compute usage metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComputeMetrics {
    /// CPU hours consumed
    pub cpu_hours: f64,
    
    /// Memory hours consumed (GB-hours)
    pub memory_gb_hours: f64,
    
    /// Query processing time in milliseconds
    pub query_processing_ms: u64,
    
    /// Indexing time in milliseconds
    pub indexing_time_ms: u64,
    
    /// Backup processing time in milliseconds
    pub backup_time_ms: u64,
    
    /// Replication processing time in milliseconds
    pub replication_time_ms: u64,
    
    /// Number of background tasks executed
    pub background_tasks: u64,
}

/// Network usage metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkMetrics {
    /// Bytes transferred in (requests)
    pub bytes_in: u64,
    
    /// Bytes transferred out (responses)
    pub bytes_out: u64,
    
    /// Cross-datacenter replication bytes
    pub replication_bytes: u64,
    
    /// Backup transfer bytes
    pub backup_bytes: u64,
    
    /// Number of network requests
    pub request_count: u64,
    
    /// Average request size in bytes
    pub avg_request_size: f64,
    
    /// Average response size in bytes
    pub avg_response_size: f64,
}

/// Aggregated usage statistics over a time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    /// Tenant identifier
    pub tenant_id: Uuid,
    
    /// Start of aggregation period
    pub period_start: DateTime<Utc>,
    
    /// End of aggregation period
    pub period_end: DateTime<Utc>,
    
    /// Aggregated metrics
    pub aggregated_metrics: UsageMetrics,
    
    /// Peak usage during period
    pub peak_metrics: UsageMetrics,
    
    /// Number of data points aggregated
    pub sample_count: u64,
}

/// Usage event for real-time tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    /// Tenant identifier
    pub tenant_id: Uuid,
    
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Event type
    pub event_type: UsageEventType,
    
    /// Event data
    pub data: HashMap<String, serde_json::Value>,
}

/// Types of usage events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageEventType {
    ApiCall {
        method: String,
        endpoint: String,
        response_time_ms: u64,
        success: bool,
    },
    StorageOperation {
        operation: String,
        bytes_affected: u64,
        collection: String,
    },
    ComputeOperation {
        operation: String,
        cpu_time_ms: u64,
        memory_used_mb: u64,
    },
    NetworkTransfer {
        direction: String, // "in" or "out"
        bytes: u64,
        endpoint: String,
    },
}

/// Usage tracking system
pub struct UsageTracker {
    config: UsageConfig,
    db_pool: PgPool,
    event_sender: mpsc::UnboundedSender<UsageEvent>,
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<UsageEvent>>>>,
    current_metrics: Arc<RwLock<HashMap<Uuid, UsageMetrics>>>,
    is_running: Arc<RwLock<bool>>,
}

impl UsageTracker {
    /// Create a new usage tracker
    pub async fn new(config: &UsageConfig) -> Result<Self> {
        info!("📊 Initializing usage tracker");
        
        let db_pool = PgPool::connect(&config.database_url).await?;
        
        // Initialize database schema
        Self::initialize_schema(&db_pool).await?;
        
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let tracker = Self {
            config: config.clone(),
            db_pool,
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
            current_metrics: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        };
        
        info!("✅ Usage tracker initialized");
        Ok(tracker)
    }
    
    /// Initialize database schema for usage tracking
    async fn initialize_schema(pool: &PgPool) -> Result<()> {
        debug!("📊 Initializing usage tracking database schema");
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS usage_metrics (
                id BIGSERIAL PRIMARY KEY,
                tenant_id UUID NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                api_calls JSONB NOT NULL,
                storage_usage JSONB NOT NULL,
                compute_usage JSONB NOT NULL,
                network_usage JSONB NOT NULL,
                custom_metrics JSONB NOT NULL DEFAULT '{}'
            );
            
            CREATE INDEX IF NOT EXISTS idx_usage_metrics_tenant_timestamp 
            ON usage_metrics(tenant_id, timestamp);
            
            CREATE INDEX IF NOT EXISTS idx_usage_metrics_timestamp 
            ON usage_metrics(timestamp);
            
            CREATE TABLE IF NOT EXISTS usage_statistics (
                id BIGSERIAL PRIMARY KEY,
                tenant_id UUID NOT NULL,
                period_start TIMESTAMPTZ NOT NULL,
                period_end TIMESTAMPTZ NOT NULL,
                aggregated_metrics JSONB NOT NULL,
                peak_metrics JSONB NOT NULL,
                sample_count BIGINT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_usage_statistics_tenant_period 
            ON usage_statistics(tenant_id, period_start, period_end);
            
            CREATE TABLE IF NOT EXISTS usage_events (
                id BIGSERIAL PRIMARY KEY,
                tenant_id UUID NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                event_type VARCHAR NOT NULL,
                event_data JSONB NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_usage_events_tenant_timestamp 
            ON usage_events(tenant_id, timestamp);
            
            CREATE INDEX IF NOT EXISTS idx_usage_events_type 
            ON usage_events(event_type);
            "#
        )
        .execute(pool)
        .await?;
        
        debug!("✅ Usage tracking database schema initialized");
        Ok(())
    }
    
    /// Start usage collection and processing
    pub async fn start_collection(&self) -> Result<()> {
        info!("🔄 Starting usage collection");
        
        {
            let mut is_running = self.is_running.write().await;
            if *is_running {
                warn!("Usage collection already running");
                return Ok(());
            }
            *is_running = true;
        }
        
        // Start event processing task
        let event_receiver = {
            let mut receiver_guard = self.event_receiver.write().await;
            receiver_guard.take()
        };
        
        if let Some(mut event_receiver) = event_receiver {
            let current_metrics = Arc::clone(&self.current_metrics);
            let is_running = Arc::clone(&self.is_running);
            
            tokio::spawn(async move {
                while let Some(event) = event_receiver.recv().await {
                    let running = { *is_running.read().await };
                    if !running {
                        break;
                    }
                    
                    // Process usage event
                    Self::process_usage_event(&current_metrics, event).await;
                }
            });
        }
        
        // Start metrics collection task
        if self.config.enabled {
            let config = self.config.clone();
            let db_pool = self.db_pool.clone();
            let current_metrics = Arc::clone(&self.current_metrics);
            let is_running = Arc::clone(&self.is_running);
            
            tokio::spawn(async move {
                let mut interval = interval(config.collection_interval);
                
                loop {
                    interval.tick().await;
                    
                    let running = { *is_running.read().await };
                    if !running {
                        break;
                    }
                    
                    // Store current metrics to database
                    if let Err(e) = Self::store_metrics(&db_pool, &current_metrics).await {
                        error!("Failed to store usage metrics: {}", e);
                    }
                }
            });
        }
        
        // Start aggregation task
        if self.config.enabled {
            let config = self.config.clone();
            let db_pool = self.db_pool.clone();
            let is_running = Arc::clone(&self.is_running);
            
            tokio::spawn(async move {
                let mut interval = interval(config.aggregation_interval);
                
                loop {
                    interval.tick().await;
                    
                    let running = { *is_running.read().await };
                    if !running {
                        break;
                    }
                    
                    // Aggregate metrics
                    if let Err(e) = Self::aggregate_metrics(&db_pool, &config).await {
                        error!("Failed to aggregate usage metrics: {}", e);
                    }
                }
            });
        }
        
        info!("✅ Usage collection started");
        Ok(())
    }
    
    /// Stop usage collection
    pub async fn stop_collection(&self) -> Result<()> {
        info!("🛑 Stopping usage collection");
        
        {
            let mut is_running = self.is_running.write().await;
            *is_running = false;
        }
        
        info!("✅ Usage collection stopped");
        Ok(())
    }
    
    /// Record a usage event
    pub async fn record_event(&self, event: UsageEvent) -> UsageResult<()> {
        // Send event to processing channel
        self.event_sender.send(event.clone()).map_err(|e| UsageError::CollectionFailed {
            message: format!("Failed to send usage event: {}", e),
        })?;
        
        // Store event to database for audit trail
        if self.config.enabled {
            sqlx::query(
                "INSERT INTO usage_events (tenant_id, timestamp, event_type, event_data) VALUES ($1, $2, $3, $4)"
            )
            .bind(event.tenant_id)
            .bind(event.timestamp)
            .bind(serde_json::to_string(&event.event_type)?)
            .bind(serde_json::to_value(&event.data)?)
            .execute(&self.db_pool)
            .await
            .map_err(|e| UsageError::StorageFailed {
                message: format!("Failed to store usage event: {}", e),
            })?;
        }
        
        Ok(())
    }
    
    /// Process a usage event and update current metrics
    async fn process_usage_event(
        current_metrics: &Arc<RwLock<HashMap<Uuid, UsageMetrics>>>,
        event: UsageEvent,
    ) {
        let mut metrics_guard = current_metrics.write().await;
        let metrics = metrics_guard.entry(event.tenant_id).or_insert_with(|| UsageMetrics {
            tenant_id: event.tenant_id,
            timestamp: Utc::now(),
            api_calls: ApiCallMetrics::default(),
            storage_usage: StorageMetrics::default(),
            compute_usage: ComputeMetrics::default(),
            network_usage: NetworkMetrics::default(),
            custom_metrics: HashMap::new(),
        });
        
        // Update metrics based on event type
        match event.event_type {
            UsageEventType::ApiCall { method, response_time_ms, success, .. } => {
                metrics.api_calls.total_calls += 1;
                if !success {
                    metrics.api_calls.failed_operations += 1;
                }
                
                match method.to_uppercase().as_str() {
                    "GET" => metrics.api_calls.read_operations += 1,
                    "POST" | "PUT" | "DELETE" => metrics.api_calls.write_operations += 1,
                    _ => {},
                }
                
                // Update average response time
                let total_time = metrics.api_calls.avg_response_time_ms * (metrics.api_calls.total_calls - 1) as f64;
                metrics.api_calls.avg_response_time_ms = (total_time + response_time_ms as f64) / metrics.api_calls.total_calls as f64;
            },
            UsageEventType::StorageOperation { bytes_affected, .. } => {
                metrics.storage_usage.total_bytes = metrics.storage_usage.total_bytes.saturating_add(bytes_affected);
            },
            UsageEventType::ComputeOperation { cpu_time_ms, memory_used_mb, .. } => {
                metrics.compute_usage.cpu_hours += cpu_time_ms as f64 / (1000.0 * 3600.0);
                metrics.compute_usage.memory_gb_hours += memory_used_mb as f64 / 1024.0 / 3600.0;
            },
            UsageEventType::NetworkTransfer { direction, bytes, .. } => {
                match direction.as_str() {
                    "in" => metrics.network_usage.bytes_in += bytes,
                    "out" => metrics.network_usage.bytes_out += bytes,
                    _ => {},
                }
                metrics.network_usage.request_count += 1;
            },
        }
        
        metrics.timestamp = event.timestamp;
    }
    
    /// Store current metrics to database
    async fn store_metrics(
        db_pool: &PgPool,
        current_metrics: &Arc<RwLock<HashMap<Uuid, UsageMetrics>>>,
    ) -> Result<()> {
        let metrics_snapshot = {
            let metrics_guard = current_metrics.read().await;
            metrics_guard.clone()
        };
        
        for (tenant_id, metrics) in metrics_snapshot {
            sqlx::query(
                r#"
                INSERT INTO usage_metrics (
                    tenant_id, timestamp, api_calls, storage_usage, 
                    compute_usage, network_usage, custom_metrics
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#
            )
            .bind(tenant_id)
            .bind(metrics.timestamp)
            .bind(serde_json::to_value(&metrics.api_calls)?)
            .bind(serde_json::to_value(&metrics.storage_usage)?)
            .bind(serde_json::to_value(&metrics.compute_usage)?)
            .bind(serde_json::to_value(&metrics.network_usage)?)
            .bind(serde_json::to_value(&metrics.custom_metrics)?)
            .execute(db_pool)
            .await?;
        }
        
        Ok(())
    }
    
    /// Aggregate metrics for billing purposes
    async fn aggregate_metrics(db_pool: &PgPool, config: &UsageConfig) -> Result<()> {
        let end_time = Utc::now();
        let start_time = end_time - config.aggregation_interval;
        
        debug!("🔄 Aggregating usage metrics from {} to {}", start_time, end_time);
        
        // Get all tenants with metrics in this period
        let tenant_rows = sqlx::query(
            "SELECT DISTINCT tenant_id FROM usage_metrics WHERE timestamp >= $1 AND timestamp < $2"
        )
        .bind(start_time)
        .bind(end_time)
        .fetch_all(db_pool)
        .await?;
        
        for tenant_row in tenant_rows {
            let tenant_id: Uuid = tenant_row.try_get("tenant_id")?;
            
            // Aggregate metrics for this tenant
            let metrics_rows = sqlx::query(
                "SELECT * FROM usage_metrics WHERE tenant_id = $1 AND timestamp >= $2 AND timestamp < $3 ORDER BY timestamp"
            )
            .bind(tenant_id)
            .bind(start_time)
            .bind(end_time)
            .fetch_all(db_pool)
            .await?;
            
            if !metrics_rows.is_empty() {
                let aggregated = Self::aggregate_metrics_data(&metrics_rows)?;
                
                // Store aggregated statistics
                sqlx::query(
                    r#"
                    INSERT INTO usage_statistics (
                        tenant_id, period_start, period_end, 
                        aggregated_metrics, peak_metrics, sample_count
                    ) VALUES ($1, $2, $3, $4, $5, $6)
                    "#
                )
                .bind(tenant_id)
                .bind(start_time)
                .bind(end_time)
                .bind(serde_json::to_value(&aggregated.aggregated_metrics)?)
                .bind(serde_json::to_value(&aggregated.peak_metrics)?)
                .bind(aggregated.sample_count as i64)
                .execute(db_pool)
                .await?;
            }
        }
        
        debug!("✅ Usage metrics aggregation completed");
        Ok(())
    }
    
    /// Aggregate metrics data from database rows
    fn aggregate_metrics_data(rows: &[sqlx::postgres::PgRow]) -> Result<UsageStatistics> {
        if rows.is_empty() {
            return Err(anyhow::anyhow!("No metrics data to aggregate"));
        }
        
        let tenant_id: Uuid = rows[0].try_get("tenant_id")?;
        let period_start: DateTime<Utc> = rows[0].try_get("timestamp")?;
        let period_end: DateTime<Utc> = rows.last().unwrap().try_get("timestamp")?;
        
        let mut aggregated = UsageMetrics {
            tenant_id,
            timestamp: period_end,
            api_calls: ApiCallMetrics::default(),
            storage_usage: StorageMetrics::default(),
            compute_usage: ComputeMetrics::default(),
            network_usage: NetworkMetrics::default(),
            custom_metrics: HashMap::new(),
        };
        
        let mut peak = aggregated.clone();
        
        for row in rows {
            let api_calls: ApiCallMetrics = serde_json::from_value(row.try_get("api_calls")?)?;
            let storage: StorageMetrics = serde_json::from_value(row.try_get("storage_usage")?)?;
            let compute: ComputeMetrics = serde_json::from_value(row.try_get("compute_usage")?)?;
            let network: NetworkMetrics = serde_json::from_value(row.try_get("network_usage")?)?;
            
            // Aggregate totals
            aggregated.api_calls.total_calls += api_calls.total_calls;
            aggregated.api_calls.read_operations += api_calls.read_operations;
            aggregated.api_calls.write_operations += api_calls.write_operations;
            aggregated.storage_usage.total_bytes = aggregated.storage_usage.total_bytes.max(storage.total_bytes);
            aggregated.compute_usage.cpu_hours += compute.cpu_hours;
            aggregated.network_usage.bytes_in += network.bytes_in;
            aggregated.network_usage.bytes_out += network.bytes_out;
            
            // Track peaks
            peak.api_calls.total_calls = peak.api_calls.total_calls.max(api_calls.total_calls);
            peak.storage_usage.total_bytes = peak.storage_usage.total_bytes.max(storage.total_bytes);
            peak.compute_usage.cpu_hours = peak.compute_usage.cpu_hours.max(compute.cpu_hours);
            peak.network_usage.bytes_in = peak.network_usage.bytes_in.max(network.bytes_in);
            peak.network_usage.bytes_out = peak.network_usage.bytes_out.max(network.bytes_out);
        }
        
        Ok(UsageStatistics {
            tenant_id,
            period_start,
            period_end,
            aggregated_metrics: aggregated,
            peak_metrics: peak,
            sample_count: rows.len() as u64,
        })
    }
    
    /// Get usage statistics for a tenant within a time period
    pub async fn get_usage_statistics(
        &self,
        tenant_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> UsageResult<Vec<UsageStatistics>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM usage_statistics 
            WHERE tenant_id = $1 AND period_start >= $2 AND period_end <= $3 
            ORDER BY period_start
            "#
        )
        .bind(tenant_id)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| UsageError::QueryFailed {
            message: format!("Failed to query usage statistics: {}", e),
        })?;
        
        let mut statistics = Vec::new();
        for row in rows {
            let stat = UsageStatistics {
                tenant_id: row.try_get("tenant_id")?,
                period_start: row.try_get("period_start")?,
                period_end: row.try_get("period_end")?,
                aggregated_metrics: serde_json::from_value(row.try_get("aggregated_metrics")?)?,
                peak_metrics: serde_json::from_value(row.try_get("peak_metrics")?)?,
                sample_count: row.try_get::<i64, _>("sample_count")? as u64,
            };
            statistics.push(stat);
        }
        
        Ok(statistics)
    }
    
    /// Get current usage metrics for a tenant
    pub async fn get_current_usage(&self, tenant_id: Uuid) -> UsageResult<Option<UsageMetrics>> {
        let metrics_guard = self.current_metrics.read().await;
        Ok(metrics_guard.get(&tenant_id).cloned())
    }
}
