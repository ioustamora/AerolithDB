//! Tenant isolation and data segregation implementation
//! 
//! Provides data isolation mechanisms for multi-tenant operations,
//! ensuring complete separation of tenant data and resources.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn, error};
use anyhow::Result;

use crate::tenant::*;
use crate::errors::{TenantError, TenantResult};

/// Tenant isolation modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationMode {
    /// Shared database with tenant prefix
    SharedWithPrefix,
    
    /// Separate schema per tenant
    SeparateSchema,
    
    /// Separate database per tenant
    SeparateDatabase,
    
    /// Separate cluster per tenant
    SeparateCluster,
}

/// Tenant data context for operations
#[derive(Debug, Clone)]
pub struct TenantContext {
    /// Tenant information
    pub tenant: Tenant,
    
    /// Isolation mode
    pub isolation_mode: IsolationMode,
    
    /// Database/schema identifier
    pub database_identifier: String,
    
    /// Collection prefix (if using shared mode)
    pub collection_prefix: Option<String>,
    
    /// Resource limits
    pub resource_limits: ResourceLimits,
    
    /// Current resource usage
    pub current_usage: ResourceUsage,
}

/// Resource limits for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum storage in bytes
    pub max_storage_bytes: u64,
    
    /// Maximum API calls per hour
    pub max_api_calls_per_hour: u64,
    
    /// Maximum concurrent connections
    pub max_concurrent_connections: u32,
    
    /// Maximum documents per collection
    pub max_documents_per_collection: u64,
    
    /// Maximum collections
    pub max_collections: u32,
    
    /// Maximum query execution time in milliseconds
    pub max_query_execution_ms: u64,
}

/// Current resource usage for a tenant
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    /// Current storage usage in bytes
    pub storage_bytes: u64,
    
    /// API calls in current hour
    pub api_calls_current_hour: u64,
    
    /// Current active connections
    pub active_connections: u32,
    
    /// Total documents across all collections
    pub total_documents: u64,
    
    /// Number of collections
    pub collection_count: u32,
}

/// Tenant isolation manager
pub struct TenantIsolationManager {
    /// Tenant contexts by ID
    contexts: Arc<RwLock<HashMap<Uuid, TenantContext>>>,
    
    /// Default isolation mode
    default_isolation_mode: IsolationMode,
    
    /// Background task handles
    background_tasks: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

impl TenantIsolationManager {
    /// Create new tenant isolation manager
    pub fn new(default_isolation_mode: IsolationMode) -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            default_isolation_mode,
            background_tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Start the isolation manager
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting tenant isolation manager");
        
        // Start resource monitoring task
        self.start_resource_monitoring().await?;
        
        info!("✅ Tenant isolation manager started");
        Ok(())
    }
    
    /// Stop the isolation manager
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping tenant isolation manager");
        
        // Cancel all background tasks
        let mut tasks = self.background_tasks.write().await;
        for task in tasks.drain(..) {
            task.abort();
        }
        
        info!("✅ Tenant isolation manager stopped");
        Ok(())
    }
    
    /// Register a tenant for isolation
    pub async fn register_tenant(&self, tenant: Tenant, isolation_mode: Option<IsolationMode>) -> Result<TenantContext> {
        let isolation_mode = isolation_mode.unwrap_or_else(|| self.default_isolation_mode.clone());
        
        info!("🏢 Registering tenant {} with isolation mode {:?}", tenant.tenant_id, isolation_mode);
        
        let context = self.create_tenant_context(tenant, isolation_mode).await?;
        
        // Store the context
        let mut contexts = self.contexts.write().await;
        contexts.insert(context.tenant.tenant_id, context.clone());
        
        info!("✅ Tenant {} registered successfully", context.tenant.tenant_id);
        Ok(context)
    }
    
    /// Unregister a tenant
    pub async fn unregister_tenant(&self, tenant_id: Uuid) -> Result<()> {
        info!("🗑️ Unregistering tenant {}", tenant_id);
        
        let mut contexts = self.contexts.write().await;
        if let Some(context) = contexts.remove(&tenant_id) {
            // Perform cleanup based on isolation mode
            self.cleanup_tenant_resources(&context).await?;
            info!("✅ Tenant {} unregistered successfully", tenant_id);
        } else {
            warn!("⚠️ Tenant {} not found for unregistration", tenant_id);
        }
        
        Ok(())
    }
    
    /// Get tenant context
    pub async fn get_tenant_context(&self, tenant_id: Uuid) -> Result<Option<TenantContext>> {
        let contexts = self.contexts.read().await;
        Ok(contexts.get(&tenant_id).cloned())
    }
    
    /// Update tenant resource usage
    pub async fn update_resource_usage(&self, tenant_id: Uuid, usage: ResourceUsage) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        if let Some(context) = contexts.get_mut(&tenant_id) {
            context.current_usage = usage;
            
            // Check for limit violations
            self.check_resource_limits(context)?;
            
            debug!("📊 Updated resource usage for tenant {}", tenant_id);
        }
        
        Ok(())
    }
    
    /// Check if operation is allowed for tenant
    pub async fn check_operation_allowed(
        &self,
        tenant_id: Uuid,
        operation: &TenantOperation,
    ) -> Result<bool> {
        let contexts = self.contexts.read().await;        if let Some(context) = contexts.get(&tenant_id) {
            self.validate_operation(context, operation)
        } else {
            Err(TenantError::NotFound { tenant_id: tenant_id.to_string() }.into())
        }
    }
    
    /// Get isolated collection name for tenant
    pub async fn get_isolated_collection_name(
        &self,
        tenant_id: Uuid,
        collection_name: &str,
    ) -> Result<String> {
        let contexts = self.contexts.read().await;
        if let Some(context) = contexts.get(&tenant_id) {
            match context.isolation_mode {
                IsolationMode::SharedWithPrefix => {
                    if let Some(prefix) = &context.collection_prefix {
                        Ok(format!("{}_{}", prefix, collection_name))
                    } else {
                        Ok(format!("tenant_{}_{}", tenant_id, collection_name))
                    }
                },
                _ => Ok(collection_name.to_string()),
            }        } else {
            Err(TenantError::NotFound { tenant_id: tenant_id.to_string() }.into())
        }
    }
    
    /// Get database identifier for tenant
    pub async fn get_database_identifier(&self, tenant_id: Uuid) -> Result<String> {
        let contexts = self.contexts.read().await;
        if let Some(context) = contexts.get(&tenant_id) {
            Ok(context.database_identifier.clone())        } else {
            Err(TenantError::NotFound { tenant_id: tenant_id.to_string() }.into())
        }
    }
    
    /// Create tenant context
    async fn create_tenant_context(
        &self,
        tenant: Tenant,
        isolation_mode: IsolationMode,
    ) -> Result<TenantContext> {
        let database_identifier = match isolation_mode {
            IsolationMode::SharedWithPrefix => "shared_db".to_string(),            IsolationMode::SeparateSchema => format!("schema_{}", tenant.tenant_id),
            IsolationMode::SeparateDatabase => format!("db_{}", tenant.tenant_id),
            IsolationMode::SeparateCluster => format!("cluster_{}", tenant.tenant_id),
        };
        
        let collection_prefix = match isolation_mode {
            IsolationMode::SharedWithPrefix => Some(format!("t_{}", tenant.tenant_id.to_string().replace('-', ""))),
            _ => None,
        };
        
        // Get resource limits based on subscription tier
        let resource_limits = self.get_resource_limits_for_tier(&tenant.subscription_tier)?;
        
        Ok(TenantContext {
            tenant,
            isolation_mode,
            database_identifier,
            collection_prefix,
            resource_limits,
            current_usage: ResourceUsage::default(),
        })
    }
    
    /// Get resource limits for subscription tier
    fn get_resource_limits_for_tier(&self, tier: &str) -> Result<ResourceLimits> {
        match tier.to_lowercase().as_str() {
            "starter" => Ok(ResourceLimits {
                max_storage_bytes: 1_000_000_000, // 1GB
                max_api_calls_per_hour: 10_000,
                max_concurrent_connections: 10,
                max_documents_per_collection: 100_000,
                max_collections: 10,
                max_query_execution_ms: 5_000,
            }),
            
            "professional" => Ok(ResourceLimits {
                max_storage_bytes: 10_000_000_000, // 10GB
                max_api_calls_per_hour: 100_000,
                max_concurrent_connections: 50,
                max_documents_per_collection: 1_000_000,
                max_collections: 100,
                max_query_execution_ms: 10_000,
            }),
            
            "enterprise" => Ok(ResourceLimits {
                max_storage_bytes: 100_000_000_000, // 100GB
                max_api_calls_per_hour: 1_000_000,
                max_concurrent_connections: 200,
                max_documents_per_collection: 10_000_000,
                max_collections: 1000,
                max_query_execution_ms: 30_000,
            }),
            
            _ => Ok(ResourceLimits {
                max_storage_bytes: 100_000_000, // 100MB
                max_api_calls_per_hour: 1_000,
                max_concurrent_connections: 5,
                max_documents_per_collection: 10_000,
                max_collections: 5,
                max_query_execution_ms: 1_000,
            }),
        }
    }
    
    /// Check resource limits for context
    fn check_resource_limits(&self, context: &TenantContext) -> Result<()> {
        let usage = &context.current_usage;
        let limits = &context.resource_limits;
        
        if usage.storage_bytes > limits.max_storage_bytes {            return Err(TenantError::ResourceLimitExceeded { 
                resource: "Storage limit exceeded".to_string() 
            }.into());
        }
        
        if usage.api_calls_current_hour > limits.max_api_calls_per_hour {            return Err(TenantError::ResourceLimitExceeded { 
                resource: "API call limit exceeded".to_string() 
            }.into());
        }
        
        if usage.active_connections > limits.max_concurrent_connections {            return Err(TenantError::ResourceLimitExceeded { 
                resource: "Connection limit exceeded".to_string() 
            }.into());
        }
        
        if usage.total_documents > limits.max_documents_per_collection * limits.max_collections as u64 {            return Err(TenantError::ResourceLimitExceeded { 
                resource: "Document limit exceeded".to_string() 
            }.into());
        }
        
        if usage.collection_count > limits.max_collections {            return Err(TenantError::ResourceLimitExceeded { 
                resource: "Collection limit exceeded".to_string() 
            }.into());
        }
        
        Ok(())
    }
    
    /// Validate tenant operation
    fn validate_operation(&self, context: &TenantContext, operation: &TenantOperation) -> Result<bool> {
        match operation {
            TenantOperation::CreateCollection => {
                Ok(context.current_usage.collection_count < context.resource_limits.max_collections)
            },
            
            TenantOperation::CreateDocument => {
                Ok(context.current_usage.total_documents < 
                   context.resource_limits.max_documents_per_collection * context.resource_limits.max_collections as u64)
            },
            
            TenantOperation::ApiCall => {
                Ok(context.current_usage.api_calls_current_hour < context.resource_limits.max_api_calls_per_hour)
            },
            
            TenantOperation::Connect => {
                Ok(context.current_usage.active_connections < context.resource_limits.max_concurrent_connections)
            },
            
            TenantOperation::Query { estimated_execution_ms } => {
                Ok(*estimated_execution_ms <= context.resource_limits.max_query_execution_ms)
            },
            
            TenantOperation::StorageWrite { bytes } => {
                Ok(context.current_usage.storage_bytes + bytes <= context.resource_limits.max_storage_bytes)
            },
        }
    }
    
    /// Cleanup tenant resources
    async fn cleanup_tenant_resources(&self, context: &TenantContext) -> Result<()> {
        match context.isolation_mode {
            IsolationMode::SharedWithPrefix => {
                // Clean up prefixed collections
                info!("🧹 Cleaning up prefixed collections for tenant {}", context.tenant.tenant_id);
                // Implementation would delete all collections with the tenant prefix
            },
            
            IsolationMode::SeparateSchema => {
                // Drop schema
                info!("🧹 Dropping schema for tenant {}", context.tenant.tenant_id);
                // Implementation would drop the entire schema
            },
            
            IsolationMode::SeparateDatabase => {
                // Drop database
                info!("🧹 Dropping database for tenant {}", context.tenant.tenant_id);
                // Implementation would drop the entire database
            },
            
            IsolationMode::SeparateCluster => {
                // Cleanup cluster
                info!("🧹 Cleaning up cluster for tenant {}", context.tenant.tenant_id);
                // Implementation would cleanup/destroy the dedicated cluster
            },
        }
        
        Ok(())
    }
    
    /// Start resource monitoring task
    async fn start_resource_monitoring(&self) -> Result<()> {
        let contexts = Arc::clone(&self.contexts);
        
        let task = tokio::spawn(async move {
            info!("🔄 Starting resource monitoring task");
            
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let contexts_read = contexts.read().await;
                for (tenant_id, context) in contexts_read.iter() {
                    // Log resource usage
                    debug!("📊 Tenant {} resource usage: storage={}B, api_calls={}, connections={}", 
                           tenant_id,
                           context.current_usage.storage_bytes,
                           context.current_usage.api_calls_current_hour,
                           context.current_usage.active_connections);
                    
                    // Check for approaching limits
                    let storage_percent = (context.current_usage.storage_bytes as f64 / 
                                         context.resource_limits.max_storage_bytes as f64) * 100.0;
                    
                    if storage_percent > 80.0 {
                        warn!("⚠️ Tenant {} approaching storage limit: {:.1}%", tenant_id, storage_percent);
                    }
                    
                    let api_percent = (context.current_usage.api_calls_current_hour as f64 / 
                                     context.resource_limits.max_api_calls_per_hour as f64) * 100.0;
                    
                    if api_percent > 80.0 {
                        warn!("⚠️ Tenant {} approaching API call limit: {:.1}%", tenant_id, api_percent);
                    }
                }
            }
        });
        
        self.background_tasks.write().await.push(task);
        Ok(())
    }
}

/// Types of tenant operations for validation
#[derive(Debug, Clone)]
pub enum TenantOperation {
    /// Create a new collection
    CreateCollection,
    
    /// Create a new document
    CreateDocument,
    
    /// Make an API call
    ApiCall,
    
    /// Establish a connection
    Connect,
    
    /// Execute a query
    Query { estimated_execution_ms: u64 },
    
    /// Write data to storage
    StorageWrite { bytes: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::TenantStatus;
    
    #[tokio::test]
    async fn test_tenant_isolation_manager() {
        let manager = TenantIsolationManager::new(IsolationMode::SharedWithPrefix);
        manager.start().await.unwrap();
        
        let tenant = Tenant {
            id: Uuid::new_v4(),
            organization_name: "Test Org".to_string(),
            organization_domain: Some("test.com".to_string()),
            subscription_tier: "starter".to_string(),
            status: TenantStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };
        
        // Register tenant
        let context = manager.register_tenant(tenant.clone(), None).await.unwrap();
        assert_eq!(context.tenant.id, tenant.id);
        assert!(matches!(context.isolation_mode, IsolationMode::SharedWithPrefix));
        
        // Test collection name isolation
        let isolated_name = manager.get_isolated_collection_name(tenant.id, "users").await.unwrap();
        assert!(isolated_name.contains("users"));
        assert!(isolated_name.contains(&tenant.id.to_string().replace('-', "")));
        
        // Test operation validation
        let allowed = manager.check_operation_allowed(
            tenant.id,
            &TenantOperation::CreateCollection,
        ).await.unwrap();
        assert!(allowed);
        
        manager.stop().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_resource_limits() {
        let manager = TenantIsolationManager::new(IsolationMode::SeparateSchema);
        manager.start().await.unwrap();
        
        let tenant = Tenant {
            id: Uuid::new_v4(),
            organization_name: "Test Org".to_string(),
            organization_domain: None,
            subscription_tier: "starter".to_string(),
            status: TenantStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };
        
        let context = manager.register_tenant(tenant.clone(), None).await.unwrap();
        
        // Test storage limit
        let large_write = TenantOperation::StorageWrite { 
            bytes: context.resource_limits.max_storage_bytes + 1 
        };
        let allowed = manager.check_operation_allowed(tenant.id, &large_write).await.unwrap();
        assert!(!allowed);
        
        manager.stop().await.unwrap();
    }
}
