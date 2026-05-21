# DOMA

A lightweight, cross-platform, terminal-based general-purpose AI chat interface with a 90s nuclear control room aesthetic.

Powers general-purpose LLM conversations using OpenCode Go / Zen subscription API keys.

## Features

- Pure Rust, single compiled binary — no runtime dependencies
- OpenAI-compatible chat completions with SSE streaming
- Session management with local persistence via `sled`
- 90s nuclear submarine hacker terminal aesthetic
- Cross-platform: Linux, macOS, Windows
- API base URL configurable via env var or config file

## Installation

### From GitHub Releases

```bash
# Linux
curl -L https://github.com/GustavoDGoat/DOMA/releases/latest/download/doma-x86_64-unknown-linux-gnu.tar.gz | tar xz
./doma

# macOS (Intel)
curl -L https://github.com/GustavoDGoat/DOMA/releases/latest/download/doma-x86_64-apple-darwin.tar.gz | tar xz
./doma

# macOS (Apple Silicon)
curl -L https://github.com/GustavoDGoat/DOMA/releases/latest/download/doma-aarch64-apple-darwin.tar.gz | tar xz
./doma

# Windows (via PowerShell)
# Download from releases page
```

### From Source

```bash
git clone https://github.com/GustavoDGoat/DOMA.git
cd DOMA
cargo build --release
./target/release/doma
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DOMA_API_BASE_URL` | API endpoint base URL | `https://opencode.ai/zen/v1` |

### API Key

On first launch, DOMA will prompt you to enter your OpenCode API key.
The key is stored locally in the `sled` database at:
- **Linux:** `~/.local/share/doma/db/`
- **macOS:** `~/Library/Application Support/doma/db/`
- **Windows:** `%APPDATA%/doma/db/`

## Keybindings

| Key | Action |
|-----|--------|
| `Enter` | Send message |
| `Esc` | Cancel stream / close error |
| `Ctrl+Q` | Quit |
| `Ctrl+N` | New session |
| `PageUp/Down` | Scroll chat history |

## Development

```bash
cargo check     # Verify compilation
cargo clippy    # Lint
cargo test      # Run tests
cargo build --release  # Production build
```

## License

MIT
