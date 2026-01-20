//! Ollama local LLM integration
//!
//! Provides integration with locally running Ollama models

use crate::core::Context;
use crate::node::{Node, NodeType};
use crate::rule::{RuleError, RuleResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info};

/// Ollama node for local LLM operations
#[derive(Debug, Clone)]
pub struct OllamaNode {
    pub id: String,
    pub model: String,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub base_url: String,
}

#[derive(Debug, Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<GenerateOptions>,
}

#[derive(Debug, Serialize)]
struct GenerateOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    model: String,
    response: String,
    done: bool,
    #[serde(default)]
    total_duration: Option<u64>,
    #[serde(default)]
    eval_count: Option<u32>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
}

impl OllamaNode {
    /// Create a new Ollama node
    pub fn new(id: impl Into<String>, model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            model: model.into(),
            prompt: prompt.into(),
            system_prompt: None,
            temperature: None,
            base_url: "http://localhost:11434".to_string(),
        }
    }

    /// Create Llama 3.1 node
    pub fn llama31(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self::new(id, "llama3.1", prompt)
    }

    /// Create Llama 2 node
    pub fn llama2(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self::new(id, "llama2", prompt)
    }

    /// Create Mistral node
    pub fn mistral(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self::new(id, "mistral", prompt)
    }

    /// Create CodeLlama node
    pub fn codellama(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self::new(id, "codellama", prompt)
    }

    /// Set system prompt
    pub fn with_system_prompt(mut self, system_prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(system_prompt.into());
        self
    }

    /// Set temperature (0.0 - 2.0)
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set base URL (default: http://localhost:11434)
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Execute Ollama API call
    async fn execute_completion(&self, ctx: &Context) -> Result<Value, RuleError> {
        let processed_prompt = self.process_prompt(&self.prompt, ctx);

        let options = if self.temperature.is_some() {
            Some(GenerateOptions {
                temperature: self.temperature,
            })
        } else {
            None
        };

        let request = GenerateRequest {
            model: self.model.clone(),
            prompt: processed_prompt.clone(),
            system: self.system_prompt.clone(),
            stream: false,
            options,
        };

        info!(
            "OllamaNode[{}]: Calling Ollama API with model {}",
            self.id, self.model
        );

        let client = Client::new();
        let url = format!("{}/api/generate", self.base_url);

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| RuleError::Eval(format!("Ollama API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(RuleError::Eval(format!(
                "Ollama API error {}: {}",
                status, error_text
            )));
        }

        let completion: GenerateResponse = response
            .json()
            .await
            .map_err(|e| RuleError::Eval(format!("Failed to parse Ollama response: {}", e)))?;

        let duration_ms = completion
            .total_duration
            .map(|d| d / 1_000_000)
            .unwrap_or(0);

        info!(
            "OllamaNode[{}]: Completion successful. Duration: {}ms, Eval tokens: {:?}, Prompt tokens: {:?}",
            self.id,
            duration_ms,
            completion.eval_count,
            completion.prompt_eval_count
        );

        Ok(serde_json::json!({
            "content": completion.response,
            "model": completion.model,
            "done": completion.done,
            "stats": {
                "duration_ms": duration_ms,
                "eval_count": completion.eval_count,
                "prompt_eval_count": completion.prompt_eval_count,
            }
        }))
    }

    fn process_prompt(&self, prompt: &str, ctx: &Context) -> String {
        let mut processed = prompt.to_string();

        for (key, value) in &ctx.data {
            let placeholder = format!("{{{{{}}}}}", key);
            if processed.contains(&placeholder) {
                let replacement = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Array(a) => serde_json::to_string(a).unwrap_or_default(),
                    Value::Object(o) => serde_json::to_string(o).unwrap_or_default(),
                    Value::Null => "null".to_string(),
                };
                processed = processed.replace(&placeholder, &replacement);
            }
        }

        processed
    }
}

#[async_trait]
impl Node for OllamaNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::AINode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!("OllamaNode[{}]: Starting Ollama completion", self.id);

        match self.execute_completion(ctx).await {
            Ok(result) => {
                info!("OllamaNode[{}]: Completion successful", self.id);

                // Store full result
                ctx.data
                    .insert(format!("{}_result", self.id), result.clone());

                // Extract and store content separately for convenience
                if let Some(content) = result.get("content").and_then(|v| v.as_str()) {
                    ctx.data.insert(
                        format!("{}_content", self.id),
                        Value::String(content.to_string()),
                    );
                }

                Ok(result)
            }
            Err(e) => {
                error!("OllamaNode[{}]: Completion failed: {}", self.id, e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_prompt_processing() {
        let node = OllamaNode::llama31("test", "Analyze this data: {{data}}");
        let mut ctx = Context {
            data: HashMap::new(),
        };
        ctx.data
            .insert("data".to_string(), Value::String("test data".to_string()));

        let processed = node.process_prompt(&node.prompt, &ctx);
        assert_eq!(processed, "Analyze this data: test data");
    }
}
