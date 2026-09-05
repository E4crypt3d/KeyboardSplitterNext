# Keyboard Splitter

A modern Windows app that turns **multiple physical keyboards into independent
virtual Xbox 360 controllers**, so two (up to four) people can play a
controller-only game on one PC - each on their own keyboard.

```
Keyboard A ──► Player 1 ──► Virtual Xbox Controller #1 ──► Game
Keyboard B ──► Player 2 ──► Virtual Xbox Controller #2 ──► Game
```

The games see plain XInput controllers. This is a from-scratch, modern
re-implementation of the classic
[KeyboardSplitterXbox](https://github.com/djlastnight/KeyboardSplitterXbox)
idea: **Tauri v2 + React + Rust**, Raw Input for device detection and
**ViGEmBus** for the virtual controllers (no custom driver is written).

---

## How it works (short version)

1. **Detection** – Windows Raw Input API lists every physical keyboard and
   watches for plug/unplug. Each keyboard gets its own entry with VID/PID.
2. **Capture** – a hidden message-only window receives Raw Input key events
   **including the source device handle**, even while the app is minimized or
   unfocused. Two keyboards pressing `W` are two different events.
3. **Mapping** – every assigned key (`W`, `Space`, `Left`, ...) maps to one
   controller action (buttons, D-Pad, triggers, left/right stick directions).
   Released keys cancel; opposite directions (`W`+`S`) zero out.
4. **Virtual controller** – the engine pushes an XInput report to a
   ViGEmBus-created Xbox 360 controller per player. Games poll XInput and see
   a real gamepad.

Everything (players, keyboard assignments, bindings) is stored as JSON
profiles and auto-saved. The app minimizes to the system tray and keeps
splitting while hidden.

> **Note on input blocking.** Raw Input *observes* keys; it does not swallow
> them (Windows does not allow per-device blocking without a kernel driver).
> That is fine for controller-only games - they ignore the keyboard entirely.
> Tools that must also block the source keyboards (so key presses never reach
> any focused window) use the Interception driver; the capture module is
> isolated behind `src-tauri/src/input/` so such a backend can be added later
> without touching the engine, mapping or UI.

---

## Requirements

- **Windows 10/11** (x64)
- **[ViGEmBus driver](https://github.com/nefarius/ViGEmBus/releases)** - the
  virtual gamepad driver (install once; the app shows a banner if missing)
- [Rust](https://rustup.rs) (stable, MSVC toolchain) and the
  [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) runtime
  (preinstalled on Windows 11 / recent Windows 10)
- [Bun](https://bun.sh) (package manager) - or swap `bun` for `npm`

## Development

```bash
bun install            # frontend deps
cd src-tauri && cargo test   # pure mapping/controller tests (any OS)

# from the project root - starts Vite + the desktop app with hot reload
bun run tauri dev
```

## Production build & installer

```bash
bun run tauri build    # produces an NSIS/MSI installer under src-tauri/target/release/bundle/
```

Install that on each player's machine, then also install ViGEmBus once.

## Continuous integration & releases

Two GitHub Actions workflows live in `.github/workflows/`:

- **CI** (`ci.yml`) - on every push/PR: frontend typecheck/lint/build + Rust
  tests on a Windows runner (compiles the Raw Input + ViGEmBus code paths).
- **Release** (`release.yml`) - **triggers automatically on a version bump**: a
  commit that raises the version in `src-tauri/tauri.conf.json` on the default
  branch builds on Windows and publishes a GitHub Release with the NSIS/MSI
  bundles (the `v<version>` tag is created for you).

Cutting a release is therefore just:

```bash
# bump "version" in src-tauri/tauri.conf.json (and keep package.json in sync),
# add a CHANGELOG.md entry in plain language, then:
git add -A && git commit -m "release v0.2.0" && git push
```

The workflow skips runs whose version was already released, so unrelated
commits do not rebuild. You can also trigger a run manually from the Actions
UI (`workflow_dispatch`). Set `releaseDraft: true` in `release.yml` if you
prefer to review each release before it goes live.

## Typical first run

1. Plug in one keyboard **per player** (they can be identical models - each
   is tracked separately).
2. **Dashboard** → assign each keyboard to a player slot. A virtual Xbox 360
   controller is created per player (Windows plays a connect sound).
3. Press **Start engine**.
4. Give each player a layout in **Mapping**: pick a built-in **game preset**
   (Football, Mortal Kombat, Tekken, racing, platformers, twin-stick and
   more), or build your own - press **Capture key**, tap a key on that
   player's keyboard, then click the controller action
   (e.g. `W` → `Left Stick ↑`, `Space` → `A`).
5. Open your game. Each player controls the game with their own keyboard.
6. Close the window to keep it running in the tray; **Quit** from the tray
   unplugs the controllers cleanly.

## Architecture

```
React UI (Tailwind)                        src/
  │  Tauri IPC (invoke / events)
Rust engine thread                         src-tauri/src/engine.rs
  │
  ├── input::devices   Raw Input enumeration (VID/PID, hot-plug)
  ├── input::capture   message-only window, per-device key events   (Windows)
  ├── mapping          key → controller-action rules + report math  (pure)
  ├── presets          built-in game layouts (FIFA/FC, PES, MK, Tekken, ...)
  └── controller       VirtualController trait
       ├── vigem       ViGEmBus Xbox 360 backend                    (Windows)
       └── (trait keeps it replaceable)

Profiles (JSON, auto-saved)                %APPDATA%/com.e4crypt3d.keyboardsplitter/profiles/
```

Guidelines followed from the spec: no custom driver; the virtual controller
backend is replaceable behind a trait; Windows-specific code lives in
`input/` and `controller/vigem.rs`; core logic is platform independent and
unit-tested (`cargo test`); MVP first (two+ keyboards → controllers → mapping
→ profiles → UI), polished extras (tray, auto-save) layered on top.

## Project structure

```
src/                      React UI
  components/             Dashboard, Mapping editor, Profiles, shared UI
  types.ts                serde-compatible DTO mirrors
  api.ts                  typed invoke()/event wrappers
src-tauri/
  src/
    core/                 shared serde model + key-name tables
    input/                devices.rs, capture.rs (Raw Input, Windows)
    mapping/              bindings + report computation (pure)
    presets.rs            built-in game starter layouts
    controller/           trait + report; vigem.rs (ViGEmBus, Windows)
    engine.rs             engine thread, player/controller lifecycle
    commands.rs           Tauri IPC
    state.rs / lib.rs     app wiring, tray, hide-to-tray
```
