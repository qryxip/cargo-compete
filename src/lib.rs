pub mod shell;

use crate::shell::{ColorChoice, Shell};
use anyhow::bail;
use std::{io::BufRead, path::PathBuf};
use structopt::{clap::AppSettings, StructOpt};
use strum::VariantNames as _;
use termcolor::WriteColor;

#[derive(StructOpt, Debug)]
#[structopt(
    about,
    author,
    bin_name("cargo"),
    global_settings(&[AppSettings::DeriveDisplayOrder, AppSettings::UnifiedHelpMessage])
)]
pub enum Opt {
    #[structopt(about, author)]
    Compete(OptCompete),
}

#[derive(StructOpt, Debug)]
pub struct OptCompete {
    #[structopt(
        long,
        value_name("WHEN"),
        possible_values(ColorChoice::VARIANTS),
        default_value("auto")
    )]
    pub color: ColorChoice,
}

impl OptCompete {
    pub fn color(&self) -> ColorChoice {
        match *self {
            Self { color } => color,
        }
    }
}

pub struct Context<R, W1, W2> {
    pub cwd: PathBuf,
    pub shell: Shell<R, W1, W2>,
}

pub fn run<R: BufRead, W1: WriteColor, W2: WriteColor>(
    opt: OptCompete,
    ctx: Context<R, W1, W2>,
) -> anyhow::Result<()> {
    bail!("TODO");
}
