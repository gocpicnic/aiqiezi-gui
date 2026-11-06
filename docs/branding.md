<!--
  docs/branding.md
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
-->

# Branding and typography

## Assets

Branding files live under `assets/`:

```text
assets/
├── aiqiezi.ico   # Windows executable icon
└── aiqiezi.png   # splash and runtime window icon
```

Both files are embedded at build time. The application does not load branding
files from the runtime directory.

- `aiqiezi.png` is a 340×340 RGBA PNG. It is decoded into an egui texture for
  the startup splash and used as the native window/taskbar icon.
- `aiqiezi.ico` contains 16, 32, 48, 64, 128, and 256 pixel images. Windows
  builds embed it in the executable through `winresource` in `build.rs`.
- Linux and macOS builds do not invoke the Windows resource compiler.

## Startup splash

The splash appears for 1.5 seconds at process startup:

- dark background;
- centered aiqiezi logo;
- title and subtitle;
- loading indicator.

The splash is presentation-only. Daemon startup and worker events continue in
the background, and the normal setup, starting, or login screen appears after
the timeout.

## Typography

The GUI intentionally uses egui's built-in embedded fonts at this phase. No
custom font files are bundled.

- Proportional UI text: **Ubuntu-Light**.
- Monospace text: **Hack**.
- egui-icons is used as the icon fallback.

The fonts are embedded by `epaint`/egui, so rendering is consistent across the
Tier 1 desktop targets. EndpointIds, socket paths, and JSON views use the
monospace family; headings, labels, buttons, and forms use the proportional
family.

Do not add a custom font without an explicit product decision. A future custom
font must:

- have a license compatible with AGPL-3.0 distribution;
- be embedded in the binary;
- retain the egui defaults as fallbacks;
- be documented here;
- be verified on Windows, macOS, and Linux.
