//! # AerolithDB SaaS/DBaaS Infrastructure
//! 
//! This module provides comprehensive SaaS (Software-as-a-Service) and DBaaS 
//! (Database-as-a-Service) capabilities for AerolithDB, enabling multi-tenant
//! operations, usage tracking, billing, and self-service provisioning.
//! 
//! ## Core Features
//! 
//! - **Multi-Tenancy**: Organization-level data isolation and resource management
//! - **Usage Tracking**: Comprehensive API call, storage, and compute metering
//! - **Billing Integration**: Automated billing calculation and invoice generation
//! - **Resource Quotas**: Configurable limits and usage enforcement
//! - **Self-Service Provisioning**: Automated cluster deployment and scaling
//! - **Enterprise SSO**: SAML, OAuth2, LDAP integration
//! 
//! ## Architecture
//! 
//! The SaaS infrastructure is built on top of AerolithDB's existing distributed
//! architecture and adds the following layers:
//! 
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                 SaaS Management Layer                   │
//! ├─────────────────┬─────────────────┬─────────────────────┤
//! │  Multi-Tenancy  │  Usage Tracking │  Billing & Quotas  │
//! ├─────────────────┼─────────────────┼─────────────────────┤
//! │  Provisioning   │  Enterprise SSO │  Analytics          │
//! └─────────────────┴─────────────────┴─────────────────────┘
//! ┌─────────────────────────────────────────────────────────┐
//! │              AerolithDB Core Platform                   │
//! │  (Storage, Consensus, Security, Query, API)             │
//! └─────────────────────────────────────────────────────────┘
//! ```

pub mod tenant;
pub mod usage;
pub mod usage_tracker;
pub mod billing;
pub mod quotas;
pub mod provisioning;
pub mod sso;
pub mod analytics;
pub mod config;
pub mod errors;
pub mod auth;
pub mod tenant_isolation;
pub mod manager;
pub mod subscription;
pub mod production_metering;

// Re-export main types for convenience
pub use tenant::*;
pub use usage::*;
pub use usage_tracker::{UsageTracker as UsageTrackerImpl, UsageEvent}; 
pub use billing::*;
pub use quotas::*;
pub use provisioning::*;
pub use sso::*;
pub use analytics::*;
pub use config::*;
pub use errors::*;
pub use auth::*;
pub use tenant_isolation::*;
pub use manager::*;
pub use subscription::*;
pub use production_metering::*;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

/// Main SaaS manager that orchestrates all SaaS-related functionality
/// 
/// This is the central coordination point for multi-tenancy, billing,
/// usage tracking, and all other SaaS features.
pub struct SaaSManager {
    /// SaaS configuration settings
    config: SaaSConfig,
    
    /// Tenant management system
    tenant_manager: Arc<TenantManager>,
      /// Usage tracking and metering
    usage_tracker: Arc<UsageTrackerImpl>,
    
    /// Billing calculation engine
    billing_engine: Arc<BillingEngine>,
    
    /// Resource quota enforcement
    quota_manager: Arc<QuotaManager>,
    
    /// Self-service provisioning
    provisioning_engine: Arc<ProvisioningEngine>,
    
    /// Enterprise SSO integration
    sso_manager: Arc<SSOManager>,
    
    /// Analytics and insights
    analytics_engine: Arc<AnalyticsEngine>,
}

impl SaaSManager {
    /// Create a new SaaS manager with the given configuration
    pub async fn new(config: SaaSConfig) -> Result<Self> {
        info!("🏢 Initializing AerolithDB SaaS infrastructure");
        
        // Initialize tenant management
        let tenant_manager = Arc::new(TenantManager::new(&config.tenant).await?);
        debug!("✅ Tenant manager initialized");
          // Initialize usage tracking        let usage_tracker = Arc::new(UsageTrackerImpl::new(config.usage.clone())?);
        debug!("✅ Usage tracker initialized");
        
        // Initialize billing engine
        let billing_engine = Arc::new(BillingEngine::new(&config.billing).await?);
        debug!("✅ Billing engine initialized");
        
        // Initialize quota management
        let quota_manager = Arc::new(QuotaManager::new(&config.quotas).await?);
        debug!("✅ Quota manager initialized");
        
        // Initialize provisioning engine
        let provisioning_engine = Arc::new(ProvisioningEngine::new(&config.provisioning).await?);
        debug!("✅ Provisioning engine initialized");
        
        // Initialize SSO manager
        let sso_manager = Arc::new(SSOManager::new(&config.sso).await?);
        debug!("✅ SSO manager initialized");
        
        // Initialize analytics engine
        let analytics_engine = Arc::new(AnalyticsEngine::new(&config.analytics).await?);
        debug!("✅ Analytics engine initialized");
        
        info!("🚀 AerolithDB SaaS infrastructure ready");
        
        Ok(Self {
            config,
            tenant_manager,
            usage_tracker,
            billing_engine,
            quota_manager,
            provisioning_engine,
            sso_manager,
            analytics_engine,
        })
    }
    
    /// Start all SaaS background services
    pub async fn start(&self) -> Result<()> {
        info!("🔄 Starting SaaS background services");
        
        // Start usage collection
        self.usage_tracker.start_collection().await?;
        
        // Start billing calculations
        self.billing_engine.start_billing_cycle().await?;
        
        // Start quota monitoring
        self.quota_manager.start_monitoring().await?;
        
        // Start analytics processing
        self.analytics_engine.start_processing().await?;
        
        info!("✅ All SaaS services started successfully");
        Ok(())
    }
    
    /// Stop all SaaS background services
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping SaaS background services");
        
        self.usage_tracker.stop_collection().await?;
        self.billing_engine.stop_billing_cycle().await?;
        self.quota_manager.stop_monitoring().await?;
        self.analytics_engine.stop_processing().await?;
        
        info!("✅ All SaaS services stopped successfully");
        Ok(())
    }
    
    /// Get reference to tenant manager
    pub fn tenant_manager(&self) -> &Arc<TenantManager> {
        &self.tenant_manager
    }
      /// Get reference to usage tracker
    pub fn usage_tracker(&self) -> &Arc<UsageTrackerImpl> {
        &self.usage_tracker
    }
    
    /// Get reference to billing engine
    pub fn billing_engine(&self) -> &Arc<BillingEngine> {
        &self.billing_engine
    }
    
    /// Get reference to quota manager
    pub fn quota_manager(&self) -> &Arc<QuotaManager> {
        &self.quota_manager
    }
    
    /// Get reference to provisioning engine
    pub fn provisioning_engine(&self) -> &Arc<ProvisioningEngine> {
        &self.provisioning_engine
    }
    
    /// Get reference to SSO manager
    pub fn sso_manager(&self) -> &Arc<SSOManager> {
        &self.sso_manager
    }
    
    /// Get reference to analytics engine
    pub fn analytics_engine(&self) -> &Arc<AnalyticsEngine> {
        &self.analytics_engine
    }
}
