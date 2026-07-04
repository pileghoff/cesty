use std::{
    collections::VecDeque,
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use object::{Object, ObjectSymbol, read::archive::ArchiveFile};

use find_clang::find_clang;
use llvm_pass::build_llvm_plugin;

use thiserror::Error;

mod find_clang;
mod llvm_pass;

/// Reads `[package.metadata.c_tests]` from the current package manifest and
/// compiles each declared C test library.
///
/// This is intended to be called from a consuming crate's `build.rs`:
///
/// ```no_run
/// fn main() {
///     cesty_build::build_c_tests();
/// }
/// ```
pub fn build_c_tests() {
    if let Err(error) = try_build_c_tests() {
        panic!("cesty C test build failed: {error}");
    }
}

/// Fallible variant of [`build_c_tests`].
pub fn try_build_c_tests() -> Result<(), CestyBuildError> {
    let manifest_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");

    build_c_tests_from_manifest(&manifest_path)
}

fn get_manifest(manifest_path: &Path) -> Result<toml::Value, CestyBuildError> {
    let manifest =
        fs::read_to_string(manifest_path).map_err(|cause| CestyBuildError::ManifestReadError {
            path: manifest_path.to_path_buf(),
            cause,
        })?;
    let manifest =
        manifest
            .parse::<toml::Value>()
            .map_err(|cause| CestyBuildError::ManifestParseError {
                path: manifest_path.to_path_buf(),
                cause,
            })?;

    Ok(manifest)
}

/// Builds C test libraries from an explicit manifest path.
pub fn build_c_tests_from_manifest(manifest_path: &Path) -> Result<(), CestyBuildError> {
    let manifest = get_manifest(manifest_path)?;

    let manifest_dir = manifest_path.parent().unwrap();

    let Some(c_tests) = manifest.get("cesty").and_then(toml::Value::as_table) else {
        return Ok(());
    };

    for (test_name, config) in c_tests {
        let config = config
            .as_table()
            .ok_or_else(|| CestyBuildError::ManifestTestParseError {
                section: test_name.clone(),
                message: "expected a table".to_owned(),
            })?;

        let sources = string_array(config, test_name, "sources", true)?;
        let includes = string_array(config, test_name, "includes", false)?;

        let mut build = cc::Build::new();
        let clang = find_clang()?;
        build.compiler(clang);

        for source in &sources {
            let path = manifest_dir.join(source);
            println!("cargo:rerun-if-changed={}", path.display());
            build.file(path);
        }
        let out_dir = env::var_os("OUT_DIR").unwrap();

        let shadow_include_path = Path::new(&out_dir).join("shadow_include").join(test_name);

        build.include(&shadow_include_path);
        let _ = fs::remove_dir_all(&shadow_include_path);
        fs::create_dir_all(&shadow_include_path).map_err(|e: std::io::Error| {
            CestyBuildError::IoError {
                path: shadow_include_path.clone(),
                cause: e,
            }
        })?;

        if let Ok(ignore) = string_array(config, test_name, "ignore", false) {
            for ignore in ignore {
                create_empty_header(&ignore, &shadow_include_path)?;
            }
        }

        if let Ok(replace) = string_pairs(config, test_name, "replace") {
            for (original, fake) in replace {
                let fake = manifest_dir.join(fake);
                shadow_header(&fake, &original, &shadow_include_path)?;
            }
        }

        for include in &includes {
            let path = manifest_dir.join(include);
            emit_header_rerun_directives(&path)?;
            build.include(path);
        }

        build.flag("-O0");
        let llvm_plugin = build_llvm_plugin()?;
        build.flag(format!("-fpass-plugin={}", llvm_plugin));
        build.compile(test_name);

        if let Some(auto_stub_key) = config.get("auto_stub") {
            if auto_stub_key.as_bool().unwrap_or(false) {
                auto_stub(test_name, &out_dir)?;
            }
        }
    }

    Ok(())
}

fn create_empty_header(
    header_name: &str,
    shadow_include_path: &Path,
) -> Result<(), CestyBuildError> {
    let path = Path::new(&shadow_include_path).join(header_name);

    if path.parent().unwrap() != shadow_include_path {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
    }

    fs::File::create(path.clone())
        .map_err(|e: std::io::Error| CestyBuildError::IoError { path, cause: e })?;

    Ok(())
}

fn shadow_header(
    fake: &Path,
    header_name_original: &str,
    shadow_include_path: &Path,
) -> Result<(), CestyBuildError> {
    let shadow_include = Path::new(&shadow_include_path).join(header_name_original);

    if let Some(shadow_parent) = shadow_include.parent() {
        if shadow_parent != shadow_include_path {
            fs::create_dir_all(shadow_include.parent().ok_or(CestyBuildError::IoError {
                path: shadow_include.to_path_buf(),
                cause: std::io::ErrorKind::NotFound.into(),
            })?)
            .map_err(|cause| CestyBuildError::IoError {
                path: shadow_parent.to_path_buf(),
                cause,
            })?;
        }
    }

    fs::copy(fake, shadow_include).map_err(|cause| CestyBuildError::IoError {
        path: fake.to_path_buf(),
        cause,
    })?;

    Ok(())
}

fn auto_stub(test_name: &str, out_dir: &OsString) -> Result<(), CestyBuildError> {
    let mut contents = String::new();
    contents.push_str("void panic(); \n");
    let archive_path = Path::new(out_dir).join(format!("lib{test_name}.a"));

    let stub_file = Path::new(out_dir).join(format!("{test_name}_stub.c"));
    let data = fs::read(archive_path.clone()).map_err(|e| CestyBuildError::IoError {
        path: archive_path,
        cause: e,
    })?;
    let archive = ArchiveFile::parse(&*data).map_err(|_| CestyBuildError::AutoStubBuildFail)?;

    for member in archive.members() {
        let member = member.map_err(|_| CestyBuildError::AutoStubBuildFail)?;

        let name = String::from_utf8_lossy(member.name());
        eprintln!("member: {name}");

        let bytes = member
            .data(&*data)
            .map_err(|_| CestyBuildError::AutoStubBuildFail)?;

        let obj = object::File::parse(bytes).map_err(|_| CestyBuildError::AutoStubBuildFail)?;

        for sym in obj.symbols() {
            if sym.is_undefined() {
                if let Ok(name) = sym.name() {
                    if !name.is_empty() {
                        contents.push_str(&format!(
                            r#"
                            void __attribute__((weak)) {}() {{
                                panic();
                            }}
                            "#,
                            name
                        ));
                    }
                }
            }
        }
    }

    fs::write(stub_file.clone(), contents).map_err(|e| CestyBuildError::IoError {
        path: stub_file.clone(),
        cause: e,
    })?;
    let mut build = cc::Build::new();
    build.file(stub_file);
    build
        .try_compile(&format!("lib{test_name}_stub.a"))
        .map_err(|_| CestyBuildError::AutoStubBuildFail)
}

fn string_pairs(
    config: &toml::map::Map<String, toml::Value>,
    test_name: &str,
    key: &'static str,
) -> Result<Vec<(String, String)>, CestyBuildError> {
    let Some(value) = config.get(key) else {
        return Ok(Vec::new());
    };

    let values = value
        .as_table()
        .ok_or_else(|| CestyBuildError::ManifestTestParseError {
            section: test_name.to_owned(),
            message: format!("`{key}` must be an array of strings"),
        })?;

    fn _cleanup(mut s: String) -> String {
        if s.starts_with('"') && s.ends_with('"') {
            s.remove(0);
            s.pop();
        }
        s
    }
    Ok(values
        .iter()
        .map(|value| (_cleanup(value.0.to_string()), _cleanup(value.1.to_string())))
        .collect())
}

fn string_array(
    config: &toml::map::Map<String, toml::Value>,
    test_name: &str,
    key: &'static str,
    required: bool,
) -> Result<VecDeque<String>, CestyBuildError> {
    let Some(value) = config.get(key) else {
        if required {
            return Err(CestyBuildError::ManifestTestParseError {
                section: test_name.to_owned(),
                message: format!("missing required `{key}` array"),
            });
        }

        return Ok(VecDeque::new());
    };

    let values = value
        .as_array()
        .ok_or_else(|| CestyBuildError::ManifestTestParseError {
            section: test_name.to_owned(),
            message: format!("`{key}` must be an array of strings"),
        })?;

    values
        .iter()
        .map(|value| {
            value.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                CestyBuildError::ManifestTestParseError {
                    section: test_name.to_owned(),
                    message: format!("`{key}` must be an array of strings"),
                }
            })
        })
        .collect()
}

fn emit_header_rerun_directives(path: &Path) -> Result<(), CestyBuildError> {
    if path.is_file() {
        if is_header(path) {
            println!("cargo:rerun-if-changed={}", path.display());
        }

        return Ok(());
    }

    if !path.exists() {
        return Err(CestyBuildError::IoError {
            path: path.to_path_buf(),
            cause: std::io::ErrorKind::NotFound.into(),
        });
    }

    for entry in fs::read_dir(path).map_err(|cause| CestyBuildError::IoError {
        path: path.to_path_buf(),
        cause,
    })? {
        let entry = entry.map_err(|cause| CestyBuildError::IoError {
            path: path.to_path_buf(),
            cause,
        })?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            emit_header_rerun_directives(&entry_path)?;
        } else if is_header(&entry_path) {
            println!("cargo:rerun-if-changed={}", entry_path.display());
        }
    }

    Ok(())
}

fn is_header(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("h" | "hh" | "hpp" | "hxx")
    )
}

#[derive(Error, Debug)]
pub enum CestyBuildError {
    #[error("No clang binary found")]
    ClangNotFound,

    #[error("No clang++ binary found")]
    ClangxxNotFound,

    #[error("No llvm-config binary found")]
    LlvmConfigNotFound,

    #[error("LLVM plugin build failed")]
    PluginBuildFailed,

    #[error("llvm-config failed")]
    LlvmConfigFailed,

    #[error("OUT_DIR env missing. Not running as part of build")]
    MissingOutDir,

    #[error("Failed to build auto_stubs")]
    AutoStubBuildFail,

    #[error("Failed to read manifest {path} ({cause})")]
    ManifestReadError {
        path: PathBuf,
        cause: std::io::Error,
    },

    #[error("Failed to parse manifest {path} ({cause})")]
    ManifestParseError {
        path: PathBuf,
        cause: toml::de::Error,
    },

    #[error("Failed to parse manifest test section {section} ({message})")]
    ManifestTestParseError { section: String, message: String },

    #[error("Io operation failed {path} ({cause})")]
    IoError {
        path: PathBuf,
        cause: std::io::Error,
    },
}
