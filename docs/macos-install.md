# macOS install notes

KeyPoint can be installed like a normal macOS app:

1. Download `KeyPoint-macOS.dmg`.
2. Open the `.dmg`.
3. Drag `KeyPoint.app` to Applications.
4. Eject the mounted `KeyPoint` disk image.
5. Open KeyPoint from Applications.

When the `.dmg` opens, Finder may show `KeyPoint` under Locations near your Mac name. That is only the mounted installer disk image. It is not where KeyPoint is installed.

## Apple cannot check this app

If macOS says Apple cannot check KeyPoint for malicious software, that DMG was not signed and notarized correctly. Do not publish that file as a normal user download. Configure Apple Developer ID signing and notarization, then create a new release.

## Required permissions

KeyPoint needs these macOS permissions:

- Accessibility
- Input Monitoring

Open System Settings, then Privacy & Security, and allow KeyPoint in both sections. These permissions are required because KeyPoint captures global keyboard input and controls the mouse.

## Removing the warning permanently

To remove this warning for all users, KeyPoint must be signed with an Apple Developer ID certificate and notarized by Apple during the release build.
