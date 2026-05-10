fn main() {
    // 仅在 Windows 上嵌入图标
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/app.ico");
        res.compile().expect("Failed to compile Windows resources");
    }
}