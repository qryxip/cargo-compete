use crate::{
    project::{MetadataExt as _, PackageExt as _},
    shell::ColorChoice,
};
use anyhow::Context as _;
use human_size::Size;
use std::path::PathBuf;
use structopt::StructOpt;
use strum::VariantNames as _;

#[derive(StructOpt, Debug)]
pub struct OptCompeteTest {
    /// Test for only the test cases
    #[structopt(long, value_name("NAME"))]
    pub testcases: Option<Vec<String>>,

    /// Display limit
    #[structopt(long, value_name("SIZE"), default_value("4KiB"))]
    pub display_limit: Size,

    /// Existing package to retrieving test cases for
    #[structopt(short, long, value_name("SPEC"))]
    pub package: Option<String>,

    /// Build the artifact in release mode, with optimizations
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

    /// Problem Index
    pub problem: String,
}

pub(crate) fn run(opt: OptCompeteTest, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteTest {
        testcases,
        display_limit,
        package,
        release,
        manifest_path,
        color,
        problem,
    } = opt;

    let crate::Context {
        cwd,
        cookies_path: _,
        shell,
    } = ctx;

    shell.set_color_choice(color);

    let manifest_path = manifest_path
        .map(Ok)
        .unwrap_or_else(|| crate::project::locate_project(cwd))?;
    let metadata = crate::project::cargo_metadata(&manifest_path)?;

    let cargo_compete_config = metadata.read_compete_toml()?;

    let member = metadata.query_for_member(package.as_deref())?;

    let package_metadata_bin = member
        .read_package_metadata()?
        .bin
        .remove(&problem)
        .with_context(|| {
            format!(
                "could not find `{}` in `package.metadata.cargo-compete.bin`",
                problem
            )
        })?;

    crate::testing::test(crate::testing::Args {
        metadata: &metadata,
        member,
        cargo_compete_config_test_suite: &cargo_compete_config.test_suite,
        package_metadata_bin: &package_metadata_bin,
        release,
        test_case_names: testcases.map(|ss| ss.into_iter().collect()),
        display_limit,
        shell,
    })
}
