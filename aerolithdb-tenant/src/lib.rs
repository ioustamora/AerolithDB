//! # AerolithDB Multi-Tenant Management System
//! 
//! This module provides comprehensive multi-tenancy support for AerolithDB,
//! enabling the database to serve multiple organizations with complete
//! data isolation, resource management, and security controls.
//! 
//! ## Features
//! 
//! - **Organization Management**: Create, update, and manage tenant organizations
//! - **Tenant Isolation**: Complete data and resource isolation between tenants
//! - **Resource Quotas**: Configurable limits based on subscription tiers
//! - **Security**: Network and data isolation with encryption key management
//! - **Compliance**: Support for GDPR, HIPAA, SOX, and other frameworks
//! 
//! ## Architecture
//! 
//! The multi-tenancy system is built on several key components:
//! 
//! - `Organization`: Represents a tenant with subscription and settings
//! - `TenantIsolation`: Manages data and resource isolation
//! - `TenantDataAccess`: Provides tenant-scoped data access
//! - `TenantIsolationManager`: Coordinates multiple tenants
//! 
//! ## Usage Example
//! 
//! ```rust
//! use aerolithdb_tenant::{Organization, SubscriptionTier, TenantIsolationManager};
//! 
//! // Create a new organization
//! let org = Organization::new(
//!     "My Company".to_string(),
//!     SubscriptionTier::Professional {
//!         storage_gb: 100,
//!         api_calls_per_month: 1000000,
//!         max_collections: 50,
//!         max_team_members: 10,
//!         priority_support: true,
//!     },
//! );
//! 
//! // Register with isolation manager
//! let mut manager = TenantIsolationManager::new();
//! manager.register_tenant(&org).unwrap();
//! 
//! // Get tenant-scoped data access
//! let access = manager.get_tenant_access(&org.id).unwrap();
//! ```

pub mod organization;
pub mod isolation;
pub mod user;
pub mod quota;

pub use organization::*;
pub use isolation::*;
pub use user::*;
pub use quota::*;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main tenant management service
pub struct TenantManager {
    isolation_manager: Arc<RwLock<TenantIsolationManager>>,
    user_manager: Arc<RwLock<TenantUserManager>>,
    quota_manager: Arc<QuotaManager>,
}

impl TenantManager {
    /// Create a new tenant manager
    pub fn new() -> Self {
        Self {
            isolation_manager: Arc::new(RwLock::new(TenantIsolationManager::new())),
            user_manager: Arc::new(RwLock::new(TenantUserManager::new())),
            quota_manager: Arc::new(QuotaManager::new()),
        }
    }

    /// Create a new organization and register it
    pub async fn create_organization(&self, name: String, tier: SubscriptionTier) -> Result<Organization> {
        let org = Organization::new(name, tier);
        
        // Register with isolation manager
        {
            let mut isolation_manager = self.isolation_manager.write().await;
            isolation_manager.register_tenant(&org)?;
        }
        
        // Initialize quota tracking
        self.quota_manager.initialize_organization(&org).await?;
        
        Ok(org)
    }

    /// Get organization by ID
    pub async fn get_organization(&self, org_id: &uuid::Uuid) -> Result<Option<Organization>> {
        // TODO: Implement persistent storage retrieval
        // For now, this is a placeholder
        Ok(None)
    }

    /// Update organization settings
    pub async fn update_organization(&self, org: &Organization) -> Result<()> {
        // Update isolation configuration if needed
        {
            let mut isolation_manager = self.isolation_manager.write().await;
            isolation_manager.remove_tenant(&org.id)?;
            isolation_manager.register_tenant(org)?;
        }
        
        // Update quota tracking
        self.quota_manager.update_organization(org).await?;
        
        Ok(())
    }

    /// Delete an organization
    pub async fn delete_organization(&self, org_id: &uuid::Uuid) -> Result<()> {
        // Remove from isolation manager
        {
            let mut isolation_manager = self.isolation_manager.write().await;
            isolation_manager.remove_tenant(org_id)?;
        }
        
        // Clean up quota tracking
        self.quota_manager.remove_organization(org_id).await?;
        
        // Remove all users from organization
        {
            let mut user_manager = self.user_manager.write().await;
            user_manager.remove_organization_users(org_id).await?;
        }
        
        Ok(())
    }

    /// Get tenant data access for an organization
    pub async fn get_tenant_access(&self, org_id: &uuid::Uuid) -> Result<TenantDataAccess> {
        let isolation_manager = self.isolation_manager.read().await;
        let access = isolation_manager.get_tenant_access(org_id)?;
        Ok(access.clone())
    }

    /// Check quota for an operation
    pub async fn check_quota(&self, org_id: &uuid::Uuid, operation: &QuotaOperation) -> Result<bool> {
        self.quota_manager.check_quota(org_id, operation).await
    }

    /// Validate tenant isolation across all organizations
    pub async fn validate_isolation(&self) -> Result<Vec<IsolationValidationResult>> {
        let isolation_manager = self.isolation_manager.read().await;
        isolation_manager.validate_isolation()
    }

    /// Get usage statistics for an organization
    pub async fn get_usage_stats(&self, org_id: &uuid::Uuid) -> Result<OrganizationUsage> {
        self.quota_manager.get_usage_stats(org_id).await
    }
}

impl Default for TenantManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tenant_manager_lifecycle() {
        let manager = TenantManager::new();
        
        // Create organization
        let org = manager
            .create_organization(
                "Test Company".to_string(),
                SubscriptionTier::Free {
                    storage_gb: 1,
                    api_calls_per_month: 10000,
                    max_collections: 5,
                },
            )
            .await
            .unwrap();
        
        // Get tenant access
        let access = manager.get_tenant_access(&org.id).await.unwrap();
        assert_eq!(access.get_isolation_config().org_id, org.id);
        
        // Check quota
        let can_create = manager
            .check_quota(&org.id, &QuotaOperation::CreateCollection)
            .await
            .unwrap();
        assert!(can_create);
        
        // Delete organization
        manager.delete_organization(&org.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_isolation_validation() {
        let manager = TenantManager::new();
        
        // Create two organizations
        let org1 = manager
            .create_organization(
                "Company One".to_string(),
                SubscriptionTier::Free {
                    storage_gb: 1,
                    api_calls_per_month: 10000,
                    max_collections: 5,
                },
            )
            .await
            .unwrap();
        
        let org2 = manager
            .create_organization(
                "Company Two".to_string(),
                SubscriptionTier::Free {
                    storage_gb: 1,
                    api_calls_per_month: 10000,
                    max_collections: 5,
                },
            )
            .await
            .unwrap();
        
        // Validate isolation
        let results = manager.validate_isolation().await.unwrap();
        assert_eq!(results.len(), 2);
        
        for result in results {
            assert!(result.is_valid, "Isolation validation failed: {:?}", result.issues);
        }
    }
}
