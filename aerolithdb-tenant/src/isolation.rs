use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;
use std::collections::HashMap;

use crate::organization::Organization;
use aerolithdb_core::types::{CollectionId, DocumentId};

/// Tenant isolation configuration and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantIsolation {
    pub org_id: Uuid,
    pub database_prefix: String,
    pub storage_namespace: String,
    pub encryption_key_id: String,
    pub network_isolation: NetworkIsolation,
    pub resource_isolation: ResourceIsolation,
}

/// Network-level isolation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIsolation {
    pub virtual_network_id: String,
    pub subnet_cidr: String,
    pub firewall_rules: Vec<FirewallRule>,
    pub api_rate_limits: ApiRateLimits,
}

/// Firewall rule for network access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub name: String,
    pub action: FirewallAction,
    pub source_cidr: String,
    pub destination_port: u16,
    pub protocol: Protocol,
}

/// Firewall actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FirewallAction {
    Allow,
    Deny,
    Log,
}

/// Network protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Protocol {
    TCP,
    UDP,
    HTTPS,
    HTTP,
}

/// API rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRateLimits {
    pub requests_per_minute: u32,
    pub burst_capacity: u32,
    pub concurrent_connections: u16,
    pub quota_enforcement: QuotaEnforcement,
}

/// Quota enforcement strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuotaEnforcement {
    Block,          // Block requests when quota exceeded
    Throttle,       // Slow down requests
    Bill,           // Allow but charge extra
    Notify,         // Allow but send notifications
}

/// Resource-level isolation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceIsolation {
    pub cpu_limit_cores: Option<f32>,
    pub memory_limit_gb: Option<u32>,
    pub storage_limit_gb: Option<u32>,
    pub io_limit_iops: Option<u32>,
    pub dedicated_resources: bool,
}

/// Tenant-aware data access layer
pub struct TenantDataAccess {
    isolation_config: TenantIsolation,
}

impl TenantDataAccess {
    /// Create a new tenant data access layer
    pub fn new(org: &Organization) -> Result<Self> {
        let isolation_config = Self::create_isolation_config(org)?;
        Ok(Self { isolation_config })
    }

    /// Create isolation configuration for an organization
    fn create_isolation_config(org: &Organization) -> Result<TenantIsolation> {
        let limits = org.get_resource_limits();
        
        Ok(TenantIsolation {
            org_id: org.id,
            database_prefix: format!("tenant_{}", org.slug),
            storage_namespace: format!("org-{}", org.id),
            encryption_key_id: format!("key-{}", org.id),
            network_isolation: NetworkIsolation {
                virtual_network_id: format!("vnet-{}", org.id),
                subnet_cidr: Self::generate_subnet_cidr(&org.id)?,
                firewall_rules: Self::default_firewall_rules(),
                api_rate_limits: ApiRateLimits {
                    requests_per_minute: limits.max_api_calls_per_minute.unwrap_or(1000),
                    burst_capacity: limits.max_api_calls_per_minute.unwrap_or(1000) * 2,
                    concurrent_connections: limits.max_concurrent_connections.unwrap_or(100),
                    quota_enforcement: QuotaEnforcement::Throttle,
                },
            },
            resource_isolation: ResourceIsolation {
                cpu_limit_cores: None, // TODO: Configure based on tier
                memory_limit_gb: None,
                storage_limit_gb: limits.max_storage_bytes.map(|b| (b / (1024 * 1024 * 1024)) as u32),
                io_limit_iops: None,
                dedicated_resources: matches!(org.subscription_tier, crate::organization::SubscriptionTier::Enterprise { .. }),
            },
        })
    }

    /// Generate a unique subnet CIDR for the organization
    fn generate_subnet_cidr(org_id: &Uuid) -> Result<String> {
        // Use org_id bytes to generate a unique subnet within 10.0.0.0/8
        let bytes = org_id.as_bytes();
        let subnet_b = bytes[0];
        let subnet_c = bytes[1];
        Ok(format!("10.{}.{}.0/24", subnet_b, subnet_c))
    }

    /// Default firewall rules for tenant isolation
    fn default_firewall_rules() -> Vec<FirewallRule> {
        vec![
            FirewallRule {
                name: "allow_https".to_string(),
                action: FirewallAction::Allow,
                source_cidr: "0.0.0.0/0".to_string(),
                destination_port: 443,
                protocol: Protocol::HTTPS,
            },
            FirewallRule {
                name: "allow_http".to_string(),
                action: FirewallAction::Allow,
                source_cidr: "0.0.0.0/0".to_string(),
                destination_port: 80,
                protocol: Protocol::HTTP,
            },
            FirewallRule {
                name: "deny_all_other".to_string(),
                action: FirewallAction::Deny,
                source_cidr: "0.0.0.0/0".to_string(),
                destination_port: 0, // All ports
                protocol: Protocol::TCP,
            },
        ]
    }

    /// Get tenant-scoped collection name
    pub fn scope_collection_name(&self, collection: &CollectionId) -> String {
        format!("{}.{}", self.isolation_config.database_prefix, collection.0)
    }

    /// Get tenant-scoped document key
    pub fn scope_document_key(&self, collection: &CollectionId, document: &DocumentId) -> String {
        format!("{}.{}.{}", 
            self.isolation_config.database_prefix, 
            collection.0, 
            document.0)
    }

    /// Get tenant storage path
    pub fn get_storage_path(&self) -> PathBuf {
        PathBuf::from("data")
            .join("tenants")
            .join(&self.isolation_config.storage_namespace)
    }

    /// Check if the tenant has access to a resource
    pub fn check_access(&self, resource: &TenantResource) -> Result<bool> {
        match resource {
            TenantResource::Collection(collection_id) => {
                // Check if collection belongs to this tenant
                Ok(collection_id.0.starts_with(&self.isolation_config.database_prefix))
            }
            TenantResource::Document(collection_id, _) => {
                // Check collection access first
                self.check_access(&TenantResource::Collection(collection_id.clone()))
            }
            TenantResource::StoragePath(path) => {
                // Check if path is within tenant storage namespace
                let tenant_path = self.get_storage_path();
                Ok(path.starts_with(&tenant_path))
            }
        }
    }

    /// Validate and enforce rate limits
    pub fn check_rate_limit(&self, request_count: u32, window_start: std::time::Instant) -> Result<bool> {
        let elapsed = window_start.elapsed();
        let window_minutes = elapsed.as_secs() as f64 / 60.0;
        
        if window_minutes >= 1.0 {
            // Reset window
            return Ok(true);
        }
        
        let rate_limit = self.isolation_config.network_isolation.api_rate_limits.requests_per_minute;
        let allowed_in_window = (rate_limit as f64 * window_minutes) as u32;
        
        Ok(request_count <= allowed_in_window)
    }

    /// Get isolation configuration
    pub fn get_isolation_config(&self) -> &TenantIsolation {
        &self.isolation_config
    }
}

/// Resources that need tenant access control
#[derive(Debug, Clone)]
pub enum TenantResource {
    Collection(CollectionId),
    Document(CollectionId, DocumentId),
    StoragePath(PathBuf),
}

/// Tenant isolation manager for the entire system
pub struct TenantIsolationManager {
    tenant_configs: HashMap<Uuid, TenantDataAccess>,
}

impl TenantIsolationManager {
    /// Create a new tenant isolation manager
    pub fn new() -> Self {
        Self {
            tenant_configs: HashMap::new(),
        }
    }

    /// Register a new tenant
    pub fn register_tenant(&mut self, org: &Organization) -> Result<()> {
        let data_access = TenantDataAccess::new(org)?;
        self.tenant_configs.insert(org.id, data_access);
        Ok(())
    }

    /// Get tenant data access layer
    pub fn get_tenant_access(&self, org_id: &Uuid) -> Result<&TenantDataAccess> {
        self.tenant_configs
            .get(org_id)
            .ok_or_else(|| anyhow!("Tenant {} not found", org_id))
    }

    /// Remove a tenant (when organization is deleted)
    pub fn remove_tenant(&mut self, org_id: &Uuid) -> Result<()> {
        self.tenant_configs
            .remove(org_id)
            .ok_or_else(|| anyhow!("Tenant {} not found", org_id))?;
        Ok(())
    }

    /// List all registered tenants
    pub fn list_tenants(&self) -> Vec<Uuid> {
        self.tenant_configs.keys().cloned().collect()
    }

    /// Validate tenant isolation (for security audits)
    pub fn validate_isolation(&self) -> Result<Vec<IsolationValidationResult>> {
        let mut results = Vec::new();
        
        for (org_id, access) in &self.tenant_configs {
            let config = access.get_isolation_config();
            
            // Check for potential isolation violations
            let mut issues = Vec::new();
            
            // Check storage namespace uniqueness
            for (other_org_id, other_access) in &self.tenant_configs {
                if org_id != other_org_id {
                    let other_config = other_access.get_isolation_config();
                    if config.storage_namespace == other_config.storage_namespace {
                        issues.push("Storage namespace collision detected".to_string());
                    }
                    if config.database_prefix == other_config.database_prefix {
                        issues.push("Database prefix collision detected".to_string());
                    }
                }
            }
            
            results.push(IsolationValidationResult {
                org_id: *org_id,
                is_valid: issues.is_empty(),
                issues,
            });
        }
        
        Ok(results)
    }
}

/// Result of tenant isolation validation
#[derive(Debug, Clone)]
pub struct IsolationValidationResult {
    pub org_id: Uuid,
    pub is_valid: bool,
    pub issues: Vec<String>,
}

impl Default for TenantIsolationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organization::{Organization, SubscriptionTier};

    #[test]
    fn test_tenant_isolation_creation() {
        let org = Organization::new(
            "Test Company".to_string(),
            SubscriptionTier::Free {
                storage_gb: 1,
                api_calls_per_month: 10000,
                max_collections: 5,
            },
        );
        
        let access = TenantDataAccess::new(&org).unwrap();
        let config = access.get_isolation_config();
        
        assert_eq!(config.org_id, org.id);
        assert_eq!(config.database_prefix, "tenant_test-company");
        assert_eq!(config.storage_namespace, format!("org-{}", org.id));
    }

    #[test]
    fn test_collection_scoping() {
        let org = Organization::new(
            "Test Company".to_string(),
            SubscriptionTier::Free {
                storage_gb: 1,
                api_calls_per_month: 10000,
                max_collections: 5,
            },
        );
        
        let access = TenantDataAccess::new(&org).unwrap();
        let collection = CollectionId("users".to_string());
        let scoped = access.scope_collection_name(&collection);
        
        assert_eq!(scoped, "tenant_test-company.users");
    }

    #[test]
    fn test_access_control() {
        let org = Organization::new(
            "Test Company".to_string(),
            SubscriptionTier::Free {
                storage_gb: 1,
                api_calls_per_month: 10000,
                max_collections: 5,
            },
        );
        
        let access = TenantDataAccess::new(&org).unwrap();
        
        // Valid collection access
        let valid_collection = CollectionId("tenant_test-company.users".to_string());
        assert!(access.check_access(&TenantResource::Collection(valid_collection)).unwrap());
        
        // Invalid collection access
        let invalid_collection = CollectionId("other_tenant.users".to_string());
        assert!(!access.check_access(&TenantResource::Collection(invalid_collection)).unwrap());
    }

    #[test]
    fn test_isolation_validation() {
        let mut manager = TenantIsolationManager::new();
        
        let org1 = Organization::new(
            "Company One".to_string(),
            SubscriptionTier::Free {
                storage_gb: 1,
                api_calls_per_month: 10000,
                max_collections: 5,
            },
        );
        
        let org2 = Organization::new(
            "Company Two".to_string(),
            SubscriptionTier::Free {
                storage_gb: 1,
                api_calls_per_month: 10000,
                max_collections: 5,
            },
        );
        
        manager.register_tenant(&org1).unwrap();
        manager.register_tenant(&org2).unwrap();
        
        let validation_results = manager.validate_isolation().unwrap();
        
        // Should have results for both organizations
        assert_eq!(validation_results.len(), 2);
        
        // Both should be valid (no collisions)
        for result in validation_results {
            assert!(result.is_valid, "Isolation validation failed for {}: {:?}", result.org_id, result.issues);
        }
    }
}
