use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Trait for LLM providers - allows swapping Claude for other providers.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a prompt to the LLM and get a response.
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse>;

    /// Provider name for logging.
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub system_prompt: String,
    pub user_message: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub tokens_used: Option<u32>,
    pub stop_reason: Option<String>,
}

/// Claude API provider (default).
pub struct ClaudeProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl ClaudeProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "claude-sonnet-4-6".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
            "system": request.system_prompt,
            "messages": [{
                "role": "user",
                "content": request.user_message
            }]
        });

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send request to Claude API")?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            anyhow::bail!("Claude API error ({}): {}", status, response_text);
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&response_text).context("Failed to parse Claude response")?;

        let content = parsed["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let tokens_used = parsed["usage"]["output_tokens"].as_u64().map(|t| t as u32);
        let stop_reason = parsed["stop_reason"].as_str().map(|s| s.to_string());

        Ok(LlmResponse {
            content,
            tokens_used,
            stop_reason,
        })
    }

    fn name(&self) -> &str {
        "Claude"
    }
}

/// Mock LLM provider for testing.
pub struct MockLlmProvider {
    responses: std::sync::Mutex<Vec<String>>,
}

impl MockLlmProvider {
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses: std::sync::Mutex::new(responses),
        }
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    async fn complete(&self, _request: &LlmRequest) -> Result<LlmResponse> {
        let mut responses = self.responses.lock().unwrap();
        let content = if responses.is_empty() {
            "// No mock response available".to_string()
        } else {
            responses.remove(0)
        };

        Ok(LlmResponse {
            content,
            tokens_used: Some(100),
            stop_reason: Some("end_turn".to_string()),
        })
    }

    fn name(&self) -> &str {
        "Mock"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockLlmProvider::new(vec!["fn main() { println!(\"Hello\"); }".to_string()]);

        let request = LlmRequest {
            system_prompt: "Convert Perl to Rust".to_string(),
            user_message: "print 'Hello';".to_string(),
            max_tokens: 1000,
            temperature: 0.0,
        };

        let response = provider.complete(&request).await.unwrap();
        assert!(response.content.contains("fn main"));
    }

    #[tokio::test]
    async fn test_mock_provider_empty() {
        let provider = MockLlmProvider::new(vec![]);
        let request = LlmRequest {
            system_prompt: String::new(),
            user_message: String::new(),
            max_tokens: 100,
            temperature: 0.0,
        };
        let response = provider.complete(&request).await.unwrap();
        assert!(response.content.contains("No mock response"));
    }
}
