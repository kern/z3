use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    if let Ok(output) = Command::new("pkg-config")
        .args(["--libs-only-L", "opus"])
        .output()
        && output.status.success()
    {
        for word in String::from_utf8_lossy(&output.stdout).split_whitespace() {
            if let Some(path) = word.strip_prefix("-L") {
                println!("cargo:rustc-link-search=native={path}");
            }
        }
    }
    println!("cargo:rustc-link-lib=opus");
}
