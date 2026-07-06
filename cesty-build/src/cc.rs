use miette::{Context, IntoDiagnostic, MietteDiagnostic, Result};
use std::env;
use std::path::PathBuf;
use std::process::Command;

#[derive(Default)]
pub struct Build {
    compiler: Option<PathBuf>,
    files: Vec<PathBuf>,
    includes: Vec<PathBuf>,
    flags: Vec<String>,
}

impl Build {
    pub fn new() -> Self {
        Self {
            compiler: None,
            files: Vec::new(),
            includes: Vec::new(),
            flags: Vec::new(),
        }
    }

    pub fn compiler<P: Into<PathBuf>>(&mut self, p: P) -> &mut Self {
        self.compiler = Some(p.into());
        self
    }

    pub fn flags<I, S>(&mut self, flags: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for f in flags {
            self.flags.push(f.into());
        }
        self
    }

    pub fn flag<S: Into<String>>(&mut self, f: S) -> &mut Self {
        self.flags.push(f.into());
        self
    }

    pub fn file<P: Into<PathBuf>>(&mut self, f: P) -> &mut Self {
        self.files.push(f.into());
        self
    }

    pub fn include<P: Into<PathBuf>>(&mut self, p: P) -> &mut Self {
        self.includes.push(p.into());
        self
    }

    /// Pop the top source, compile it and return the path to the resulting object file
    fn compile_source(&self, src: &PathBuf) -> Result<PathBuf> {
        let compiler = match &self.compiler {
            Some(p) => p.clone(),
            None => PathBuf::from("clang"),
        };

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or_else(|_| ".".into()));

        // ensure object dir exists
        let obj_dir = out_dir.join("cesty-obj");
        let _ = std::fs::remove_dir_all(&obj_dir);
        std::fs::create_dir_all(&obj_dir)
            .into_diagnostic()
            .wrap_err(format!("failed to create object dir {}", obj_dir.display()))?;

        let file_stem = src
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed");
        let obj_path = obj_dir.join(format!("{}.o", file_stem));

        // build clang args
        let mut cmd = Command::new(&compiler);
        cmd.arg("-c");
        cmd.arg(src);
        cmd.arg("-o");
        cmd.arg(&obj_path);

        for inc in &self.includes {
            cmd.arg("-I");
            cmd.arg(inc);
        }

        for f in &self.flags {
            cmd.arg(f);
        }

        let output = cmd.output().into_diagnostic().wrap_err(format!(
            "Failed to launch compile \"{}\"",
            compiler.display()
        ))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(MietteDiagnostic::new(format!(
                "Failed to compile source file {}",
                src.display()
            ))
            .with_help(format!(
                "Command:\n  {:?}\n\nStdout:\n{}\nStderr:\n{}",
                cmd, stdout, stderr
            ))
            .into());
        }

        Ok(obj_path)
    }

    fn compile_objects(&self) -> Result<Vec<PathBuf>> {
        self.files.iter().map(|s| self.compile_source(s)).collect()
    }

    pub fn try_compile(&self, name: &str) -> Result<()> {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or_else(|_| ".".into()));
        let archive_name = if name.starts_with("lib") && name.ends_with(".a") {
            name.to_string()
        } else {
            format!("lib{}.a", name)
        };
        let name = if name.starts_with("lib") && name.ends_with(".a") {
            name.trim_start_matches("lib")
                .trim_end_matches(".a")
                .to_string()
        } else {
            name.to_string()
        };

        let archive_path = out_dir.join(&archive_name);

        let object_files = self.compile_objects()?;

        // create archive with ar
        let mut ar_cmd = Command::new("ar");
        ar_cmd.arg("rcs");
        ar_cmd.arg(&archive_path);
        for obj in &object_files {
            ar_cmd.arg(obj);
        }

        let output = ar_cmd
            .output()
            .into_diagnostic()
            .wrap_err("Failed to launch ar")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(MietteDiagnostic::new(format!(
                "Failed to archive object files {}",
                archive_name
            ))
            .with_help(format!("\nStdout:\n{}\nStderr:\n{}", stdout, stderr))
            .into());
        }

        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib=static={}", name);

        Ok(())
    }
}
