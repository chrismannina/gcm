"""Git operations and diff analysis utilities"""

import subprocess
from typing import Optional, Tuple
from git import Repo, InvalidGitRepositoryError


class GitAnalyzer:
    """Handles git operations and diff analysis"""

    def __init__(self, repo_path: str = "."):
        try:
            self.repo = Repo(repo_path)
        except InvalidGitRepositoryError:
            raise ValueError(f"Not a git repository: {repo_path}")

    def get_staged_diff(self) -> str:
        """Get the staged changes diff"""
        try:
            # Use git command directly for better diff output
            result = subprocess.run(
                ["git", "diff", "--staged"],
                capture_output=True,
                text=True,
                cwd=self.repo.working_dir
            )
            return result.stdout
        except subprocess.CalledProcessError as e:
            raise RuntimeError(f"Failed to get staged diff: {e}")

    def has_staged_changes(self) -> bool:
        """Check if there are any staged changes"""
        diff = self.get_staged_diff()
        return bool(diff.strip())

    def stage_all_changes(self) -> None:
        """Stage all changes (git add .)"""
        try:
            self.repo.git.add(".")
        except Exception as e:
            raise RuntimeError(f"Failed to stage changes: {e}")

    def commit_with_message(self, message: str) -> None:
        """Create a commit with the given message"""
        try:
            self.repo.index.commit(message)
        except Exception as e:
            raise RuntimeError(f"Failed to commit: {e}")

    def get_project_context(self) -> dict:
        """Get project context from common files"""
        context = {}

        # Try to read common context files
        context_files = ["README.md", "package.json", "pyproject.toml", "setup.py"]

        for filename in context_files:
            try:
                with open(f"{self.repo.working_dir}/{filename}", "r") as f:
                    context[filename] = f.read()[:1000]  # Limit to first 1000 chars
            except FileNotFoundError:
                continue
            except Exception:
                continue

        return context