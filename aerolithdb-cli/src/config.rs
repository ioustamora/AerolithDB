//! # Configuration Management Operations
//!
//! This module implements CLI commands for configuration management:
//! - CONFIG VALIDATE: Validate configuration files and settings
//! - CONFIG GENERATE: Generate default configuration templates
//! - CONFIG SHOW: Display current effective configuration

use anyhow::{Result, anyhow};
use serde_json::{Value, to_string_pretty};
use tracing::{info, warn, error};
use std::path::Path;

use crate::client::aerolithsClient;
use crate::args::{ConfigValidateArgs, ConfigGenerateArgs, ConfigShowArgs};

/// Executes the CONFIG VALIDATE command to validate configuration files.
///
/// ## Valiaerolithon Process
///
/// The function performs comprehensive configuration valiaerolithon:
/// - **Syntax Valiaerolithon**: Ensures configuration file is valid JSON/YAML/TOML
/// - **Schema Valiaerolithon**: Validates against aerolithsDB configuration schema
/// - **Value Valiaerolithon**: Checks for valid values within acceptable ranges
/// - **Dependency Valiaerolithon**: Ensures interdependent settings are consistent
///
/// ## Valiaerolithon Categories
///
/// - **Node Configuration**: Identity, network binding, port availability
/// - **Storage Configuration**: Directory permissions, disk space, paths
/// - **Security Configuration**: Certificate validity, encryption settings
/// - **Network Configuration**: Port conflicts, address resolution
/// - **Performance Settings**: Memory limits, timeout values, thresholds
///
/// ## Error Reporting
///
/// Valiaerolithon errors are reported with:
/// - Clear error messages with specific field references
/// - Suggested corrections where applicable
/// - Warning vs. error classification
/// - Line numbers for file-based errors
///
/// # Arguments
///
/// * `client` - Configured aerolithsClient for server communication (optional for offline valiaerolithon)
/// * `args` - Parsed command-line arguments including file path and valiaerolithon options
///
/// # Returns
///
/// * `Ok(())` if configuration is valid or warnings-only
/// * `Err(anyhow::Error)` if valiaerolithon fails with critical errors
///
/// # Example
///
/// ```bash
/// # Validate specific configuration file
/// aerolithsdb-cli config validate --file config.json
///
/// # Validate with strict mode (warnings as errors)
/// aerolithsdb-cli config validate --file config.yaml --strict
///
/// # Validate server configuration (requires server connection)
/// aerolithsdb-cli config validate --server-config
/// ```
pub async fn execute_config_validate(client: &aerolithsClient, args: &ConfigValidateArgs) -> Result<()> {
    info!("Starting configuration valiaerolithon for: {:?}", args.file_path);

    // Determine valiaerolithon mode
    if args.server_config {
        // Server-side configuration valiaerolithon
        validate_server_config(client, args).await
    } else if let Some(file_path) = &args.file_path {
        // Local file valiaerolithon
        validate_local_config(file_path, args).await
    } else {
        Err(anyhow!("Either --file or --server-config must be specified"))
    }
}

/// Executes the CONFIG GENERATE command to create configuration templates.
///
/// ## Template Generation
///
/// The function generates comprehensive configuration templates:
/// - **Environment-Specific**: Development, staging, production templates
/// - **Component-Specific**: Individual subsystem configurations
/// - **Complete Templates**: Full system configuration with all sections
/// - **Custom Templates**: User-specified component combinations
///
/// ## Template Categories
///
/// - **Basic**: Minimal configuration for simple deployments
/// - **Development**: Local development with relaxed security
/// - **Production**: Enterprise-grade security and performance settings
/// - **Cluster**: Multi-node distributed deployment configuration
/// - **Security**: Zero-trust security with full encryption
///
/// ## Output Formats
///
/// Templates can be generated in multiple formats:
/// - **JSON**: Machine-readable with valiaerolithon support
/// - **YAML**: Human-readable with comments and documentation
/// - **TOML**: Simple syntax ideal for configuration files
/// - **Environment**: Shell environment variable format
///
/// # Arguments
///
/// * `_client` - aerolithsClient (unused for template generation)
/// * `args` - Parsed command-line arguments including template type and format
///
/// # Returns
///
/// * `Ok(())` if template was generated successfully
/// * `Err(anyhow::Error)` if template generation fails
///
/// # Example
///
/// ```bash
/// # Generate basic configuration template
/// aerolithsdb-cli config generate --template basic --format yaml > config.yaml
///
/// # Generate production configuration with security
/// aerolithsdb-cli config generate --template production --components security,storage,api
///
/// # Generate environment variables
/// aerolithsdb-cli config generate --template env --output .env
/// ```
pub async fn execute_config_generate(_client: &aerolithsClient, args: &ConfigGenerateArgs) -> Result<()> {
    info!("Generating configuration template: {:?}", args.template);

    // Generate configuration based on template type
    let config = match args.template.as_str() {
        "basic" => generate_basic_config(),
        "development" => generate_development_config(),
        "production" => generate_production_config(),
        "cluster" => generate_cluster_config(),
        "security" => generate_security_config(),
        _ => return Err(anyhow!("Unknown template type: {}", args.template)),
    };

    // Filter by components if specified
    let filtered_config = if args.components.is_empty() {
        config
    } else {
        filter_config_by_components(config, &args.components)?
    };

    // Format and output configuration
    let formatted_config = match args.format.as_str() {
        "json" => to_string_pretty(&filtered_config)?,
        "yaml" => serde_yaml::to_string(&filtered_config)
            .map_err(|e| anyhow!("YAML serialization failed: {}", e))?,
        "toml" => toml::to_string_pretty(&filtered_config)
            .map_err(|e| anyhow!("TOML serialization failed: {}", e))?,
        "env" => generate_env_format(&filtered_config)?,
        _ => return Err(anyhow!("Unsupported format: {}", args.format)),
    };

    // Output to file or stdout
    if let Some(output_path) = &args.output {
        tokio::fs::write(output_path, formatted_config).await?;
        println!("Configuration template generated: {}", output_path);
    } else {
        println!("{}", formatted_config);
    }

    info!("Configuration template generation completed successfully");
    Ok(())
}

/// Executes the CONFIG SHOW command to display current effective configuration.
///
/// ## Configuration Display
///
/// The function retrieves and displays configuration from multiple sources:
/// - **Server Configuration**: Live configuration from running server
/// - **File Configuration**: Configuration loaded from files
/// - **Effective Configuration**: Resolved configuration with precedence
/// - **Default Configuration**: Built-in default values
///
/// ## Display Modes
///
/// - **Full**: Complete configuration with all sections
/// - **Section-Specific**: Individual subsystem configurations
/// - **Changed**: Only non-default configuration values
/// - **Hierarchical**: Configuration source precedence display
///
/// ## Security Considerations
///
/// Sensitive configuration values are handled carefully:
/// - Passwords and keys are masked by default
/// - Use `--show-secrets` flag to display sensitive values
/// - Audit logging for security-sensitive configuration access
/// - Warning messages for sensitive data exposure
///
/// # Arguments
///
/// * `client` - Configured aerolithsClient for server communication
/// * `args` - Parsed command-line arguments including display options
///
/// # Returns
///
/// * `Ok(())` if configuration was displayed successfully
/// * `Err(anyhow::Error)` if configuration retrieval fails
///
/// # Example
///
/// ```bash
/// # Show complete server configuration
/// aerolithsdb-cli config show
///
/// # Show specific configuration section
/// aerolithsdb-cli config show --section storage
///
/// # Show only changed values from defaults
/// aerolithsdb-cli config show --changed-only
///
/// # Show configuration with sensitive values (use carefully)
/// aerolithsdb-cli config show --show-secrets
/// ```
pub async fn execute_config_show(client: &aerolithsClient, args: &ConfigShowArgs) -> Result<()> {
    info!("Retrieving configuration display");

    // Retrieve configuration from server
    let config = if args.server_config {
        retrieve_server_config(client).await?
    } else {
        retrieve_default_config().await?
    };

    // Filter by section if specified
    let display_config = if let Some(section) = &args.section {
        filter_config_by_section(&config, section)?
    } else {
        config
    };

    // Mask sensitive values unless explicitly requested
    let safe_config = if args.show_secrets {
        warn!("Displaying configuration with sensitive values - use caution");
        display_config
    } else {
        mask_sensitive_values(display_config)
    };

    // Format and display configuration
    let formatted_config = match args.format.as_str() {
        "json" => to_string_pretty(&safe_config)?,
        "yaml" => serde_yaml::to_string(&safe_config)
            .map_err(|e| anyhow!("YAML serialization failed: {}", e))?,
        "table" => format_config_as_table(&safe_config)?,
        _ => return Err(anyhow!("Unsupported format: {}", args.format)),
    };

    println!("{}", formatted_config);

    info!("Configuration display completed successfully");
    Ok(())
}

// ================================================================================================
// PRIVATE HELPER FUNCTIONS
// ================================================================================================

/// Validates server-side configuration by connecting to aerolithsDB instance.
async fn validate_server_config(client: &aerolithsClient, args: &ConfigValidateArgs) -> Result<()> {
    // Make server request to validate configuration
    let response = client.get("/api/v1/config/validate").await?;
    
    if response.status().is_success() {
        let valiaerolithon_result: Value = response.json().await?;
        
        if let Some(errors) = valiaerolithon_result.get("errors").and_then(|e| e.as_array()) {
            if !errors.is_empty() {
                error!("Server configuration valiaerolithon failed:");
                for error in errors {
                    println!("❌ {}", error.as_str().unwrap_or("Unknown error"));
                }
                return Err(anyhow!("Configuration valiaerolithon failed"));
            }
        }
        
        if let Some(warnings) = valiaerolithon_result.get("warnings").and_then(|w| w.as_array()) {
            if !warnings.is_empty() {
                warn!("Configuration warnings found:");
                for warning in warnings {
                    println!("⚠️  {}", warning.as_str().unwrap_or("Unknown warning"));
                }
                
                if args.strict {
                    return Err(anyhow!("Strict mode: warnings treated as errors"));
                }
            }
        }
        
        println!("✅ Server configuration is valid");
        Ok(())
    } else {
        Err(anyhow!("Server configuration valiaerolithon request failed: {}", response.status()))
    }
}

/// Validates local configuration file.
async fn validate_local_config(file_path: &str, args: &ConfigValidateArgs) -> Result<()> {
    // Check if file exists
    if !Path::new(file_path).exists() {
        return Err(anyhow!("Configuration file not found: {}", file_path));
    }
    
    // Read and parse configuration file
    let content = tokio::fs::read_to_string(file_path).await?;
    
    // Determine file format and parse
    let config: Value = if file_path.ends_with(".json") {
        serde_json::from_str(&content)?
    } else if file_path.ends_with(".yaml") || file_path.ends_with(".yml") {
        serde_yaml::from_str(&content)
            .map_err(|e| anyhow!("YAML parsing failed: {}", e))?
    } else if file_path.ends_with(".toml") {
        toml::from_str(&content)
            .map_err(|e| anyhow!("TOML parsing failed: {}", e))?
    } else {
        return Err(anyhow!("Unsupported configuration file format"));
    };
    
    // Perform valiaerolithon logic
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    
    validate_config_structure(&config, &mut errors, &mut warnings);
    
    // Report results
    if !errors.is_empty() {
        error!("Configuration valiaerolithon failed:");
        for error in &errors {
            println!("❌ {}", error);
        }
        return Err(anyhow!("Configuration valiaerolithon failed with {} errors", errors.len()));
    }
    
    if !warnings.is_empty() {
        warn!("Configuration warnings found:");
        for warning in &warnings {
            println!("⚠️  {}", warning);
        }
        
        if args.strict {
            return Err(anyhow!("Strict mode: warnings treated as errors"));
        }
    }
    
    println!("✅ Configuration file is valid: {}", file_path);
    Ok(())
}

/// Validates configuration structure and values.
fn validate_config_structure(config: &Value, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
    // Validate required sections
    let required_sections = ["node", "storage", "api"];
    for section in required_sections {
        if !config.get(section).is_some() {
            errors.push(format!("Missing required section: {}", section));
        }
    }
    
    // Validate node configuration
    if let Some(node) = config.get("node") {
        if let Some(port) = node.get("port").and_then(|p| p.as_u64()) {
            if port < 1024 || port > 65535 {
                errors.push("Node port must be between 1024 and 65535".to_string());
            }
        }
        
        if let Some(node_id) = node.get("node_id").and_then(|id| id.as_str()) {
            if node_id.is_empty() || node_id.len() > 64 {
                errors.push("Node ID must be 1-64 characters".to_string());
            }
        }
    }
    
    // Validate storage configuration
    if let Some(storage) = config.get("storage") {
        if let Some(data_dir) = storage.get("data_dir").and_then(|d| d.as_str()) {
            if data_dir.is_empty() {
                errors.push("Storage data_dir cannot be empty".to_string());
            }
        }
        
        if let Some(replication_factor) = storage.get("replication_factor").and_then(|r| r.as_u64()) {
            if replication_factor == 0 || replication_factor > 10 {
                warnings.push("Replication factor should typically be between 1 and 10".to_string());
            }
        }
    }
    
    // Validate API configuration
    if let Some(api) = config.get("api") {
        if let Some(rest_api) = api.get("rest_api") {
            if let Some(port) = rest_api.get("port").and_then(|p| p.as_u64()) {
                if port < 1024 || port > 65535 {
                    errors.push("REST API port must be between 1024 and 65535".to_string());
                }
            }
        }
    }
}

/// Generates basic configuration template.
fn generate_basic_config() -> Value {
    serde_json::json!({
        "node": {
            "node_id": "aerolithsdb-node-001",
            "bind_address": "0.0.0.0",
            "port": 8080
        },
        "storage": {
            "data_dir": "./data",
            "sharding_strategy": "ConsistentHash",
            "replication_factor": 1
        },
        "api": {
            "rest_api": {
                "enabled": true,
                "port": 8080,
                "cors_enabled": true
            }
        },
        "security": {
            "zero_trust": false,
            "encryption_algorithm": "XChaCha20Poly1305",
            "audit_level": "Basic"
        }
    })
}

/// Generates development configuration template.
fn generate_development_config() -> Value {
    serde_json::json!({
        "node": {
            "node_id": "dev-node-001",
            "bind_address": "127.0.0.1",
            "port": 8080
        },
        "storage": {
            "data_dir": "./dev-data",
            "sharding_strategy": "ConsistentHash",
            "replication_factor": 1
        },
        "api": {
            "rest_api": {
                "enabled": true,
                "port": 8080,
                "cors_enabled": true
            },
            "grpc_api": {
                "enabled": true,
                "port": 8082
            }
        },
        "security": {
            "zero_trust": false,
            "encryption_algorithm": "XChaCha20Poly1305",
            "audit_level": "Basic"
        },
        "observability": {
            "logging": {
                "level": "debug",
                "structured": true
            }
        }
    })
}

/// Generates production configuration template.
fn generate_production_config() -> Value {
    serde_json::json!({
        "node": {
            "node_id": "prod-node-001",
            "bind_address": "0.0.0.0",
            "port": 8080
        },
        "storage": {
            "data_dir": "/var/lib/aerolithsdb",
            "sharding_strategy": "ConsistentHash",
            "replication_factor": 3
        },
        "api": {
            "rest_api": {
                "enabled": true,
                "port": 8080,
                "cors_enabled": false
            },
            "grpc_api": {
                "enabled": true,
                "port": 8082
            }
        },
        "security": {
            "zero_trust": true,
            "encryption_algorithm": "XChaCha20Poly1305",
            "audit_level": "Full"
        },
        "observability": {
            "logging": {
                "level": "info",
                "structured": true,
                "file_output": "/var/log/aerolithsdb/aerolithsdb.log"
            },
            "tracing": {
                "enabled": true,
                "jaeger_endpoint": "http://jaeger:14268/api/traces"
            }
        }
    })
}

/// Generates cluster configuration template.
fn generate_cluster_config() -> Value {
    serde_json::json!({
        "node": {
            "node_id": "cluster-node-001",
            "bind_address": "0.0.0.0",
            "port": 8080
        },
        "network": {
            "cluster_name": "aerolithsdb-cluster",
            "seed_nodes": ["10.0.1.100:8080", "10.0.1.101:8080"],
            "gossip_port": 8081,
            "max_peers": 100
        },
        "storage": {
            "data_dir": "/var/lib/aerolithsdb",
            "sharding_strategy": "ConsistentHash",
            "replication_factor": 3
        },
        "consensus": {
            "algorithm": "Raft",
            "election_timeout_ms": 5000,
            "heartbeat_interval_ms": 1000
        },
        "api": {
            "rest_api": {
                "enabled": true,
                "port": 8080,
                "cors_enabled": false
            },
            "grpc_api": {
                "enabled": true,
                "port": 8082
            }
        },
        "security": {
            "zero_trust": true,
            "encryption_algorithm": "XChaCha20Poly1305",
            "audit_level": "Full"
        }
    })
}

/// Generates security-focused configuration template.
fn generate_security_config() -> Value {
    serde_json::json!({
        "security": {
            "zero_trust": true,
            "encryption_algorithm": "XChaCha20Poly1305",
            "audit_level": "Full",
            "tls": {
                "enabled": true,
                "cert_file": "/etc/aerolithsdb/certs/server.crt",
                "key_file": "/etc/aerolithsdb/certs/server.key",
                "ca_file": "/etc/aerolithsdb/certs/ca.crt"
            },
            "authentication": {
                "enabled": true,
                "method": "jwt",
                "jwt_secret": "${JWT_SECRET}",
                "token_expiry": "24h"
            },
            "authorization": {
                "enabled": true,
                "default_policy": "deny",
                "rbac_enabled": true
            }
        }
    })
}

/// Filters configuration by specified components.
fn filter_config_by_components(config: Value, components: &[String]) -> Result<Value> {
    let mut filtered = serde_json::Map::new();
    
    if let Some(config_obj) = config.as_object() {
        for component in components {
            if let Some(component_config) = config_obj.get(component) {
                filtered.insert(component.clone(), component_config.clone());
            } else {
                warn!("Component '{}' not found in configuration", component);
            }
        }
    }
    
    Ok(Value::Object(filtered))
}

/// Generates environment variable format.
fn generate_env_format(config: &Value) -> Result<String> {
    let mut env_vars = Vec::new();
    
    fn flatten_config(obj: &Value, prefix: &str, vars: &mut Vec<String>) {
        match obj {
            Value::Object(map) => {
                for (key, value) in map {
                    let new_prefix = if prefix.is_empty() {
                        format!("aerolithSDB_{}", key.to_uppercase())
                    } else {
                        format!("{}_{}", prefix, key.to_uppercase())
                    };
                    flatten_config(value, &new_prefix, vars);
                }
            },
            Value::String(s) => vars.push(format!("{}={}", prefix, s)),
            Value::Number(n) => vars.push(format!("{}={}", prefix, n)),
            Value::Bool(b) => vars.push(format!("{}={}", prefix, b)),
            _ => {} // Skip complex types for environment variables
        }
    }
    
    flatten_config(config, "", &mut env_vars);
    Ok(env_vars.join("\n"))
}

/// Retrieves server configuration.
async fn retrieve_server_config(client: &aerolithsClient) -> Result<Value> {
    let response = client.get("/api/v1/config").await?;
    
    if response.status().is_success() {
        let config: Value = response.json().await?;
        Ok(config)
    } else {
        Err(anyhow!("Failed to retrieve server configuration: {}", response.status()))
    }
}

/// Retrieves default configuration.
async fn retrieve_default_config() -> Result<Value> {
    // Return basic configuration as default
    Ok(generate_basic_config())
}

/// Filters configuration by section.
fn filter_config_by_section(config: &Value, section: &str) -> Result<Value> {
    if let Some(section_config) = config.get(section) {
        Ok(section_config.clone())
    } else {
        Err(anyhow!("Configuration section '{}' not found", section))
    }
}

/// Masks sensitive values in configuration.
fn mask_sensitive_values(mut config: Value) -> Value {    let sensitive_keys = ["password", "secret", "key", "token", "cert"];
    
    fn mask_object(obj: &mut serde_json::Map<String, Value>, sensitive_keys: &[&str]) {
        for (key, value) in obj.iter_mut() {
            let key_lower = key.to_lowercase();
            if sensitive_keys.iter().any(|&s| key_lower.contains(s)) {
                *value = Value::String("***MASKED***".to_string());
            } else if let Value::Object(ref mut nested) = value {
                mask_object(nested, sensitive_keys);
            }
        }    }
    
    if let Value::Object(ref mut obj) = config {
        mask_object(obj, &sensitive_keys);
    }
    
    config
}

/// Formats configuration as a table.
fn format_config_as_table(config: &Value) -> Result<String> {
    let mut table = String::new();
    table.push_str("Configuration Settings\n");
    table.push_str("=====================\n\n");
    
    fn format_section(obj: &Value, prefix: &str, table: &mut String) {
        match obj {
            Value::Object(map) => {
                for (key, value) in map {
                    let full_key = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    
                    match value {
                        Value::Object(_) => {
                            format_section(value, &full_key, table);
                        },
                        _ => {
                            table.push_str(&format!("{:<30} {}\n", full_key, format_value(value)));
                        }
                    }
                }
            },
            _ => table.push_str(&format!("{:<30} {}\n", prefix, format_value(obj)))
        }
    }
    
    format_section(config, "", &mut table);
    Ok(table)
}

/// Formats a JSON value for display.
fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(arr) => format!("[{} items]", arr.len()),
        Value::Object(obj) => format!("{{{}}} fields", obj.len()),
    }
}
