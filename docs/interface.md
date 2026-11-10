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

## Clipboard

Long public identifiers are clipboard-friendly:

- setup and Change EndpointId fields have `Paste` and `Copy`;
- Inspector → Server has `Copy` beside both EndpointIds;
- Server Info has `Copy JSON`.

`Paste` trims whitespace and rejects content that is not a valid EndpointId.
Native `Ctrl+V` / `Cmd+V` and `Ctrl+C` / `Cmd+C` continue to work in text
fields. These shortcuts use eframe’s explicit `clipboard` feature, which also
enables `RequestPaste` and native clipboard integration. Password fields are
never copied by the application.

On Linux Wayland, clipboard access depends on the compositor supporting a
data-control protocol. If `Paste` reports an error, use native `Ctrl+V` as the
fallback.

## Menus

- **File**: Refresh; Exit and stop client.
- **Edit**: Clear Search; Settings (deferred).
- **View**: Show/hide Sidebar; Show/hide Inspector; Grid/List view.
- **Server**: Login; Change Password; Server Info; Change EndpointId; Reconnect;
  Refresh Status.

**Reconnect** asks the running daemon to refresh its iroh network state after
sleep or a network change. It does not restart the daemon or the GUI. Read-only
backend requests also retry once after a timeout.
- **Help**: About aiqiezi; Debug submenu.

Login, password change, EndpointId change, server info, and About are dialogs.

## Debug menu

**Help → Debug** provides:

- **Runtime Info…** — CLI path, daemon launch command, data/runtime/socket
  paths, CLI log path, versions, and current EndpointIds;
- **Recent Requests…** — the last 100 GUI-to-daemon requests with duration and
  outcome;
- **Show CLI Log…** — tail of `<AIQIEZI_DATA_DIR>/client.log`;
- **Copy Debug Info** — copies the runtime and request information.

The GUI launches the CLI daemon only once. Other actions are JSON over the local
socket, not separate CLI subprocesses. Recent Request timings are the quickest
way to distinguish GUI, local-socket, and backend latency.

Passwords and request parameters are never included in the debug log.

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
