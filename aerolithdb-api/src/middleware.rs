//! SaaS middleware for multi-tenant authentication, authorization, and request routing
//! 
//! Provides middleware functions for:
//! - Tenant identification and validation
//! - Request authentication and authorization  
//! - Usage tracking and quota enforcement
//! - Request routing based on tenant configuration

use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use uuid::Uuid;
use tracing::{warn, debug, error};

// TODO: Re-enable when aerolithdb-saas dependency is restored
// use aerolithdb_saas::{SaaSManager, TenantManager};
use aerolithdb_security::SecurityFramework;

/// SaaS middleware context extracted from requests
#[derive(Debug, Clone)]
pub struct SaaSContext {
    pub tenant_id: Option<Uuid>,
    pub user_id: Option<String>,
    pub organization_domain: Option<String>,
    pub subscription_tier: Option<String>,
    pub authenticated: bool,
}

/// Middleware state for SaaS operations
// TODO: Restore when SaaS dependency is available
/*
#[derive(Clone)]
pub struct SaaSMiddlewareState {
    pub saas_manager: Arc<SaaSManager>,
    pub security: Arc<SecurityFramework>,
}
*/

/// Extract tenant context from request headers and authentication
// TODO: Restore when SaaS dependency is available
/*
pub async fn extract_tenant_context(
    State(state): State<SaaSMiddlewareState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut context = SaaSContext {
        tenant_id: None,
        user_id: None,
        organization_domain: None,
        subscription_tier: None,
        authenticated: false,
    };

    // Extract tenant information from headers
    if let Some(tenant_header) = headers.get("X-Tenant-ID") {
        if let Ok(tenant_str) = tenant_header.to_str() {
            if let Ok(tenant_id) = Uuid::parse_str(tenant_str) {
                context.tenant_id = Some(tenant_id);
                debug!("Extracted tenant ID from header: {}", tenant_id);
            }
        }
    }

    // Extract organization domain from headers or Host header
    if let Some(domain_header) = headers.get("X-Organization-Domain") {
        if let Ok(domain) = domain_header.to_str() {
            context.organization_domain = Some(domain.to_string());
            debug!("Extracted organization domain: {}", domain);
        }
    } else if let Some(host_header) = headers.get("Host") {
        if let Ok(host) = host_header.to_str() {
            // Try to resolve tenant from subdomain pattern (e.g., acme.aerolithdb.com)
            if let Some(subdomain) = extract_subdomain(host) {
                context.organization_domain = Some(subdomain);
                debug!("Extracted subdomain as organization: {}", subdomain);
            }
        }
    }

    // Resolve tenant from organization domain if not already set
    if context.tenant_id.is_none() && context.organization_domain.is_some() {
        if let Some(ref domain) = context.organization_domain {
            match state.saas_manager.tenant_manager().get_tenant_by_domain(domain).await {
                Ok(Some(tenant)) => {
                    context.tenant_id = Some(tenant.tenant_id);
                    context.subscription_tier = Some(tenant.subscription_tier);
                    debug!("Resolved tenant {} from domain {}", tenant.tenant_id, domain);
                },
                Ok(None) => {
                    warn!("No tenant found for domain: {}", domain);
                },
                Err(e) => {
                    error!("Error resolving tenant from domain {}: {}", domain, e);
                },
            }
        }
    }

    // Extract authentication information
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];                // TODO: Implement token validation with security framework
                // For now, we'll do basic validation
                if !token.is_empty() {
                    context.authenticated = true;
                    context.user_id = Some("authenticated_user".to_string());
                    debug!("Authenticated user with token");
                } else {
                    warn!("Empty token provided");
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        }
    }

    // Add context to request extensions
    request.extensions_mut().insert(context);    Ok(next.run(request).await)
}
*/

/// Validate tenant access and enforce quotas
// TODO: Restore when SaaS dependency is available
/*
pub async fn enforce_tenant_quotas(
    State(state): State<SaaSMiddlewareState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract SaaS context from request
    let context = request.extensions().get::<SaaSContext>().cloned()
        .unwrap_or_else(|| SaaSContext {
            tenant_id: None,
            user_id: None,
            organization_domain: None,
            subscription_tier: None,
            authenticated: false,
        });

    // Skip quota enforcement for certain paths
    let path = request.uri().path();
    if is_quota_exempt_path(path) {
        return Ok(next.run(request).await);
    }

    // Enforce tenant access if tenant is identified
    if let Some(tenant_id) = context.tenant_id {
        // Validate tenant status
        match state.saas_manager.tenant_manager().validate_tenant_access(tenant_id, path).await {
            Ok(true) => {
                debug!("Tenant {} access validated for path: {}", tenant_id, path);
            },
            Ok(false) => {
                warn!("Tenant {} access denied for path: {}", tenant_id, path);
                return Err(StatusCode::FORBIDDEN);
            },
            Err(e) => {
                error!("Error validating tenant access: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            },
        }

        // Check operation quotas
        let operation_type = determine_operation_type(&request);
        let resource_delta = estimate_resource_delta(&request);

        match state.saas_manager.quota_manager()
            .check_operation_allowed(tenant_id, &operation_type, resource_delta).await {
            Ok(true) => {
                debug!("Quota check passed for tenant {} operation: {}", tenant_id, operation_type);
            },
            Ok(false) => {
                warn!("Quota exceeded for tenant {} operation: {}", tenant_id, operation_type);
                return Err(StatusCode::TOO_MANY_REQUESTS);
            },
            Err(e) => {
                error!("Error checking quotas: {}", e);
                // Continue with warning rather than blocking
                warn!("Quota check failed, allowing request: {}", e);
            },
        }
    } else if requires_tenant_context(path) {
        warn!("Request to {} requires tenant context but none provided", path);
        return Err(StatusCode::BAD_REQUEST);
    }    Ok(next.run(request).await)
}
*/

/// Track usage metrics for billing and analytics
// TODO: Restore when SaaS dependency is available
/*
pub async fn track_usage_metrics(
    State(state): State<SaaSMiddlewareState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = std::time::Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    
    // Extract tenant context
    let context = request.extensions().get::<SaaSContext>().cloned();

    // Process request
    let response = next.run(request).await;
    
    // Calculate metrics
    let duration = start_time.elapsed();
    let status_code = response.status();
    
    // Record usage metrics asynchronously
    if let Some(context) = context {
        if let Some(tenant_id) = context.tenant_id {
            let state_clone = state.clone();
            let method_str = method.to_string();
            let path_clone = path.clone();
            
            tokio::spawn(async move {
                if let Err(e) = record_usage_metrics(
                    &state_clone.saas_manager,
                    tenant_id,
                    &method_str,
                    &path_clone,
                    status_code,
                    duration,
                ).await {
                    error!("Failed to record usage metrics: {}", e);
                }
            });
        }
    }

    Ok(response)
}

/// Record usage metrics for tenant
async fn record_usage_metrics(
    saas_manager: &SaaSManager,
    tenant_id: Uuid,
    method: &str,
    path: &str,
    status: StatusCode,
    duration: std::time::Duration,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Recording usage metrics for tenant {}: {} {} - {} ({:?})", 
           tenant_id, method, path, status, duration);

    // Track API call
    saas_manager.usage_tracker().record_api_call(
        tenant_id,
        method,
        path,
        status.as_u16(),
        duration.as_millis() as u64,
    ).await?;

    // Update tenant usage statistics
    if let Ok(Some(mut tenant)) = saas_manager.tenant_manager().get_tenant(tenant_id).await {
        tenant.current_usage.api_calls_current_hour += 1;
        tenant.current_usage.last_activity = Some(chrono::Utc::now());
        
        if let Err(e) = saas_manager.tenant_manager()
            .update_tenant_usage(tenant_id, tenant.current_usage).await {
            error!("Failed to update tenant usage: {}", e);
        }
    }

    Ok(())
}

/// Extract subdomain from host header
fn extract_subdomain(host: &str) -> Option<String> {
    // Remove port if present
    let host = host.split(':').next().unwrap_or(host);
    
    // Check for subdomain pattern (e.g., acme.aerolithdb.com)
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() >= 3 && (parts[1] == "aerolithdb" || parts[1] == "localhost") {
        Some(parts[0].to_string())
    } else {
        None
    }
}

/// Check if path is exempt from quota enforcement
fn is_quota_exempt_path(path: &str) -> bool {
    path.starts_with("/health") || 
    path.starts_with("/api/v1/saas/admin") ||
    path.starts_with("/api/v1/payment") ||
    path.contains("/auth/")
}

/// Check if path requires tenant context
fn requires_tenant_context(path: &str) -> bool {
    path.starts_with("/api/v1/collections") ||
    path.starts_with("/api/v1/saas/tenants") ||
    path.starts_with("/api/v1/saas/billing")
}

/// Determine operation type from request
fn determine_operation_type(request: &Request) -> String {
    let method = request.method();
    let path = request.uri().path();
    
    if path.contains("/documents") {
        match method {
            &axum::http::Method::GET => "get_document".to_string(),
            &axum::http::Method::POST => "create_document".to_string(),
            &axum::http::Method::PUT => "update_document".to_string(),
            &axum::http::Method::DELETE => "delete_document".to_string(),
            _ => "api_call".to_string(),
        }
    } else if path.contains("/query") {
        "query_documents".to_string()
    } else {
        "api_call".to_string()
    }
}

/// Estimate resource delta for operation
fn estimate_resource_delta(request: &Request) -> u64 {
    let method = request.method();
    let path = request.uri().path();
    
    // Estimate based on operation type
    if path.contains("/documents") && method == &axum::http::Method::POST {
        // Estimate document size - would typically check content-length
        1024 // 1KB default estimate
    } else if path.contains("/query") {
        // Query operations don't directly consume storage
        0
    } else {
        // Default API call estimate        1
    }
}
*/

/// Helper struct for user information from security framework
pub struct UserInfo {
    pub user_id: String,
    pub roles: Vec<String>,
    pub tenant_id: Option<Uuid>,
}

// TODO: Implement proper token validation in SecurityFramework
