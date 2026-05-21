# macOS install notes

KeyPoint can be installed like a normal macOS app:

1. Download `KeyPoint-macOS.dmg`.
2. Open the `.dmg`.
3. Drag `KeyPoint.app` to Applications.
4. Open KeyPoint from Applications.

## Apple cannot check this app

If macOS says Apple cannot check KeyPoint for malicious software, the app is not broken. It means this build has not been notarized by Apple yet.

Try this first:

1. Open Applications.
2. Hold `Control` and click `KeyPoint.app`.
3. Choose Open.
4. Choose Open again in the warning dialog.

If macOS still blocks the app, remove the download quarantine flag:

```sh
xattr -rd com.apple.quarantine /Applications/KeyPoint.app
```

Then open KeyPoint again.

## Required permissions

KeyPoint needs these macOS permissions:

- Accessibility
- Input Monitoring

Open System Settings, then Privacy & Security, and allow KeyPoint in both sections. These permissions are required because KeyPoint captures global keyboard input and controls the mouse.

## Removing the warning permanently

To remove this warning for all users, KeyPoint must be signed with an Apple Developer ID certificate and notarized by Apple during the release build.
