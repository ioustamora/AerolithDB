//! Resource quota management and enforcement
//! 
//! Enforces resource limits for tenants and provides quota violation handling.

use crate::config::{QuotaConfig, QuotaEnforcementAction};
use crate::errors::{QuotaError, QuotaResult};
use crate::tenant::{Tenant, TenantManager, TenantUsage};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::interval;
use tracing::{info, debug, warn, error};
use uuid::Uuid;

/// Quota violation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaViolation {
    pub tenant_id: Uuid,
    pub quota_type: String,
    pub current_usage: u64,
    pub limit: u64,
    pub violation_percentage: f64,
    pub timestamp: DateTime<Utc>,
    pub enforcement_action: QuotaEnforcementAction,
}

/// Quota manager for enforcing resource limits
pub struct QuotaManager {
    config: QuotaConfig,
    tenant_manager: Arc<TenantManager>,
    is_running: Arc<tokio::sync::RwLock<bool>>,
}

impl QuotaManager {
    /// Create a new quota manager
    pub async fn new(config: &QuotaConfig) -> Result<Self> {
        info!("📊 Initializing quota manager");
        
        // Note: In a real implementation, this would be injected
        let tenant_manager = Arc::new(TenantManager::new(&crate::config::TenantConfig::default()).await?);
        
        let manager = Self {
            config: config.clone(),
            tenant_manager,
            is_running: Arc::new(tokio::sync::RwLock::new(false)),
        };
        
        info!("✅ Quota manager initialized");
        Ok(manager)
    }
    
    /// Start quota monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        info!("🔄 Starting quota monitoring");
        
        {
            let mut is_running = self.is_running.write().await;
            if *is_running {
                warn!("Quota monitoring already running");
                return Ok(());
            }
            *is_running = true;
        }
        
        if self.config.enabled {
            let config = self.config.clone();
            let tenant_manager = Arc::clone(&self.tenant_manager);
            let is_running = Arc::clone(&self.is_running);
            
            tokio::spawn(async move {
                let mut interval = interval(config.check_interval);
                
                loop {
                    interval.tick().await;
                    
                    let running = { *is_running.read().await };
                    if !running {
                        break;
                    }
                    
                    // Check quotas for all tenants
                    if let Err(e) = Self::check_all_quotas(&config, &tenant_manager).await {
                        error!("Quota checking failed: {}", e);
                    }
                }
            });
        }
        
        info!("✅ Quota monitoring started");
        Ok(())
    }
    
    /// Stop quota monitoring
    pub async fn stop_monitoring(&self) -> Result<()> {
        info!("🛑 Stopping quota monitoring");
        
        {
            let mut is_running = self.is_running.write().await;
            *is_running = false;
        }
        
        info!("✅ Quota monitoring stopped");
        Ok(())
    }
    
    /// Check quotas for all tenants
    async fn check_all_quotas(
        config: &QuotaConfig,
        tenant_manager: &TenantManager,
    ) -> Result<()> {
        debug!("📊 Checking quotas for all tenants");
        
        let tenants = tenant_manager.list_tenants(Some(1000), Some(0)).await?;
        
        for tenant in tenants {
            if let Err(e) = Self::check_tenant_quotas(config, &tenant).await {
                error!("Failed to check quotas for tenant {}: {}", tenant.tenant_id, e);
            }
        }
        
        debug!("✅ Quota checking completed");
        Ok(())
    }
    
    /// Check quotas for a specific tenant
    async fn check_tenant_quotas(config: &QuotaConfig, tenant: &Tenant) -> QuotaResult<()> {
        let usage = &tenant.current_usage;
        let limits = &tenant.limits;
        
        // Check storage quota
        if usage.storage_bytes > limits.max_storage_bytes {
            let violation = QuotaViolation {
                tenant_id: tenant.tenant_id,
                quota_type: "storage".to_string(),
                current_usage: usage.storage_bytes,
                limit: limits.max_storage_bytes,
                violation_percentage: (usage.storage_bytes as f64 / limits.max_storage_bytes as f64) * 100.0,
                timestamp: Utc::now(),
                enforcement_action: config.enforcement_action.clone(),
            };
            
            Self::handle_quota_violation(config, &violation).await?;
        }
        
        // Check API calls quota
        if usage.api_calls_current_hour > limits.max_api_calls_per_hour {
            let violation = QuotaViolation {
                tenant_id: tenant.tenant_id,
                quota_type: "api_calls".to_string(),
                current_usage: usage.api_calls_current_hour,
                limit: limits.max_api_calls_per_hour,
                violation_percentage: (usage.api_calls_current_hour as f64 / limits.max_api_calls_per_hour as f64) * 100.0,
                timestamp: Utc::now(),
                enforcement_action: config.enforcement_action.clone(),
            };
            
            Self::handle_quota_violation(config, &violation).await?;
        }
        
        // Check connections quota
        if usage.active_connections > limits.max_connections {
            let violation = QuotaViolation {
                tenant_id: tenant.tenant_id,
                quota_type: "connections".to_string(),
                current_usage: usage.active_connections as u64,
                limit: limits.max_connections as u64,
                violation_percentage: (usage.active_connections as f64 / limits.max_connections as f64) * 100.0,
                timestamp: Utc::now(),
                enforcement_action: config.enforcement_action.clone(),
            };
            
            Self::handle_quota_violation(config, &violation).await?;
        }
        
        Ok(())
    }
    
    /// Handle quota violation
    async fn handle_quota_violation(
        config: &QuotaConfig,
        violation: &QuotaViolation,
    ) -> QuotaResult<()> {
        warn!("🚨 Quota violation detected: {} for tenant {} ({:.1}% of limit)",
              violation.quota_type, violation.tenant_id, violation.violation_percentage);
        
        match &violation.enforcement_action {
            QuotaEnforcementAction::Block => {
                // Block new operations for this tenant
                error!("🚫 Blocking operations for tenant {} due to {} quota violation",
                       violation.tenant_id, violation.quota_type);
                // TODO: Implement operation blocking mechanism
            },
            QuotaEnforcementAction::Throttle => {
                // Throttle operations for this tenant
                warn!("🐌 Throttling operations for tenant {} due to {} quota violation",
                      violation.tenant_id, violation.quota_type);
                // TODO: Implement throttling mechanism
            },
            QuotaEnforcementAction::AllowWithOverage => {
                // Allow with overage billing
                info!("💰 Allowing overage for tenant {} on {} quota (will be billed)",
                      violation.tenant_id, violation.quota_type);
                // TODO: Implement overage billing
            },
            QuotaEnforcementAction::WarnOnly => {
                // Just log warning
                warn!("⚠️ Quota warning for tenant {} on {} quota",
                      violation.tenant_id, violation.quota_type);
            },
        }
        
        Ok(())
    }
    
    /// Check if an operation is allowed for a tenant
    pub async fn check_operation_allowed(
        &self,
        tenant_id: Uuid,
        operation_type: &str,
        resource_delta: u64,
    ) -> QuotaResult<bool> {
        if !self.config.enabled {
            return Ok(true);
        }
        
        let tenant = self.tenant_manager
            .get_tenant(tenant_id)
            .await
            .map_err(|e| QuotaError::CheckFailed {
                message: format!("Failed to get tenant: {}", e),
            })?
            .ok_or_else(|| QuotaError::CheckFailed {
                message: format!("Tenant not found: {}", tenant_id),
            })?;
        
        match operation_type {
            "storage" => {
                let new_usage = tenant.current_usage.storage_bytes + resource_delta;
                if new_usage > tenant.limits.max_storage_bytes {
                    return match self.config.enforcement_action {
                        QuotaEnforcementAction::Block => Ok(false),
                        QuotaEnforcementAction::Throttle => Ok(false), // Simplified - would implement rate limiting
                        QuotaEnforcementAction::AllowWithOverage => Ok(true),
                        QuotaEnforcementAction::WarnOnly => Ok(true),
                    };
                }
            },
            "api_call" => {
                let new_usage = tenant.current_usage.api_calls_current_hour + resource_delta;
                if new_usage > tenant.limits.max_api_calls_per_hour {
                    return match self.config.enforcement_action {
                        QuotaEnforcementAction::Block => Ok(false),
                        QuotaEnforcementAction::Throttle => Ok(false),
                        QuotaEnforcementAction::AllowWithOverage => Ok(true),
                        QuotaEnforcementAction::WarnOnly => Ok(true),
                    };
                }
            },
            "connection" => {
                let new_usage = tenant.current_usage.active_connections + resource_delta as u32;
                if new_usage > tenant.limits.max_connections {
                    return match self.config.enforcement_action {
                        QuotaEnforcementAction::Block => Ok(false),
                        QuotaEnforcementAction::Throttle => Ok(false),
                        QuotaEnforcementAction::AllowWithOverage => Ok(true),
                        QuotaEnforcementAction::WarnOnly => Ok(true),
                    };
                }
            },
            _ => {
                return Err(QuotaError::CheckFailed {
                    message: format!("Unknown operation type: {}", operation_type),
                });
            }
        }
        
        Ok(true)
    }
}
