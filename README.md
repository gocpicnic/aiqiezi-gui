# aiqiezi-gui

Native desktop GUI for the aiqiezi single-backend photo and video archive.

## Status

Planned. The GUI is intentionally deferred until the server and CLI are
functional.

The GUI module is not implemented yet.

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
8. Choose the video decode backend and add in-app playback.

## Documentation

- [AGENTS.md](AGENTS.md): implementation rules for AI coding agents
- [docs/video-playback.md](docs/video-playback.md): playback architecture and
  decoder backend trade-offs
- `../aiqiezi-server/README.md`: system architecture
- `../aiqiezi-cli/README.md`: shared client library and CLI
