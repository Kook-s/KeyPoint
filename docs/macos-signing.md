# macOS signing and notarization

Unsigned macOS apps can be blocked by Gatekeeper with a warning that Apple cannot check the app for malicious software.

To publish a macOS build that opens normally for most users, the release build must be signed and notarized.

## Requirements

- Apple Developer Program membership
- Developer ID Application certificate
- App-specific password for the Apple ID
- Apple Team ID

## GitHub Actions secrets

The macOS release workflow requires these repository secrets. Without them, the macOS release job fails instead of uploading an unsigned DMG:

- `APPLE_CERTIFICATE`: base64-encoded `.p12` certificate
- `APPLE_CERTIFICATE_PASSWORD`: password for the `.p12`
- `APPLE_ID`: Apple ID email
- `APPLE_PASSWORD`: app-specific password
- `APPLE_TEAM_ID`: Apple Team ID
- `KEYCHAIN_PASSWORD`: temporary CI keychain password

The `.p12` must contain a `Developer ID Application` certificate. The workflow imports it, signs the app, submits it to Apple for notarization, staples the result, and only then uploads the DMG release asset.

Create the base64 certificate value on macOS:

```sh
openssl base64 -A -in /path/to/developer-id-application.p12 -out certificate-base64.txt
```

Use the contents of `certificate-base64.txt` as `APPLE_CERTIFICATE`.
