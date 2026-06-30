use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    CestyBuildError,
    find_clang::{find_clangxx, find_llvm_config},
};

pub fn build_llvm_plugin() -> Result<String, CestyBuildError> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("llvm-pass")
        .join("cesty-llvm.cpp");

    println!("cargo:rerun-if-changed={}", src.display());
    let out_dir = env::var("OUT_DIR").map_err(|_| CestyBuildError::MissingOutDir)?;
    let out = PathBuf::from(out_dir)
        .join("cesty.so")
        .to_str()
        .unwrap()
        .to_string();

    println!("cargo::rerun-if-env-changed=CLANGXX");
    let clang_bin = find_clangxx()?;
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
        ])?)
        .status();
    if status.is_err() {
        return Err(CestyBuildError::PluginBuildFailed);
    }

    Ok(out)
}

fn llvm_config(args: &[&str]) -> Result<Vec<String>, CestyBuildError> {
    let llvm_config_bin = find_llvm_config()?;
    let output = Command::new(llvm_config_bin)
        .args(args)
        .output()
        .map_err(|_| CestyBuildError::LlvmConfigFailed)?;

    Ok(String::from_utf8(output.stdout)
        .map_err(|_| CestyBuildError::LlvmConfigFailed)?
        .split_whitespace()
        .map(str::to_owned)
        .collect())
}
