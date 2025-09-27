use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::git::ProjectContext;

pub struct LLMClient {
    model: String,
    api_key: String,
    client: Client,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

impl LLMClient {
    pub fn new(model: String, api_key: String) -> Self {
        Self {
            model,
            api_key,
            client: Client::new(),
        }
    }

    pub async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&ProjectContext>,
        num_suggestions: usize,
    ) -> Result<Vec<String>> {
        if diff.trim().is_empty() {
            return Err(anyhow::anyhow!("No changes to analyze"));
        }

        let prompt = self.build_prompt(diff, context);

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are an expert at writing conventional commit messages. Generate clear, descriptive commit messages following the conventional commits format.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        let request_body = json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": 100,
            "temperature": 0.3,
            "n": num_suggestions,
        });

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to OpenAI API")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        let response_body: OpenAIResponse = response
            .json()
            .await
            .context("Failed to parse OpenAI response")?;

        let messages = response_body
            .choices
            .into_iter()
            .map(|choice| {
                let mut message = choice.message.content.trim().to_string();
                if message.starts_with('"') && message.ends_with('"') {
                    message = message[1..message.len() - 1].to_string();
                }
                message
            })
            .collect();

        Ok(messages)
    }

    fn build_prompt(&self, diff: &str, context: Option<&ProjectContext>) -> String {
        let mut prompt = format!(
            r#"Generate a conventional commit message for the following git diff.

The message should:
- Follow conventional commits format (type: description)
- Be concise but descriptive
- Use appropriate type (feat, fix, docs, style, refactor, test, chore)
- Be written in present tense
- Not exceed 72 characters for the subject line

Git diff:
```
{}
```"#,
            &diff[..diff.len().min(3000)]
        );

        if let Some(ctx) = context {
            prompt.push_str("\n\nProject context:\n");

            if let Some(name) = &ctx.project_name {
                prompt.push_str(&format!("Project: {}\n", name));
            }

            if let Some(readme) = &ctx.readme_content {
                prompt.push_str(&format!(
                    "\nREADME.md (excerpt):\n```\n{}\n```\n",
                    &readme[..readme.len().min(500)]
                ));
            }

            if !ctx.recent_commits.is_empty() {
                prompt.push_str("\nRecent commits:\n");
                for commit in ctx.recent_commits.iter().take(5) {
                    prompt.push_str(&format!("- {}\n", commit));
                }
            }
        }

        prompt.push_str("\n\nGenerate only the commit message, no explanation:");

        prompt
    }
}