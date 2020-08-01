use crate::shell::Shell;
use anyhow::bail;
use itertools::Itertools as _;
use std::{
    ffi::{OsStr, OsString},
    fmt,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub(crate) struct ProcessBuilder {
    program: OsString,
    args: Vec<OsString>,
    cwd: PathBuf,
}

impl ProcessBuilder {
    pub(crate) fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    pub(crate) fn args(&mut self, args: &[impl AsRef<OsStr>]) -> &mut Self {
        self.args.extend(args.iter().map(|s| s.as_ref().to_owned()));
        self
    }

    pub(crate) fn exec(&self) -> anyhow::Result<()> {
        let status = std::process::Command::new(&self.program)
            .args(&self.args)
            .current_dir(&self.cwd)
            .status()?;

        if !status.success() {
            bail!("{} didn't exit successfully: {}", self, status);
        }
        Ok(())
    }

    pub(crate) fn exec_with_shell_status(&self, shell: &mut Shell) -> anyhow::Result<()> {
        shell.status("Running", self)?;
        self.exec()
    }
}

impl fmt::Display for ProcessBuilder {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "`{}{}`",
            shell_escape::escape(self.program.to_string_lossy()),
            self.args.iter().format_with("", |arg, f| f(&format_args!(
                " {}",
                shell_escape::escape(arg.to_string_lossy()),
            ))),
        )
    }
}

pub(crate) fn process(program: impl AsRef<OsStr>, cwd: impl AsRef<Path>) -> ProcessBuilder {
    ProcessBuilder {
        program: program.as_ref().to_owned(),
        args: vec![],
        cwd: cwd.as_ref().to_owned(),
    }
}
