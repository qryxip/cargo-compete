mod commands;
mod fs;
mod process;
mod project;
pub mod shell;
mod testing;
mod web;

use crate::{
    commands::{
        init::OptCompeteInit, login::OptCompeteLogin, participate::OptCompeteParticipate,
        retrieve_testcases::OptCompeteRetrieveTestcases, submit::OptCompeteSubmit,
        test::OptCompeteTest,
    },
    shell::Shell,
};
use semver::Version;
use std::path::PathBuf;
use structopt::{clap::AppSettings, StructOpt};

static ATCODER_RUST_VERSION: Version = semver(1, 42, 0);
static CODEFORCES_RUST_VERSION: Version = semver(1, 42, 0);
static YUKICODER_RUST_VERSION: Version = semver(1, 44, 1);

const fn semver(major: u64, minor: u64, patch: u64) -> Version {
    Version {
        major,
        minor,
        patch,
        pre: vec![],
        build: vec![],
    }
}

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
pub enum OptCompete {
    /// Create workspaces in a repository
    #[structopt(author, visible_alias("i"))]
    Init(OptCompeteInit),

    /// Login to a platform
    #[structopt(author, visible_alias("l"))]
    Login(OptCompeteLogin),

    /// Register to a contest
    #[structopt(author, visible_alias("p"))]
    Participate(OptCompeteParticipate),

    /// Retrieve data
    #[structopt(author, visible_alias("r"))]
    Retrieve(OptCompeteRetrieve),

    /// Alias for `retrieve testcases`
    #[structopt(author, visible_alias("d"))]
    Download(OptCompeteRetrieveTestcases),

    /// Test your code
    #[structopt(author, visible_alias("t"))]
    Test(OptCompeteTest),

    /// Submit your code
    #[structopt(author, visible_alias("s"))]
    Submit(OptCompeteSubmit),
}

#[derive(StructOpt, Debug)]
pub enum OptCompeteRetrieve {
    /// Retrieve test cases
    #[structopt(author, visible_alias("t"))]
    Testcases(OptCompeteRetrieveTestcases),
}

pub struct Context<'s> {
    pub cwd: PathBuf,
    pub shell: &'s mut Shell,
}

pub fn run(opt: OptCompete, ctx: Context<'_>) -> anyhow::Result<()> {
    match opt {
        OptCompete::Init(opt) => commands::init::run(opt, ctx),
        OptCompete::Login(opt) => commands::login::run(opt, ctx),
        OptCompete::Participate(opt) => commands::participate::run(opt, ctx),
        OptCompete::Retrieve(OptCompeteRetrieve::Testcases(opt)) | OptCompete::Download(opt) => {
            commands::retrieve_testcases::run(opt, ctx)
        }
        OptCompete::Test(opt) => commands::test::run(opt, ctx),
        OptCompete::Submit(opt) => commands::submit::run(opt, ctx),
    }
}
