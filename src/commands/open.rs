use crate::{
    project::{MetadataExt as _, PackageExt as _, PackageMetadataCargoCompeteBin},
    shell::ColorChoice,
};
use maplit::hashset;
use std::{collections::HashSet, path::PathBuf};
use structopt::StructOpt;
use strum::VariantNames as _;

#[derive(StructOpt, Debug)]
pub struct OptCompeteOpen {
    /// Retrieves system test cases
    #[structopt(long)]
    pub full: bool,

    /// Problem indexes
    #[structopt(long)]
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

pub(crate) fn run(opt: OptCompeteOpen, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteOpen {
        full,
        problems,
        package,
        manifest_path,
        color,
    } = opt;

    let crate::Context { cwd, shell } = ctx;

    shell.set_color_choice(color);

    let problems = problems.map(|ps| ps.into_iter().collect::<HashSet<_>>());

    let manifest_path = manifest_path
        .map(Ok)
        .unwrap_or_else(|| crate::project::locate_project(&cwd))?;
    let metadata = crate::project::cargo_metadata(&manifest_path)?;
    let workspace_metadata = metadata.read_workspace_metadata()?;

    let member = metadata.query_for_member(package)?;

    let package_metadata_bin = member.read_package_metadata()?.bin;

    let mut urls = vec![];
    let mut file_paths = vec![];
    let mut missing = hashset!();

    for (index, PackageMetadataCargoCompeteBin { name, problem, .. }) in &package_metadata_bin {
        if problems.as_ref().map_or(true, |ps| ps.contains(index)) {
            urls.extend(problem.url());

            let test_suite_path = crate::testing::test_suite_path(
                &metadata.workspace_root,
                &workspace_metadata.test_suite,
                &problem,
            )?;

            if !test_suite_path.exists() {
                missing.insert(index.clone());
            }

            file_paths.push((&member.bin_target(&name)?.src_path, test_suite_path));
        }
    }

    if !missing.is_empty() {
        shell.status("Retrieving", "missing test cases")?;

        crate::web::retrieve_testcases::dl_for_existing_package(
            &package_metadata_bin,
            Some(&missing),
            full,
            &metadata.workspace_root,
            &workspace_metadata.test_suite,
            shell,
        )?;
    }

    crate::open::open(
        &urls,
        workspace_metadata.open,
        &file_paths,
        member.manifest_path.parent().unwrap(),
        &cwd,
        shell,
    )
}
