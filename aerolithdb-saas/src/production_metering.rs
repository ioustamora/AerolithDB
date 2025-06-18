//! Production-Scale Usage Metering System
//! 
//! High-performance, scalable usage tracking system for monitoring API calls,
//! storage usage, compute consumption, and custom metrics across all tenants.

use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration as TokioDuration};
use uuid::Uuid;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};

use crate::tenant::TenantId;
use crate::subscription::{Subscription, SubscriptionManager};
use crate::errors::SaaSError;

/// High-performance usage event for real-time processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub tenant_id: TenantId,
    pub subscription_id: Option<Uuid>,
    pub event_type: UsageEventType,
    pub resource: String,
    pub quantity: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub correlation_id: Option<String>,
}

/// Types of usage events tracked
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UsageEventType {
    /// API operation (read, write, query)
    ApiCall {
        endpoint: String,
        method: String,
        status_code: u16,
        response_time_ms: u64,
    },
    /// Storage operations
    Storage {
        operation: StorageOperation,
        data_size_bytes: u64,
        tier: String,
    },
    /// Compute usage
    Compute {
        operation: ComputeOperation,
        cpu_time_ms: u64,
        memory_mb: u64,
    },
    /// Network bandwidth
    Bandwidth {
        direction: BandwidthDirection,
        bytes_transferred: u64,
        region: String,
    },
    /// Custom usage metric
    Custom {
        metric_name: String,
        value: f64,
        dimensions: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageOperation {
    Read,
    Write,
    Delete,
    List,
    Copy,
    Backup,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComputeOperation {
    Query,
    Index,
    Backup,
    Replication,
    Analytics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BandwidthDirection {
    Ingress,
    Egress,
    Internal,
}

/// Real-time usage aggregation
#[derive(Debug, Default)]
pub struct UsageAggregation {
    pub api_calls: AtomicU64,
    pub storage_reads: AtomicU64,
    pub storage_writes: AtomicU64,
    pub storage_bytes: AtomicU64,
    pub compute_time_ms: AtomicU64,
    pub bandwidth_bytes: AtomicU64,
    pub custom_metrics: Arc<DashMap<String, AtomicU64>>,
    pub last_updated: Arc<RwLock<DateTime<Utc>>>,
}

impl UsageAggregation {
    pub fn new() -> Self {
        Self {
            api_calls: AtomicU64::new(0),
            storage_reads: AtomicU64::new(0),
            storage_writes: AtomicU64::new(0),
            storage_bytes: AtomicU64::new(0),
            compute_time_ms: AtomicU64::new(0),
            bandwidth_bytes: AtomicU64::new(0),
            custom_metrics: Arc::new(DashMap::new()),
            last_updated: Arc::new(RwLock::new(Utc::now())),
        }
    }

    pub async fn add_event(&self, event: &UsageEvent) {
        match &event.event_type {
            UsageEventType::ApiCall { .. } => {
                self.api_calls.fetch_add(1, Ordering::Relaxed);
            }
            UsageEventType::Storage { operation, data_size_bytes, .. } => {
                match operation {
                    StorageOperation::Read => self.storage_reads.fetch_add(1, Ordering::Relaxed),
                    StorageOperation::Write => self.storage_writes.fetch_add(1, Ordering::Relaxed),
                    _ => 0,
                };
                self.storage_bytes.fetch_add(*data_size_bytes, Ordering::Relaxed);
            }
            UsageEventType::Compute { cpu_time_ms, .. } => {
                self.compute_time_ms.fetch_add(*cpu_time_ms, Ordering::Relaxed);
            }
            UsageEventType::Bandwidth { bytes_transferred, .. } => {
                self.bandwidth_bytes.fetch_add(*bytes_transferred, Ordering::Relaxed);
            }
            UsageEventType::Custom { metric_name, value, .. } => {
                if let Some(metric) = self.custom_metrics.get(metric_name) {
                    metric.fetch_add(*value as u64, Ordering::Relaxed);
                } else {
                    self.custom_metrics.insert(metric_name.clone(), AtomicU64::new(*value as u64));
                }
            }
        }

        *self.last_updated.write().await = Utc::now();
    }

    pub fn get_snapshot(&self) -> UsageSnapshot {
        let custom_metrics: HashMap<String, u64> = self.custom_metrics
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().load(Ordering::Relaxed)))
            .collect();

        UsageSnapshot {
            api_calls: self.api_calls.load(Ordering::Relaxed),
            storage_reads: self.storage_reads.load(Ordering::Relaxed),
            storage_writes: self.storage_writes.load(Ordering::Relaxed),
            storage_bytes: self.storage_bytes.load(Ordering::Relaxed),
            compute_time_ms: self.compute_time_ms.load(Ordering::Relaxed),
            bandwidth_bytes: self.bandwidth_bytes.load(Ordering::Relaxed),
            custom_metrics,
        }
    }
}

/// Immutable usage snapshot for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub api_calls: u64,
    pub storage_reads: u64,
    pub storage_writes: u64,
    pub storage_bytes: u64,
    pub compute_time_ms: u64,
    pub bandwidth_bytes: u64,
    pub custom_metrics: HashMap<String, u64>,
}

/// Production-scale usage metering engine
pub struct ProductionUsageMeter {
    // Real-time aggregations by tenant
    tenant_usage: Arc<DashMap<TenantId, UsageAggregation>>,
    
    // High-throughput event processing
    event_sender: mpsc::UnboundedSender<UsageEvent>,
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<UsageEvent>>>>,
    
    // Batch processing for efficiency
    batch_size: usize,
    batch_interval_seconds: u64,
    
    // Persistent storage for historical data
    historical_storage: Arc<dyn UsageStorage + Send + Sync>,
    
    // Subscription manager for limit checking
    subscription_manager: Arc<SubscriptionManager>,
    
    // Control flags
    is_running: AtomicBool,
    enable_real_time_alerts: AtomicBool,
}

impl ProductionUsageMeter {
    pub fn new(
        subscription_manager: Arc<SubscriptionManager>,
        storage: Arc<dyn UsageStorage + Send + Sync>,
    ) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        Self {
            tenant_usage: Arc::new(DashMap::new()),
            event_sender: sender,
            event_receiver: Arc::new(RwLock::new(Some(receiver))),
            batch_size: 1000,
            batch_interval_seconds: 60,
            historical_storage: storage,
            subscription_manager,
            is_running: AtomicBool::new(false),
            enable_real_time_alerts: AtomicBool::new(true),
        }
    }

    /// Start the usage metering engine
    pub async fn start(&self) -> Result<()> {
        if self.is_running.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.is_running.store(true, Ordering::Relaxed);

        // Start event processing task
        self.start_event_processor().await?;
        
        // Start periodic aggregation task
        self.start_aggregation_task().await?;
        
        // Start limit checking task
        self.start_limit_checker().await?;
        
        // Start cleanup task
        self.start_cleanup_task().await?;

        tracing::info!("Production usage meter started");
        Ok(())
    }

    /// Stop the usage metering engine
    pub async fn stop(&self) -> Result<()> {
        self.is_running.store(false, Ordering::Relaxed);
        tracing::info!("Production usage meter stopped");
        Ok(())
    }

    /// Record a usage event (high-performance, non-blocking)
    pub fn record_event(&self, event: UsageEvent) -> Result<()> {
        if !self.is_running.load(Ordering::Relaxed) {
            return Err(SaaSError::MeteringNotRunning.into());
        }

        self.event_sender.send(event)
            .map_err(|_| SaaSError::MeteringChannelClosed)?;
        
        Ok(())
    }

    /// Get current usage for a tenant
    pub async fn get_current_usage(&self, tenant_id: TenantId) -> Option<UsageSnapshot> {
        self.tenant_usage.get(&tenant_id)
            .map(|usage| usage.get_snapshot())
    }

    /// Get usage for multiple tenants
    pub async fn get_bulk_usage(&self, tenant_ids: &[TenantId]) -> HashMap<TenantId, UsageSnapshot> {
        let mut results = HashMap::new();
        
        for tenant_id in tenant_ids {
            if let Some(usage) = self.get_current_usage(*tenant_id).await {
                results.insert(*tenant_id, usage);
            }
        }
        
        results
    }

    /// Get historical usage data
    pub async fn get_historical_usage(
        &self,
        tenant_id: TenantId,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<HistoricalUsage>> {
        self.historical_storage.get_usage_history(tenant_id, start_date, end_date).await
    }

    /// Get usage trends and analytics
    pub async fn get_usage_analytics(
        &self,
        tenant_id: TenantId,
        timeframe: AnalyticsTimeframe,
    ) -> Result<UsageAnalytics> {
        let end_date = Utc::now();
        let start_date = match timeframe {
            AnalyticsTimeframe::Hour => end_date - Duration::hours(1),
            AnalyticsTimeframe::Day => end_date - Duration::days(1),
            AnalyticsTimeframe::Week => end_date - Duration::weeks(1),
            AnalyticsTimeframe::Month => end_date - Duration::days(30),
            AnalyticsTimeframe::Quarter => end_date - Duration::days(90),
            AnalyticsTimeframe::Year => end_date - Duration::days(365),
        };

        let historical_data = self.get_historical_usage(tenant_id, start_date, end_date).await?;
        let current_usage = self.get_current_usage(tenant_id).await.unwrap_or_default();

        Ok(UsageAnalytics::calculate(historical_data, current_usage, timeframe))
    }

    /// Check if tenant is within usage limits
    pub async fn check_usage_limits(&self, tenant_id: TenantId) -> Result<UsageLimitStatus> {
        let subscription = self.subscription_manager
            .get_subscription_by_tenant(tenant_id)
            .await
            .ok_or(SaaSError::SubscriptionNotFound(Uuid::nil()))?;

        let current_usage = self.get_current_usage(tenant_id).await
            .unwrap_or_default();

        let limit_checks = self.subscription_manager
            .check_usage_limits(subscription.id)
            .await?;

        Ok(UsageLimitStatus {
            within_limits: limit_checks.values().all(|&v| v),
            limit_violations: limit_checks.into_iter()
                .filter(|(_, within_limit)| !within_limit)
                .map(|(metric, _)| metric)
                .collect(),
            current_usage,
            subscription_id: subscription.id,
        })
    }

    // Private methods

    async fn start_event_processor(&self) -> Result<()> {
        let receiver = self.event_receiver.write().await.take()
            .ok_or(SaaSError::MeteringAlreadyStarted)?;

        let tenant_usage = Arc::clone(&self.tenant_usage);
        let historical_storage = Arc::clone(&self.historical_storage);
        let is_running = Arc::new(AtomicBool::new(true));
        let batch_size = self.batch_size;

        tokio::spawn(async move {
            let mut receiver = receiver;
            let mut batch = Vec::with_capacity(batch_size);

            while is_running.load(Ordering::Relaxed) {
                // Collect events into batches
                while batch.len() < batch_size {
                    match receiver.try_recv() {
                        Ok(event) => {
                            // Update real-time aggregation
                            let usage = tenant_usage
                                .entry(event.tenant_id)
                                .or_insert_with(UsageAggregation::new);
                            usage.add_event(&event).await;
                            
                            batch.push(event);
                        }
                        Err(mpsc::error::TryRecvError::Empty) => break,
                        Err(mpsc::error::TryRecvError::Disconnected) => {
                            is_running.store(false, Ordering::Relaxed);
                            break;
                        }
                    }
                }

                // Process batch if not empty
                if !batch.is_empty() {
                    if let Err(e) = historical_storage.store_events(&batch).await {
                        tracing::error!("Failed to store usage events: {}", e);
                    }
                    batch.clear();
                }

                // Small delay to prevent busy waiting
                tokio::time::sleep(TokioDuration::from_millis(10)).await;
            }
        });

        Ok(())
    }

    async fn start_aggregation_task(&self) -> Result<()> {
        let tenant_usage = Arc::clone(&self.tenant_usage);
        let historical_storage = Arc::clone(&self.historical_storage);
        let is_running = Arc::new(AtomicBool::new(true));
        let interval_seconds = self.batch_interval_seconds;

        tokio::spawn(async move {
            let mut interval = interval(TokioDuration::from_secs(interval_seconds));

            while is_running.load(Ordering::Relaxed) {
                interval.tick().await;

                for entry in tenant_usage.iter() {
                    let tenant_id = *entry.key();
                    let usage = entry.value();
                    let snapshot = usage.get_snapshot();

                    if let Err(e) = historical_storage.store_aggregation(tenant_id, &snapshot).await {
                        tracing::error!("Failed to store usage aggregation for tenant {}: {}", tenant_id, e);
                    }
                }
            }
        });

        Ok(())
    }

    async fn start_limit_checker(&self) -> Result<()> {
        let meter = Arc::new(self as *const Self);
        let is_running = Arc::new(AtomicBool::new(true));
        let enable_alerts = Arc::new(AtomicBool::new(true));

        tokio::spawn(async move {
            let mut interval = interval(TokioDuration::from_secs(300)); // Check every 5 minutes

            while is_running.load(Ordering::Relaxed) {
                interval.tick().await;

                if !enable_alerts.load(Ordering::Relaxed) {
                    continue;
                }

                // Check limits for all active tenants
                // This would be implemented based on subscription data
                // For now, this is a placeholder
                tracing::debug!("Checking usage limits for all tenants");
            }
        });

        Ok(())
    }

    async fn start_cleanup_task(&self) -> Result<()> {
        let tenant_usage = Arc::clone(&self.tenant_usage);
        let is_running = Arc::new(AtomicBool::new(true));

        tokio::spawn(async move {
            let mut interval = interval(TokioDuration::from_secs(3600)); // Cleanup every hour

            while is_running.load(Ordering::Relaxed) {
                interval.tick().await;

                let now = Utc::now();
                let mut to_remove = Vec::new();

                for entry in tenant_usage.iter() {
                    let last_updated = *entry.value().last_updated.read().await;
                    if now.signed_duration_since(last_updated).num_hours() > 24 {
                        to_remove.push(*entry.key());
                    }
                }

                for tenant_id in to_remove {
                    tenant_usage.remove(&tenant_id);
                    tracing::debug!("Cleaned up stale usage data for tenant {}", tenant_id);
                }
            }
        });

        Ok(())
    }
}

impl Default for UsageSnapshot {
    fn default() -> Self {
        Self {
            api_calls: 0,
            storage_reads: 0,
            storage_writes: 0,
            storage_bytes: 0,
            compute_time_ms: 0,
            bandwidth_bytes: 0,
            custom_metrics: HashMap::new(),
        }
    }
}

/// Storage trait for persistent usage data
#[async_trait::async_trait]
pub trait UsageStorage {
    async fn store_events(&self, events: &[UsageEvent]) -> Result<()>;
    async fn store_aggregation(&self, tenant_id: TenantId, snapshot: &UsageSnapshot) -> Result<()>;
    async fn get_usage_history(
        &self,
        tenant_id: TenantId,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<HistoricalUsage>>;
}

/// Historical usage data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalUsage {
    pub tenant_id: TenantId,
    pub timestamp: DateTime<Utc>,
    pub period_type: PeriodType,
    pub usage_data: UsageSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeriodType {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

/// Usage analytics timeframe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsTimeframe {
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

/// Usage analytics results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAnalytics {
    pub timeframe: AnalyticsTimeframe,
    pub current_usage: UsageSnapshot,
    pub previous_period_usage: UsageSnapshot,
    pub growth_rates: GrowthRates,
    pub usage_trends: Vec<TrendPoint>,
    pub peak_usage: UsageSnapshot,
    pub average_usage: UsageSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthRates {
    pub api_calls_growth: f64,
    pub storage_growth: f64,
    pub compute_growth: f64,
    pub bandwidth_growth: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub timestamp: DateTime<Utc>,
    pub usage: UsageSnapshot,
}

impl UsageAnalytics {
    pub fn calculate(
        historical_data: Vec<HistoricalUsage>,
        current_usage: UsageSnapshot,
        timeframe: AnalyticsTimeframe,
    ) -> Self {
        // Calculate trends, growth rates, etc.
        // This is a simplified implementation
        
        let trends: Vec<TrendPoint> = historical_data
            .into_iter()
            .map(|h| TrendPoint {
                timestamp: h.timestamp,
                usage: h.usage_data,
            })
            .collect();

        let growth_rates = GrowthRates {
            api_calls_growth: 0.0, // Calculate based on historical data
            storage_growth: 0.0,
            compute_growth: 0.0,
            bandwidth_growth: 0.0,
        };

        Self {
            timeframe,
            current_usage: current_usage.clone(),
            previous_period_usage: UsageSnapshot::default(),
            growth_rates,
            usage_trends: trends,
            peak_usage: current_usage.clone(),
            average_usage: current_usage,
        }
    }
}

/// Usage limit status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLimitStatus {
    pub within_limits: bool,
    pub limit_violations: Vec<String>,
    pub current_usage: UsageSnapshot,
    pub subscription_id: Uuid,
}
