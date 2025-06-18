//! Comprehensive SaaS manager that orchestrates all SaaS features
//! 
//! Provides a unified interface for managing all SaaS infrastructure
//! including tenants, usage tracking, billing, quotas, and provisioning.

use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;
use tracing::{info, debug, warn, error};
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::tenant::*;
use crate::usage_tracker::*;
use crate::billing::*;
use crate::quotas::*;
use crate::provisioning::*;
use crate::sso::*;
use crate::analytics::*;
use crate::config::*;
use crate::auth::*;
use crate::tenant_isolation::*;
use crate::errors::*;

/// Main SaaS manager that coordinates all SaaS features
pub struct SaaSManager {
    /// Configuration
    config: SaaSConfig,
    
    /// Tenant management
    tenant_manager: Arc<TenantManager>,
    
    /// Usage tracking
    usage_tracker: Arc<UsageTracker>,
    
    /// Billing management
    billing_manager: Arc<BillingManager>,
    
    /// Quota management
    quota_manager: Arc<QuotaManager>,
    
    /// Provisioning management
    provisioning_manager: Arc<ProvisioningManager>,
    
    /// SSO management
    sso_manager: Arc<SSOManager>,
    
    /// Analytics management
    analytics_manager: Arc<AnalyticsManager>,
    
    /// Authentication management
    auth_manager: Arc<SaaSAuthManager>,
    
    /// Tenant isolation management
    isolation_manager: Arc<TenantIsolationManager>,
    
    /// Service health status
    service_health: Arc<RwLock<HashMap<String, ServiceHealth>>>,
    
    /// Background task handles
    background_tasks: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

/// Service health status
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    /// Service name
    pub service_name: String,
    
    /// Is service healthy
    pub is_healthy: bool,
    
    /// Last health check
    pub last_check: chrono::DateTime<chrono::Utc>,
    
    /// Error message if unhealthy
    pub error_message: Option<String>,
    
    /// Service metrics
    pub metrics: HashMap<String, f64>,
}

/// SaaS service status
#[derive(Debug, Clone)]
pub struct SaaSStatus {
    /// Overall health
    pub overall_health: bool,
    
    /// Individual service health
    pub services: HashMap<String, ServiceHealth>,
    
    /// Active tenants count
    pub active_tenants: u64,
    
    /// Total API calls today
    pub api_calls_today: u64,
    
    /// Total storage used (bytes)
    pub total_storage_bytes: u64,
    
    /// System metrics
    pub system_metrics: HashMap<String, f64>,
}

impl SaaSManager {
    /// Create new SaaS manager
    pub async fn new(config: SaaSConfig) -> Result<Self> {
        info!("🚀 Initializing SaaS manager");
        
        // Initialize tenant manager
        let tenant_manager = Arc::new(TenantManager::new().await?);
        
        // Initialize usage tracker
        let usage_tracker = Arc::new(UsageTrackerFactory::create_with_config(config.usage.clone())?);
        
        // Initialize billing manager
        let billing_manager = Arc::new(BillingManager::new(config.billing.clone()).await?);
        
        // Initialize quota manager
        let quota_manager = Arc::new(QuotaManager::new(config.quotas.clone()).await?);
        
        // Initialize provisioning manager
        let provisioning_manager = Arc::new(ProvisioningManager::new(config.provisioning.clone()).await?);
        
        // Initialize SSO manager
        let sso_manager = Arc::new(SSOManager::new(config.sso.clone()).await?);
        
        // Initialize analytics manager
        let analytics_manager = Arc::new(AnalyticsManager::new(config.analytics.clone()).await?);
        
        // Initialize authentication manager
        let auth_config = AuthConfig {
            jwt_secret: "your-jwt-secret-here".to_string(), // Should come from config
            token_expiration: chrono::Duration::hours(1),
            session_timeout: chrono::Duration::hours(24),
            enable_session_cleanup: true,
            cleanup_interval: chrono::Duration::minutes(15),
            max_sessions_per_user: 5,
            jwt_issuer: "aerolithdb".to_string(),
            jwt_audience: "aerolithdb-api".to_string(),
        };
        let auth_manager = Arc::new(SaaSAuthManager::new(Arc::clone(&tenant_manager), auth_config)?);
        
        // Initialize tenant isolation manager
        let isolation_manager = Arc::new(TenantIsolationManager::new(IsolationMode::SharedWithPrefix));
        
        Ok(Self {
            config,
            tenant_manager,
            usage_tracker,
            billing_manager,
            quota_manager,
            provisioning_manager,
            sso_manager,
            analytics_manager,
            auth_manager,
            isolation_manager,
            service_health: Arc::new(RwLock::new(HashMap::new())),
            background_tasks: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// Start all SaaS services
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting SaaS manager and all services");
        
        // Start individual services
        self.tenant_manager.start().await?;
        self.usage_tracker.start().await?;
        self.billing_manager.start().await?;
        self.quota_manager.start().await?;
        self.provisioning_manager.start().await?;
        self.sso_manager.start().await?;
        self.analytics_manager.start().await?;
        self.auth_manager.start().await?;
        self.isolation_manager.start().await?;
        
        // Start health monitoring
        self.start_health_monitoring().await?;
        
        // Start metrics collection
        self.start_metrics_collection().await?;
        
        info!("✅ All SaaS services started successfully");
        Ok(())
    }
    
    /// Stop all SaaS services
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping all SaaS services");
        
        // Cancel background tasks
        let mut tasks = self.background_tasks.write().await;
        for task in tasks.drain(..) {
            task.abort();
        }
        
        // Stop individual services
        self.isolation_manager.stop().await?;
        self.auth_manager.stop().await?;
        self.analytics_manager.stop().await?;
        self.sso_manager.stop().await?;
        self.provisioning_manager.stop().await?;
        self.quota_manager.stop().await?;
        self.billing_manager.stop().await?;
        self.usage_tracker.stop().await?;
        self.tenant_manager.stop().await?;
        
        info!("✅ All SaaS services stopped");
        Ok(())
    }
    
    /// Get SaaS status
    pub async fn get_status(&self) -> Result<SaaSStatus> {
        let service_health = self.service_health.read().await;
        let overall_health = service_health.values().all(|h| h.is_healthy);
        
        // Get active tenants count
        let active_tenants = self.tenant_manager.get_active_tenant_count().await?;
        
        // Get system metrics (simplified for demo)
        let mut system_metrics = HashMap::new();
        system_metrics.insert("uptime_seconds".to_string(), 3600.0); // Would be real uptime
        system_metrics.insert("memory_usage_mb".to_string(), 512.0); // Would be real memory usage
        system_metrics.insert("cpu_usage_percent".to_string(), 25.0); // Would be real CPU usage
        
        Ok(SaaSStatus {
            overall_health,
            services: service_health.clone(),
            active_tenants,
            api_calls_today: 0, // Would be aggregated from usage tracker
            total_storage_bytes: 0, // Would be aggregated from usage tracker
            system_metrics,
        })
    }
    
    /// Get tenant manager
    pub fn tenant_manager(&self) -> &Arc<TenantManager> {
        &self.tenant_manager
    }
    
    /// Get usage tracker
    pub fn usage_tracker(&self) -> &Arc<UsageTracker> {
        &self.usage_tracker
    }
    
    /// Get billing manager
    pub fn billing_manager(&self) -> &Arc<BillingManager> {
        &self.billing_manager
    }
    
    /// Get quota manager
    pub fn quota_manager(&self) -> &Arc<QuotaManager> {
        &self.quota_manager
    }
    
    /// Get provisioning manager
    pub fn provisioning_manager(&self) -> &Arc<ProvisioningManager> {
        &self.provisioning_manager
    }
    
    /// Get SSO manager
    pub fn sso_manager(&self) -> &Arc<SSOManager> {
        &self.sso_manager
    }
    
    /// Get analytics manager
    pub fn analytics_manager(&self) -> &Arc<AnalyticsManager> {
        &self.analytics_manager
    }
    
    /// Get authentication manager
    pub fn auth_manager(&self) -> &Arc<SaaSAuthManager> {
        &self.auth_manager
    }
    
    /// Get isolation manager
    pub fn isolation_manager(&self) -> &Arc<TenantIsolationManager> {
        &self.isolation_manager
    }
    
    /// Create a new tenant with all required setup
    pub async fn create_tenant_complete(
        &self,
        organization_name: String,
        organization_domain: Option<String>,
        subscription_tier: String,
        billing_info: Option<serde_json::Value>,
    ) -> Result<Tenant> {
        info!("🏢 Creating complete tenant setup for {}", organization_name);
        
        // Create tenant
        let mut tenant = self.tenant_manager.create_tenant_request(CreateTenantRequest {
            organization_name: organization_name.clone(),
            organization_domain: organization_domain.clone(),
            subscription_tier: subscription_tier.clone(),
            billing_info,
            metadata: HashMap::new(),
        }).await?;
        
        // Set up tenant isolation
        self.isolation_manager.register_tenant(tenant.clone(), None).await?;
        
        // Initialize quota tracking
        self.quota_manager.initialize_tenant_quotas(tenant.id).await?;
        
        // Set up billing
        if subscription_tier != "trial" {
            self.billing_manager.setup_tenant_billing(tenant.id).await?;
        }
        
        // Initialize analytics
        self.analytics_manager.initialize_tenant_analytics(tenant.id).await?;
        
        info!("✅ Complete tenant setup finished for {}", organization_name);
        Ok(tenant)
    }
    
    /// Delete a tenant and clean up all resources
    pub async fn delete_tenant_complete(&self, tenant_id: Uuid) -> Result<()> {
        info!("🗑️ Starting complete tenant deletion for {}", tenant_id);
        
        // Stop billing
        if let Err(e) = self.billing_manager.finalize_tenant_billing(tenant_id).await {
            warn!("⚠️ Failed to finalize billing for tenant {}: {}", tenant_id, e);
        }
        
        // Clean up analytics
        if let Err(e) = self.analytics_manager.cleanup_tenant_analytics(tenant_id).await {
            warn!("⚠️ Failed to cleanup analytics for tenant {}: {}", tenant_id, e);
        }
        
        // Clean up quotas
        if let Err(e) = self.quota_manager.cleanup_tenant_quotas(tenant_id).await {
            warn!("⚠️ Failed to cleanup quotas for tenant {}: {}", tenant_id, e);
        }
        
        // Clean up isolation
        if let Err(e) = self.isolation_manager.unregister_tenant(tenant_id).await {
            warn!("⚠️ Failed to cleanup isolation for tenant {}: {}", tenant_id, e);
        }
        
        // Delete tenant
        self.tenant_manager.delete_tenant(tenant_id).await?;
        
        info!("✅ Complete tenant deletion finished for {}", tenant_id);
        Ok(())
    }
    
    /// Process tenant usage event
    pub async fn process_tenant_usage_event(
        &self,
        tenant_id: Uuid,
        event_type: crate::usage_tracker::UsageEventType,
    ) -> Result<()> {
        // Track usage
        let event = crate::usage_tracker::UsageEvent {
            tenant_id,
            event_type: event_type.clone(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        
        self.usage_tracker.track_event(event).await?;
        
        // Check quotas
        match event_type {
            crate::usage_tracker::UsageEventType::ApiCall { .. } => {
                let allowed = self.quota_manager.check_quota_limit(
                    tenant_id,
                    "api_calls".to_string(),
                    1,
                ).await?;
                
                if !allowed {
                    warn!("⚠️ API call quota exceeded for tenant {}", tenant_id);
                    return Err(SaaSError::Quota(crate::errors::QuotaError::LimitExceeded {
                        tenant_id,
                        resource: "api_calls".to_string(),
                        limit: 0, // Would be actual limit
                        current: 0, // Would be actual usage
                    }));
                }
            },
            
            crate::usage_tracker::UsageEventType::StorageOperation { bytes_written, .. } => {
                let allowed = self.quota_manager.check_quota_limit(
                    tenant_id,
                    "storage".to_string(),
                    bytes_written,
                ).await?;
                
                if !allowed {
                    warn!("⚠️ Storage quota exceeded for tenant {}", tenant_id);
                    return Err(SaaSError::Quota(crate::errors::QuotaError::LimitExceeded {
                        tenant_id,
                        resource: "storage".to_string(),
                        limit: 0,
                        current: 0,
                    }));
                }
            },
            
            _ => {} // Other event types
        }
        
        Ok(())
    }
    
    /// Start health monitoring task
    async fn start_health_monitoring(&self) -> Result<()> {
        let service_health = Arc::clone(&self.service_health);
        
        let task = tokio::spawn(async move {
            info!("🔄 Starting SaaS health monitoring");
            
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let mut health = service_health.write().await;
                let now = chrono::Utc::now();
                
                // Check service health (simplified for demo)
                let services = vec![
                    "tenant_manager",
                    "usage_tracker", 
                    "billing_manager",
                    "quota_manager",
                    "provisioning_manager",
                    "sso_manager",
                    "analytics_manager",
                    "auth_manager",
                    "isolation_manager",
                ];
                
                for service in services {
                    health.insert(service.to_string(), ServiceHealth {
                        service_name: service.to_string(),
                        is_healthy: true, // Would perform actual health checks
                        last_check: now,
                        error_message: None,
                        metrics: HashMap::new(),
                    });
                }
                
                debug!("💚 Health check completed for {} services", health.len());
            }
        });
        
        self.background_tasks.write().await.push(task);
        Ok(())
    }
    
    /// Start metrics collection task
    async fn start_metrics_collection(&self) -> Result<()> {
        let analytics_manager = Arc::clone(&self.analytics_manager);
        
        let task = tokio::spawn(async move {
            info!("🔄 Starting SaaS metrics collection");
            
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // Collect and aggregate metrics
                if let Err(e) = analytics_manager.collect_system_metrics().await {
                    error!("❌ Failed to collect system metrics: {}", e);
                }
            }
        });
        
        self.background_tasks.write().await.push(task);
        Ok(())
    }
}

/// SaaS manager factory
pub struct SaaSManagerFactory;

impl SaaSManagerFactory {
    /// Create SaaS manager with default configuration
    pub async fn create_default() -> Result<SaaSManager> {
        let config = Self::default_config();
        SaaSManager::new(config).await
    }
    
    /// Create SaaS manager with custom configuration
    pub async fn create_with_config(config: SaaSConfig) -> Result<SaaSManager> {
        SaaSManager::new(config).await
    }
    
    /// Get default SaaS configuration
    pub fn default_config() -> SaaSConfig {
        SaaSConfig {
            tenant: TenantConfig {
                enabled: true,
                default_isolation_level: IsolationLevel::Shared,
                database_url: "sqlite://saas.db".to_string(),
                max_tenants_per_cluster: 1000,
                default_limits: TenantLimits {
                    max_storage_gb: 1,
                    max_api_calls_per_hour: 10000,
                    max_collections: 10,
                    max_users: 5,
                },
            },
            usage: UsageConfig {
                enabled: true,
                aggregation_interval_seconds: 60,
                cleanup_interval_seconds: 3600,
                retention_seconds: 86400 * 7, // 7 days
                batch_size: 1000,
                max_events_per_second: 10000,
            },
            billing: BillingConfig {
                enabled: true,
                currency: "USD".to_string(),
                tax_rate: 0.08,
                billing_cycle: BillingCycle::Monthly,
                grace_period_days: 7,
                auto_suspend_on_overdue: true,
                payment_methods: vec!["card".to_string(), "crypto".to_string()],
            },
            quotas: QuotaConfig {
                enabled: true,
                enforcement_mode: QuotaEnforcementMode::Strict,
                grace_period_seconds: 300,
                warning_threshold_percent: 80,
                auto_scale_enabled: false,
            },
            provisioning: ProvisioningConfig {
                enabled: true,
                auto_provisioning: true,
                default_cluster_size: 3,
                max_cluster_size: 10,
                provisioning_timeout_seconds: 300,
                health_check_interval_seconds: 30,
            },
            sso: SSOConfig {
                enabled: true,
                providers: vec![
                    SSOProvider {
                        name: "google".to_string(),
                        client_id: "".to_string(),
                        client_secret: "".to_string(),
                        enabled: false,
                    },
                    SSOProvider {
                        name: "github".to_string(),
                        client_id: "".to_string(),
                        client_secret: "".to_string(),
                        enabled: false,
                    },
                ],
                session_timeout_hours: 24,
                require_mfa: false,
            },
            analytics: AnalyticsConfig {
                enabled: true,
                retention_days: 90,
                real_time_enabled: true,
                aggregation_interval_seconds: 300,
                export_enabled: true,
                export_formats: vec!["json".to_string(), "csv".to_string()],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_saas_manager_creation() {
        let manager = SaaSManagerFactory::create_default().await.unwrap();
        assert!(manager.config.tenant.enabled);
        assert!(manager.config.usage.enabled);
        assert!(manager.config.billing.enabled);
    }
    
    #[tokio::test]
    async fn test_saas_manager_lifecycle() {
        let manager = SaaSManagerFactory::create_default().await.unwrap();
        
        // Start services
        manager.start().await.unwrap();
        
        // Get status
        let status = manager.get_status().await.unwrap();
        assert!(status.services.len() > 0);
        
        // Stop services
        manager.stop().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_complete_tenant_operations() {
        let manager = SaaSManagerFactory::create_default().await.unwrap();
        manager.start().await.unwrap();
        
        // Create tenant
        let tenant = manager.create_tenant_complete(
            "Test Organization".to_string(),
            Some("test.com".to_string()),
            "professional".to_string(),
            None,
        ).await.unwrap();
        
        assert_eq!(tenant.organization_name, "Test Organization");
        assert_eq!(tenant.subscription_tier, "professional");
        
        // Delete tenant
        manager.delete_tenant_complete(tenant.id).await.unwrap();
        
        manager.stop().await.unwrap();
    }
}
