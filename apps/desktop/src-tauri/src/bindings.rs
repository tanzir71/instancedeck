pub fn commands() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::new()
        // The JSON envelope uses JavaScript numbers. Profile IDs and stored
        // dates remain strings; request numeric fields are bounded u32.
        .dangerously_cast_bigints_to_number()
        .commands(tauri_specta::collect_commands![
            crate::commands::desktop_request
        ])
}

#[cfg(test)]
mod tests {
    #[test]
    fn export_frontend_bindings_without_starting_a_window() {
        super::commands()
            .export(
                specta_typescript::Typescript::default(),
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/bindings.ts"),
            )
            .expect("Rust command types must export to TypeScript");
    }
}
