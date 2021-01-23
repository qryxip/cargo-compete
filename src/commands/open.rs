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

    /// Open for only binaries
    #[structopt(long, value_name("NAME_OR_ALIAS"))]
    pub bin: Option<Vec<String>>,

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

    let manifest_path = manifest_path
        .map(|p| Ok(cwd.join(p.strip_prefix(".").unwrap_or(&p))))
        .unwrap_or_else(|| crate::project::locate_project(&cwd))?;
    let metadata = crate::project::cargo_metadata(&manifest_path, cwd)?;
    let member = metadata.query_for_member(package.as_deref())?;
    let package_metadata = member.read_package_metadata(shell)?;
    let cargo_compete_config = crate::config::load_for_package(&member, shell)?;

    let mut urls = vec![];
    let mut file_paths = vec![];
    let mut missing = hashset!();

    for (name, PackageMetadataCargoCompeteBin { alias, problem }) in &package_metadata.bin {
        if bin
            .as_ref()
            .map_or(true, |bin| bin.contains(name) || bin.contains(alias))
        {
            urls.push(problem.clone());

            let test_suite_path = crate::testing::test_suite_path(
                &metadata.workspace_root,
                member.manifest_dir_utf8(),
                &cargo_compete_config.test_suite,
                name,
                alias,
                &problem,
                shell,
            )?;

            if !test_suite_path.exists() {
                missing.insert(name.clone());
            }

            file_paths.push((&member.bin_target_by_name(name)?.src_path, test_suite_path));
        }
    }

    if !missing.is_empty() {
        shell.status("Retrieving", "missing test cases")?;

        crate::web::retrieve_testcases::dl_for_existing_package(
            &member,
            &package_metadata.bin,
            Some(&missing),
            full,
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
