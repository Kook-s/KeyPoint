# KeyPoint

[한국어](README.md)

KeyPoint is a local desktop utility that converts selected keyboard input into mouse control while mouse mode is enabled.

KeyPoint is built with Tauri and runs as a desktop app on macOS and Windows.

## Controls

- Toggle mouse mode: `Command + Shift + Z` on macOS, `Ctrl + Shift + Z` on Windows by default
- Move cursor: `W` `A` `S` `D`
- Left click: short press `J`
- Drag: hold `J` and move with `W` `A` `S` `D`
- Right click: `L`
- Scroll: hold `K` and press `W` `A` `S` `D`

## Build From Source

Prerequisites:

- Node.js 20 or newer
- Rust stable
- Tauri platform prerequisites for your OS

```sh
npm install
npm run tauri:build
```

Build outputs are created under `src-tauri/target/release/bundle/`.

On macOS, if the `.dmg` bundling step fails because the terminal cannot control Finder, build the app bundle directly:

```sh
npm run tauri:build:app
```

On Windows, build installable `.exe` and `.msi` files:

```sh
npm run tauri:build:windows
```

Windows installer outputs are created under:

- `src-tauri/target/release/bundle/nsis/`
- `src-tauri/target/release/bundle/msi/`

See `docs/windows-build.md` for the full Windows build checklist.

Windows distribution files are published through GitHub Releases. Pushing a version tag such as `v0.1.4` runs GitHub Actions on Windows, builds the `.exe` and `.msi` installers, and uploads them to that release.

For development:

```sh
npm run tauri:dev
```

## Settings

Settings are stored as JSON under the user config directory:

- macOS: `~/Library/Application Support/KeyPoint/config.json`
- Windows: `%APPDATA%\KeyPoint\config.json`

All shortcuts can be changed in the settings window. Action keys accept single `A-Z` keys, and the mouse-mode toggle accepts modifier combinations such as `Command + Shift + Z` or `Ctrl + Shift + Z`.
