# KeyPoint

KeyPoint는 마우스 모드가 켜져 있을 때 지정한 키보드 입력을 마우스 조작으로 바꿔 주는 로컬 데스크톱 유틸리티입니다.

Tauri 기반으로 만들어졌으며 macOS와 Windows에서 데스크톱 앱으로 실행됩니다.

## 조작 방법

- 마우스 모드 전환: macOS 기본값은 `Command + Shift + Z`, Windows 기본값은 `Ctrl + Shift + Z`
- 커서 이동: `W` `A` `S` `D`
- 좌클릭: `J` 짧게 누르기
- 드래그: `J`를 누른 채 `W` `A` `S` `D`로 이동
- 우클릭: `L`
- 스크롤: `K`를 누른 채 `W` `A` `S` `D`

## 소스에서 빌드

준비 사항:

- Node.js 20 이상
- Rust stable
- 사용하는 운영체제에 맞는 Tauri 플랫폼 요구 사항

```sh
npm install
npm run tauri:build
```

빌드 결과물은 `src-tauri/target/release/bundle/` 아래에 생성됩니다.

macOS에서 `.dmg` 번들링 단계가 Finder 제어 권한 문제로 실패하면 앱 번들만 직접 빌드할 수 있습니다.

```sh
npm run tauri:build:app
```

Windows에서 설치 가능한 `.exe`와 `.msi` 파일을 빌드하려면 다음 명령을 실행합니다.

```sh
npm run tauri:build:windows
```

Windows 설치 파일은 다음 경로에 생성됩니다.

- `src-tauri/target/release/bundle/nsis/`
- `src-tauri/target/release/bundle/msi/`

Windows 빌드 체크리스트는 `docs/windows-build.md`를 참고하세요.

개발 모드 실행:

```sh
npm run tauri:dev
```

## 설정

설정은 사용자 설정 디렉터리의 JSON 파일에 저장됩니다.

- macOS: `~/Library/Application Support/KeyPoint/config.json`
- Windows: `%APPDATA%\KeyPoint\config.json`

설정 창에서 모든 단축키를 바꿀 수 있습니다. 동작 키는 단일 `A-Z` 키를 사용할 수 있고, 마우스 모드 전환 키는 `Command + Shift + Z` 또는 `Ctrl + Shift + Z` 같은 조합을 사용할 수 있습니다.

## 라이선스

MIT

<details>
<summary>English</summary>

KeyPoint is a local desktop utility that converts selected keyboard input into mouse control while mouse mode is enabled.

KeyPoint is built with Tauri and runs as a desktop app on macOS and Windows.

### Controls

- Toggle mouse mode: `Command + Shift + Z` on macOS, `Ctrl + Shift + Z` on Windows by default
- Move cursor: `W` `A` `S` `D`
- Left click: short press `J`
- Drag: hold `J` and move with `W` `A` `S` `D`
- Right click: `L`
- Scroll: hold `K` and press `W` `A` `S` `D`

### Build From Source

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

### Settings

Settings are stored as JSON under the user config directory:

- macOS: `~/Library/Application Support/KeyPoint/config.json`
- Windows: `%APPDATA%\KeyPoint\config.json`

All shortcuts can be changed in the settings window. Action keys accept single `A-Z` keys, and the mouse-mode toggle accepts modifier combinations such as `Command + Shift + Z` or `Ctrl + Shift + Z`.

### License

MIT

</details>
