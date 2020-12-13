use crate::{
    project::{PackageExt as _, PackageMetadataCargoCompeteBin},
    shell::Shell,
    web::credentials,
};
use heck::KebabCase as _;
use indexmap::IndexMap;
use krates::cm;
use maplit::btreemap;
use snowchains_core::{
    testsuite::{Additional, BatchTestSuite, TestSuite},
    web::{
        Atcoder, AtcoderRetrieveFullTestCasesCredentials,
        AtcoderRetrieveSampleTestCasesCredentials, Codeforces,
        CodeforcesRetrieveSampleTestCasesCredentials, CookieStorage, PlatformKind,
        ProblemsInContest, RetrieveFullTestCases, RetrieveTestCases, RetrieveTestCasesOutcome,
        RetrieveTestCasesOutcomeProblemTextFiles, Yukicoder,
        YukicoderRetrieveFullTestCasesCredentials, YukicoderRetrieveTestCasesTargets,
    },
};
use std::{
    borrow::BorrowMut as _,
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashSet},
    path::{Path, PathBuf},
};
use url::Url;

#[allow(clippy::too_many_arguments)]
pub(crate) fn dl_for_existing_package(
    package: &cm::Package,
    package_metadata_bin: &IndexMap<String, PackageMetadataCargoCompeteBin>,
    bin_indexes: Option<&HashSet<String>>,
    full: bool,
    workspace_root: &Path,
    test_suite_path: &liquid::Template,
    cookies_path: &Path,
    shell: &mut Shell,
) -> anyhow::Result<()> {
    let mut targets: BTreeMap<_, BTreeMap<_, BTreeSet<_>>> = btreemap!();
    let mut bin_indexes = bin_indexes.cloned();

    for (bin_index, PackageMetadataCargoCompeteBin { name, problem, .. }) in package_metadata_bin {
        if bin_indexes
            .as_mut()
            .map_or(true, |bin_indexes| bin_indexes.remove(bin_index))
        {
            targets
                .entry(PlatformKind::from_url(problem)?)
                .or_default()
                .entry(problem)
                .or_default()
                .insert((name, bin_index));
        }
    }

    for bin_index in bin_indexes.into_iter().flatten() {
        shell.warn(format!("no such index: {}", bin_index))?;
    }

    let mut outcomes = vec![];

    if let Some(targets) = targets.remove(&PlatformKind::Atcoder) {
        let urls = targets.keys().copied().cloned().collect();
        let targets = ProblemsInContest::Urls { urls };
        outcomes.push(dl_from_atcoder(targets, full, cookies_path, shell)?);
    }

    if let Some(targets) = targets.remove(&PlatformKind::Codeforces) {
        let urls = targets.keys().copied().cloned().collect();
        let targets = ProblemsInContest::Urls { urls };
        outcomes.push(dl_from_codeforces(targets, cookies_path, shell)?);
    }

    if let Some(targets) = targets.remove(&PlatformKind::Yukicoder) {
        let urls = targets.keys().copied().cloned().collect();
        let targets = YukicoderRetrieveTestCasesTargets::Urls(urls);
        outcomes.push(dl_from_yukicoder(targets, full, shell)?);
    }

    for outcome in outcomes {
        save_test_cases(
            workspace_root,
            package.manifest_dir_utf8(),
            test_suite_path,
            outcome,
            |url, _| {
                targets
                    .values()
                    .flat_map(|m| m.get(url))
                    .flatten()
                    .map(|&(bin_name, _)| bin_name.clone())
                    .collect()
            },
            |url, _| {
                targets
                    .values()
                    .flat_map(|m| m.get(url))
                    .flatten()
                    .map(|&(_, bin_alias)| bin_alias.clone())
                    .collect()
            },
            shell,
        )?;
    }
    Ok(())
}

pub(crate) fn dl_from_atcoder(
    targets: ProblemsInContest,
    full: bool,
    cookies_path: &Path,
    shell: &mut Shell,
) -> anyhow::Result<RetrieveTestCasesOutcome> {
    let shell = RefCell::new(shell.borrow_mut());

    let credentials = AtcoderRetrieveSampleTestCasesCredentials {
        username_and_password: &mut credentials::username_and_password(
            &shell,
            "Username: ",
            "Password: ",
        ),
    };

    let full = if full {
        Some(RetrieveFullTestCases {
            credentials: AtcoderRetrieveFullTestCasesCredentials {
                dropbox_access_token: credentials::dropbox_access_token()?,
            },
        })
    } else {
        None
    };

    let cookie_storage = CookieStorage::with_jsonl(cookies_path)?;

    Atcoder::exec(RetrieveTestCases {
        targets,
        credentials,
        full,
        cookie_storage,
        timeout: crate::web::TIMEOUT,
        shell: &shell,
    })
}

pub(crate) fn dl_from_codeforces(
    targets: ProblemsInContest,
    cookies_path: &Path,
    shell: &mut Shell,
) -> anyhow::Result<RetrieveTestCasesOutcome> {
    let shell = RefCell::new(shell.borrow_mut());

    let credentials = CodeforcesRetrieveSampleTestCasesCredentials {
        username_and_password: &mut credentials::username_and_password(
            &shell,
            "Username: ",
            "Password: ",
        ),
    };

    let cookie_storage = CookieStorage::with_jsonl(cookies_path)?;

    Codeforces::exec(RetrieveTestCases {
        targets,
        credentials,
        full: None,
        cookie_storage,
        timeout: crate::web::TIMEOUT,
        shell: &shell,
    })
}

pub(crate) fn dl_from_yukicoder(
    targets: YukicoderRetrieveTestCasesTargets,
    full: bool,
    shell: &mut Shell,
) -> anyhow::Result<RetrieveTestCasesOutcome> {
    let full = if full {
        Some(RetrieveFullTestCases {
            credentials: YukicoderRetrieveFullTestCasesCredentials {
                api_key: credentials::yukicoder_api_key(shell)?,
            },
        })
    } else {
        None
    };

    let shell = RefCell::new(shell.borrow_mut());

    Yukicoder::exec(RetrieveTestCases {
        targets,
        credentials: (),
        full,
        cookie_storage: (),
        timeout: crate::web::TIMEOUT,
        shell: &shell,
    })
}

pub(crate) fn save_test_cases(
    workspace_root: &Path,
    pkg_manifest_dir: &str,
    path: &liquid::Template,
    outcome: RetrieveTestCasesOutcome,
    bin_names: impl Fn(&Url, &str) -> Vec<String>,
    bin_aliases: impl Fn(&Url, &str) -> Vec<String>,
    shell: &mut Shell,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut acc = vec![];

    for snowchains_core::web::RetrieveTestCasesOutcomeProblem {
        index,
        url,
        mut test_suite,
        text_files,
        ..
    } in outcome.problems
    {
        for (bin_name, bin_alias) in bin_names(&url, &index).into_iter().flat_map(|bin_name| {
            bin_aliases(&url, &index)
                .into_iter()
                .map(move |bin_alias| (bin_name.clone(), bin_alias))
        }) {
            let path = crate::testing::test_suite_path(
                workspace_root,
                pkg_manifest_dir,
                path,
                &bin_name,
                &bin_alias,
                &url,
                shell,
            )?;

            acc.push(path.clone());

            let txt_path = |dir_file_name: &str, txt_file_name: &str| -> _ {
                path.with_file_name(index.to_kebab_case())
                    .join(dir_file_name)
                    .join(txt_file_name)
                    .with_extension("txt")
            };

            for (name, RetrieveTestCasesOutcomeProblemTextFiles { r#in, out }) in &text_files {
                let in_path = txt_path("in", name);
                crate::fs::create_dir_all(in_path.parent().unwrap())?;
                crate::fs::write(in_path, &r#in)?;
                if let Some(out) = out {
                    let out_path = txt_path("out", name);
                    crate::fs::create_dir_all(out_path.parent().unwrap())?;
                    crate::fs::write(out_path, &r#out)?;
                }
            }

            if !text_files.is_empty() {
                if let TestSuite::Batch(BatchTestSuite { cases, extend, .. }) = &mut test_suite {
                    cases.clear();

                    extend.push(Additional::Text {
                        path: format!("./{}", index.to_kebab_case()),
                        r#in: "/in/*.txt".to_owned(),
                        out: "/out/*.txt".to_owned(),
                        timelimit: None,
                        r#match: None,
                    })
                }
            }

            crate::fs::create_dir_all(path.parent().unwrap())?;
            crate::fs::write(&path, test_suite.to_yaml_pretty())?;

            shell.status(
                "Saved",
                format!(
                    "{} to {}",
                    match &test_suite {
                        TestSuite::Batch(BatchTestSuite { cases, .. }) => {
                            match cases.len() + text_files.len() {
                                0 => "no test cases".to_owned(),
                                1 => "1 test case".to_owned(),
                                n => format!("{} test cases", n),
                            }
                        }
                        TestSuite::Interactive(_) =>
                            "no test cases (interactive problem)".to_owned(),
                        TestSuite::Unsubmittable =>
                            "no test cases (unsubmittable problem)".to_owned(),
                    },
                    if text_files.is_empty() {
                        format!("{}", path.display())
                    } else {
                        format!(
                            "{}",
                            path.with_file_name(format!(
                                "{{{index}.yml, {index}/}}",
                                index = index.to_kebab_case(),
                            ))
                            .display(),
                        )
                    },
                ),
            )?;
        }
    }

    Ok(acc)
}
