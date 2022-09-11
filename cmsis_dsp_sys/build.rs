use std::process::Command;
use std::string::String;

fn main() {
    let result = Command::new("./build.sh")
        .output()
        .expect("failed to execute process");
    assert!(
        result.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );

    // TODO: This needs to be somehow passed from the bash script?
    use std::env;
    use std::path::{PathBuf};
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("No OUT_DIR"));
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=arm_cortexM7lfdp_math");

    // Don't needlesly re-run the build script
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build.sh");
}
