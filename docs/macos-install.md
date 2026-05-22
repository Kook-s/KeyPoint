# macOS install notes

KeyPoint can be installed from a locally built macOS disk image:

1. Open the built `.dmg`.
2. Drag `KeyPoint.app` to Applications.
3. Eject the mounted `KeyPoint` disk image.
4. Open KeyPoint from Applications.

When the `.dmg` opens, Finder may show `KeyPoint` under Locations near your Mac name. That is only the mounted installer disk image. It is not where KeyPoint is installed.

## Required permissions

KeyPoint needs these macOS permissions:

- Accessibility
- Input Monitoring

Open System Settings, then Privacy & Security, and allow KeyPoint in both sections. These permissions are required because KeyPoint captures global keyboard input and controls the mouse.
