"""LLM client for generating commit messages"""

import os
from typing import List, Optional
from openai import OpenAI


class LLMClient:
    """OpenAI API client for generating commit messages"""

    def __init__(self, model: str = "gpt-4o-mini", api_key: Optional[str] = None):
        self.model = model
        self.client = OpenAI(api_key=api_key or os.getenv("OPENAI_API_KEY"))

    def generate_commit_message(
        self,
        diff: str,
        context: dict = None,
        num_suggestions: int = 1
    ) -> List[str]:
        """Generate commit message(s) based on git diff and context"""

        if not diff.strip():
            raise ValueError("No changes to analyze")

        # Build the prompt
        prompt = self._build_prompt(diff, context)

        try:
            response = self.client.chat.completions.create(
                model=self.model,
                messages=[
                    {
                        "role": "system",
                        "content": "You are an expert at writing conventional commit messages. Generate clear, descriptive commit messages following the conventional commits format."
                    },
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                max_tokens=100,
                temperature=0.3,
                n=num_suggestions
            )

            messages = []
            for choice in response.choices:
                message = choice.message.content.strip()
                # Clean up the message - remove quotes if present
                if message.startswith('"') and message.endswith('"'):
                    message = message[1:-1]
                messages.append(message)

            return messages

        except Exception as e:
            raise RuntimeError(f"Failed to generate commit message: {e}")

    def _build_prompt(self, diff: str, context: dict = None) -> str:
        """Build the prompt for the LLM"""

        prompt = f"""Generate a conventional commit message for the following git diff.

The message should:
- Follow conventional commits format (type: description)
- Be concise but descriptive
- Use appropriate type (feat, fix, docs, style, refactor, test, chore)
- Be written in present tense
- Not exceed 72 characters for the subject line

Git diff:
```
{diff[:3000]}  # Limit diff size
```"""

        if context:
            prompt += "\n\nProject context:\n"
            for filename, content in context.items():
                prompt += f"\n{filename}:\n```\n{content[:500]}\n```\n"

        prompt += "\n\nGenerate only the commit message, no explanation:"

        return prompt