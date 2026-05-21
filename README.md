# KeyPoint

KeyPoint is a local desktop utility that converts selected keyboard input into mouse control while mouse mode is enabled.

KeyPoint is built with Tauri, so end users do not need to run a terminal server. Install the macOS or Windows build from GitHub Releases and run it like a normal desktop app.

Repository: <https://github.com/Kook-s/KeyPoint>

## Download

[Download for macOS](https://github.com/Kook-s/KeyPoint/releases/latest/download/KeyPoint-macOS.dmg)

[Download for Windows](https://github.com/Kook-s/KeyPoint/releases/latest/download/KeyPoint-Windows.exe)

[Download Windows MSI](https://github.com/Kook-s/KeyPoint/releases/latest/download/KeyPoint-Windows.msi)

The download links above install the latest published version directly.

- macOS: open the `.dmg`, then drag KeyPoint to Applications
- Windows: run the downloaded `.exe` installer

On macOS, grant Accessibility and Input Monitoring permission when prompted or from System Settings. KeyPoint needs those permissions to capture global keyboard input and move/click the mouse.

On Windows, the installer uses Microsoft WebView2. If WebView2 is not already installed, the installer downloads it automatically.

All release files are also available at <https://github.com/Kook-s/KeyPoint/releases>.

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

For development:

```sh
npm run tauri:dev
```

## Release

This repository includes a GitHub Actions workflow that builds macOS and Windows installers when a version tag is pushed.

```sh
git tag v0.1.0
git push origin v0.1.0
```

The workflow creates a draft GitHub Release with downloadable installer files. Review the release notes and publish it from GitHub.

After the first release is published, users can always download the latest installer from:

<https://github.com/Kook-s/KeyPoint/releases/latest>

## Settings

Settings are stored as JSON under the user config directory:

- macOS: `~/Library/Application Support/KeyPoint/config.json`
- Windows: `%APPDATA%\KeyPoint\config.json`

All shortcuts can be changed in the settings window. Action keys accept single `A-Z` keys, and the mouse-mode toggle accepts modifier combinations such as `Command + Shift + Z` or `Ctrl + Shift + Z`.

## License

MIT
