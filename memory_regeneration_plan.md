# Plan to Add Memory and Regeneration Features to gcm

## Overview
Enhance gcm with a memory/cache system to store generated commit messages and provide better regeneration options with additional context.

## Implementation Steps

### 1. Create Cache Manager Module (`gcm/cache.py`)
- Store generated messages with metadata (diff hash, timestamp, messages)
- Cache keyed by hash of staged diff + commit SHA
- Temporary file storage in `~/.gcm/cache/` or `.git/gcm-cache/`
- Auto-cleanup for old entries (24 hours)

### 2. Add Interactive Session Manager (`gcm/session.py`)
- Track current session state (last generated messages, diff state)
- Allow selection from previously generated messages
- Support regeneration with variations

### 3. Enhance CLI with New Commands
- Add `--regen` flag to regenerate with variation
- Add `--context "additional context"` option for custom context
- Add `--select` flag to choose from cached messages
- Add `--clear-cache` to clear message cache

### 4. Modify Existing Components

#### cli.py
- Add new flags and options
- Integrate session/cache managers
- Handle regeneration logic

#### llm_client.py
- Add variation temperature parameter
- Support context injection
- Handle regeneration requests differently

#### git_utils.py
- Add diff hashing method
- Add method to get current commit SHA

## User Experience Flow

1. **First run**: Generate and cache messages
2. **If not committed**: `gcm` shows cached + option to regenerate
3. **`gcm --regen`**: Generate new variation
4. **`gcm --context "fixing CI issue"`**: Regenerate with context
5. **`gcm --select`**: Interactive selection from cache

## Key Features

### Smart Caching
- Only cache if at same commit/diff state
- Use hash of diff + commit SHA as cache key
- Detect when diff has changed

### Quick Access
- Instant retrieval of previous generations
- No API calls needed for cached messages
- Show timestamp of generation

### Context Enhancement
- Add context without full regeneration
- Append context to existing prompt
- Keep history of context additions

### Variation Control
- Different temperatures for regeneration
- Ensure variations are actually different
- Track which variation user prefers

### Auto-cleanup
- Prevent cache bloat
- 24-hour TTL for cache entries
- Option to manually clear cache

## Implementation Details

### Cache Structure
```python
{
    "cache_key": "diff_hash_commit_sha",
    "timestamp": "2024-01-20T10:30:00",
    "diff_hash": "abc123...",
    "commit_sha": "def456...",
    "messages": [
        "feat: add user authentication",
        "feat: implement login functionality"
    ],
    "context": "original context",
    "additional_context": []
}
```

### CLI Examples
```bash
# Generate and cache
gcm

# Show cached or regenerate
gcm  # Shows: "Found cached messages (generated 2 min ago). Use --regen for new suggestions."

# Regenerate with variation
gcm --regen

# Add context and regenerate
gcm --context "This fixes the CI pipeline issue"

# Select from cached
gcm --select  # Interactive menu

# Clear cache
gcm --clear-cache
```

## Files to Create/Modify

### New Files
- `gcm/cache.py` - Cache manager implementation
- `gcm/session.py` - Session state manager

### Modified Files
- `gcm/cli.py` - Add new CLI options and integrate managers
- `gcm/llm_client.py` - Support variations and context injection
- `gcm/git_utils.py` - Add utility methods for hashing

## Testing Considerations

- Test cache persistence across sessions
- Test cache invalidation on diff changes
- Test regeneration produces different results
- Test context addition improves relevance
- Test auto-cleanup functionality

## Future Enhancements

- Learn from user selections to improve future generations
- Support for team-shared message templates
- Integration with commit history analysis
- Smart context detection from changed files