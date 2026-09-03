fn main() {
    #[cfg(target_os = "windows")]
    {
        let _ = embed_resource::compile("icons/icon.rc", embed_resource::NONE);
    }
}
