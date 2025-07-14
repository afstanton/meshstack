use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use std::path::Path;

pub struct CommandUnderTest<'a> {
    cmd: Command,
    temp_dir_path: &'a Path,
}

impl<'a> CommandUnderTest<'a> {
    pub fn new(temp_dir_path: &'a Path) -> Self {
        let mut cmd = Command::cargo_bin("meshstack").unwrap();
        cmd.current_dir(temp_dir_path);
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

    pub fn assert(self) -> assert_cmd::assert::Assert {
        self.cmd.assert()
    }

    pub fn current_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.cmd.current_dir(path);
        self
    }
}

#[macro_export]
macro_rules! assert_meshstack_command {
    ($temp_dir:expr, $command:expr) => {
        CommandUnderTest::new($temp_dir.path())
            .args($command.split_whitespace())
    };
    ($temp_dir:expr, $command:expr, $($arg:expr),*) => {
        CommandUnderTest::new($temp_dir.path())
            .args($command.split_whitespace())
            $(.arg($arg))*
    };
}
