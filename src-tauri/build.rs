fn main() {
    // Only try to link Npcap SDK if the npcap feature is enabled
    #[cfg(feature = "npcap")]
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let npcap_lib_path = std::path::Path::new(&manifest_dir)
            .join("npcap_sdk")
            .join("Lib")
            .join("x64");

        // Only add the link path if the Npcap SDK exists
        if npcap_lib_path.exists() {
            println!(
                "cargo:rustc-link-search=native={}",
                npcap_lib_path.display()
            );
            println!("cargo:warning=Npcap SDK found, Npcap support enabled");
        } else {
            println!(
                "cargo:warning=Npcap SDK not found at expected path. Npcap support will be limited to runtime detection"
            );
        }
    }

    #[cfg(not(feature = "npcap"))]
    {
        println!(
            "cargo:warning=Building without Npcap support (npcap feature disabled). WinDivert will be used for all packet capture."
        );
    }

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
