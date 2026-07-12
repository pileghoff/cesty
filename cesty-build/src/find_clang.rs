use miette::{Context, Result};
use std::{env, path::PathBuf};
use which::which;

pub fn find_clang_generic(bin_name: &str, env_name: &str) -> Option<PathBuf> {
    println!("cargo::rerun-if-env-changed={}", env_name);
    if let Ok(bin) = env::var(env_name) {
        return Some(PathBuf::from(bin));
    }

    for postfix in ["", "-22", "-21", "-20", "-19", "-18", "-17", "-16"] {
        if let Ok(bin) = which(format!("{}{}", bin_name, postfix)) {
            return Some(bin);
        }
    }

    None
}

pub fn find_clang() -> Result<PathBuf> {
    find_clang_generic("clang", "CLANG").wrap_err(
        "Could not find 'clang' compiler. Checked for: clang, clang-22, clang-21, ..., clang-16\n\
         You can set the CLANG environment variable to specify the path to a clang executable.\n\
         Install LLVM/Clang from: https://llvm.org/download.html",
    )
}

pub fn find_clangxx() -> Result<PathBuf> {
    find_clang_generic("clang++", "CLANGXX").wrap_err(
        "Could not find 'clang++' compiler. Checked for: clang++, clang++-22, clang++-21, ..., clang++-16\n\
         You can set the CLANGXX environment variable to specify the path to a clang++ executable.\n\
         Install LLVM/Clang from: https://llvm.org/download.html"
    )
}

pub fn find_llvm_config() -> Result<PathBuf> {
    find_clang_generic("llvm-config", "LLVM_CONFIG").wrap_err(
        "Could not find 'llvm-config' tool. Checked for: llvm-config, llvm-config-22, llvm-config-21, ..., llvm-config-16\n\
         You can set the LLVM_CONFIG environment variable to specify the path to an llvm-config executable.\n\
         Install LLVM from: https://llvm.org/download.html"
    )
}
