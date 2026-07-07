use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use crate::find_clang::{find_clangxx, find_llvm_config};
use miette::{Context, IntoDiagnostic, Result, ensure};

pub fn build_llvm_plugin() -> Result<String> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("llvm-pass")
        .join("cesty-llvm.cpp");

    println!("cargo:rerun-if-changed={}", src.display());
    let out_dir = env::var("OUT_DIR").into_diagnostic().wrap_err(
        "No OUT_DIR environment variable found. The OUT_DIR variable is set by Cargo during the build process. \
         Ensure that cesty-build is only being used from a build.rs script.",
    )?;
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
    ensure!(
        status.is_ok(),
        "Failed to build the Cesty LLVM plugin. The plugin is a shared library that extends the LLVM compiler. \
         Check that clang++ and llvm-config are properly installed and compatible. \
         See compiler output above for details."
    );

    Ok(out)
}

fn llvm_config(args: &[&str]) -> Result<Vec<String>> {
    let llvm_config_bin = find_llvm_config()?;
    let output = Command::new(llvm_config_bin.clone())
        .args(args)
        .output()
        .into_diagnostic()
        .wrap_err(format!(
            "Failed to execute llvm-config with arguments: {:?}. \
             This tool is needed to determine LLVM compiler flags and libraries.",
            args
        ))?;

    Ok(String::from_utf8(output.stdout)
        .into_diagnostic()
        .wrap_err(format!(
            "llvm-config output is not valid UTF-8. \
             The tool at {:?} produced non-UTF-8 output when called with arguments: {:?}",
            llvm_config_bin, args
        ))?
        .split_whitespace()
        .map(str::to_owned)
        .collect())
}
