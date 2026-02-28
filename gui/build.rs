// Build script: compiles the C++ slowhttptest binary and places it in OUT_DIR
// so that it can be embedded into the GUI executable via include_bytes!.
//
// On non-Unix targets the C++ code cannot compile (it requires POSIX APIs),
// so we write an empty placeholder and the Rust code skips embedding.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target_family = env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or_default();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let project_root = manifest_dir
        .parent()
        .expect("gui directory must be inside the project root");

    // Register rerun-if-changed for C++ sources and build system files
    let src_dir = project_root.join("src");
    if let Ok(entries) = std::fs::read_dir(&src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "cc" || ext == "h" {
                    println!("cargo:rerun-if-changed={}", path.display());
                }
            }
        }
    }
    println!(
        "cargo:rerun-if-changed={}",
        project_root.join("configure").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        project_root.join("configure.ac").display()
    );

    let binary_path = out_dir.join("slowhttptest");

    if target_family != "unix" {
        // Non-Unix: create an empty placeholder so include_bytes! compiles
        std::fs::write(&binary_path, b"").expect("Failed to write placeholder");
        return;
    }

    // Out-of-tree build to avoid polluting the source tree
    let build_dir = out_dir.join("slowhttptest-build");
    std::fs::create_dir_all(&build_dir).expect("Failed to create build directory");

    // Run configure
    let configure_script = project_root.join("configure");
    let status = Command::new(configure_script)
        .arg("--disable-dependency-tracking")
        .env("CXXFLAGS", "-O2")
        .current_dir(&build_dir)
        .status()
        .expect("Failed to run configure – is a C++ compiler installed?");
    assert!(
        status.success(),
        "configure failed with exit code {:?}",
        status.code()
    );

    // Run make (only build the binary, skip install)
    let status = Command::new("make")
        .arg("-j2")
        .current_dir(&build_dir)
        .status()
        .expect("Failed to run make");
    assert!(
        status.success(),
        "make failed with exit code {:?}",
        status.code()
    );

    // Strip the binary to reduce embedded size
    let built_binary = build_dir.join("src").join("slowhttptest");
    assert!(
        built_binary.exists(),
        "slowhttptest binary not found at {}",
        built_binary.display()
    );

    let _ = Command::new("strip")
        .arg(&built_binary)
        .status();

    std::fs::copy(&built_binary, &binary_path).expect("Failed to copy slowhttptest to OUT_DIR");
}
