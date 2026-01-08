#[cfg(target_os = "windows")]
fn main() {
    // Set the Windows subsystem to "windows" to avoid console window
    println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
}

#[cfg(not(target_os = "windows"))]
fn main() {
    // Do nothing for non-Windows platforms
}
