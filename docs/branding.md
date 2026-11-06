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
└── aiqiezi.png   # runtime window and taskbar icon
```

Both files are embedded at build time. The application does not load branding
files from the runtime directory.

- `aiqiezi.png` is a 340×340 RGBA PNG used as the native window/taskbar icon.
- `aiqiezi.ico` contains 16, 32, 48, 64, 128, and 256 pixel images. Windows
  builds embed it in the executable through `winresource` in `build.rs`.
- Linux and macOS builds do not invoke the Windows resource compiler.

There is no startup splash. The main workspace does not display the logo,
wordmark, or a brand banner; it follows the LightCraft-style chrome-only layout.
The icon is used only by the operating system.

## Workspace style

- LightCraft-inspired dark chrome and a blue accent.
- Menu bar, toolbar, resizable library sidebar, central asset grid, right
  inspector, and bottom status bar.
- No logo inside the application workspace.
- FilmCraft is not a layout reference; it is only a reference for future video
  playback.

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
