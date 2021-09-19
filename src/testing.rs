use crate::{project::PackageExt as _, shell::Shell};
use anyhow::ensure;
use az::SaturatingAs as _;
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata as cm;
use human_size::{Byte, Size};
use liquid::object;
use maplit::btreemap;
use snowchains_core::{
    judge::CommandExpression,
    testsuite::{PartialBatchTestCase, TestSuite},
    web::PlatformKind,
};
use std::{
    collections::{BTreeMap, HashSet},
    env,
    path::Path,
    sync::Arc,
};
use url::Url;

pub(crate) struct Args<'a> {
    pub(crate) metadata: &'a cm::Metadata,
    pub(crate) member: &'a cm::Package,
    pub(crate) bin: &'a cm::Target,
    pub(crate) bin_alias: &'a str,
    pub(crate) cargo_compete_config_test_suite: &'a liquid::Template,
    pub(crate) problem_url: &'a Url,
    pub(crate) release: bool,
    pub(crate) test_case_names: Option<HashSet<String>>,
    pub(crate) display_limit: Size,
    pub(crate) cookies_path: &'a Path,
    pub(crate) shell: &'a mut Shell,
}

pub(crate) fn test(args: Args<'_>) -> anyhow::Result<()> {
    let Args {
        metadata,
        member,
        bin,
        bin_alias,
        cargo_compete_config_test_suite,
        problem_url,
        release,
        test_case_names,
        display_limit,
        cookies_path,
        shell,
    } = args;

    let test_suite_path = test_suite_path(
        &metadata.workspace_root,
        member.manifest_dir(),
        cargo_compete_config_test_suite,
        &bin.name,
        bin_alias,
        problem_url,
        shell,
    )?;

    let test_suite = crate::fs::read_yaml(&test_suite_path)?;

    let test_cases = match test_suite {
        TestSuite::Batch(test_suite) => test_suite.load_test_cases(
            test_suite_path.parent().unwrap().as_ref(),
            test_case_names,
            |override_problem_url| {
                fn read(path: &Path) -> anyhow::Result<Arc<str>> {
                    crate::fs::read_to_string(path).map(Into::into)
                }

                let problem_url = override_problem_url.unwrap_or(problem_url);

                let system_test_cases_dir =
                    crate::web::retrieve_testcases::system_test_cases_dir(problem_url)?;

                let text_files = |dir_name: &str| -> anyhow::Result<Vec<_>> {
                    let paths = crate::fs::read_dir(system_test_cases_dir.join(dir_name))?;
                    Ok(paths
                        .into_iter()
                        .filter(|p| p.extension() == Some("txt".as_ref()))
                        .map(|p| {
                            let s = p
                                .file_stem()
                                .expect("should not be empty")
                                .to_string_lossy()
                                .into_owned();
                            (s, p)
                        })
                        .collect())
                };

                if !system_test_cases_dir.join("in").exists() {
                    crate::web::retrieve_testcases::dl_only_system_test_cases(
                        problem_url,
                        cookies_path,
                        &metadata.workspace_root,
                        shell,
                    )?;
                }

                let mut system_test_cases: BTreeMap<_, (Option<_>, Option<_>)> = btreemap!();

                for (name, path) in text_files("in")? {
                    system_test_cases.entry(name).or_default().0 = Some(read(&path)?);
                }
                for (name, path) in text_files("out")? {
                    system_test_cases.entry(name).or_default().1 = Some(read(&path)?);
                }

                Ok(system_test_cases
                    .into_iter()
                    .flat_map(|(name, (r#in, out))| {
                        let r#in = r#in?;
                        Some(PartialBatchTestCase {
                            name: Some(name),
                            r#in,
                            out,
                            timelimit: None,
                            r#match: None,
                        })
                    })
                    .collect())
            },
        )?,
        TestSuite::Interactive(_) => {
            shell.warn("tests for `Interactive` problems are currently not supported")?;
            vec![]
        }
        TestSuite::Unsubmittable => {
            shell.warn("this is `Unsubmittable` problem")?;
            vec![]
        }
    };

    crate::process::process(crate::process::cargo_exe()?)
        .arg("build")
        .arg(if bin.kind == ["example".to_owned()] {
            "--example"
        } else {
            "--bin"
        })
        .arg(&bin.name)
        .args(if release { &["--release"] } else { &[] })
        .arg("--manifest-path")
        .arg(&member.manifest_path)
        .cwd(&metadata.workspace_root)
        .exec_with_shell_status(shell)?;

    let artifact = metadata
        .target_directory
        .join(if release { "release" } else { "debug" })
        .join(if bin.kind == ["example".to_owned()] {
            "examples"
        } else {
            ""
        })
        .join(&bin.name)
        .with_extension(env::consts::EXE_EXTENSION);

    ensure!(
        artifact.exists(),
        "`cargo build` succeeded but `{}` was not produced. probably this is a bug",
        artifact,
    );

    let outcome = snowchains_core::judge::judge(
        shell.progress_draw_target(),
        tokio::signal::ctrl_c,
        &CommandExpression {
            program: artifact.into(),
            args: vec![],
            cwd: metadata.workspace_root.clone().into(),
            env: btreemap!(),
        },
        &test_cases,
    )?;

    let display_limit = display_limit.into::<Byte>().value().saturating_as();

    writeln!(shell.err())?;
    outcome.print_pretty(shell.err(), Some(display_limit))?;
    outcome.error_on_fail()
}

pub(crate) fn test_suite_path(
    workspace_root: &Utf8Path,
    pkg_manifest_dir: &Utf8Path,
    cargo_compete_config_test_suite: &liquid::Template,
    bin_name: &str,
    bin_alias: &str,
    problem_url: &Url,
    shell: &mut Shell,
) -> anyhow::Result<Utf8PathBuf> {
    let contest = match PlatformKind::from_url(problem_url) {
        Ok(PlatformKind::Atcoder) => Some(snowchains_core::web::atcoder_contest_id(problem_url)?),
        Ok(PlatformKind::Codeforces) => {
            Some(snowchains_core::web::codeforces_contest_id(problem_url)?.to_string())
        }
        _ => None,
    };

    let vars = object!({
        "manifest_dir": pkg_manifest_dir,
        "contest": contest,
        "bin_name": bin_name,
        "bin_alias": bin_alias,
    });

    let vars_including_deprecated = object!({
        "manifest_dir": pkg_manifest_dir,
        "contest": contest,
        "bin_name": bin_name,
        "bin_alias": bin_alias,
        "problem": bin_alias,
    });

    let (test_suite_path, uses_deprecated_vars) = cargo_compete_config_test_suite
        .render(&vars)
        .map(|r| (r, false))
        .or_else(|_| {
            cargo_compete_config_test_suite
                .render(&vars_including_deprecated)
                .map(|r| (r, true))
        })?;
    let test_suite_path = Utf8Path::new(&test_suite_path);
    let test_suite_path = test_suite_path.strip_prefix(".").unwrap_or(test_suite_path);

    if uses_deprecated_vars {
        shell.warn("deprecated variables used for `.test-suite` in compete.toml")?;
        shell.warn("- `problem` is deprecated. use `bin_alias` instead.")?;
    }

    Ok(workspace_root.join(test_suite_path))
}
