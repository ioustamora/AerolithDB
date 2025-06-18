//! Billing and invoice generation system
//! 
//! Automated billing calculations, invoice generation, and payment processing
//! integration for SaaS operations.

use crate::config::{BillingConfig, BillingProvider, PricingTier};
use crate::errors::{BillingError, BillingResult};
use crate::usage::{UsageStatistics, UsageTracker};
use crate::tenant::{Tenant, TenantManager};
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::interval;
use tracing::{info, debug, warn, error};
use uuid::Uuid;

/// Invoice data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    /// Unique invoice identifier
    pub invoice_id: Uuid,
    
    /// Tenant identifier
    pub tenant_id: Uuid,
    
    /// Invoice number (human-readable)
    pub invoice_number: String,
    
    /// Billing period start
    pub period_start: DateTime<Utc>,
    
    /// Billing period end
    pub period_end: DateTime<Utc>,
    
    /// Line items
    pub line_items: Vec<InvoiceLineItem>,
    
    /// Subtotal before tax
    pub subtotal: f64,
    
    /// Tax amount
    pub tax_amount: f64,
    
    /// Total amount due
    pub total_amount: f64,
    
    /// Currency
    pub currency: String,
    
    /// Invoice status
    pub status: InvoiceStatus,
    
    /// Due date
    pub due_date: DateTime<Utc>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Payment information
    pub payment_info: Option<PaymentInfo>,
    
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Invoice line item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    /// Line item description
    pub description: String,
    
    /// Quantity
    pub quantity: f64,
    
    /// Unit price
    pub unit_price: f64,
    
    /// Total price for this line item
    pub total_price: f64,
    
    /// Pricing tier this item belongs to
    pub pricing_tier: Option<String>,
    
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Invoice status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvoiceStatus {
    /// Draft invoice, not yet sent
    Draft,
    
    /// Sent to customer
    Sent {
        sent_at: DateTime<Utc>,
    },
    
    /// Payment pending
    Pending,
    
    /// Fully paid
    Paid {
        paid_at: DateTime<Utc>,
        payment_method: String,
    },
    
    /// Partially paid
    PartiallyPaid {
        amount_paid: f64,
        last_payment_at: DateTime<Utc>,
    },
    
    /// Overdue
    Overdue {
        overdue_since: DateTime<Utc>,
    },
    
    /// Failed payment
    Failed {
        failure_reason: String,
        failed_at: DateTime<Utc>,
    },
    
    /// Cancelled
    Cancelled {
        reason: String,
        cancelled_at: DateTime<Utc>,
    },
}

/// Payment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInfo {
    /// Payment method (credit card, bank transfer, etc.)
    pub payment_method: String,
    
    /// External payment ID (from payment processor)
    pub external_payment_id: Option<String>,
    
    /// Amount paid
    pub amount_paid: f64,
    
    /// Payment timestamp
    pub paid_at: DateTime<Utc>,
    
    /// Transaction fees
    pub transaction_fee: f64,
    
    /// Payment status
    pub status: PaymentStatus,
}

/// Payment method information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethod {
    pub id: String,
    pub payment_type: PaymentType,
    pub is_default: bool,
    pub billing_address: Option<BillingAddress>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Payment method types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentType {
    CreditCard {
        last_four: String,
        brand: String,
        exp_month: u32,
        exp_year: u32,
    },
    Crypto {
        wallet_address: String,
        blockchain: String,
        token: String,
    },
    BankTransfer {
        account_last_four: String,
        routing_number: String,
    },
    PayPal {
        email: String,
    },
}

/// Billing address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAddress {
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
}

/// Payment processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    pub payment_id: String,
    pub status: PaymentStatus,
    pub amount: u64, // in cents
    pub currency: String,
    pub processed_at: DateTime<Utc>,
    pub failure_reason: Option<String>,
    pub transaction_id: Option<String>,
}

/// Payment status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Cancelled,
    Requires3DSecure,
    RequiresCapture,
}

/// Billing calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingCalculation {
    /// Tenant identifier
    pub tenant_id: Uuid,
    
    /// Billing period
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    
    /// Usage statistics used for calculation
    pub usage_stats: Vec<UsageStatistics>,
    
    /// Applied pricing tier
    pub pricing_tier: PricingTier,
    
    /// Detailed cost breakdown
    pub cost_breakdown: CostBreakdown,
    
    /// Total cost
    pub total_cost: f64,
    
    /// Credits applied
    pub credits_applied: f64,
    
    /// Final amount due
    pub amount_due: f64,
}

/// Detailed cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// Base subscription fee
    pub base_fee: f64,
    
    /// Storage costs
    pub storage_cost: f64,
    
    /// API call costs
    pub api_cost: f64,
    
    /// Compute costs
    pub compute_cost: f64,
    
    /// Network transfer costs
    pub network_cost: f64,
    
    /// Additional service costs
    pub additional_services: HashMap<String, f64>,
    
    /// Overage charges
    pub overage_charges: f64,
}

/// Billing engine for automated billing calculations
pub struct BillingEngine {
    config: BillingConfig,
    db_pool: PgPool,
    usage_tracker: Arc<UsageTracker>,
    tenant_manager: Arc<TenantManager>,
    is_running: Arc<tokio::sync::RwLock<bool>>,
}

impl BillingEngine {
    /// Create a new billing engine
    pub async fn new(config: &BillingConfig) -> Result<Self> {
        info!("💰 Initializing billing engine");
        
        // For this implementation, we'll use the same database as usage tracking
        // In production, you might want a separate billing database
        let db_pool = PgPool::connect("postgresql://localhost/aerolithdb_billing").await?;
        
        // Initialize database schema
        Self::initialize_schema(&db_pool).await?;
        
        // Note: In a real implementation, these would be injected dependencies
        let usage_tracker = Arc::new(UsageTracker::new(&crate::config::UsageConfig::default()).await?);
        let tenant_manager = Arc::new(TenantManager::new(&crate::config::TenantConfig::default()).await?);
        
        let engine = Self {
            config: config.clone(),
            db_pool,
            usage_tracker,
            tenant_manager,
            is_running: Arc::new(tokio::sync::RwLock::new(false)),
        };
        
        info!("✅ Billing engine initialized");
        Ok(engine)
    }
    
    /// Initialize database schema for billing
    async fn initialize_schema(pool: &PgPool) -> Result<()> {
        debug!("💰 Initializing billing database schema");
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS invoices (
                invoice_id UUID PRIMARY KEY,
                tenant_id UUID NOT NULL,
                invoice_number VARCHAR NOT NULL UNIQUE,
                period_start TIMESTAMPTZ NOT NULL,
                period_end TIMESTAMPTZ NOT NULL,
                line_items JSONB NOT NULL,
                subtotal DECIMAL(12,2) NOT NULL,
                tax_amount DECIMAL(12,2) NOT NULL,
                total_amount DECIMAL(12,2) NOT NULL,
                currency VARCHAR(3) NOT NULL,
                status JSONB NOT NULL,
                due_date TIMESTAMPTZ NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                payment_info JSONB,
                metadata JSONB NOT NULL DEFAULT '{}'
            );
            
            CREATE INDEX IF NOT EXISTS idx_invoices_tenant_id ON invoices(tenant_id);
            CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices((status->>'status'));
            CREATE INDEX IF NOT EXISTS idx_invoices_due_date ON invoices(due_date);
            CREATE INDEX IF NOT EXISTS idx_invoices_period ON invoices(period_start, period_end);
            
            CREATE TABLE IF NOT EXISTS billing_calculations (
                id BIGSERIAL PRIMARY KEY,
                tenant_id UUID NOT NULL,
                period_start TIMESTAMPTZ NOT NULL,
                period_end TIMESTAMPTZ NOT NULL,
                usage_stats JSONB NOT NULL,
                pricing_tier JSONB NOT NULL,
                cost_breakdown JSONB NOT NULL,
                total_cost DECIMAL(12,2) NOT NULL,
                credits_applied DECIMAL(12,2) NOT NULL DEFAULT 0,
                amount_due DECIMAL(12,2) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_billing_calculations_tenant 
            ON billing_calculations(tenant_id);
            
            CREATE TABLE IF NOT EXISTS payment_transactions (
                id BIGSERIAL PRIMARY KEY,
                invoice_id UUID REFERENCES invoices(invoice_id),
                tenant_id UUID NOT NULL,
                external_payment_id VARCHAR,
                amount DECIMAL(12,2) NOT NULL,
                currency VARCHAR(3) NOT NULL,
                payment_method VARCHAR NOT NULL,
                status VARCHAR NOT NULL,
                transaction_fee DECIMAL(12,2) NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                completed_at TIMESTAMPTZ,
                metadata JSONB NOT NULL DEFAULT '{}'
            );
            
            CREATE INDEX IF NOT EXISTS idx_payment_transactions_invoice 
            ON payment_transactions(invoice_id);
            CREATE INDEX IF NOT EXISTS idx_payment_transactions_tenant 
            ON payment_transactions(tenant_id);
            "#
        )
        .execute(pool)
        .await?;
        
        debug!("✅ Billing database schema initialized");
        Ok(())
    }
    
    /// Start billing cycle processing
    pub async fn start_billing_cycle(&self) -> Result<()> {
        info!("🔄 Starting billing cycle processing");
        
        {
            let mut is_running = self.is_running.write().await;
            if *is_running {
                warn!("Billing cycle already running");
                return Ok(());
            }
            *is_running = true;
        }
        
        if self.config.enabled {
            let config = self.config.clone();
            let db_pool = self.db_pool.clone();
            let usage_tracker = Arc::clone(&self.usage_tracker);
            let tenant_manager = Arc::clone(&self.tenant_manager);
            let is_running = Arc::clone(&self.is_running);
            
            tokio::spawn(async move {
                let mut interval = interval(config.billing_interval);
                
                loop {
                    interval.tick().await;
                    
                    let running = { *is_running.read().await };
                    if !running {
                        break;
                    }
                    
                    // Process billing for all tenants
                    if let Err(e) = Self::process_billing_cycle(
                        &db_pool, 
                        &config, 
                        &usage_tracker, 
                        &tenant_manager
                    ).await {
                        error!("Billing cycle processing failed: {}", e);
                    }
                }
            });
        }
        
        info!("✅ Billing cycle processing started");
        Ok(())
    }
    
    /// Stop billing cycle processing
    pub async fn stop_billing_cycle(&self) -> Result<()> {
        info!("🛑 Stopping billing cycle processing");
        
        {
            let mut is_running = self.is_running.write().await;
            *is_running = false;
        }
        
        info!("✅ Billing cycle processing stopped");
        Ok(())
    }
    
    /// Process billing cycle for all tenants
    async fn process_billing_cycle(
        db_pool: &PgPool,
        config: &BillingConfig,
        usage_tracker: &UsageTracker,
        tenant_manager: &TenantManager,
    ) -> Result<()> {
        debug!("💰 Processing billing cycle");
        
        let end_time = Utc::now();
        let start_time = end_time - config.billing_interval;
        
        // Get all active tenants
        let tenants = tenant_manager.list_tenants(Some(1000), Some(0)).await?;
        
        for tenant in tenants {
            if let Err(e) = Self::process_tenant_billing(
                db_pool,
                config,
                usage_tracker,
                &tenant,
                start_time,
                end_time,
            ).await {
                error!("Failed to process billing for tenant {}: {}", tenant.tenant_id, e);
            }
        }
        
        debug!("✅ Billing cycle processing completed");
        Ok(())
    }
    
    /// Process billing for a specific tenant
    async fn process_tenant_billing(
        db_pool: &PgPool,
        config: &BillingConfig,
        usage_tracker: &UsageTracker,
        tenant: &Tenant,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> BillingResult<()> {
        debug!("💰 Processing billing for tenant: {}", tenant.tenant_id);
        
        // Get usage statistics for the billing period
        let usage_stats = usage_tracker
            .get_usage_statistics(tenant.tenant_id, start_time, end_time)
            .await
            .map_err(|e| BillingError::CalculationFailed {
                message: format!("Failed to get usage statistics: {}", e),
            })?;
        
        if usage_stats.is_empty() {
            debug!("No usage statistics found for tenant: {}", tenant.tenant_id);
            return Ok(());
        }
        
        // Find applicable pricing tier
        let pricing_tier = config
            .pricing_tiers
            .iter()
            .find(|tier| tier.name == tenant.subscription_tier)
            .ok_or_else(|| BillingError::InvalidPricing {
                message: format!("Pricing tier not found: {}", tenant.subscription_tier),
            })?;
        
        // Calculate billing
        let calculation = Self::calculate_billing(
            tenant,
            pricing_tier,
            &usage_stats,
            start_time,
            end_time,
        )?;
        
        // Store billing calculation
        Self::store_billing_calculation(db_pool, &calculation).await?;
        
        // Generate invoice if amount due
        if calculation.amount_due > 0.0 {
            let invoice = Self::generate_invoice(config, tenant, &calculation)?;
            Self::store_invoice(db_pool, &invoice).await?;
            
            info!("💰 Generated invoice for tenant {}: ${:.2}", 
                  tenant.tenant_id, invoice.total_amount);
        }
        
        Ok(())
    }
    
    /// Calculate billing for a tenant
    fn calculate_billing(
        tenant: &Tenant,
        pricing_tier: &PricingTier,
        usage_stats: &[UsageStatistics],
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> BillingResult<BillingCalculation> {
        let mut cost_breakdown = CostBreakdown {
            base_fee: pricing_tier.base_fee,
            storage_cost: 0.0,
            api_cost: 0.0,
            compute_cost: 0.0,
            network_cost: 0.0,
            additional_services: HashMap::new(),
            overage_charges: 0.0,
        };
        
        // Aggregate usage across all statistics
        let mut total_storage_gb = 0.0;
        let mut total_api_calls = 0u64;
        let mut total_cpu_hours = 0.0;
        let mut total_network_gb = 0.0;
        
        for stats in usage_stats {
            total_storage_gb += stats.aggregated_metrics.storage_usage.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            total_api_calls += stats.aggregated_metrics.api_calls.total_calls;
            total_cpu_hours += stats.aggregated_metrics.compute_usage.cpu_hours;
            total_network_gb += (stats.aggregated_metrics.network_usage.bytes_in + 
                               stats.aggregated_metrics.network_usage.bytes_out) as f64 / (1024.0 * 1024.0 * 1024.0);
        }
        
        // Calculate costs
        cost_breakdown.storage_cost = total_storage_gb * pricing_tier.storage_price_per_gb;
        cost_breakdown.api_cost = (total_api_calls as f64 / 1000.0) * pricing_tier.api_price_per_1k_calls;
        cost_breakdown.compute_cost = total_cpu_hours * pricing_tier.compute_price_per_cpu_hour;
        cost_breakdown.network_cost = total_network_gb * pricing_tier.network_price_per_gb;
        
        // Check for overages
        if total_storage_gb > pricing_tier.included_quotas.max_storage_bytes as f64 / (1024.0 * 1024.0 * 1024.0) {
            let overage_gb = total_storage_gb - pricing_tier.included_quotas.max_storage_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            cost_breakdown.overage_charges += overage_gb * pricing_tier.storage_price_per_gb * 2.0; // 2x rate for overage
        }
        
        let total_cost = cost_breakdown.base_fee + 
                        cost_breakdown.storage_cost + 
                        cost_breakdown.api_cost + 
                        cost_breakdown.compute_cost + 
                        cost_breakdown.network_cost + 
                        cost_breakdown.overage_charges;
        
        Ok(BillingCalculation {
            tenant_id: tenant.tenant_id,
            period_start: start_time,
            period_end: end_time,
            usage_stats: usage_stats.to_vec(),
            pricing_tier: pricing_tier.clone(),
            cost_breakdown,
            total_cost,
            credits_applied: 0.0, // TODO: Implement credits system
            amount_due: total_cost,
        })
    }
    
    /// Generate invoice from billing calculation
    fn generate_invoice(
        config: &BillingConfig,
        tenant: &Tenant,
        calculation: &BillingCalculation,
    ) -> BillingResult<Invoice> {
        let invoice_id = Uuid::new_v4();
        let invoice_number = format!("INV-{}-{}", 
                                   tenant.tenant_id.simple(), 
                                   Utc::now().format("%Y%m%d"));
        
        let mut line_items = Vec::new();
        
        // Base fee line item
        if calculation.cost_breakdown.base_fee > 0.0 {
            line_items.push(InvoiceLineItem {
                description: format!("{} Plan - Base Fee", calculation.pricing_tier.name),
                quantity: 1.0,
                unit_price: calculation.cost_breakdown.base_fee,
                total_price: calculation.cost_breakdown.base_fee,
                pricing_tier: Some(calculation.pricing_tier.name.clone()),
                metadata: HashMap::new(),
            });
        }
        
        // Storage line item
        if calculation.cost_breakdown.storage_cost > 0.0 {
            let storage_gb = calculation.usage_stats.iter()
                .map(|s| s.aggregated_metrics.storage_usage.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
                .sum::<f64>();
            
            line_items.push(InvoiceLineItem {
                description: "Storage Usage".to_string(),
                quantity: storage_gb,
                unit_price: calculation.pricing_tier.storage_price_per_gb,
                total_price: calculation.cost_breakdown.storage_cost,
                pricing_tier: Some(calculation.pricing_tier.name.clone()),
                metadata: HashMap::new(),
            });
        }
        
        // API calls line item
        if calculation.cost_breakdown.api_cost > 0.0 {
            let api_calls = calculation.usage_stats.iter()
                .map(|s| s.aggregated_metrics.api_calls.total_calls)
                .sum::<u64>();
            
            line_items.push(InvoiceLineItem {
                description: "API Calls".to_string(),
                quantity: api_calls as f64 / 1000.0,
                unit_price: calculation.pricing_tier.api_price_per_1k_calls,
                total_price: calculation.cost_breakdown.api_cost,
                pricing_tier: Some(calculation.pricing_tier.name.clone()),
                metadata: HashMap::new(),
            });
        }
        
        let subtotal = calculation.amount_due;
        let tax_amount = subtotal * config.tax_rate;
        let total_amount = subtotal + tax_amount;
        
        Ok(Invoice {
            invoice_id,
            tenant_id: tenant.tenant_id,
            invoice_number,
            period_start: calculation.period_start,
            period_end: calculation.period_end,
            line_items,
            subtotal,
            tax_amount,
            total_amount,
            currency: config.currency.clone(),
            status: InvoiceStatus::Draft,
            due_date: Utc::now() + Duration::days(30), // 30 days due date
            created_at: Utc::now(),
            payment_info: None,
            metadata: HashMap::new(),
        })
    }
    
    /// Store billing calculation in database
    async fn store_billing_calculation(
        db_pool: &PgPool,
        calculation: &BillingCalculation,
    ) -> BillingResult<()> {
        sqlx::query(
            r#"
            INSERT INTO billing_calculations (
                tenant_id, period_start, period_end, usage_stats,
                pricing_tier, cost_breakdown, total_cost, 
                credits_applied, amount_due
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(calculation.tenant_id)
        .bind(calculation.period_start)
        .bind(calculation.period_end)
        .bind(serde_json::to_value(&calculation.usage_stats)?)
        .bind(serde_json::to_value(&calculation.pricing_tier)?)
        .bind(serde_json::to_value(&calculation.cost_breakdown)?)
        .bind(calculation.total_cost)
        .bind(calculation.credits_applied)
        .bind(calculation.amount_due)
        .execute(db_pool)
        .await
        .map_err(|e| BillingError::CalculationFailed {
            message: format!("Failed to store billing calculation: {}", e),
        })?;
        
        Ok(())
    }
    
    /// Store invoice in database
    async fn store_invoice(db_pool: &PgPool, invoice: &Invoice) -> BillingResult<()> {
        sqlx::query(
            r#"
            INSERT INTO invoices (
                invoice_id, tenant_id, invoice_number, period_start, period_end,
                line_items, subtotal, tax_amount, total_amount, currency,
                status, due_date, payment_info, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#
        )
        .bind(invoice.invoice_id)
        .bind(invoice.tenant_id)
        .bind(&invoice.invoice_number)
        .bind(invoice.period_start)
        .bind(invoice.period_end)
        .bind(serde_json::to_value(&invoice.line_items)?)
        .bind(invoice.subtotal)
        .bind(invoice.tax_amount)
        .bind(invoice.total_amount)
        .bind(&invoice.currency)
        .bind(serde_json::to_value(&invoice.status)?)
        .bind(invoice.due_date)
        .bind(serde_json::to_value(&invoice.payment_info)?)
        .bind(serde_json::to_value(&invoice.metadata)?)
        .execute(db_pool)
        .await
        .map_err(|e| BillingError::InvoiceGenerationFailed {
            message: format!("Failed to store invoice: {}", e),
        })?;
        
        Ok(())
    }
    
    /// Get invoices for a tenant
    pub async fn get_tenant_invoices(
        &self,
        tenant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> BillingResult<Vec<Invoice>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        
        let rows = sqlx::query(
            "SELECT * FROM invoices WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(tenant_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| BillingError::InvoiceGenerationFailed {
            message: format!("Failed to fetch invoices: {}", e),
        })?;
        
        let mut invoices = Vec::new();
        for row in rows {
            let invoice = Invoice {
                invoice_id: row.try_get("invoice_id")?,
                tenant_id: row.try_get("tenant_id")?,
                invoice_number: row.try_get("invoice_number")?,
                period_start: row.try_get("period_start")?,
                period_end: row.try_get("period_end")?,
                line_items: serde_json::from_value(row.try_get("line_items")?)?,
                subtotal: row.try_get::<rust_decimal::Decimal, _>("subtotal")?.to_f64().unwrap_or(0.0),
                tax_amount: row.try_get::<rust_decimal::Decimal, _>("tax_amount")?.to_f64().unwrap_or(0.0),
                total_amount: row.try_get::<rust_decimal::Decimal, _>("total_amount")?.to_f64().unwrap_or(0.0),
                currency: row.try_get("currency")?,
                status: serde_json::from_value(row.try_get("status")?)?,
                due_date: row.try_get("due_date")?,
                created_at: row.try_get("created_at")?,
                payment_info: serde_json::from_value(row.try_get("payment_info")?)?,
                metadata: serde_json::from_value(row.try_get("metadata")?)?,
            };
            invoices.push(invoice);
        }
        
        Ok(invoices)
    }
    
    /// Get overdue invoices
    pub async fn get_overdue_invoices(&self) -> Result<Vec<Invoice>> {
        let now = Utc::now();
        
        let rows = sqlx::query(
            "SELECT * FROM invoices WHERE due_date < $1 AND (status->>'status') = 'Pending'"
        )
        .bind(now)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| BillingError::InvoiceGenerationFailed {
            message: format!("Failed to fetch overdue invoices: {}", e),
        })?;
        
        let mut invoices = Vec::new();
        for row in rows {
            let invoice = Invoice {
                invoice_id: row.try_get("invoice_id")?,
                tenant_id: row.try_get("tenant_id")?,
                invoice_number: row.try_get("invoice_number")?,
                period_start: row.try_get("period_start")?,
                period_end: row.try_get("period_end")?,
                line_items: serde_json::from_value(row.try_get("line_items")?)?,
                subtotal: row.try_get::<rust_decimal::Decimal, _>("subtotal")?.to_f64().unwrap_or(0.0),
                tax_amount: row.try_get::<rust_decimal::Decimal, _>("tax_amount")?.to_f64().unwrap_or(0.0),
                total_amount: row.try_get::<rust_decimal::Decimal, _>("total_amount")?.to_f64().unwrap_or(0.0),
                currency: row.try_get("currency")?,
                status: serde_json::from_value(row.try_get("status")?)?,
                due_date: row.try_get("due_date")?,
                created_at: row.try_get("created_at")?,
                payment_info: serde_json::from_value(row.try_get("payment_info")?)?,
                metadata: serde_json::from_value(row.try_get("metadata")?)?,
            };
            invoices.push(invoice);
        }
        
        Ok(invoices)
    }
    
    /// Get failed payments
    pub async fn get_failed_payments(&self) -> Result<Vec<PaymentResult>> {
        let rows = sqlx::query(
            "SELECT * FROM payment_transactions WHERE status = 'Failed'"
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| BillingError::InvoiceGenerationFailed {
            message: format!("Failed to fetch failed payments: {}", e),
        })?;
        
        let mut payments = Vec::new();
        for row in rows {
            let payment = PaymentResult {
                payment_id: row.try_get("id")?.to_string(),
                status: serde_json::from_value(row.try_get("status")?)?,
                amount: (row.try_get::<rust_decimal::Decimal, _>("amount")?.to_f64().unwrap_or(0.0) * 100.0) as u64,
                currency: row.try_get("currency")?,
                processed_at: row.try_get("created_at")?,
                failure_reason: None, // TODO: Map failure reason
                transaction_id: None, // TODO: Map transaction ID
            };
            payments.push(payment);
        }
        
        Ok(payments)
    }
    
    /// Get expired trials
    pub async fn get_expired_trials(&self) -> Result<Vec<TrialInfo>> {
        let now = Utc::now();
        
        let rows = sqlx::query(
            "SELECT * FROM tenants WHERE trial_ended < $1 AND subscription_tier = 'free'"
        )
        .bind(now)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| BillingError::InvoiceGenerationFailed {
            message: format!("Failed to fetch expired trials: {}", e),
        })?;
        
        let mut trials = Vec::new();
        for row in rows {
            let trial = TrialInfo {
                tenant_id: row.try_get("tenant_id")?,
                trial_started: row.try_get("trial_started")?,
                trial_ended: row.try_get("trial_ended")?,
                plan_id: row.try_get("plan_id")?,
                payment_method_id: row.try_get("payment_method_id")?,
            };
            trials.push(trial);
        }
        
        Ok(trials)
    }
}

/// Automated billing enforcement engine
pub struct BillingEnforcementEngine {
    billing_engine: Arc<BillingEngine>,
    enforcement_enabled: AtomicBool,
    grace_period_hours: u64,
    suspension_enabled: AtomicBool,
}

impl BillingEnforcementEngine {
    pub fn new(billing_engine: Arc<BillingEngine>) -> Self {
        Self {
            billing_engine,
            enforcement_enabled: AtomicBool::new(true),
            grace_period_hours: 72, // 3 days grace period
            suspension_enabled: AtomicBool::new(true),
        }
    }

    /// Start the enforcement background task
    pub async fn start_enforcement_task(&self) -> Result<()> {
        let mut interval = interval(tokio::time::Duration::from_hours(1));
        let enforcement = Arc::new(self.clone());

        tokio::spawn(async move {
            loop {
                interval.tick().await;
                
                if enforcement.enforcement_enabled.load(Ordering::Relaxed) {
                    if let Err(e) = enforcement.process_enforcement_cycle().await {
                        error!("Billing enforcement cycle failed: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Process one enforcement cycle
    async fn process_enforcement_cycle(&self) -> Result<()> {
        info!("Starting billing enforcement cycle");

        // Check for overdue invoices
        let overdue_invoices = self.billing_engine.get_overdue_invoices().await?;
        
        for invoice in overdue_invoices {
            self.handle_overdue_invoice(&invoice).await?;
        }

        // Check for failed payments
        let failed_payments = self.billing_engine.get_failed_payments().await?;
        
        for payment in failed_payments {
            self.handle_failed_payment(&payment).await?;
        }

        // Check for expired trials
        let expired_trials = self.billing_engine.get_expired_trials().await?;
        
        for trial in expired_trials {
            self.handle_expired_trial(&trial).await?;
        }

        info!("Completed billing enforcement cycle");
        Ok(())
    }

    /// Handle overdue invoice
    async fn handle_overdue_invoice(&self, invoice: &Invoice) -> Result<()> {
        let now = Utc::now();
        let overdue_duration = now.signed_duration_since(invoice.due_date);
        
        if overdue_duration.num_hours() < self.grace_period_hours as i64 {
            // Still in grace period, send reminder
            self.send_payment_reminder(invoice).await?;
        } else if self.suspension_enabled.load(Ordering::Relaxed) {
            // Grace period expired, suspend services
            self.suspend_tenant_services(invoice.tenant_id).await?;
            
            // Send suspension notice
            self.send_suspension_notice(invoice).await?;
            
            info!("Suspended services for tenant {} due to overdue invoice {}", 
                  invoice.tenant_id, invoice.invoice_id);
        }
        
        Ok(())
    }

    /// Handle failed payment
    async fn handle_failed_payment(&self, payment: &PaymentResult) -> Result<()> {
        // Retry payment up to 3 times with exponential backoff
        let retry_count = self.get_payment_retry_count(&payment.payment_id).await?;
        
        if retry_count < 3 {
            let delay_hours = 2_u64.pow(retry_count as u32); // 2, 4, 8 hours
            
            tokio::spawn({
                let billing_engine = Arc::clone(&self.billing_engine);
                let payment_id = payment.payment_id.clone();
                
                async move {
                    tokio::time::sleep(tokio::time::Duration::from_hours(delay_hours)).await;
                    
                    if let Err(e) = billing_engine.retry_failed_payment(&payment_id).await {
                        error!("Failed to retry payment {}: {}", payment_id, e);
                    }
                }
            });
        } else {
            // Max retries exceeded, mark as failed
            self.mark_payment_permanently_failed(&payment.payment_id).await?;
            
            // Send payment failure notice
            self.send_payment_failure_notice(payment).await?;
        }
        
        Ok(())
    }

    /// Handle expired trial
    async fn handle_expired_trial(&self, trial_info: &TrialInfo) -> Result<()> {
        // Check if customer has provided payment method
        if trial_info.payment_method_id.is_some() {
            // Attempt to create subscription and charge
            match self.billing_engine.convert_trial_to_subscription(trial_info).await {
                Ok(subscription_id) => {
                    info!("Successfully converted trial to subscription: {}", subscription_id);
                    self.send_trial_conversion_success(trial_info).await?;
                }
                Err(e) => {
                    error!("Failed to convert trial to subscription: {}", e);
                    self.downgrade_to_free_tier(trial_info.tenant_id).await?;
                    self.send_trial_conversion_failure(trial_info).await?;
                }
            }
        } else {
            // No payment method, downgrade to free tier
            self.downgrade_to_free_tier(trial_info.tenant_id).await?;
            self.send_trial_expired_notice(trial_info).await?;
        }
        
        Ok(())
    }

    /// Send payment reminder
    async fn send_payment_reminder(&self, invoice: &Invoice) -> Result<()> {
        // TODO: Integrate with email service
        info!("Sending payment reminder for invoice {} to tenant {}", 
              invoice.invoice_id, invoice.tenant_id);
        Ok(())
    }

    /// Send suspension notice
    async fn send_suspension_notice(&self, invoice: &Invoice) -> Result<()> {
        // TODO: Integrate with email service
        info!("Sending suspension notice for invoice {} to tenant {}", 
              invoice.invoice_id, invoice.tenant_id);
        Ok(())
    }

    /// Send payment failure notice
    async fn send_payment_failure_notice(&self, payment: &PaymentResult) -> Result<()> {
        // TODO: Integrate with email service
        info!("Sending payment failure notice for payment {}", payment.payment_id);
        Ok(())
    }

    /// Send trial conversion success notice
    async fn send_trial_conversion_success(&self, trial_info: &TrialInfo) -> Result<()> {
        info!("Sending trial conversion success notice to tenant {}", trial_info.tenant_id);
        Ok(())
    }

    /// Send trial conversion failure notice
    async fn send_trial_conversion_failure(&self, trial_info: &TrialInfo) -> Result<()> {
        info!("Sending trial conversion failure notice to tenant {}", trial_info.tenant_id);
        Ok(())
    }

    /// Send trial expired notice
    async fn send_trial_expired_notice(&self, trial_info: &TrialInfo) -> Result<()> {
        info!("Sending trial expired notice to tenant {}", trial_info.tenant_id);
        Ok(())
    }

    /// Suspend tenant services
    async fn suspend_tenant_services(&self, tenant_id: Uuid) -> Result<()> {
        // TODO: Integrate with tenant management to suspend services
        info!("Suspending services for tenant {}", tenant_id);
        Ok(())
    }

    /// Downgrade to free tier
    async fn downgrade_to_free_tier(&self, tenant_id: Uuid) -> Result<()> {
        // TODO: Integrate with subscription management
        info!("Downgrading tenant {} to free tier", tenant_id);
        Ok(())
    }

    /// Get payment retry count
    async fn get_payment_retry_count(&self, _payment_id: &str) -> Result<u32> {
        // TODO: Implement actual retry count tracking
        Ok(0)
    }

    /// Mark payment as permanently failed
    async fn mark_payment_permanently_failed(&self, _payment_id: &str) -> Result<()> {
        // TODO: Implement payment status update
        Ok(())
    }

    /// Enable/disable enforcement
    pub fn set_enforcement_enabled(&self, enabled: bool) {
        self.enforcement_enabled.store(enabled, Ordering::Relaxed);
    }

    /// Enable/disable suspension
    pub fn set_suspension_enabled(&self, enabled: bool) {
        self.suspension_enabled.store(enabled, Ordering::Relaxed);
    }

    /// Set grace period
    pub fn set_grace_period_hours(&mut self, hours: u64) {
        self.grace_period_hours = hours;
    }
}

impl Clone for BillingEnforcementEngine {
    fn clone(&self) -> Self {
        Self {
            billing_engine: Arc::clone(&self.billing_engine),
            enforcement_enabled: AtomicBool::new(self.enforcement_enabled.load(Ordering::Relaxed)),
            grace_period_hours: self.grace_period_hours,
            suspension_enabled: AtomicBool::new(self.suspension_enabled.load(Ordering::Relaxed)),
        }
    }
}

/// Trial information for expired trials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialInfo {
    pub tenant_id: Uuid,
    pub trial_started: DateTime<Utc>,
    pub trial_ended: DateTime<Utc>,
    pub plan_id: String,
    pub payment_method_id: Option<String>,
}
