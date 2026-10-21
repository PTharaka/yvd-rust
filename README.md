# Rust download engine

This directory is the Rust foundation for the next version of YouTube Downloader Pro. It is intentionally independent from the current JavaFX application.

## Current structure

- `model.rs` - queue tasks, options, statuses, and UI-facing events
- `command.rs` - deterministic yt-dlp/ffmpeg command construction
- `process.rs` - process lifecycle boundary for start, pause, cancel, and completion
- `queue.rs` - queue management and retry state transitions
- `metadata.rs` - metadata probing through yt-dlp
- `settings.rs` - persisted output, format, cookie, and retry settings
- `persistence.rs` - queue-state persistence
- `main.rs` - CLI entry point for metadata, queue add, and queue list
- `src-tauri/` - Tauri v2 desktop shell and command bridge
- `frontend/` - Vite frontend for the desktop shell

The engine continues to use `yt-dlp` for extraction and `ffmpeg` for merging/conversion. A Tauri or Slint frontend can be added later without moving process logic into the UI.

## Run

```sh
cargo run --bin yvd-cli -- metadata 'https://www.youtube.com/watch?v=VIDEO_ID'
cargo run --bin yvd-cli -- add 'https://www.youtube.com/watch?v=VIDEO_ID' downloads
cargo run --bin yvd-cli -- list
```

The queue file is local application state and is intentionally not committed. The
future Tauri or Slint frontend can use the library modules directly instead of
parsing CLI output.

## Tauri desktop app

```sh
cd frontend
npm install
npm run build
cd ..
frontend/node_modules/.bin/tauri dev --config src-tauri/tauri.conf.json
```

For a macOS application bundle:

```sh
cd frontend
npm run build
cd ..
CARGO_TARGET_DIR=/tmp/yvd-rust-target frontend/node_modules/.bin/tauri build --config src-tauri/tauri.conf.json
```
