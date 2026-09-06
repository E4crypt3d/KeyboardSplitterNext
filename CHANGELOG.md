# Changelog

All notable changes to Keyboard Splitter, explained in plain language.

## [0.3.4] - 2026-09-06

### Fixed
- Fixed a bug where the ViGEmBus download button (and the other external
  links) did nothing: the app's webview silently ignored them, so the driver
  page never opened. They now open in your system browser.
- Fixed a bug where driver trouble - a missing or half-installed ViGEmBus
  driver - could take the whole app down at startup. The engine now survives
  and shows the "driver not found" banner instead, with a **Re-check driver**
  button so you can retry right after installing the driver, no restart
  needed.
- Fixed a bug where crash minidumps were too small to diagnose heap
  corruption: they only stored thread info, not the memory contents that show
  what actually corrupted the heap. Dumps now include full memory, so the
  next crash can be pinpointed.

### Changed
- **The app now runs as administrator** (Windows asks for confirmation each
  time it starts). This gives the driver connection maximum privileges, and
  the old "run as administrator" workaround for invisible gamepads is now
  the default behavior.

## [0.3.3] - 2026-09-06

### Fixed
- Fixed a bug where no keyboard input reached the app at all: a packet-size
  check added in the previous release compared the buffer's length (which
  stayed empty) instead of its actual size, so every key event was rejected
  before it could be processed. Typing, "Assign by key" and live gameplay
  all work again.
- Fixed a bug where a hard crash (the "The instruction at ... referenced
  memory at ..." Windows error) left no trace at all: every crash now also
  writes a short report naming the faulty component, plus a full minidump,
  to `%APPDATA%\com.e4crypt3d.keyboardsplitter\crash-<time>-<id>.log` /
  `.dmp`, so the cause can actually be identified and fixed.
- Fixed a bug where the tray icon could be created empty when the app icon
  was missing - it now always uses a valid image.

### Changed
- **New app icon**: the split-keyboard logo with the two arrows is now used
  everywhere - window, taskbar, system tray and installer.

## [0.3.2] - 2026-09-06

### Fixed
- Fixed a bug where the app could crash with a memory/heap error as soon as it
  opened: a keyboard whose device path ended inside its ID marker made the
  device-name parser read past the end of the string and abort the whole app.
- Fixed a bug where a single malformed input packet or any unexpected internal
  error while listening to keyboards could take the whole app down - the input
  listener now recovers and keeps working instead.
- Fixed a bug where the app closed without a trace when something went wrong
  internally at startup: every crash is now written to
  `%APPDATA%\com.e4crypt3d.keyboardsplitter\crash-<time>-<id>.log`, so a
  report can actually be diagnosed.

### Added
- **Portable version**: every release now also ships as a single zip - unzip
  anywhere and run the app, no installation required (it still needs the
  ViGEmBus driver, like the installer version).

## [0.3.1] - 2026-09-05

### Fixed
- Fixed a bug where every key press was processed up to eight times
  (duplicate events, wasted CPU) - each key event is now handled exactly
  once, with a reused buffer that stops keystrokes from allocating memory.
- Fixed a bug where the app could freeze for a moment when you clicked
  buttons like Start engine or Save profile - those requests now wait for
  the engine on a background task with a timeout instead of blocking the
  interface.
- Fixed a bug where the app kept using a dead driver connection after the
  ViGEmBus driver was updated or reinstalled mid-session - the "Reconnect
  controllers" button now starts over with a fresh connection.

### Changed
- **Security**: the app now runs under a strict Content Security Policy,
  blocking scripts and content that did not come from the app itself.
- All players share one connection to the ViGEmBus driver instead of opening
  one per player - controllers connect faster and the app uses fewer system
  handles.
- The interface is snappier: clicking a button applies the fresh state it
  already received (one round-trip less per click), and rapid change
  notifications are combined into a single update per animation frame.

## [0.3.0] - 2026-09-05

### Added
- **A getting-started checklist** on the Dashboard that walks you through the
  whole setup - install the driver, plug in the keyboards, assign them and
  start the engine - and ticks off each step as you complete it.
- **Assign a keyboard by pressing a key on it** - with two identical
  keyboards you no longer have to guess which list entry is which. Press
  "Assign by key" under a player and tap any key on the keyboard you mean
  (Esc cancels).
- **Test mode** - switch it on, press the keys you bound and watch the
  Dashboard show the exact controller action each key triggers. Verify a
  mapping before you even launch the game.
- **The Mapping editor now tells you which game preset a player is
  currently using** - the picker highlights the matching preset, and shows a
  clear notice when a player's custom layout matches none of the built-in
  ones anymore (e.g. after in-game rebinding).
- A "Not working?" help card on the Dashboard with the answers to the most
  common questions: where virtual pads actually show up (Xbox pads never
  appear in joy.cpl), what to do when the driver is installed but no pad
  appears, and which online modes block virtual controllers.
- Only one copy of the app runs at a time now - launching it again while it
  sits in the system tray just closes the duplicate instead of two copies
  fighting over the same keyboards.

### Fixed
- Fixed a bug where dropdown lists (like the game-preset picker) opened with
  a white background instead of matching the app's dark theme.

## [0.2.0] - 2026-09-05

### Added
- **Built-in game presets** - get started in one click with ready-made button
  layouts for popular games, applyable to any player:
  Football (FIFA / EA FC), Football (eFootball / PES), Mortal Kombat,
  Tekken, Street Fighter 6, arcade racing games, co-op platformers
  (Cuphead, Overcooked, It Takes Two and friends), twin-stick shooters
  and a gamepad-style shooter layout. Every preset can still be fine-tuned.
- A link to the project's GitHub page in the app bar.

### Fixed
- Fixed a bug where the app used a generic identifier that Windows and the
  automated release builder refused - the app now installs and releases
  under its own unique identity.
- Fixed a bug where choosing "unassign" for a keyboard did nothing.
- Fixed a bug where a saved setup could secretly hand the same keyboard to
  two players; each keyboard now always feeds exactly one player.
- Fixed a bug where controller trouble during gameplay could flood the app
  with error notifications - you now only hear about it when something
  actually changes.
- Made gameplay snappier: while you play, keystrokes go straight to the
  virtual controllers instead of being relayed through the interface.

## [0.1.0] - 2026-09-05

### Added
- First release: connect several physical keyboards to one PC and play
  together, each player on their own keyboard.
- Detects every connected keyboard separately, and notices when keyboards
  are plugged in or removed while the app is running.
- Creates up to four virtual Xbox 360 controllers (requires the free
  ViGEmBus driver, installed once).
- Map any key to any controller button, D-pad direction, stick or trigger -
  per player, so everyone controls their own virtual gamepad.
- Save, load and delete named profiles; your setup is auto-saved as you go.
- Runs quietly in the system tray when minimized.
- Automatic updates to profiles when a keyboard is re-plugged.

### Notes
- Some games expect buttons differently per title and year - presets and
  the built-in mapper cover the most common layouts; each game's controller
  options can usually match them too.
- Cheap keyboards may not register every combination of keys pressed at the
  same time (ghosting). Each player has their own keyboard, which keeps
  conflicts rare.
