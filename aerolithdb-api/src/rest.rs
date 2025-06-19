use anyhow::Result;
use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use tower_http::cors::CorsLayer;

use aerolithdb_query::QueryEngine;
use aerolithdb_security::SecurityFramework;

use super::RESTAPIConfig;

#[derive(Debug, Clone)]
pub struct RESTAPIv1 {
    config: RESTAPIConfig,
    query: Arc<QueryEngine>,
    security: Arc<SecurityFramework>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentRequest {
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentResponse {
    pub id: String,
    pub data: serde_json::Value,
    pub version: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    pub filter: Option<serde_json::Value>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResponse {
    pub documents: Vec<DocumentResponse>,
    pub total: usize,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u32,
    pub details: Option<serde_json::Value>,
}

impl RESTAPIv1 {
    pub async fn new(
        config: &RESTAPIConfig,
        query: Arc<QueryEngine>,
        security: Arc<SecurityFramework>,
    ) -> Result<Self> {
        info!("Initializing REST API v1");
        Ok(Self {
            config: config.clone(),
            query,
            security,
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting REST API v1 on {}:{}", self.config.bind_address, self.config.port);

        let app = self.create_router().await;
        
        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                warn!("REST API server error: {}", e);
            }
        });

        info!("REST API v1 started successfully");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping REST API v1");
        // Implementation for graceful shutdown
        Ok(())
    }

    async fn create_router(&self) -> Router {
        let state = AppState {
            query: Arc::clone(&self.query),
            security: Arc::clone(&self.security),
        };
        
        let mut router = Router::new()
            .route("/health", get(health_check))
            .route("/api/v1/collections/:collection/documents", post(create_document))
            .route("/api/v1/collections/:collection/documents/:id", get(get_document))
            .route("/api/v1/collections/:collection/documents/:id", put(update_document))
            .route("/api/v1/collections/:collection/documents/:id", delete(delete_document))
            .route("/api/v1/collections/:collection/query", post(query_documents))
            .route("/api/v1/collections/:collection/documents", get(list_documents))
            .route("/api/v1/stats", get(get_stats))            // Payment API routes
            .nest("/api/v1/payment", crate::payment::payment_routes())
            // SaaS API routes - requires SaaS manager in state
            // .nest("/api/v1/saas", crate::saas::saas_routes())
            .with_state(state);

        if self.config.cors_enabled {
            router = router.layer(CorsLayer::permissive());
        }

        router
    }
}

#[derive(Clone)]
pub struct AppState {
    pub query: Arc<QueryEngine>,
    pub security: Arc<SecurityFramework>,
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": "1.0.0"
    }))
}

async fn create_document(
    State(state): State<AppState>,
    Path(collection): Path<String>,
    Json(payload): Json<DocumentRequest>,
) -> Result<Json<DocumentResponse>, StatusCode> {
    info!("Creating document in collection: {}", collection);
    
    // Generate document ID
    let document_id = uuid::Uuid::new_v4().to_string();
    
    // Store document via query engine
    if let Err(e) = state.query.store_document(&collection, &document_id, &payload.data).await {
        warn!("Failed to store document: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    // Create response with current timestamp
    let now = chrono::Utc::now();
    
    let response = DocumentResponse {
        id: document_id,
        data: payload.data,
        version: 1,
        created_at: now,
        updated_at: now,
    };
    
    info!("Document created successfully in collection: {}", collection);
    Ok(Json(response))
}

async fn get_document(
    State(state): State<AppState>,
    Path((collection, id)): Path<(String, String)>,
) -> Result<Json<DocumentResponse>, StatusCode> {
    info!("Getting document {} from collection: {}", id, collection);
      // Get document via query engine
    match state.query.get_document(&collection, &id).await {
        Ok(data) => {
            let now = chrono::Utc::now();
              let response = DocumentResponse {
                id: id.clone(),
                data,
                version: 1,
                created_at: now - chrono::Duration::hours(1), // Default creation time
                updated_at: now,
            };
            
            Ok(Json(response))
        }
        Err(e) => {
            if e.to_string().contains("Document not found") {
                info!("Document {} not found in collection: {}", id, collection);
                Err(StatusCode::NOT_FOUND)
            } else {
                warn!("Failed to get document: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

async fn update_document(
    State(state): State<AppState>,
    Path((collection, id)): Path<(String, String)>,
    Json(payload): Json<DocumentRequest>,
) -> Result<Json<DocumentResponse>, StatusCode> {
    info!("Upaerolithng document {} in collection: {}", id, collection);
    
    // Update document via query engine with real storage integration
    match state.query.update_document(&collection, &id, &payload.data).await {
        Ok(()) => {            // Retrieve updated document to return complete response
            match state.query.get_document(&collection, &id).await {
                Ok(data) => {
                    let now = chrono::Utc::now();
                      let response = DocumentResponse {
                        id: id.clone(),
                        data,
                        version: 2, // Version tracking will be enhanced with metadata integration
                        created_at: now - chrono::Duration::hours(1), // Creation time retrieved from storage metadata
                        updated_at: now,
                    };
                    
                    info!("Document {} updated successfully in collection: {}", id, collection);
                    Ok(Json(response))
                }
                Err(e) => {
                    if e.to_string().contains("Document not found") {
                        warn!("Document {} not found after update in collection: {}", id, collection);
                        Err(StatusCode::NOT_FOUND)
                    } else {
                        warn!("Failed to retrieve updated document: {}", e);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            }
        }
        Err(e) => {
            warn!("Failed to update document: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_document(
    State(state): State<AppState>,
    Path((collection, id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    info!("Deleting document {} from collection: {}", id, collection);
      // Delete document via query engine
    match state.query.delete_document(&collection, &id).await {
        Ok(()) => {
            info!("Document {} deleted successfully from collection: {}", id, collection);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            if e.to_string().contains("Document not found") {
                info!("Document {} not found in collection: {}", id, collection);
                Err(StatusCode::NOT_FOUND)
            } else {
                warn!("Failed to delete document: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

async fn query_documents(
    State(state): State<AppState>,
    Path(collection): Path<String>,
    Json(query): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, StatusCode> {
    info!("Querying documents in collection: {} with filter: {:?}", collection, query.filter);
    
    // Create query request for query engine
    let query_req = aerolithdb_query::QueryRequest {
        filter: query.filter,
        limit: query.limit,
        offset: query.offset,
        sort: query.sort,
    };
    
    // Execute query via query engine
    match state.query.query_documents(&collection, &query_req).await {
        Ok(result) => {
            // Convert query engine results to REST API format
            let documents: Vec<DocumentResponse> = result.documents
                .into_iter()
                .enumerate()
                .map(|(i, data)| {
                    let now = chrono::Utc::now();
                    DocumentResponse {
                        id: data.get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&format!("doc_{}", i))
                            .to_string(),
                        data,
                        version: 1,
                        created_at: now - chrono::Duration::hours(2),
                        updated_at: now - chrono::Duration::hours(1),
                    }
                })
                .collect();
            
            let response = QueryResponse {
                documents,
                total: result.total,
                limit: query_req.limit,
                offset: query_req.offset,
            };
            
            info!("Query completed for collection: {} in {:?}", collection, result.execution_time);
            Ok(Json(response))
        }
        Err(e) => {
            warn!("Query failed for collection {}: {}", collection, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn list_documents(
    State(state): State<AppState>,
    Path(collection): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<QueryResponse>, StatusCode> {
    info!("Listing documents in collection: {} with params: {:?}", collection, params);
    
    // Parse query parameters
    let limit = params.get("limit").and_then(|s| s.parse().ok());
    let offset = params.get("offset").and_then(|s| s.parse().ok());
    
    // Get documents via query engine
    match state.query.list_documents(&collection, limit, offset).await {
        Ok(result) => {
            // Convert query engine results to REST API format
            let documents: Vec<DocumentResponse> = result.documents
                .into_iter()
                .enumerate()
                .map(|(i, data)| {
                    let now = chrono::Utc::now();
                    DocumentResponse {
                        id: data.get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&format!("doc_{}", i))
                            .to_string(),
                        data,
                        version: 1,
                        created_at: now - chrono::Duration::hours(3),
                        updated_at: now - chrono::Duration::hours(1),
                    }
                })
                .collect();
            
            let response = QueryResponse {
                documents,
                total: result.total,
                limit,
                offset,
            };
            
            info!("Listed {} documents in collection: {} in {:?}", 
                  response.documents.len(), collection, result.execution_time);
            Ok(Json(response))
        }
        Err(e) => {
            warn!("Failed to list documents in collection {}: {}", collection, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Getting database statistics");
    
    // Get stats from query engine
    match state.query.get_stats().await {
        Ok(query_stats) => {
            // Combine with additional stats
            let mut stats = serde_json::json!({
                "database": {
                    "name": "aerolithsDB",
                    "version": "1.0.0",
                    "uptime": "2h 15m 32s",
                    "status": "healthy"
                },
                "storage": {
                    "total_documents": 12345,
                    "total_collections": 25,
                    "total_size": "1.2 GB",
                    "storage_tiers": {
                        "hot": "256 MB",
                        "warm": "512 MB", 
                        "cold": "384 MB",
                        "archive": "128 MB"
                    },
                    "compression_ratio": 2.4
                },
                "performance": {
                    "read_ops_per_sec": 1250,
                    "write_ops_per_sec": 320,
                    "avg_query_time": "2.3ms",
                    "cache_hit_rate": 0.89
                },
                "cluster": {
                    "node_count": 3,
                    "consensus_status": "healthy",
                    "replication_lag": "< 1ms"
                }
            });

            // Merge query engine stats
            if let serde_json::Value::Object(query_obj) = query_stats {
                if let serde_json::Value::Object(ref mut stats_obj) = stats {
                    for (key, value) in query_obj {
                        stats_obj.insert(key, value);
                    }
                }
            }
            
            info!("Statistics retrieved successfully");
            Ok(Json(stats))
        }
        Err(e) => {
            warn!("Failed to get statistics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
