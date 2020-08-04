use crate::{
    project::{PackageMetadataCargoCompeteBin, TargetProblem, TargetProblemYukicoder},
    shell::Shell,
    web::credentials,
};
use cargo_metadata::Package;
use heck::KebabCase as _;
use indexmap::IndexMap;
use liquid::object;
use maplit::btreemap;
use snowchains_core::{
    testsuite::{Additional, BatchTestSuite, TestSuite},
    web::{
        Atcoder, AtcoderRetrieveFullTestCasesCredentials,
        AtcoderRetrieveSampleTestCasesCredentials, AtcoderRetrieveTestCasesTargets, Codeforces,
        CodeforcesRetrieveSampleTestCasesCredentials, CodeforcesRetrieveTestCasesTargets,
        CookieStorage, RetrieveFullTestCases, RetrieveTestCases, RetrieveTestCasesOutcome,
        RetrieveTestCasesOutcomeContest, RetrieveTestCasesOutcomeProblem,
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

pub(crate) fn dl_for_existing_package(
    package: &Package,
    package_metadata_bin: &mut IndexMap<String, PackageMetadataCargoCompeteBin>,
    bin_indexes: Option<&HashSet<String>>,
    full: bool,
    workspace_root: &Path,
    test_suite_path: &liquid::Template,
    shell: &mut Shell,
) -> anyhow::Result<()> {
    let mut atcoder_targets = btreemap!();
    let mut codeforces_targets = btreemap!();
    let mut yukicoder_problem_targets = btreemap!();
    let mut yukicoder_contest_targets = btreemap!();

    for (bin_index, PackageMetadataCargoCompeteBin { problem, .. }) in package_metadata_bin {
        if bin_indexes.map_or(true, |bin_indexes| bin_indexes.contains(bin_index)) {
            match problem {
                TargetProblem::Atcoder { contest, index, .. } => atcoder_targets
                    .entry(contest.clone())
                    .or_insert_with(BTreeMap::new)
                    .insert(index.clone(), bin_index.clone()),
                TargetProblem::Codeforces { contest, index, .. } => codeforces_targets
                    .entry(contest.clone())
                    .or_insert_with(BTreeMap::new)
                    .insert(index.clone(), bin_index.clone()),
                TargetProblem::Yukicoder(target) => match target {
                    TargetProblemYukicoder::Problem { no, .. } => {
                        yukicoder_problem_targets.insert(no.to_string(), bin_index.clone())
                    }
                    TargetProblemYukicoder::Contest { contest, index, .. } => {
                        yukicoder_contest_targets
                            .entry(contest.clone())
                            .or_insert_with(BTreeMap::new)
                            .insert(index.clone(), bin_index.clone())
                    }
                },
            };
        }
    }

    let mut outcomes = vec![];
    let mut urls = vec![];

    for (contest, mut problems) in atcoder_targets {
        let problem_indexes = problems.keys().cloned().collect();
        let outcome = dl_from_atcoder(&contest, Some(problem_indexes), full, shell)?;
        for RetrieveTestCasesOutcomeProblem { index, url, .. } in &outcome.problems {
            if let Some(bin_index) = problems.remove(index) {
                urls.push((bin_index, url.clone()));
            }
        }
        outcomes.push(outcome);
    }
    for (contest, mut problems) in codeforces_targets {
        let problem_indexes = problems.keys().cloned().collect();
        let outcome = dl_from_codeforces(&contest, Some(problem_indexes), shell)?;
        for RetrieveTestCasesOutcomeProblem { index, url, .. } in &outcome.problems {
            if let Some(bin_index) = problems.remove(index) {
                urls.push((bin_index, url.clone()));
            }
        }
        outcomes.push(outcome);
    }
    if !yukicoder_problem_targets.is_empty() {
        let nos = yukicoder_problem_targets.keys().cloned().collect();
        let outcome = dl_from_yukicoder(None, Some(nos), full, shell)?;
        for RetrieveTestCasesOutcomeProblem { index, url, .. } in &outcome.problems {
            if let Some(bin_index) = yukicoder_problem_targets.remove(index) {
                urls.push((bin_index, url.clone()));
            }
        }
        outcomes.push(outcome);
    }
    for (contest, mut problems) in yukicoder_contest_targets {
        let problem_indexes = problems.keys().cloned().collect();
        let outcome = dl_from_yukicoder(Some(&contest), Some(problem_indexes), full, shell)?;
        for RetrieveTestCasesOutcomeProblem { index, url, .. } in &outcome.problems {
            if let Some(bin_index) = problems.remove(index) {
                urls.push((bin_index, url.clone()));
            }
        }
        outcomes.push(outcome);
    }

    for outcome in outcomes {
        save_test_cases(workspace_root, test_suite_path, outcome, shell)?;
    }

    let mut added_url = false;
    let mut new_package_metadata_bin =
        crate::fs::read_to_string(&package.manifest_path)?.parse::<toml_edit::Document>()?;
    let bin = &mut new_package_metadata_bin["package"]["metadata"]["cargo-compete"]["bin"];

    for (bin_index, url) in urls {
        let bin_url = &mut bin[bin_index]["problem"]["url"];
        if bin_url.is_none() {
            *bin_url = toml_edit::value(url.as_str());
            added_url = true;
        }
    }

    if added_url {
        crate::fs::write(&package.manifest_path, new_package_metadata_bin.to_string())?;
        shell.status("Modified", package.manifest_path.display())?;
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
