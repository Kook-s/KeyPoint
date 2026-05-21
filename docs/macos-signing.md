# macOS signing and notarization

Unsigned macOS apps can be blocked by Gatekeeper with a warning that Apple cannot check the app for malicious software.

To publish a macOS build that opens normally for most users, the release build must be signed and notarized.

Apple requires this for apps distributed outside the Mac App Store:

- Developer ID signing lets Gatekeeper verify the developer.
- Notarization lets Apple scan the signed app and issue a ticket Gatekeeper can trust.

References:

- <https://developer.apple.com/developer-id/>
- <https://developer.apple.com/support/developer-id/>
- <https://v2.tauri.app/distribute/sign/macos/>

## What you need

- Apple Developer Program membership
- Developer ID Application certificate
- App-specific password for the Apple ID
- Apple Team ID
- GitHub repository admin access to add Actions secrets

The Apple Developer Program is required. A free Apple ID is not enough for public notarized distribution.

## 1. Join Apple Developer Program

1. Go to <https://developer.apple.com/programs/>.
2. Enroll with the Apple ID that will own the app distribution.
3. Complete Apple Developer Program payment and approval.

Only the account holder can create Developer ID certificates.

## 2. Create a Certificate Signing Request

On your Mac:

1. Open Keychain Access.
2. In the menu bar, choose Keychain Access, then Certificate Assistant, then Request a Certificate From a Certificate Authority.
3. Enter your Apple ID email.
4. Select Saved to disk.
5. Save the CSR file.

## 3. Create a Developer ID Application certificate

1. Open <https://developer.apple.com/account/resources/certificates/list>.
2. Click the add button.
3. Select Developer ID Application.
4. Upload the CSR file.
5. Download the generated `.cer` certificate.
6. Open the `.cer` file on your Mac so it is added to Keychain Access.

In Keychain Access, the certificate must appear under My Certificates and include its private key.

## 4. Export the certificate as `.p12`

1. Open Keychain Access.
2. Select My Certificates.
3. Find `Developer ID Application: ...`.
4. Expand it and make sure the private key is present.
5. Right-click the certificate entry.
6. Choose Export.
7. Save it as `developer-id-application.p12`.
8. Set a strong export password. This password becomes `APPLE_CERTIFICATE_PASSWORD`.

## 5. Create an app-specific password

1. Open <https://account.apple.com/account/manage>.
2. Sign in with the Apple ID used for Developer Program access.
3. Create an app-specific password.
4. Save it. This value becomes `APPLE_PASSWORD`.

## 6. Find your Team ID

Open <https://developer.apple.com/account/> and check Membership details.

The Team ID value becomes `APPLE_TEAM_ID`.

## GitHub Actions secrets

The macOS release workflow requires these repository secrets. Without them, the macOS release job fails instead of uploading an unsigned DMG:

- `APPLE_CERTIFICATE`: base64-encoded `.p12` certificate
- `APPLE_CERTIFICATE_PASSWORD`: password for the `.p12`
- `APPLE_ID`: Apple ID email
- `APPLE_PASSWORD`: app-specific password
- `APPLE_TEAM_ID`: Apple Team ID
- `KEYCHAIN_PASSWORD`: temporary CI keychain password

The `.p12` must contain a `Developer ID Application` certificate. The workflow imports it, signs the app, submits it to Apple for notarization, staples the result, and only then uploads the DMG release asset.

## 7. Prepare GitHub secret values

Run this from the repository root on your Mac:

```sh
./scripts/prepare-macos-signing-secrets.sh /path/to/developer-id-application.p12
```

It writes:

- `certificate-base64.txt`: use its contents for `APPLE_CERTIFICATE`
- `keychain-password.txt`: use its contents for `KEYCHAIN_PASSWORD`

Set the remaining values manually:

- `APPLE_CERTIFICATE_PASSWORD`: the password you used when exporting the `.p12`
- `APPLE_ID`: your Apple ID email
- `APPLE_PASSWORD`: app-specific password
- `APPLE_TEAM_ID`: your Apple Team ID

## 8. Add GitHub repository secrets

1. Open <https://github.com/Kook-s/KeyPoint/settings/secrets/actions>.
2. Click New repository secret.
3. Add all six secrets listed above.

## 9. Create a notarized release

After secrets are configured:

```sh
git tag v0.1.4
git push origin v0.1.4
```

GitHub Actions will build a signed and notarized `KeyPoint-macOS.dmg`. After the workflow finishes, the direct download link will point to the notarized DMG:

<https://github.com/Kook-s/KeyPoint/releases/latest/download/KeyPoint-macOS.dmg>

## Verification

After downloading the release DMG, the app should open without the "Apple cannot check this app" block.

If it still appears, the release was not notarized correctly. Check the macOS job logs for signing or notarization errors.
