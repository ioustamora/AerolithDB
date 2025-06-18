//! Error types for SaaS infrastructure

use thiserror::Error;

/// Main error type for SaaS operations
#[derive(Error, Debug)]
pub enum SaaSError {
    /// Tenant-related errors
    #[error("Tenant error: {0}")]
    Tenant(#[from] TenantError),
    
    /// Usage tracking errors
    #[error("Usage tracking error: {0}")]
    Usage(#[from] UsageError),
    
    /// Billing-related errors
    #[error("Billing error: {0}")]
    Billing(#[from] BillingError),
    
    /// Quota enforcement errors
    #[error("Quota error: {0}")]
    Quota(#[from] QuotaError),
    
    /// Provisioning errors
    #[error("Provisioning error: {0}")]
    Provisioning(#[from] ProvisioningError),
    
    /// SSO-related errors
    #[error("SSO error: {0}")]
    SSO(#[from] SSOError),
    
    /// Analytics errors
    #[error("Analytics error: {0}")]
    Analytics(#[from] AnalyticsError),
    
    /// Database errors
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Generic errors
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Tenant management errors
#[derive(Error, Debug)]
pub enum TenantError {
    /// Tenant not found
    #[error("Tenant not found: {tenant_id}")]
    NotFound { tenant_id: String },
    
    /// Tenant already exists
    #[error("Tenant already exists: {tenant_id}")]
    AlreadyExists { tenant_id: String },
    
    /// Invalid tenant configuration
    #[error("Invalid tenant configuration: {message}")]
    InvalidConfig { message: String },
    
    /// Tenant limit exceeded
    #[error("Tenant limit exceeded: {limit_type}")]
    LimitExceeded { limit_type: String },
    
    /// Isolation violation
    #[error("Tenant isolation violation: {message}")]
    IsolationViolation { message: String },
    
    /// Resource allocation failed
    #[error("Resource allocation failed: {resource}")]
    ResourceAllocationFailed { resource: String },
}

/// Usage tracking errors
#[derive(Error, Debug)]
pub enum UsageError {
    /// Metrics collection failed
    #[error("Metrics collection failed: {message}")]
    CollectionFailed { message: String },
    
    /// Invalid metrics data
    #[error("Invalid metrics data: {message}")]
    InvalidData { message: String },
    
    /// Aggregation failed
    #[error("Metrics aggregation failed: {message}")]
    AggregationFailed { message: String },
    
    /// Storage failed
    #[error("Metrics storage failed: {message}")]
    StorageFailed { message: String },
    
    /// Query failed
    #[error("Metrics query failed: {message}")]
    QueryFailed { message: String },
}

/// Billing system errors
#[derive(Error, Debug)]
pub enum BillingError {
    /// Billing calculation failed
    #[error("Billing calculation failed: {message}")]
    CalculationFailed { message: String },
    
    /// Invalid pricing configuration
    #[error("Invalid pricing configuration: {message}")]
    InvalidPricing { message: String },
    
    /// Payment processing failed
    #[error("Payment processing failed: {message}")]
    PaymentFailed { message: String },
    
    /// Invoice generation failed
    #[error("Invoice generation failed: {message}")]
    InvoiceGenerationFailed { message: String },
    
    /// Billing provider error
    #[error("Billing provider error: {provider} - {message}")]
    ProviderError { provider: String, message: String },
    
    /// Insufficient funds
    #[error("Insufficient funds for tenant: {tenant_id}")]
    InsufficientFunds { tenant_id: String },
    
    /// Payment method required
    #[error("Payment method required for tenant: {tenant_id}")]
    PaymentMethodRequired { tenant_id: String },
}

/// Quota management errors
#[derive(Error, Debug)]
pub enum QuotaError {
    /// Quota exceeded
    #[error("Quota exceeded for tenant {tenant_id}: {quota_type} = {current}/{limit}")]
    QuotaExceeded {
        tenant_id: String,
        quota_type: String,
        current: u64,
        limit: u64,
    },
    
    /// Invalid quota configuration
    #[error("Invalid quota configuration: {message}")]
    InvalidConfig { message: String },
    
    /// Quota enforcement failed
    #[error("Quota enforcement failed: {message}")]
    EnforcementFailed { message: String },
    
    /// Quota check failed
    #[error("Quota check failed: {message}")]
    CheckFailed { message: String },
}

/// Provisioning errors
#[derive(Error, Debug)]
pub enum ProvisioningError {
    /// Cluster creation failed
    #[error("Cluster creation failed: {message}")]
    ClusterCreationFailed { message: String },
    
    /// Cluster deletion failed
    #[error("Cluster deletion failed: {message}")]
    ClusterDeletionFailed { message: String },
    
    /// Scaling failed
    #[error("Cluster scaling failed: {message}")]
    ScalingFailed { message: String },
    
    /// Cloud provider error
    #[error("Cloud provider error: {provider} - {message}")]
    CloudProviderError { provider: String, message: String },
    
    /// Resource unavailable
    #[error("Resource unavailable: {resource}")]
    ResourceUnavailable { resource: String },
    
    /// Invalid cluster configuration
    #[error("Invalid cluster configuration: {message}")]
    InvalidConfig { message: String },
}

/// SSO integration errors
#[derive(Error, Debug)]
pub enum SSOError {
    /// Authentication failed
    #[error("SSO authentication failed: {message}")]
    AuthenticationFailed { message: String },
    
    /// Invalid SSO configuration
    #[error("Invalid SSO configuration: {message}")]
    InvalidConfig { message: String },
    
    /// Provider unavailable
    #[error("SSO provider unavailable: {provider}")]
    ProviderUnavailable { provider: String },
    
    /// Token validation failed
    #[error("Token validation failed: {message}")]
    TokenValidationFailed { message: String },
    
    /// User mapping failed
    #[error("User mapping failed: {message}")]
    UserMappingFailed { message: String },
}

/// Analytics errors
#[derive(Error, Debug)]
pub enum AnalyticsError {
    /// Data processing failed
    #[error("Analytics data processing failed: {message}")]
    ProcessingFailed { message: String },
    
    /// Invalid analytics configuration
    #[error("Invalid analytics configuration: {message}")]
    InvalidConfig { message: String },
    
    /// Query execution failed
    #[error("Analytics query execution failed: {message}")]
    QueryFailed { message: String },
    
    /// Report generation failed
    #[error("Report generation failed: {message}")]
    ReportGenerationFailed { message: String },
    
    /// Machine learning model error
    #[error("ML model error: {message}")]
    MLModelError { message: String },
}

/// Result type alias for SaaS operations
pub type SaaSResult<T> = Result<T, SaaSError>;

/// Result type alias for tenant operations
pub type TenantResult<T> = Result<T, TenantError>;

/// Result type alias for usage operations
pub type UsageResult<T> = Result<T, UsageError>;

/// Result type alias for billing operations
pub type BillingResult<T> = Result<T, BillingError>;

/// Result type alias for quota operations
pub type QuotaResult<T> = Result<T, QuotaError>;

/// Result type alias for provisioning operations
pub type ProvisioningResult<T> = Result<T, ProvisioningError>;

/// Result type alias for SSO operations
pub type SSOResult<T> = Result<T, SSOError>;

/// Result type alias for analytics operations
pub type AnalyticsResult<T> = Result<T, AnalyticsError>;
