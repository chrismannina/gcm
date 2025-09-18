"""Configuration management"""

import os
import yaml
from pathlib import Path
from typing import Dict, Any, Optional
from dotenv import load_dotenv


class Config:
    """Configuration manager for gcm"""

    DEFAULT_CONFIG = {
        "model": "gpt-4o-mini",
        "max_tokens": 100,
        "context": ["README.md"],
        "api_key": None
    }

    def __init__(self):
        self.config = self.DEFAULT_CONFIG.copy()
        # Load .env file first
        load_dotenv()
        self._load_config()

    def _load_config(self):
        """Load configuration from files"""
        # Check for project-level config
        project_config = Path(".gcm.yml")
        if project_config.exists():
            self._merge_config_file(project_config)

        # Check for user-level config
        user_config = Path.home() / ".gcmrc"
        if user_config.exists():
            self._merge_config_file(user_config)

        # Override with environment variables
        # Check for OpenAI API key
        if os.getenv("OPENAI_API_KEY"):
            self.config["api_key"] = os.getenv("OPENAI_API_KEY")
        # Also check for Anthropic API key as an alternative
        elif os.getenv("ANTHROPIC_API_KEY"):
            self.config["api_key"] = os.getenv("ANTHROPIC_API_KEY")
            self.config["provider"] = "anthropic"

    def _merge_config_file(self, config_path: Path):
        """Merge configuration from a YAML file"""
        try:
            with open(config_path, 'r') as f:
                file_config = yaml.safe_load(f) or {}
                self.config.update(file_config)
        except Exception as e:
            print(f"Warning: Could not load config from {config_path}: {e}")

    def get(self, key: str, default=None):
        """Get configuration value"""
        return self.config.get(key, default)

    def set(self, key: str, value: Any):
        """Set configuration value"""
        self.config[key] = value