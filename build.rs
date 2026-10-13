//! Embeds the app icon and version information in the Windows executable, so Explorer,
//! the taskbar and installer shortcuts show the right icon.

fn main() {
    println!("cargo:rerun-if-changed=docs/assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon("docs/assets/icon.ico")
        .set("ProductName", "Internet Speed Monitor")
        .set("FileDescription", "Internet Speed Monitor")
        .set("CompanyName", "Omri Lotan")
        .set("LegalCopyright", "Unlicense");
    resource
        .compile()
        .expect("embedding Windows resources needs the MSVC resource compiler (rc.exe)");
}
