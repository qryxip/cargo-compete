use crate::{
    oj_api,
    project::{PackageExt as _, PackageMetadataCargoCompeteBin},
    shell::Shell,
    web::credentials,
};
use anyhow::Context;
use indexmap::{indexmap, IndexMap};
use krates::cm;
use maplit::btreemap;
use snowchains_core::{
    testsuite::{Additional, BatchTestSuite, Match, PartialBatchTestCase, TestSuite},
    web::{
        Atcoder, AtcoderRetrieveFullTestCasesCredentials,
        AtcoderRetrieveSampleTestCasesCredentials, Codeforces,
        CodeforcesRetrieveSampleTestCasesCredentials, CookieStorage, PlatformKind,
        ProblemsInContest, RetrieveFullTestCases, RetrieveTestCases, RetrieveTestCasesOutcome,
        Yukicoder, YukicoderRetrieveFullTestCasesCredentials, YukicoderRetrieveTestCasesTargets,
    },
};
use std::{
    borrow::BorrowMut as _,
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashSet},
    iter,
    path::{Path, PathBuf},
    time::Duration,
};
use url::Url;

#[allow(clippy::too_many_arguments)]
pub(crate) fn dl_for_existing_package(
    package: &cm::Package,
    package_metadata_bin: &IndexMap<String, PackageMetadataCargoCompeteBin>,
    bin_name_aliases: Option<&HashSet<String>>,
    full: bool,
    workspace_root: &Path,
    test_suite_path: &liquid::Template,
    cookies_path: &Path,
    shell: &mut Shell,
) -> anyhow::Result<()> {
    let mut snowchains_targets: BTreeMap<_, BTreeMap<_, BTreeSet<_>>> = btreemap!();
    let mut oj_targets: BTreeMap<_, BTreeSet<_>> = btreemap!();
    let mut bin_name_aliases = bin_name_aliases.cloned();

    for (name, PackageMetadataCargoCompeteBin { alias, problem }) in package_metadata_bin {
        let matched = bin_name_aliases.as_mut().map_or(true, |bin_name_aliases| {
            bin_name_aliases.remove(name) || bin_name_aliases.remove(alias)
        });
        if matched {
            if let Ok(platform) = PlatformKind::from_url(problem) {
                snowchains_targets
                    .entry(platform)
                    .or_default()
                    .entry(problem)
                    .or_default()
                    .insert((name, alias));
            } else {
                oj_targets.entry(problem).or_default().insert((name, alias));
            }
        }
    }

    for bin_name_or_alias in bin_name_aliases.into_iter().flatten() {
        shell.warn(format!("no such `bin`: {}", bin_name_or_alias))?;
    }

    let mut outcome = vec![];

    if let Some(targets) = snowchains_targets.remove(&PlatformKind::Atcoder) {
        let urls = targets.keys().copied().cloned().collect();
        let targets = ProblemsInContest::Urls { urls };
        outcome.extend(dl_from_atcoder(targets, full, cookies_path, shell)?);
    }

    if let Some(targets) = snowchains_targets.remove(&PlatformKind::Codeforces) {
        let urls = targets.keys().copied().cloned().collect();
        let targets = ProblemsInContest::Urls { urls };
        outcome.extend(dl_from_codeforces(targets, cookies_path, shell)?);
    }

    if let Some(targets) = snowchains_targets.remove(&PlatformKind::Yukicoder) {
        let urls = targets.keys().copied().cloned().collect();
        let targets = YukicoderRetrieveTestCasesTargets::Urls(urls);
        outcome.extend(dl_from_yukicoder(targets, full, shell)?);
    }

    let mut outcome = outcome.into_iter().map(Into::into).collect::<Vec<_>>();

    for url in oj_targets.keys() {
        outcome.push(Problem::from_oj_api(
            oj_api::get_problem(url, full, workspace_root, shell)?,
            full,
        ));
    }

    save_test_cases(
        workspace_root,
        package.manifest_dir_utf8(),
        test_suite_path,
        outcome,
        |url, _| {
            snowchains_targets
                .values()
                .chain(iter::once(&oj_targets))
                .flat_map(|m| m.get(url))
                .flatten()
                .map(|&(bin_name, _)| bin_name.clone())
                .collect()
        },
        |url, _| {
            snowchains_targets
                .values()
                .chain(iter::once(&oj_targets))
                .flat_map(|m| m.get(url))
                .flatten()
                .map(|&(_, bin_alias)| bin_alias.clone())
                .collect()
        },
        shell,
    )?;
    Ok(())
}

pub(crate) fn dl_from_atcoder(
    targets: ProblemsInContest,
    full: bool,
    cookies_path: &Path,
    shell: &mut Shell,
) -> anyhow::Result<Vec<Problem<String>>> {
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
    .map(|RetrieveTestCasesOutcome { problems, .. }| problems.into_iter().map(Into::into).collect())
}

pub(crate) fn dl_from_codeforces(
    targets: ProblemsInContest,
    cookies_path: &Path,
    shell: &mut Shell,
) -> anyhow::Result<Vec<Problem<String>>> {
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
    .map(|RetrieveTestCasesOutcome { problems, .. }| problems.into_iter().map(Into::into).collect())
}

pub(crate) fn dl_from_yukicoder(
    targets: YukicoderRetrieveTestCasesTargets,
    full: bool,
    shell: &mut Shell,
) -> anyhow::Result<Vec<Problem<String>>> {
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
    .map(|RetrieveTestCasesOutcome { problems, .. }| problems.into_iter().map(Into::into).collect())
}

pub(crate) fn save_test_cases<I>(
    workspace_root: &Path,
    pkg_manifest_dir: &str,
    path: &liquid::Template,
    problems: Vec<Problem<I>>,
    bin_names: impl Fn(&Url, &I) -> Vec<String>,
    bin_aliases: impl Fn(&Url, &I) -> Vec<String>,
    shell: &mut Shell,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut acc = vec![];

    for Problem {
        index,
        url,
        mut test_suite,
        text_files,
        ..
    } in problems
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
                path.with_file_name(&bin_alias)
                    .join(dir_file_name)
                    .join(txt_file_name)
                    .with_extension("txt")
            };

            for (name, (input, output)) in &text_files {
                let in_path = txt_path("in", name);
                crate::fs::create_dir_all(in_path.parent().unwrap())?;
                crate::fs::write(in_path, input)?;
                if let Some(out) = output {
                    let out_path = txt_path("out", name);
                    crate::fs::create_dir_all(out_path.parent().unwrap())?;
                    crate::fs::write(out_path, &r#out)?;
                }
            }

            if !text_files.is_empty() {
                if let TestSuite::Batch(BatchTestSuite { cases, extend, .. }) = &mut test_suite {
                    cases.clear();

                    extend.push(Additional::Text {
                        path: format!("./{}", bin_alias),
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
                            path.with_file_name(format!("{{{0}.yml, {0}/}}", bin_alias))
                                .display(),
                        )
                    },
                ),
            )?;
        }
    }

    Ok(acc)
}

pub(crate) struct Problem<I> {
    pub(crate) index: I,
    pub(crate) url: Url,
    pub(crate) test_suite: TestSuite,
    pub(crate) text_files: IndexMap<String, (String, Option<String>)>,
    pub(crate) contest_url: Option<Url>,
}

impl Problem<Option<String>> {
    fn from_oj_api(problem: oj_api::Problem, system: bool) -> Self {
        let (cases, text_files) = if system {
            let num_digits = problem.tests.len().to_string().len();
            let zero_pad = |n: usize| -> String {
                let n = n.to_string();
                itertools::repeat_n('0', num_digits - n.len())
                    .chain(n.chars())
                    .collect()
            };
            let text_files = problem
                .tests
                .into_iter()
                .enumerate()
                .map(
                    |(
                        nth,
                        oj_api::ProblemTest {
                            name,
                            input,
                            output,
                        },
                    )| {
                        (name.unwrap_or_else(|| zero_pad(nth)), (input, Some(output)))
                    },
                )
                .collect();
            (vec![], text_files)
        } else {
            let cases = problem
                .tests
                .into_iter()
                .map(
                    |oj_api::ProblemTest {
                         name,
                         input,
                         output,
                     }| PartialBatchTestCase {
                        name,
                        r#in: input.into(),
                        out: Some(output.into()),
                        timelimit: None,
                        r#match: None,
                    },
                )
                .collect();
            (cases, indexmap!())
        };

        Self {
            index: problem.context.alphabet,
            url: problem.url,
            test_suite: TestSuite::Batch(BatchTestSuite {
                timelimit: problem.time_limit.map(Duration::from_millis),
                r#match: Match::Exact,
                cases,
                extend: vec![],
            }),
            text_files,
            contest_url: problem.context.contest.as_ref().and_then(|c| c.url.clone()),
        }
    }
}

impl Problem<String> {
    pub(crate) fn from_oj_api_with_alphabet(
        problem: oj_api::Problem,
        system: bool,
    ) -> anyhow::Result<Self> {
        let Problem {
            index,
            url,
            test_suite,
            text_files,
            contest_url,
        } = Problem::<Option<String>>::from_oj_api(problem, system);

        let index =
            index.with_context(|| "`context.alphabet` is required for the `new` command")?;

        Ok(Self {
            index,
            url,
            test_suite,
            text_files,
            contest_url,
        })
    }
}

impl From<Problem<String>> for Problem<Option<String>> {
    fn from(problem: Problem<String>) -> Self {
        Self {
            index: Some(problem.index),
            url: problem.url,
            test_suite: problem.test_suite,
            text_files: problem.text_files,
            contest_url: problem.contest_url,
        }
    }
}

impl From<snowchains_core::web::RetrieveTestCasesOutcomeProblem> for Problem<String> {
    fn from(problem: snowchains_core::web::RetrieveTestCasesOutcomeProblem) -> Self {
        Self {
            index: problem.index,
            url: problem.url,
            test_suite: problem.test_suite,
            text_files: problem
                .text_files
                .into_iter()
                .map(|(k, v)| (k, (v.r#in, v.out)))
                .collect(),
            contest_url: problem.contest.map(|c| c.url),
        }
    }
}
