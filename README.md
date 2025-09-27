# gcm

Generate intelligent git commit messages using AI.

## Install

### From source
```bash
git clone https://github.com/yourusername/gcm
cd gcm
cargo install --path .
```

### Direct cargo install
```bash
cargo install --git https://github.com/yourusername/gcm
```

## Uninstall

To remove gcm from your system:

```bash
# If installed with cargo
cargo uninstall gcm

# To also remove configuration files
rm -rf ~/.gcmrc
rm -rf ~/.gcm
```

## Setup

Set your OpenAI API key:
```bash
export OPENAI_API_KEY="sk-..."
```

## Usage

```bash
# Generate commit message for staged changes
gcm

# Generate and commit immediately
gcm -c

# Stage all changes and generate
gcm -a

# Stage all and commit
gcm -ac

# Get 3 message suggestions
gcm -n 3
```

## Configuration (Optional)

Create `~/.gcmrc` or `.gcm.yml` in your project:

```yaml
model: gpt-4o-mini
max_tokens: 100
```

## Examples

```bash
$ git add src/main.rs
$ gcm
Generated commit message:
  feat: add async runtime support for API calls

$ gcm -ac
Staging all changes...
Generated commit message:
  fix: resolve memory leak in request handler
Committed with message: fix: resolve memory leak in request handler
```

## Requirements

- Rust 1.70+
- Git
- OpenAI API key

## Build from source

```bash
cargo build --release
./target/release/gcm
```