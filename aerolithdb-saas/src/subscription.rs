//! Subscription Management System
//! 
//! Provides comprehensive subscription lifecycle management, billing enforcement,
//! and automated payment processing for SaaS operations.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use uuid::Uuid;


use crate::billing::{BillingEngine, Invoice, PaymentMethod};
use crate::errors::SaaSError;

/// Subscription status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionStatus {
    /// Subscription is active and services available
    Active,
    /// Subscription is in trial period
    Trial,
    /// Payment failed, grace period active
    PastDue,
    /// Subscription cancelled but still active until period end
    Cancelled,
    /// Subscription expired, services suspended
    Expired,
    /// Subscription suspended due to policy violation
    Suspended,
    /// Subscription in setup/provisioning state
    Incomplete,
}

/// Subscription billing interval
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BillingInterval {
    Monthly,
    Quarterly,
    Yearly,
    Custom(Duration),
}

/// Subscription plan definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlan {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price_monthly: u64, // in cents
    pub price_yearly: Option<u64>, // in cents, with discount
    pub features: Vec<String>,
    pub limits: PlanLimits,
    pub trial_days: Option<u32>,
    pub is_active: bool,
}

/// Resource limits for subscription plans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanLimits {
    pub api_calls_per_month: Option<u64>,
    pub storage_gb: Option<u64>,
    pub compute_hours_per_month: Option<u64>,
    pub bandwidth_gb_per_month: Option<u64>,
    pub max_databases: Option<u32>,
    pub max_users: Option<u32>,
    pub custom_limits: HashMap<String, serde_json::Value>,
}

/// Active subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub plan_id: String,
    pub status: SubscriptionStatus,
    pub billing_interval: BillingInterval,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub trial_start: Option<DateTime<Utc>>,
    pub trial_end: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub payment_method_id: Option<String>,
    pub discount_percent: Option<f32>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionUsage {
    pub subscription_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub api_calls: u64,
    pub storage_used_gb: f64,
    pub compute_hours: f64,
    pub bandwidth_used_gb: f64,
    pub custom_usage: HashMap<String, f64>,
    pub overage_charges: u64, // in cents
}

/// Subscription modification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionModification {
    pub new_plan_id: Option<String>,
    pub new_billing_interval: Option<BillingInterval>,
    pub proration_behavior: ProrationBehavior,
    pub effective_date: Option<DateTime<Utc>>,
}

/// Proration behavior for subscription changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProrationBehavior {
    CreateProrations,
    None,
    AlwaysInvoice,
}

/// Subscription management engine
pub struct SubscriptionManager {
    subscriptions: Arc<RwLock<HashMap<Uuid, Subscription>>>,
    plans: Arc<RwLock<HashMap<String, SubscriptionPlan>>>,
    usage_tracking: Arc<RwLock<HashMap<Uuid, SubscriptionUsage>>>,
    billing_engine: Arc<BillingEngine>,
}

impl SubscriptionManager {
    pub fn new(billing_engine: Arc<BillingEngine>) -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            plans: Arc::new(RwLock::new(HashMap::new())),
            usage_tracking: Arc::new(RwLock::new(HashMap::new())),
            billing_engine,
        }
    }

    /// Initialize with default subscription plans
    pub async fn initialize_default_plans(&self) -> Result<()> {
        let mut plans = self.plans.write().await;
        
        // Starter plan
        plans.insert("starter".to_string(), SubscriptionPlan {
            id: "starter".to_string(),
            name: "Starter".to_string(),
            description: "Perfect for small projects and prototyping".to_string(),
            price_monthly: 2900, // $29/month
            price_yearly: Some(29000), // $290/year (2 months free)
            features: vec![
                "Up to 3 databases".to_string(),
                "5GB storage".to_string(),
                "100K API calls/month".to_string(),
                "Email support".to_string(),
            ],
            limits: PlanLimits {
                api_calls_per_month: Some(100_000),
                storage_gb: Some(5),
                compute_hours_per_month: Some(100),
                bandwidth_gb_per_month: Some(10),
                max_databases: Some(3),
                max_users: Some(5),
                custom_limits: HashMap::new(),
            },
            trial_days: Some(14),
            is_active: true,
        });

        // Professional plan
        plans.insert("professional".to_string(), SubscriptionPlan {
            id: "professional".to_string(),
            name: "Professional".to_string(),
            description: "Ideal for growing businesses and production workloads".to_string(),
            price_monthly: 9900, // $99/month
            price_yearly: Some(99000), // $990/year (2 months free)
            features: vec![
                "Unlimited databases".to_string(),
                "100GB storage".to_string(),
                "5M API calls/month".to_string(),
                "Priority support".to_string(),
                "Advanced analytics".to_string(),
                "SSO integration".to_string(),
            ],
            limits: PlanLimits {
                api_calls_per_month: Some(5_000_000),
                storage_gb: Some(100),
                compute_hours_per_month: Some(1000),
                bandwidth_gb_per_month: Some(100),
                max_databases: None,
                max_users: Some(25),
                custom_limits: HashMap::new(),
            },
            trial_days: Some(30),
            is_active: true,
        });

        // Enterprise plan
        plans.insert("enterprise".to_string(), SubscriptionPlan {
            id: "enterprise".to_string(),
            name: "Enterprise".to_string(),
            description: "For large organizations with custom requirements".to_string(),
            price_monthly: 49900, // $499/month
            price_yearly: Some(499000), // $4990/year (2 months free)
            features: vec![
                "Unlimited everything".to_string(),
                "1TB+ storage".to_string(),
                "Unlimited API calls".to_string(),
                "24/7 phone support".to_string(),
                "Custom integrations".to_string(),
                "Dedicated success manager".to_string(),
                "On-premise deployment".to_string(),
            ],
            limits: PlanLimits {
                api_calls_per_month: None,
                storage_gb: Some(1000),
                compute_hours_per_month: None,
                bandwidth_gb_per_month: None,
                max_databases: None,
                max_users: None,
                custom_limits: HashMap::new(),
            },
            trial_days: Some(60),
            is_active: true,
        });

        Ok(())
    }

    /// Create a new subscription
    pub async fn create_subscription(
        &self,
        tenant_id: Uuid,
        plan_id: &str,
        billing_interval: BillingInterval,
        payment_method_id: Option<String>,
        start_trial: bool,
    ) -> Result<Subscription> {
        let plans = self.plans.read().await;
        let plan = plans.get(plan_id)
            .ok_or_else(|| SaaSError::PlanNotFound(plan_id.to_string()))?;

        let now = Utc::now();
        let (period_start, period_end, trial_start, trial_end, status) = if start_trial && plan.trial_days.is_some() {
            let trial_days = plan.trial_days.unwrap();
            let trial_end = now + Duration::days(trial_days as i64);
            (now, trial_end, Some(now), Some(trial_end), SubscriptionStatus::Trial)
        } else {
            let period_end = match billing_interval {
                BillingInterval::Monthly => now + Duration::days(30),
                BillingInterval::Quarterly => now + Duration::days(90),
                BillingInterval::Yearly => now + Duration::days(365),
                BillingInterval::Custom(duration) => now + duration,
            };
            (now, period_end, None, None, SubscriptionStatus::Active)
        };

        let subscription = Subscription {
            id: Uuid::new_v4(),
            tenant_id,
            plan_id: plan_id.to_string(),
            status,
            billing_interval,
            current_period_start: period_start,
            current_period_end: period_end,
            trial_start,
            trial_end,
            cancelled_at: None,
            ended_at: None,
            payment_method_id,
            discount_percent: None,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        };

        // Store subscription
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription.id, subscription.clone());

        // Initialize usage tracking
        self.initialize_usage_tracking(&subscription).await?;

        // Schedule renewal task
        self.schedule_renewal_task(&subscription).await?;

        Ok(subscription)
    }

    /// Get subscription by ID
    pub async fn get_subscription(&self, subscription_id: Uuid) -> Option<Subscription> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.get(&subscription_id).cloned()
    }

    /// Get subscription by tenant ID
    pub async fn get_subscription_by_tenant(&self, tenant_id: Uuid) -> Option<Subscription> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.values()
            .find(|sub| sub.tenant_id == tenant_id)
            .cloned()
    }

    /// Update subscription
    pub async fn modify_subscription(
        &self,
        subscription_id: Uuid,
        modification: SubscriptionModification,
    ) -> Result<Subscription> {
        let mut subscriptions = self.subscriptions.write().await;
        let subscription = subscriptions.get_mut(&subscription_id)
            .ok_or_else(|| SaaSError::SubscriptionNotFound(subscription_id))?;

        let now = Utc::now();
        let effective_date = modification.effective_date.unwrap_or(now);

        // Handle plan change
        if let Some(new_plan_id) = modification.new_plan_id {
            let plans = self.plans.read().await;
            let _new_plan = plans.get(&new_plan_id)
                .ok_or_else(|| SaaSError::PlanNotFound(new_plan_id.clone()))?;

            // Calculate proration if needed
            if modification.proration_behavior == ProrationBehavior::CreateProrations {
                self.create_proration_invoice(subscription, &new_plan_id, effective_date).await?;
            }

            subscription.plan_id = new_plan_id;
        }

        // Handle billing interval change
        if let Some(new_interval) = modification.new_billing_interval {
            subscription.billing_interval = new_interval;
        }

        subscription.updated_at = now;
        let updated_subscription = subscription.clone();

        Ok(updated_subscription)
    }

    /// Cancel subscription
    pub async fn cancel_subscription(
        &self,
        subscription_id: Uuid,
        at_period_end: bool,
    ) -> Result<Subscription> {
        let mut subscriptions = self.subscriptions.write().await;
        let subscription = subscriptions.get_mut(&subscription_id)
            .ok_or_else(|| SaaSError::SubscriptionNotFound(subscription_id))?;

        let now = Utc::now();

        if at_period_end {
            subscription.status = SubscriptionStatus::Cancelled;
            subscription.cancelled_at = Some(now);
        } else {
            subscription.status = SubscriptionStatus::Expired;
            subscription.cancelled_at = Some(now);
            subscription.ended_at = Some(now);
            subscription.current_period_end = now;
        }

        subscription.updated_at = now;
        let updated_subscription = subscription.clone();

        Ok(updated_subscription)
    }

    /// Process subscription renewals
    pub async fn process_renewals(&self) -> Result<Vec<Uuid>> {
        let now = Utc::now();
        let mut renewed_subscriptions = Vec::new();

        let subscriptions = self.subscriptions.read().await;
        for subscription in subscriptions.values() {
            if subscription.current_period_end <= now && 
               matches!(subscription.status, SubscriptionStatus::Active | SubscriptionStatus::Trial) {
                
                match self.renew_subscription(subscription.clone()).await {
                    Ok(_) => renewed_subscriptions.push(subscription.id),
                    Err(e) => {
                        tracing::error!("Failed to renew subscription {}: {}", subscription.id, e);
                        // Mark as past due
                        let mut subs = self.subscriptions.write().await;
                        if let Some(sub) = subs.get_mut(&subscription.id) {
                            sub.status = SubscriptionStatus::PastDue;
                            sub.updated_at = now;
                        }
                    }
                }
            }
        }

        Ok(renewed_subscriptions)
    }

    /// Check if subscription has access to a feature
    pub async fn has_feature_access(&self, subscription_id: Uuid, feature: &str) -> Result<bool> {
        let subscription = self.get_subscription(subscription_id).await
            .ok_or_else(|| SaaSError::SubscriptionNotFound(subscription_id))?;

        if !matches!(subscription.status, SubscriptionStatus::Active | SubscriptionStatus::Trial) {
            return Ok(false);
        }

        let plans = self.plans.read().await;
        let plan = plans.get(&subscription.plan_id)
            .ok_or_else(|| SaaSError::PlanNotFound(subscription.plan_id))?;

        Ok(plan.features.contains(&feature.to_string()))
    }

    /// Check if usage is within limits
    pub async fn check_usage_limits(&self, subscription_id: Uuid) -> Result<HashMap<String, bool>> {
        let subscription = self.get_subscription(subscription_id).await
            .ok_or_else(|| SaaSError::SubscriptionNotFound(subscription_id))?;

        let plans = self.plans.read().await;
        let plan = plans.get(&subscription.plan_id)
            .ok_or_else(|| SaaSError::PlanNotFound(subscription.plan_id))?;

        let usage_tracking = self.usage_tracking.read().await;
        let usage = usage_tracking.get(&subscription_id)
            .ok_or_else(|| SaaSError::UsageNotFound(subscription_id))?;

        let mut results = HashMap::new();

        // Check API calls
        if let Some(limit) = plan.limits.api_calls_per_month {
            results.insert("api_calls".to_string(), usage.api_calls <= limit);
        }

        // Check storage
        if let Some(limit) = plan.limits.storage_gb {
            results.insert("storage".to_string(), usage.storage_used_gb <= limit as f64);
        }

        // Check compute hours
        if let Some(limit) = plan.limits.compute_hours_per_month {
            results.insert("compute".to_string(), usage.compute_hours <= limit as f64);
        }

        // Check bandwidth
        if let Some(limit) = plan.limits.bandwidth_gb_per_month {
            results.insert("bandwidth".to_string(), usage.bandwidth_used_gb <= limit as f64);
        }

        Ok(results)
    }

    /// Get current usage for subscription
    pub async fn get_usage(&self, subscription_id: Uuid) -> Option<SubscriptionUsage> {
        let usage_tracking = self.usage_tracking.read().await;
        usage_tracking.get(&subscription_id).cloned()
    }

    /// Record usage event
    pub async fn record_usage(
        &self,
        subscription_id: Uuid,
        usage_type: &str,
        amount: f64,
    ) -> Result<()> {
        let mut usage_tracking = self.usage_tracking.write().await;
        let usage = usage_tracking.get_mut(&subscription_id)
            .ok_or_else(|| SaaSError::UsageNotFound(subscription_id))?;

        match usage_type {
            "api_calls" => usage.api_calls += amount as u64,
            "storage_gb" => usage.storage_used_gb = amount,
            "compute_hours" => usage.compute_hours += amount,
            "bandwidth_gb" => usage.bandwidth_used_gb += amount,
            custom => {
                *usage.custom_usage.entry(custom.to_string()).or_insert(0.0) += amount;
            }
        }

        Ok(())
    }

    // Private helper methods

    async fn initialize_usage_tracking(&self, subscription: &Subscription) -> Result<()> {
        let usage = SubscriptionUsage {
            subscription_id: subscription.id,
            period_start: subscription.current_period_start,
            period_end: subscription.current_period_end,
            api_calls: 0,
            storage_used_gb: 0.0,
            compute_hours: 0.0,
            bandwidth_used_gb: 0.0,
            custom_usage: HashMap::new(),
            overage_charges: 0,
        };

        let mut usage_tracking = self.usage_tracking.write().await;
        usage_tracking.insert(subscription.id, usage);
        Ok(())
    }

    async fn schedule_renewal_task(&self, _subscription: &Subscription) -> Result<()> {
        // TODO: Implement actual task scheduling
        // This would integrate with a job queue system like Redis/Sidekiq
        Ok(())
    }

    async fn renew_subscription(&self, subscription: Subscription) -> Result<()> {
        // Calculate new period dates
        let new_period_start = subscription.current_period_end;
        let new_period_end = match subscription.billing_interval {
            BillingInterval::Monthly => new_period_start + Duration::days(30),
            BillingInterval::Quarterly => new_period_start + Duration::days(90),
            BillingInterval::Yearly => new_period_start + Duration::days(365),
            BillingInterval::Custom(duration) => new_period_start + duration,
        };

        // Calculate amount to charge
        let plans = self.plans.read().await;
        let plan = plans.get(&subscription.plan_id)
            .ok_or_else(|| SaaSError::PlanNotFound(subscription.plan_id.clone()))?;

        let amount = match subscription.billing_interval {
            BillingInterval::Monthly => plan.price_monthly,
            BillingInterval::Yearly => plan.price_yearly.unwrap_or(plan.price_monthly * 12),
            BillingInterval::Quarterly => plan.price_monthly * 3,
            BillingInterval::Custom(_) => plan.price_monthly, // Fallback
        };

        // Apply discount if any
        let final_amount = if let Some(discount) = subscription.discount_percent {
            (amount as f32 * (1.0 - discount / 100.0)) as u64
        } else {
            amount
        };

        // Process payment
        if let Some(payment_method_id) = &subscription.payment_method_id {
            self.billing_engine.process_payment(
                subscription.tenant_id,
                final_amount,
                payment_method_id,
                &format!("Subscription renewal for plan {}", plan.name),
            ).await?;
        }

        // Update subscription
        let mut subscriptions = self.subscriptions.write().await;
        if let Some(sub) = subscriptions.get_mut(&subscription.id) {
            sub.current_period_start = new_period_start;
            sub.current_period_end = new_period_end;
            sub.updated_at = Utc::now();
            
            // Move from trial to active if trial ended
            if sub.status == SubscriptionStatus::Trial {
                sub.status = SubscriptionStatus::Active;
            }
        }

        // Reset usage tracking for new period
        self.reset_usage_tracking(subscription.id, new_period_start, new_period_end).await?;

        Ok(())
    }

    async fn create_proration_invoice(
        &self,
        _subscription: &Subscription,
        _new_plan_id: &str,
        _effective_date: DateTime<Utc>,
    ) -> Result<Invoice> {
        // TODO: Implement proration calculation and invoice creation
        // This would calculate the difference between old and new plans
        // and create appropriate credits/charges
        todo!("Proration implementation")
    }

    async fn reset_usage_tracking(
        &self,
        subscription_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<()> {
        let mut usage_tracking = self.usage_tracking.write().await;
        if let Some(usage) = usage_tracking.get_mut(&subscription_id) {
            usage.period_start = period_start;
            usage.period_end = period_end;
            usage.api_calls = 0;
            usage.storage_used_gb = 0.0;
            usage.compute_hours = 0.0;
            usage.bandwidth_used_gb = 0.0;
            usage.custom_usage.clear();
            usage.overage_charges = 0;
        }
        Ok(())
    }
}

/// Subscription lifecycle events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionEvent {
    Created { subscription_id: Uuid, plan_id: String },
    Renewed { subscription_id: Uuid, period_end: DateTime<Utc> },
    Modified { subscription_id: Uuid, changes: Vec<String> },
    Cancelled { subscription_id: Uuid, reason: Option<String> },
    Expired { subscription_id: Uuid },
    PaymentFailed { subscription_id: Uuid, error: String },
    TrialEnded { subscription_id: Uuid },
}
