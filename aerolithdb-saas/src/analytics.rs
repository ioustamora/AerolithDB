//! Advanced analytics and insights for SaaS operations

use crate::config::AnalyticsConfig;
use crate::errors::{SaaSError, SaaSResult};
use crate::usage::{UsageMetrics as UsageMetric, UsageStatistics as UsageRecord};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, debug, warn, error};
use uuid::Uuid;

/// Analytics metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Timer,
}

/// Analytics data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

/// Analytics query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsQuery {
    pub metric_name: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub filters: HashMap<String, String>,
    pub aggregation: AggregationType,
    pub interval: Option<chrono::Duration>,
}

/// Aggregation types for analytics queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Average,
    Min,
    Max,
    Count,
    P50,
    P90,
    P95,
    P99,
}

/// Analytics insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub insight_type: InsightType,
    pub title: String,
    pub description: String,
    pub severity: InsightSeverity,
    pub tenant_id: Option<Uuid>,
    pub recommendations: Vec<String>,
    pub data: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Types of insights that can be generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightType {
    PerformanceAnomaly,
    UsageSpike,
    CostOptimization,
    SecurityAlert,
    CapacityPlanning,
    UserBehavior,
    SystemHealth,
    ComplianceIssue,
}

/// Insight severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Tenant usage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUsageSummary {
    pub tenant_id: Uuid,
    pub period_start: chrono::DateTime<chrono::Utc>,
    pub period_end: chrono::DateTime<chrono::Utc>,
    pub api_calls: u64,
    pub storage_bytes: u64,
    pub compute_hours: f64,
    pub bandwidth_bytes: u64,
    pub top_operations: Vec<(String, u64)>,
    pub cost_breakdown: HashMap<String, f64>,
    pub growth_metrics: HashMap<String, f64>,
}

/// Performance benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmark {
    pub operation: String,
    pub p50_latency: f64,
    pub p90_latency: f64,
    pub p95_latency: f64,
    pub p99_latency: f64,
    pub throughput: f64,
    pub error_rate: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Analytics engine for processing usage data and generating insights
pub struct AnalyticsEngine {
    config: AnalyticsConfig,
    metrics_store: Arc<tokio::sync::RwLock<HashMap<String, Vec<DataPoint>>>>,
    insights: Arc<tokio::sync::RwLock<Vec<Insight>>>,
    processing_active: Arc<tokio::sync::RwLock<bool>>,
}

/// Type alias for backward compatibility
pub type AnalyticsManager = AnalyticsEngine;

impl AnalyticsEngine {
    /// Create a new analytics engine
    pub async fn new(config: &AnalyticsConfig) -> Result<Self> {
        info!("📊 Initializing analytics engine");
        
        let engine = Self {
            config: config.clone(),
            metrics_store: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            insights: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            processing_active: Arc::new(tokio::sync::RwLock::new(false)),
        };
        
        info!("✅ Analytics engine initialized");
        Ok(engine)
    }
    
    /// Start analytics processing
    pub async fn start_processing(&self) -> Result<()> {
        if !self.config.enabled {
            info!("📊 Analytics processing is disabled");
            return Ok(());
        }
        
        info!("🔄 Starting analytics processing");
        *self.processing_active.write().await = true;
        
        // Start background processing tasks
        let processing_active = self.processing_active.clone();
        let insights = self.insights.clone();
        let metrics_store = self.metrics_store.clone();
        let processing_interval = self.config.processing_interval;
        
        tokio::spawn(async move {
            while *processing_active.read().await {
                Self::process_insights(&insights, &metrics_store).await;
                tokio::time::sleep(tokio::time::Duration::from_secs(processing_interval * 60)).await;
            }
        });
        
        info!("✅ Analytics processing started");
        Ok(())
    }
    
    /// Stop analytics processing
    pub async fn stop_processing(&self) -> Result<()> {
        info!("🛑 Stopping analytics processing");
        *self.processing_active.write().await = false;
        
        info!("✅ Analytics processing stopped");
        Ok(())
    }
    
    /// Record a metric data point
    pub async fn record_metric(&self, metric_name: String, value: f64, labels: HashMap<String, String>) -> SaaSResult<()> {
        let data_point = DataPoint {
            timestamp: chrono::Utc::now(),
            value,
            labels,
        };
        
        let mut store = self.metrics_store.write().await;
        store.entry(metric_name.clone())
            .or_insert_with(Vec::new)
            .push(data_point);
        
        // Keep only recent data points to manage memory
        if let Some(points) = store.get_mut(&metric_name) {
            let cutoff = chrono::Utc::now() - self.config.retention_period;
            points.retain(|p| p.timestamp > cutoff);
        }
        
        debug!("📈 Recorded metric: {} = {}", metric_name, value);
        Ok(())
    }
    
    /// Query analytics data
    pub async fn query(&self, query: AnalyticsQuery) -> SaaSResult<Vec<DataPoint>> {
        let store = self.metrics_store.read().await;
        let points = store.get(&query.metric_name)
            .ok_or_else(|| SaaSError::InvalidOperation {
                message: format!("Metric not found: {}", query.metric_name),
            })?;
        
        // Filter by time range
        let filtered_points: Vec<_> = points.iter()
            .filter(|p| p.timestamp >= query.start_time && p.timestamp <= query.end_time)
            .filter(|p| {
                // Apply label filters
                query.filters.iter().all(|(key, value)| {
                    p.labels.get(key).map_or(false, |v| v == value)
                })
            })
            .cloned()
            .collect();
        
        // Apply aggregation if interval is specified
        if let Some(interval) = query.interval {
            Ok(self.aggregate_data_points(filtered_points, interval, query.aggregation))
        } else {
            Ok(filtered_points)
        }
    }
    
    /// Generate tenant usage summary
    pub async fn generate_tenant_summary(&self, tenant_id: Uuid, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> SaaSResult<TenantUsageSummary> {
        let mut summary = TenantUsageSummary {
            tenant_id,
            period_start: start,
            period_end: end,
            api_calls: 0,
            storage_bytes: 0,
            compute_hours: 0.0,
            bandwidth_bytes: 0,
            top_operations: Vec::new(),
            cost_breakdown: HashMap::new(),
            growth_metrics: HashMap::new(),
        };
        
        // Query various metrics for the tenant
        let tenant_filter = {
            let mut filter = HashMap::new();
            filter.insert("tenant_id".to_string(), tenant_id.to_string());
            filter
        };
        
        // API calls
        if let Ok(api_data) = self.query(AnalyticsQuery {
            metric_name: "api_calls".to_string(),
            start_time: start,
            end_time: end,
            filters: tenant_filter.clone(),
            aggregation: AggregationType::Sum,
            interval: None,
        }).await {
            summary.api_calls = api_data.iter().map(|p| p.value as u64).sum();
        }
        
        // Storage usage
        if let Ok(storage_data) = self.query(AnalyticsQuery {
            metric_name: "storage_bytes".to_string(),
            start_time: start,
            end_time: end,
            filters: tenant_filter.clone(),
            aggregation: AggregationType::Average,
            interval: None,
        }).await {
            summary.storage_bytes = storage_data.last().map(|p| p.value as u64).unwrap_or(0);
        }
        
        // Compute hours
        if let Ok(compute_data) = self.query(AnalyticsQuery {
            metric_name: "compute_hours".to_string(),
            start_time: start,
            end_time: end,
            filters: tenant_filter.clone(),
            aggregation: AggregationType::Sum,
            interval: None,
        }).await {
            summary.compute_hours = compute_data.iter().map(|p| p.value).sum();
        }
        
        // Calculate cost breakdown
        summary.cost_breakdown.insert("api".to_string(), summary.api_calls as f64 * 0.001); // $0.001 per call
        summary.cost_breakdown.insert("storage".to_string(), summary.storage_bytes as f64 / 1_000_000_000.0 * 0.10); // $0.10 per GB
        summary.cost_breakdown.insert("compute".to_string(), summary.compute_hours * 0.50); // $0.50 per hour
        
        // Calculate growth metrics (compare with previous period)
        let previous_start = start - (end - start);
        if let Ok(previous_summary) = self.generate_tenant_summary(tenant_id, previous_start, start).await {
            if previous_summary.api_calls > 0 {
                let growth = ((summary.api_calls as f64 - previous_summary.api_calls as f64) / previous_summary.api_calls as f64) * 100.0;
                summary.growth_metrics.insert("api_calls_growth".to_string(), growth);
            }
        }
        
        Ok(summary)
    }
    
    /// Generate performance benchmarks
    pub async fn generate_performance_benchmarks(&self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> SaaSResult<Vec<PerformanceBenchmark>> {
        let mut benchmarks = Vec::new();
        
        // Query latency metrics for different operations
        let operations = vec!["get_document", "put_document", "query_documents", "delete_document"];
        
        for operation in operations {
            let latency_query = AnalyticsQuery {
                metric_name: format!("{}_latency", operation),
                start_time: start,
                end_time: end,
                filters: HashMap::new(),
                aggregation: AggregationType::P50,
                interval: None,
            };
            
            if let Ok(latency_data) = self.query(latency_query).await {
                if !latency_data.is_empty() {                    let mut values: Vec<f64> = latency_data.iter().map(|p| p.value).collect();
                    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    
                    let benchmark = PerformanceBenchmark {
                        operation: operation.to_string(),
                        p50_latency: Self::percentile(&values, 50.0),
                        p90_latency: Self::percentile(&values, 90.0),
                        p95_latency: Self::percentile(&values, 95.0),
                        p99_latency: Self::percentile(&values, 99.0),
                        throughput: values.len() as f64 / (end - start).num_seconds() as f64,
                        error_rate: 0.0, // Would calculate from error metrics
                        timestamp: chrono::Utc::now(),
                    };
                    
                    benchmarks.push(benchmark);
                }
            }
        }
        
        Ok(benchmarks)
    }
    
    /// Get current insights
    pub async fn get_insights(&self, tenant_id: Option<Uuid>) -> Vec<Insight> {
        let insights = self.insights.read().await;
        insights.iter()
            .filter(|i| tenant_id.is_none() || i.tenant_id == tenant_id)
            .cloned()
            .collect()
    }
    
    /// Process and generate insights
    async fn process_insights(
        insights: &Arc<tokio::sync::RwLock<Vec<Insight>>>,
        metrics_store: &Arc<tokio::sync::RwLock<HashMap<String, Vec<DataPoint>>>>
    ) {
        debug!("🧠 Processing analytics insights");
        
        let store = metrics_store.read().await;
        let mut new_insights = Vec::new();
        
        // Detect performance anomalies
        if let Some(latency_points) = store.get("api_latency") {
            let recent_points: Vec<_> = latency_points.iter()
                .filter(|p| p.timestamp > chrono::Utc::now() - chrono::Duration::hours(1))
                .collect();
            
            if !recent_points.is_empty() {
                let avg_latency: f64 = recent_points.iter().map(|p| p.value).sum::<f64>() / recent_points.len() as f64;
                
                if avg_latency > 1000.0 { // Over 1 second
                    new_insights.push(Insight {
                        insight_type: InsightType::PerformanceAnomaly,
                        title: "High API Latency Detected".to_string(),
                        description: format!("Average API latency is {}ms, which is above normal thresholds", avg_latency),
                        severity: InsightSeverity::High,
                        tenant_id: None,
                        recommendations: vec![
                            "Check database query performance".to_string(),
                            "Review recent deployments".to_string(),
                            "Consider scaling resources".to_string(),
                        ],
                        data: {
                            let mut data = HashMap::new();
                            data.insert("avg_latency".to_string(), serde_json::Value::from(avg_latency));
                            data
                        },
                        created_at: chrono::Utc::now(),
                    });
                }
            }
        }
        
        // Detect usage spikes
        if let Some(api_points) = store.get("api_calls") {
            let recent_points: Vec<_> = api_points.iter()
                .filter(|p| p.timestamp > chrono::Utc::now() - chrono::Duration::hours(1))
                .collect();
            
            if recent_points.len() > 100 { // High API usage
                new_insights.push(Insight {
                    insight_type: InsightType::UsageSpike,
                    title: "Unusual API Usage Spike".to_string(),
                    description: format!("Detected {} API calls in the last hour, which is above normal patterns", recent_points.len()),
                    severity: InsightSeverity::Medium,
                    tenant_id: None,
                    recommendations: vec![
                        "Review API usage patterns".to_string(),
                        "Check for potential abuse".to_string(),
                        "Consider rate limiting".to_string(),
                    ],
                    data: {
                        let mut data = HashMap::new();
                        data.insert("api_calls_count".to_string(), serde_json::Value::Number(recent_points.len().into()));
                        data
                    },
                    created_at: chrono::Utc::now(),
                });
            }
        }
        
        // Add insights
        let mut insights_write = insights.write().await;
        insights_write.extend(new_insights);
        
        // Keep only recent insights
        let cutoff = chrono::Utc::now() - chrono::Duration::days(7);
        insights_write.retain(|i| i.created_at > cutoff);
        
        debug!("✅ Analytics insights processing completed");
    }
    
    /// Aggregate data points by interval
    fn aggregate_data_points(&self, points: Vec<DataPoint>, interval: chrono::Duration, aggregation: AggregationType) -> Vec<DataPoint> {
        if points.is_empty() {
            return points;
        }
        
        let mut aggregated = Vec::new();
        let start_time = points.first().unwrap().timestamp;
        let end_time = points.last().unwrap().timestamp;
        
        let mut current_time = start_time;
        while current_time < end_time {
            let next_time = current_time + interval;
            let interval_points: Vec<_> = points.iter()
                .filter(|p| p.timestamp >= current_time && p.timestamp < next_time)
                .collect();
            
            if !interval_points.is_empty() {
                let values: Vec<f64> = interval_points.iter().map(|p| p.value).collect();
                let aggregated_value = match aggregation {
                    AggregationType::Sum => values.iter().sum(),
                    AggregationType::Average => values.iter().sum::<f64>() / values.len() as f64,
                    AggregationType::Min => values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                    AggregationType::Max => values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
                    AggregationType::Count => values.len() as f64,
                    AggregationType::P50 => Self::percentile(&values, 50.0),
                    AggregationType::P90 => Self::percentile(&values, 90.0),
                    AggregationType::P95 => Self::percentile(&values, 95.0),
                    AggregationType::P99 => Self::percentile(&values, 99.0),
                };
                
                aggregated.push(DataPoint {
                    timestamp: current_time + interval / 2, // Middle of interval
                    value: aggregated_value,
                    labels: HashMap::new(), // Aggregate loses specific labels
                });
            }
            
            current_time = next_time;
        }
        
        aggregated
    }
    
    /// Calculate percentile from sorted values
    fn percentile(values: &[f64], percentile: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let index = (percentile / 100.0 * (sorted_values.len() - 1) as f64).round() as usize;
        sorted_values[index.min(sorted_values.len() - 1)]
    }
    
    /// Get analytics statistics
    pub async fn get_analytics_stats(&self) -> HashMap<String, serde_json::Value> {
        let store = self.metrics_store.read().await;
        let insights = self.insights.read().await;
        
        let mut stats = HashMap::new();
        stats.insert("metrics_tracked".to_string(), serde_json::Value::Number(store.len().into()));
        stats.insert("total_data_points".to_string(), serde_json::Value::Number(store.values().map(|v| v.len()).sum::<usize>().into()));
        stats.insert("active_insights".to_string(), serde_json::Value::Number(insights.len().into()));
        stats.insert("processing_active".to_string(), serde_json::Value::Bool(*self.processing_active.blocking_read()));
        
        stats
    }
}
