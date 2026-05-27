# Windows installer build

Windows installers must be built on Windows or in a Windows CI runner.

## Prerequisites

- Windows 10 or newer
- Node.js 20 or newer
- Rust stable
- Microsoft Visual Studio Build Tools with the C++ desktop workload
- WebView2 runtime, or internet access during installation so the installer can download it

## Build

From PowerShell:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
.\scripts\build-windows.ps1
```

Or run the commands manually:

```powershell
npm ci
npm run tauri:build:windows
```

The installable outputs are created here:

- `src-tauri\target\release\bundle\nsis\` for the `.exe` installer
- `src-tauri\target\release\bundle\msi\` for the `.msi` installer

The NSIS installer installs for the current Windows user by default, so administrator permission is not required for normal installation.

## GitHub release

Pushing a version tag builds the Windows installers in GitHub Actions and uploads them to the matching GitHub Release:

```sh
git tag v0.1.5
git push origin main v0.1.5
```

The downloadable release assets are the generated `.exe` NSIS installer and `.msi` installer. The workflow can also be run manually from GitHub Actions; manual runs keep the files as workflow artifacts instead of publishing a release.
