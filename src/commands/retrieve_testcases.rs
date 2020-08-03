use crate::{project::MetadataExt as _, shell::ColorChoice};
use std::path::PathBuf;
use structopt::StructOpt;
use strum::VariantNames as _;

#[derive(StructOpt, Debug)]
pub struct OptCompeteRetrieveTestcases {
    /// Retrieves system test cases
    #[structopt(long)]
    pub full: bool,

    /// Open URL and files
    #[structopt(long)]
    pub open: bool,

    /// Problem Indexes
    #[structopt(long, value_name("STRING"))]
    pub problems: Option<Vec<String>>,

    /// Existing package to retrieving test cases for
    #[structopt(short, long, value_name("SPEC"))]
    pub package: Option<String>,

    /// Path to Cargo.toml
    #[structopt(long, value_name("PATH"))]
    pub manifest_path: Option<PathBuf>,

    /// Coloring
    #[structopt(
        long,
        value_name("WHEN"),
        possible_values(ColorChoice::VARIANTS),
        default_value("auto")
    )]
    pub color: ColorChoice,

    /// Creates pacakge afresh & retrieve test cases for the contest ID
    pub contest: Option<String>,
}

pub(crate) fn run(opt: OptCompeteRetrieveTestcases, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteRetrieveTestcases {
        full,
        open,
        package,
        manifest_path,
        color,
        problems,
        contest,
    } = opt;

    let crate::Context { cwd, shell } = ctx;

    shell.set_color_choice(color);

    let manifest_path = manifest_path
        .map(Ok)
        .unwrap_or_else(|| crate::project::locate_project(&cwd))?;
    let metadata = crate::project::cargo_metadata(&manifest_path)?;
    let workspace_metadata = metadata.read_workspace_metadata()?;

    let member = metadata.query_for_member(package)?;

    todo!();
}
