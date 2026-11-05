# aiqiezi-gui

Native desktop GUI for the aiqiezi single-backend photo and video archive.

## Status

First connection and authentication milestone implemented:

- native `eframe`/`egui` window;
- connection to the local client daemon over the shared local socket;
- client and server EndpointId display;
- server EndpointId configuration;
- login and password change forms;
- server-info request;
- background Tokio worker with UI event channels.

Asset browsing, thumbnails, search, and video playback remain deferred.

## Platform support

Tier 1 targets are x86_64 Windows, x86_64 macOS, and x86_64 Linux. The GUI is
distributed as a source build only: there are no installers, signing, or
notarization yet.

Build prerequisites:

- Linux: X11/Wayland and graphics development libraries required by
  `eframe`/`wgpu`;
- macOS: Xcode command-line tools;
- Windows: the MSVC Rust toolchain and Visual Studio C++ build tools.

ARM64 targets, including Apple Silicon, are not yet supported. Source builds
may work on those hosts, but they are untested and outside the support matrix.

## Purpose

The GUI is intended to provide:

- browsing of large photo/video collections;
- thumbnails and previews;
- folder navigation;
- tags, albums, and search;
- upload and download progress;
- responsive handling of very large result sets.

## Architecture

```text
egui GUI
    |
    | local socket: JSON Lines
    v
aiqiezi client daemon
    |
    | persistent client key + iroh JSON + iroh-blobs
    v
aiqiezi-server
```

The GUI uses the shared `aiqiezi-client` library from the CLI repository. It
does not access PostgreSQL or the server filesystem directly, and it does not
create a second iroh endpoint.

## Running

Start the local client daemon first:

```bash
aiqiezi start
```

Then run the GUI from source:

```bash
cargo run
```

If the daemon is not running, the GUI shows an explicit setup state with the
command to start it.

The GUI and CLI use the same `AIQIEZI_DATA_DIR` and `AIQIEZI_RUNTIME_DIR`, so
they share the persistent client key and server EndpointId configuration.

Set both variables to the same absolute paths before starting either process.
The defaults are relative to the current directory, so starting the CLI and GUI
from different directories will not connect them to the same daemon.

## Development

```bash
cargo check
cargo test
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo run
```

The GUI depends on the shared client library by local path:

```toml
aiqiezi-client = { path = "../aiqiezi-cli/crates/aiqiezi-client" }
```

Expected sibling layout:

```text
aiqiezi-v0/
├── aiqiezi-server/
├── aiqiezi-cli/
└── aiqiezi-gui/
```

The GUI is built from source; no installers or signing are provided.

## Why egui

- native desktop application;
- Rust-only stack;
- immediate-mode UI;
- easy integration with the shared Rust client library;
- no browser, HTTP server, public domain, or TLS certificate required.

## Video playback

egui has no built-in video widget or codec, but it can play video: decode
frames off the UI thread, upload them as wgpu textures, register the texture
with egui, and paint it with `ui.painter().image(...)`. Audio is played
separately with `cpal`, and the audio clock drives playback.

This is the approach used by [FilmCraft](https://github.com/storytold/filmcraft),
a pure-Rust video editor built on eframe/egui + wgpu. FilmCraft uses its own
pure-Rust decoders plus OS hardware decoding; it does not use FFmpeg, mpv, or
GStreamer at runtime.

Playback remains deferred until browsing and thumbnails work. The decode
backend is not selected yet. Candidates are:

- FFmpeg via `ffmpeg-next` (recommended for broad phone/camera format support);
- embedded libmpv;
- pure-Rust decoders (large implementation effort);
- an external player fallback for unsupported formats.

Do not add a hidden HTTP streaming server. See
[docs/video-playback.md](docs/video-playback.md) for the reference architecture
and trade-offs.

## Security

The first login uses the server bootstrap password and must be followed by a
password change. The GUI prompts for passwords without echo and never stores
them. The password change enrolls the shared client EndpointId in the server
whitelist; after that, access is controlled by the EndpointId whitelist. There
is no session-token flow.

Login and password change must use the persistent client key. The server
enrolls the connection EndpointId, so a temporary key would enroll the wrong
identity.

The GUI never creates or stores a second client identity. The daemon owns the
key and the iroh connection.

## Future work

1. Add virtualized asset browsing.
2. Add thumbnails.
3. Add search, tags, and albums.
4. Add upload/download progress.
5. Add optional daemon lifecycle management or auto-start.
6. Choose the video decode backend and add in-app playback.

## Documentation

- [AGENTS.md](AGENTS.md): implementation rules for AI coding agents
- [docs/video-playback.md](docs/video-playback.md): playback architecture and
  decoder backend trade-offs
- `../aiqiezi-server/README.md`: system architecture
- `../aiqiezi-cli/README.md`: shared client library and CLI
