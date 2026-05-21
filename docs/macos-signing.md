# macOS signing and notarization

Unsigned macOS apps can be blocked by Gatekeeper with a warning that Apple cannot check the app for malicious software.

To publish a macOS build that opens normally for most users, the release build must be signed and notarized.

## Requirements

- Apple Developer Program membership
- Developer ID Application certificate
- App-specific password for the Apple ID
- Apple Team ID

## GitHub Actions secrets

Add these repository secrets before enabling notarized releases:

- `APPLE_CERTIFICATE`: base64-encoded `.p12` certificate
- `APPLE_CERTIFICATE_PASSWORD`: password for the `.p12`
- `APPLE_SIGNING_IDENTITY`: Developer ID Application identity name
- `APPLE_ID`: Apple ID email
- `APPLE_PASSWORD`: app-specific password
- `APPLE_TEAM_ID`: Apple Team ID

After those secrets are configured, the macOS release workflow can be updated to import the certificate and run the Tauri build with signing and notarization enabled.
