use crate::shell::Shell;
use anyhow::{bail, Context as _};
use itertools::Itertools as _;
use std::{
    env,
    ffi::{OsStr, OsString},
    fmt,
    io::Write as _,
    path::{Path, PathBuf},
    process::Stdio,
};

#[derive(Debug)]
pub(crate) struct ProcessBuilder<C: Presence<PathBuf>> {
    program: OsString,
    args: Vec<OsString>,
    cwd: C::Value,
    display_cwd: bool,
    pipe_input: Option<Vec<u8>>,
}

impl<C: Presence<PathBuf>> ProcessBuilder<C> {
    pub(crate) fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    pub(crate) fn args(mut self, args: &[impl AsRef<OsStr>]) -> Self {
        self.args.extend(args.iter().map(|s| s.as_ref().to_owned()));
        self
    }

    pub(crate) fn cwd(self, cwd: impl AsRef<Path>) -> ProcessBuilder<Present> {
        ProcessBuilder {
            program: self.program,
            args: self.args,
            cwd: cwd.as_ref().to_owned(),
            display_cwd: self.display_cwd,
            pipe_input: self.pipe_input,
        }
    }

    pub(crate) fn display_cwd(self) -> Self {
        Self {
            display_cwd: true,
            ..self
        }
    }

    pub(crate) fn pipe_input(mut self, pipe_input: Option<impl Into<Vec<u8>>>) -> Self {
        self.pipe_input = pipe_input.map(Into::into);
        self
    }
}

impl ProcessBuilder<Present> {
    pub(crate) fn exec(&self) -> anyhow::Result<()> {
        let status = self.spawn(Stdio::inherit())?.wait()?;
        if !status.success() {
            bail!("{} didn't exit successfully: {}", self, status);
        }
        Ok(())
    }

    pub(crate) fn exec_with_shell_status(&self, shell: &mut Shell) -> anyhow::Result<()> {
        shell.status("Running", self)?;
        self.exec()
    }

    fn read(&self) -> anyhow::Result<String> {
        let std::process::Output { status, stdout, .. } =
            self.spawn(Stdio::piped())?.wait_with_output()?;
        if !status.success() {
            bail!("{} didn't exit successfully: {}", self, status);
        }
        String::from_utf8(stdout).with_context(|| "non UTF-8 output")
    }

    pub(crate) fn read_with_shell_status(&self, shell: &mut Shell) -> anyhow::Result<String> {
        shell.status("Running", self)?;
        self.read()
    }

    fn spawn(&self, stdout: Stdio) -> anyhow::Result<std::process::Child> {
        let mut child = std::process::Command::new(&self.program)
            .args(&self.args)
            .current_dir(&self.cwd)
            .stdin(if self.pipe_input.is_some() {
                Stdio::piped()
            } else {
                Stdio::inherit()
            })
            .stdout(stdout)
            .spawn()?;

        if let (Some(mut stdin), Some(pipe_input)) = (child.stdin.take(), self.pipe_input.as_ref())
        {
            stdin.write_all(pipe_input)?;
            stdin.flush()?;
        }

        Ok(child)
    }
}

impl fmt::Display for ProcessBuilder<Present> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "`{}{}`{}",
            shell_escape::escape(self.program.to_string_lossy()),
            self.args.iter().format_with("", |arg, f| f(&format_args!(
                " {}",
                shell_escape::escape(arg.to_string_lossy()),
            ))),
            if self.display_cwd {
                format!(" in {}", self.cwd.display())
            } else {
                "".to_owned()
            }
        )
    }
}

pub(crate) trait Presence<T> {
    type Value;
}

#[derive(Debug)]
pub(crate) enum NotPresent {}

impl<T> Presence<T> for NotPresent {
    type Value = ();
}

#[derive(Debug)]
pub(crate) enum Present {}

impl<T> Presence<T> for Present {
    type Value = T;
}

pub(crate) fn process(program: impl AsRef<Path>) -> ProcessBuilder<NotPresent> {
    ProcessBuilder {
        program: program.as_ref().into(),
        args: vec![],
        cwd: (),
        display_cwd: false,
        pipe_input: None,
    }
}

pub(crate) fn with_which(
    program: impl AsRef<Path>,
    cwd: impl AsRef<Path>,
) -> anyhow::Result<ProcessBuilder<Present>> {
    let (program, cwd) = (program.as_ref(), cwd.as_ref().to_owned());
    let program = which(program, &cwd)?.into();

    Ok(ProcessBuilder {
        program,
        args: vec![],
        cwd,
        display_cwd: false,
        pipe_input: None,
    })
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
