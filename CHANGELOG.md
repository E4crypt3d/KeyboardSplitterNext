# Changelog

All notable changes to Keyboard Splitter, explained in plain language.

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
