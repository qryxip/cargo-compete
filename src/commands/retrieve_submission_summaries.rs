use crate::{
    project::{MetadataExt as _, PackageExt as _, PackageMetadataCargoCompeteBin, TargetProblem},
    shell::ColorChoice,
    web::credentials,
};
use anyhow::bail;
use indexmap::indexset;
use snowchains_core::web::{
    Atcoder, AtcoderRetrieveSubmissionSummariesCredentials,
    AtcoderRetrieveSubmissionSummariesTarget, CookieStorage, RetrieveSubmissionSummaries,
};
use std::{borrow::BorrowMut as _, cell::RefCell, path::PathBuf};
use structopt::StructOpt;
use strum::VariantNames as _;

#[derive(StructOpt, Debug)]
pub struct OptCompeteRetrieveSubmissionSummaries {
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

    /// Index for `package.metadata.cargo-compete.bin`
    pub problem: Option<String>,
}

pub(crate) fn run(
    opt: OptCompeteRetrieveSubmissionSummaries,
    ctx: crate::Context<'_>,
) -> anyhow::Result<()> {
    let OptCompeteRetrieveSubmissionSummaries {
        package,
        manifest_path,
        color,
        problem,
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
    let metadata = crate::project::cargo_metadata(&manifest_path, cwd)?;
    let member = metadata.query_for_member(package.as_deref())?;
    let package_metadata = member.read_package_metadata(shell)?;
    crate::config::load_from_rel_path(&member.manifest_path, &package_metadata.config, shell)?;

    let mut atcoder_targets = indexset!();

    for (
        bin_index,
        PackageMetadataCargoCompeteBin {
            problem: target, ..
        },
    ) in &package_metadata.bin
    {
        if problem.as_ref().map_or(true, |p| p == bin_index) {
            match target {
                TargetProblem::Atcoder { contest, .. } => {
                    atcoder_targets.insert(contest.clone());
                }
                TargetProblem::Codeforces { .. } => {
                    todo!("`retrieve submission-summaries` for Codeforces is not implemented");
                }
                TargetProblem::Yukicoder(_) => {
                    todo!("`retrieve submission-summaries` for yukicoder is not implemented");
                }
            }
        }
    }

    if atcoder_targets.len() > 1 {
        bail!("found multiple candicates. specify the target with argument");
    }

    let cookie_storage = CookieStorage::with_jsonl(cookies_path)?;
    let timeout = crate::web::TIMEOUT;

    if let Some(contest) = atcoder_targets.into_iter().next() {
        let outcome = {
            let shell = RefCell::new(shell.borrow_mut());

            let credentials = AtcoderRetrieveSubmissionSummariesCredentials {
                username_and_password: &mut credentials::username_and_password(
                    &shell,
                    "Username: ",
                    "Password: ",
                ),
            };

            Atcoder::exec(RetrieveSubmissionSummaries {
                target: AtcoderRetrieveSubmissionSummariesTarget { contest },
                credentials,
                cookie_storage,
                timeout,
                shell: &shell,
            })?
        };

        writeln!(shell.out(), "{}", outcome.to_json())?;
        shell.out().flush()?;
    } else {
        bail!("`package.metadata.cargo-compete.bin` is empty");
    }

    Ok(())
}
