fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/capslang.ico");
    res.set("ProductName", "CapsLang");
    res.set("FileDescription", "CapsLock language switcher for Windows");
    res.set("LegalCopyright", "Copyright © NakornCode");
    if let Err(error) = res.compile() {
        eprintln!("winres warning: {error}");
    }
}
