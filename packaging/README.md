# Packaging

Lienzo publishes a Linux build. The GNOME application is `lienzo-gnome`.

- Linux: Flatpak. The current manifest still launches the Qt `patchy` binary.
- Windows, macOS, and browser packaging files stay in this directory so upstream commits that touch them can still be read. They are not Lienzo release artifacts.

Release packaging must include:

- Dependency license notices.
- Module SBOM/license metadata for deployed third-party runtime components.
- Debug symbol upload.
- Crash-reporting configuration.
- Auto-update metadata.

The Windows zip package and installer are created by `scripts\release\build-release.bat`; they currently include Qt module SPDX notices copied from the local Qt installation. Publishing automation, notarization, and Linux packaging scripts are placeholders until CI has a real signing and publishing environment.
