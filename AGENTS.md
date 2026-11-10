<!--
  AGENTS.md
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
-->

# AGENTS.md

## AI assistant rules

- Always communicate in English.
- Explain code, errors, and design decisions in English.
- Ask clarification questions in English.
- Read this file and the shared client library API before changing behavior.
- Never commit secrets, client keys, media, or runtime data.
- Keep the GUI a client of the backend; never bypass the backend.

## Project context

`aiqiezi-gui` is the native desktop UI for the aiqiezi single-backend
photo/video archive. See [README.md](README.md) and the server repository for
the system architecture.

Mandatory constraints:

- use `aiqiezi-client`;
- iroh JSON and iroh-blobs only;
- no HTTP client or hidden HTTP server;
- no direct PostgreSQL access;
- no direct server filesystem access;
- do not shell out to the CLI for normal reads or commands;
- the GUI may spawn the bundled `aiqiezi` binary only for local daemon
  lifecycle management;
- initial enrollment uses the server bootstrap password;
- after the first password change, authorization is the server-side EndpointId
  whitelist;
- no session-token flow;
- talk to the local `aiqiezi` client daemon over the shared local socket;
- do not create a second iroh endpoint or client key;
- status: first connection and authentication milestone implemented.

## Platform support

Tier 1 platforms are x86_64 Windows, x86_64 macOS, and x86_64 Linux. Tagged
releases publish prebuilt binaries for these targets. Do not add installer,
signing, or notarization pipelines as part of normal feature work. ARM64
targets are deferred.

Rules:

- keep `eframe`/`egui` and `wgpu` usage portable across the three Tier 1
  targets;
- use `aiqiezi-client` for platform paths and key storage; do not add
  Unix-only APIs or shell commands;
- keep platform-specific blocking work off the UI thread;
- document any new native dependency and its per-OS build requirement;
- Windows builds use the Windows SDK resource compiler through `winresource`
  to embed `assets/aiqiezi.ico`;
- on Linux, prefer X11/XWayland when `DISPLAY` is available, matching
  LightCraft; allow native Wayland with `AIQIEZI_WAYLAND=1`;
- add ARM64 only after it is explicitly added to the support matrix.

## Release rules

- A `v*` tag builds Linux x86_64, macOS x86_64, and Windows x86_64 archives.
- Build Linux on Ubuntu 22.04 for a stable glibc baseline.
- Build macOS on `macos-15-intel` and Windows with the MSVC toolchain.
- Every archive contains `aiqiezi-gui`, the matching `aiqiezi` daemon, and
  `LICENSE`.
- Resolve the CLI revision from the `aiqiezi-client` pin in `Cargo.toml`; do not
  hardcode a second daemon revision.
- Bundle the daemon by building the pinned CLI revision for the same target.
- Publish `SHA256SUMS` with every release.
- Do not publish ARM64, installers, signed, or notarized artifacts yet.
- Windows builds use the static CRT and embed `assets/aiqiezi.ico`.
- macOS binaries are unsigned and may require the user to bypass Gatekeeper
  manually.
- Linux binaries are dynamically linked against glibc; document the baseline.
- Never commit release artifacts or credentials.

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

The GUI uses the shared `aiqiezi-client` library for local socket requests.
It does not own an iroh endpoint and does not create a second client identity.

## Repository layout

```text
aiqiezi-gui/
├── .github/
│   └── workflows/release.yml
├── Cargo.toml
├── Makefile
├── assets/
│   ├── aiqiezi.ico
│   └── aiqiezi.png
├── build.rs
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── daemon_process.rs
│   ├── logo.rs
│   ├── logging.rs
│   ├── theme.rs
│   ├── version.rs
│   └── worker.rs
├── docs/
└── AGENTS.md
```

## Stack

- Rust 2021.
- `eframe` / `egui` 0.37 with the `wgpu` backend for the desktop UI.
- Keep eframe features explicit: `accesskit`, `clipboard`, `default_fonts`,
  `links`, `wayland`, `wgpu`, and `x11`. Native keyboard copy/paste and
  `RequestPaste` depend on the `clipboard` feature.
- `wgpu` for video frame presentation when playback is added.
- `cpal` for audio output when playback is added.
- Tokio runtime in a background thread.
- Channels between async workers and the UI thread.
- `aiqiezi-client` from GitHub, pinned to a CLI revision, for all local socket
  and backend communication.
- `image` with PNG support for the embedded window icon.
- egui's embedded fonts: Ubuntu-Light for proportional text and Hack for
  monospace text. No custom font is bundled at this phase.
- `tracing` for structured logs.

## Logging rules

- Write lifecycle logs to stderr so terminal launches show them.
- Log GUI startup, daemon spawn, daemon readiness, reconnects, and failures.
- Do not log passwords, request parameters, secret keys, or tickets.
- `AIQIEZI_LOG_LEVEL` overrides `RUST_LOG`; otherwise use
  `aiqiezi_gui=info,aiqiezi_client=info,eframe=info,winit=warn,iroh=warn`.
- Keep the CLI daemon log at `<AIQIEZI_DATA_DIR>/client.log`.
- Do not add a GUI debug window or per-command timing UI.

## UI rules

- Follow the LightCraft-style workspace: menu bar, toolbar, left library sidebar,
  central asset grid, right inspector, and bottom status bar. See
  [docs/interface.md](docs/interface.md).
- Use a charcoal dark theme and the blue accent token; do not add a logo to the
  workspace.
- Use resizable fixed panels. Do not add a full docking dependency without an
  explicit decision.
- Long public EndpointIds must have explicit Paste/Copy affordances; keep native
  keyboard clipboard shortcuts working.
- Never copy password fields or secrets to the clipboard.
- Wire GUI Reconnect actions to the daemon's `reconnect` command so the iroh
  endpoint can recover after sleep or a network change; do not restart the
  daemon for a network refresh.
- Use FilmCraft only as a reference for video playback; do not copy its editing
  workspace or add it as a dependency.
- Never block the egui render thread on network or disk I/O.
- Use virtualized lists for large collections.
- Paginate API requests.
- Cache thumbnails on disk with bounded size.
- Show explicit states: loading, empty, partial, offline, and error.
- Keep destructive actions behind confirmation dialogs.
- Do not load all assets or all thumbnails at once.

## Branding and icons

- Keep branding assets under `assets/`.
- Embed the PNG at compile time; do not depend on files beside the executable at
  runtime.
- Use the PNG only for the runtime window/taskbar icon.
- Embed the ICO into Windows executables with `build.rs`; keep the Linux/macOS
  builds independent of the Windows resource compiler.
- Do not show a startup splash, logo, or wordmark in the main workspace. Follow
  the LightCraft-style chrome-only shell.
- Do not add a second HTTP or asset-loading service for branding.

## Typography

- Use egui's embedded default fonts at this phase.
- Proportional UI text uses Ubuntu-Light.
- Monospace text uses Hack.
- Do not add custom font files without an explicit product decision.
- If a custom font is approved later, verify its license, embed it in the
  binary, retain egui fonts as fallbacks, document it in `docs/branding.md`, and
  test all Tier 1 platforms.

## Build and bundle

`make build` produces a runnable directory containing both the GUI and the
client daemon:

```text
dist/aiqiezi-gui
dist/aiqiezi
dist/LICENSE
```

The Makefile reads the pinned `aiqiezi-cli` revision from `Cargo.toml` and
builds the `aiqiezi` binary from that Git revision. The pin must be a CLI
revision that supports `aiqiezi start [<server-endpoint-id>]` and network-change
recovery. The current pin is the CLI networking-recovery commit (after the
`v0.1.1` release). The GUI and daemon must use the same client protocol
revision. No local CLI checkout is required.

`AIQIEZI_CLI_BIN=/path/to/aiqiezi make build` allows a locally built daemon to
be bundled instead.

Do not copy a stale daemon binary without updating the pinned client revision.
The GUI cannot work without the daemon running.

## Data flow

```text
UI action
  -> command channel
  -> background Tokio worker
  -> aiqiezi-client local socket client
  -> aiqiezi client daemon
  -> server
  -> event channel
  -> UI state update
```

Do not call blocking operations from `ui()`.

## Client daemon lifecycle

The daemon owns the persistent client key and server connection. The GUI reads
the same `AIQIEZI_DATA_DIR` and `AIQIEZI_RUNTIME_DIR` as the CLI and connects to
`<AIQIEZI_RUNTIME_DIR>/client.sock`. Both processes must use the same absolute
paths; the defaults are relative to the current directory.

The GUI may spawn the bundled `aiqiezi start` process only for daemon lifecycle:

- on launch, if the daemon is not running, start it automatically;
- on first run with no server EndpointId, show the setup form, save the
  EndpointId through the daemon or CLI, then start the daemon;
- the window close icon exits only the GUI and leaves the daemon running;
- **File → Exit and stop client** sends `shutdown` to the local daemon and then
  closes the GUI.

Find the CLI in this order: `AIQIEZI_CLI_BIN`, next to the GUI executable, then
`PATH`. Redirect daemon output to `<data_dir>/client.log`.

Do not use the CLI for normal reads or commands, and do not start a second iroh
endpoint or client key.

## Video playback

egui has no built-in video widget, but in-app playback is supported by
presenting decoded frames as textures. Playback is deferred until browsing and
thumbnails are functional.

Mandatory architecture when playback is implemented:

- enable the `wgpu` feature of `eframe`;
- demux and decode on worker threads, never in `ui()`;
- keep video frame and audio queues bounded; drop late frames instead of
  increasing latency;
- upload decoded frames as wgpu textures and register them through
  `egui_wgpu` (`register_native_texture` or
  `update_egui_texture_from_wgpu_texture`);
- paint the texture with `ui.painter().image(...)`;
- play audio with `cpal`; use the audio clock as the playback master and a
  wall clock when there is no audio track or output device;
- flush queues and restart from a keyframe when seeking;
- download media through `aiqiezi-client` into a bounded local cache; do not
  add a hidden HTTP streaming server.

The decode backend must be chosen explicitly before implementation. FFmpeg via
`ffmpeg-next` is the recommended default for broad phone/camera format
coverage. Pure-Rust decoders are a large effort; embedded libmpv and an
external player remain alternatives for unsupported formats.

Reference implementation: [FilmCraft](https://github.com/storytold/filmcraft).
See [docs/video-playback.md](docs/video-playback.md) for the pipeline and
trade-offs. Do not copy FilmCraft's internal codec crates wholesale.

## Security rules

- Never store the server password or session tokens.
- The first login uses the server bootstrap password and must be followed by a
  password change. The GUI prompts for passwords without echo and discards
  them after the request.
- The local daemon owns the persistent client key and iroh EndpointId.
- Login and password change use the daemon's persistent key; the server enrolls
  the connection EndpointId.
- The daemon stores the client secret key with restrictive permissions.
- After the first password change, authorization is managed by the EndpointId
  whitelist.
- A disabled or removed EndpointId is rejected at connection time.
- Redact secrets in logs and UI error messages.
- Treat downloaded media and metadata as untrusted input.

## Commands

```bash
make build
make run
make test
make clippy
make ci

cargo check
cargo test
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo run
```

## Source file rules

Every new `.rs`, `.sql`, or `.md` source file must start with the project
license header, using the file path relative to the repository root:

```rust
/*
  relative/path.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/
```

For Markdown use an HTML comment with the same content.

## Testing rules

- Unit-test UI state transitions, status parsing, and formatting.
- Test pagination and cache behavior.
- Test error and offline states.
- Test local-socket request and response handling.
- Integration tests must not require production data or secrets.
- Do not make GUI tests depend on a live production server.

## Agent workflow

1. Read this file and the `aiqiezi-client` API.
2. Keep long-running work off the UI thread.
3. Preserve the remote/local source distinction if caching is added.
4. Run formatting, tests, and checks.
5. Report any new protocol or client-library requirements.
