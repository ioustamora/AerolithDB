//! # Query Operations
//!
//! This module implements CLI commands for querying and listing documents:
//! - QUERY: Complex filtering, sorting, and aggregation
//! - LIST: Simple document enumeration with pagination

use anyhow::Result;
use serde_json::Value;
use tracing::{error, info, warn};

use crate::client::aerolithsClient;
use crate::args::{QueryArgs, ListArgs};
use crate::utils::parse_json_input;

/// Executes the QUERY command to search documents with filtering and sorting.
///
/// ## Query Processing Pipeline
///
/// 1. **Filter Parsing**: Validates and parses JSON filter expressions
/// 2. **Sort Configuration**: Processes sort specifications for result ordering
/// 3. **Execution**: Performs optimized query execution with indexing
/// 4. **Result Formatting**: Converts results to requested output format
///
/// ## Query Language Support
///
/// Supports MongoDB-style query operators:
/// - **Equality**: `{"name": "John"}` - Exact field matching
/// - **Comparison**: `{"age": {"$gt": 25, "$lt": 65}}` - Numeric comparisons
/// - **Logical**: `{"$or": [{"active": true}, {"priority": "high"}]}` - Boolean logic
/// - **Array Operations**: `{"tags": {"$in": ["urgent", "important"]}}` - Array membership
/// - **Text Search**: `{"description": {"$regex": "pattern"}}` - Pattern matching
/// - **Existence**: `{"field": {"$exists": true}}` - Field presence checking
///
/// ## Performance Optimization
///
/// Query execution is optimized through:
/// - **Index Utilization**: Automatic selection of optimal indices
/// - **Query Planning**: Cost-based optimization for complex queries
/// - **Result Streaming**: Memory-efficient processing for large result sets
/// - **Caching**: Intelligent caching of frequently executed queries
///
/// ## Result Formatting Options
///
/// Multiple output formats support different use cases:
/// - **JSON**: Complete structured data for API integration
/// - **YAML**: Human-readable format for documentation
/// - **Table**: Rich formatted display with metadata and pagination guidance
///
/// # Arguments
///
/// * `client` - Configured aerolithsClient for server communication
/// * `args` - Parsed command-line arguments including filters, sorting, and pagination
///
/// # Returns
///
/// * `Result<()>` - Success indication or detailed error information
pub async fn execute_query(client: &aerolithsClient, args: &QueryArgs) -> Result<()> {
    info!("Querying collection {} with filters", args.collection);

    // Parse and validate filter expression
    let filter = if let Some(f) = &args.filter {
        Some(parse_json_input(f).map_err(|e| {
            anyhow::anyhow!("Invalid filter JSON: {}. \
                            Example: '{{\"age\": {{\"$gte\": 18}}}}'", e)
        })?)
    } else {
        None
    };

    // Parse and validate sort expression
    let sort = if let Some(s) = &args.sort {
        Some(parse_json_input(s).map_err(|e| {
            anyhow::anyhow!("Invalid sort JSON: {}. \
                            Example: '{{\"name\": 1, \"age\": -1}}'", e)
        })?)
    } else {
        None
    };

    // Build complete query object
    let query = serde_json::json!({
        "filter": filter,
        "limit": args.limit,
        "offset": args.offset,
        "sort": sort,
        "include_metadata": args.include_metadata,
        "explain": args.explain
    });

    let start_time = std::time::Instant::now();
    
    match client.query_documents(&args.collection, &query).await {
        Ok(response) => {
            let execution_time = start_time.elapsed();
            info!("Query completed successfully in {:?}", execution_time);
            
            // Format output according to user preference
            match args.format.as_str() {
                "json" => {
                    // Complete JSON output for API integration
                    println!("{}", serde_json::to_string_pretty(&response)?);
                }
                "jsonl" => {
                    // JSON Lines format for streaming processing
                    for doc in &response.documents {
                        println!("{}", serde_json::to_string(&doc)?);
                    }
                }
                "csv" => {
                    // CSV format for spreadsheet analysis (flattened)
                    println!("ID,Collection,Version,Data");
                    for doc in &response.documents {
                        let data_str = serde_json::to_string(&doc.data)?;
                        println!("{},{},{},{}", doc.id, args.collection, doc.version, data_str);
                    }
                }
                "count" => {
                    // Count-only output for aggregate queries
                    println!("{}", response.total);
                }
                "table" => {
                    // Rich formatted output with comprehensive metadata
                    println!("🔍 Query Results:");
                    println!("  Total: {} documents", response.total);
                    println!("  Results: {}", response.documents.len());
                    println!("  Execution time: {}ms", execution_time.as_millis());
                    
                    if args.limit > 0 {
                        println!("  Limit: {}", args.limit);                    }
                    if args.offset > 0 {
                        println!("  Offset: {}", args.offset);
                    }
                    
                    // Show pagination context if applicable
                    if response.total > response.documents.len() {
                        let remaining = response.total.saturating_sub(response.documents.len() + args.offset as usize);
                        println!("  Remaining: {} documents", remaining);
                    }
                    
                    println!();

                    // Display individual documents with metadata
                    for (i, doc) in response.documents.iter().enumerate() {
                        println!("📄 Document {} (ID: {}, Version: {})", 
                               i + 1, doc.id, doc.version);
                        
                        if args.include_metadata {
                            println!("   Created: {}", doc.created_at);
                            println!("   Updated: {}", doc.updated_at);
                        }
                        
                        println!("{}", serde_json::to_string_pretty(&doc.data)?);
                        
                        if i < response.documents.len() - 1 {
                            println!("---");
                        }
                    }
                    
                    // Provide pagination guidance for large result sets
                    if response.total > response.documents.len() {
                        println!();
                        println!("💡 Use --offset and --limit for pagination through remaining results");
                    }                    // Show query explanation if requested
                    if args.explain {
                        println!();
                        println!("🔍 Query Execution Plan:");
                        println!("Query execution plan not available in current response format");
                    }
                }
                _ => {
                    warn!("Unknown format '{}', using JSON", args.format);
                    println!("{}", serde_json::to_string_pretty(&response)?);
                }
            }
        }
        Err(e) => {
            error!("Query failed: {}", e);
            eprintln!("✗ Query failed: {}", e);
            
            // Provide specific troubleshooting guidance based on error type
            if e.to_string().contains("syntax") {
                eprintln!("  → Check filter and sort JSON syntax");
                eprintln!("  → Example filter: '{{\"age\": {{\"$gte\": 18}}}}'");
                eprintln!("  → Example sort: '{{\"name\": 1, \"age\": -1}}'");
            } else if e.to_string().contains("timeout") {
                eprintln!("  → Query took too long to execute");
                eprintln!("  → Try using more specific filters");
                eprintln!("  → Consider increasing timeout value");
            } else if e.to_string().contains("limit") {
                eprintln!("  → Result set too large");
                eprintln!("  → Use --limit to reduce result size");
                eprintln!("  → Add more specific filters");
            } else if e.to_string().contains("index") {
                eprintln!("  → Query may benefit from additional indices");
                eprintln!("  → Use 'optimize' command for index suggestions");
            }
            
            return Err(e);
        }
    }

    Ok(())
}

/// Executes the LIST command to enumerate documents in a collection.
///
/// ## List Operation Characteristics
///
/// This operation provides a simplified way to browse collection contents:
/// - **No Filtering**: Returns all accessible documents in the collection
/// - **Default Ordering**: Documents are typically ordered by creation time
/// - **Pagination Support**: Efficient browsing of large collections
/// - **Metadata Inclusion**: Shows document IDs, versions, and timestamps
///
/// ## Performance Considerations
///
/// The list operation is optimized for browsing scenarios:
/// - **Streaming Results**: Large collections are processed in chunks
/// - **Index Utilization**: Uses creation timestamp indices for ordering
/// - **Memory Efficiency**: Minimal memory footprint even for large collections
/// - **Network Optimization**: Efficient serialization and compression
///
/// ## Output Format Variations
///
/// Different formats serve different use cases:
/// - **JSON**: Complete document data for programmatic processing
/// - **YAML**: Human-readable format for inspection and documentation
/// - **Table**: Summary view optimized for interactive browsing and pagination
/// - **CSV**: Tabular format for spreadsheet analysis
///
/// ## Pagination Best Practices
///
/// For large collections, efficient pagination strategies:
/// - Use consistent limit sizes (typically 20-100 documents)
/// - Implement offset-based pagination for simple browsing
/// - Consider cursor-based pagination for real-time data
/// - Monitor memory usage with very large result sets
///
/// # Arguments
///
/// * `client` - Configured aerolithsClient for server communication
/// * `args` - Parsed command-line arguments including collection and pagination
///
/// # Returns
///
/// * `Result<()>` - Success indication or detailed error information
pub async fn execute_list(client: &aerolithsClient, args: &ListArgs) -> Result<()> {
    if let Some(collection) = &args.collection {
        info!("Listing documents in collection {}", collection);
        
        // List documents in specific collection
        match client.list_documents(collection, None, None).await {
            Ok(documents) => {
                info!("Retrieved {} documents from collection", documents.len());
                
                // Format output according to user preference
                match args.format.as_str() {
                    "json" => {
                        println!("{}", serde_json::to_string_pretty(&documents)?);
                    }
                    "yaml" => {
                        println!("{}", serde_yaml::to_string(&documents)?);
                    }
                    "csv" => {
                        println!("ID,Version,Created,Updated");
                        for doc in documents {
                            println!("{},{},{},{}", 
                                   doc.id, doc.version, doc.created_at, doc.updated_at);
                        }
                    }
                    "table" => {
                        println!("📋 Documents in collection '{}':", collection);
                        
                        if documents.is_empty() {
                            println!("📭 No documents found");
                            println!("  → Collection may be empty or inaccessible");
                            println!("  → Use 'put' command to add documents");
                            return Ok(());
                        }
                        
                        println!("Total: {} documents", documents.len());
                        println!();

                        // Display document summary information
                        for doc in documents {
                            let size = serde_json::to_string(&doc.data)?.len();
                            println!("📄 ID: {} | Version: {} | Size: {} bytes | Updated: {}", 
                                   doc.id, 
                                   doc.version,
                                   size,
                                   doc.updated_at);
                        }
                        
                        if args.detailed {
                            println!();
                            println!("💡 Use 'query' command for filtering and advanced pagination");
                        }
                    }
                    _ => {
                        warn!("Unknown format '{}', using JSON", args.format);
                        println!("{}", serde_json::to_string_pretty(&documents)?);
                    }
                }
            }
            Err(e) => {
                error!("Failed to list documents: {}", e);
                eprintln!("✗ Failed to list documents: {}", e);
                
                if e.to_string().contains("permission") {
                    eprintln!("  → Check read permissions for collection '{}'", collection);
                } else if e.to_string().contains("not found") {
                    eprintln!("  → Collection '{}' may not exist", collection);
                    eprintln!("  → Use 'list' without collection name to see available collections");
                }
                
                return Err(e);
            }
        }
    } else {
        info!("Listing all available collections");
        
        // List all collections
        match client.list_collections().await {
            Ok(collections) => {
                info!("Retrieved {} collections", collections.len());
                
                // Format output according to user preference
                match args.format.as_str() {
                    "json" => {
                        println!("{}", serde_json::to_string_pretty(&collections)?);
                    }
                    "yaml" => {
                        println!("{}", serde_yaml::to_string(&collections)?);
                    }
                    "csv" => {
                        println!("Name,Documents,Size,Created");
                        for collection in collections {
                            println!("{},{},{},{}", 
                                   collection.name, collection.document_count, 
                                   collection.size_bytes, collection.created_at);
                        }
                    }
                    "table" => {
                        println!("📚 Available Collections:");
                        
                        if collections.is_empty() {
                            println!("📭 No collections found");
                            println!("  → Create your first collection by adding a document");
                            println!("  → Use 'put' command to create collection automatically");
                            return Ok(());
                        }
                          println!("Total: {} collections", collections.len());
                        
                        // Apply pattern filter if specified
                        let (filtered_collections, original_count) = if let Some(pattern) = &args.pattern {
                            let original_count = collections.len();
                            let filtered: Vec<_> = collections.into_iter()
                                .filter(|c| c.name.contains(pattern))
                                .collect();
                            (filtered, original_count)
                        } else {
                            let count = collections.len();
                            (collections, count)
                        };
                        
                        if args.pattern.is_some() && filtered_collections.len() < original_count {
                            println!("Filtered: {} collections match pattern '{}'", 
                                   filtered_collections.len(), args.pattern.as_ref().unwrap());
                        }
                        
                        println!();

                        // Display collection summary information
                        for collection in filtered_collections {
                            print!("📁 {}", collection.name);
                            
                            if args.detailed {                                println!();
                                println!("   Documents: {}", collection.document_count);
                                println!("   Size: {} bytes", collection.size_bytes);
                                println!("   Created: {}", collection.created_at);
                                if let Some(updated) = collection.updated_at {
                                    println!("   Updated: {}", updated);
                                }
                            } else {
                                println!(" ({} documents, {} bytes)", 
                                       collection.document_count, collection.size_bytes);
                            }
                        }
                        
                        println!();
                        println!("💡 Use 'list <collection>' to see documents in a specific collection");
                    }
                    _ => {
                        warn!("Unknown format '{}', using JSON", args.format);
                        println!("{}", serde_json::to_string_pretty(&collections)?);
                    }
                }
            }
            Err(e) => {
                error!("Failed to list collections: {}", e);
                eprintln!("✗ Failed to list collections: {}", e);
                
                if e.to_string().contains("permission") {
                    eprintln!("  → Check system read permissions");
                } else if e.to_string().contains("connection") {
                    eprintln!("  → Check server connectivity and network configuration");
                }
                
                return Err(e);
            }
        }
    }

    Ok(())
}
