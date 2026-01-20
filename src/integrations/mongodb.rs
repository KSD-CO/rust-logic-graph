//! MongoDB integration for document operations
//!
//! Provides async NoSQL database operations

use crate::core::Context;
use crate::node::{Node, NodeType};
use crate::rule::{RuleError, RuleResult};
use async_trait::async_trait;
use mongodb::{
    bson::{self, Document},
    Client, Collection,
};
use serde_json::Value;
use tracing::{error, info};

/// MongoDB database node
#[derive(Debug, Clone)]
pub struct MongoNode {
    pub id: String,
    pub database: String,
    pub collection: String,
    pub operation: MongoOperation,
    pub client: Option<Client>,
}

#[derive(Debug, Clone)]
pub enum MongoOperation {
    Find { filter: String },
    FindOne { filter: String },
    Insert { document: String },
    Update { filter: String, update: String },
    Delete { filter: String },
}

impl MongoNode {
    /// Create a new MongoDB find node
    pub fn find(
        id: impl Into<String>,
        database: impl Into<String>,
        collection: impl Into<String>,
        filter: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            database: database.into(),
            collection: collection.into(),
            operation: MongoOperation::Find {
                filter: filter.into(),
            },
            client: None,
        }
    }

    /// Create a new MongoDB find_one node
    pub fn find_one(
        id: impl Into<String>,
        database: impl Into<String>,
        collection: impl Into<String>,
        filter: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            database: database.into(),
            collection: collection.into(),
            operation: MongoOperation::FindOne {
                filter: filter.into(),
            },
            client: None,
        }
    }

    /// Create a new MongoDB insert node
    pub fn insert(
        id: impl Into<String>,
        database: impl Into<String>,
        collection: impl Into<String>,
        document: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            database: database.into(),
            collection: collection.into(),
            operation: MongoOperation::Insert {
                document: document.into(),
            },
            client: None,
        }
    }

    /// Initialize with MongoDB client
    pub async fn with_client(mut self, mongodb_url: &str) -> Result<Self, RuleError> {
        let client = Client::with_uri_str(mongodb_url)
            .await
            .map_err(|e| RuleError::Eval(format!("Failed to connect to MongoDB: {}", e)))?;
        self.client = Some(client);
        Ok(self)
    }

    /// Get collection
    fn get_collection(&self) -> Result<Collection<Document>, RuleError> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| RuleError::Eval("MongoDB client not initialized".to_string()))?;

        Ok(client.database(&self.database).collection(&self.collection))
    }

    /// Execute MongoDB operation
    async fn execute_operation(&self, ctx: &Context) -> Result<Value, RuleError> {
        let collection = self.get_collection()?;

        match &self.operation {
            MongoOperation::Find { filter } => {
                let processed_filter = self.process_json(filter, ctx)?;
                let filter_doc = self.json_to_document(&processed_filter)?;

                info!("MongoNode[{}]: FIND with filter: {:?}", self.id, filter_doc);

                use futures::stream::TryStreamExt;

                let cursor = collection
                    .find(filter_doc, None)
                    .await
                    .map_err(|e| RuleError::Eval(format!("Find failed: {}", e)))?;

                let docs: Vec<Document> = cursor
                    .try_collect()
                    .await
                    .map_err(|e| RuleError::Eval(format!("Failed to collect results: {}", e)))?;

                let mut results = Vec::new();
                for doc in docs {
                    if let Ok(json) = bson::to_bson(&doc) {
                        if let Ok(value) = serde_json::to_value(json) {
                            results.push(value);
                        }
                    }
                }

                Ok(Value::Array(results))
            }

            MongoOperation::FindOne { filter } => {
                let processed_filter = self.process_json(filter, ctx)?;
                let filter_doc = self.json_to_document(&processed_filter)?;

                info!(
                    "MongoNode[{}]: FIND_ONE with filter: {:?}",
                    self.id, filter_doc
                );

                let result = collection
                    .find_one(filter_doc, None)
                    .await
                    .map_err(|e| RuleError::Eval(format!("FindOne failed: {}", e)))?;

                match result {
                    Some(doc) => {
                        let bson_val = bson::to_bson(&doc).map_err(|e| {
                            RuleError::Eval(format!("BSON conversion failed: {}", e))
                        })?;
                        let json_val = serde_json::to_value(bson_val).map_err(|e| {
                            RuleError::Eval(format!("JSON conversion failed: {}", e))
                        })?;
                        Ok(json_val)
                    }
                    None => Ok(Value::Null),
                }
            }

            MongoOperation::Insert { document } => {
                let processed_doc = self.process_json(document, ctx)?;
                let doc = self.json_to_document(&processed_doc)?;

                info!("MongoNode[{}]: INSERT document: {:?}", self.id, doc);

                let result = collection
                    .insert_one(doc, None)
                    .await
                    .map_err(|e| RuleError::Eval(format!("Insert failed: {}", e)))?;

                Ok(Value::String(result.inserted_id.to_string()))
            }

            MongoOperation::Update { filter, update } => {
                let processed_filter = self.process_json(filter, ctx)?;
                let processed_update = self.process_json(update, ctx)?;
                let filter_doc = self.json_to_document(&processed_filter)?;
                let update_doc = self.json_to_document(&processed_update)?;

                info!(
                    "MongoNode[{}]: UPDATE filter: {:?}, update: {:?}",
                    self.id, filter_doc, update_doc
                );

                let result = collection
                    .update_many(filter_doc, update_doc, None)
                    .await
                    .map_err(|e| RuleError::Eval(format!("Update failed: {}", e)))?;

                Ok(Value::Number(result.modified_count.into()))
            }

            MongoOperation::Delete { filter } => {
                let processed_filter = self.process_json(filter, ctx)?;
                let filter_doc = self.json_to_document(&processed_filter)?;

                info!("MongoNode[{}]: DELETE filter: {:?}", self.id, filter_doc);

                let result = collection
                    .delete_many(filter_doc, None)
                    .await
                    .map_err(|e| RuleError::Eval(format!("Delete failed: {}", e)))?;

                Ok(Value::Number(result.deleted_count.into()))
            }
        }
    }

    fn process_json(&self, json_str: &str, ctx: &Context) -> Result<Value, RuleError> {
        let mut processed = json_str.to_string();

        for (key, value) in &ctx.data {
            let placeholder = format!("{{{{{}}}}}", key);
            if processed.contains(&placeholder) {
                let replacement = match value {
                    Value::String(s) => format!("\"{}\"", s),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => continue,
                };
                processed = processed.replace(&placeholder, &replacement);
            }
        }

        serde_json::from_str(&processed)
            .map_err(|e| RuleError::Eval(format!("JSON parse failed: {}", e)))
    }

    fn json_to_document(&self, json: &Value) -> Result<Document, RuleError> {
        let bson_val = bson::to_bson(json)
            .map_err(|e| RuleError::Eval(format!("BSON conversion failed: {}", e)))?;

        if let bson::Bson::Document(doc) = bson_val {
            Ok(doc)
        } else {
            Err(RuleError::Eval("Expected BSON document".to_string()))
        }
    }
}

#[async_trait]
impl Node for MongoNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::DBNode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!("MongoNode[{}]: Starting operation", self.id);

        match self.execute_operation(ctx).await {
            Ok(result) => {
                info!("MongoNode[{}]: Operation successful", self.id);
                ctx.data
                    .insert(format!("{}_result", self.id), result.clone());
                Ok(result)
            }
            Err(e) => {
                error!("MongoNode[{}]: Operation failed: {}", self.id, e);
                Err(e)
            }
        }
    }
}
