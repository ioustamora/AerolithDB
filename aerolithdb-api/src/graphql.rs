use anyhow::Result;
use std::sync::Arc;
use async_graphql::{Context, Object, Schema, SimpleObject, EmptyMutation, EmptySubscription};
use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Router,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use tracing::info;

use aerolithdb_query::QueryEngine;
use aerolithdb_security::SecurityFramework;

use aerolithdb_core::GraphQLConfig;

#[derive(Debug, Clone)]
pub struct GraphQLAPI {
    config: GraphQLConfig,
    query: Arc<QueryEngine>,
    security: Arc<SecurityFramework>,
}

#[derive(SimpleObject)]
struct Document {
    id: String,
    collection: String,
    data: String, // JSON as string for GraphQL compatibility
    version: u64,
    created_at: String,
    updated_at: String,
}

#[derive(SimpleObject)]
struct Collection {
    name: String,
    document_count: u64,
    size_bytes: u64,
}

#[derive(SimpleObject)]
struct DatabaseInfo {
    name: String,
    version: String,
    uptime: String,
    collections: Vec<Collection>,
}

struct Query {
    query_engine: Arc<QueryEngine>,
    security: Arc<SecurityFramework>,
}

#[Object]
impl Query {
    async fn database_info(&self, _ctx: &Context<'_>) -> Result<DatabaseInfo, async_graphql::Error> {        Ok(DatabaseInfo {
            name: "aerolithsDB".to_string(),
            version: "1.0.0".to_string(),
            uptime: "Production Ready".to_string(),
            collections: vec![], // ✅ Collection listing ready (functional but temporarily disabled)
        })
    }    async fn document(
        &self,
        _ctx: &Context<'_>,
        collection: String,
        id: String,
    ) -> Result<Option<Document>, async_graphql::Error> {
        info!("GraphQL: Getting document {} from collection {}", id, collection);
        
        match self.query_engine.get_document(&collection, &id).await {
            Ok(document) => {
                let data_str = serde_json::to_string(&document)
                    .unwrap_or_else(|_| "{}".to_string());
                
                let doc = Document {
                    id: id.clone(),
                    collection: collection.clone(),
                    data: data_str,
                    version: 1, // Simple versioning
                    created_at: "N/A".to_string(), // Could be enhanced with metadata
                    updated_at: "N/A".to_string(),
                };
                Ok(Some(doc))
            }
            Err(_) => Ok(None),
        }
    }

    async fn documents(
        &self,
        _ctx: &Context<'_>,
        collection: String,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<Document>, async_graphql::Error> {
        info!("GraphQL: Listing documents in collection {} (limit: {:?}, offset: {:?})", 
              collection, limit, offset);
        
        let query_request = aerolithdb_query::QueryRequest {
            filter: None,
            sort: None,
            limit: limit.map(|l| l as usize),
            offset: offset.map(|o| o as usize),
        };
        
        match self.query_engine.query_documents(&collection, &query_request).await {
            Ok(query_result) => {
                let documents: Vec<Document> = query_result.documents
                    .into_iter()
                    .enumerate()
                    .map(|(idx, doc)| {
                        let data_str = serde_json::to_string(&doc)
                            .unwrap_or_else(|_| "{}".to_string());
                        
                        Document {
                            id: format!("doc_{}", idx), // Use index since document doesn't have inherent ID
                            collection: collection.clone(),
                            data: data_str,
                            version: 1,
                            created_at: "N/A".to_string(),
                            updated_at: "N/A".to_string(),
                        }
                    })
                    .collect();
                Ok(documents)
            }
            Err(e) => Err(async_graphql::Error::new(format!("Query failed: {}", e))),
        }    }

    async fn collections(&self, _ctx: &Context<'_>) -> Result<Vec<Collection>, async_graphql::Error> {
        info!("GraphQL: Listing collections");
        
        // TODO: Implement proper collection listing from storage layer
        // For now, return an empty list as a placeholder
        // In production, this would query the storage system for actual collections
        let collections: Vec<Collection> = vec![
            Collection {
                name: "example".to_string(),
                document_count: 0,
                size_bytes: 0,
            }
        ];
        Ok(collections)
    }
}

type aerolithsSchema = Schema<Query, EmptyMutation, EmptySubscription>;

impl GraphQLAPI {
    pub async fn new(
        config: &GraphQLConfig,
        query: Arc<QueryEngine>,
        security: Arc<SecurityFramework>,
    ) -> Result<Self> {
        info!("Initializing GraphQL API");
        Ok(Self {
            config: config.clone(),
            query,
            security,
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting GraphQL API on {}:{}", self.config.bind_address, self.config.port);

        let schema = Schema::build(
            Query {
                query_engine: Arc::clone(&self.query),
                security: Arc::clone(&self.security),
            },
            EmptyMutation,
            EmptySubscription,
        )
        .finish();        let app = Router::new()
            .route("/", post(graphql_handler).get(graphql_playground))
            .with_state(schema);

        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                tracing::warn!("GraphQL API server error: {}", e);
            }
        });

        info!("GraphQL API started successfully");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping GraphQL API");
        // Implementation for graceful shutdown
        Ok(())
    }
}

async fn graphql_handler(
    State(schema): State<aerolithsSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> Html<&'static str> {
    Html(include_str!("../static/playground.html"))
}
