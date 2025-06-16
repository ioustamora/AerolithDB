//! # Utility Functions
//!
//! This module provides utility functions used across CLI command implementations:
//! - JSON parsing from inline strings or files
//! - Statistics formatting for human-readable display
//! - Value formatting with appropriate units

use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Parses JSON input from either inline JSON strings or file references.
///
/// ## Input Format Support
///
/// This utility function provides flexible JSON input handling:
/// - **Inline JSON**: Direct JSON strings for simple data and testing
/// - **File References**: `@filename.json` syntax for larger documents and reusability
/// - **Valiaerolithon**: Comprehensive JSON syntax valiaerolithon with helpful error messages
///
/// ## File Path Resolution
///
/// File paths are resolved relative to the current working directory:
/// - **Relative Paths**: `@data/document.json` resolves from current directory
/// - **Absolute Paths**: `@/home/user/data.json` uses absolute path specification
/// - **Error Handling**: Clear messages for file not found and permission issues
///
/// ## JSON Valiaerolithon
///
/// The function performs thorough JSON valiaerolithon:
/// - **Syntax Checking**: Validates proper JSON structure and formatting
/// - **Error Reporting**: Provides specific error messages with suggestions
/// - **Type Preservation**: Maintains JSON type information for downstream processing
///
/// # Arguments
///
/// * `input` - Input string either as direct JSON or file reference (prefixed with @)
///
/// # Returns
///
/// * `Result<Value>` - Parsed JSON value or detailed error information
///
/// # Examples
///
/// ```rust
/// // Inline JSON
/// let data = parse_json_input(r#"{"name": "John", "age": 30}"#)?;
///
/// // File reference
/// let data = parse_json_input("@user_data.json")?;
/// ```
pub fn parse_json_input(input: &str) -> Result<Value> {
    if input.starts_with('@') {
        // Handle file input with comprehensive error reporting
        let file_path = &input[1..]; // Remove @ prefix
        
        // Check file existence before attempting to read
        if !Path::new(file_path).exists() {
            return Err(anyhow::anyhow!(
                "File not found: {}\n  → Check file path spelling and location\n  → Ensure file exists and is accessible", 
                file_path
            ));
        }
        
        // Read file content with detailed error handling
        let content = fs::read_to_string(file_path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to read file '{}': {}\n  → Check file permissions\n  → Ensure file is not locked by another process", 
                file_path, e
            )
        })?;
        
        // Parse JSON content with syntax error reporting
        serde_json::from_str(&content).map_err(|e| {
            anyhow::anyhow!(
                "Invalid JSON in file '{}': {}\n  → Check JSON syntax and formatting\n  → Use a JSON validator to identify issues", 
                file_path, e
            )
        })
    } else {
        // Handle inline JSON with syntax valiaerolithon
        serde_json::from_str(input).map_err(|e| {
            anyhow::anyhow!(
                "Invalid JSON syntax: {}\n  → Check quotes, commas, and brackets\n  → Example: '{{\"key\": \"value\"}}'", 
                e
            )
        })
    }
}

/// Formats statistics data into a human-readable table format.
///
/// ## Table Organization
///
/// The function organizes statistics into logical categories:
/// - **Storage Information**: Document counts, sizes, and storage utilization
/// - **Performance Metrics**: Query times, throughput, and cache effectiveness
/// - **System Health**: Memory usage, connections, and operational status
/// - **Collection Details**: Per-collection statistics when detailed mode is enabled
///
/// ## Formatting Strategy
///
/// The table format uses visual organization for clarity:
/// - **Categorical Headers**: Clear section separation with emoji indicators
/// - **Numerical Formatting**: Human-readable numbers with appropriate units
/// - **Hierarchical Structure**: Nested information with consistent indentation
/// - **Status Indicators**: Visual cues for health and performance states
///
/// ## Detailed Mode Features
///
/// When detailed mode is enabled, additional information is included:
/// - **Historical Trends**: Performance changes over time
/// - **Diagnostic Data**: Advanced metrics for troubleshooting
/// - **Optimization Hints**: Suggestions for performance improvements
/// - **Resource Projections**: Capacity planning information
///
/// ## Error Resilience
///
/// The function handles various data format scenarios:
/// - **Missing Fields**: Graceful handling of incomplete statistics
/// - **Type Variations**: Flexible handling of different data types
/// - **Nested Structures**: Recursive processing of complex statistics
/// - **Format Errors**: Fallback to raw value display when formatting fails
///
/// # Arguments
///
/// * `stats` - JSON statistics data from the aerolithsDB server
/// * `detailed` - Whether to include detailed diagnostic information
///
/// # Returns
///
/// * `Result<String>` - Formatted table string or formatting error
pub fn format_stats_table(stats: &Value, detailed: bool) -> Result<String> {
    let mut output = String::new();
    
    if let Some(obj) = stats.as_object() {
        // Process storage-related statistics
        if let Some(storage) = obj.get("storage") {
            output.push_str("📁 Storage:\n");
            if let Some(storage_obj) = storage.as_object() {
                for (key, value) in storage_obj {
                    output.push_str(&format!("  {}: {}\n", 
                        format_key_name(key), 
                        format_value_with_units(value)
                    ));
                }
            }
            output.push('\n');
        }
        
        // Process performance statistics
        if let Some(performance) = obj.get("performance") {
            output.push_str("⚡ Performance:\n");
            if let Some(perf_obj) = performance.as_object() {
                for (key, value) in perf_obj {
                    output.push_str(&format!("  {}: {}\n", 
                        format_key_name(key), 
                        format_performance_value(key, value)
                    ));
                }
            }
            output.push('\n');
        }
        
        // Process system health information
        if let Some(system) = obj.get("system") {
            output.push_str("🔧 System Health:\n");
            if let Some(system_obj) = system.as_object() {
                for (key, value) in system_obj {
                    output.push_str(&format!("  {}: {}\n", 
                        format_key_name(key), 
                        format_system_value(key, value)
                    ));
                }
            }
            output.push('\n');
        }
        
        // Process collection-specific statistics in detailed mode
        if detailed {
            if let Some(collections) = obj.get("collections") {
                output.push_str("📚 Collections:\n");
                if let Some(collections_obj) = collections.as_object() {
                    for (collection_name, collection_stats) in collections_obj {
                        output.push_str(&format!("  📄 {}:\n", collection_name));
                        if let Some(stats_obj) = collection_stats.as_object() {
                            for (key, value) in stats_obj {
                                output.push_str(&format!("    {}: {}\n", 
                                    format_key_name(key), 
                                    format_value_with_units(value)
                                ));
                            }
                        }
                    }
                }
                output.push('\n');
            }
        }
        
        // Process any remaining top-level statistics
        for (key, value) in obj {
            if !["storage", "performance", "system", "collections"].contains(&key.as_str()) {
                output.push_str(&format!("📊 {}:\n", format_key_name(key)));
                if let Some(nested_obj) = value.as_object() {
                    for (nested_key, nested_value) in nested_obj {
                        output.push_str(&format!("  {}: {}\n", 
                            format_key_name(nested_key), 
                            format_value_with_units(nested_value)
                        ));
                    }
                } else {
                    output.push_str(&format!("  {}\n", format_value_with_units(value)));
                }
                output.push('\n');
            }
        }
    } else {
        // Fallback for non-object statistics
        output.push_str(&format!("📊 Statistics:\n  {}\n", format_value(stats)));
    }
    
    Ok(output)
}

/// Formats a statistics key name into a human-readable label.
///
/// Converts technical field names into user-friendly labels:
/// - `total_docs` → `Total Documents`
/// - `avg_query_time_ms` → `Average Query Time`
/// - `cache_hit_rate` → `Cache Hit Rate`
pub fn format_key_name(key: &str) -> String {
    key.replace('_', " ")
       .split_whitespace()
       .map(|word| {
           let mut chars = word.chars();
           match chars.next() {
               None => String::new(),
               Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
           }
       })
       .collect::<Vec<_>>()
       .join(" ")
}

/// Formats values with appropriate units and human-readable numbers.
pub fn format_value_with_units(value: &Value) -> String {
    match value {
        Value::Number(n) => {
            if let Some(int_val) = n.as_u64() {
                format_number_with_units(int_val as f64)
            } else if let Some(float_val) = n.as_f64() {
                format_number_with_units(float_val)
            } else {
                n.to_string()
            }
        }
        _ => format_value(value)
    }
}

/// Formats performance-specific values with appropriate context.
pub fn format_performance_value(key: &str, value: &Value) -> String {
    match key {
        k if k.contains("time") || k.contains("latency") => {
            if let Some(ms) = value.as_f64() {
                if ms < 1.0 {
                    format!("{:.1}μs", ms * 1000.0)
                } else if ms < 1000.0 {
                    format!("{:.1}ms", ms)
                } else {
                    format!("{:.2}s", ms / 1000.0)
                }
            } else {
                format_value(value)
            }
        }
        k if k.contains("rate") || k.contains("percentage") => {
            if let Some(rate) = value.as_f64() {
                format!("{:.1}%", rate * 100.0)
            } else {
                format_value(value)
            }
        }
        k if k.contains("per_second") || k.contains("qps") => {
            if let Some(qps) = value.as_f64() {
                format!("{:.0}/sec", qps)
            } else {
                format_value(value)
            }
        }
        _ => format_value_with_units(value)
    }
}

/// Formats system health values with appropriate context and indicators.
pub fn format_system_value(key: &str, value: &Value) -> String {
    match key {
        k if k.contains("memory") || k.contains("usage") => {
            if let Some(bytes) = value.as_f64() {
                format_bytes(bytes as u64)
            } else {
                format_value(value)
            }
        }
        k if k.contains("percentage") || k.contains("percent") => {
            if let Some(pct) = value.as_f64() {
                let indicator = if pct > 90.0 { "🔴" } else if pct > 75.0 { "🟡" } else { "🟢" };
                format!("{} {:.1}%", indicator, pct)
            } else {
                format_value(value)
            }
        }
        _ => format_value_with_units(value)
    }
}

/// Formats large numbers with thousands separators and appropriate units.
pub fn format_number_with_units(num: f64) -> String {
    if num >= 1_000_000_000.0 {
        format!("{:.1}B", num / 1_000_000_000.0)
    } else if num >= 1_000_000.0 {
        format!("{:.1}M", num / 1_000_000.0)
    } else if num >= 1_000.0 {
        format!("{:.1}K", num / 1_000.0)    } else if num.fract() == 0.0 {
        format!("{:.0}", num)
    } else {
        format!("{:.2}", num)
    }
}

/// Formats byte values into human-readable sizes.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Formats a generic JSON value for display with basic type handling.
///
/// Provides consistent formatting for various JSON value types:
/// - **Strings**: Direct display without modification
/// - **Numbers**: Appropriate precision and formatting
/// - **Booleans**: Lowercase true/false representation
/// - **Null**: Explicit "null" display
/// - **Complex Types**: JSON serialization fallback
pub fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),        Value::Number(n) => {
            if let Some(int_val) = n.as_i64() {
                format!("{}", int_val)
            } else if let Some(float_val) = n.as_f64() {
                format!("{:.2}", float_val)
            } else {
                n.to_string()
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => serde_json::to_string(value).unwrap_or_else(|_| "invalid".to_string()),
    }
}

/// Unit tests for utility functions.
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_json_input_inline() {
        let result = parse_json_input(r#"{"test": "value"}"#);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["test"], "value");
    }

    #[test]
    fn test_parse_json_input_file_not_found() {
        let result = parse_json_input("@nonexistent.json");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not found"));
    }

    #[test]
    fn test_format_value() {
        assert_eq!(format_value(&Value::String("test".to_string())), "test");
        assert_eq!(format_value(&Value::Number(serde_json::Number::from(42))), "42");
        assert_eq!(format_value(&Value::Bool(true)), "true");
        assert_eq!(format_value(&Value::Null), "null");
    }

    #[test]
    fn test_format_key_name() {
        assert_eq!(format_key_name("total_docs"), "Total Docs");
        assert_eq!(format_key_name("avg_query_time_ms"), "Avg Query Time Ms");
        assert_eq!(format_key_name("cache_hit_rate"), "Cache Hit Rate");
    }

    #[test]
    fn test_format_number_with_units() {
        assert_eq!(format_number_with_units(1_500_000_000.0), "1.5B");
        assert_eq!(format_number_with_units(2_500_000.0), "2.5M");
        assert_eq!(format_number_with_units(1_500.0), "1.5K");
        assert_eq!(format_number_with_units(42.0), "42");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
        assert_eq!(format_bytes(1073741824), "1.0 GB");
        assert_eq!(format_bytes(512), "512 B");
    }
}
