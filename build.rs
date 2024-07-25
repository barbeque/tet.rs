#[cfg(target_os = "macos")]
fn main() {
    // For Mac only, point the linker at /Library/Frameworks
    //println!("cargo:rustc-link-search=framework=/Library/Frameworks");
    //println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
}

#[cfg(not(target_os = "macos"))]
fn main() {

}
