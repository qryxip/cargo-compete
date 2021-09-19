use crate::{
    project::{MetadataExt as _, PackageExt as _},
    shell::ColorChoice,
};
use std::{collections::HashSet, path::PathBuf};
use structopt::StructOpt;
use strum::VariantNames as _;

#[derive(StructOpt, Debug)]
pub struct OptCompeteRetrieveTestcases {
    /// Retrieves system test cases
    #[structopt(long)]
    pub full: bool,

    /// Overwrites the existing test files
    #[structopt(long)]
    pub overwrite: bool,

    /// Retrieve only the problems for the binary target
    #[structopt(long, value_name("NAME_OR_ALIAS"))]
    pub bin: Option<Vec<String>>,

    /// Retrieve only the problems for the example target
    #[structopt(long, value_name("NAME_OR_ALIAS"))]
    pub example: Option<Vec<String>>,

    /// Package (see `cargo help pkgid`)
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
}

pub(crate) fn run(opt: OptCompeteRetrieveTestcases, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteRetrieveTestcases {
        full,
        overwrite,
        bin,
        example,
        package,
        manifest_path,
        color,
    } = opt;

    let crate::Context {
        cwd,
        cookies_path,
        shell,
    } = ctx;

    shell.set_color_choice(color);

    let bin = bin.map(|s| s.into_iter().collect::<HashSet<_>>());
    let bin = bin.as_ref();
    let example = example.map(|s| s.into_iter().collect::<HashSet<_>>());
    let example = example.as_ref();

    let manifest_path = manifest_path
        .map(|p| Ok(cwd.join(p.strip_prefix(".").unwrap_or(&p))))
        .unwrap_or_else(|| crate::project::locate_project(&cwd))?;
    let metadata = crate::project::cargo_metadata(&manifest_path, cwd)?;
    let member = metadata.query_for_member(package.as_deref())?;
    let package_metadata = member.read_package_metadata(shell)?;
    let (cargo_compete_config, _) = crate::config::load_for_package(member, shell)?;

    crate::web::retrieve_testcases::dl_for_existing_package(
        member,
        &package_metadata.bin,
        &package_metadata.example,
        bin,
        example,
        full,
        overwrite,
        &metadata.workspace_root,
        &cargo_compete_config.test_suite,
        &cookies_path,
        shell,
    )
}
