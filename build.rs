fn main() {
    #[cfg(target_os = "windows")]
    {
        embed_resource::compile("icons/icon.rc", embed_resource::NONE);
    }
}
