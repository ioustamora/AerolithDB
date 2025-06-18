//! AerolithDB Integration Layer
//! 
//! Provides integration components that bridge core AerolithDB functionality
//! with SaaS features, enabling multi-tenant operations, billing integration,
//! and comprehensive usage tracking.

pub mod saas;

pub use saas::*;
