use crate::{
    config::CargoCompeteConfigTestProfile,
    project::{MetadataExt as _, PackageExt as _},
    shell::ColorChoice,
};
use human_size::Size;
use std::path::PathBuf;
use structopt::StructOpt;
use strum::VariantNames as _;

#[derive(StructOpt, Debug)]
#[structopt(usage(
    r"cargo compete test [OPTIONS] <index>
    cargo compete test [OPTIONS] --src <PATH>",
))]
pub struct OptCompeteTest {
    /// Path to the source code
    #[structopt(
        long,
        value_name("PATH"),
        required_unless("index"),
        conflicts_with("index")
    )]
    pub src: Option<PathBuf>,

    /// Test for only the test cases
    #[structopt(long, value_name("NAME"))]
    pub testcases: Option<Vec<String>>,

    /// Display limit
    #[structopt(long, value_name("SIZE"), default_value("4KiB"))]
    pub display_limit: Size,

    /// Existing package to retrieving test cases for
    #[structopt(short, long, value_name("SPEC"))]
    pub package: Option<String>,

    /// Build in debug mode. Overrides `test.profile` in compete.toml
    #[structopt(long, conflicts_with("release"))]
    pub debug: bool,

    /// Build in release mode. Overrides `test.profile` in compete.toml
    #[structopt(long)]
    pub release: bool,

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

    #[structopt(required_unless("src"))]
    /// Index for `package.metadata.cargo-compete.bin`
    pub index: Option<String>,
}

pub(crate) fn run(opt: OptCompeteTest, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteTest {
        src,
        testcases,
        display_limit,
        package,
        debug,
        release,
        manifest_path,
        color,
        index,
    } = opt;

    let crate::Context {
        cwd,
        cookies_path: _,
        shell,
    } = ctx;

    shell.set_color_choice(color);

    let manifest_path = manifest_path
        .map(|p| Ok(cwd.join(p.strip_prefix(".").unwrap_or(&p))))
        .unwrap_or_else(|| crate::project::locate_project(&cwd))?;
    let metadata = crate::project::cargo_metadata(&manifest_path, &cwd)?;
    let member = metadata.query_for_member(package.as_deref())?;
    let package_metadata = member.read_package_metadata(shell)?;
    let cargo_compete_config =
        crate::config::load_from_rel_path(&member.manifest_path, &package_metadata.config, shell)?;

    let (bin, target_problem) = if let Some(src) = src {
        let src = cwd.join(src.strip_prefix(".").unwrap_or(&src));
        let bin = member.bin_target_by_src_path(src)?;
        let target_problem = &package_metadata.bin_by_bin_name(&bin.name)?.problem;
        (bin, target_problem)
    } else if let Some(index) = index {
        let package_metadata_bin = package_metadata.bin_by_bin_index(index)?;
        let bin = member.bin_target_by_name(&package_metadata_bin.name)?;
        (bin, &package_metadata_bin.problem)
    } else {
        unreachable!()
    };

    crate::testing::test(crate::testing::Args {
        metadata: &metadata,
        member,
        bin,
        cargo_compete_config_test_suite: &cargo_compete_config.test_suite,
        target_problem,
        release: if debug {
            false
        } else if release {
            true
        } else {
            cargo_compete_config.test.profile == CargoCompeteConfigTestProfile::Release
        },
        test_case_names: testcases.map(|ss| ss.into_iter().collect()),
        display_limit,
        shell,
    })
}
