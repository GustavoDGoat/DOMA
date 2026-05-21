Create a file named `AGENTS.md` in the root of your project directory and paste this inside:

````markdown
# DOMA - System Architecture & Agent Instructions

## 1. Project Overview

DOMA is a lightweight, cross-platform, terminal-based general-purpose AI chat interface written in Rust. It functions by intercepting and utilizing OpenCode Go / Zen subscription API keys (which are traditionally locked or optimized for IDE/code tasks) and re-routing them for general LLM chat operations locally.

### Core Philosophy

- **Zero Bloat:** Pure compiled binary with no runtime dependencies (No Node, No Python, No heavy dynamic libraries).
- **High-Stakes 90s Terminal Vibe:** The UI must resemble a dangerous 1990s industrial or nuclear command station interface.
- **Local-First & Blazing Fast:** Zero network overhead on the local machine; rendering must happen fluidly at 30-60 FPS.

---

## 2. Technical Stack Specifications

- **Language:** Rust (Stable)
- **TUI Architecture:** `ratatui` with the `crossterm` backend.
- **Async Runtime:** `tokio` (multi-threaded feature flag enabled).
- **Local Storage Engine:** `sled` (Pure-Rust embedded Key-Value store).
- **Serialization:** `serde` + `serde_json` or `bincode`.
- **Native Integrations:** `rfd` (Rust File Dialog) for native OS-level visual file picking without blocking the main rendering loop.
- **Image Processing:** `image` crate (for base64 encoding and payload preparation).
- **Network Client:** `reqwest` (Asynchronous feature).

---

## 3. Detailed Architectural Constraints

### A. Non-Blocking Event Loop (Crucial)

The main execution thread runs the `ratatui` UI render tick loop. Any blocking calls will cause the terminal interface to freeze.

- **UI Thread:** Captures input keys, renders frame updates, and displays system logs.
- **Async Background Workers:** OpenCode API calls, image base64 conversions, and database flushes must be handled using asynchronous `tokio` tasks or separate threads communicating via `tokio::sync::mpsc`.

### B. Image Upload Pipeline

When a user requests an image attach (Triggered by `Ctrl+P` or custom action mapping):

1. Spawn a dedicated OS native thread to execute `rfd::FileDialog::new().pick_file()`.
2. Do **not** block the `ratatui` frame drawing loop. While open, transition the state machine to `AppState::PayloadInjection`.
3. Upon completion, pass the `PathBuf` back via an un-bounded channel.
4. Process and compress the target image on the background worker using the `image` crate before serializing to base64.

### C. Storage Topology (`sled`)

All state persistence is handled locally via a singular database managed in the user's localized platform data directory (determined cross-platform via the `dirs` crate):

- **Linux/macOS:** `~/.local/share/doma/db/`
- **Windows:** `%USERPROFILE%\AppData\Local\doma\db\`

Use separate `sled::Tree` handles to segregate keys:

- `"config"`: Stores API tokens and configuration flags.
- `"sessions"`: Tracks active conversation IDs and general metadata.
- `"messages"`: Chronologically sorted key combinations structured as `{session_id}_{timestamp}`.

---

## 4. Design & UX Manifesto (90s Nuclear Control Room)

When generating UI rendering functions or designing text layouts, strictly adhere to the following visual aesthetic:

```text
+-----------------------------------------------------------------------+
| [!] SYSTEM STATUS: REACTING CORE - NOMINAL  |  TUNNEL: OPENCODE-GO    |
+-----------------------------------------------------------------------+
|                                                                       |
|  ASSISTANT > CRITICAL EXCURSION DETECTED IN LOGIC LOOP.               |
|              PLEASE REMIT SUB-ROUTINE INSTRUCTIONS.                   |
|                                                                       |
|                                                                       |
|                                                                       |
+-----------------------------------------------------------------------+
| USER > [ATTACHED: ENV_MAP.PNG] What does this telemetry say?          |
+-----------------------------------------------------------------------+
| [Ctrl+P] Inject Payload  |  [Ctrl+Q] Purge Session  | [Esc] Safe Mode |
+-----------------------------------------------------------------------+
```
````

- **Color Profile:** Strict high-contrast monochrome. Use glowing Terminal Green (`Color::Green`) or Industrial Amber (`Color::Rgb(255, 176, 0)`) on flat Black (`Color::Black`). No gradients, no soft pastels.
- **Status Headers:** The top pane must always house a critical diagnostic dashboard displaying active system constraints (e.g., `CORE TEMPERATURE`, `BUFFER POOL STRESS`, `API TUNNEL INTEGRITY`).
- **Alert Modals:** Errors or timeouts must present as critical safety systems failing. Use blinking alerts, heavy ASCII block symbols, and industrial warning framing (`[!!] CRITICAL EXCURSION FAULT [!!]`).
- **Framing:** Use standard block text symbols, heavy borders, and dash padding (`=`, `+`, `|`, `-`).

---

## 5. Agent Instructions for Code Generation

1. **Strict Idiomatic Rust:** Follow `clippy` lints rigorously. Always choose safe memory layouts, minimize allocations in the main loop, and explicitly leverage proper error handling with `Result` types over quick panics (`.unwrap()`).
2. **No Extraneous UI Libraries:** Write layouts using native `ratatui::layout` constraints (`Constraint::Length`, `Constraint::Percentage`). Keep widgets self-contained.
3. **Encapsulated State Engine:** Separate the interface model (`App`) cleanly from the network agent (`OpenCodeClient`) and storage operations (`StorageEngine`). Do not cross-contaminate UI code with raw database transactions.

```

```
