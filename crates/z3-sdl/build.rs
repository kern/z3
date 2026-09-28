use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    let output = Command::new("pkg-config")
        .args(["--libs-only-L", "--libs-only-l", "sdl2"])
        .output()
        .expect("pkg-config is required to locate SDL2");
    assert!(
        output.status.success(),
        "SDL2 development files are required"
    );
    let flags = String::from_utf8(output.stdout).expect("pkg-config output is UTF-8");
    for flag in flags.split_whitespace() {
        if let Some(path) = flag.strip_prefix("-L") {
            println!("cargo:rustc-link-search=native={path}");
        } else if let Some(library) = flag.strip_prefix("-l") {
            println!("cargo:rustc-link-lib={library}");
        }
    }
}
