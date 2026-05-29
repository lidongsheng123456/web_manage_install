fn main() {
    #[cfg(windows)]
    {
        // Embed the Windows privilege manifest into the final executable.
        let windows = tauri_build::WindowsAttributes::new()
            .app_manifest(include_str!("resource/xml/windows_admin_manifest.xml"));

        let attrs = tauri_build::Attributes::new().windows_attributes(windows);
        tauri_build::try_build(attrs).expect("failed to run Tauri build script");
    }

    #[cfg(not(windows))]
    tauri_build::build();
}
