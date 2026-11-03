# aiqiezi-gui

Native desktop GUI for the aiqiezi single-backend photo and video archive.

## Status

Planned. The GUI is intentionally deferred until the server and CLI are
functional.

The GUI module is not implemented yet.

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
    v
aiqiezi-client
    |
    | iroh JSON + iroh-blobs
    v
aiqiezi-server
```

The GUI uses the shared `aiqiezi-client` library from the CLI repository. It
does not access PostgreSQL or the server filesystem directly.

## Why egui

- native desktop application;
- Rust-only stack;
- immediate-mode UI;
- easy integration with the shared Rust client library;
- no browser, HTTP server, public domain, or TLS certificate required.

## Video playback

egui has no built-in video player. Video playback is deferred.

When implemented, the strategy must be explicit:

- launch an external player;
- embed libmpv;
- or decode frames with ffmpeg and handle audio separately.

Do not add a hidden HTTP streaming server.

## Security

The GUI uses the shared client key and stable iroh EndpointId. There is no
password or session-token flow. Server access is controlled by the EndpointId
whitelist.

## Future work

1. Scaffold eframe/egui application.
2. Integrate `aiqiezi-client`.
3. Add connection and enrollment display.
4. Add virtualized asset browsing.
5. Add thumbnails.
6. Add search, tags, and albums.
7. Add upload/download progress.
8. Decide on video playback integration.

## Documentation

- [AGENTS.md](AGENTS.md): implementation rules for AI coding agents
- `../aiqiezi-server/README.md`: system architecture
- `../aiqiezi-cli/README.md`: shared client library and CLI
