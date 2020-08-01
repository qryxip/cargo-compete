use crate::{
    project::{
        PackageExt as _, PackageMetadataCargoCompeteBin, TargetProblem, TargetProblemYukicoder,
        WorkspaceMetadataCargoCompete, WorkspaceMetadataCargoCompetePlatform,
        WorkspaceMetadataCargoCompetePlatformViaBinary,
    },
    shell::Shell,
};
use anyhow::Context as _;
use az::SaturatingAs as _;
use cargo_metadata::{Metadata, Package};
use human_size::{Byte, Size};
use maplit::btreemap;
use snowchains_core::{judge::CommandExpression, testsuite::TestSuite};
use std::{env, path::Path};

pub(crate) struct Args<'a> {
    pub(crate) metadata: &'a Metadata,
    pub(crate) member: &'a Package,
    pub(crate) workspace_metadata: &'a WorkspaceMetadataCargoCompete,
    pub(crate) package_metadata_bin: &'a PackageMetadataCargoCompeteBin,
    pub(crate) release: bool,
    pub(crate) display_limit: Size,
    pub(crate) shell: &'a mut Shell,
}

pub(crate) fn test(args: Args<'_>) -> anyhow::Result<()> {
    let Args {
        metadata,
        member,
        workspace_metadata,
        package_metadata_bin,
        release,
        display_limit,
        shell,
    } = args;

    let bin = member.bin_target(&package_metadata_bin.name)?;

    let test_suite_path = match &package_metadata_bin.problem {
        TargetProblem::Atcoder { contest, index }
        | TargetProblem::Codeforces { contest, index }
        | TargetProblem::Yukicoder(TargetProblemYukicoder::Contest { contest, index }) => {
            let test_suite_path = workspace_metadata
                .test_suite
                .eval(&btreemap!("contest" => &**contest, "problem" => &index))?;
            let test_suite_path = Path::new(&test_suite_path);
            let test_suite_path = test_suite_path
                .strip_prefix(".")
                .unwrap_or(&test_suite_path);
            metadata.workspace_root.join(test_suite_path)
        }
        TargetProblem::Yukicoder(TargetProblemYukicoder::Problem { no }) => {
            let no = no.to_string();
            let test_suite_path = workspace_metadata
                .test_suite
                .eval(&btreemap!("contest" => "problems", "problem" => &no))?;
            let test_suite_path = Path::new(&test_suite_path);
            let test_suite_path = test_suite_path
                .strip_prefix(".")
                .unwrap_or(&test_suite_path);
            metadata.workspace_root.join(test_suite_path)
        }
    };

    let test_suite = crate::fs::read_yaml(&test_suite_path)?;

    let test_cases = match test_suite {
        TestSuite::Batch(test_suite) => {
            test_suite.load_test_cases(test_suite_path.parent().unwrap())?
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

    let cargo_exe = env::var_os("CARGO").with_context(|| "`$CARGO` should be present")?;

    let (build_program, target_arg, build_artifact) =
        if let WorkspaceMetadataCargoCompetePlatform::Atcoder {
            via_binary:
                Some(WorkspaceMetadataCargoCompetePlatformViaBinary {
                    target, use_cross, ..
                }),
        } = &workspace_metadata.platform
        {
            (
                if *use_cross {
                    "cross".into()
                } else {
                    cargo_exe
                },
                vec!["--target", target],
                metadata
                    .target_directory
                    .join(target)
                    .join(if release { "release" } else { "debug" })
                    .join(&bin.name)
                    .with_extension(if cfg!(windows) { "exe" } else { "" }),
            )
        } else {
            (
                cargo_exe,
                vec![],
                metadata
                    .target_directory
                    .join(if release { "release" } else { "debug" })
                    .join(&bin.name)
                    .with_extension(if cfg!(windows) { "exe" } else { "" }),
            )
        };

    let cwd = member.manifest_path.parent().unwrap();

    crate::process::process(build_program, cwd)
        .args(&["build", "--bin"])
        .arg(&bin.name)
        .args(if release { &["--release"] } else { &[] })
        .args(&target_arg)
        .exec_with_shell_status(shell)?;

    let outcome = snowchains_core::judge::judge(
        shell.progress_draw_target(),
        tokio::signal::ctrl_c,
        &CommandExpression {
            program: build_artifact.into(),
            args: vec![],
            cwd: cwd.to_owned(),
            env: btreemap!(),
        },
        &test_cases,
    )?;

    let display_limit = display_limit.into::<Byte>().value().saturating_as();

    writeln!(shell.err())?;
    outcome.print_pretty(shell.err(), Some(display_limit))?;
    outcome.error_on_fail()
}
