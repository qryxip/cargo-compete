#![warn(rust_2018_idioms)]

use anyhow::Context as _;
use cargo_compete::{shell::Shell, Context, Opt};
use std::env;
use structopt::clap;
use structopt::StructOpt as _;
use termcolor::{Color, ColorSpec, WriteColor};

fn main() {
    let Opt::Compete(opt) = Opt::from_args();
    let mut shell = Shell::new();

    let result = env::current_dir()
        .with_context(|| "could not get the current directory")
        .and_then(|cwd| {
            let ctx = Context {
                cwd,
                shell: &mut shell,
            };
            cargo_compete::run(opt, ctx)
        });

    if let Err(err) = result {
        exit_with_error(err, shell.err());
    }
}

fn exit_with_error(err: anyhow::Error, mut wtr: impl WriteColor) -> ! {
    if let Some(err) = err.downcast_ref::<clap::Error>() {
        err.exit();
    }

    let mut bold_red = ColorSpec::new();
    bold_red
        .set_reset(false)
        .set_bold(true)
        .set_fg(Some(Color::Red));

    let _ = wtr.set_color(&bold_red);
    let _ = write!(wtr, "error:");
    let _ = wtr.reset();
    let _ = writeln!(wtr, " {}", err);

    for cause in err.chain().skip(1) {
        let _ = writeln!(wtr);
        let _ = wtr.set_color(&bold_red);
        let _ = write!(wtr, "Caused by:");
        let _ = wtr.reset();
        let _ = writeln!(wtr, "\n  {}", cause);
    }

    let _ = wtr.flush();

    std::process::exit(1);
}
