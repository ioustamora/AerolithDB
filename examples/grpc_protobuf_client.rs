use anyhow::Result;
use tonic::Request;

use aerolithdb_api::proto::*;

/// Example gRPC client demonstrating Protocol Buffer integration
/// 
/// This example shows how to connect to aerolithsDB using the generated
/// Protocol Buffer client, providing type-safe operations with
/// cross-language compatibility.
#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 aerolithsDB gRPC Protocol Buffer Client Example");
    println!("==============================================");

    // Connect to the gRPC server
    let mut client = DataServiceClient::connect("http://127.0.0.1:8082").await?;
    println!("✅ Connected to aerolithsDB gRPC server");

    // Health check
    println!("\n📊 Performing health check...");
    let health_request = Request::new(HealthCheckRequest {});
    
    match client.health_check(health_request).await {
        Ok(response) => {
            let status = response.into_inner().status;
            println!("✅ Health check: {:?}", 
                health_check_response::ServingStatus::from_i32(status));
        }
        Err(e) => {
            println!("❌ Health check failed: {}", e);
            return Ok(());
        }
    }

    // Example document data
    let example_doc = serde_json::json!({
        "name": "Protocol Buffer Test",
        "type": "example",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "features": [
            "Cross-language support",
            "Type safety",
            "Efficient serialization"
        ]
    });

    // Store a document
    println!("\n📝 Storing example document...");
    let put_request = Request::new(PutDocumentRequest {
        collection: "examples".to_string(),
        document_id: "protobuf_test".to_string(),
        data: serde_json::to_vec(&example_doc)?,
        metadata: std::collections::HashMap::new(),
    });

    match client.put_document(put_request).await {
        Ok(response) => {
            let inner = response.into_inner();
            match inner.result {
                Some(put_document_response::Result::Success(result)) => {
                    println!("✅ Document stored: ID={}, Version={}", 
                        result.document_id, result.version);
                }
                Some(put_document_response::Result::Error(error)) => {
                    println!("❌ Storage error: {}", error.message);
                }
                None => {
                    println!("❌ Unexpected response format");
                }
            }
        }
        Err(e) => {
            println!("❌ gRPC call failed: {}", e);
        }
    }

    // Retrieve the document
    println!("\n📖 Retrieving document...");
    let get_request = Request::new(GetDocumentRequest {
        collection: "examples".to_string(),
        document_id: "protobuf_test".to_string(),
    });

    match client.get_document(get_request).await {
        Ok(response) => {
            let inner = response.into_inner();
            match inner.result {
                Some(get_document_response::Result::Document(doc)) => {
                    let doc_data: serde_json::Value = serde_json::from_slice(&doc.data)?;
                    println!("✅ Document retrieved:");
                    println!("   ID: {}", doc.id);
                    println!("   Collection: {}", doc.collection);
                    println!("   Data: {}", serde_json::to_string_pretty(&doc_data)?);
                    println!("   Version: {}", doc.version);
                }
                Some(get_document_response::Result::Error(error)) => {
                    println!("❌ Retrieval error: {}", error.message);
                }
                None => {
                    println!("❌ Unexpected response format");
                }
            }
        }
        Err(e) => {
            println!("❌ gRPC call failed: {}", e);
        }
    }

    // Query documents
    println!("\n🔍 Querying documents...");
    let query_request = Request::new(QueryDocumentsRequest {
        collection: "examples".to_string(),
        filter: None,
        sort: None,
        limit: Some(10),
        offset: Some(0),
    });

    match client.query_documents(query_request).await {
        Ok(response) => {
            let inner = response.into_inner();
            match inner.result {
                Some(query_documents_response::Result::Success(result)) => {
                    println!("✅ Query completed:");
                    println!("   Found {} documents", result.documents.len());
                    println!("   Total count: {}", result.total_count);
                    println!("   Execution time: {:.2}ms", result.execution_time_ms);
                }
                Some(query_documents_response::Result::Error(error)) => {
                    println!("❌ Query error: {}", error.message);
                }
                None => {
                    println!("❌ Unexpected response format");
                }
            }
        }
        Err(e) => {
            println!("❌ gRPC call failed: {}", e);
        }
    }

    // Get database stats
    println!("\n📊 Getting database statistics...");
    let stats_request = Request::new(GetStatsRequest {
        collection: None,
    });

    match client.get_stats(stats_request).await {
        Ok(response) => {
            let inner = response.into_inner();
            match inner.result {
                Some(get_stats_response::Result::Stats(stats)) => {
                    println!("✅ Database statistics:");
                    println!("   Version: {}", stats.database_version);
                    println!("   Uptime: {}s", stats.uptime_seconds);
                    println!("   Total documents: {}", stats.total_documents);
                    println!("   Total size: {} bytes", stats.total_size_bytes);
                    println!("   Collections: {}", stats.collections.len());
                    
                    for collection in stats.collections {
                        println!("     - {}: {} docs, {} bytes", 
                            collection.name, 
                            collection.document_count,
                            collection.size_bytes);
                    }
                }
                Some(get_stats_response::Result::Error(error)) => {
                    println!("❌ Stats error: {}", error.message);
                }
                None => {
                    println!("❌ Unexpected response format");
                }
            }
        }
        Err(e) => {
            println!("❌ gRPC call failed: {}", e);
        }
    }

    println!("\n🎉 Protocol Buffer gRPC client example completed!");
    println!("✨ This demonstrates type-safe, cross-language gRPC communication");
    
    Ok(())
}
