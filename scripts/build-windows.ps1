$ErrorActionPreference = "Stop"

Write-Host "Installing Node dependencies..."
npm ci

Write-Host "Building KeyPoint Windows installers..."
npm run tauri:build:windows

Write-Host ""
Write-Host "Done. Installer outputs:"
Write-Host "  src-tauri\target\release\bundle\nsis"
Write-Host "  src-tauri\target\release\bundle\msi"
