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

    // Don't needlesly re-run the build script
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build.sh");
}
