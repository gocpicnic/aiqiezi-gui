<!--
  docs/interface.md
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
-->

# Interface

The GUI follows the LightCraft-style workspace instead of a form-first layout.

## Layout

```text
menu bar
toolbar
┌──────────────┬──────────────────────────────┬────────────────┐
│ Library      │                              │ Inspector      │
│ sidebar      │ asset grid / empty state     │ Info / Server  │
└──────────────┴──────────────────────────────┴────────────────┘
status bar
```

- The left sidebar and right inspector are resizable.
- The central area is reserved for the asset grid; asset browsing is not
  implemented yet.
- The status bar shows daemon connection state, item count, server EndpointId,
  and GUI version.
- There is no startup splash and no logo or wordmark in the workspace. The app
  icon is used only by the operating system.

## Menus

- **File**: Refresh; Exit and stop client.
- **Edit**: Clear Search; Settings (deferred).
- **View**: Show/hide Sidebar; Show/hide Inspector; Grid/List view.
- **Server**: Login; Change Password; Server Info; Change EndpointId; Reconnect;
  Refresh Status.
- **Help**: About aiqiezi.

Login, password change, EndpointId change, server info, and About are dialogs.

## Startup states

1. No server EndpointId: centered setup with instructions to obtain it from
   `aiqiezi-server whoami`.
2. Starting: centered daemon startup indicator.
3. Connected: LightCraft-style workspace.
4. Error: daemon error with Retry.

After first EndpointId setup, the Login dialog opens automatically. Later
launches go directly to the workspace; Login remains under the Server menu.

## Exit behavior

- Window close: exits only the GUI; the local daemon keeps running.
- **File → Exit and stop client**: sends `shutdown` to the local daemon and then
  closes the GUI.

## Deferred

- asset browsing, thumbnails, upload, and download;
- persistent panel sizes and light themes;
- full dockable panels;
- video playback. FilmCraft is the reference for playback only, not for this
  workspace layout.
