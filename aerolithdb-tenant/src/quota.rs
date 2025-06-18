use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::organization::{Organization, SubscriptionTier};

/// Quota operation types for tracking and enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuotaOperation {
    CreateCollection,
    DeleteCollection,
    StoreDocument { size_bytes: u64 },
    DeleteDocument { size_bytes: u64 },
    ApiCall { endpoint: String, cost_units: u32 },
    StorageRead { bytes_read: u64 },
    StorageWrite { bytes_written: u64 },
    BackupCreate { size_bytes: u64 },
    QueryExecution { complexity: u32 },
    DataTransfer { bytes_transferred: u64 },
}

/// Current usage statistics for an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationUsage {
    pub org_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub storage_used_bytes: u64,
    pub api_calls_count: u32,
    pub collections_count: u32,
    pub documents_count: u64,
    pub bandwidth_used_bytes: u64,
    pub backup_size_bytes: u64,
    pub query_executions: u32,
    pub last_updated: DateTime<Utc>,
    pub daily_breakdown: Vec<DailyUsage>,
}

/// Daily usage breakdown for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    pub date: DateTime<Utc>,
    pub api_calls: u32,
    pub storage_bytes: u64,
    pub bandwidth_bytes: u64,
    pub queries_executed: u32,
    pub errors_count: u32,
}

/// Quota limits based on subscription tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaLimits {
    pub max_storage_bytes: Option<u64>,
    pub max_api_calls_per_month: Option<u32>,
    pub max_collections: Option<u32>,
    pub max_documents_per_collection: Option<u64>,
    pub max_concurrent_connections: Option<u16>,
    pub max_query_complexity: Option<u32>,
    pub max_bandwidth_per_month: Option<u64>,
    pub max_backup_retention_days: Option<u32>,
    pub rate_limit_per_minute: Option<u32>,
}

/// Quota enforcement actions when limits are exceeded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuotaEnforcementAction {
    Block,
    Throttle { delay_ms: u64 },
    Bill { overage_rate: f64 },
    Warn { threshold_percent: u8 },
}

/// Quota enforcement policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaPolicy {
    pub storage_action: QuotaEnforcementAction,
    pub api_action: QuotaEnforcementAction,
    pub bandwidth_action: QuotaEnforcementAction,
    pub connection_action: QuotaEnforcementAction,
    pub warning_thresholds: HashMap<String, u8>, // metric -> percentage
}

/// Real-time rate limiting state
#[derive(Debug, Clone)]
pub struct RateLimitState {
    pub requests_in_window: u32,
    pub window_start: DateTime<Utc>,
    pub window_duration: Duration,
    pub last_request: DateTime<Utc>,
}

/// Usage tracking entry for detailed analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEntry {
    pub id: Uuid,
    pub org_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub operation: QuotaOperation,
    pub resource_consumed: u64,
    pub user_id: Option<Uuid>,
    pub api_endpoint: Option<String>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
}

/// Quota management system
pub struct QuotaManager {
    usage_stats: Arc<RwLock<HashMap<Uuid, OrganizationUsage>>>,
    rate_limits: Arc<RwLock<HashMap<Uuid, RateLimitState>>>,
    usage_entries: Arc<RwLock<Vec<UsageEntry>>>,
    quota_policies: HashMap<String, QuotaPolicy>, // tier_name -> policy
}

impl QuotaManager {
    /// Create a new quota manager
    pub fn new() -> Self {
        let mut quota_policies = HashMap::new();
        
        // Define default quota policies for each tier
        quota_policies.insert("free".to_string(), QuotaPolicy {
            storage_action: QuotaEnforcementAction::Block,
            api_action: QuotaEnforcementAction::Block,
            bandwidth_action: QuotaEnforcementAction::Throttle { delay_ms: 1000 },
            connection_action: QuotaEnforcementAction::Block,
            warning_thresholds: {
                let mut thresholds = HashMap::new();
                thresholds.insert("storage".to_string(), 80);
                thresholds.insert("api_calls".to_string(), 80);
                thresholds.insert("bandwidth".to_string(), 90);
                thresholds
            },
        });
        
        quota_policies.insert("professional".to_string(), QuotaPolicy {
            storage_action: QuotaEnforcementAction::Warn { threshold_percent: 90 },
            api_action: QuotaEnforcementAction::Bill { overage_rate: 0.01 },
            bandwidth_action: QuotaEnforcementAction::Bill { overage_rate: 0.05 },
            connection_action: QuotaEnforcementAction::Throttle { delay_ms: 100 },
            warning_thresholds: {
                let mut thresholds = HashMap::new();
                thresholds.insert("storage".to_string(), 85);
                thresholds.insert("api_calls".to_string(), 85);
                thresholds.insert("bandwidth".to_string(), 90);
                thresholds
            },
        });
        
        quota_policies.insert("enterprise".to_string(), QuotaPolicy {
            storage_action: QuotaEnforcementAction::Warn { threshold_percent: 95 },
            api_action: QuotaEnforcementAction::Warn { threshold_percent: 95 },
            bandwidth_action: QuotaEnforcementAction::Warn { threshold_percent: 95 },
            connection_action: QuotaEnforcementAction::Warn { threshold_percent: 90 },
            warning_thresholds: {
                let mut thresholds = HashMap::new();
                thresholds.insert("storage".to_string(), 90);
                thresholds.insert("api_calls".to_string(), 90);
                thresholds.insert("bandwidth".to_string(), 95);
                thresholds
            },
        });
        
        Self {
            usage_stats: Arc::new(RwLock::new(HashMap::new())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            usage_entries: Arc::new(RwLock::new(Vec::new())),
            quota_policies,
        }
    }

    /// Initialize quota tracking for a new organization
    pub async fn initialize_organization(&self, org: &Organization) -> Result<()> {
        let now = Utc::now();
        let usage = OrganizationUsage {
            org_id: org.id,
            period_start: now,
            period_end: now + Duration::days(30), // Monthly billing cycle
            storage_used_bytes: 0,
            api_calls_count: 0,
            collections_count: 0,
            documents_count: 0,
            bandwidth_used_bytes: 0,
            backup_size_bytes: 0,
            query_executions: 0,
            last_updated: now,
            daily_breakdown: vec![],
        };
        
        let rate_limit_state = RateLimitState {
            requests_in_window: 0,
            window_start: now,
            window_duration: Duration::minutes(1),
            last_request: now,
        };
        
        {
            let mut usage_stats = self.usage_stats.write().await;
            usage_stats.insert(org.id, usage);
        }
        
        {
            let mut rate_limits = self.rate_limits.write().await;
            rate_limits.insert(org.id, rate_limit_state);
        }
        
        Ok(())
    }

    /// Update organization quota settings
    pub async fn update_organization(&self, org: &Organization) -> Result<()> {
        // Reset rate limits when tier changes
        let now = Utc::now();
        let rate_limit_state = RateLimitState {
            requests_in_window: 0,
            window_start: now,
            window_duration: Duration::minutes(1),
            last_request: now,
        };
        
        {
            let mut rate_limits = self.rate_limits.write().await;
            rate_limits.insert(org.id, rate_limit_state);
        }
        
        Ok(())
    }

    /// Remove organization from quota tracking
    pub async fn remove_organization(&self, org_id: &Uuid) -> Result<()> {
        {
            let mut usage_stats = self.usage_stats.write().await;
            usage_stats.remove(org_id);
        }
        
        {
            let mut rate_limits = self.rate_limits.write().await;
            rate_limits.remove(org_id);
        }
        
        {
            let mut usage_entries = self.usage_entries.write().await;
            usage_entries.retain(|entry| &entry.org_id != org_id);
        }
        
        Ok(())
    }

    /// Check if an operation is allowed under current quotas
    pub async fn check_quota(&self, org_id: &Uuid, operation: &QuotaOperation) -> Result<bool> {
        let usage_stats = self.usage_stats.read().await;
        let usage = usage_stats.get(org_id)
            .ok_or_else(|| anyhow!("Organization not found in quota tracking"))?;
        
        // TODO: Get actual organization to check limits
        // For now, use placeholder limits
        let limits = QuotaLimits {
            max_storage_bytes: Some(1024 * 1024 * 1024), // 1GB
            max_api_calls_per_month: Some(10000),
            max_collections: Some(10),
            max_documents_per_collection: Some(10000),
            max_concurrent_connections: Some(100),
            max_query_complexity: Some(1000),
            max_bandwidth_per_month: Some(10 * 1024 * 1024 * 1024), // 10GB
            max_backup_retention_days: Some(30),
            rate_limit_per_minute: Some(100),
        };
        
        match operation {
            QuotaOperation::CreateCollection => {
                if let Some(max_collections) = limits.max_collections {
                    Ok(usage.collections_count < max_collections)
                } else {
                    Ok(true)
                }
            }
            QuotaOperation::StoreDocument { size_bytes } => {
                if let Some(max_storage) = limits.max_storage_bytes {
                    Ok(usage.storage_used_bytes + size_bytes <= max_storage)
                } else {
                    Ok(true)
                }
            }
            QuotaOperation::ApiCall { .. } => {
                if let Some(max_calls) = limits.max_api_calls_per_month {
                    Ok(usage.api_calls_count < max_calls)
                } else {
                    Ok(true)
                }
            }
            QuotaOperation::QueryExecution { complexity } => {
                if let Some(max_complexity) = limits.max_query_complexity {
                    Ok(*complexity <= max_complexity)
                } else {
                    Ok(true)
                }
            }
            _ => Ok(true), // Allow other operations for now
        }
    }

    /// Check rate limits for an organization
    pub async fn check_rate_limit(&self, org_id: &Uuid) -> Result<bool> {
        let mut rate_limits = self.rate_limits.write().await;
        let now = Utc::now();
        
        if let Some(rate_state) = rate_limits.get_mut(org_id) {
            // Check if we need to reset the window
            if now - rate_state.window_start > rate_state.window_duration {
                rate_state.requests_in_window = 0;
                rate_state.window_start = now;
            }
            
            // TODO: Get actual rate limit from organization settings
            let rate_limit = 100; // placeholder
            
            if rate_state.requests_in_window >= rate_limit {
                Ok(false) // Rate limited
            } else {
                rate_state.requests_in_window += 1;
                rate_state.last_request = now;
                Ok(true)
            }
        } else {
            Err(anyhow!("Organization not found in rate limiting"))
        }
    }

    /// Record usage for an operation
    pub async fn record_usage(&self, 
        org_id: &Uuid, 
        operation: QuotaOperation, 
        user_id: Option<Uuid>
    ) -> Result<()> {
        let now = Utc::now();
        
        // Create usage entry for detailed tracking
        let entry = UsageEntry {
            id: Uuid::new_v4(),
            org_id: *org_id,
            timestamp: now,
            operation: operation.clone(),
            resource_consumed: self.calculate_resource_consumption(&operation),
            user_id,
            api_endpoint: None, // TODO: Extract from context
            client_ip: None,    // TODO: Extract from context
            user_agent: None,   // TODO: Extract from context
        };
        
        {
            let mut usage_entries = self.usage_entries.write().await;
            usage_entries.push(entry);
        }
        
        // Update aggregated usage statistics
        {
            let mut usage_stats = self.usage_stats.write().await;
            if let Some(usage) = usage_stats.get_mut(org_id) {
                match operation {
                    QuotaOperation::CreateCollection => {
                        usage.collections_count += 1;
                    }
                    QuotaOperation::DeleteCollection => {
                        usage.collections_count = usage.collections_count.saturating_sub(1);
                    }
                    QuotaOperation::StoreDocument { size_bytes } => {
                        usage.storage_used_bytes += size_bytes;
                        usage.documents_count += 1;
                    }
                    QuotaOperation::DeleteDocument { size_bytes } => {
                        usage.storage_used_bytes = usage.storage_used_bytes.saturating_sub(size_bytes);
                        usage.documents_count = usage.documents_count.saturating_sub(1);
                    }
                    QuotaOperation::ApiCall { .. } => {
                        usage.api_calls_count += 1;
                    }
                    QuotaOperation::QueryExecution { .. } => {
                        usage.query_executions += 1;
                    }
                    QuotaOperation::DataTransfer { bytes_transferred } => {
                        usage.bandwidth_used_bytes += bytes_transferred;
                    }
                    _ => {} // Handle other operations as needed
                }
                
                usage.last_updated = now;
            }
        }
        
        Ok(())
    }

    /// Get current usage statistics for an organization
    pub async fn get_usage_stats(&self, org_id: &Uuid) -> Result<OrganizationUsage> {
        let usage_stats = self.usage_stats.read().await;
        usage_stats.get(org_id)
            .cloned()
            .ok_or_else(|| anyhow!("Organization not found in quota tracking"))
    }

    /// Get detailed usage history for an organization
    pub async fn get_usage_history(&self, 
        org_id: &Uuid, 
        start_date: DateTime<Utc>, 
        end_date: DateTime<Utc>
    ) -> Result<Vec<UsageEntry>> {
        let usage_entries = self.usage_entries.read().await;
        let filtered_entries: Vec<UsageEntry> = usage_entries
            .iter()
            .filter(|entry| {
                &entry.org_id == org_id && 
                entry.timestamp >= start_date && 
                entry.timestamp <= end_date
            })
            .cloned()
            .collect();
        
        Ok(filtered_entries)
    }

    /// Calculate billing amount for an organization for a given period
    pub async fn calculate_billing(&self, 
        org_id: &Uuid, 
        period_start: DateTime<Utc>, 
        period_end: DateTime<Utc>
    ) -> Result<BillingCalculation> {
        let usage_history = self.get_usage_history(org_id, period_start, period_end).await?;
        
        let mut calculation = BillingCalculation {
            org_id: *org_id,
            period_start,
            period_end,
            base_cost: 0.0,
            overage_costs: HashMap::new(),
            total_cost: 0.0,
            usage_summary: HashMap::new(),
        };
        
        // Aggregate usage
        let mut api_calls = 0u32;
        let mut storage_bytes = 0u64;
        let mut bandwidth_bytes = 0u64;
        
        for entry in &usage_history {
            match &entry.operation {
                QuotaOperation::ApiCall { .. } => api_calls += 1,
                QuotaOperation::StoreDocument { size_bytes } => storage_bytes += size_bytes,
                QuotaOperation::DataTransfer { bytes_transferred } => bandwidth_bytes += bytes_transferred,
                _ => {}
            }
        }
        
        calculation.usage_summary.insert("api_calls".to_string(), api_calls as f64);
        calculation.usage_summary.insert("storage_gb".to_string(), storage_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
        calculation.usage_summary.insert("bandwidth_gb".to_string(), bandwidth_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
        
        // TODO: Calculate actual costs based on subscription tier and overages
        calculation.base_cost = 29.0; // Placeholder base cost
        calculation.total_cost = calculation.base_cost;
        
        Ok(calculation)
    }

    /// Generate usage analytics for an organization
    pub async fn generate_analytics(&self, org_id: &Uuid) -> Result<UsageAnalytics> {
        let usage = self.get_usage_stats(org_id).await?;
        let now = Utc::now();
        let thirty_days_ago = now - Duration::days(30);
        let usage_history = self.get_usage_history(org_id, thirty_days_ago, now).await?;
        
        // Calculate trends
        let mut daily_api_calls: HashMap<String, u32> = HashMap::new();
        let mut daily_storage: HashMap<String, u64> = HashMap::new();
        
        for entry in &usage_history {
            let date_key = entry.timestamp.format("%Y-%m-%d").to_string();
            
            match &entry.operation {
                QuotaOperation::ApiCall { .. } => {
                    *daily_api_calls.entry(date_key).or_insert(0) += 1;
                }
                QuotaOperation::StoreDocument { size_bytes } => {
                    *daily_storage.entry(date_key).or_insert(0) += size_bytes;
                }
                _ => {}
            }
        }
        
        Ok(UsageAnalytics {
            org_id: *org_id,
            current_usage: usage,
            daily_trends: daily_api_calls,
            storage_trends: daily_storage,
            peak_usage_hour: self.calculate_peak_hour(&usage_history),
            efficiency_score: self.calculate_efficiency_score(&usage_history),
            recommendations: self.generate_recommendations(&usage_history),
        })
    }

    /// Calculate resource consumption for an operation
    fn calculate_resource_consumption(&self, operation: &QuotaOperation) -> u64 {
        match operation {
            QuotaOperation::StoreDocument { size_bytes } => *size_bytes,
            QuotaOperation::DataTransfer { bytes_transferred } => *bytes_transferred,
            QuotaOperation::BackupCreate { size_bytes } => *size_bytes,
            QuotaOperation::ApiCall { cost_units, .. } => *cost_units as u64,
            _ => 1, // Default unit consumption
        }
    }

    /// Calculate peak usage hour from history
    fn calculate_peak_hour(&self, usage_history: &[UsageEntry]) -> u8 {
        let mut hourly_counts = vec![0u32; 24];
        
        for entry in usage_history {
            let hour = entry.timestamp.hour() as usize;
            hourly_counts[hour] += 1;
        }
        
        hourly_counts
            .iter()
            .enumerate()
            .max_by_key(|(_, &count)| count)
            .map(|(hour, _)| hour as u8)
            .unwrap_or(0)
    }

    /// Calculate efficiency score based on usage patterns
    fn calculate_efficiency_score(&self, usage_history: &[UsageEntry]) -> f64 {
        if usage_history.is_empty() {
            return 0.0;
        }
        
        // Simple efficiency calculation based on error rates and usage patterns
        // In a real implementation, this would be more sophisticated
        let total_operations = usage_history.len() as f64;
        let unique_days = usage_history
            .iter()
            .map(|e| e.timestamp.date_naive())
            .collect::<std::collections::HashSet<_>>()
            .len() as f64;
        
        // Higher scores for consistent daily usage
        (unique_days / 30.0) * 100.0
    }

    /// Generate optimization recommendations
    fn generate_recommendations(&self, usage_history: &[UsageEntry]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if usage_history.len() > 1000 {
            recommendations.push("Consider implementing request batching to reduce API call overhead".to_string());
        }
        
        let storage_operations = usage_history
            .iter()
            .filter(|e| matches!(e.operation, QuotaOperation::StoreDocument { .. }))
            .count();
        
        if storage_operations > 100 {
            recommendations.push("Consider enabling compression to reduce storage costs".to_string());
        }
        
        recommendations
    }
}

/// Billing calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingCalculation {
    pub org_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub base_cost: f64,
    pub overage_costs: HashMap<String, f64>,
    pub total_cost: f64,
    pub usage_summary: HashMap<String, f64>,
}

/// Usage analytics for an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAnalytics {
    pub org_id: Uuid,
    pub current_usage: OrganizationUsage,
    pub daily_trends: HashMap<String, u32>,
    pub storage_trends: HashMap<String, u64>,
    pub peak_usage_hour: u8,
    pub efficiency_score: f64,
    pub recommendations: Vec<String>,
}

impl Default for QuotaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organization::{Organization, SubscriptionTier};

    #[tokio::test]
    async fn test_quota_initialization() {
        let manager = QuotaManager::new();
        let org = Organization::new(
            "Test Company".to_string(),
            SubscriptionTier::Free {
                storage_gb: 1,
                api_calls_per_month: 10000,
                max_collections: 5,
            },
        );
        
        manager.initialize_organization(&org).await.unwrap();
        
        let usage = manager.get_usage_stats(&org.id).await.unwrap();
        assert_eq!(usage.org_id, org.id);
        assert_eq!(usage.storage_used_bytes, 0);
        assert_eq!(usage.api_calls_count, 0);
    }

    #[tokio::test]
    async fn test_usage_recording() {
        let manager = QuotaManager::new();
        let org = Organization::new(
            "Test Company".to_string(),
            SubscriptionTier::Free {
                storage_gb: 1,
                api_calls_per_month: 10000,
                max_collections: 5,
            },
        );
        
        manager.initialize_organization(&org).await.unwrap();
        
        // Record some operations
        manager.record_usage(&org.id, QuotaOperation::CreateCollection, None).await.unwrap();
        manager.record_usage(&org.id, QuotaOperation::StoreDocument { size_bytes: 1024 }, None).await.unwrap();
        manager.record_usage(&org.id, QuotaOperation::ApiCall { endpoint: "/api/test".to_string(), cost_units: 1 }, None).await.unwrap();
        
        let usage = manager.get_usage_stats(&org.id).await.unwrap();
        assert_eq!(usage.collections_count, 1);
        assert_eq!(usage.storage_used_bytes, 1024);
        assert_eq!(usage.api_calls_count, 1);
        assert_eq!(usage.documents_count, 1);
    }

    #[tokio::test]
    async fn test_quota_checking() {
        let manager = QuotaManager::new();
        let org = Organization::new(
            "Test Company".to_string(),
            SubscriptionTier::Free {
                storage_gb: 1,
                api_calls_per_month: 10000,
                max_collections: 5,
            },
        );
        
        manager.initialize_organization(&org).await.unwrap();
        
        // Should allow creating collections within limit
        let can_create = manager.check_quota(&org.id, &QuotaOperation::CreateCollection).await.unwrap();
        assert!(can_create);
        
        // Should allow storing documents within storage limit
        let can_store = manager.check_quota(&org.id, &QuotaOperation::StoreDocument { size_bytes: 1024 }).await.unwrap();
        assert!(can_store);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let manager = QuotaManager::new();
        let org = Organization::new(
            "Test Company".to_string(),
            SubscriptionTier::Free {
                storage_gb: 1,
                api_calls_per_month: 10000,
                max_collections: 5,
            },
        );
        
        manager.initialize_organization(&org).await.unwrap();
        
        // First request should be allowed
        let allowed = manager.check_rate_limit(&org.id).await.unwrap();
        assert!(allowed);
        
        // Subsequent requests should also be allowed within rate limit
        for _ in 0..50 {
            let allowed = manager.check_rate_limit(&org.id).await.unwrap();
            assert!(allowed);
        }
    }
}
