use crate::{
    oj_api,
    project::{PackageExt as _, PackageMetadataCargoCompeteBinExample},
    shell::Shell,
    web::credentials,
};
use anyhow::{ensure, Context};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata as cm;
use indexmap::{indexmap, IndexMap};
use maplit::{btreemap, btreeset};
use percent_encoding::PercentDecode;
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
    borrow::{BorrowMut as _, Cow},
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashSet},
    iter,
    path::{self, Path, PathBuf},
    time::Duration,
};
use url::Url;

pub(crate) fn dl_only_system_test_cases(
    url: &Url,
    cookies_path: &Path,
    cwd: &Utf8Path,
    shell: &mut Shell,
) -> anyhow::Result<()> {
    let system_test_cases_dir = &system_test_cases_dir(url)?;
    let in_dir = &system_test_cases_dir.join("in");
    let out_dir = &system_test_cases_dir.join("out");

    let Problem { text_files, .. } = match url.host_str() {
        Some("atcoder.jp") => {
            let shell = RefCell::new(shell.borrow_mut());

            let username_and_password =
                &mut credentials::username_and_password(&shell, "Username: ", "Password: ");

            take(Atcoder::exec(RetrieveTestCases {
                targets: ProblemsInContest::Urls {
                    urls: btreeset!(url.clone()),
                },
                credentials: AtcoderRetrieveSampleTestCasesCredentials {
                    username_and_password,
                },
                full: Some(RetrieveFullTestCases {
                    credentials: AtcoderRetrieveFullTestCasesCredentials {
                        dropbox_access_token: credentials::dropbox_access_token()?,
                    },
                }),
                cookie_storage: CookieStorage::with_jsonl(cookies_path)?,
                timeout: crate::web::TIMEOUT,
                shell: &shell,
            })?)
        }
        Some("yukicoder.me") => take(Yukicoder::exec(RetrieveTestCases {
            targets: YukicoderRetrieveTestCasesTargets::Urls(btreeset!(url.clone())),
            credentials: (),
            full: Some(RetrieveFullTestCases {
                credentials: YukicoderRetrieveFullTestCasesCredentials {
                    api_key: credentials::yukicoder_api_key(shell)?,
                },
            }),
            cookie_storage: (),
            timeout: crate::web::TIMEOUT,
            shell: &RefCell::new(shell.borrow_mut()),
        })?),
        _ => {
            let problem = oj_api::get_problem(url, true, cwd, shell)?;
            Problem::from_oj_api(problem, true)
        }
    };

    if !text_files.is_empty() {
        crate::fs::create_dir_all(in_dir)?;
    }
    if text_files.values().any(|(_, o)| o.is_some()) {
        crate::fs::create_dir_all(out_dir)?;
    }

    for (name, (input, output)) in text_files {
        let file_name = &format!("{}.txt", name);
        crate::fs::write(in_dir.join(file_name), input)?;
        if let Some(output) = output {
            crate::fs::write(out_dir.join(file_name), output)?;
        }
    }
    return Ok(());

    fn take(outcome: RetrieveTestCasesOutcome) -> Problem<Option<String>> {
        { outcome }.problems.pop().unwrap().into()
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn dl_for_existing_package(
    package: &cm::Package,
    package_metadata_bin: &IndexMap<String, PackageMetadataCargoCompeteBinExample>,
    package_metadata_example: &IndexMap<String, PackageMetadataCargoCompeteBinExample>,
    bin_name_aliases: Option<&HashSet<String>>,
    example_name_aliases: Option<&HashSet<String>>,
    full: bool,
    overwrite: bool,
    workspace_root: &Utf8Path,
    test_suite_path: &liquid::Template,
    cookies_path: &Path,
    shell: &mut Shell,
) -> anyhow::Result<()> {
    let mut snowchains_targets: BTreeMap<_, BTreeMap<_, BTreeSet<_>>> = btreemap!();
    let mut oj_targets: BTreeMap<_, BTreeSet<_>> = btreemap!();
    let mut bin_name_aliases = bin_name_aliases.cloned();
    let mut example_name_aliases = example_name_aliases.cloned();

    for (name, PackageMetadataCargoCompeteBinExample { alias, problem }) in itertools::chain(
        package_metadata_bin.iter().filter(
            |&(name, PackageMetadataCargoCompeteBinExample { alias, .. })| {
                bin_name_aliases
                    .as_mut()
                    .map_or(true, |ss| ss.remove(name) || ss.remove(alias))
            },
        ),
        package_metadata_example.iter().filter(
            |&(name, PackageMetadataCargoCompeteBinExample { alias, .. })| {
                example_name_aliases
                    .as_mut()
                    .map_or(true, |ss| ss.remove(name) || ss.remove(alias))
            },
        ),
    ) {
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

    for bin_name_or_alias in bin_name_aliases.into_iter().flatten() {
        shell.warn(format!("no such `bin`: {}", bin_name_or_alias))?;
    }

    let mut outcome = vec![];

    if let Some(targets) = snowchains_targets.get(&PlatformKind::Atcoder) {
        let urls = targets.keys().copied().cloned().collect();
        let targets = ProblemsInContest::Urls { urls };
        outcome.extend(dl_from_atcoder(targets, full, cookies_path, shell)?);
    }

    if let Some(targets) = snowchains_targets.get(&PlatformKind::Codeforces) {
        let urls = targets.keys().copied().cloned().collect();
        let targets = ProblemsInContest::Urls { urls };
        outcome.extend(dl_from_codeforces(targets, cookies_path, shell)?);
    }

    if let Some(targets) = snowchains_targets.get(&PlatformKind::Yukicoder) {
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
        package.manifest_dir(),
        test_suite_path,
        overwrite,
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

pub(crate) fn system_test_cases_dir(problem_url: &Url) -> anyhow::Result<PathBuf> {
    let system_test_cases_dir = dirs_next::cache_dir()
        .with_context(|| "could not find the cache directory")?
        .join("cargo-compete")
        .join("system-test-cases");

    Ok(iter::once(problem_url.host_str().unwrap_or_default())
        .chain(problem_url.path_segments().into_iter().flatten())
        .map(percent_encoding::percent_decode_str)
        .map(PercentDecode::decode_utf8_lossy)
        .map(Cow::into_owned)
        .fold(system_test_cases_dir, |d, p| d.join(p)))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn save_test_cases<I>(
    workspace_root: &Utf8Path,
    pkg_manifest_dir: &Utf8Path,
    path: &liquid::Template,
    overwrite: bool,
    problems: Vec<Problem<I>>,
    bin_names: impl Fn(&Url, &I) -> Vec<String>,
    bin_aliases: impl Fn(&Url, &I) -> Vec<String>,
    shell: &mut Shell,
) -> anyhow::Result<Vec<Utf8PathBuf>> {
    let mut acc = vec![];

    for Problem {
        index,
        url,
        mut test_suite,
        text_files,
        ..
    } in problems
    {
        let system_test_cases_dir = system_test_cases_dir(&url)?;

        crate::fs::create_dir_all(system_test_cases_dir.join("in"))?;
        if text_files.values().any(|(_, o)| o.is_some()) {
            crate::fs::create_dir_all(system_test_cases_dir.join("out"))?;
        }

        for (name, (input, output)) in &text_files {
            let path = |dir_name: &str| -> _ {
                system_test_cases_dir
                    .join(dir_name)
                    .join(name)
                    .with_extension("txt")
            };
            crate::fs::write(path("in"), input)?;
            if let Some(output) = output {
                crate::fs::write(path("out"), output)?;
            }
        }

        let empty = match &test_suite {
            TestSuite::Batch(BatchTestSuite { cases, .. }) => cases.is_empty(),
            _ => true,
        } && text_files.is_empty();

        let contains_any_out = match &test_suite {
            TestSuite::Batch(BatchTestSuite { cases, .. }) => cases
                .iter()
                .any(|PartialBatchTestCase { out, .. }| out.is_some()),
            _ => false,
        } || text_files.values().any(|(_, o)| o.is_some());

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

            ensure!(
                overwrite || !path.exists(),
                "`{}` already exists. run with `--overwrite` to overwrite",
                path,
            );

            acc.push(path.clone());

            if !empty {
                crate::fs::create_dir_all(path.with_file_name(&bin_alias).join("in"))?;
                crate::fs::write(path.with_file_name(&bin_alias).join("in").join(".gitkeep"), "")?;
            }
            if contains_any_out {
                crate::fs::create_dir_all(path.with_file_name(&bin_alias).join("out"))?;
                crate::fs::write(path.with_file_name(&bin_alias).join("out").join(".gitkeep"), "")?;
            }

            if let TestSuite::Batch(BatchTestSuite { cases, extend, .. }) = &mut test_suite {
                if text_files.is_empty() {
                    extend.push(Additional::Text {
                        path: format!("./{}", bin_alias).into(),
                        r#in: "/in/*.txt".to_owned(),
                        out: "/out/*.txt".to_owned(),
                        timelimit: None,
                        r#match: None,
                    });
                } else {
                    cases.clear();
                    extend.push(Additional::SystemTestCases { problem: None });
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
                    if empty {
                        path.with_file_name(format!("{}.yml", bin_alias))
                    } else {
                        path.with_file_name(format!(
                            "{{{0}.yml, {0}{1}}}",
                            bin_alias,
                            path::MAIN_SEPARATOR,
                        ))
                    },
                ),
            )?;
        }
    }
    acc.sort();
    Ok(acc)
}

#[derive(Debug)]
pub(crate) struct Problem<I> {
    pub(crate) index: I,
    pub(crate) url: Url,
    pub(crate) test_suite: TestSuite,
    pub(crate) text_files: IndexMap<String, (String, Option<String>)>,
    pub(crate) contest_url: Option<Url>,
}

impl Problem<Option<String>> {
    pub(crate) fn from_oj_api(problem: oj_api::Problem, system: bool) -> Self {
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

impl From<snowchains_core::web::RetrieveTestCasesOutcomeProblem> for Problem<Option<String>> {
    fn from(problem: snowchains_core::web::RetrieveTestCasesOutcomeProblem) -> Self {
        Problem::<String>::from(problem).into()
    }
}
