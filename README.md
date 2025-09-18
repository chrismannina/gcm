# gcm

Git commit message generator using LLMs.

## Install

```bash
pip install gcm
```

## Setup

```bash
export OPENAI_API_KEY="sk-..."
```

## Usage

```bash
git add .
gcm                  # generate message
gcm -c               # generate and commit
gcm -a               # stage all, then generate
gcm -ac              # stage all and commit
gcm -n 3             # show 3 alternatives
gcm --model gpt-4    # use specific model
```

## Config

Optional. Create `~/.gcmrc` or `.gcm.yml`:

```yaml
model: gpt-4o-mini
context:
  - README.md
  - package.json
max_tokens: 100
```

## Examples

```bash
$ git add src/auth.py
$ gcm
Generated commit message:
  feat: add JWT token validation to authentication module

$ gcm -c
Committed with message: fix: resolve null pointer exception in user service
```

## Development

```bash
git clone <repo>
cd gcm
pip install -e .
```

## Requirements

- Python 3.8+
- Git
- OpenAI API key
