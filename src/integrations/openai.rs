//! OpenAI integration with streaming support
//!
//! Provides GPT-4 and other OpenAI model integrations

use crate::core::Context;
use crate::node::{Node, NodeType};
use crate::rule::{RuleResult, RuleError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, error};

/// OpenAI node for LLM operations
#[derive(Debug, Clone)]
pub struct OpenAINode {
    pub id: String,
    pub model: String,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl OpenAINode {
    /// Create a new OpenAI node
    pub fn new(id: impl Into<String>, model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            model: model.into(),
            prompt: prompt.into(),
            system_prompt: None,
            temperature: 0.7,
            max_tokens: None,
            api_key: None,
        }
    }

    /// Create GPT-4 node
    pub fn gpt4(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self::new(id, "gpt-4", prompt)
    }

    /// Create GPT-4 Turbo node
    pub fn gpt4_turbo(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self::new(id, "gpt-4-turbo-preview", prompt)
    }

    /// Create GPT-3.5 Turbo node
    pub fn gpt35_turbo(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self::new(id, "gpt-3.5-turbo", prompt)
    }

    /// Set system prompt
    pub fn with_system_prompt(mut self, system_prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(system_prompt.into());
        self
    }

    /// Set temperature (0.0 - 2.0)
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Execute OpenAI API call
    async fn execute_completion(&self, ctx: &Context) -> Result<Value, RuleError> {
        let api_key = if let Some(key) = &self.api_key {
            key.clone()
        } else {
            std::env::var("OPENAI_API_KEY")
                .map_err(|_| RuleError::Eval("OpenAI API key not provided".to_string()))?
        };

        let processed_prompt = self.process_prompt(&self.prompt, ctx);

        let mut messages = Vec::new();

        if let Some(system_prompt) = &self.system_prompt {
            messages.push(Message {
                role: "system".to_string(),
                content: system_prompt.clone(),
            });
        }

        messages.push(Message {
            role: "user".to_string(),
            content: processed_prompt.clone(),
        });

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        info!("OpenAINode[{}]: Calling OpenAI API with model {}", self.id, self.model);

        let client = Client::new();
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| RuleError::Eval(format!("OpenAI API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(RuleError::Eval(format!("OpenAI API error {}: {}", status, error_text)));
        }

        let completion: ChatCompletionResponse = response.json()
            .await
            .map_err(|e| RuleError::Eval(format!("Failed to parse OpenAI response: {}", e)))?;

        let choice = completion.choices.first()
            .ok_or_else(|| RuleError::Eval("No completion choices returned".to_string()))?;

        info!(
            "OpenAINode[{}]: Completion successful. Tokens: {} prompt + {} completion = {} total",
            self.id,
            completion.usage.prompt_tokens,
            completion.usage.completion_tokens,
            completion.usage.total_tokens
        );

        Ok(serde_json::json!({
            "content": choice.message.content,
            "finish_reason": choice.finish_reason,
            "usage": {
                "prompt_tokens": completion.usage.prompt_tokens,
                "completion_tokens": completion.usage.completion_tokens,
                "total_tokens": completion.usage.total_tokens,
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
impl Node for OpenAINode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::AINode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!("OpenAINode[{}]: Starting OpenAI completion", self.id);

        match self.execute_completion(ctx).await {
            Ok(result) => {
                info!("OpenAINode[{}]: Completion successful", self.id);

                // Store full result
                ctx.data.insert(format!("{}_result", self.id), result.clone());

                // Extract and store content separately for convenience
                if let Some(content) = result.get("content").and_then(|v| v.as_str()) {
                    ctx.data.insert(format!("{}_content", self.id), Value::String(content.to_string()));
                }

                Ok(result)
            }
            Err(e) => {
                error!("OpenAINode[{}]: Completion failed: {}", self.id, e);
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
        let node = OpenAINode::new("test", "gpt-4", "Analyze this data: {{data}}");
        let mut ctx = Context {
            data: HashMap::new(),
        };
        ctx.data.insert("data".to_string(), Value::String("test data".to_string()));

        let processed = node.process_prompt(&node.prompt, &ctx);
        assert_eq!(processed, "Analyze this data: test data");
    }
}
