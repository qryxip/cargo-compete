use crate::{
    project::{PackageMetadataCargoCompeteBin, TargetProblem, TargetProblemYukicoder},
    shell::Shell,
    web::credentials,
};
use heck::KebabCase as _;
use indexmap::IndexMap;
use liquid::object;
use maplit::{btreemap, btreeset};
use snowchains_core::{
    testsuite::{Additional, BatchTestSuite, TestSuite},
    web::{
        Atcoder, AtcoderRetrieveFullTestCasesCredentials,
        AtcoderRetrieveSampleTestCasesCredentials, AtcoderRetrieveTestCasesTargets, Codeforces,
        CodeforcesRetrieveSampleTestCasesCredentials, CodeforcesRetrieveTestCasesTargets,
        CookieStorage, RetrieveFullTestCases, RetrieveTestCases, RetrieveTestCasesOutcome,
        RetrieveTestCasesOutcomeContest, RetrieveTestCasesOutcomeProblemTextFiles, Yukicoder,
        YukicoderRetrieveFullTestCasesCredentials, YukicoderRetrieveTestCasesTargets,
    },
};
use std::{
    borrow::BorrowMut as _,
    cell::RefCell,
    collections::{BTreeSet, HashSet},
    path::{Path, PathBuf},
};

pub(crate) fn dl_for_existing_package(
    package_metadata_bin: &IndexMap<String, PackageMetadataCargoCompeteBin>,
    indexes: Option<&HashSet<String>>,
    full: bool,
    workspace_root: &Path,
    test_suite_path: &liquid::Template,
    shell: &mut Shell,
) -> anyhow::Result<()> {
    let mut atcoder_targets = btreemap!();
    let mut codeforces_targets = btreemap!();
    let mut yukicoder_problem_targets = btreeset!();
    let mut yukicoder_contest_targets = btreemap!();

    for (index, PackageMetadataCargoCompeteBin { problem, .. }) in package_metadata_bin {
        if indexes.map_or(true, |indexes| indexes.contains(index)) {
            match problem {
                TargetProblem::Atcoder { contest, index, .. } => atcoder_targets
                    .entry(contest.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(index.clone()),
                TargetProblem::Codeforces { contest, index, .. } => codeforces_targets
                    .entry(contest.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(index.clone()),
                TargetProblem::Yukicoder(target) => match target {
                    TargetProblemYukicoder::Problem { no, .. } => {
                        yukicoder_problem_targets.insert(*no)
                    }
                    TargetProblemYukicoder::Contest { contest, index, .. } => {
                        yukicoder_contest_targets
                            .entry(contest.clone())
                            .or_insert_with(BTreeSet::new)
                            .insert(index.clone())
                    }
                },
            };
        }
    }

    let mut outcomes = vec![];

    for (contest, problems) in atcoder_targets {
        outcomes.push(dl_from_atcoder(&contest, Some(problems), full, shell)?);
    }
    for (contest, problems) in codeforces_targets {
        outcomes.push(dl_from_codeforces(&contest, Some(problems), shell)?);
    }
    if !yukicoder_problem_targets.is_empty() {
        outcomes.push(dl_from_yukicoder(
            None,
            Some(
                yukicoder_problem_targets
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            ),
            full,
            shell,
        )?);
    }
    for (contest, problems) in yukicoder_contest_targets {
        outcomes.push(dl_from_yukicoder(
            Some(&contest),
            Some(problems),
            full,
            shell,
        )?);
    }

    for outcome in outcomes {
        save_test_cases(workspace_root, test_suite_path, outcome, shell)?;
    }
    Ok(())
}

pub(crate) fn dl_from_atcoder(
    contest: &str,
    problems: Option<BTreeSet<String>>,
    full: bool,
    shell: &mut Shell,
) -> anyhow::Result<RetrieveTestCasesOutcome> {
    let targets = AtcoderRetrieveTestCasesTargets {
        contest: contest.to_owned(),
        problems,
    };

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

    let cookie_storage = CookieStorage::with_jsonl(credentials::cookies_path()?)?;

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
    contest: &str,
    problems: Option<BTreeSet<String>>,
    shell: &mut Shell,
) -> anyhow::Result<RetrieveTestCasesOutcome> {
    let targets = CodeforcesRetrieveTestCasesTargets {
        contest: contest.to_owned(),
        problems,
    };

    let shell = RefCell::new(shell.borrow_mut());

    let credentials = CodeforcesRetrieveSampleTestCasesCredentials {
        username_and_password: &mut credentials::username_and_password(
            &shell,
            "Username: ",
            "Password: ",
        ),
    };

    let cookie_storage = CookieStorage::with_jsonl(credentials::cookies_path()?)?;

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
    contest: Option<&str>,
    problems: Option<BTreeSet<String>>,
    full: bool,
    shell: &mut Shell,
) -> anyhow::Result<RetrieveTestCasesOutcome> {
    let targets = if let Some(contest) = contest {
        YukicoderRetrieveTestCasesTargets::Contest(contest.to_owned(), problems)
    } else {
        YukicoderRetrieveTestCasesTargets::ProblemNos(problems.unwrap_or_default())
    };

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
    path: &liquid::Template,
    outcome: RetrieveTestCasesOutcome,
    shell: &mut Shell,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut acc = vec![];

    let contest = outcome
        .contest
        .as_ref()
        .map(|RetrieveTestCasesOutcomeContest { id, .. }| &**id)
        .unwrap_or("problems");

    for snowchains_core::web::RetrieveTestCasesOutcomeProblem {
        index,
        mut test_suite,
        text_files,
        ..
    } in outcome.problems
    {
        let path = path.render(&object!({ "contest": contest, "problem": &index }))?;
        let path = Path::new(&path);
        let path = workspace_root.join(path.strip_prefix(".").unwrap_or(&path));

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
                    TestSuite::Interactive(_) => "no test cases (interactive problem)".to_owned(),
                    TestSuite::Unsubmittable => "no test cases (unsubmittable problem)".to_owned(),
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

    Ok(acc)
}
