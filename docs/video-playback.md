<!--
  docs/video-playback.md
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
-->

# Video playback design

## Status

Video playback is deferred until asset browsing, thumbnails, and the CLI are
functional. The architecture is selected; the decode backend is still an open
decision.

This document corrects an earlier claim in the repository: egui has no
built-in video player, but it can display video when the application decodes
frames and presents them as textures.

## What egui can and cannot do

egui cannot:

- demux video containers;
- decode video or audio codecs;
- play audio;
- synchronize audio and video by itself.

egui can:

- paint arbitrary textures with `ui.painter().image(...)`;
- register a wgpu texture through `eframe::egui_wgpu` and receive an
  `egui::TextureId`;
- update that texture every displayed frame;
- request repaints at the display cadence.

Video playback is therefore an application responsibility: decode on worker
threads, upload frames to the GPU, and let egui paint the result. Audio output
and clock synchronization are separate responsibilities.

## Reference implementation: FilmCraft

[FilmCraft](https://github.com/storytold/filmcraft) is a pure-Rust nonlinear
video editor built on egui. It demonstrates the same approach this project
needs. Relevant parts:

| Concern | FilmCraft implementation |
| --- | --- |
| UI | `eframe` / `egui` with the `wgpu` backend |
| Frame presentation | `filmcraft-gpu` composites frames with wgpu and WGSL, registers the output as a native egui texture, and paints it with `ui.painter().image(...)` |
| Texture registration | `renderer.register_native_texture(...)` and `renderer.update_egui_texture_from_wgpu_texture(...)` in `crates/ui-egui/src/app/gpu.rs` |
| Video decoding | Its own pure-Rust decoders and demuxers in `crates/codecs/` (H.264, HEVC, VP8/VP9, AV1, ProRes, DNX, APV, MP4/MOV, Matroska, MXF, MPEG-TS, OGG, AVI) |
| Hardware decoding | `filmcraft-platform` uses VideoToolbox on macOS, Media Foundation + D3D11/DXVA on Windows, and VA-API/NVDEC on Linux, with software fallback |
| Audio output | `cpal` in `apps/filmcraft/src/audio.rs`; the played audio clock drives playback |
| Playback scheduling | Preroll, decode-ahead workers, bounded frame queues, and repaint scheduling |
| Not used | No FFmpeg, GStreamer, libmpv, embedded browser, or HTTP streaming server at runtime |

FilmCraft is a large project with dozens of internal codec crates. Those crates
are not published libraries and should not be copied wholesale. The reusable
part is the architecture, not the codec implementations.

## Proposed aiqiezi-gui pipeline

```text
UI action
  -> command channel
  -> playback worker
       +-- demux / decode video
       |     -> bounded frame queue
       |     -> wgpu texture
       |     -> egui TextureId
       |     -> ui.painter().image(...)
       |
       +-- demux / decode audio
             -> bounded audio ring buffer
             -> cpal output stream
             -> audio clock = playback master
```

Rules:

- decoding and demuxing run off the egui render thread;
- the UI thread only uploads/presents the latest frame and paints it;
- frame and audio queues are bounded; when the decoder falls behind, drop
  frames instead of increasing latency;
- the audio clock is the playback master; use a wall clock when there is no
  audio track or no output device;
- seeking flushes queues and restarts decoding from the nearest keyframe;
- playback reads from a local cache file downloaded through
  `aiqiezi-client`; do not add an HTTP streaming server.

## Decode backend options

| Option | Format coverage | Cost | Notes |
| --- | --- | --- | --- |
| FFmpeg via `ffmpeg-next` or `video-rs` | Excellent | Medium | Recommended default for a phone/camera archive; adds C libraries and packaging/licensing work |
| Embedded libmpv | Excellent | Medium-high | Mature playback, but an external render surface, a C dependency, and duplicated playback UI |
| Pure-Rust codecs | Limited to what is implemented | Very high | FilmCraft-scale effort; not realistic for this project |
| External player | Excellent | Low | No in-app playback; keep as a fallback for unsupported formats |

Recommendation for the first in-app player: FFmpeg behind a small
`VideoDecoder` abstraction. That keeps format coverage broad now and allows a
future pure-Rust or libmpv backend without changing the UI or playback
scheduler.

## Audio

- Use `cpal` for desktop output.
- Decode audio with the selected backend; FFmpeg can provide both audio and
  video decoding.
- Resample to the output device rate and mix to the device channel layout.
- Keep a small play-ahead buffer, not an unbounded queue.
- If no audio device is available, continue video playback silently with a
  wall clock.

## Performance and memory

- Cache thumbnails separately from playback data.
- Limit the decoded frame cache by count and bytes.
- Limit the local playback cache by size and evict old entries.
- Do not load a whole video into memory.
- Show explicit states: loading, buffering, playing, paused, ended, and error.

## Phased rollout

1. Thumbnails and still-image previews.
2. Server-side video metadata: duration, dimensions, codec, and container.
3. Local download of the original through `aiqiezi-client`.
4. External-player fallback for unsupported formats.
5. In-app playback using the texture pipeline above.
6. Audio-clock synchronization and hardware decoding evaluation.

## Open decisions

- Final decode backend and its packaging strategy.
- Whether hardware decoding is required for the first release.
- Local playback cache location, size limit, and eviction policy.
- Whether audio is required in the first in-app player or video-only playback is
  acceptable.
