//! SaaS authentication and authorization middleware
//! 
//! Provides tenant-aware authentication, authorization, and session management
//! for multi-tenant SaaS operations.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use chrono::{DateTime, Utc, Duration};
use anyhow::Result;
use tracing::{info, debug, warn, error};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    middleware::Next,
};

use crate::tenant::*;
use crate::errors::{SaaSError, TenantError};

/// JWT token claims for SaaS authentication
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaaSClaims {
    /// Subject (user ID)
    pub sub: String,
    
    /// Tenant ID
    pub tenant_id: Uuid,
    
    /// User roles within the tenant
    pub roles: Vec<String>,
    
    /// User permissions
    pub permissions: Vec<String>,
    
    /// Subscription tier
    pub subscription_tier: String,
    
    /// Token expiration time
    pub exp: i64,
    
    /// Token issued at time
    pub iat: i64,
    
    /// Token issuer
    pub iss: String,
    
    /// Audience
    pub aud: String,
    
    /// Session ID
    pub session_id: Uuid,
    
    /// Custom claims
    pub custom: HashMap<String, serde_json::Value>,
}

/// User session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// Session ID
    pub session_id: Uuid,
    
    /// User ID
    pub user_id: String,
    
    /// Tenant ID
    pub tenant_id: Uuid,
    
    /// User roles
    pub roles: Vec<String>,
    
    /// User permissions
    pub permissions: Vec<String>,
    
    /// Session created at
    pub created_at: DateTime<Utc>,
    
    /// Last accessed at
    pub last_accessed: DateTime<Utc>,
    
    /// Session expires at
    pub expires_at: DateTime<Utc>,
    
    /// IP address
    pub ip_address: Option<String>,
    
    /// User agent
    pub user_agent: Option<String>,
    
    /// Active status
    pub is_active: bool,
}

/// Authentication context for requests
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// JWT claims
    pub claims: SaaSClaims,
    
    /// User session
    pub session: UserSession,
    
    /// Tenant information
    pub tenant: Tenant,
    
    /// Is authenticated
    pub is_authenticated: bool,
    
    /// Authentication method
    pub auth_method: AuthMethod,
}

/// Authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// JWT token authentication
    JWT,
    
    /// API key authentication
    ApiKey,
    
    /// SSO authentication
    SSO,
    
    /// Service account authentication
    ServiceAccount,
}

/// SaaS authentication manager
pub struct SaaSAuthManager {
    /// JWT encoding key
    encoding_key: EncodingKey,
    
    /// JWT decoding key
    decoding_key: DecodingKey,
    
    /// Active sessions
    active_sessions: Arc<RwLock<HashMap<Uuid, UserSession>>>,
    
    /// Tenant manager reference
    tenant_manager: Arc<TenantManager>,
    
    /// Configuration
    config: AuthConfig,
    
    /// Background task handles
    background_tasks: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// JWT secret
    pub jwt_secret: String,
    
    /// Token expiration duration
    pub token_expiration: Duration,
    
    /// Session timeout duration
    pub session_timeout: Duration,
    
    /// Enable session cleanup
    pub enable_session_cleanup: bool,
    
    /// Session cleanup interval
    pub cleanup_interval: Duration,
    
    /// Maximum sessions per user
    pub max_sessions_per_user: u32,
    
    /// JWT issuer
    pub jwt_issuer: String,
    
    /// JWT audience
    pub jwt_audience: String,
}

impl SaaSAuthManager {
    /// Create new authentication manager
    pub fn new(tenant_manager: Arc<TenantManager>, config: AuthConfig) -> Result<Self> {
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
        
        Ok(Self {
            encoding_key,
            decoding_key,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            tenant_manager,
            config,
            background_tasks: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// Start the authentication manager
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting SaaS authentication manager");
        
        if self.config.enable_session_cleanup {
            self.start_session_cleanup().await?;
        }
        
        info!("✅ SaaS authentication manager started");
        Ok(())
    }
    
    /// Stop the authentication manager
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping SaaS authentication manager");
        
        // Cancel background tasks
        let mut tasks = self.background_tasks.write().await;
        for task in tasks.drain(..) {
            task.abort();
        }
        
        info!("✅ SaaS authentication manager stopped");
        Ok(())
    }
    
    /// Authenticate user and create session
    pub async fn authenticate_user(
        &self,
        tenant_id: Uuid,
        user_id: &str,
        password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(String, UserSession)> {
        info!("🔐 Authenticating user {} for tenant {}", user_id, tenant_id);
        
        // Verify tenant exists and is active
        let tenant = self.tenant_manager.get_tenant(tenant_id).await?
            .ok_or_else(|| TenantError::NotFound(tenant_id))?;
        
        if !matches!(tenant.status, TenantStatus::Active) {
            return Err(TenantError::Inactive(tenant_id).into());
        }
        
        // In a real implementation, verify password against user store
        // For now, simulate successful authentication
        
        // Create user session
        let session = self.create_user_session(
            tenant_id,
            user_id,
            vec!["user".to_string()], // Default role
            vec![], // Permissions would be loaded from user store
            ip_address,
            user_agent,
        ).await?;
        
        // Generate JWT token
        let token = self.generate_jwt_token(&session, &tenant).await?;
        
        info!("✅ User {} authenticated successfully", user_id);
        Ok((token, session))
    }
    
    /// Validate JWT token and get authentication context
    pub async fn validate_token(&self, token: &str) -> Result<AuthContext> {
        debug!("🔍 Validating JWT token");
        
        // Decode and validate JWT
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&[&self.config.jwt_audience]);
        validation.set_issuer(&[&self.config.jwt_issuer]);
        
        let token_data = decode::<SaaSClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| SaaSError::Internal(anyhow::anyhow!("Invalid token: {}", e)))?;
        
        let claims = token_data.claims;
        
        // Get session information
        let sessions = self.active_sessions.read().await;
        let session = sessions.get(&claims.session_id)
            .ok_or_else(|| SaaSError::Internal(anyhow::anyhow!("Session not found")))?
            .clone();
        
        // Verify session is still active
        if !session.is_active || session.expires_at < Utc::now() {
            return Err(SaaSError::Internal(anyhow::anyhow!("Session expired")));
        }
        
        // Get tenant information
        let tenant = self.tenant_manager.get_tenant(claims.tenant_id).await?
            .ok_or_else(|| TenantError::NotFound(claims.tenant_id))?;
        
        // Update last accessed time
        drop(sessions);
        self.update_session_access(claims.session_id).await?;
        
        Ok(AuthContext {
            claims,
            session,
            tenant,
            is_authenticated: true,
            auth_method: AuthMethod::JWT,
        })
    }
    
    /// Create user session
    async fn create_user_session(
        &self,
        tenant_id: Uuid,
        user_id: &str,
        roles: Vec<String>,
        permissions: Vec<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<UserSession> {
        let session_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + self.config.session_timeout;
        
        let session = UserSession {
            session_id,
            user_id: user_id.to_string(),
            tenant_id,
            roles,
            permissions,
            created_at: now,
            last_accessed: now,
            expires_at,
            ip_address,
            user_agent,
            is_active: true,
        };
        
        // Store session
        let mut sessions = self.active_sessions.write().await;
        
        // Check session limit per user
        let user_sessions = sessions.values()
            .filter(|s| s.user_id == user_id && s.tenant_id == tenant_id && s.is_active)
            .count();
        
        if user_sessions >= self.config.max_sessions_per_user as usize {
            // Remove oldest session
            if let Some(oldest_session_id) = sessions.values()
                .filter(|s| s.user_id == user_id && s.tenant_id == tenant_id && s.is_active)
                .min_by_key(|s| s.created_at)
                .map(|s| s.session_id) {
                sessions.remove(&oldest_session_id);
            }
        }
        
        sessions.insert(session_id, session.clone());
        
        debug!("📝 Created session {} for user {}", session_id, user_id);
        Ok(session)
    }
    
    /// Generate JWT token for session
    async fn generate_jwt_token(&self, session: &UserSession, tenant: &Tenant) -> Result<String> {
        let now = Utc::now();
        let exp = now + self.config.token_expiration;
        
        let claims = SaaSClaims {
            sub: session.user_id.clone(),
            tenant_id: session.tenant_id,
            roles: session.roles.clone(),
            permissions: session.permissions.clone(),
            subscription_tier: tenant.subscription_tier.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            iss: self.config.jwt_issuer.clone(),
            aud: self.config.jwt_audience.clone(),
            session_id: session.session_id,
            custom: HashMap::new(),
        };
        
        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| SaaSError::Internal(anyhow::anyhow!("Failed to generate token: {}", e)))?;
        
        debug!("🎫 Generated JWT token for session {}", session.session_id);
        Ok(token)
    }
    
    /// Update session last accessed time
    async fn update_session_access(&self, session_id: Uuid) -> Result<()> {
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.last_accessed = Utc::now();
        }
        Ok(())
    }
    
    /// Revoke session
    pub async fn revoke_session(&self, session_id: Uuid) -> Result<()> {
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.is_active = false;
            info!("🚫 Revoked session {}", session_id);
        }
        Ok(())
    }
    
    /// Get active sessions for user
    pub async fn get_user_sessions(&self, tenant_id: Uuid, user_id: &str) -> Result<Vec<UserSession>> {
        let sessions = self.active_sessions.read().await;
        let user_sessions = sessions.values()
            .filter(|s| s.user_id == user_id && s.tenant_id == tenant_id && s.is_active)
            .cloned()
            .collect();
        
        Ok(user_sessions)
    }
    
    /// Start session cleanup task
    async fn start_session_cleanup(&self) -> Result<()> {
        let active_sessions = Arc::clone(&self.active_sessions);
        let cleanup_interval = self.config.cleanup_interval;
        
        let task = tokio::spawn(async move {
            info!("🔄 Starting session cleanup task");
            
            let mut interval = tokio::time::interval(cleanup_interval.to_std().unwrap());
            
            loop {
                interval.tick().await;
                
                let now = Utc::now();
                let mut sessions = active_sessions.write().await;
                let initial_count = sessions.len();
                
                // Remove expired sessions
                sessions.retain(|_, session| {
                    session.is_active && session.expires_at > now
                });
                
                let final_count = sessions.len();
                if initial_count > final_count {
                    debug!("🧹 Cleaned up {} expired sessions", initial_count - final_count);
                }
            }
        });
        
        self.background_tasks.write().await.push(task);
        Ok(())
    }
}

/// Axum middleware for SaaS authentication
pub async fn saas_auth_middleware(
    State(auth_manager): State<Arc<SaaSAuthManager>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
    let token = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Validate token and get auth context
    let auth_context = auth_manager.validate_token(token).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Add auth context to request extensions
    request.extensions_mut().insert(auth_context);
    
    // Continue to next middleware/handler
    Ok(next.run(request).await)
}

/// Extract authentication context from request
pub fn extract_auth_context(request: &Request) -> Option<&AuthContext> {
    request.extensions().get::<AuthContext>()
}

/// Check if user has required permission
pub fn has_permission(auth_context: &AuthContext, required_permission: &str) -> bool {
    auth_context.session.permissions.contains(&required_permission.to_string()) ||
    auth_context.session.roles.contains(&"admin".to_string())
}

/// Check if user has required role
pub fn has_role(auth_context: &AuthContext, required_role: &str) -> bool {
    auth_context.session.roles.contains(&required_role.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::{TenantManager, TenantStatus};
    
    #[tokio::test]
    async fn test_saas_auth_manager() {
        let tenant_manager = Arc::new(TenantManager::new().await.unwrap());
        let config = AuthConfig {
            jwt_secret: "test_secret".to_string(),
            token_expiration: Duration::hours(1),
            session_timeout: Duration::hours(24),
            enable_session_cleanup: false,
            cleanup_interval: Duration::minutes(15),
            max_sessions_per_user: 5,
            jwt_issuer: "aerolithdb".to_string(),
            jwt_audience: "aerolithdb-api".to_string(),
        };
        
        let auth_manager = SaaSAuthManager::new(tenant_manager.clone(), config).unwrap();
        auth_manager.start().await.unwrap();
        
        // Create test tenant
        let tenant = Tenant {
            id: Uuid::new_v4(),
            organization_name: "Test Org".to_string(),
            organization_domain: Some("test.com".to_string()),
            subscription_tier: "professional".to_string(),
            status: TenantStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        tenant_manager.create_tenant(tenant.clone()).await.unwrap();
        
        // Test authentication
        let (token, session) = auth_manager.authenticate_user(
            tenant.id,
            "test_user",
            "password",
            Some("127.0.0.1".to_string()),
            Some("test-agent".to_string()),
        ).await.unwrap();
        
        assert!(!token.is_empty());
        assert_eq!(session.user_id, "test_user");
        assert_eq!(session.tenant_id, tenant.id);
        
        // Test token validation
        let auth_context = auth_manager.validate_token(&token).await.unwrap();
        assert!(auth_context.is_authenticated);
        assert_eq!(auth_context.session.user_id, "test_user");
        
        auth_manager.stop().await.unwrap();
    }
}
