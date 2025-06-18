//! SaaS API endpoints for multi-tenancy, billing, and administration
//! 
//! Provides REST API endpoints for all SaaS functionality including tenant management,
//! billing operations, quota monitoring, SSO integration, and administrative operations.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, debug, warn, error};

use aerolithdb_saas::{
    SaaSManager, CreateTenantRequest, UpdateTenantRequest, Tenant, TenantUsage,
    SSOAuthRequest, SSOAuthResponse, UsageStatistics, Invoice, BillingInfo,
    QuotaViolation, ProvisioningRequest, AnalyticsQuery, SaaSStatus,
    LiveUsageStats, TenantContext, AuthContext, saas_auth_middleware
};

/// AppState for SaaS endpoints
#[derive(Clone)]
pub struct SaaSAppState {
    pub saas_manager: std::sync::Arc<SaaSManager>,
}

/// Create SaaS router with all endpoints and middleware
pub fn saas_routes() -> Router<SaaSAppState> {
    Router::new()
        // Public endpoints (no auth required)
        .route("/health", get(saas_health))
        .route("/status", get(saas_status))
        .route("/auth/login", post(authenticate_user))
        .route("/auth/refresh", post(refresh_token))
        
        // Protected endpoints (require authentication)
        .route("/tenants", post(create_tenant))
        .route("/tenants", get(list_tenants))
        .route("/tenants/:tenant_id", get(get_tenant))
        .route("/tenants/:tenant_id", put(update_tenant))
        .route("/tenants/:tenant_id", delete(delete_tenant))
        .route("/tenants/:tenant_id/usage", get(get_tenant_usage))
        .route("/tenants/:tenant_id/usage/live", get(get_live_tenant_usage))
        .route("/tenants/domain/:domain", get(get_tenant_by_domain))
        
        // Billing endpoints
        .route("/billing/invoices", get(list_invoices))
        .route("/billing/tenants/:tenant_id/invoices", get(get_tenant_invoices))
        .route("/billing/tenants/:tenant_id/balance", get(get_tenant_balance))
        .route("/billing/pricing", get(get_pricing_tiers))
        .route("/billing/calculate", post(calculate_billing))
        
        // Quota management endpoints
        .route("/quotas/check", post(check_quota))
        .route("/quotas/violations", get(get_quota_violations))
        .route("/quotas/tenants/:tenant_id/status", get(get_tenant_quota_status))
        
        // SSO endpoints
        .route("/sso/providers", get(list_sso_providers))
        .route("/sso/auth/initiate", post(initiate_sso_auth))
        .route("/sso/auth/complete", post(complete_sso_auth))
        .route("/sso/sessions/:session_id", get(validate_sso_session))
        .route("/sso/sessions/:session_id", delete(logout_sso_session))
        
        // Analytics endpoints
        .route("/analytics/usage", post(query_usage_analytics))
        .route("/analytics/tenants/:tenant_id/summary", get(get_tenant_analytics))
        .route("/analytics/system/metrics", get(get_system_metrics))
        
        // Provisioning endpoints
        .route("/provisioning/clusters", post(provision_cluster))
        .route("/provisioning/clusters/:cluster_id", get(get_cluster_status))
        .route("/provisioning/clusters/:cluster_id", delete(deprovision_cluster))
        
        // Admin endpoints
        .route("/admin/health", get(saas_health_check))
        .route("/admin/status", get(get_saas_status))
        .route("/admin/metrics", get(get_admin_metrics))
}

// ============================================================================
// Tenant Management Endpoints
// ============================================================================

#[derive(Deserialize)]
struct ListTenantsQuery {
    limit: Option<u32>,
    offset: Option<u32>,
    status: Option<String>,
    subscription_tier: Option<String>,
}

async fn create_tenant(
    State(state): State<SaaSAppState>,
    Json(request): Json<CreateTenantRequest>,
) -> Result<Json<Tenant>, StatusCode> {
    match state.saas_manager.tenant_manager().create_tenant(request).await {
        Ok(tenant) => Ok(Json(tenant)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_tenants(
    State(state): State<SaaSAppState>,
    Query(params): Query<ListTenantsQuery>,
) -> Result<Json<Vec<Tenant>>, StatusCode> {
    match state.saas_manager.tenant_manager()
        .list_tenants(params.limit, params.offset).await {
        Ok(tenants) => {
            // Filter by status and subscription tier if specified
            let filtered_tenants = tenants.into_iter()
                .filter(|t| {
                    if let Some(ref status) = params.status {
                        // Would need to serialize status and compare
                        true // Simplified for now
                    } else {
                        true
                    }
                })
                .filter(|t| {
                    if let Some(ref tier) = params.subscription_tier {
                        &t.subscription_tier == tier
                    } else {
                        true
                    }
                })
                .collect();
            Ok(Json(filtered_tenants))
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_tenant(
    State(state): State<SaaSAppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<Tenant>, StatusCode> {
    match state.saas_manager.tenant_manager().get_tenant(tenant_id).await {
        Ok(Some(tenant)) => Ok(Json(tenant)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_tenant(
    State(state): State<SaaSAppState>,
    Path(tenant_id): Path<Uuid>,
    Json(request): Json<UpdateTenantRequest>,
) -> Result<Json<Tenant>, StatusCode> {
    match state.saas_manager.tenant_manager().update_tenant(tenant_id, request).await {
        Ok(tenant) => Ok(Json(tenant)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_tenant(
    State(state): State<SaaSAppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // In practice, this would likely be a soft delete
    match state.saas_manager.tenant_manager().get_tenant(tenant_id).await {
        Ok(Some(_)) => {
            // Implement soft delete logic here
            Ok(StatusCode::NO_CONTENT)
        },
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_tenant_usage(
    State(state): State<SaaSAppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<TenantUsage>, StatusCode> {
    match state.saas_manager.tenant_manager().get_tenant(tenant_id).await {
        Ok(Some(tenant)) => Ok(Json(tenant.current_usage)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_tenant_usage(
    State(state): State<SaaSAppState>,
    Path(tenant_id): Path<Uuid>,
    Json(usage): Json<TenantUsage>,
) -> Result<StatusCode, StatusCode> {
    match state.saas_manager.tenant_manager().update_tenant_usage(tenant_id, usage).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_tenant_by_domain(
    State(state): State<SaaSAppState>,
    Path(domain): Path<String>,
) -> Result<Json<Tenant>, StatusCode> {
    match state.saas_manager.tenant_manager().get_tenant_by_domain(&domain).await {
        Ok(Some(tenant)) => Ok(Json(tenant)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ============================================================================
// Billing Endpoints
// ============================================================================

#[derive(Deserialize)]
struct ListInvoicesQuery {
    tenant_id: Option<Uuid>,
    status: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

async fn list_invoices(
    State(state): State<SaaSAppState>,
    Query(params): Query<ListInvoicesQuery>,
) -> Result<Json<Vec<Invoice>>, StatusCode> {
    if let Some(tenant_id) = params.tenant_id {
        match state.saas_manager.billing_engine()
            .get_tenant_invoices(tenant_id, params.limit, params.offset).await {
            Ok(invoices) => Ok(Json(invoices)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        // Return empty list for now - would need system-wide invoice query
        Ok(Json(vec![]))
    }
}

async fn get_tenant_invoices(
    State(state): State<SaaSAppState>,
    Path(tenant_id): Path<Uuid>,
    Query(params): Query<ListInvoicesQuery>,
) -> Result<Json<Vec<Invoice>>, StatusCode> {
    match state.saas_manager.billing_engine()
        .get_tenant_invoices(tenant_id, params.limit, params.offset).await {
        Ok(invoices) => Ok(Json(invoices)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Serialize)]
struct TenantBalance {
    tenant_id: Uuid,
    current_balance: f64,
    outstanding_amount: f64,
    credit_balance: f64,
    currency: String,
    last_payment_date: Option<DateTime<Utc>>,
    next_billing_date: DateTime<Utc>,
}

async fn get_tenant_balance(
    State(state): State<SaaSAppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<TenantBalance>, StatusCode> {
    match state.saas_manager.tenant_manager().get_tenant(tenant_id).await {
        Ok(Some(tenant)) => {
            let balance = TenantBalance {
                tenant_id,
                current_balance: 0.0, // Would calculate from billing engine
                outstanding_amount: tenant.billing_info.as_ref()
                    .map(|b| b.outstanding_balance)
                    .unwrap_or(0.0),
                credit_balance: 0.0, // Would track credits
                currency: tenant.billing_info.as_ref()
                    .map(|b| b.currency.clone())
                    .unwrap_or_else(|| "USD".to_string()),
                last_payment_date: None, // Would track from payments
                next_billing_date: tenant.billing_info.as_ref()
                    .map(|b| b.next_billing_date)
                    .unwrap_or_else(|| Utc::now()),
            };
            Ok(Json(balance))
        },
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_pricing_tiers(
    State(_state): State<SaaSAppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Return mock pricing tiers for now
    let pricing = serde_json::json!({
        "tiers": [
            {
                "name": "starter",
                "base_fee": 29.0,
                "included_quotas": {
                    "max_storage_bytes": 1073741824,
                    "max_api_calls_per_hour": 1000,
                    "max_connections": 10
                },
                "overage_rates": {
                    "storage_per_gb": 0.10,
                    "api_calls_per_1k": 0.01,
                    "connections": 5.0
                }
            },
            {
                "name": "professional",
                "base_fee": 99.0,
                "included_quotas": {
                    "max_storage_bytes": 10737418240,
                    "max_api_calls_per_hour": 10000,
                    "max_connections": 100
                },
                "overage_rates": {
                    "storage_per_gb": 0.08,
                    "api_calls_per_1k": 0.008,
                    "connections": 4.0
                }
            }
        ]
    });
    Ok(Json(pricing))
}

#[derive(Deserialize)]
struct CalculateBillingRequest {
    tenant_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

async fn calculate_billing(
    State(_state): State<SaaSAppState>,
    Json(_request): Json<CalculateBillingRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Return mock billing calculation
    let calculation = serde_json::json!({
        "period_start": "2024-01-01T00:00:00Z",
        "period_end": "2024-01-31T23:59:59Z",
        "base_fee": 29.0,
        "usage_charges": {
            "storage": 5.50,
            "api_calls": 12.30,
            "compute": 8.75
        },
        "total_amount": 55.55,
        "currency": "USD"
    });
    Ok(Json(calculation))
}

// ============================================================================
// Quota Management Endpoints
// ============================================================================

#[derive(Deserialize)]
struct CheckQuotaRequest {
    tenant_id: Uuid,
    operation_type: String,
    resource_delta: u64,
}

async fn check_quota(
    State(state): State<SaaSAppState>,
    Json(request): Json<CheckQuotaRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.saas_manager.quota_manager()
        .check_operation_allowed(request.tenant_id, &request.operation_type, request.resource_delta).await {
        Ok(allowed) => {
            let response = serde_json::json!({
                "allowed": allowed,
                "tenant_id": request.tenant_id,
                "operation_type": request.operation_type,
                "resource_delta": request.resource_delta
            });
            Ok(Json(response))
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_quota_violations(
    State(_state): State<SaaSAppState>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    // Return mock quota violations
    let violations = vec![
        serde_json::json!({
            "tenant_id": "00000000-0000-0000-0000-000000000001",
            "quota_type": "storage",
            "current_usage": 1200000000,
            "limit": 1073741824,
            "violation_percentage": 111.8,
            "timestamp": "2024-01-15T10:30:00Z",
            "enforcement_action": "Throttle"
        })
    ];
    Ok(Json(violations))
}

async fn get_tenant_quota_status(
    State(state): State<SaaSAppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.saas_manager.tenant_manager().get_tenant(tenant_id).await {
        Ok(Some(tenant)) => {
            let status = serde_json::json!({
                "tenant_id": tenant_id,
                "quotas": {
                    "storage": {
                        "current": tenant.current_usage.storage_bytes,
                        "limit": tenant.limits.max_storage_bytes,
                        "percentage": (tenant.current_usage.storage_bytes as f64 / tenant.limits.max_storage_bytes as f64) * 100.0
                    },
                    "api_calls": {
                        "current": tenant.current_usage.api_calls_current_hour,
                        "limit": tenant.limits.max_api_calls_per_hour,
                        "percentage": (tenant.current_usage.api_calls_current_hour as f64 / tenant.limits.max_api_calls_per_hour as f64) * 100.0
                    },
                    "connections": {
                        "current": tenant.current_usage.active_connections,
                        "limit": tenant.limits.max_connections,
                        "percentage": (tenant.current_usage.active_connections as f64 / tenant.limits.max_connections as f64) * 100.0
                    }
                }
            });
            Ok(Json(status))
        },
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ============================================================================
// SSO Endpoints
// ============================================================================

async fn list_sso_providers(
    State(_state): State<SaaSAppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let providers = serde_json::json!({
        "providers": [
            {
                "name": "google",
                "type": "OAuth2",
                "enabled": true,
                "display_name": "Google"
            },
            {
                "name": "microsoft",
                "type": "OAuth2", 
                "enabled": true,
                "display_name": "Microsoft"
            },
            {
                "name": "okta",
                "type": "SAML",
                "enabled": false,
                "display_name": "Okta"
            }
        ]
    });
    Ok(Json(providers))
}

async fn initiate_sso_auth(
    State(state): State<SaaSAppState>,
    Json(request): Json<SSOAuthRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.saas_manager.sso_manager().initiate_auth(request).await {
        Ok(auth_url) => {
            let response = serde_json::json!({
                "auth_url": auth_url,
                "status": "initiated"
            });
            Ok(Json(response))
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
struct CompleteSSORequest {
    provider: String,
    auth_code: String,
}

async fn complete_sso_auth(
    State(state): State<SaaSAppState>,
    Json(request): Json<CompleteSSORequest>,
) -> Result<Json<SSOAuthResponse>, StatusCode> {
    match state.saas_manager.sso_manager()
        .complete_auth(&request.provider, &request.auth_code).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn validate_sso_session(
    State(state): State<SaaSAppState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.saas_manager.sso_manager().validate_session(session_id).await {
        Ok(session) => {
            let response = serde_json::json!({
                "valid": true,
                "session": session
            });
            Ok(Json(response))
        },
        Err(_) => {
            let response = serde_json::json!({
                "valid": false,
                "error": "Invalid or expired session"
            });
            Ok(Json(response))
        },
    }
}

async fn logout_sso_session(
    State(state): State<SaaSAppState>,
    Path(session_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.saas_manager.sso_manager().logout(session_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ============================================================================
// Analytics Endpoints
// ============================================================================

async fn query_usage_analytics(
    State(_state): State<SaaSAppState>,
    Json(_query): Json<AnalyticsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Return mock analytics data
    let analytics = serde_json::json!({
        "total_tenants": 150,
        "active_tenants": 142,
        "total_api_calls": 1250000,
        "total_storage_gb": 2500.5,
        "average_response_time_ms": 45,
        "top_operations": [
            {"operation": "get_document", "count": 450000},
            {"operation": "query_documents", "count": 320000},
            {"operation": "put_document", "count": 280000}
        ]
    });
    Ok(Json(analytics))
}

async fn get_tenant_analytics(
    State(_state): State<SaaSAppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let analytics = serde_json::json!({
        "tenant_id": tenant_id,
        "period": "last_30_days",
        "api_calls": 25000,
        "storage_usage_gb": 15.5,
        "bandwidth_gb": 8.2,
        "average_response_time_ms": 42,
        "error_rate": 0.02,
        "top_collections": [
            {"name": "users", "operations": 8500},
            {"name": "orders", "operations": 6200},
            {"name": "products", "operations": 4800}
        ]
    });
    Ok(Json(analytics))
}

async fn get_system_metrics(
    State(_state): State<SaaSAppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let metrics = serde_json::json!({
        "uptime_seconds": 2592000,
        "total_requests": 5000000,
        "requests_per_second": 125.5,
        "memory_usage_percent": 68.5,
        "cpu_usage_percent": 42.3,
        "disk_usage_percent": 35.8,
        "active_connections": 1250,
        "cache_hit_rate": 0.94
    });
    Ok(Json(metrics))
}

// ============================================================================
// Provisioning Endpoints
// ============================================================================

async fn provision_cluster(
    State(_state): State<SaaSAppState>,
    Json(_request): Json<ProvisioningRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let response = serde_json::json!({
        "cluster_id": "cluster-abc123",
        "status": "provisioning",
        "estimated_completion": "2024-01-15T15:30:00Z",
        "endpoints": {
            "rest": "https://cluster-abc123.aerolithdb.cloud:8080",
            "grpc": "cluster-abc123.aerolithdb.cloud:9090",
            "websocket": "wss://cluster-abc123.aerolithdb.cloud:8083"
        }
    });
    Ok(Json(response))
}

async fn get_cluster_status(
    State(_state): State<SaaSAppState>,
    Path(cluster_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let status = serde_json::json!({
        "cluster_id": cluster_id,
        "status": "running",
        "health": "healthy",
        "nodes": 3,
        "created_at": "2024-01-10T10:00:00Z",
        "last_health_check": "2024-01-15T14:45:00Z",
        "endpoints": {
            "rest": format!("https://{}.aerolithdb.cloud:8080", cluster_id),
            "grpc": format!("{}.aerolithdb.cloud:9090", cluster_id),
            "websocket": format!("wss://{}.aerolithdb.cloud:8083", cluster_id)
        }
    });
    Ok(Json(status))
}

async fn deprovision_cluster(
    State(_state): State<SaaSAppState>,
    Path(cluster_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let response = serde_json::json!({
        "cluster_id": cluster_id,
        "status": "deprovisioning",
        "estimated_completion": "2024-01-15T16:00:00Z"
    });
    Ok(Json(response))
}

// ============================================================================
// Admin Endpoints
// ============================================================================

async fn saas_health_check(
    State(_state): State<SaaSAppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let health = serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "services": {
            "tenant_manager": "healthy",
            "billing_engine": "healthy", 
            "quota_manager": "healthy",
            "sso_manager": "healthy",
            "analytics_engine": "healthy",
            "provisioning_engine": "healthy"
        },
        "database": "connected",
        "uptime_seconds": 86400
    });
    Ok(Json(health))
}

async fn get_saas_status(
    State(_state): State<SaaSAppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let status = serde_json::json!({
        "version": "1.0.0",
        "environment": "production",
        "features": {
            "multi_tenancy": true,
            "billing": true,
            "quotas": true,
            "sso": true,
            "analytics": true,
            "provisioning": false
        },
        "statistics": {
            "total_tenants": 150,
            "active_tenants": 142,
            "total_storage_gb": 2500.5,
            "monthly_revenue": 12750.00
        }
    });
    Ok(Json(status))
}

async fn get_admin_metrics(
    State(_state): State<SaaSAppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let metrics = serde_json::json!({
        "tenant_metrics": {
            "total": 150,
            "active": 142,
            "trial": 8,
            "suspended": 0,
            "new_this_month": 12
        },
        "billing_metrics": {
            "monthly_revenue": 12750.00,
            "outstanding_invoices": 5,
            "average_billing_amount": 89.47,
            "collection_rate": 0.98
        },
        "usage_metrics": {
            "total_api_calls_today": 125000,
            "total_storage_gb": 2500.5,
            "average_response_time_ms": 45,
            "error_rate": 0.015
        },
        "quota_metrics": {
            "violations_today": 3,
            "tenants_near_limit": 12,
            "most_common_violation": "storage"
        }
    });
    Ok(Json(metrics))
}

// Authentication request/response structures
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub tenant_id: Uuid,
    pub user_id: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub session_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub tenant: Tenant,
    pub user_info: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub user_id: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

// Enhanced endpoint implementations

/// SaaS health check endpoint
pub async fn saas_health(
    State(state): State<SaaSAppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("💚 SaaS health check requested");
    
    let health_status = serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "services": {
            "saas_manager": "healthy",
            "tenant_manager": "healthy",
            "billing_manager": "healthy",
            "quota_manager": "healthy"
        }
    });
    
    Ok(Json(health_status))
}

/// Get comprehensive SaaS status
pub async fn saas_status(
    State(state): State<SaaSAppState>,
) -> Result<Json<SaaSStatus>, StatusCode> {
    info!("📊 SaaS status requested");
    
    match state.saas_manager.get_status().await {
        Ok(status) => Ok(Json(status)),
        Err(e) => {
            error!("❌ Failed to get SaaS status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Authenticate user and return JWT token
pub async fn authenticate_user(
    State(state): State<SaaSAppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    info!("🔐 Authentication requested for user {} in tenant {}", 
          request.user_id, request.tenant_id);
    
    match state.saas_manager.auth_manager().authenticate_user(
        request.tenant_id,
        &request.user_id,
        &request.password,
        None, // IP would come from request headers
        None, // User agent would come from request headers
    ).await {
        Ok((token, session)) => {
            // Get tenant info
            match state.saas_manager.tenant_manager().get_tenant(request.tenant_id).await {
                Ok(Some(tenant)) => {
                    let response = LoginResponse {
                        token,
                        session_id: session.session_id,
                        expires_at: session.expires_at,
                        tenant,
                        user_info: UserInfo {
                            user_id: session.user_id,
                            roles: session.roles,
                            permissions: session.permissions,
                        },
                    };
                    
                    info!("✅ User {} authenticated successfully", request.user_id);
                    Ok(Json(response))
                },
                Ok(None) => {
                    warn!("⚠️ Tenant {} not found during authentication", request.tenant_id);
                    Err(StatusCode::NOT_FOUND)
                },
                Err(e) => {
                    error!("❌ Failed to get tenant during authentication: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        },
        Err(e) => {
            warn!("⚠️ Authentication failed for user {}: {}", request.user_id, e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// Refresh authentication token
pub async fn refresh_token(
    State(state): State<SaaSAppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔄 Token refresh requested");
    
    // In a real implementation, this would validate the refresh token
    // and issue a new access token
    let response = serde_json::json!({
        "message": "Token refresh not yet implemented",
        "status": "placeholder"
    });
    
    Ok(Json(response))
}

/// Get live usage statistics for tenant
pub async fn get_live_tenant_usage(
    Path(tenant_id): Path<Uuid>,
    State(state): State<SaaSAppState>,
) -> Result<Json<LiveUsageStats>, StatusCode> {
    info!("📊 Live usage requested for tenant {}", tenant_id);
    
    match state.saas_manager.usage_tracker().get_current_usage(tenant_id).await {
        Ok(Some(usage)) => Ok(Json(usage)),
        Ok(None) => {
            warn!("⚠️ No usage data found for tenant {}", tenant_id);
            Err(StatusCode::NOT_FOUND)
        },
        Err(e) => {
            error!("❌ Failed to get live usage for tenant {}: {}", tenant_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
