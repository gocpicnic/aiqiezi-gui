# aiqiezi-gui

[![CI](https://github.com/gocpicnic/aiqiezi-gui/actions/workflows/ci.yml/badge.svg)](https://github.com/gocpicnic/aiqiezi-gui/actions/workflows/ci.yml)

Native desktop GUI for
[aiqiezi-server](https://github.com/gocpicnic/aiqiezi-server), the self-hosted
photo and video archive backend. The GUI connects through the local
`aiqiezi` client daemon, handles first-run server EndpointId setup, and provides
login and server administration.

## Status

First connection and authentication milestone implemented:

- native `eframe`/`egui` window with app/window icon;
- LightCraft-inspired dark workspace: menu bar, toolbar, sidebar,
  asset-grid canvas, inspector, and status bar;
- automatic local client daemon startup;
- first-run server EndpointId setup;
- login, password change, server-info, and EndpointId dialogs;
- background Tokio worker with UI event channels;
- window close leaves the daemon running;
- **File → Exit and stop client** stops the daemon and exits.

There is no startup splash and no logo or wordmark inside the workspace.

Asset browsing, thumbnails, search, and video playback remain deferred.

## Interface

The workspace has a LightCraft-style layout:

```text
menu bar
toolbar
┌──────────────┬──────────────────────────────┬────────────────┐
│ Library      │                              │ Inspector      │
│ sidebar      │ asset grid / empty state     │ Info / Server  │
└──────────────┴──────────────────────────────┴────────────────┘
status bar
```

Menus:

```text
File    Edit    View    Server    Help
```

The sidebar and inspector are resizable. Login, password change, EndpointId
change, server info, and About are dialogs. The app opens directly into setup,
daemon-starting, or the workspace state; there is no splash.

EndpointId fields include `Paste` and `Copy`, and the inspector and Server Info
dialog can copy full public IDs and JSON. Native `Ctrl+V` / `Cmd+V` and
`Ctrl+C` / `Cmd+C` remain available. On Linux Wayland, clipboard access depends
on compositor data-control support; if `Paste` is unavailable, it reports an
error and `Ctrl+V` remains the fallback.

After sleep, a network switch, or a temporary outage, use **Server → Reconnect**
or the Reconnect button in the inspector. This refreshes the daemon's iroh
network state without restarting it.

See [docs/interface.md](docs/interface.md) for the full interaction guide.

## Platform support

Tier 1 targets are x86_64 Windows, x86_64 macOS, and x86_64 Linux. Tagged
releases provide prebuilt binaries for these targets. There are no installers,
signing, or notarization yet; source builds remain supported.

Build prerequisites:

- Linux: X11/Wayland and graphics development libraries required by
  `eframe`/`wgpu`. When `DISPLAY` is available, the GUI uses X11/XWayland by
  default, matching LightCraft. Set `AIQIEZI_WAYLAND=1` to force native Wayland.
- macOS: Xcode command-line tools;
- Windows: the MSVC Rust toolchain, Visual Studio C++ build tools, and the
  Windows SDK resource compiler used to embed the application icon.

ARM64 targets, including Apple Silicon, are not yet supported. Source builds
may work on those hosts, but they are untested and outside the support matrix.

## Releases

Pushing a `v*` tag runs the GitHub Actions release workflow. Each archive
contains:

```text
aiqiezi-gui
aiqiezi
LICENSE
```

Release assets:

```text
aiqiezi-gui-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz
aiqiezi-gui-X.Y.Z-x86_64-apple-darwin.tar.gz
aiqiezi-gui-X.Y.Z-x86_64-pc-windows-msvc.zip
SHA256SUMS
```

The bundled `aiqiezi` daemon comes from the CLI revision pinned in
`Cargo.toml`, so the GUI and daemon use the same protocol.

Linux binaries require compatible X11/Wayland/OpenGL libraries and a glibc
runtime at least as new as the Ubuntu 22.04 build runner. macOS and Windows
binaries are unsigned; Gatekeeper and SmartScreen may show warnings. No
installers, signing, notarization, or ARM64 binaries are provided.

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

## Branding assets

Branding files live in `assets/`:

```text
assets/aiqiezi.png   # runtime window icon
assets/aiqiezi.ico   # Windows executable icon
```

The PNG is embedded in the binary as the runtime window/taskbar icon. The ICO
is embedded into Windows executables through `build.rs`; it is not shown inside
the application.

## Typography

The GUI currently uses egui's embedded default fonts only:

- proportional UI text: **Ubuntu-Light**;
- monospace text: **Hack**.

No custom font files are bundled in this phase. EndpointIds, socket paths, and
JSON views use the monospace family; headings, labels, buttons, and forms use
the proportional family. See [docs/branding.md](docs/branding.md) for the asset
and font policy.

Eframe’s `clipboard`, `default_fonts`, `links`, `accesskit`, `wayland`, `wgpu`,
and `x11` features are enabled explicitly. Native keyboard copy/paste and
`RequestPaste` depend on the `clipboard` feature.

## Running

Run the GUI from source:

```bash
cargo run
```

Or run the bundled build:

```bash
make build
cd dist
./aiqiezi-gui
```

The GUI starts the local client daemon automatically. On the first run it asks
for the aiqiezi-server EndpointId:

```bash
# On the server host
aiqiezi-server whoami
```

After the EndpointId is saved, the GUI starts the daemon and opens the Login
dialog. Later launches reuse the saved EndpointId and attach to an
already-running daemon.

For source runs, the GUI finds the daemon from `AIQIEZI_CLI_BIN`, next to the GUI
binary, or on `PATH`. `make build` bundles both binaries in `dist/`.

The GUI and CLI use the same `AIQIEZI_DATA_DIR` and `AIQIEZI_RUNTIME_DIR`, so
they share the persistent client key and server EndpointId configuration.

Set both variables to the same absolute paths before starting either process.
The defaults are relative to the current directory, so starting the CLI and GUI
from different directories will not connect them to the same daemon.

Closing the window exits only the GUI and leaves the daemon running. Use
**File → Exit and stop client** to stop the daemon as well.

## Logging

When the GUI is started from a terminal, lifecycle logs are written to stderr
with timestamps, targets, and thread names:

```bash
cargo run
```

The default level is `info` for `aiqiezi_gui` and `aiqiezi_client`, with
`eframe` at `info` and `winit`/`iroh` at `warn`. Override it with:

```bash
AIQIEZI_LOG_LEVEL=debug cargo run
```

or use `RUST_LOG` when `AIQIEZI_LOG_LEVEL` is unset:

```bash
RUST_LOG=aiqiezi_gui=debug,iroh=info cargo run
```

The CLI daemon log remains at `<AIQIEZI_DATA_DIR>/client.log`.

## Development

```bash
cargo check
cargo test
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo run
```

The GUI depends on the shared client library through a pinned Git dependency:

```toml
aiqiezi-client = { git = "https://github.com/gocpicnic/aiqiezi-cli", rev = "664e72fe41f45d3934602e988864cae9aab40cf2" }
```

Update the pinned revision when the shared client library changes. No local
checkout of the CLI repository is required.

The GUI can be built from source or downloaded from a tagged release. No
installers or signing are provided.

## Runnable bundle

Build the GUI and bundle the matching `aiqiezi` client daemon:

```bash
make build
```

Output:

```text
dist/
├── aiqiezi-gui
├── aiqiezi
└── LICENSE
```

On Windows the binaries use `.exe`.

The Makefile reads the pinned `aiqiezi-cli` revision from `Cargo.toml` and
installs the daemon from that Git revision, so the GUI and daemon use the same
client protocol. No local CLI checkout is required.

Run the bundle:

```bash
cd dist
./aiqiezi-gui
```

The GUI starts `./aiqiezi` automatically. On first run it asks for the server
EndpointId, then shows login.

To bundle a locally built CLI instead:

```bash
AIQIEZI_CLI_BIN=/path/to/aiqiezi make build
```

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
5. Choose the video decode backend and add in-app playback.

## Documentation

- [AGENTS.md](AGENTS.md): implementation rules for AI coding agents
- [docs/interface.md](docs/interface.md): workspace layout, menus, and startup
  states
- [docs/branding.md](docs/branding.md): branding assets, icons, and font policy
- [docs/video-playback.md](docs/video-playback.md): playback architecture and
  decoder backend trade-offs
- [aiqiezi-server](https://github.com/gocpicnic/aiqiezi-server): system
  architecture
- [aiqiezi-cli](https://github.com/gocpicnic/aiqiezi-cli): shared client
  library and CLI
