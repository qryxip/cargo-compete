use crate::{
    project::{
        PackageExt as _, PackageMetadataCargoCompeteBin, TargetProblem, TargetProblemYukicoder,
    },
    shell::Shell,
};
use anyhow::ensure;
use az::SaturatingAs as _;
use cargo_metadata::{Metadata, Package};
use human_size::{Byte, Size};
use liquid::object;
use maplit::btreemap;
use snowchains_core::{judge::CommandExpression, testsuite::TestSuite};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

pub(crate) struct Args<'a> {
    pub(crate) metadata: &'a Metadata,
    pub(crate) member: &'a Package,
    pub(crate) cargo_compete_config_test_suite: &'a liquid::Template,
    pub(crate) package_metadata_bin: &'a PackageMetadataCargoCompeteBin,
    pub(crate) release: bool,
    pub(crate) test_case_names: Option<HashSet<String>>,
    pub(crate) display_limit: Size,
    pub(crate) shell: &'a mut Shell,
}

pub(crate) fn test(args: Args<'_>) -> anyhow::Result<()> {
    let Args {
        metadata,
        member,
        cargo_compete_config_test_suite,
        package_metadata_bin,
        release,
        test_case_names,
        display_limit,
        shell,
    } = args;

    let bin = member.bin_target(&package_metadata_bin.name)?;

    let test_suite_path = test_suite_path(
        &metadata.workspace_root,
        member.manifest_dir_utf8(),
        cargo_compete_config_test_suite,
        &package_metadata_bin.problem,
    )?;

    let test_suite = crate::fs::read_yaml(&test_suite_path)?;

    let test_cases = match test_suite {
        TestSuite::Batch(test_suite) => {
            test_suite.load_test_cases(test_suite_path.parent().unwrap(), test_case_names)?
        }
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
        .args(&["build", "--bin", &bin.name])
        .args(if release { &["--release"] } else { &[] })
        .arg("--manifest-path")
        .arg(&member.manifest_path)
        .cwd(&metadata.workspace_root)
        .exec_with_shell_status(shell)?;

    let artifact = metadata
        .target_directory
        .join(if release { "release" } else { "debug" })
        .join(&bin.name)
        .with_extension(if cfg!(windows) { "exe" } else { "" });

    ensure!(artifact.exists(), "`{}` does not exist", artifact.display());

    let outcome = snowchains_core::judge::judge(
        shell.progress_draw_target(),
        tokio::signal::ctrl_c,
        &CommandExpression {
            program: artifact.into(),
            args: vec![],
            cwd: metadata.workspace_root.clone(),
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
    workspace_root: &Path,
    pkg_manifest_dir: &str,
    cargo_compete_config_test_suite: &liquid::Template,
    target_problem: &TargetProblem,
) -> anyhow::Result<PathBuf> {
    let (contest, problem) = match target_problem {
        TargetProblem::Atcoder { contest, index, .. }
        | TargetProblem::Codeforces { contest, index, .. }
        | TargetProblem::Yukicoder(TargetProblemYukicoder::Contest { contest, index, .. }) => {
            (&**contest, index.clone())
        }
        TargetProblem::Yukicoder(TargetProblemYukicoder::Problem { no, .. }) => {
            ("problems", no.to_string())
        }
    };

    let vars = object!({
        "manifest_dir": pkg_manifest_dir,
        "contest": contest,
        "problem": problem,
    });

    let test_suite_path = cargo_compete_config_test_suite.render(&vars)?;
    let test_suite_path = Path::new(&test_suite_path);
    let test_suite_path = test_suite_path
        .strip_prefix(".")
        .unwrap_or(&test_suite_path);
    Ok(workspace_root.join(test_suite_path))
}
