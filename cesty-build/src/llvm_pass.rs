use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

pub fn build_llvm_plugin() -> PathBuf {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("llvm-pass")
        .join("cesty-llvm.cpp");

    println!("cargo:rerun-if-changed={}", src.display());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("cesty.so");

    println!("cargo::rerun-if-env-changed=CLANGXX");
    let clang_bin = env::var("CLANGXX").unwrap_or("cc".to_string());
    let status = Command::new(clang_bin)
        .args(["-shared", "-fPIC", "-o"])
        .arg(out.clone())
        .arg(src)
        .args(llvm_config(&[
            "--cxxflags",
            "--ldflags",
            "--system-libs",
            "--libs",
            "core",
            "passes",
        ]))
        .status()
        .unwrap();

    assert!(status.success());
    out
}

fn llvm_config(args: &[&str]) -> Vec<String> {
    println!("cargo::rerun-if-env-changed=LLVM_CONFIG");
    let llvm_config_bin = env::var("LLVM_CONFIG").unwrap_or("llvm-config-18".to_string());
    let output = Command::new(llvm_config_bin).args(args).output().unwrap();

    assert!(output.status.success());

    String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}
