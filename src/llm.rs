use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::git::{ProjectContext, StagedChanges};

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
        staged_changes: &StagedChanges,
        context: Option<&ProjectContext>,
        num_suggestions: usize,
    ) -> Result<Vec<String>> {
        if staged_changes.diff_text.trim().is_empty() {
            return Err(anyhow::anyhow!("No changes to analyze"));
        }

        let prompt = self.build_prompt(staged_changes, context);

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a senior software engineer expert at writing conventional commit messages. Analyze code changes deeply to understand the actual purpose and impact. Generate commit messages that clearly explain what changed and why, following conventional commits format. Focus on the business logic and user impact, not just file changes.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        let request_body = json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": 150,
            "temperature": 0.5,
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

    fn build_prompt(&self, staged_changes: &StagedChanges, context: Option<&ProjectContext>) -> String {
        let mut prompt = String::new();

        // Start with file summary
        prompt.push_str("## Files Changed:\n");
        for file in &staged_changes.files_changed {
            prompt.push_str(&format!("- {} ({})\n", file.path, file.status));
        }
        prompt.push_str(&format!("\n## Statistics: +{} insertions, -{} deletions\n\n",
            staged_changes.insertions, staged_changes.deletions));

        // Smart diff inclusion - prioritize important parts
        let diff_text = &staged_changes.diff_text;
        let max_diff_chars = 8000; // Increase from 3000

        prompt.push_str("## Code Changes:\n```diff\n");
        if diff_text.len() <= max_diff_chars {
            prompt.push_str(diff_text);
        } else {
            // Include file headers and key changes
            let mut included_chars = 0;
            let mut in_important_section = false;

            for line in diff_text.lines() {
                // Always include file headers and function definitions
                if line.starts_with("diff --git") ||
                   line.starts_with("+++") ||
                   line.starts_with("---") ||
                   line.starts_with("@@") ||
                   (line.starts_with("+") && (line.contains("fn ") ||
                                               line.contains("def ") ||
                                               line.contains("class ") ||
                                               line.contains("function ") ||
                                               line.contains("interface ") ||
                                               line.contains("struct "))) {
                    prompt.push_str(line);
                    prompt.push('\n');
                    in_important_section = true;
                    included_chars += line.len() + 1;
                } else if included_chars < max_diff_chars {
                    prompt.push_str(line);
                    prompt.push('\n');
                    included_chars += line.len() + 1;
                } else if in_important_section {
                    prompt.push_str("\n... (diff truncated for brevity) ...\n");
                    break;
                }
            }
        }
        prompt.push_str("\n```\n");

        if let Some(ctx) = context {
            prompt.push_str("\n## Project Context:\n");

            if let Some(name) = &ctx.project_name {
                prompt.push_str(&format!("- **Project**: {}\n", name));
            }

            // Detect project type from files
            let project_type = if ctx.cargo_toml.is_some() {
                "Rust"
            } else if ctx.package_json.is_some() {
                "JavaScript/TypeScript"
            } else {
                "General"
            };
            prompt.push_str(&format!("- **Type**: {} project\n", project_type));

            if !ctx.recent_commits.is_empty() {
                prompt.push_str("\n## Recent Commit Style (follow similar patterns):\n");
                for commit in ctx.recent_commits.iter().take(5) {
                    prompt.push_str(&format!("- {}\n", commit));
                }
            }
        }

        prompt.push_str("\n## Instructions:\n");
        prompt.push_str("Generate a conventional commit message that:\n");
        prompt.push_str("1. Uses the appropriate type prefix (feat/fix/docs/style/refactor/test/chore/perf)\n");
        prompt.push_str("2. Has a clear, specific description of WHAT changed and WHY\n");
        prompt.push_str("3. Is under 72 characters for the subject line\n");
        prompt.push_str("4. Uses imperative mood (\"add\" not \"added\" or \"adds\")\n");
        prompt.push_str("5. Does NOT just repeat the file names that changed\n");
        prompt.push_str("6. Focuses on the business logic or user-facing impact\n");
        prompt.push_str("\n**Generate ONLY the commit message, no explanations or quotes:**");

        prompt
    }
}