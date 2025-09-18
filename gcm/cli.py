"""Command line interface for gcm"""

import sys
import click
from typing import Optional

from .git_utils import GitAnalyzer
from .llm_client import LLMClient
from .config import Config


@click.command()
@click.option("-c", "--commit", is_flag=True, help="Generate message and commit")
@click.option("-a", "--all", is_flag=True, help="Stage all changes first")
@click.option("-n", "--number", default=1, help="Number of message suggestions")
@click.option("--amend", is_flag=True, help="Amend the previous commit")
@click.option("--model", help="Override the LLM model to use")
@click.version_option()
def main(commit: bool, all: bool, number: int, amend: bool, model: Optional[str]):
    """Generate intelligent git commit messages using LLMs"""

    try:
        # Load configuration
        config = Config()

        # Override model if specified
        if model:
            config.set("model", model)

        # Initialize git analyzer
        git_analyzer = GitAnalyzer()

        # Stage all changes if requested
        if all:
            click.echo("Staging all changes...")
            git_analyzer.stage_all_changes()

        # Check for staged changes
        if not git_analyzer.has_staged_changes():
            click.echo("No staged changes found. Use 'git add' to stage changes or use -a flag.", err=True)
            sys.exit(1)

        # Get diff and context
        diff = git_analyzer.get_staged_diff()
        context = git_analyzer.get_project_context()

        # Initialize LLM client
        api_key = config.get("api_key")
        if not api_key:
            click.echo("Error: OpenAI API key not found. Set OPENAI_API_KEY environment variable or configure in .gcmrc", err=True)
            sys.exit(1)

        llm_client = LLMClient(
            model=config.get("model"),
            api_key=api_key
        )

        # Generate commit messages
        click.echo("Generating commit message...")
        try:
            messages = llm_client.generate_commit_message(
                diff=diff,
                context=context,
                num_suggestions=number
            )
        except Exception as e:
            click.echo(f"Error generating commit message: {e}", err=True)
            sys.exit(1)

        # Display messages
        if number == 1:
            message = messages[0]
            click.echo(f"\nGenerated commit message:")
            click.echo(f"  {message}")

            # Commit if requested
            if commit:
                try:
                    git_analyzer.commit_with_message(message)
                    click.echo(f"\nCommitted with message: {message}")
                except Exception as e:
                    click.echo(f"Error committing: {e}", err=True)
                    sys.exit(1)
        else:
            click.echo(f"\nGenerated {len(messages)} commit message suggestions:")
            for i, message in enumerate(messages, 1):
                click.echo(f"  {i}. {message}")

    except ValueError as e:
        click.echo(f"Error: {e}", err=True)
        sys.exit(1)
    except Exception as e:
        click.echo(f"Unexpected error: {e}", err=True)
        sys.exit(1)


if __name__ == "__main__":
    main()