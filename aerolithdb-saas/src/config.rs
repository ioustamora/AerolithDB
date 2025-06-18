//! SaaS configuration management
//! 
//! Centralized configuration for all SaaS features including multi-tenancy,
//! usage tracking, billing, quotas, provisioning, SSO, and analytics.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Main SaaS configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaaSConfig {
    /// Multi-tenancy configuration
    pub tenant: TenantConfig,
    
    /// Usage tracking configuration
    pub usage: UsageConfig,
    
    /// Billing configuration
    pub billing: BillingConfig,
    
    /// Resource quotas configuration
    pub quotas: QuotaConfig,
    
    /// Self-service provisioning configuration
    pub provisioning: ProvisioningConfig,
    
    /// Enterprise SSO configuration
    pub sso: SSOConfig,
    
    /// Analytics configuration
    pub analytics: AnalyticsConfig,
}

/// Multi-tenancy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    /// Enable multi-tenancy features
    pub enabled: bool,
    
    /// Default tenant isolation level
    pub default_isolation_level: IsolationLevel,
    
    /// Database connection for tenant metadata
    pub database_url: String,
    
    /// Maximum number of tenants per cluster
    pub max_tenants_per_cluster: u32,
    
    /// Default resource limits for new tenants
    pub default_limits: TenantLimits,
}

/// Tenant isolation levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// Shared infrastructure with logical separation
    Shared,
    /// Dedicated resources within shared cluster
    Dedicated,
    /// Completely private cluster
    Private,
}

/// Default resource limits for tenants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantLimits {
    /// Maximum storage in bytes
    pub max_storage_bytes: u64,
    
    /// Maximum API calls per hour
    pub max_api_calls_per_hour: u64,
    
    /// Maximum concurrent connections
    pub max_connections: u32,
    
    /// Maximum collections
    pub max_collections: u32,
    
    /// Maximum documents per collection
    pub max_documents_per_collection: u64,
}

/// Usage tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageConfig {
    /// Enable usage tracking
    pub enabled: bool,
    
    /// Metrics collection interval
    pub collection_interval: Duration,
    
    /// Metrics aggregation interval  
    pub aggregation_interval: Duration,
    
    /// Metrics retention period
    pub retention_period: Duration,
    
    /// Database connection for usage data
    pub database_url: String,
    
    /// Track detailed API call metrics
    pub track_api_calls: bool,
    
    /// Track storage usage metrics
    pub track_storage: bool,
    
    /// Track compute usage metrics
    pub track_compute: bool,
    
    /// Track network usage metrics
    pub track_network: bool,
}

/// Billing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingConfig {
    /// Enable automated billing
    pub enabled: bool,
    
    /// Default billing provider
    pub provider: BillingProvider,
    
    /// Billing calculation interval
    pub billing_interval: Duration,
    
    /// Invoice generation interval
    pub invoice_interval: Duration,
    
    /// Default currency
    pub currency: String,
    
    /// Tax rate (as decimal, e.g., 0.08 for 8%)
    pub tax_rate: f64,
    
    /// Pricing tiers
    pub pricing_tiers: Vec<PricingTier>,
    
    /// Payment method requirements
    pub require_payment_method: bool,
    
    /// Grace period for overdue payments
    pub payment_grace_period: Duration,
}

/// Supported billing providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BillingProvider {
    /// Stripe integration
    Stripe {
        api_key: String,
        webhook_secret: String,
    },
    /// AWS Billing integration
    AwsBilling {
        access_key: String,
        secret_key: String,
        region: String,
    },
    /// Manual billing (for testing/custom implementations)
    Manual,
}

/// Pricing tier configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTier {
    /// Tier name (e.g., "starter", "professional", "enterprise")
    pub name: String,
    
    /// Tier description
    pub description: String,
    
    /// Base monthly fee
    pub base_fee: f64,
    
    /// Storage pricing (per GB per month)
    pub storage_price_per_gb: f64,
    
    /// API call pricing (per 1000 calls)
    pub api_price_per_1k_calls: f64,
    
    /// Compute pricing (per CPU hour)
    pub compute_price_per_cpu_hour: f64,
    
    /// Network pricing (per GB transferred)
    pub network_price_per_gb: f64,
    
    /// Included quotas (free tier)
    pub included_quotas: TenantLimits,
    
    /// Maximum limits for this tier
    pub max_limits: TenantLimits,
}

/// Resource quota configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaConfig {
    /// Enable quota enforcement
    pub enabled: bool,
    
    /// Quota check interval
    pub check_interval: Duration,
    
    /// Default quota enforcement action
    pub enforcement_action: QuotaEnforcementAction,
    
    /// Grace period before quota enforcement
    pub grace_period: Duration,
    
    /// Send warnings when approaching limits
    pub enable_warnings: bool,
    
    /// Warning threshold (percentage of quota)
    pub warning_threshold: f64,
}

/// Quota enforcement actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuotaEnforcementAction {
    /// Block new operations
    Block,
    /// Throttle operations
    Throttle,
    /// Allow with billing overage
    AllowWithOverage,
    /// Warn only (no enforcement)
    WarnOnly,
}

/// Self-service provisioning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningConfig {
    /// Enable self-service provisioning
    pub enabled: bool,
    
    /// Default cloud provider
    pub default_provider: CloudProvider,
    
    /// Supported instance types
    pub instance_types: Vec<InstanceType>,
    
    /// Default cluster configuration
    pub default_cluster_config: ClusterConfig,
    
    /// Auto-scaling configuration
    pub auto_scaling: AutoScalingConfig,
    
    /// Kubernetes configuration (if applicable)
    pub kubernetes: Option<KubernetesConfig>,
}

/// Supported cloud providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudProvider {
    Aws {
        access_key: String,
        secret_key: String,
        region: String,
    },
    Azure {
        tenant_id: String,
        client_id: String,
        client_secret: String,
    },
    Gcp {
        project_id: String,
        service_account_key: String,
    },
    /// On-premises deployment
    OnPremises,
}

/// Instance type configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceType {
    /// Instance type name
    pub name: String,
    
    /// CPU cores
    pub cpu_cores: u32,
    
    /// Memory in GB
    pub memory_gb: u32,
    
    /// Storage in GB
    pub storage_gb: u32,
    
    /// Network bandwidth in Mbps
    pub network_mbps: u32,
    
    /// Hourly cost
    pub hourly_cost: f64,
}

/// Default cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Number of nodes
    pub node_count: u32,
    
    /// Replication factor
    pub replication_factor: u32,
    
    /// Enable high availability
    pub high_availability: bool,
    
    /// Enable encryption
    pub encryption_enabled: bool,
    
    /// Backup configuration
    pub backup_enabled: bool,
    
    /// Backup retention days
    pub backup_retention_days: u32,
}

/// Auto-scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoScalingConfig {
    /// Enable auto-scaling
    pub enabled: bool,
    
    /// Minimum number of nodes
    pub min_nodes: u32,
    
    /// Maximum number of nodes
    pub max_nodes: u32,
    
    /// Scale up threshold (CPU percentage)
    pub scale_up_threshold: f64,
    
    /// Scale down threshold (CPU percentage)
    pub scale_down_threshold: f64,
    
    /// Scale up cooldown period
    pub scale_up_cooldown: Duration,
    
    /// Scale down cooldown period
    pub scale_down_cooldown: Duration,
}

/// Kubernetes configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    /// Kubeconfig file path
    pub kubeconfig_path: String,
    
    /// Default namespace
    pub namespace: String,
    
    /// Helm chart repository
    pub helm_chart_repo: String,
    
    /// Helm chart version
    pub helm_chart_version: String,
}

/// Enterprise SSO configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSOConfig {
    /// Enable SSO integration
    pub enabled: bool,
    
    /// Supported SSO providers
    pub providers: Vec<SSOProvider>,
    
    /// Default SSO provider
    pub default_provider: Option<String>,
    
    /// Session timeout
    pub session_timeout: Duration,
    
    /// Require SSO for all users
    pub require_sso: bool,
}

/// SSO provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSOProvider {
    /// Provider name
    pub name: String,
    
    /// Provider type
    pub provider_type: SSOProviderType,
    
    /// Provider-specific configuration
    pub config: SSOProviderConfig,
}

/// SSO provider types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SSOProviderType {
    Saml,
    OAuth2,
    Ldap,
    ActiveDirectory,
}

/// SSO provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SSOProviderConfig {
    Saml {
        idp_url: String,
        idp_certificate: String,
        sp_certificate: String,
        sp_private_key: String,
    },
    OAuth2 {
        client_id: String,
        client_secret: String,
        authorization_url: String,
        token_url: String,
        user_info_url: String,
    },
    Ldap {
        server_url: String,
        bind_dn: String,
        bind_password: String,
        user_search_base: String,
        user_search_filter: String,
    },
    ActiveDirectory {
        domain: String,
        server: String,
        port: u16,
        use_tls: bool,
    },
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// Enable analytics processing
    pub enabled: bool,
    
    /// Analytics data retention period
    pub retention_period: Duration,
    
    /// Analytics processing interval
    pub processing_interval: Duration,
    
    /// Database connection for analytics data
    pub database_url: String,
    
    /// Enable real-time analytics
    pub real_time_enabled: bool,
    
    /// Machine learning insights
    pub ml_insights_enabled: bool,
    
    /// Performance optimization recommendations
    pub optimization_recommendations: bool,
    
    /// Usage pattern analysis
    pub usage_pattern_analysis: bool,
}

impl Default for SaaSConfig {
    fn default() -> Self {
        Self {
            tenant: TenantConfig::default(),
            usage: UsageConfig::default(),
            billing: BillingConfig::default(),
            quotas: QuotaConfig::default(),
            provisioning: ProvisioningConfig::default(),
            sso: SSOConfig::default(),
            analytics: AnalyticsConfig::default(),
        }
    }
}

impl Default for TenantConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_isolation_level: IsolationLevel::Shared,
            database_url: "postgresql://localhost/aerolithdb_saas".to_string(),
            max_tenants_per_cluster: 1000,
            default_limits: TenantLimits::default(),
        }
    }
}

impl Default for TenantLimits {
    fn default() -> Self {
        Self {
            max_storage_bytes: 1_000_000_000, // 1GB
            max_api_calls_per_hour: 10_000,
            max_connections: 100,
            max_collections: 10,
            max_documents_per_collection: 100_000,
        }
    }
}

impl Default for UsageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: Duration::from_secs(60), // 1 minute
            aggregation_interval: Duration::from_secs(3600), // 1 hour
            retention_period: Duration::from_secs(86400 * 90), // 90 days
            database_url: "postgresql://localhost/aerolithdb_usage".to_string(),
            track_api_calls: true,
            track_storage: true,
            track_compute: true,
            track_network: true,
        }
    }
}

impl Default for BillingConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for testing
            provider: BillingProvider::Manual,
            billing_interval: Duration::from_secs(86400), // Daily
            invoice_interval: Duration::from_secs(86400 * 30), // Monthly
            currency: "USD".to_string(),
            tax_rate: 0.0,
            pricing_tiers: vec![PricingTier::default()],
            require_payment_method: false,
            payment_grace_period: Duration::from_secs(86400 * 7), // 7 days
        }
    }
}

impl Default for PricingTier {
    fn default() -> Self {
        Self {
            name: "starter".to_string(),
            description: "Starter tier for small projects".to_string(),
            base_fee: 0.0,
            storage_price_per_gb: 0.10,
            api_price_per_1k_calls: 0.01,
            compute_price_per_cpu_hour: 0.05,
            network_price_per_gb: 0.02,
            included_quotas: TenantLimits::default(),
            max_limits: TenantLimits {
                max_storage_bytes: 10_000_000_000, // 10GB
                max_api_calls_per_hour: 100_000,
                max_connections: 1000,
                max_collections: 100,
                max_documents_per_collection: 1_000_000,
            },
        }
    }
}

impl Default for QuotaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(300), // 5 minutes
            enforcement_action: QuotaEnforcementAction::WarnOnly,
            grace_period: Duration::from_secs(3600), // 1 hour
            enable_warnings: true,
            warning_threshold: 0.8, // 80%
        }
    }
}

impl Default for ProvisioningConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default
            default_provider: CloudProvider::OnPremises,
            instance_types: vec![],
            default_cluster_config: ClusterConfig::default(),
            auto_scaling: AutoScalingConfig::default(),
            kubernetes: None,
        }
    }
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            node_count: 3,
            replication_factor: 2,
            high_availability: true,
            encryption_enabled: true,
            backup_enabled: true,
            backup_retention_days: 30,
        }
    }
}

impl Default for AutoScalingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_nodes: 1,
            max_nodes: 10,
            scale_up_threshold: 80.0,
            scale_down_threshold: 20.0,
            scale_up_cooldown: Duration::from_secs(300),
            scale_down_cooldown: Duration::from_secs(600),
        }
    }
}

impl Default for SSOConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            providers: vec![],
            default_provider: None,
            session_timeout: Duration::from_secs(28800), // 8 hours
            require_sso: false,
        }
    }
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_period: Duration::from_secs(86400 * 365), // 1 year
            processing_interval: Duration::from_secs(3600), // 1 hour
            database_url: "postgresql://localhost/aerolithdb_analytics".to_string(),
            real_time_enabled: true,
            ml_insights_enabled: false,
            optimization_recommendations: true,
            usage_pattern_analysis: true,
        }
    }
}
