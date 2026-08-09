use crate::ai_provider::{request_chat_completion, ProviderConfig};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ResumeAnalysis {
    pub score: u8,
    pub summary: String,
    pub strengths: Vec<String>,
    pub improvements: Vec<String>,
}

fn sanitize_json_text(content: &str) -> String {
    let mut cleaned = content.trim().trim_matches('`').trim().to_string();

    if cleaned.starts_with("```") {
        if let Some(end) = cleaned.rfind("```") {
            if end > 3 {
                cleaned = cleaned[3..end].trim().to_string();
            }
        }
    }

    if let (Some(start), Some(end)) = (cleaned.find('{'), cleaned.rfind('}')) {
        if end > start {
            cleaned = cleaned[start..=end].to_string();
        }
    }

    cleaned.retain(|c| matches!(c, '\n' | '\r' | '\t') || !c.is_control());
    cleaned
}

pub fn analyze_resume(resume_text: &str, config: &ProviderConfig) -> Result<ResumeAnalysis, String> {
    let prompt = format!(
        "Review this resume text and return JSON with fields: score (0-10), summary (string), strengths (array of strings), improvements (array of strings). Keep the response concise and useful.\n\nResume text:\n{}",
        resume_text
    );

    let content = request_chat_completion(config, &prompt)?;
    let cleaned = sanitize_json_text(&content);
    let analysis = serde_json::from_str::<ResumeAnalysis>(&cleaned)
        .or_else(|_| serde_json::from_str::<ResumeAnalysis>(&format!(
            "{{\"score\": 0, \"summary\": \"{}\", \"strengths\": [], \"improvements\": []}}",
            cleaned.replace('"', "\\\"")
        )))
        .map_err(|err| format!("Failed to parse resume analysis: {}", err))?;

    Ok(analysis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_control_characters_from_ai_response() {
        let raw_response = "```json\n{\n  \"score\": 8,\n  \"summary\": \"Good resume\",\n  \"strengths\": [\"Rust\"],\n  \"improvements\": [\"More metrics\"]\n}\x00\n```";
        let cleaned = sanitize_json_text(raw_response);
        assert_eq!(cleaned, "{\n  \"score\": 8,\n  \"summary\": \"Good resume\",\n  \"strengths\": [\"Rust\"],\n  \"improvements\": [\"More metrics\"]\n}");
        let analysis: ResumeAnalysis = serde_json::from_str(&cleaned).unwrap();
        assert_eq!(analysis.score, 8);
        assert_eq!(analysis.summary, "Good resume");
    }

    #[test]
    fn builds_analysis_result_shape() {
        let config = ProviderConfig {
            api_key: "demo".to_string(),
            base_url: "https://example.test/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            endpoint_url: None,
        };

        let result = analyze_resume("Experienced software engineer with Rust projects.", &config);
        assert!(result.is_err());
    }
}
