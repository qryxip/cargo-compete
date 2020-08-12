use crate::{
    project::{MetadataExt as _, PackageExt as _, PackageMetadataCargoCompeteBin},
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

    /// Retrieve only the problems
    #[structopt(long, value_name("INDEX"))]
    pub problems: Option<Vec<String>>,

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
        problems,
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

    let problems = problems.map(|ps| ps.into_iter().collect::<HashSet<_>>());
    let problems = problems.as_ref();

    let manifest_path = manifest_path
        .map(Ok)
        .unwrap_or_else(|| crate::project::locate_project(cwd))?;
    let metadata = crate::project::cargo_metadata(&manifest_path)?;
    let cargo_compete_config = metadata.read_compete_toml()?;

    let member = metadata.query_for_member(package)?;
    let package_metadata_bin = member.read_package_metadata()?.bin;

    let mut urls = vec![];
    let mut file_paths = vec![];

    for (index, PackageMetadataCargoCompeteBin { name, problem, .. }) in &package_metadata_bin {
        if problems.map_or(true, |ps| ps.contains(index)) {
            urls.extend(problem.url());

            let test_suite_path = crate::testing::test_suite_path(
                &metadata.workspace_root,
                member.manifest_dir_utf8(),
                &cargo_compete_config.test_suite,
                &problem,
            )?;

            file_paths.push((&member.bin_target(&name)?.src_path, test_suite_path));
        }
    }

    crate::web::retrieve_testcases::dl_for_existing_package(
        &member,
        &mut { package_metadata_bin },
        problems,
        full,
        &metadata.workspace_root,
        &cargo_compete_config.test_suite,
        &cookies_path,
        shell,
    )?;

    Ok(())
}
