//! Multi-tenancy infrastructure for AerolithDB SaaS
//! 
//! Provides organization-level data isolation, resource management, and tenant lifecycle management.

use crate::config::{TenantConfig, IsolationLevel, TenantLimits};
use crate::errors::{TenantError, TenantResult};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn, error};
use uuid::Uuid;

/// Tenant information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    /// Unique tenant identifier
    pub tenant_id: Uuid,
    
    /// Organization name
    pub organization_name: String,
    
    /// Organization domain (for email-based tenant resolution)
    pub organization_domain: Option<String>,
    
    /// Tenant isolation level
    pub isolation_level: IsolationLevel,
    
    /// Resource limits for this tenant
    pub limits: TenantLimits,
    
    /// Current resource usage
    pub current_usage: TenantUsage,
    
    /// Tenant status
    pub status: TenantStatus,
    
    /// Subscription tier
    pub subscription_tier: String,
    
    /// Billing information
    pub billing_info: BillingInfo,
    
    /// Metadata and custom properties
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Tenant expiration (for trial accounts)
    pub expires_at: Option<DateTime<Utc>>,
}

/// Current resource usage for a tenant
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantUsage {
    /// Current storage usage in bytes
    pub storage_bytes: u64,
    
    /// API calls in current hour
    pub api_calls_current_hour: u64,
    
    /// Current active connections
    pub active_connections: u32,
    
    /// Current number of collections
    pub collections_count: u32,
    
    /// Total number of documents across all collections
    pub total_documents: u64,
    
    /// Last usage update timestamp
    pub last_updated: DateTime<Utc>,
}

/// Tenant status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TenantStatus {
    /// Active and operational
    Active,
    
    /// Suspended due to quota violations or payment issues
    Suspended {
        reason: String,
        suspended_at: DateTime<Utc>,
    },
    
    /// Trial period
    Trial {
        trial_ends_at: DateTime<Utc>,
    },
    
    /// Deactivated (soft delete)
    Deactivated {
        reason: String,
        deactivated_at: DateTime<Utc>,
    },
    
    /// Pending activation (e.g., awaiting payment method)
    Pending {
        reason: String,
    },
}

/// Tenant billing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingInfo {
    /// Billing email
    pub billing_email: String,
    
    /// Billing address
    pub billing_address: Option<BillingAddress>,
    
    /// Payment method information (tokenized)
    pub payment_method: Option<String>,
    
    /// Billing cycle (monthly, yearly, etc.)
    pub billing_cycle: BillingCycle,
    
    /// Next billing date
    pub next_billing_date: DateTime<Utc>,
    
    /// Outstanding balance
    pub outstanding_balance: f64,
    
    /// Currency
    pub currency: String,
}

/// Billing address information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAddress {
    pub company: Option<String>,
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
}

/// Billing cycle options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BillingCycle {
    Monthly,
    Quarterly,
    Yearly,
}

/// Tenant management system
pub struct TenantManager {
    config: TenantConfig,
    db_pool: PgPool,
    tenant_cache: Arc<RwLock<HashMap<Uuid, Tenant>>>,
    domain_to_tenant: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl TenantManager {
    /// Create a new tenant manager
    pub async fn new(config: &TenantConfig) -> Result<Self> {
        info!("🏢 Initializing tenant manager");
        
        // Connect to database
        let db_pool = PgPool::connect(&config.database_url).await?;
        
        // Initialize database schema
        Self::initialize_schema(&db_pool).await?;
        
        let manager = Self {
            config: config.clone(),
            db_pool,
            tenant_cache: Arc::new(RwLock::new(HashMap::new())),
            domain_to_tenant: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Load existing tenants into cache
        manager.load_tenants_into_cache().await?;
        
        info!("✅ Tenant manager initialized successfully");
        Ok(manager)
    }
    
    /// Initialize database schema for tenant management
    async fn initialize_schema(pool: &PgPool) -> Result<()> {
        debug!("📊 Initializing tenant database schema");
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tenants (
                tenant_id UUID PRIMARY KEY,
                organization_name VARCHAR NOT NULL,
                organization_domain VARCHAR UNIQUE,
                isolation_level VARCHAR NOT NULL,
                limits JSONB NOT NULL,
                current_usage JSONB NOT NULL DEFAULT '{}',
                status JSONB NOT NULL,
                subscription_tier VARCHAR NOT NULL,
                billing_info JSONB NOT NULL,
                metadata JSONB NOT NULL DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                expires_at TIMESTAMPTZ
            );
            
            CREATE INDEX IF NOT EXISTS idx_tenants_organization_domain 
            ON tenants(organization_domain);
            
            CREATE INDEX IF NOT EXISTS idx_tenants_status 
            ON tenants((status->>'status'));
            
            CREATE INDEX IF NOT EXISTS idx_tenants_subscription_tier 
            ON tenants(subscription_tier);
            
            CREATE TABLE IF NOT EXISTS tenant_namespaces (
                tenant_id UUID REFERENCES tenants(tenant_id) ON DELETE CASCADE,
                namespace VARCHAR NOT NULL,
                collection_name VARCHAR NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (tenant_id, namespace, collection_name)
            );
            
            CREATE INDEX IF NOT EXISTS idx_tenant_namespaces_tenant 
            ON tenant_namespaces(tenant_id);
            "#
        )
        .execute(pool)
        .await?;
        
        debug!("✅ Tenant database schema initialized");
        Ok(())
    }
    
    /// Load existing tenants into cache
    async fn load_tenants_into_cache(&self) -> Result<()> {
        debug!("📚 Loading tenants into cache");
        
        let rows = sqlx::query("SELECT * FROM tenants WHERE status->>'status' = 'Active'")
            .fetch_all(&self.db_pool)
            .await?;
        
        let mut tenant_cache = self.tenant_cache.write().await;
        let mut domain_to_tenant = self.domain_to_tenant.write().await;
        
        for row in rows {
            let tenant = self.row_to_tenant(row)?;
            
            tenant_cache.insert(tenant.tenant_id, tenant.clone());
            
            if let Some(domain) = &tenant.organization_domain {
                domain_to_tenant.insert(domain.clone(), tenant.tenant_id);
            }
        }
        
        info!("📚 Loaded {} tenants into cache", tenant_cache.len());
        Ok(())
    }
    
    /// Convert database row to Tenant struct
    fn row_to_tenant(&self, row: sqlx::postgres::PgRow) -> Result<Tenant> {
        Ok(Tenant {
            tenant_id: row.try_get("tenant_id")?,
            organization_name: row.try_get("organization_name")?,
            organization_domain: row.try_get("organization_domain")?,
            isolation_level: serde_json::from_value(row.try_get("isolation_level")?)?,
            limits: serde_json::from_value(row.try_get("limits")?)?,
            current_usage: serde_json::from_value(row.try_get("current_usage")?)?,
            status: serde_json::from_value(row.try_get("status")?)?,
            subscription_tier: row.try_get("subscription_tier")?,
            billing_info: serde_json::from_value(row.try_get("billing_info")?)?,
            metadata: serde_json::from_value(row.try_get("metadata")?)?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            expires_at: row.try_get("expires_at")?,
        })
    }
    
    /// Create a new tenant
    pub async fn create_tenant(&self, request: CreateTenantRequest) -> TenantResult<Tenant> {
        info!("🆕 Creating new tenant: {}", request.organization_name);
        
        // Validate organization domain uniqueness
        if let Some(domain) = &request.organization_domain {
            if self.get_tenant_by_domain(domain).await?.is_some() {
                return Err(TenantError::AlreadyExists {
                    tenant_id: format!("domain:{}", domain),
                });
            }
        }
        
        let tenant_id = Uuid::new_v4();
        let now = Utc::now();
        
        let tenant = Tenant {
            tenant_id,
            organization_name: request.organization_name.clone(),
            organization_domain: request.organization_domain.clone(),
            isolation_level: request.isolation_level.unwrap_or(self.config.default_isolation_level.clone()),
            limits: request.limits.unwrap_or(self.config.default_limits.clone()),
            current_usage: TenantUsage::default(),
            status: request.initial_status.unwrap_or(TenantStatus::Active),
            subscription_tier: request.subscription_tier.unwrap_or_else(|| "starter".to_string()),
            billing_info: request.billing_info,
            metadata: request.metadata.unwrap_or_default(),
            created_at: now,
            updated_at: now,
            expires_at: request.expires_at,
        };
        
        // Insert into database
        sqlx::query(
            r#"
            INSERT INTO tenants (
                tenant_id, organization_name, organization_domain, isolation_level,
                limits, current_usage, status, subscription_tier, billing_info,
                metadata, created_at, updated_at, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#
        )
        .bind(tenant.tenant_id)
        .bind(&tenant.organization_name)
        .bind(&tenant.organization_domain)
        .bind(serde_json::to_value(&tenant.isolation_level)?)
        .bind(serde_json::to_value(&tenant.limits)?)
        .bind(serde_json::to_value(&tenant.current_usage)?)
        .bind(serde_json::to_value(&tenant.status)?)
        .bind(&tenant.subscription_tier)
        .bind(serde_json::to_value(&tenant.billing_info)?)
        .bind(serde_json::to_value(&tenant.metadata)?)
        .bind(tenant.created_at)
        .bind(tenant.updated_at)
        .bind(tenant.expires_at)
        .execute(&self.db_pool)
        .await
        .map_err(|e| TenantError::ResourceAllocationFailed {
            resource: format!("Database insertion: {}", e),
        })?;
        
        // Add to cache
        {
            let mut cache = self.tenant_cache.write().await;
            cache.insert(tenant_id, tenant.clone());
        }
        
        // Add domain mapping if provided
        if let Some(domain) = &tenant.organization_domain {
            let mut domain_map = self.domain_to_tenant.write().await;
            domain_map.insert(domain.clone(), tenant_id);
        }
        
        info!("✅ Created tenant: {} ({})", tenant.organization_name, tenant_id);
        Ok(tenant)
    }
    
    /// Get tenant by ID
    pub async fn get_tenant(&self, tenant_id: Uuid) -> TenantResult<Option<Tenant>> {
        // Check cache first
        {
            let cache = self.tenant_cache.read().await;
            if let Some(tenant) = cache.get(&tenant_id) {
                return Ok(Some(tenant.clone()));
            }
        }
        
        // Query database
        let row = sqlx::query("SELECT * FROM tenants WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| TenantError::ResourceAllocationFailed {
                resource: format!("Database query: {}", e),
            })?;
        
        if let Some(row) = row {
            let tenant = self.row_to_tenant(row).map_err(|e| TenantError::InvalidConfig {
                message: format!("Failed to parse tenant data: {}", e),
            })?;
            
            // Update cache
            {
                let mut cache = self.tenant_cache.write().await;
                cache.insert(tenant_id, tenant.clone());
            }
            
            Ok(Some(tenant))
        } else {
            Ok(None)
        }
    }
    
    /// Get tenant by organization domain
    pub async fn get_tenant_by_domain(&self, domain: &str) -> TenantResult<Option<Tenant>> {
        // Check domain mapping cache
        let tenant_id = {
            let domain_map = self.domain_to_tenant.read().await;
            domain_map.get(domain).copied()
        };
        
        if let Some(tenant_id) = tenant_id {
            return self.get_tenant(tenant_id).await;
        }
        
        // Query database
        let row = sqlx::query("SELECT * FROM tenants WHERE organization_domain = $1")
            .bind(domain)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| TenantError::ResourceAllocationFailed {
                resource: format!("Database query: {}", e),
            })?;
        
        if let Some(row) = row {
            let tenant = self.row_to_tenant(row).map_err(|e| TenantError::InvalidConfig {
                message: format!("Failed to parse tenant data: {}", e),
            })?;
            
            // Update caches
            {
                let mut cache = self.tenant_cache.write().await;
                cache.insert(tenant.tenant_id, tenant.clone());
            }
            {
                let mut domain_map = self.domain_to_tenant.write().await;
                domain_map.insert(domain.to_string(), tenant.tenant_id);
            }
            
            Ok(Some(tenant))
        } else {
            Ok(None)
        }
    }
    
    /// Update tenant information
    pub async fn update_tenant(&self, tenant_id: Uuid, updates: UpdateTenantRequest) -> TenantResult<Tenant> {
        let mut tenant = self.get_tenant(tenant_id)
            .await?
            .ok_or_else(|| TenantError::NotFound {
                tenant_id: tenant_id.to_string(),
            })?;
        
        // Apply updates
        if let Some(organization_name) = updates.organization_name {
            tenant.organization_name = organization_name;
        }
        if let Some(limits) = updates.limits {
            tenant.limits = limits;
        }
        if let Some(status) = updates.status {
            tenant.status = status;
        }
        if let Some(subscription_tier) = updates.subscription_tier {
            tenant.subscription_tier = subscription_tier;
        }
        if let Some(billing_info) = updates.billing_info {
            tenant.billing_info = billing_info;
        }
        if let Some(metadata) = updates.metadata {
            tenant.metadata = metadata;
        }
        if let Some(expires_at) = updates.expires_at {
            tenant.expires_at = expires_at;
        }
        
        tenant.updated_at = Utc::now();
        
        // Update database
        sqlx::query(
            r#"
            UPDATE tenants SET
                organization_name = $2,
                limits = $3,
                status = $4,
                subscription_tier = $5,
                billing_info = $6,
                metadata = $7,
                updated_at = $8,
                expires_at = $9
            WHERE tenant_id = $1
            "#
        )
        .bind(tenant_id)
        .bind(&tenant.organization_name)
        .bind(serde_json::to_value(&tenant.limits)?)
        .bind(serde_json::to_value(&tenant.status)?)
        .bind(&tenant.subscription_tier)
        .bind(serde_json::to_value(&tenant.billing_info)?)
        .bind(serde_json::to_value(&tenant.metadata)?)
        .bind(tenant.updated_at)
        .bind(tenant.expires_at)
        .execute(&self.db_pool)
        .await
        .map_err(|e| TenantError::ResourceAllocationFailed {
            resource: format!("Database update: {}", e),
        })?;
        
        // Update cache
        {
            let mut cache = self.tenant_cache.write().await;
            cache.insert(tenant_id, tenant.clone());
        }
        
        info!("✅ Updated tenant: {} ({})", tenant.organization_name, tenant_id);
        Ok(tenant)
    }
    
    /// Update tenant usage statistics
    pub async fn update_tenant_usage(&self, tenant_id: Uuid, usage: TenantUsage) -> TenantResult<()> {
        sqlx::query("UPDATE tenants SET current_usage = $2, updated_at = $3 WHERE tenant_id = $1")
            .bind(tenant_id)
            .bind(serde_json::to_value(&usage)?)
            .bind(Utc::now())
            .execute(&self.db_pool)
            .await
            .map_err(|e| TenantError::ResourceAllocationFailed {
                resource: format!("Usage update: {}", e),
            })?;
        
        // Update cache if tenant is loaded
        {
            let mut cache = self.tenant_cache.write().await;
            if let Some(tenant) = cache.get_mut(&tenant_id) {
                tenant.current_usage = usage;
                tenant.updated_at = Utc::now();
            }
        }
        
        Ok(())
    }
    
    /// List all tenants with pagination
    pub async fn list_tenants(&self, limit: Option<u32>, offset: Option<u32>) -> TenantResult<Vec<Tenant>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        
        let rows = sqlx::query("SELECT * FROM tenants ORDER BY created_at DESC LIMIT $1 OFFSET $2")
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| TenantError::ResourceAllocationFailed {
                resource: format!("Database query: {}", e),
            })?;
        
        let mut tenants = Vec::new();
        for row in rows {
            let tenant = self.row_to_tenant(row).map_err(|e| TenantError::InvalidConfig {
                message: format!("Failed to parse tenant data: {}", e),
            })?;
            tenants.push(tenant);
        }
        
        Ok(tenants)
    }
    
    /// Generate namespace for tenant collection
    pub fn get_tenant_namespace(&self, tenant_id: Uuid, collection: &str) -> String {
        format!("tenant_{}_{}", tenant_id.simple(), collection)
    }
    
    /// Validate tenant access to resource
    pub async fn validate_tenant_access(&self, tenant_id: Uuid, resource: &str) -> TenantResult<bool> {
        let tenant = self.get_tenant(tenant_id)
            .await?
            .ok_or_else(|| TenantError::NotFound {
                tenant_id: tenant_id.to_string(),
            })?;
        
        // Check tenant status
        match tenant.status {
            TenantStatus::Active => {},
            TenantStatus::Trial { trial_ends_at } => {
                if Utc::now() > trial_ends_at {
                    return Err(TenantError::LimitExceeded {
                        limit_type: "Trial period expired".to_string(),
                    });
                }
            },
            TenantStatus::Suspended { reason, .. } => {
                return Err(TenantError::IsolationViolation {
                    message: format!("Tenant suspended: {}", reason),
                });
            },
            TenantStatus::Deactivated { reason, .. } => {
                return Err(TenantError::IsolationViolation {
                    message: format!("Tenant deactivated: {}", reason),
                });
            },
            TenantStatus::Pending { reason } => {
                return Err(TenantError::IsolationViolation {
                    message: format!("Tenant pending: {}", reason),
                });
            },
        }
        
        // Check expiration
        if let Some(expires_at) = tenant.expires_at {
            if Utc::now() > expires_at {
                return Err(TenantError::LimitExceeded {
                    limit_type: "Tenant expired".to_string(),
                });
            }
        }
        
        Ok(true)
    }
}

/// Request structure for creating a new tenant
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantRequest {
    pub organization_name: String,
    pub organization_domain: Option<String>,
    pub isolation_level: Option<IsolationLevel>,
    pub limits: Option<TenantLimits>,
    pub initial_status: Option<TenantStatus>,
    pub subscription_tier: Option<String>,
    pub billing_info: BillingInfo,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Request structure for updating a tenant
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTenantRequest {
    pub organization_name: Option<String>,
    pub limits: Option<TenantLimits>,
    pub status: Option<TenantStatus>,
    pub subscription_tier: Option<String>,
    pub billing_info: Option<BillingInfo>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub expires_at: Option<Option<DateTime<Utc>>>,
}
