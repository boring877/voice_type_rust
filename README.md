# Voice Type

Voice Type is a desktop push-to-talk speech-to-text app built with Rust, Tauri, React, and Bun.

Hold your hotkey, speak, release, and the app transcribes your speech with Groq Whisper and types the result into the active window.

## Current State

- Version: `0.2.0`
- Desktop shell: `Tauri 2`
- Frontend: `React + TypeScript + Vite`
- Backend engine: `Rust + Tokio`
- Package manager: `Bun`

## Features

- Push-to-talk transcription
- Global hotkey capture
- Floating recording HUD
- Appearance customization
- Optional grammar cleanup
- Text cleanup filters
- Local config storage

## Project Layout

```text
.
|- Cargo.toml
|- src/                  # Rust backend engine
|- frontend/             # React desktop UI
|- src-tauri/            # Tauri shell and command bridge
|- package.json
`- assets/
```

## Development

### Prerequisites

- Rust `1.85+`
- Bun
- Windows: WebView2
- Linux: Tauri/WebKitGTK prerequisites
- macOS: Xcode command line tools

### Commands

```bash
bun install
bun run tauri:dev
```

Useful checks:

```bash
bun run build
cargo check --lib
cargo check --manifest-path src-tauri/Cargo.toml
```

## Configuration

Config is stored at:

- Windows: `%APPDATA%\voice-type\config.json`
- macOS: `~/Library/Application Support/voice-type/config.json`
- Linux: `~/.config/voice-type/config.json`

You can either:

- put the API key in the app settings, or
- leave `api_key` empty and use `GROQ_API_KEY`

## Notes

- This repo is the current codebase.
- The older Python-based implementation should be treated as legacy.
- Bundled public image sources are documented in [ASSET_SOURCES.md](/c:/Users/Borin/OneDrive/Documents/voice_type_rust/ASSET_SOURCES.md).
