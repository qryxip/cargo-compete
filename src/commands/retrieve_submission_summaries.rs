use crate::{
    project::{MetadataExt as _, PackageExt as _, PackageMetadataCargoCompeteBinExample},
    shell::ColorChoice,
    web::credentials,
};
use anyhow::{bail, Context as _};
use indexmap::indexset;
use snowchains_core::web::{
    Atcoder, AtcoderRetrieveSubmissionSummariesCredentials,
    AtcoderRetrieveSubmissionSummariesTarget, CookieStorage, PlatformKind,
    RetrieveSubmissionSummaries,
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

    /// Name or alias for a binary
    pub bin_name_or_alias: Option<String>,
}

pub(crate) fn run(
    opt: OptCompeteRetrieveSubmissionSummaries,
    ctx: crate::Context<'_>,
) -> anyhow::Result<()> {
    let OptCompeteRetrieveSubmissionSummaries {
        package,
        manifest_path,
        color,
        bin_name_or_alias,
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
    crate::config::load_for_package(member, shell)?;

    let mut atcoder_targets = indexset!();

    for (
        bin_name,
        PackageMetadataCargoCompeteBinExample {
            alias: bin_alias,
            problem: url,
            ..
        },
    ) in &package_metadata.bin
    {
        if bin_name_or_alias
            .as_ref()
            .map_or(true, |s| [bin_name, bin_alias].contains(&s))
        {
            match PlatformKind::from_url(url).with_context(|| "unsupported platform")? {
                PlatformKind::Atcoder => {
                    atcoder_targets.insert(snowchains_core::web::atcoder_contest_id(url)?);
                }
                PlatformKind::Codeforces => {
                    todo!("`retrieve submission-summaries` for Codeforces is not implemented");
                }
                PlatformKind::Yukicoder => {
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
