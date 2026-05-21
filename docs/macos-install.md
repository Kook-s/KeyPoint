# macOS install notes

KeyPoint can be installed like a normal macOS app:

1. Download `KeyPoint-macOS.dmg`.
2. Open the `.dmg`.
3. Drag `KeyPoint.app` to Applications.
4. Eject the mounted `KeyPoint` disk image.
5. Open KeyPoint from Applications.

When the `.dmg` opens, Finder may show `KeyPoint` under Locations near your Mac name. That is only the mounted installer disk image. It is not where KeyPoint is installed.

## Apple cannot check this app

If macOS says Apple cannot check KeyPoint for malicious software, the app is not broken. It means this build has not been notarized by Apple yet.

Do not run KeyPoint from the mounted `.dmg` window. Copy it to Applications first.

Try this first:

1. Open Applications.
2. Hold `Control` and click `KeyPoint.app`.
3. Choose Open.
4. Choose Open again in the warning dialog.

If macOS still blocks the app:

1. Open System Settings.
2. Go to Privacy & Security.
3. Scroll to Security.
4. Click Open Anyway for KeyPoint.
5. Confirm Open.

If the app is still blocked, remove the download quarantine flag:

```sh
xattr -rd com.apple.quarantine /Applications/KeyPoint.app
```

Then open KeyPoint from Applications again.

## Required permissions

KeyPoint needs these macOS permissions:

- Accessibility
- Input Monitoring

Open System Settings, then Privacy & Security, and allow KeyPoint in both sections. These permissions are required because KeyPoint captures global keyboard input and controls the mouse.

## Removing the warning permanently

To remove this warning for all users, KeyPoint must be signed with an Apple Developer ID certificate and notarized by Apple during the release build.
