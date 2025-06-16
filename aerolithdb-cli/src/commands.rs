//! # CLI Command Handlers
//!
//! This module re-exports all command handler functions to provide a unified
//! interface for the main CLI application. It organizes command handlers by
//! functional area and provides a clean API for command execution.

// Re-export document operation handlers
pub use crate::document::{execute_put, execute_get, execute_delete};

// Re-export query operation handlers  
pub use crate::query::{execute_query, execute_list};

// Re-export analytics handlers
pub use crate::analytics::{execute_analytics, execute_optimize, execute_stats};

// Re-export configuration management handlers
pub use crate::config::{execute_config_validate, execute_config_generate, execute_config_show};

// Re-export batch operation handlers
pub use crate::batch::{execute_batch_put, execute_batch_delete, execute_batch_import, execute_batch_export};

// Re-export argument structures for command parsing
pub use crate::args::*;
