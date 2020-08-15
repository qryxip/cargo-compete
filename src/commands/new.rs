use crate::{project::MetadataExt as _, shell::ColorChoice};
use anyhow::Context as _;
use snowchains_core::web::{
    PlatformKind, RetrieveTestCasesOutcome, RetrieveTestCasesOutcomeContest,
    RetrieveTestCasesOutcomeProblem,
};
use std::path::PathBuf;
use structopt::StructOpt;
use strum::VariantNames as _;
use url::Url;

#[derive(StructOpt, Debug)]
pub struct OptCompeteNew {
    /// Retrieve system test cases
    #[structopt(long)]
    pub full: bool,

    /// Open URLs and files
    #[structopt(long)]
    pub open: bool,

    /// Retrieve only the problems
    #[structopt(long, value_name("INDEX"))]
    pub problems: Option<Vec<String>>,

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

    /// Contest ID. Required for some platforms
    pub contest: Option<String>,
}

pub fn run(opt: OptCompeteNew, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteNew {
        full,
        open,
        problems,
        manifest_path,
        color,
        contest,
    } = opt;

    let crate::Context {
        cwd,
        cookies_path,
        shell,
    } = ctx;

    shell.set_color_choice(color);

    let manifest_path = manifest_path
        .map(|p| Ok(cwd.join(p.strip_prefix(".").unwrap_or(&p))))
        .unwrap_or_else(|| crate::project::locate_project(cwd))?;
    let metadata = crate::project::cargo_metadata(&manifest_path)?;
    let cargo_compete_config = metadata.read_compete_toml()?;

    match cargo_compete_config.template.platform {
        PlatformKind::Atcoder => {
            let contest = contest.with_context(|| "`contest` is required for AtCoder")?;
            let problems = problems.map(|ps| ps.into_iter().collect());

            let outcome = crate::web::retrieve_testcases::dl_from_atcoder(
                &contest,
                problems,
                full,
                &cookies_path,
                shell,
            )?;

            let package_name = outcome
                .contest
                .as_ref()
                .map(|RetrieveTestCasesOutcomeContest { id, .. }| id)
                .unwrap_or(&contest);

            let problems = outcome
                .problems
                .iter()
                .map(|RetrieveTestCasesOutcomeProblem { index, url, .. }| (&**index, url))
                .collect();

            let workspace_root = metadata.workspace_root.clone();
            let pkg_manifest_dir = metadata.workspace_root.join(package_name);
            let urls = urls(&outcome);

            let file_paths = itertools::zip_eq(
                metadata.add_member(package_name, &problems, false, shell)?,
                crate::web::retrieve_testcases::save_test_cases(
                    &workspace_root,
                    &pkg_manifest_dir.to_str().expect("this is from JSON"),
                    &cargo_compete_config.test_suite,
                    outcome,
                    shell,
                )?,
            )
            .collect::<Vec<_>>();

            if open {
                crate::open::open(
                    &urls,
                    cargo_compete_config.open,
                    &file_paths,
                    &pkg_manifest_dir,
                    &workspace_root,
                    shell,
                )?;
            }
        }
        PlatformKind::Codeforces => {
            let contest = contest.with_context(|| "`contest` is required for Codeforces")?;
            let problems = problems.map(|ps| ps.into_iter().collect());

            let outcome = crate::web::retrieve_testcases::dl_from_codeforces(
                &contest,
                problems,
                &cookies_path,
                shell,
            )?;

            let package_name = outcome
                .contest
                .as_ref()
                .map(|RetrieveTestCasesOutcomeContest { id, .. }| id)
                .unwrap_or(&contest);

            let problems = outcome
                .problems
                .iter()
                .map(|RetrieveTestCasesOutcomeProblem { index, url, .. }| (&**index, url))
                .collect();

            let workspace_root = metadata.workspace_root.clone();
            let pkg_manifest_dir = metadata.workspace_root.join(package_name);
            let urls = urls(&outcome);

            let file_paths = itertools::zip_eq(
                metadata.add_member(package_name, &problems, false, shell)?,
                crate::web::retrieve_testcases::save_test_cases(
                    &workspace_root,
                    &pkg_manifest_dir.to_str().expect("this is from JSON"),
                    &cargo_compete_config.test_suite,
                    outcome,
                    shell,
                )?,
            )
            .collect::<Vec<_>>();

            if open {
                crate::open::open(
                    &urls,
                    cargo_compete_config.open,
                    &file_paths,
                    &pkg_manifest_dir,
                    &workspace_root,
                    shell,
                )?;
            }
        }
        PlatformKind::Yukicoder => {
            let contest = contest.as_deref();
            let problems = problems.map(|ps| ps.into_iter().collect());

            let outcome =
                crate::web::retrieve_testcases::dl_from_yukicoder(contest, problems, full, shell)?;

            let package_name = outcome
                .contest
                .as_ref()
                .map(|RetrieveTestCasesOutcomeContest { id, .. }| &**id)
                .or(contest);
            let is_no = package_name.is_none();
            let package_name = package_name.unwrap_or("problems");

            let problems = outcome
                .problems
                .iter()
                .map(|RetrieveTestCasesOutcomeProblem { index, url, .. }| (&**index, url))
                .collect();

            let workspace_root = metadata.workspace_root.clone();
            let pkg_manifest_dir = metadata.workspace_root.join(package_name);
            let urls = urls(&outcome);

            let file_paths = itertools::zip_eq(
                metadata.add_member(package_name, &problems, is_no, shell)?,
                crate::web::retrieve_testcases::save_test_cases(
                    &workspace_root,
                    &pkg_manifest_dir.to_str().expect("this is from JSON"),
                    &cargo_compete_config.test_suite,
                    outcome,
                    shell,
                )?,
            )
            .collect::<Vec<_>>();

            if open {
                crate::open::open(
                    &urls,
                    cargo_compete_config.open,
                    &file_paths,
                    &pkg_manifest_dir,
                    &workspace_root,
                    shell,
                )?;
            }
        }
    }
    Ok(())
}

fn urls(outcome: &RetrieveTestCasesOutcome) -> Vec<Url> {
    outcome.problems.iter().map(|p| p.url.clone()).collect()
}
