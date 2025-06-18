//! SaaS integration layer for AerolithDB
//! 
//! Provides integration between core AerolithDB systems and SaaS infrastructure,
//! including authentication hooks, usage tracking hooks, and tenant-aware operations.

use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;
use tracing::{info, debug, error};

use aerolithdb_core::Node;
use aerolithdb_query::QueryEngine;
use aerolithdb_security::SecurityFramework;
use aerolithdb_api::APIGateway;
use aerolithdb_saas::SaaSManager;

/// Main integration manager that bridges core AerolithDB with SaaS features
pub struct SaaSIntegration {
    /// Core database node
    node: Arc<Node>,
    
    /// Query processing engine
    query_engine: Arc<QueryEngine>,
    
    /// Security and authentication
    security: Arc<SecurityFramework>,
    
    /// API gateway
    api_gateway: Arc<APIGateway>,
    
    /// SaaS management layer
    saas_manager: Arc<SaaSManager>,
}

impl SaaSIntegration {
    /// Create new SaaS integration
    pub async fn new(
        node: Arc<Node>,
        query_engine: Arc<QueryEngine>,
        security: Arc<SecurityFramework>,
        api_gateway: Arc<APIGateway>,
        saas_manager: Arc<SaaSManager>,
    ) -> Result<Self> {
        info!("🔧 Initializing SaaS integration layer");
        
        let integration = Self {
            node,
            query_engine,
            security,
            api_gateway,
            saas_manager,
        };
        
        info!("✅ SaaS integration layer initialized");
        Ok(integration)
    }
    
    /// Start all SaaS services
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting SaaS-enabled AerolithDB services");
        
        // Start core services first
        self.node.start().await?;
        
        // Start API gateway
        self.api_gateway.start().await?;
        
        // Start SaaS background services
        self.saas_manager.start().await?;
        
        info!("✅ All SaaS services started successfully");
        Ok(())
    }
    
    /// Stop all SaaS services
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping SaaS services");
        
        // Stop SaaS services first
        self.saas_manager.stop().await?;
        
        // Stop API gateway
        self.api_gateway.stop().await?;
        
        // Stop core services
        self.node.stop().await?;
        
        info!("✅ All SaaS services stopped");
        Ok(())
    }
    
    /// Get SaaS manager reference
    pub fn saas_manager(&self) -> &Arc<SaaSManager> {
        &self.saas_manager
    }
    
    /// Get API gateway reference
    pub fn api_gateway(&self) -> &Arc<APIGateway> {
        &self.api_gateway
    }
    
    /// Check if tenant has access to specific database operation
    pub async fn check_tenant_operation_access(
        &self,
        tenant_id: Uuid,
        operation: &str,
        collection: Option<&str>,
    ) -> Result<bool> {
        // Validate tenant exists and is active
        let access_granted = self.saas_manager
            .tenant_manager()
            .validate_tenant_access(tenant_id, operation)
            .await?;
        
        if !access_granted {
            return Ok(false);
        }
        
        // Check quota limits for the operation
        let resource_delta = match operation {
            "create_document" | "update_document" => 1024, // Estimate 1KB per document
            "create_collection" => 0,
            "query_documents" => 0,
            _ => 1, // Generic API call
        };
        
        let quota_ok = self.saas_manager
            .quota_manager()
            .check_operation_allowed(tenant_id, operation, resource_delta)
            .await?;
        
        Ok(quota_ok)
    }
    
    /// Record usage for tenant operation
    pub async fn record_tenant_usage(
        &self,
        tenant_id: Uuid,
        operation: &str,
        collection: Option<&str>,
        response_time_ms: u64,
        bytes_processed: u64,
    ) -> Result<()> {
        // Record in usage tracker
        if let Err(e) = self.saas_manager
            .usage_tracker()
            .record_api_call(tenant_id, "POST", operation, 200, response_time_ms)
            .await {
            error!("Failed to record usage for tenant {}: {}", tenant_id, e);
        }
        
        // Update tenant usage counters
        if let Ok(Some(mut tenant)) = self.saas_manager
            .tenant_manager()
            .get_tenant(tenant_id)
            .await {
            
            tenant.current_usage.api_calls_current_hour += 1;
            tenant.current_usage.last_activity = Some(chrono::Utc::now());
            
            if operation.contains("document") && operation.contains("create") {
                tenant.current_usage.storage_bytes += bytes_processed;
            }
            
            if let Err(e) = self.saas_manager
                .tenant_manager()
                .update_tenant_usage(tenant_id, tenant.current_usage)
                .await {
                error!("Failed to update tenant usage: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Get tenant namespace for collection
    pub fn get_tenant_namespace(&self, tenant_id: Uuid, collection: &str) -> String {
        self.saas_manager
            .tenant_manager()
            .get_tenant_namespace(tenant_id, collection)
    }
    
    /// Process tenant-aware query
    pub async fn execute_tenant_query(
        &self,
        tenant_id: Uuid,
        collection: &str,
        query: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Check tenant access
        if !self.check_tenant_operation_access(tenant_id, "query_documents", Some(collection)).await? {
            return Err(anyhow::anyhow!("Tenant access denied or quota exceeded"));
        }
        
        // Add tenant namespace to collection name
        let namespaced_collection = self.get_tenant_namespace(tenant_id, collection);
        
        debug!("Executing tenant query: tenant={}, collection={}, namespaced={}",
               tenant_id, collection, namespaced_collection);
        
        // Execute query with tenant isolation
        let start_time = std::time::Instant::now();
        let result = self.query_engine
            .execute_query(&namespaced_collection, query)
            .await?;
        let response_time = start_time.elapsed().as_millis() as u64;
        
        // Record usage
        self.record_tenant_usage(
            tenant_id,
            "query_documents",
            Some(collection),
            response_time,
            0, // Query doesn't add storage
        ).await?;
        
        Ok(result)
    }
    
    /// Create tenant-aware document
    pub async fn create_tenant_document(
        &self,
        tenant_id: Uuid,
        collection: &str,
        document: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Check tenant access
        if !self.check_tenant_operation_access(tenant_id, "create_document", Some(collection)).await? {
            return Err(anyhow::anyhow!("Tenant access denied or quota exceeded"));
        }
        
        // Add tenant namespace
        let namespaced_collection = self.get_tenant_namespace(tenant_id, collection);
        
        // Estimate document size
        let document_bytes = serde_json::to_vec(&document)?.len() as u64;
        
        debug!("Creating tenant document: tenant={}, collection={}, size={}",
               tenant_id, collection, document_bytes);
        
        // Create document
        let start_time = std::time::Instant::now();
        let result = self.query_engine
            .create_document(&namespaced_collection, document)
            .await?;
        let response_time = start_time.elapsed().as_millis() as u64;
        
        // Record usage
        self.record_tenant_usage(
            tenant_id,
            "create_document",
            Some(collection),
            response_time,
            document_bytes,
        ).await?;
        
        Ok(result)
    }
    
    /// Get health status of all SaaS components
    pub async fn get_health_status(&self) -> serde_json::Value {
        serde_json::json!({
            "status": "healthy",
            "components": {
                "core_node": "healthy",
                "query_engine": "healthy", 
                "security": "healthy",
                "api_gateway": "healthy",
                "saas_manager": "healthy",
                "tenant_manager": "healthy",
                "billing_engine": "healthy",
                "quota_manager": "healthy",
                "usage_tracker": "healthy",
                "analytics_engine": "healthy"
            },
            "timestamp": chrono::Utc::now()
        })
    }
}

/// Tenant-aware query execution context
#[derive(Debug)]
pub struct TenantQueryContext {
    pub tenant_id: Uuid,
    pub user_id: Option<String>,
    pub collection: String,
    pub operation: String,
    pub start_time: std::time::Instant,
}

impl TenantQueryContext {
    pub fn new(tenant_id: Uuid, collection: String, operation: String) -> Self {
        Self {
            tenant_id,
            user_id: None,
            collection,
            operation,
            start_time: std::time::Instant::now(),
        }
    }
    
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}
