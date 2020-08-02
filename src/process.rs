use crate::shell::Shell;
use anyhow::{bail, Context as _};
use itertools::Itertools as _;
use std::{
    env,
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
            "`{}{}` in {}",
            shell_escape::escape(self.program.to_string_lossy()),
            self.args.iter().format_with("", |arg, f| f(&format_args!(
                " {}",
                shell_escape::escape(arg.to_string_lossy()),
            ))),
            self.cwd.display(),
        )
    }
}

pub(crate) fn process(program: impl AsRef<Path>, cwd: impl AsRef<Path>) -> ProcessBuilder {
    ProcessBuilder {
        program: program.as_ref().into(),
        args: vec![],
        cwd: cwd.as_ref().into(),
    }
}

pub(crate) fn with_which(
    program: impl AsRef<Path>,
    cwd: impl AsRef<Path>,
) -> anyhow::Result<ProcessBuilder> {
    let (program, cwd) = (program.as_ref(), cwd.as_ref());
    let program = which(program, cwd)?;
    Ok(process(program, cwd))
}

pub(crate) fn which(
    binary_name: impl AsRef<OsStr>,
    cwd: impl AsRef<Path>,
) -> anyhow::Result<PathBuf> {
    let binary_name = binary_name.as_ref();
    which::which_in(binary_name, env::var_os("PATH"), cwd)
        .with_context(|| format!("`{}` not found", binary_name.to_string_lossy()))
}

pub(crate) fn cargo_exe() -> anyhow::Result<PathBuf> {
    env::var_os("CARGO")
        .with_context(|| "`$CARGO` should be present")
        .map(Into::into)
}
