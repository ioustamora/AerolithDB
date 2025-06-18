//! # aerolithsDB Command Line Interface (CLI)
//!
//! This is the primary command-line client for interacting with aerolithsDB distributed database
//! instances. The CLI provides comprehensive functionality for document management, querying,
//! analytics, administration, and system monitoring through an intuitive command structure.
//!
//! ## Features
//!
//! - **Document Operations**: Store, retrieve, update, and delete documents with full CRUD support
//! - **Advanced Querying**: Complex filters, sorting, pagination, and aggregation queries
//! - **Collection Management**: Create, list, and manage document collections
//! - **Node Operations**: Join/leave networks, monitor node health and status
//! - **Network Administration**: Network creation, topology management, peer monitoring
//! - **System Analytics**: Real-time metrics, performance monitoring, and analytics
//! - **Configuration Management**: Validate, generate, and manage configuration files
//! - **Multiple Output Formats**: JSON, YAML, and formatted table output support
//!
//! ## Command Structure
//!
//! The CLI follows a hierarchical command structure:
//! ```
//! aerolithsdb-cli [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGUMENTS]
//! ```
//!
//! ### Global Options
//! - `--server`: aerolithsDB server endpoint (default: http://localhost:8080)
//! - `--timeout`: Request timeout in seconds (default: 30)
//! - `--format`: Output format (json, yaml, table)
//! - `--verbose`: Enable debug-level logging
//!
//! ### Command Categories
//! - `document`: Document CRUD operations
//! - `collection`: Collection management
//! - `query`: Advanced querying and analytics
//! - `node`: Node lifecycle and status
//! - `network`: Network administration
//! - `status`: System monitoring and metrics
//! - `config`: Configuration management
//!
//! ## Usage Examples
//!
//! ```bash
//! # Store a document
//! aerolithsdb-cli document put users user123 '{"name": "John", "age": 30}'
//!
//! # Query with filters
//! aerolithsdb-cli query search users --filter '{"age": {"$gte": 18}}' --limit 100
//!
//! # Monitor system status
//! aerolithsdb-cli status system --format table
//!
//! # Network operations
//! aerolithsdb-cli node join my-network --capabilities "storage,compute"
//! ```
//!
//! ## Error Handling
//!
//! The CLI provides comprehensive error handling with:
//! - Structured error messages with context
//! - HTTP status code interpretation
//! - Network connectivity diagnostics
//! - Input valiaerolithon and helpful suggestions
//! - Exit codes for scripting integration

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, error};
use tracing_subscriber;
use std::time::Duration;

mod client;
mod commands;
mod document;
mod query;
mod analytics;
mod config;
mod batch;
mod args;
mod utils;
// mod wallet;  // Temporarily disabled
mod crypto_wallet;
mod saas;
mod tui;

use client::aerolithsClient;
use commands::*;
use crypto_wallet::{WalletArgs, handle_wallet_command};
use saas::{SaaSArgs, handle_saas_command};

/// aerolithsDB CLI - Command line client for aerolithsDB distributed database.
///
/// This tool provides comprehensive access to aerolithsDB functionality including
/// document management, querying, analytics, node administration, and system monitoring.
/// Supports multiple output formats and extensive configuration options for
/// integration with scripts and automation workflows.
#[derive(Parser)]
#[command(name = "aerolithsdb-cli")]
#[command(about = "A CLI client for aerolithsDB distributed database")]
#[command(version = "1.0.0")]
#[command(long_about = "
aerolithsDB CLI provides comprehensive command-line access to aerolithsDB distributed database instances.
Supports document operations, advanced querying, node management, network administration,
and real-time analytics with multiple output formats for both interactive use and automation.

For detailed help on any command, use: aerolithsdb-cli <command> --help
")]
struct Cli {
    /// aerolithsDB server endpoint URL.
    /// 
    /// Specifies the base URL of the aerolithsDB server to connect to. Supports HTTP and HTTPS
    /// protocols. The CLI will automatically append appropriate API paths for different operations.
    /// Can include custom ports and paths as needed.
    /// 
    /// Examples:
    /// - http://localhost:8080 (default local development)
    /// - https://aerolithsdb.company.com (production HTTPS)
    /// - http://10.0.1.100:9090 (custom network/port)
    #[arg(short, long, default_value = "http://localhost:8080")]
    url: String,
    
    /// Request timeout duration in seconds.
    /// 
    /// Maximum time to wait for server responses before timing out. Applies to all
    /// HTTP requests made by the CLI. For long-running operations like large queries
    /// or analytics, consider increasing this value. The timeout includes both
    /// connection establishment and response time.
    #[arg(short, long, default_value = "30")]
    timeout: u64,
    
    /// Enable verbose debug logging.
    /// 
    /// When enabled, shows detailed debug information including:
    /// - HTTP request/response details
    /// - Internal operation timing
    /// - Network communication traces
    /// - Error stack traces and context
    /// Useful for troubleshooting connectivity and performance issues.    #[arg(short, long)]
    verbose: bool,
    
    /// Launch the Terminal User Interface (TUI).
    /// 
    /// When enabled, launches an interactive terminal interface instead of
    /// executing traditional CLI commands. The TUI provides real-time monitoring,
    /// node management, cluster visualization, test execution, and configuration
    /// management through an intuitive tabbed interface.
    /// 
    /// Examples:
    /// - aerolithsdb-cli --tui (launch TUI mode)
    /// - aerolithsdb-cli --no-tui status (force CLI mode)
    #[arg(long, default_value = "false")]
    tui: bool,
    
    /// Primary command to execute.
    /// 
    /// The CLI is organized into command groups for different functional areas.
    /// Each command group contains related subcommands with specific options
    /// and arguments appropriate for that domain.
    /// 
    /// Note: This is ignored when --tui flag is used.
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Primary command categories available in the aerolithsDB CLI.
/// 
/// Each command category focuses on a specific aspect of aerolithsDB functionality:
/// - Document operations for data management
/// - Collection operations for schema and organization
/// - Query operations for data retrieval and analysis
/// - Node operations for cluster management
/// - Network operations for topology and connectivity
/// - Status operations for monitoring and diagnostics
/// - Config operations for system configuration
#[derive(Subcommand)]
enum Commands {
    /// Store a document in a collection.
    /// 
    /// Creates or updates a document in the specified collection with the provided data.
    /// Supports various data input methods including inline JSON, file paths, and
    /// advanced options for encryption, replication, and retention policies.
    Put(PutArgs),
    
    /// Retrieve a document from a collection.
    /// 
    /// Fetches a specific document by its ID from the given collection.
    /// Supports multiple output formats and can handle non-existent documents gracefully.
    /// Provides detailed metadata including version information and timestamps.
    Get(GetArgs),
    
    /// Delete a document from a collection.
    /// 
    /// Removes a document from the specified collection. Includes safety features
    /// like confirmation prompts (unless forced) and provides clear feedback on
    /// the operation success. Handles cases where documents don't exist gracefully.
    Delete(DeleteArgs),
    
    /// Query documents with filters and options.
    /// 
    /// Performs advanced queries against collections with support for complex filters,
    /// sorting, pagination, and aggregation. Optimizes query execution based on
    /// available indices and provides performance hints and execution statistics.
    Query(QueryArgs),
    
    /// List documents in a collection.
    /// 
    /// Retrieves a paginated list of documents from a collection. Useful for
    /// browsing collection contents, getting overviews of data, and implementing
    /// pagination in applications. Supports various output formats for integration.
    List(ListArgs),
    
    /// Show database and collection statistics.
    /// 
    /// Retrieves comprehensive statistics about the database system, including
    /// storage usage, document counts, index information, performance metrics,
    /// and health indicators. Can be scoped to specific collections or system-wide.
    Stats(StatsArgs),
    
    /// Check server health and connectivity.
    ///    /// Performs a health check against the aerolithsDB server to verify connectivity,
    /// service availability, and basic operational status. Useful for monitoring
    /// scripts and automated health checks in production environments.
    Health,
    
    /// Start analytics collection and monitoring.
    /// 
    /// Initiates real-time analytics collection for specified collections and metrics.
    /// Can stream results to external systems for monitoring, alerting, and business
    /// intelligence. Supports customizable metric selection and output destinations.
    Analytics(AnalyticsArgs),
    
    /// Analyze and optimize query performance.
    /// 
    /// Performs query analysis and optimization recommenaerolithons based on historical
    /// query patterns, index usage, and performance metrics. Suggests index creation,
    /// query rewriting, and configuration optimizations for improved performance.
    Optimize(OptimizeArgs),

    // ================================================================================================
    // CONFIGURATION MANAGEMENT COMMANDS
    // ================================================================================================

    /// Validate configuration files and settings.
    /// 
    /// Provides comprehensive configuration valiaerolithon including syntax checking,
    /// schema valiaerolithon, and value verification. Supports both local file and
    /// server configuration valiaerolithon with detailed error reporting.
    ConfigValidate(ConfigValidateArgs),
    
    /// Generate configuration templates.
    /// 
    /// Creates configuration templates for various deployment scenarios including
    /// development, production, and cluster configurations. Supports multiple
    /// output formats and customizable component selection.
    ConfigGenerate(ConfigGenerateArgs),
    
    /// Display current configuration.
    /// 
    /// Shows current effective configuration from server or defaults with
    /// security-conscious sensitive value masking and flexible formatting options.
    ConfigShow(ConfigShowArgs),

    // ================================================================================================
    // BATCH OPERATIONS COMMANDS
    // ================================================================================================

    /// Bulk document insertion from files or streams.
    /// 
    /// High-performance bulk document insertion supporting multiple input formats,
    /// parallel processing, and error resilience. Ideal for data migration,
    /// initial data loading, and ETL pipelines.
    BatchPut(BatchPutArgs),
    
    /// Bulk document deletion with safety features.
    /// 
    /// Efficient bulk document deletion supporting ID lists, filter-based selection,
    /// and comprehensive safety features including dry-run mode, confirmations,
    /// and backup creation.
    BatchDelete(BatchDeleteArgs),
    
    /// Import data from external formats.
    /// 
    /// Comprehensive data import supporting CSV, XML, JSON, and other formats
    /// with field mapping, valiaerolithon, and transformation capabilities.
    /// Designed for data migration and integration workflows.
    BatchImport(BatchImportArgs),
    
    /// Export data to external formats.
    /// 
    /// Flexible data export supporting multiple output formats, filtering,
    /// field selection, and compression. Optimized for backup, analysis,
    /// and data integration workflows.
    BatchExport(BatchExportArgs),

    // ================================================================================================
    // WALLET MANAGEMENT COMMANDS
    // ================================================================================================

    /// Create a new wallet.
    ///    // /// Generates a new wallet with a secure keypair and optional metadata.
    // /// The wallet can be used for transaction signing, authentication,
    // /// and secure storage of sensitive information.
    // WalletCreate(WalletCreateArgs),  // Temporarily disabled
    
    // /// Import an existing wallet.
    // /// 
    // /// Imports a wallet from a file or standard input. Supports various
    // /// formats including JSON, YAML, and binary. The import process
    // /// includes key derivation, metadata extraction, and integrity verification.
    // WalletImport(WalletImportArgs),  // Temporarily disabled
    
    // /// Export a wallet to a file or standard output.
    // /// 
    // /// Exports the specified wallet including its keys and metadata.
    // /// Supports encryption and compression options for secure and efficient
    // /// storage. The export process creates a portable wallet archive.
    // WalletExport(WalletExportArgs),  // Temporarily disabled
    
    // /// List available wallets.
    // /// 
    // /// Displays a list of all wallets managed by the CLI including
    // /// metadata such as creation date, last modified date, and key
    // /// fingerprint. Supports filtering and formatting options.
    // WalletList(WalletListArgs),  // Temporarily disabled
    
    // /// Get wallet details.
    // /// 
    // /// Retrieves detailed information about a specific wallet including
    // /// its keys, metadata, and usage statistics. Supports output formatting
    // /// and filtering options.
    // WalletGet(WalletGetArgs),  // Temporarily disabled
      
    // /// Delete a wallet.
    // /// 
    // /// Permanently removes a wallet and its associated keys from the
    // /// system. Includes safety features like confirmation prompts
    // /// /// and provides clear feedback on the operation success.
    // WalletDelete(WalletDeleteArgs),  // Temporarily disabled
    
    /// Cryptocurrency wallet and payment operations.
    /// 
    /// Connect to Tron and Solana wallets, check USDT/USDC balances,
    /// make payments for services, and manage payment history.
    /// Supports both testnet and mainnet environments.
    CryptoWallet(WalletArgs),
    
    /// SaaS management operations.
    /// 
    /// Manage tenants, billing, quotas, SSO, analytics, and admin functions
    /// for AerolithDB SaaS/DBaaS deployments. Includes tenant creation,
    /// billing management, usage tracking, and system administration.
    Saas(SaaSArgs),
}

/// Main CLI entry point with comprehensive error handling and logging setup.
/// 
/// This function orchestrates the entire CLI execution flow:
/// 1. Parses command-line arguments and validates inputs
/// 2. Configures logging based on verbosity settings
/// 3. Establishes connection to the aerolithsDB server
/// 4. Routes commands to appropriate handlers
/// 5. Manages error reporting and exit codes
/// 
/// ## Error Handling Strategy
/// 
/// The CLI provides user-friendly error messages while maintaining detailed
/// logging for debugging. Network errors, authentication failures, and
/// server errors are handled gracefully with actionable feedback.
/// 
/// ## Exit Codes
/// 
/// - 0: Successful execution
/// - 1: General error (network, server, or operation failure)
/// - 2: Invalid arguments or configuration
/// 
/// ## Logging Configuration
/// 
/// Logging is configured based on the --verbose flag:
/// - Normal mode: INFO level for CLI operations
/// - Verbose mode: DEBUG level with detailed traces
/// 
/// Logs include structured information for easy parsing and analysis.
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize structured logging with appropriate level based on verbosity.
    // The logging configuration provides clean, readable output for end users
    // while maintaining detailed debugging information when requested.
    let level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("aerolithsdb_cli={}", level))
        .init();
      info!("aerolithsDB CLI starting, connecting to: {}", cli.url);
    
    // Create HTTP client with configured timeout and connection parameters.
    // The client handles all communication with the aerolithsDB server including
    // authentication, request formatting, and response parsing.
    let client = aerolithsClient::new(cli.url, Some(Duration::from_secs(cli.timeout)))?;
    
    // Check if TUI mode is requested or if no command is provided (default to TUI)
    if cli.tui || cli.command.is_none() {
        info!("Launching Terminal User Interface (TUI)");
        return tui::launch_tui(client).await;
    }
    
    // Route the command to the appropriate handler with comprehensive error handling.
    // Each command handler is responsible for input valiaerolithon, server communication,
    // result formatting, and user feedback.
    match cli.command.unwrap() {
        Commands::Put(args) => {
            execute_put(&client, &args).await?;
        }
        Commands::Get(args) => {
            execute_get(&client, &args).await?;
        }
        Commands::Delete(args) => {
            execute_delete(&client, &args).await?;
        }
        Commands::Query(args) => {
            execute_query(&client, &args).await?;
        }
        Commands::List(args) => {
            execute_list(&client, &args).await?;
        }
        Commands::Stats(args) => {
            execute_stats(&client, &args).await?;
        }
        Commands::Health => {
            // Health check command provides immediate feedback on server status.
            // Uses visual indicators (✓/✗) for quick status recognition and
            // appropriate exit codes for script integration.
            match client.health_check().await {
                Ok(true) => {
                    info!("Server is healthy");
                    println!("✓ Server is healthy");
                }
                Ok(false) => {
                    error!("Server is not healthy");
                    println!("✗ Server is not healthy");
                    std::process::exit(1);
                }
                Err(e) => {
                    error!("Health check failed: {}", e);
                    println!("✗ Health check failed: {}", e);
                    std::process::exit(1);
                }
            }        }
        Commands::Analytics(args) => {
            execute_analytics(&client, &args).await?;
        }
        Commands::Optimize(args) => {
            execute_optimize(&client, &args).await?;
        }

        // Configuration management commands
        Commands::ConfigValidate(args) => {
            config::execute_config_validate(&client, &args).await?;
        }
        Commands::ConfigGenerate(args) => {
            config::execute_config_generate(&client, &args).await?;
        }
        Commands::ConfigShow(args) => {
            config::execute_config_show(&client, &args).await?;
        }

        // Batch operations commands
        Commands::BatchPut(args) => {
            batch::execute_batch_put(&client, &args).await?;
        }
        Commands::BatchDelete(args) => {
            batch::execute_batch_delete(&client, &args).await?;
        }
        Commands::BatchImport(args) => {
            batch::execute_batch_import(&client, &args).await?;
        }
        Commands::BatchExport(args) => {
            batch::execute_batch_export(&client, &args).await?;
        }        // // Wallet management commands - temporarily disabled
        // Commands::WalletCreate(args) => {
        //     wallet::execute_wallet_create(&client, &args).await?;
        // }
        // Commands::WalletImport(args) => {
        //     wallet::execute_wallet_import(&client, &args).await?;
        // }
        // Commands::WalletExport(args) => {
        //     wallet::execute_wallet_export(&client, &args).await?;
        // }
        // Commands::WalletList(args) => {
        //     wallet::execute_wallet_list(&client, &args).await?;
        // }
        // Commands::WalletGet(args) => {
        //     wallet::execute_wallet_get(&client, &args).await?;
        // }
        // Commands::WalletDelete(args) => {
        //     wallet::execute_wallet_delete(&client, &args).await?;
        // }
        
        // Cryptocurrency wallet and payment commands
        Commands::CryptoWallet(args) => {
            handle_wallet_command(args, &client).await?;
        }
        
        // SaaS management commands
        Commands::Saas(args) => {
            handle_saas_command(&client, args).await?;
        }
    }
      Ok(())
}
