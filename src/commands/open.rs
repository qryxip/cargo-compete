use crate::{
    project::{MetadataExt as _, PackageExt as _, PackageMetadataCargoCompeteBinExample},
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

    /// Open for only binary targets
    #[structopt(long, value_name("NAME_OR_ALIAS"))]
    pub bin: Option<Vec<String>>,

    /// Open for only example targets
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

pub(crate) fn run(opt: OptCompeteOpen, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteOpen {
        full,
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

    let bin = bin.map(|bin| bin.into_iter().collect::<HashSet<_>>());
    let bin = bin.as_ref();
    let example = example.map(|example| example.into_iter().collect::<HashSet<_>>());
    let example = example.as_ref();

    let manifest_path = manifest_path
        .map(|p| Ok(cwd.join(p.strip_prefix(".").unwrap_or(&p))))
        .unwrap_or_else(|| crate::project::locate_project(&cwd))?;
    let metadata = crate::project::cargo_metadata(manifest_path, cwd)?;
    let member = metadata.query_for_member(package.as_deref())?;
    let package_metadata = member.read_package_metadata(shell)?;
    let (cargo_compete_config, _) = crate::config::load_for_package(member, shell)?;

    let mut urls = vec![];
    let mut file_paths = vec![];
    let mut missing = [hashset!(), hashset!()];

    for (i, (name, PackageMetadataCargoCompeteBinExample { alias, problem })) in itertools::chain(
        package_metadata.bin.iter().filter(
            |&(name, PackageMetadataCargoCompeteBinExample { alias, .. })| {
                bin.map_or(true, |s| s.contains(name) || s.contains(alias))
            },
        ),
        package_metadata.example.iter().filter(
            |&(name, PackageMetadataCargoCompeteBinExample { alias, .. })| {
                example.map_or(true, |s| s.contains(name) || s.contains(alias))
            },
        ),
    )
    .enumerate()
    {
        urls.push(problem.clone());

        let test_suite_path = crate::testing::test_suite_path(
            &metadata.workspace_root,
            member.manifest_dir(),
            &cargo_compete_config.test_suite,
            name,
            alias,
            problem,
            shell,
        )?;

        if !test_suite_path.exists() {
            missing[i].insert(name.clone());
        }

        file_paths.push((
            &member.bin_like_target_by_name(name)?.src_path,
            test_suite_path,
        ));
    }

    let [missing_bins, missing_examples] = &missing;

    if !(missing_bins.is_empty() && missing_examples.is_empty()) {
        shell.status("Retrieving", "missing test cases")?;

        crate::web::retrieve_testcases::dl_for_existing_package(
            member,
            &package_metadata.bin,
            &package_metadata.example,
            Some(missing_bins),
            Some(missing_examples),
            full,
            true,
            &metadata.workspace_root,
            &cargo_compete_config.test_suite,
            &cookies_path,
            shell,
        )?;
    }

    crate::open::open(
        &urls,
        cargo_compete_config.open,
        &file_paths,
        member.manifest_path.parent().unwrap(),
        &metadata.workspace_root,
        shell,
    )
}
