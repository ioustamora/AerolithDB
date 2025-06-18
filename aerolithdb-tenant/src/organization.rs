use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Organization entity representing a tenant in the multi-tenant system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub slug: String, // URL-friendly identifier
    pub display_name: String,
    pub description: Option<String>,
    pub subscription_tier: SubscriptionTier,
    pub status: OrganizationStatus,
    pub settings: OrganizationSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription tiers with different feature sets and limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionTier {
    Free {
        storage_gb: u32,
        api_calls_per_month: u32,
        max_collections: u32,
    },
    Starter {
        storage_gb: u32,
        api_calls_per_month: u32,
        max_collections: u32,
        max_team_members: u32,
    },
    Professional {
        storage_gb: u32,
        api_calls_per_month: u32,
        max_collections: u32,
        max_team_members: u32,
        priority_support: bool,
    },
    Enterprise {
        storage_gb: Option<u32>, // None = unlimited
        api_calls_per_month: Option<u32>,
        max_collections: Option<u32>,
        max_team_members: Option<u32>,
        dedicated_support: bool,
        sla_guarantees: bool,
        custom_features: Vec<String>,
    },
}

/// Organization operational status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrganizationStatus {
    Active,
    Suspended,
    PendingPayment,
    Trial,
    Canceled,
}

/// Organization-specific configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettings {
    pub default_region: String,
    pub backup_retention_days: u32,
    pub security_level: SecurityLevel,
    pub compliance_requirements: Vec<ComplianceFramework>,
    pub notification_preferences: NotificationSettings,
    pub feature_flags: HashMap<String, bool>,
}

/// Security level configuration for the organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Standard,
    Enhanced,
    Maximum,
}

/// Supported compliance frameworks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceFramework {
    GDPR,
    HIPAA,
    SOX,
    PCIDSS,
    SOC2,
}

/// Notification preferences for the organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email_notifications: bool,
    pub webhook_url: Option<String>,
    pub slack_integration: Option<SlackConfig>,
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub webhook_url: String,
    pub channel: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub storage_usage_percent: u8,
    pub api_usage_percent: u8,
    pub error_rate_percent: f32,
}

/// Resource limits and quotas for an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_storage_bytes: Option<u64>,
    pub max_api_calls_per_minute: Option<u32>,
    pub max_concurrent_connections: Option<u16>,
    pub max_collections: Option<u32>,
    pub max_documents_per_collection: Option<u64>,
    pub max_query_complexity: Option<u32>,
    pub max_backup_retention_days: Option<u32>,
}

impl Organization {
    /// Create a new organization with default settings
    pub fn new(name: String, tier: SubscriptionTier) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            slug: Self::generate_slug(&name),
            display_name: name.clone(),
            name,
            description: None,
            subscription_tier: tier,
            status: OrganizationStatus::Trial,
            settings: OrganizationSettings::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Generate a URL-friendly slug from organization name
    fn generate_slug(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Get resource limits based on subscription tier
    pub fn get_resource_limits(&self) -> ResourceLimits {
        match &self.subscription_tier {
            SubscriptionTier::Free { storage_gb, api_calls_per_month, max_collections } => {
                ResourceLimits {
                    max_storage_bytes: Some(*storage_gb as u64 * 1024 * 1024 * 1024),
                    max_api_calls_per_minute: Some(*api_calls_per_month / (30 * 24 * 60)),
                    max_concurrent_connections: Some(10),
                    max_collections: Some(*max_collections),
                    max_documents_per_collection: Some(10000),
                    max_query_complexity: Some(100),
                    max_backup_retention_days: Some(7),
                }
            }
            SubscriptionTier::Starter { storage_gb, api_calls_per_month, max_collections, .. } => {
                ResourceLimits {
                    max_storage_bytes: Some(*storage_gb as u64 * 1024 * 1024 * 1024),
                    max_api_calls_per_minute: Some(*api_calls_per_month / (30 * 24 * 60)),
                    max_concurrent_connections: Some(50),
                    max_collections: Some(*max_collections),
                    max_documents_per_collection: Some(100000),
                    max_query_complexity: Some(500),
                    max_backup_retention_days: Some(30),
                }
            }
            SubscriptionTier::Professional { storage_gb, api_calls_per_month, max_collections, .. } => {
                ResourceLimits {
                    max_storage_bytes: Some(*storage_gb as u64 * 1024 * 1024 * 1024),
                    max_api_calls_per_minute: Some(*api_calls_per_month / (30 * 24 * 60)),
                    max_concurrent_connections: Some(200),
                    max_collections: Some(*max_collections),
                    max_documents_per_collection: Some(1000000),
                    max_query_complexity: Some(2000),
                    max_backup_retention_days: Some(90),
                }
            }
            SubscriptionTier::Enterprise { .. } => {
                ResourceLimits {
                    max_storage_bytes: None, // Unlimited
                    max_api_calls_per_minute: None,
                    max_concurrent_connections: None,
                    max_collections: None,
                    max_documents_per_collection: None,
                    max_query_complexity: None,
                    max_backup_retention_days: None,
                }
            }
        }
    }

    /// Check if organization can perform an operation based on limits
    pub fn check_quota(&self, operation: &QuotaOperation) -> Result<bool> {
        let limits = self.get_resource_limits();
        
        match operation {
            QuotaOperation::CreateCollection => {
                if let Some(max_collections) = limits.max_collections {
                    // TODO: Implement actual collection count check
                    Ok(true) // Placeholder
                } else {
                    Ok(true) // Unlimited
                }
            }
            QuotaOperation::StoreDocument { size_bytes } => {
                if let Some(max_storage) = limits.max_storage_bytes {
                    // TODO: Implement actual storage usage check
                    Ok(*size_bytes <= max_storage) // Simplified check
                } else {
                    Ok(true) // Unlimited
                }
            }
            QuotaOperation::ApiCall => {
                if let Some(max_calls_per_minute) = limits.max_api_calls_per_minute {
                    // TODO: Implement rate limiting check
                    Ok(true) // Placeholder
                } else {
                    Ok(true) // Unlimited
                }
            }
        }
    }
}

/// Operations that need quota checking
#[derive(Debug, Clone)]
pub enum QuotaOperation {
    CreateCollection,
    StoreDocument { size_bytes: u64 },
    ApiCall,
}

impl Default for OrganizationSettings {
    fn default() -> Self {
        Self {
            default_region: "us-east-1".to_string(),
            backup_retention_days: 30,
            security_level: SecurityLevel::Standard,
            compliance_requirements: vec![],
            notification_preferences: NotificationSettings::default(),
            feature_flags: HashMap::new(),
        }
    }
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            email_notifications: true,
            webhook_url: None,
            slack_integration: None,
            alert_thresholds: AlertThresholds {
                storage_usage_percent: 80,
                api_usage_percent: 80,
                error_rate_percent: 5.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slug_generation() {
        assert_eq!(Organization::generate_slug("My Company"), "my-company");
        assert_eq!(Organization::generate_slug("Test & Co."), "test-co");
        assert_eq!(Organization::generate_slug("123-Test-456"), "123-test-456");
    }

    #[test]
    fn test_free_tier_limits() {
        let org = Organization::new(
            "Test Org".to_string(),
            SubscriptionTier::Free {
                storage_gb: 1,
                api_calls_per_month: 10000,
                max_collections: 5,
            },
        );
        
        let limits = org.get_resource_limits();
        assert_eq!(limits.max_storage_bytes, Some(1024 * 1024 * 1024));
        assert_eq!(limits.max_collections, Some(5));
    }

    #[test]
    fn test_enterprise_unlimited() {
        let org = Organization::new(
            "Enterprise Org".to_string(),
            SubscriptionTier::Enterprise {
                storage_gb: None,
                api_calls_per_month: None,
                max_collections: None,
                max_team_members: None,
                dedicated_support: true,
                sla_guarantees: true,
                custom_features: vec!["custom_auth".to_string()],
            },
        );
        
        let limits = org.get_resource_limits();
        assert_eq!(limits.max_storage_bytes, None);
        assert_eq!(limits.max_collections, None);
    }
}
