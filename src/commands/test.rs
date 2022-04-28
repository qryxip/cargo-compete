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
    r"cargo compete test [OPTIONS] <bin-name-or-alias>
    cargo compete test [OPTIONS] --src <PATH>",
))]
pub struct OptCompeteTest {
    /// Path to the source code
    #[structopt(
        long,
        value_name("PATH"),
        required_unless("name-or-alias"),
        conflicts_with("name-or-alias")
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
    /// Name or alias for a `bin`/`example`
    pub name_or_alias: Option<String>,
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
        name_or_alias,
    } = opt;

    let crate::Context {
        cwd,
        cookies_path,
        shell,
    } = ctx;

    shell.set_color_choice(color);

    let manifest_path = manifest_path
        .map(|p| Ok(cwd.join(p.strip_prefix(".").unwrap_or(&p))))
        .unwrap_or_else(|| crate::project::locate_project(&cwd))?;
    let metadata = crate::project::cargo_metadata(&manifest_path, &cwd)?;
    let member = metadata.query_for_member(package.as_deref())?.to_owned();
    let (cargo_compete_config, _) = crate::config::load_for_package(&member, shell)?;

    let (bin_binder, pkg_md_bin_example) = if let Some(src) = src {
        member.bin_binder_from_src(cwd.join(src.strip_prefix(".").unwrap_or(&src)), shell)?
    } else if let Some(name_or_alias) = &name_or_alias {
        member.bin_binder_from_name_or_alias(name_or_alias, shell)?
    } else {
        unreachable!()
    };

    let member_for_arg = metadata.query_for_member(package.as_deref())?;
    crate::testing::test(crate::testing::Args {
        metadata: &metadata,
        member: member_for_arg,
        bin_binder: &bin_binder,
        bin_alias: &pkg_md_bin_example.alias,
        cargo_compete_config_test_suite: &cargo_compete_config.test_suite,
        problem_url: &pkg_md_bin_example.problem,
        toolchain: cargo_compete_config.test.toolchain.as_deref(),
        release: if debug {
            false
        } else if release {
            true
        } else {
            cargo_compete_config.test.profile == CargoCompeteConfigTestProfile::Release
        },
        test_case_names: testcases.map(|ss| ss.into_iter().collect()),
        display_limit,
        cookies_path: &cookies_path,
        shell,
    })
}
