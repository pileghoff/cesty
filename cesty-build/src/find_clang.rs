use std::{env, path::PathBuf};
use which::which;

use crate::CestyBuildError;

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

pub fn find_clang() -> Result<PathBuf, CestyBuildError> {
    find_clang_generic("clang", "CLANG").ok_or(CestyBuildError::ClangNotFound)
}

pub fn find_clangxx() -> Result<PathBuf, CestyBuildError> {
    find_clang_generic("clang++", "CLANGXX").ok_or(CestyBuildError::ClangxxNotFound)
}

pub fn find_llvm_config() -> Result<PathBuf, CestyBuildError> {
    find_clang_generic("llvm-config", "LLVM_CONFIG").ok_or(CestyBuildError::ClangxxNotFound)
}
