# Voice Type

Voice Type is now organized as a Rust backend engine with a `React + TypeScript + Vite + Tauri` desktop shell.

## Structure

```text
voice_type_rust/
|- Cargo.toml            # Shared Rust backend crate
|- src/                  # Backend engine: audio, API, input, processing, shared types
|- frontend/             # React + TypeScript UI
|- src-tauri/            # Tauri desktop shell and Rust bridge
|- package.json          # Frontend and Tauri scripts
`- assets/               # Icons and bundled assets
```

## What Lives Where

- `src/api`: Groq transcription and grammar API code
- `src/audio`: microphone capture and WAV encoding
- `src/input`: global hotkeys and typed text output
- `src/processing`: cleanup and post-processing of transcription text
- `src/tasks`: background recording/transcription tasks
- `src/types`: config and runtime-shared Rust types
- `src-tauri/src`: Tauri runtime manager, tray, and command bridge
- `frontend/src`: React desktop UI

## Development

### Prerequisites

- Rust 1.85+
- Bun
- Windows: WebView2
- Linux: usual Tauri/WebKitGTK prerequisites plus audio/input deps
- macOS: Xcode command line tools

### Commands

```bash
# Install frontend dependencies
bun install

# Run the desktop app in development
bun run tauri:dev

# Build the frontend only
bun run build

# Check the backend library
cargo check --lib

# Check the Tauri desktop shell
cargo check --manifest-path src-tauri/Cargo.toml
```

## Configuration

Config is stored at:

- Windows: `%APPDATA%\\voice-type\\config.json`
- macOS: `~/Library/Application Support/voice-type/config.json`
- Linux: `~/.config/voice-type/config.json`

You can leave `"api_key"` empty and provide the secret through the `GROQ_API_KEY` environment variable instead.

## Main Stack

- Rust backend
- Tokio
- Tauri 2
- React
- TypeScript
- Vite
- cpal
- rdev
- reqwest

## Notes

- The old egui frontend path has been removed.
- The Rust crate under `src/` is now the reusable backend engine for the Tauri shell.
- Wallpaper previews are handled by the Tauri/React frontend rather than the old Rust GUI layer.
