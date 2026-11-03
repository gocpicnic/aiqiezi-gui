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

`aiqiezi-gui` is the planned native desktop UI for the aiqiezi single-backend
photo/video archive. See [README.md](README.md) and the server repository for
the system architecture.

Mandatory constraints:

- use `aiqiezi-client`;
- iroh JSON and iroh-blobs only;
- no HTTP client or hidden HTTP server;
- no direct PostgreSQL access;
- no direct server filesystem access;
- no shelling out to the CLI for normal reads;
- no login, password, or session-token flow;
- status: deferred until server and CLI are functional.

## Architecture

```text
egui GUI
    |
    v
aiqiezi-client library
    |
    | iroh JSON + iroh-blobs
    v
aiqiezi-server
```

## Repository layout

```text
aiqiezi-gui/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── ui/
│   └── workers/
└── AGENTS.md
```

## Stack

- Rust 2021.
- `eframe` / `egui` for the desktop UI.
- Tokio runtime in a background thread.
- Channels between async workers and the UI thread.
- `aiqiezi-client` for all backend communication.
- `image` for local thumbnail decode/cache if needed.
- `tracing` for structured logs.

## UI rules

- Never block the egui render thread on network or disk I/O.
- Use virtualized lists for large collections.
- Paginate API requests.
- Cache thumbnails on disk with bounded size.
- Show explicit states: loading, empty, partial, offline, and error.
- Keep destructive actions behind confirmation dialogs.
- Do not load all assets or all thumbnails at once.

## Data flow

```text
UI action
  -> command channel
  -> background tokio task
  -> aiqiezi-client
  -> server
  -> response channel
  -> UI state update
```

Do not call blocking operations from `update()`.

## Video playback

Video playback is deferred.

When added, choose one explicit strategy:

- open an external player;
- embed libmpv;
- decode with ffmpeg into textures and handle audio separately.

Do not add a hidden HTTP streaming server.

## Security rules

- Never store passwords or session tokens.
- Use the shared client key and stable iroh EndpointId.
- Store the client secret key with restrictive permissions.
- Server authorization is managed by the EndpointId whitelist.
- A disabled or removed EndpointId is rejected at connection time.
- Redact secrets in logs and UI error messages.
- Do not log endpoint tickets in full.
- Treat downloaded media and metadata as untrusted input.

## Commands

```bash
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

- Unit-test UI state transitions and formatting.
- Test pagination and cache behavior.
- Test error and offline states.
- Integration tests must not require production data or secrets.
- Do not make GUI tests depend on a live production server.

## Agent workflow

1. Read this file and the `aiqiezi-client` API.
2. Keep long-running work off the UI thread.
3. Preserve the remote/local source distinction if caching is added.
4. Run formatting, tests, and checks.
5. Report any new protocol or client-library requirements.
