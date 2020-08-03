use crate::{
    project::{MetadataExt as _, WorkspaceMetadataCargoCompetePlatform},
    shell::ColorChoice,
};
use anyhow::Context as _;
use heck::KebabCase as _;
use snowchains_core::web::{
    RetrieveTestCasesOutcome, RetrieveTestCasesOutcomeContest, RetrieveTestCasesOutcomeProblem,
};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use strum::VariantNames as _;
use url::Url;

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

    let member = if let Some(package) = package {
        Some(metadata.query_for_member(Some(&package))?)
    } else {
        metadata
            .workspace_members
            .iter()
            .map(|id| &metadata[id])
            .find(|p| p.manifest_path == manifest_path)
    };

    if let Some(member) = member {
        todo!();
    } else {
        match workspace_metadata.platform {
            WorkspaceMetadataCargoCompetePlatform::Atcoder { .. } => {
                let contest = contest.with_context(|| "`contest` is required for AtCoder")?;
                let problems = problems.map(|ps| ps.into_iter().collect());

                let outcome = crate::web::retrieve_testcases::dl_from_atcoder(
                    &contest, problems, full, shell,
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

                metadata.add_member(package_name, &problems, false, shell)?;

                let file_paths = itertools::zip_eq(
                    src_paths(&pkg_manifest_dir, &outcome),
                    crate::web::retrieve_testcases::save_test_cases(
                        &workspace_root,
                        &workspace_metadata.test_suite,
                        outcome,
                        shell,
                    )?,
                )
                .collect::<Vec<_>>();

                if open {
                    crate::open::open(
                        &urls,
                        workspace_metadata.open,
                        &file_paths,
                        &pkg_manifest_dir,
                        &cwd,
                        shell,
                    )?;
                }
            }
            WorkspaceMetadataCargoCompetePlatform::Codeforces => {
                let contest = contest.with_context(|| "`contest` is required for Codeforces")?;
                let problems = problems.map(|ps| ps.into_iter().collect());

                let outcome =
                    crate::web::retrieve_testcases::dl_from_codeforces(&contest, problems, shell)?;

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

                metadata.add_member(package_name, &problems, false, shell)?;

                let file_paths = itertools::zip_eq(
                    src_paths(&pkg_manifest_dir, &outcome),
                    crate::web::retrieve_testcases::save_test_cases(
                        &workspace_root,
                        &workspace_metadata.test_suite,
                        outcome,
                        shell,
                    )?,
                )
                .collect::<Vec<_>>();

                if open {
                    crate::open::open(
                        &urls,
                        workspace_metadata.open,
                        &file_paths,
                        &pkg_manifest_dir,
                        &cwd,
                        shell,
                    )?;
                }
            }
            WorkspaceMetadataCargoCompetePlatform::Yukicoder => {
                let contest = contest.as_deref();
                let problems = problems.map(|ps| ps.into_iter().collect());

                let outcome = crate::web::retrieve_testcases::dl_from_yukicoder(
                    contest, problems, full, shell,
                )?;

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

                metadata.add_member(package_name, &problems, is_no, shell)?;

                let file_paths = itertools::zip_eq(
                    src_paths(&pkg_manifest_dir, &outcome),
                    crate::web::retrieve_testcases::save_test_cases(
                        &workspace_root,
                        &workspace_metadata.test_suite,
                        outcome,
                        shell,
                    )?,
                )
                .collect::<Vec<_>>();

                if open {
                    crate::open::open(
                        &urls,
                        workspace_metadata.open,
                        &file_paths,
                        &pkg_manifest_dir,
                        &cwd,
                        shell,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn src_paths(pkg_manifest_dir: &Path, outcome: &RetrieveTestCasesOutcome) -> Vec<PathBuf> {
    outcome
        .problems
        .iter()
        .map(|problem| {
            pkg_manifest_dir
                .join("src")
                .join("bin")
                .join(problem.index.to_kebab_case())
                .with_extension("rs")
        })
        .collect()
}

fn urls(outcome: &RetrieveTestCasesOutcome) -> Vec<Url> {
    outcome.problems.iter().map(|p| p.url.clone()).collect()
}
