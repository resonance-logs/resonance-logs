fn main() {
    // Use the standard debug_assertions cfg to differentiate dev vs release.
    // The previous cfg!(dev) was not a recognized configuration predicate,
    // causing the release branch (with manifest build) to run even in dev builds.
    if cfg!(debug_assertions) {
        println!("DEBUG (dev) BUILD");
        tauri_build::build();
    } else {
        let mut windows = tauri_build::WindowsAttributes::new();
        windows = windows.app_manifest(include_str!("app.manifest"));
        tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
            .expect("failed to run build script");
    }
}
