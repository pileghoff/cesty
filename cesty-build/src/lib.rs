use std::{
    env,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

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
pub fn try_build_c_tests() -> Result<(), BuildError> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").map_err(|_| BuildError::MissingEnv {
        name: "CARGO_MANIFEST_DIR",
    })?;
    let manifest_path = Path::new(&manifest_dir).join("Cargo.toml");

    build_c_tests_from_manifest(&manifest_path)
}

/// Builds C test libraries from an explicit manifest path.
pub fn build_c_tests_from_manifest(manifest_path: impl AsRef<Path>) -> Result<(), BuildError> {
    let manifest_path = manifest_path.as_ref();
    let manifest_dir = manifest_path
        .parent()
        .ok_or_else(|| BuildError::InvalidManifestPath {
            path: manifest_path.to_path_buf(),
        })?;

    let manifest =
        fs::read_to_string(manifest_path).map_err(|source| BuildError::ReadManifest {
            path: manifest_path.to_path_buf(),
            source,
        })?;
    let manifest = manifest
        .parse::<toml::Value>()
        .map_err(|source| BuildError::ParseManifest {
            path: manifest_path.to_path_buf(),
            source,
        })?;

    let Some(c_tests) = manifest.get("cesty").and_then(toml::Value::as_table) else {
        println!("cargo:rerun-if-changed={}", manifest_path.display());
        return Ok(());
    };

    println!("cargo:rerun-if-changed={}", manifest_path.display());

    for (test_name, config) in c_tests {
        let config = config
            .as_table()
            .ok_or_else(|| BuildError::InvalidTestConfig {
                test_name: test_name.clone(),
                message: "expected a table".to_owned(),
            })?;

        let sources = string_array(config, test_name, "sources", true)?;
        let includes = string_array(config, test_name, "includes", false)?;

        let mut build = cc::Build::new();

        for source in &sources {
            let path = manifest_dir.join(source);
            println!("cargo:rerun-if-changed={}", path.display());
            build.file(path);
        }

        for include in &includes {
            let path = manifest_dir.join(include);
            emit_header_rerun_directives(&path)?;
            build.include(path);
        }

        build.compile(test_name);
    }

    Ok(())
}

fn string_array(
    config: &toml::map::Map<String, toml::Value>,
    test_name: &str,
    key: &'static str,
    required: bool,
) -> Result<Vec<String>, BuildError> {
    let Some(value) = config.get(key) else {
        if required {
            return Err(BuildError::InvalidTestConfig {
                test_name: test_name.to_owned(),
                message: format!("missing required `{key}` array"),
            });
        }

        return Ok(Vec::new());
    };

    let values = value
        .as_array()
        .ok_or_else(|| BuildError::InvalidTestConfig {
            test_name: test_name.to_owned(),
            message: format!("`{key}` must be an array of strings"),
        })?;

    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| BuildError::InvalidTestConfig {
                    test_name: test_name.to_owned(),
                    message: format!("`{key}` must be an array of strings"),
                })
        })
        .collect()
}

fn emit_header_rerun_directives(path: &Path) -> Result<(), BuildError> {
    if path.is_file() {
        if is_header(path) {
            println!("cargo:rerun-if-changed={}", path.display());
        }

        return Ok(());
    }

    if !path.exists() {
        return Err(BuildError::MissingIncludePath {
            path: path.to_path_buf(),
        });
    }

    for entry in fs::read_dir(path).map_err(|source| BuildError::ReadIncludeDir {
        path: path.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| BuildError::ReadIncludeDir {
            path: path.to_path_buf(),
            source,
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

#[derive(Debug)]
pub enum BuildError {
    MissingEnv {
        name: &'static str,
    },
    InvalidManifestPath {
        path: PathBuf,
    },
    ReadManifest {
        path: PathBuf,
        source: std::io::Error,
    },
    ParseManifest {
        path: PathBuf,
        source: toml::de::Error,
    },
    InvalidTestConfig {
        test_name: String,
        message: String,
    },
    MissingIncludePath {
        path: PathBuf,
    },
    ReadIncludeDir {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::MissingEnv { name } => write!(formatter, "`{name}` is not set"),
            BuildError::InvalidManifestPath { path } => {
                write!(formatter, "`{}` has no parent directory", path.display())
            }
            BuildError::ReadManifest { path, source } => {
                write!(formatter, "failed to read `{}`: {source}", path.display())
            }
            BuildError::ParseManifest { path, source } => {
                write!(formatter, "failed to parse `{}`: {source}", path.display())
            }
            BuildError::InvalidTestConfig { test_name, message } => {
                write!(
                    formatter,
                    "invalid c_tests config for `{test_name}`: {message}"
                )
            }
            BuildError::MissingIncludePath { path } => {
                write!(
                    formatter,
                    "include path `{}` does not exist",
                    path.display()
                )
            }
            BuildError::ReadIncludeDir { path, source } => {
                write!(
                    formatter,
                    "failed to read include directory `{}`: {source}",
                    path.display()
                )
            }
        }
    }
}

impl Error for BuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BuildError::ReadManifest { source, .. } => Some(source),
            BuildError::ParseManifest { source, .. } => Some(source),
            BuildError::ReadIncludeDir { source, .. } => Some(source),
            _ => None,
        }
    }
}
