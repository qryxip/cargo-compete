use indicatif::ProgressDrawTarget;
use snowchains_core::{color_spec, web::StatusCodeColor};
use std::{
    fmt,
    io::{self, BufRead, Write},
};
use strum::{EnumString, EnumVariantNames};
use termcolor::{BufferedStandardStream, Color, NoColor, WriteColor};

pub struct Shell {
    input: ShellIn,
    output: ShellOut,
    needs_clear: bool,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            input: ShellIn::stdin(),
            output: ShellOut::stream(),
            needs_clear: false,
        }
    }

    pub fn from_read_write(rdr: Box<dyn BufRead>, wtr: Box<dyn Write>) -> Self {
        Self {
            input: ShellIn::Reader(rdr),
            output: ShellOut::Write(NoColor::new(wtr)),
            needs_clear: false,
        }
    }

    pub(crate) fn progress_draw_target(&self) -> ProgressDrawTarget {
        if self.output.stderr_tty() {
            ProgressDrawTarget::stderr()
        } else {
            ProgressDrawTarget::hidden()
        }
    }

    pub(crate) fn out(&mut self) -> &mut dyn Write {
        self.output.stdout()
    }

    pub fn err(&mut self) -> &mut dyn WriteColor {
        self.output.stderr()
    }

    pub(crate) fn set_color_choice(&mut self, color: ColorChoice) {
        self.output.set_color_choice(color);
    }

    pub(crate) fn warn(&mut self, message: impl fmt::Display) -> io::Result<()> {
        if self.needs_clear {
            self.err_erase_line();
        }

        let stderr = self.err();

        stderr.set_color(color_spec!(Bold, Fg(Color::Yellow)))?;
        write!(stderr, "warning:")?;
        stderr.reset()?;

        writeln!(stderr, " {message}")?;

        stderr.flush()
    }

    pub(crate) fn status(
        &mut self,
        status: impl fmt::Display,
        message: impl fmt::Display,
    ) -> io::Result<()> {
        self.status_with_color(status, message, Color::Green)
    }

    pub(crate) fn status_with_color(
        &mut self,
        status: impl fmt::Display,
        message: impl fmt::Display,
        color: Color,
    ) -> io::Result<()> {
        if self.needs_clear {
            self.err_erase_line();
        }
        self.output.message_stderr(status, message, color)
    }

    fn err_erase_line(&mut self) {
        if let ShellOut::Stream {
            stderr,
            stderr_tty: true,
            ..
        } = &mut self.output
        {
            err_erase_line(stderr);
            let _ = stderr.flush();
            self.needs_clear = false;
        }

        #[cfg(unix)]
        fn err_erase_line(stderr: &mut impl Write) {
            let _ = stderr.write_all(b"\x1B[K");
        }

        #[cfg(windows)]
        fn err_erase_line(stderr: &mut impl Write) {
            if let Some((width, _)) = term_size::dimensions_stderr() {
                let _ = write!(stderr, "{}\r", " ".repeat(width));
            }
        }
    }

    pub(crate) fn read_reply(&mut self, prompt: &str) -> io::Result<String> {
        if self.needs_clear {
            self.err_erase_line();
        }

        let stderr = self.err();

        write!(stderr, "{prompt}")?;
        stderr.flush()?;
        self.input.read_reply()
    }

    pub(crate) fn read_password(&mut self, prompt: &str) -> io::Result<String> {
        if self.needs_clear {
            self.err_erase_line();
        }

        let stderr = self.err();

        write!(stderr, "{prompt}")?;
        stderr.flush()?;
        self.input.read_password()
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl snowchains_core::web::Shell for Shell {
    fn progress_draw_target(&self) -> ProgressDrawTarget {
        self.progress_draw_target()
    }

    fn print_ansi(&mut self, message: &[u8]) -> io::Result<()> {
        fwdansi::write_ansi(self.err(), message)
    }

    fn warn<T: fmt::Display>(&mut self, message: T) -> io::Result<()> {
        self.warn(message)
    }

    fn on_request(&mut self, req: &reqwest::blocking::Request) -> io::Result<()> {
        if let ShellOut::Stream {
            stderr,
            stderr_tty: true,
            ..
        } = &mut self.output
        {
            stderr.set_color(color_spec!(Bold, Fg(Color::Cyan)))?;
            write!(stderr, "{:>12}", req.method())?;
            stderr.reset()?;
            write!(stderr, " {} ...\r", req.url())?;
            stderr.flush()?;

            self.needs_clear = true;
        }
        Ok(())
    }

    fn on_response(
        &mut self,
        _: &reqwest::blocking::Response,
        _: StatusCodeColor,
    ) -> io::Result<()> {
        if self.needs_clear {
            self.err_erase_line();
        }
        Ok(())
    }
}

enum ShellIn {
    Tty,
    PipedStdin,
    Reader(Box<dyn BufRead>),
}

impl ShellIn {
    fn stdin() -> Self {
        if atty::is(atty::Stream::Stdin) {
            Self::Tty
        } else {
            Self::PipedStdin
        }
    }
}

impl ShellIn {
    fn read_reply(&mut self) -> io::Result<String> {
        match self {
            Self::Tty | Self::PipedStdin => rprompt::read_reply(),
            Self::Reader(r) => rpassword::read_password_with_reader(Some(r)),
        }
    }

    fn read_password(&mut self) -> io::Result<String> {
        match self {
            Self::Tty => rpassword::read_password_from_tty(None),
            Self::PipedStdin => rprompt::read_reply(),
            Self::Reader(r) => rpassword::read_password_with_reader(Some(r)),
        }
    }
}

enum ShellOut {
    Write(NoColor<Box<dyn Write>>),
    Stream {
        stdout: BufferedStandardStream,
        stderr: BufferedStandardStream,
        stderr_tty: bool,
    },
}

impl ShellOut {
    fn stream() -> Self {
        Self::Stream {
            stdout: BufferedStandardStream::stdout(if atty::is(atty::Stream::Stdout) {
                termcolor::ColorChoice::Auto
            } else {
                termcolor::ColorChoice::Never
            }),
            stderr: BufferedStandardStream::stderr(if atty::is(atty::Stream::Stderr) {
                termcolor::ColorChoice::Auto
            } else {
                termcolor::ColorChoice::Never
            }),
            stderr_tty: atty::is(atty::Stream::Stderr),
        }
    }

    fn stdout(&mut self) -> &mut dyn Write {
        match self {
            Self::Write(wtr) => wtr,
            Self::Stream { stdout, .. } => stdout,
        }
    }

    fn stderr(&mut self) -> &mut dyn WriteColor {
        match self {
            Self::Write(wtr) => wtr,
            Self::Stream { stderr, .. } => stderr,
        }
    }

    fn stderr_tty(&self) -> bool {
        match *self {
            Self::Write(_) => false,
            Self::Stream { stderr_tty, .. } => stderr_tty,
        }
    }

    fn set_color_choice(&mut self, color: ColorChoice) {
        if let Self::Stream { stdout, stderr, .. } = self {
            let _ = stdout.flush();
            let _ = stderr.flush();

            *stdout = BufferedStandardStream::stdout(
                color.to_termcolor_color_choice(atty::Stream::Stdout),
            );

            *stderr = BufferedStandardStream::stderr(
                color.to_termcolor_color_choice(atty::Stream::Stderr),
            );
        }
    }

    fn message_stderr(
        &mut self,
        status: impl fmt::Display,
        message: impl fmt::Display,
        color: Color,
    ) -> io::Result<()> {
        let stderr = self.stderr();

        stderr.set_color(color_spec!(Bold, Fg(color)))?;
        write!(stderr, "{status:>12}")?;
        stderr.reset()?;

        writeln!(stderr, " {message}")?;
        stderr.flush()
    }
}

#[derive(EnumString, EnumVariantNames, strum::Display, Clone, Copy, Debug)]
#[strum(serialize_all = "kebab-case")]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl ColorChoice {
    fn to_termcolor_color_choice(self, stream: atty::Stream) -> termcolor::ColorChoice {
        match (self, atty::is(stream)) {
            (Self::Auto, true) => termcolor::ColorChoice::Auto,
            (Self::Always, _) => termcolor::ColorChoice::Always,
            _ => termcolor::ColorChoice::Never,
        }
    }
}
