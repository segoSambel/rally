fn main() {
    // Embed the app icon into the Windows executable.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("rally.rc", embed_resource::NONE)
            .manifest_required()
            .unwrap();
    }
}
