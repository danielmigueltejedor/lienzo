//! The names this app's release scripts give its packages, checked against what the in-app
//! updater looks for on each system. A rename in `packaging/` that the updater does not know
//! about would leave people on the old version without a word, so it fails here instead.
#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use lienzo_ui_egui::UPDATE_APP;

fn packaging(path: &str) -> String {
    let file = format!("{}/../../packaging/{path}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{file}: {e}"))
}

#[test]
fn the_release_scripts_make_the_packages_the_updater_installs() {
    let macos = packaging("macos/package.sh");
    assert!(macos.contains(r#"APP="$WORK/Lienzo.app""#), "the bundle the updater swaps");
    assert!(macos.contains(r#"DMG="$DIST/lienzo-$VERSION-macos-$ARCH.dmg""#));
    assert!(macos.contains("ARCH=universal"));
    let linux = packaging("linux/package.sh");
    assert!(linux.contains(r#"BASENAME="lienzo-$VERSION-linux-$ARCH""#));
    assert!(linux.contains("$BASENAME.AppImage"));
    assert!(packaging("windows/package.ps1").contains(r#""lienzo-$Version-windows-$Arch.msi""#));
    assert!(packaging("windows/lienzo.wxs").contains(r#"Key="Software\Lienzo" Name="InstallDir" Value="[INSTALLFOLDER]""#));
    assert_eq!(UPDATE_APP.bundle, "Lienzo.app");
    assert_eq!(UPDATE_APP.repo, "getnodalia/lienzo");
}

#[test]
fn every_published_package_reaches_the_systems_it_is_for() {
    for (package, systems) in [
        ("lienzo-1.0.0-macos-universal.dmg", &[("macos", "aarch64"), ("macos", "x86_64")][..]),
        ("lienzo-1.0.0-linux-x86_64.AppImage", &[("linux", "x86_64")]),
        ("lienzo-1.0.0-linux-aarch64.AppImage", &[("linux", "aarch64")]),
        ("lienzo-1.0.0-windows-x64.msi", &[("windows", "x86_64")]),
        ("lienzo-1.0.0-windows-arm64.msi", &[("windows", "aarch64")]),
    ] {
        let found = nodalia_update::systems_for_package(package);
        for system in systems {
            assert!(found.contains(system), "{package} is not offered to {system:?}");
        }
    }
}
