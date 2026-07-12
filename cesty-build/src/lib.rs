use std::{env, ffi::OsString, fs, path::Path};

use object::{Object, ObjectSymbol, read::archive::ArchiveFile};

use find_clang::find_clang;
use llvm_pass::build_llvm_plugin;

use miette::{Context, IntoDiagnostic, Result, ensure};
mod cc;
mod config_getters;
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
        eprint!("{:?}", error);
        std::process::exit(-1);
    }
}

/// Fallible variant of [`build_c_tests`].
pub fn try_build_c_tests() -> Result<()> {
    let manifest_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");

    build_c_tests_from_manifest(&manifest_path)
}

fn get_manifest(manifest_path: &Path) -> Result<toml::Value> {
    let manifest = fs::read_to_string(manifest_path)
        .into_diagnostic()
        .wrap_err(format!(
            "Failed to read Cargo.toml at '{}'. Check that the file exists and is readable.",
            manifest_path.display()
        ))?;
    let manifest = manifest
        .parse::<toml::Value>()
        .into_diagnostic()
        .wrap_err(format!(
            "Failed to parse Cargo.toml at '{}' as TOML. Check the TOML syntax in the file.",
            manifest_path.display()
        ))?;

    Ok(manifest)
}

/// Builds C test libraries from an explicit manifest path.
pub fn build_c_tests_from_manifest(manifest_path: &Path) -> Result<()> {
    let manifest = get_manifest(manifest_path)?;

    let manifest_dir = manifest_path.parent().unwrap();

    let Some(c_tests) = manifest.get("cesty").and_then(toml::Value::as_table) else {
        return Ok(());
    };

    for (test_name, config) in c_tests {
        let config = config.as_table().wrap_err(format!(
            "Test configuration '{}' must be a table (object) in Cargo.toml [cesty.{}], but found: {}. \
             Each test should be: [cesty.test_name]\n  sources = [...]\n  # ... other fields",
            test_name,
            test_name,
            config.type_str()
        ))?;

        let sources = config_getters::string_array(config, test_name, "sources", true)?;
        let includes = config_getters::string_array(config, test_name, "includes", false)?;
        let flags = config_getters::string_array(config, test_name, "flags", false)?;

        let mut build = cc::Build::new();
        let clang = find_clang()?;
        build.compiler(clang);
        build.flags(flags);

        for source in &sources {
            let path = manifest_dir.join(source);
            println!("cargo:rerun-if-changed={}", path.display());
            build.file(path);
        }
        let out_dir = env::var_os("OUT_DIR").unwrap();

        let shadow_include_path = Path::new(&out_dir).join("shadow_include").join(test_name);

        build.include(&shadow_include_path);
        let _ = fs::remove_dir_all(&shadow_include_path);
        fs::create_dir_all(&shadow_include_path)
            .into_diagnostic()
            .wrap_err(format!(
                "Failed to create shadow include directory at '{}'. \
                 Shadow includes are temporary headers used to mock or replace system headers during compilation.",
                shadow_include_path.display()
            ))?;

        if let Ok(ignore) = config_getters::string_array(config, test_name, "ignore", false) {
            for ignore in ignore {
                create_empty_header(&ignore, &shadow_include_path).context(format!(
                    "Failed while processing 'ignore' configuration for header '{}' in test '{}'",
                    ignore, test_name
                ))?;
            }
        }

        if let Ok(replace) = config_getters::string_pairs(config, test_name, "replace") {
            for (original, fake) in replace {
                let fake = manifest_dir.join(fake);
                shadow_header(&fake, &original, &shadow_include_path)
                    .context(format!(
                        "Failed to process 'replace' configuration in test '{}': replacing '{}' with '{}'",
                        test_name, original, fake.display()
                    ))?;
            }
        }

        for include in &includes {
            let path = manifest_dir.join(include);
            emit_header_rerun_directives(&path).context(format!(
                "Failed to process include path '{}' from 'includes' configuration in test '{}'",
                path.display(),
                test_name
            ))?;
            build.include(path);
        }

        build.flag("-O0");
        let llvm_plugin = build_llvm_plugin()?;
        build.flag(format!("-fpass-plugin={}", llvm_plugin));
        build
            .try_compile(test_name)
            .wrap_err(format!("Failed to build C sources for test {test_name}"))?;

        if let Some(auto_stub_key) = config.get("auto_stub")
            && auto_stub_key.as_bool().unwrap_or(false)
        {
            auto_stub(test_name, &out_dir)?;
        }
    }

    Ok(())
}

fn create_empty_header(header_name: &str, shadow_include_path: &Path) -> Result<()> {
    let path = Path::new(&shadow_include_path).join(header_name);

    let parent = path
        .parent()
        .expect("Internal error: path parent is always valid");
    if parent != shadow_include_path {
        fs::create_dir_all(parent)
            .into_diagnostic()
            .wrap_err(format!(
                "Failed to create directory structure for shadow header at {}",
                parent.display()
            ))?
    }

    fs::File::create(path.clone())
        .into_diagnostic()
        .wrap_err(format!(
            "Failed to create empty header file at {}",
            path.display()
        ))?;

    Ok(())
}

fn shadow_header(
    fake: &Path,
    header_name_original: &str,
    shadow_include_path: &Path,
) -> Result<()> {
    let shadow_include = Path::new(&shadow_include_path).join(header_name_original);
    let shadow_parent = shadow_include
        .parent()
        .expect("Internal error: path parent is always valid");

    if shadow_parent != shadow_include_path {
        fs::create_dir_all(shadow_parent)
            .into_diagnostic()
            .wrap_err(format!(
                "Failed to create directory structure for shadow header at {}",
                shadow_parent.display()
            ))?;
    }

    fs::copy(fake, shadow_include.clone())
        .into_diagnostic()
        .wrap_err(format!(
            "Failed to copy header from {} to {}",
            fake.display(),
            shadow_include.display()
        ))?;
    Ok(())
}

fn auto_stub(test_name: &str, out_dir: &OsString) -> Result<()> {
    let mut contents = String::new();
    contents.push_str("void cesty_panic(const char*); \n");
    let archive_path = Path::new(out_dir).join(format!("lib{test_name}.a"));

    let stub_file = Path::new(out_dir).join(format!("{test_name}_stub.c"));
    let data = fs::read(archive_path.clone())
        .into_diagnostic()
        .wrap_err(format!(
            "Failed to read compiled test library archive at {}. This archive should have been created by the C compiler.",
            archive_path.display()
        ))?;
    let archive = ArchiveFile::parse(&*data)
        .into_diagnostic()
        .wrap_err(format!(
            "Failed to parse the compiled test library archive at {}. The archive may be corrupted or in an unsupported format.",
            archive_path.display()
        ))?;
    for member in archive.members() {
        let member = member
            .into_diagnostic()
            .context(format!(
                "Failed to read archive member while processing test library '{}' for auto-stub generation",
                test_name
            ))?;
        let name = String::from_utf8_lossy(member.name());

        let bytes = member
            .data(&*data)
            .into_diagnostic()
            .context(format!(
                "Failed to extract data from archive member '{}' while processing test library '{}' for auto-stub generation",
                name, test_name
            ))?;

        let obj = object::File::parse(bytes)
            .into_diagnostic()
            .context(format!(
                "Failed to parse object file from archive member '{}' in test library '{}' for auto-stub generation",
                name, test_name
            ))?;
        for sym in obj.symbols() {
            if sym.is_undefined()
                && let Ok(name) = sym.name()
                && !name.is_empty()
            {
                contents.push_str(&format!(
                    r#"
                            void __attribute__((weak)) {}() {{
                                cesty_panic(__func__);
                            }}
                            "#,
                    name
                ));
            }
        }
    }

    fs::write(stub_file.clone(), contents)
        .into_diagnostic()
        .wrap_err(format!(
            "Failed to write auto-generated stub file at {}. Check that the output directory is writable.",
            stub_file.display()
        ))?;
    let mut build = cc::Build::new();
    build.file(stub_file);
    build
        .try_compile(&format!("lib{test_name}_stub.a"))
        .wrap_err(format!(
            "Failed to compile auto-generated stubs for test '{}'. \
             This file provides weak implementations for functions that the test library references but aren't available. \
             Check the compiler error above for details.",
            test_name
        ))
}

fn emit_header_rerun_directives(path: &Path) -> Result<()> {
    if path.is_file() {
        if is_header(path) {
            println!("cargo:rerun-if-changed={}", path.display());
        }

        return Ok(());
    }

    ensure!(
        path.exists(),
        format!(
            "Header directory '{}' does not exist. Check that the path specified in the 'includes' configuration is correct.",
            path.display()
        )
    );

    for entry in fs::read_dir(path).into_diagnostic().wrap_err(format!(
        "Failed to read header directory '{}' to scan for changes",
        path.display()
    ))? {
        let entry = entry.into_diagnostic().wrap_err(format!(
            "Failed to read entry while scanning header directory '{}'",
            path.display()
        ))?;
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
