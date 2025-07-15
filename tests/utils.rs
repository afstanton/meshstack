use assert_cmd::prelude::*;
use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;

pub struct CommandUnderTest<'a> {
    cmd: Command,
    temp_dir_path: &'a Path,
}

impl<'a> CommandUnderTest<'a> {
    pub fn new(temp_dir_path: &'a Path) -> Self {
        let mut cmd = Command::cargo_bin("meshstack").unwrap();
        cmd.current_dir(temp_dir_path);

        // Copy templates directory to the temporary test directory
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let templates_src = project_root.join("templates");
        let templates_dest = temp_dir_path.join("templates");

        if templates_src.exists() {
            copy_dir_all(&templates_src, &templates_dest).expect("Failed to copy templates directory");
        } else {
            panic!("Templates source directory does not exist: {:?}", templates_src);
        }

        CommandUnderTest { cmd, temp_dir_path }
    }

    pub fn arg(mut self, arg: impl AsRef<std::ffi::OsStr>) -> Self {
        self.cmd.arg(arg);
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>) -> Self {
        self.cmd.args(args);
        self
    }

    pub fn env(mut self, key: impl AsRef<std::ffi::OsStr>, val: impl AsRef<std::ffi::OsStr>) -> Self {
        self.cmd.env(key, val);
        self
    }

    pub fn assert(mut self) -> assert_cmd::assert::Assert {
        self.cmd.assert()
    }

    pub fn current_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.cmd.current_dir(path);
        self
    }
}

// Helper function to copy a directory recursively
fn copy_dir_all(src: &Path, dst: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}
