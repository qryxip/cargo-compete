use indicatif::ProgressDrawTarget;
use snowchains_core::{color_spec, web::StatusCodeColor};
use std::{
    fmt,
    io::{self, BufRead, Stdin, StdinLock, Write},
};
use strum::{EnumString, EnumVariantNames};
use termcolor::{Color, WriteColor};

pub struct Shell<R, W1, W2> {
    stdin: Input<R>,
    stdout: W1,
    stderr: W2,
    stderr_tty: bool,
    needs_clear: bool,
}

impl<R: BufRead, W1: WriteColor, W2: WriteColor> Shell<R, W1, W2> {
    pub fn new(stdin: Input<R>, stdout: W1, stderr: W2, stderr_tty: bool) -> Self {
        Self {
            stdin,
            stdout,
            stderr,
            stderr_tty,
            needs_clear: false,
        }
    }
}

impl<R, W1, W2: WriteColor> Shell<R, W1, W2> {
    pub(crate) fn warn(&mut self, message: impl fmt::Display) -> io::Result<()> {
        if self.needs_clear {
            self.err_erase_line();
        }

        self.stderr
            .set_color(color_spec!(Bold, Fg(Color::Yellow)))?;
        write!(self.stderr, "warning:")?;
        self.stderr.reset()?;

        writeln!(self.stderr, " {}", message)?;

        self.stderr.flush()
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

        self.stderr.set_color(color_spec!(Bold, Fg(color)))?;
        write!(self.stderr, "{:>12}", status)?;
        self.stderr.reset()?;

        writeln!(self.stderr, " {}", message)
    }

    fn err_erase_line(&mut self) {
        if self.stderr_tty && self.stderr.supports_color() {
            err_erase_line(&mut self.stderr);
            self.needs_clear = false;
        }

        #[cfg(unix)]
        fn err_erase_line(mut stderr: impl Write) {
            let _ = stderr.write_all(b"\x1B[K");
        }

        #[cfg(windows)]
        fn err_erase_line(mut stderr: impl Write) {
            if let Some((width, _)) = term_size::dimensions_stderr() {
                let _ = write!(stderr, "{}\r", " ".repeat(width));
            }
        }
    }
}

impl<R, W1, W2: WriteColor> snowchains_core::web::Shell for Shell<R, W1, W2> {
    fn progress_draw_target(&self) -> ProgressDrawTarget {
        if self.stderr_tty {
            ProgressDrawTarget::stderr()
        } else {
            ProgressDrawTarget::hidden()
        }
    }

    fn print_ansi(&mut self, message: &[u8]) -> io::Result<()> {
        fwdansi::write_ansi(&mut self.stderr, message)
    }

    fn warn<T: fmt::Display>(&mut self, message: T) -> io::Result<()> {
        self.warn(message)
    }

    fn on_request(&mut self, req: &reqwest::blocking::Request) -> io::Result<()> {
        self.status_with_color(req.method(), format!("{} ...", req.url()), Color::Cyan)?;
        self.needs_clear = true;
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

pub enum Input<R> {
    Tty,
    Piped(R),
}

impl<'a> Input<StdinLock<'a>> {
    pub fn from_stdin(stdin: &'a Stdin) -> Self {
        if atty::is(atty::Stream::Stdin) {
            Self::Tty
        } else {
            Self::Piped(stdin.lock())
        }
    }
}

#[derive(EnumString, EnumVariantNames, Clone, Copy, Debug)]
#[strum(serialize_all = "kebab-case")]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl ColorChoice {
    pub fn to_termcolor_color_choice(self, stream: atty::Stream) -> termcolor::ColorChoice {
        match (self, atty::is(stream)) {
            (Self::Auto, true) => termcolor::ColorChoice::Auto,
            (Self::Always, _) => termcolor::ColorChoice::Always,
            _ => termcolor::ColorChoice::Never,
        }
    }
}
