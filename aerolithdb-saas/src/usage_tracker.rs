//! Real-time usage tracking implementation
//! 
//! Provides live tracking of tenant usage metrics including API calls,
//! storage consumption, and resource utilization.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, debug, warn, error};
use anyhow::Result;

use crate::config::UsageConfig;
use crate::errors::{UsageError, UsageResult};
use crate::usage::*;

/// Real-time usage tracker
pub struct UsageTracker {
    /// Configuration
    config: UsageConfig,
    
    /// In-memory usage cache
    metrics_cache: Arc<RwLock<HashMap<Uuid, UsageMetrics>>>,
    
    /// Background task handles
    background_tasks: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
    
    /// Event sender for usage events
    event_sender: mpsc::UnboundedSender<UsageEvent>,
    
    /// Event receiver for processing
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<UsageEvent>>>>,
}

/// Usage event for real-time tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    /// Tenant ID
    pub tenant_id: Uuid,
    
    /// Event type
    pub event_type: UsageEventType,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Event metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of usage events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageEventType {
    /// API call event
    ApiCall {
        method: String,
        endpoint: String,
        response_time_ms: u64,
        status_code: u16,
        bytes_sent: u64,
        bytes_received: u64,
    },
    
    /// Storage operation event
    StorageOperation {
        operation: String,
        collection: String,
        bytes_written: u64,
        bytes_read: u64,
        documents_affected: u64,
    },
    
    /// Query execution event
    QueryExecution {
        query_type: String,
        execution_time_ms: u64,
        documents_scanned: u64,
        documents_returned: u64,
        bytes_transferred: u64,
    },
    
    /// Network operation event
    NetworkOperation {
        operation: String,
        bytes_sent: u64,
        bytes_received: u64,
        duration_ms: u64,
    },
    
    /// Custom event
    Custom {
        event_name: String,
        value: f64,
        unit: String,
    },
}

/// Live usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveUsageStats {
    /// Tenant ID
    pub tenant_id: Uuid,
    
    /// Current period start
    pub period_start: DateTime<Utc>,
    
    /// Last updated
    pub last_updated: DateTime<Utc>,
    
    /// API calls in current period
    pub api_calls_count: u64,
    
    /// Storage bytes used
    pub storage_bytes: u64,
    
    /// Compute time used (milliseconds)
    pub compute_time_ms: u64,
    
    /// Network bytes transferred
    pub network_bytes: u64,
    
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

impl UsageTracker {
    /// Create new usage tracker
    pub fn new(config: UsageConfig) -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Ok(Self {
            config,
            metrics_cache: Arc::new(RwLock::new(HashMap::new())),
            background_tasks: Arc::new(RwLock::new(Vec::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        })
    }
    
    /// Start the usage tracker
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting usage tracker");
        
        // Start event processing task
        self.start_event_processor().await?;
        
        // Start metrics aggregation task
        self.start_metrics_aggregator().await?;
        
        // Start cleanup task
        self.start_cleanup_task().await?;
        
        info!("✅ Usage tracker started successfully");
        Ok(())
    }
    
    /// Stop the usage tracker
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping usage tracker");
        
        // Cancel all background tasks
        let mut tasks = self.background_tasks.write().await;
        for task in tasks.drain(..) {
            task.abort();
        }
        
        info!("✅ Usage tracker stopped");
        Ok(())
    }
    
    /// Track a usage event
    pub async fn track_event(&self, event: UsageEvent) -> Result<()> {
        debug!("📊 Tracking usage event: {:?}", event.event_type);
        
        self.event_sender.send(event)
            .map_err(|e| UsageError::TrackingFailed(format!("Failed to send event: {}", e)))?;
        
        Ok(())
    }
    
    /// Track API call
    pub async fn track_api_call(
        &self,
        tenant_id: Uuid,
        method: &str,
        endpoint: &str,
        response_time_ms: u64,
        status_code: u16,
        bytes_sent: u64,
        bytes_received: u64,
    ) -> Result<()> {
        let event = UsageEvent {
            tenant_id,
            event_type: UsageEventType::ApiCall {
                method: method.to_string(),
                endpoint: endpoint.to_string(),
                response_time_ms,
                status_code,
                bytes_sent,
                bytes_received,
            },
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        
        self.track_event(event).await
    }
    
    /// Track storage operation
    pub async fn track_storage_operation(
        &self,
        tenant_id: Uuid,
        operation: &str,
        collection: &str,
        bytes_written: u64,
        bytes_read: u64,
        documents_affected: u64,
    ) -> Result<()> {
        let event = UsageEvent {
            tenant_id,
            event_type: UsageEventType::StorageOperation {
                operation: operation.to_string(),
                collection: collection.to_string(),
                bytes_written,
                bytes_read,
                documents_affected,
            },
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        
        self.track_event(event).await
    }
    
    /// Get current usage stats for tenant
    pub async fn get_current_usage(&self, tenant_id: Uuid) -> Result<Option<LiveUsageStats>> {
        let cache = self.metrics_cache.read().await;
        
        if let Some(metrics) = cache.get(&tenant_id) {
            let stats = LiveUsageStats {
                tenant_id,
                period_start: metrics.timestamp,
                last_updated: Utc::now(),
                api_calls_count: metrics.api_calls.total_calls,
                storage_bytes: metrics.storage_usage.total_bytes,
                compute_time_ms: metrics.compute_usage.total_compute_ms,
                network_bytes: metrics.network_usage.total_bytes,
                custom_metrics: metrics.custom_metrics.clone(),
            };
            Ok(Some(stats))
        } else {
            Ok(None)
        }
    }
    
    /// Get usage history for tenant
    pub async fn get_usage_history(
        &self,
        tenant_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<UsageMetrics>> {
        // In a real implementation, this would query the database
        // For now, return empty history
        debug!("📈 Getting usage history for tenant {} from {} to {}", 
               tenant_id, start_date, end_date);
        Ok(Vec::new())
    }
    
    /// Start event processing task
    async fn start_event_processor(&self) -> Result<()> {
        let mut receiver = self.event_receiver.write().await
            .take()
            .ok_or_else(|| UsageError::InternalError("Event receiver already taken".to_string()))?;
        
        let metrics_cache = Arc::clone(&self.metrics_cache);
        let config = self.config.clone();
        
        let task = tokio::spawn(async move {
            info!("🔄 Starting usage event processor");
            
            while let Some(event) = receiver.recv().await {
                if let Err(e) = Self::process_usage_event(event, &metrics_cache, &config).await {
                    error!("❌ Failed to process usage event: {}", e);
                }
            }
            
            info!("🛑 Usage event processor stopped");
        });
        
        self.background_tasks.write().await.push(task);
        Ok(())
    }
    
    /// Start metrics aggregation task
    async fn start_metrics_aggregator(&self) -> Result<()> {
        let metrics_cache = Arc::clone(&self.metrics_cache);
        let config = self.config.clone();
        
        let task = tokio::spawn(async move {
            info!("🔄 Starting metrics aggregator");
            
            let mut interval = interval(Duration::from_secs(config.aggregation_interval_seconds));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::aggregate_metrics(&metrics_cache).await {
                    error!("❌ Failed to aggregate metrics: {}", e);
                }
            }
        });
        
        self.background_tasks.write().await.push(task);
        Ok(())
    }
    
    /// Start cleanup task
    async fn start_cleanup_task(&self) -> Result<()> {
        let metrics_cache = Arc::clone(&self.metrics_cache);
        let config = self.config.clone();
        
        let task = tokio::spawn(async move {
            info!("🔄 Starting usage cleanup task");
            
            let mut interval = interval(Duration::from_secs(config.cleanup_interval_seconds));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::cleanup_old_metrics(&metrics_cache, &config).await {
                    error!("❌ Failed to cleanup old metrics: {}", e);
                }
            }
        });
        
        self.background_tasks.write().await.push(task);
        Ok(())
    }
    
    /// Process a single usage event
    async fn process_usage_event(
        event: UsageEvent,
        metrics_cache: &Arc<RwLock<HashMap<Uuid, UsageMetrics>>>,
        _config: &UsageConfig,
    ) -> Result<()> {
        let mut cache = metrics_cache.write().await;
        
        let metrics = cache.entry(event.tenant_id).or_insert_with(|| {
            UsageMetrics {
                tenant_id: event.tenant_id,
                timestamp: event.timestamp,
                api_calls: ApiCallMetrics::default(),
                storage_usage: StorageMetrics::default(),
                compute_usage: ComputeMetrics::default(),
                network_usage: NetworkMetrics::default(),
                custom_metrics: HashMap::new(),
            }
        });
        
        // Update metrics based on event type
        match event.event_type {
            UsageEventType::ApiCall { response_time_ms, bytes_sent, bytes_received, .. } => {
                metrics.api_calls.total_calls += 1;
                metrics.api_calls.read_operations += 1; // Simplified
                metrics.network_usage.total_bytes += bytes_sent + bytes_received;
                metrics.compute_usage.total_compute_ms += response_time_ms;
            },
            
            UsageEventType::StorageOperation { bytes_written, bytes_read, documents_affected, .. } => {
                metrics.storage_usage.total_bytes += bytes_written;
                metrics.storage_usage.documents_count += documents_affected;
                metrics.network_usage.total_bytes += bytes_written + bytes_read;
            },
            
            UsageEventType::QueryExecution { execution_time_ms, documents_scanned, bytes_transferred, .. } => {
                metrics.compute_usage.total_compute_ms += execution_time_ms;
                metrics.compute_usage.query_operations += 1;
                metrics.network_usage.total_bytes += bytes_transferred;
                // Update storage metrics for scanned documents
                metrics.storage_usage.documents_count = 
                    metrics.storage_usage.documents_count.max(documents_scanned);
            },
            
            UsageEventType::NetworkOperation { bytes_sent, bytes_received, duration_ms, .. } => {
                metrics.network_usage.total_bytes += bytes_sent + bytes_received;
                metrics.compute_usage.total_compute_ms += duration_ms;
            },
            
            UsageEventType::Custom { event_name, value, .. } => {
                *metrics.custom_metrics.entry(event_name).or_insert(0.0) += value;
            },
        }
        
        debug!("📊 Updated usage metrics for tenant {}", event.tenant_id);
        Ok(())
    }
    
    /// Aggregate metrics periodically
    async fn aggregate_metrics(
        _metrics_cache: &Arc<RwLock<HashMap<Uuid, UsageMetrics>>>,
    ) -> Result<()> {
        // In a real implementation, this would:
        // 1. Calculate rolling averages
        // 2. Update persistent storage
        // 3. Generate billing events
        // 4. Update quota usage
        
        debug!("📊 Aggregating usage metrics");
        Ok(())
    }
    
    /// Cleanup old metrics
    async fn cleanup_old_metrics(
        metrics_cache: &Arc<RwLock<HashMap<Uuid, UsageMetrics>>>,
        config: &UsageConfig,
    ) -> Result<()> {
        let cutoff_time = Utc::now() - chrono::Duration::seconds(config.retention_seconds as i64);
        let mut cache = metrics_cache.write().await;
        
        let initial_count = cache.len();
        cache.retain(|_, metrics| metrics.timestamp > cutoff_time);
        let final_count = cache.len();
        
        if initial_count > final_count {
            debug!("🧹 Cleaned up {} old usage metrics entries", initial_count - final_count);
        }
        
        Ok(())
    }
}

/// Usage tracker factory
pub struct UsageTrackerFactory;

impl UsageTrackerFactory {
    /// Create a new usage tracker with default configuration
    pub fn create_default() -> Result<UsageTracker> {
        let config = UsageConfig {
            enabled: true,
            aggregation_interval_seconds: 60,
            cleanup_interval_seconds: 3600,
            retention_seconds: 86400 * 7, // 7 days
            batch_size: 1000,
            max_events_per_second: 10000,
        };
        
        UsageTracker::new(config)
    }
    
    /// Create a new usage tracker with custom configuration
    pub fn create_with_config(config: UsageConfig) -> Result<UsageTracker> {
        UsageTracker::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;
    
    #[tokio::test]
    async fn test_usage_tracker_creation() {
        let tracker = UsageTrackerFactory::create_default().unwrap();
        assert!(tracker.config.enabled);
    }
    
    #[tokio::test]
    async fn test_api_call_tracking() {
        let tracker = UsageTrackerFactory::create_default().unwrap();
        tracker.start().await.unwrap();
        
        let tenant_id = Uuid::new_v4();
        
        // Track an API call
        tracker.track_api_call(
            tenant_id,
            "GET",
            "/api/v1/documents",
            150,
            200,
            1024,
            2048,
        ).await.unwrap();
        
        // Allow some time for processing
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Check usage
        let usage = tracker.get_current_usage(tenant_id).await.unwrap();
        assert!(usage.is_some());
        
        let usage = usage.unwrap();
        assert_eq!(usage.tenant_id, tenant_id);
        assert_eq!(usage.api_calls_count, 1);
        
        tracker.stop().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_storage_operation_tracking() {
        let tracker = UsageTrackerFactory::create_default().unwrap();
        tracker.start().await.unwrap();
        
        let tenant_id = Uuid::new_v4();
        
        // Track storage operation
        tracker.track_storage_operation(
            tenant_id,
            "write",
            "documents",
            1024,
            0,
            1,
        ).await.unwrap();
        
        // Allow some time for processing
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Check usage
        let usage = tracker.get_current_usage(tenant_id).await.unwrap();
        assert!(usage.is_some());
        
        let usage = usage.unwrap();
        assert_eq!(usage.storage_bytes, 1024);
        
        tracker.stop().await.unwrap();
    }
}
