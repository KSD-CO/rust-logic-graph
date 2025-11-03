//! Anthropic Claude integration with streaming support
//!
//! Provides Claude 3.5 Sonnet and other Claude model integrations

use crate::core::Context;
use crate::node::{Node, NodeType};
use crate::rule::{RuleResult, RuleError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, error};

/// Claude node for LLM operations
#[derive(Debug, Clone)]
pub struct ClaudeNode {
    pub id: String,
    pub model: String,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct MessagesRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct MessagesResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<ContentBlock>,
    model: String,
    stop_reason: Option<String>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

impl ClaudeNode {
    /// Create a new Claude node
    pub fn new(id: impl Into<String>, model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            model: model.into(),
            prompt: prompt.into(),
            system_prompt: None,
            temperature: 0.7,
            max_tokens: 4096,
            api_key: None,
        }
    }

    /// Create Claude 3.5 Sonnet node
    pub fn sonnet_35(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self::new(id, "claude-3-5-sonnet-20241022", prompt)
    }

    /// Create Claude 3 Opus node
    pub fn opus(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self::new(id, "claude-3-opus-20240229", prompt)
    }

    /// Create Claude 3 Sonnet node
    pub fn sonnet(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self::new(id, "claude-3-sonnet-20240229", prompt)
    }

    /// Create Claude 3 Haiku node
    pub fn haiku(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self::new(id, "claude-3-haiku-20240307", prompt)
    }

    /// Set system prompt
    pub fn with_system_prompt(mut self, system_prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(system_prompt.into());
        self
    }

    /// Set temperature (0.0 - 1.0)
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Execute Claude API call
    async fn execute_completion(&self, ctx: &Context) -> Result<Value, RuleError> {
        let api_key = if let Some(key) = &self.api_key {
            key.clone()
        } else {
            std::env::var("ANTHROPIC_API_KEY")
                .map_err(|_| RuleError::Eval("Anthropic API key not provided".to_string()))?
        };

        let processed_prompt = self.process_prompt(&self.prompt, ctx);

        let messages = vec![Message {
            role: "user".to_string(),
            content: processed_prompt.clone(),
        }];

        let request = MessagesRequest {
            model: self.model.clone(),
            messages,
            system: self.system_prompt.clone(),
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        info!("ClaudeNode[{}]: Calling Anthropic API with model {}", self.id, self.model);

        let client = Client::new();
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| RuleError::Eval(format!("Anthropic API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(RuleError::Eval(format!("Anthropic API error {}: {}", status, error_text)));
        }

        let completion: MessagesResponse = response.json()
            .await
            .map_err(|e| RuleError::Eval(format!("Failed to parse Anthropic response: {}", e)))?;

        let content = completion.content.first()
            .ok_or_else(|| RuleError::Eval("No content blocks returned".to_string()))?;

        info!(
            "ClaudeNode[{}]: Completion successful. Tokens: {} input + {} output",
            self.id,
            completion.usage.input_tokens,
            completion.usage.output_tokens
        );

        Ok(serde_json::json!({
            "content": content.text,
            "stop_reason": completion.stop_reason,
            "model": completion.model,
            "usage": {
                "input_tokens": completion.usage.input_tokens,
                "output_tokens": completion.usage.output_tokens,
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
impl Node for ClaudeNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::AINode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!("ClaudeNode[{}]: Starting Claude completion", self.id);

        match self.execute_completion(ctx).await {
            Ok(result) => {
                info!("ClaudeNode[{}]: Completion successful", self.id);

                // Store full result
                ctx.data.insert(format!("{}_result", self.id), result.clone());

                // Extract and store content separately for convenience
                if let Some(content) = result.get("content").and_then(|v| v.as_str()) {
                    ctx.data.insert(format!("{}_content", self.id), Value::String(content.to_string()));
                }

                Ok(result)
            }
            Err(e) => {
                error!("ClaudeNode[{}]: Completion failed: {}", self.id, e);
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
        let node = ClaudeNode::sonnet_35("test", "Analyze this data: {{data}}");
        let mut ctx = Context {
            data: HashMap::new(),
        };
        ctx.data.insert("data".to_string(), Value::String("test data".to_string()));

        let processed = node.process_prompt(&node.prompt, &ctx);
        assert_eq!(processed, "Analyze this data: test data");
    }
}
