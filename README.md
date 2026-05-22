# DOMA

**A lightweight, cross-platform, terminal-based general-purpose AI chat interface.**

DOMA uses OpenCode Go / Zen subscription API keys to power general-purpose LLM conversations — not just coding. Features a 90s nuclear control room aesthetic with green/amber monochrome, heavy ASCII borders, and industrial framing.

---

## Features

- **Pure Rust** — single compiled binary, no runtime dependencies (no Node, no Python, no JVM)
- **OpenAI-compatible API** — SSE streaming chat completions, works with OpenCode Go & Zen
- **Image upload** — attach images via native OS file dialog (`Ctrl+P`), auto-resize & base64 encode
- **Multimodal models** — `[IMG]` tagged in model picker; send text + images together
- **Session management** — multiple conversations persisted locally via `sled` embedded database
- **Session switching** — browse and switch sessions with `Ctrl+S`
- **Export/Import** — save sessions as `.json` (`/export`) and restore them (`/import`)
- **Search** — `Ctrl+F` to search message history, jump to results
- **Command system** — `/help`, `/clear`, `/models`, `/new`, `/undo`, `/export`, `/import`
- **Model selection** — pick from available models with `Ctrl+M`, choice persists across restarts
- **Auto session titles** — sessions auto-name from the first message you send
- **Auto-scroll** — view follows new content; `PageUp` stops, `PageDown` resumes
- **Markdown stripping** — bold, code fences, headings, inline code cleaned from display
- **90s nuclear aesthetic** — green/amber on black, heavy borders, industrial error modals
- **Cross-platform** — Linux, macOS, Windows

---

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

# Windows
# Download doma-x86_64-pc-windows-msvc.zip from releases page
```

### Package Managers

- **Homebrew** (macOS/Linux): `brew install anomalyco/tap/doma` (future)
- **Scoop** (Windows): `scoop bucket add doma https://github.com/GustavoDGoat/DOMA` (future)
- **AUR** (Arch Linux): `yay -S doma-bin` (future)

### From Source

```bash
git clone https://github.com/GustavoDGoat/DOMA.git
cd DOMA
cargo build --release
./target/release/doma
```

---

## Quick Start

1. **Launch DOMA**: `./target/release/doma`
2. **Enter API key**: On first launch, you'll be prompted for your OpenCode API key (masked input)
3. **Select model**: Pick a model from the list — models tagged `[IMG]` support image upload
4. **Start chatting**: Type a message and press `Enter`

API key and model choice are saved locally for next launch.

---

## Commands

Type these directly in the input bar:

| Command | Description |
|---------|-------------|
| `/help` | Show all commands and keybindings |
| `/clear` | Clear current session messages |
| `/models` | Open model selection popup |
| `/new` | Create a new session |
| `/export` | Export current session to `.json` file |
| `/import` | Import a session from `.json` file |
| `/undo` | Remove the last assistant response |

---

## Keybindings

| Key | Action |
|-----|--------|
| `Enter` | Send message / confirm |
| `Esc` | Cancel stream / close error / close popup / abort operation |
| `Ctrl+P` | Attach an image (opens native file picker) |
| `Ctrl+D` | Detach / remove attached image |
| `Ctrl+S` | Open session switcher popup |
| `Ctrl+F` | Open search overlay |
| `Ctrl+M` | Open model selection popup |
| `Ctrl+N` | Create new session |
| `Ctrl+Q` | Quit DOMA |
| `↑/↓` | Scroll chat line-by-line / navigate popup lists |
| `PageUp` | Scroll up (stops auto-scroll, shows `[LOCK]`) |
| `PageDown` | Scroll down |

---

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DOMA_API_BASE_URL` | `https://opencode.ai/zen/go/v1` | API endpoint base URL |

### Data Storage

DOMA stores all data locally in a `sled` embedded database:

| Platform | Path |
|----------|------|
| **Linux** | `~/.local/share/doma/db/` |
| **macOS** | `~/Library/Application Support/doma/db/` |
| **Windows** | `%APPDATA%/doma/db/` |

Three storage trees:
- **config** — API key, model selection, base URL, active session
- **sessions** — conversation metadata (title, created date)
- **messages** — message history per session

---

## API Compatibility

DOMA uses OpenAI-compatible chat completions (`/v1/chat/completions` with SSE streaming). It is tested with:

- **OpenCode Go** — `https://opencode.ai/zen/go/v1` (subscription-based, $10/month)
- **OpenCode Zen** — `https://opencode.ai/zen/v1` (pay-as-you-go credits)

You can point it at any OpenAI-compatible endpoint by setting `DOMA_API_BASE_URL`.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  main.rs        Binary entry — init terminal, tracing       │
│  lib.rs         Module declarations                         │
│  app.rs         State machine + event loop + keybindings    │
│  config/        API key, base URL, model persistence        │
│  client/        OpenAI-compatible API client, SSE streamer  │
│  storage/       sled embedded DB (config, sessions, msgs)   │
│  ui/            TUI rendering (header, chat, input, status) │
└─────────────────────────────────────────────────────────────┘
```

### State Machine

```
Boot → ApiKeyInput → SelectingModel → Idle
                                          ↓
Idle → (Enter) → WaitingResponse → Idle
Idle → (Ctrl+P) → PickingFile → ProcessingImage → Idle
Idle → (Ctrl+S) → SessionList → Idle
Idle → (Ctrl+F) → Searching → Idle
Idle → (Ctrl+M) → SelectingModel → Idle
```

---

## Development

```bash
cargo check              # Verify compilation
cargo clippy             # Lint (deny warnings)
cargo test               # Run tests
cargo build --release    # Production build (LTO + strip)
```

### CI/CD

GitHub Actions workflows in `.github/workflows/`:
- **ci.yml** — Build, lint, and test on ubuntu/macos/windows
- **release.yml** — Tag-triggered builds for 4 targets, GitHub Releases

---

## License

MIT
