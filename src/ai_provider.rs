use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub endpoint_url: Option<String>,
}

impl ProviderConfig {
    pub fn from_env() -> Result<Self, String> {
        let api_key = std::env::var("API_KEY").map_err(|_| {
            "Missing API_KEY environment variable. Set API_KEY to an OpenAI-compatible token or Gemini key.".to_string()
        })?;

        let base_url = std::env::var("AI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let endpoint_url = std::env::var("AI_ENDPOINT").ok();
        let model = std::env::var("AI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());

        if api_key.starts_with("AQ.") && endpoint_url.is_none() && base_url == "https://api.openai.com/v1" {
            return Err("Detected a Gemini-style API key. Please set AI_ENDPOINT to the correct Gemini-compatible chat endpoint or set AI_BASE_URL to your Gemini gateway URL.".to_string());
        }

        Ok(Self {
            api_key,
            base_url,
            model,
            endpoint_url,
        })
    }
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChoiceResponse>,
}

#[derive(Debug, Deserialize)]
struct ChoiceResponse {
    message: MessageResponse,
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
    content: String,
}

pub fn parse_chat_response(body: &str) -> Result<String, String> {
    let response: ChatCompletionResponse = serde_json::from_str(body).map_err(|err| {
        format!("Invalid provider response: {}", err)
    })?;

    response
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
        .ok_or_else(|| "Provider response did not include any choices.".to_string())
}

fn build_request_url(config: &ProviderConfig) -> String {
    let base_url = config
        .endpoint_url
        .as_deref()
        .unwrap_or_else(|| config.base_url.as_str())
        .trim_end_matches('/');

    if base_url.ends_with("chat/completions") {
        base_url.to_string()
    } else {
        format!("{}/chat/completions", base_url)
    }
}

pub fn request_chat_completion(config: &ProviderConfig, prompt: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let url = build_request_url(config);

    let payload = json!({
        "model": config.model,
        "messages": [
            {"role": "system", "content": "You are a resume review assistant."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.2
    });

    let response = client
        .post(&url)
        .bearer_auth(&config.api_key)
        .json(&payload)
        .send()
        .map_err(|err| format!("Failed to contact AI provider: {}", err))?;

    let status = response.status();
    let body = response.text().map_err(|err| format!("Failed to read provider response: {}", err))?;

    if !status.is_success() {
        return Err(format!("AI provider returned {}: {}", status, body));
    }

    parse_chat_response(&body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_chat_completion_content() {
        let body = r#"{"choices":[{"message":{"content":"Hello from AI"}}]}"#;
        let content = parse_chat_response(body).unwrap();
        assert_eq!(content, "Hello from AI");
    }

    #[test]
    fn loads_provider_config_from_environment() {
        std::env::set_var("API_KEY", "abc123");
        std::env::set_var("AI_BASE_URL", "https://example.test/v1");
        std::env::set_var("AI_MODEL", "gpt-4o-mini");

        let config = ProviderConfig::from_env().unwrap();
        assert_eq!(config.api_key, "abc123");
        assert_eq!(config.base_url, "https://example.test/v1");
        assert_eq!(config.model, "gpt-4o-mini");

        std::env::remove_var("API_KEY");
        std::env::remove_var("AI_BASE_URL");
        std::env::remove_var("AI_MODEL");
    }
}
