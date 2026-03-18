# Voice Type

Voice Type is a desktop push-to-talk speech-to-text app built with Rust, Tauri, React, and Bun.

Hold your hotkey, speak, release, and the app transcribes your speech with Groq Whisper and types the result into the active window.

## Current State

- Version: `0.3.2`
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

## Building for Release

### NSIS Installer

```bash
bun run tauri build
```

Output: `src-tauri/target/release/bundle/nsis/Voice Type_0.3.2_x64-setup.exe`

### Microsoft Store (MSIX)

Requires Windows SDK with `makeappx.exe` (already installed).

```bash
bun run tauri build
bun run msix
```

Output: `msix-output/Boring877.VoiceType_0.3.2.0_x64__kg07y93afj4jj.msix`

Upload the `.msix` file to [Microsoft Partner Center](https://partner.microsoft.com/dashboard) to update the Store listing.

> A symlink to the MSIX output folder is on the Desktop (`voice-type-msix`) for quick access.
