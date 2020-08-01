use crate::{
    project::{
        MetadataExt as _, PackageExt as _, PackageMetadataCargoCompeteProblem,
        PackageMetadataCargoCompeteProblemYukicoder, TemplateString,
        WorkspaceMetadataCargoCompetePlatform,
    },
    shell::{ColorChoice, Shell},
    web::credentials,
};
use anyhow::Context as _;
use heck::KebabCase as _;
use maplit::{btreemap, btreeset};
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
    collections::BTreeSet,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use strum::VariantNames as _;

#[derive(StructOpt, Debug)]
pub struct OptCompeteRetrieveTestcases {
    /// Retrieves system test cases
    #[structopt(long)]
    pub full: bool,

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
        let mut atcoder_targets = btreemap!();
        let mut codeforces_targets = btreemap!();
        let mut yukicoder_problem_targets = btreeset!();
        let mut yukicoder_contest_targets = btreemap!();

        for (_, target) in member.read_package_metadata()?.problems {
            match target {
                PackageMetadataCargoCompeteProblem::Atcoder { contest, index } => atcoder_targets
                    .entry(contest)
                    .or_insert_with(BTreeSet::new)
                    .insert(index),
                PackageMetadataCargoCompeteProblem::Codeforces { contest, index } => {
                    codeforces_targets
                        .entry(contest)
                        .or_insert_with(BTreeSet::new)
                        .insert(index)
                }
                PackageMetadataCargoCompeteProblem::Yukicoder(target) => match target {
                    PackageMetadataCargoCompeteProblemYukicoder::Problem { no } => {
                        yukicoder_problem_targets.insert(no)
                    }
                    PackageMetadataCargoCompeteProblemYukicoder::Contest { contest, index } => {
                        yukicoder_contest_targets
                            .entry(contest)
                            .or_insert_with(BTreeSet::new)
                            .insert(index)
                    }
                },
            };
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
            save_test_cases(
                &metadata.workspace_root,
                &workspace_metadata.test_suite,
                outcome,
                shell,
            )?;
        }
    } else {
        match workspace_metadata.platform {
            WorkspaceMetadataCargoCompetePlatform::Atcoder { .. } => {
                let contest = contest.with_context(|| "`contest` is required for AtCoder")?;
                let problems = problems.map(|ps| ps.into_iter().collect());

                let outcome = dl_from_atcoder(&contest, problems, full, shell)?;

                let package_name = outcome
                    .contest
                    .as_ref()
                    .map(|RetrieveTestCasesOutcomeContest { id, .. }| id)
                    .unwrap_or(&contest);

                let problem_indexes = outcome
                    .problems
                    .iter()
                    .map(|RetrieveTestCasesOutcomeProblem { index, .. }| index.clone())
                    .collect();

                let workspace_root = metadata.workspace_root.clone();

                metadata.add_member(package_name, &problem_indexes, false, shell)?;

                save_test_cases(
                    &workspace_root,
                    &workspace_metadata.test_suite,
                    outcome,
                    shell,
                )?;
            }
            WorkspaceMetadataCargoCompetePlatform::Codeforces => {
                let contest = contest.with_context(|| "`contest` is required for Codeforces")?;
                let problems = problems.map(|ps| ps.into_iter().collect());

                let outcome = dl_from_codeforces(&contest, problems, shell)?;

                let package_name = outcome
                    .contest
                    .as_ref()
                    .map(|RetrieveTestCasesOutcomeContest { id, .. }| id)
                    .unwrap_or(&contest);

                let problem_indexes = outcome
                    .problems
                    .iter()
                    .map(|RetrieveTestCasesOutcomeProblem { index, .. }| index.clone())
                    .collect();

                let workspace_root = metadata.workspace_root.clone();

                metadata.add_member(package_name, &problem_indexes, false, shell)?;

                save_test_cases(
                    &workspace_root,
                    &workspace_metadata.test_suite,
                    outcome,
                    shell,
                )?;
            }
            WorkspaceMetadataCargoCompetePlatform::Yukicoder => {
                let contest = contest.as_deref();
                let problems = problems.map(|ps| ps.into_iter().collect());

                let outcome = dl_from_yukicoder(contest, problems, full, shell)?;

                let package_name = outcome
                    .contest
                    .as_ref()
                    .map(|RetrieveTestCasesOutcomeContest { id, .. }| &**id)
                    .or(contest);
                let is_no = package_name.is_none();
                let package_name = package_name.unwrap_or("problems");

                let problem_indexes = outcome
                    .problems
                    .iter()
                    .map(|RetrieveTestCasesOutcomeProblem { index, .. }| index.clone())
                    .collect();

                let workspace_root = metadata.workspace_root.clone();

                metadata.add_member(package_name, &problem_indexes, is_no, shell)?;

                save_test_cases(
                    &workspace_root,
                    &workspace_metadata.test_suite,
                    outcome,
                    shell,
                )?;
            }
        }
    }
    Ok(())
}

fn dl_from_atcoder(
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

fn dl_from_codeforces(
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

fn dl_from_yukicoder(
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

fn save_test_cases(
    workspace_root: &Path,
    path: &TemplateString,
    outcome: RetrieveTestCasesOutcome,
    shell: &mut Shell,
) -> anyhow::Result<()> {
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
        let path = path.eval(&btreemap!("contest" => contest, "problem" => &index))?;
        let path = Path::new(&path);
        let path = workspace_root.join(path.strip_prefix(".").unwrap_or(&path));

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

    Ok(())
}
