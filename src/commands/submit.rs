use crate::{
    project::{
        CargoCompeteConfigSubmitViaBinary, MetadataExt as _, PackageExt as _, TargetProblem,
        TargetProblemYukicoder,
    },
    shell::ColorChoice,
    web::credentials,
};
use anyhow::Context as _;
use human_size::Size;
use liquid::object;
use prettytable::{
    cell,
    format::{FormatBuilder, LinePosition, LineSeparator},
    row, Table,
};
use snowchains_core::web::{
    Atcoder, AtcoderSubmitCredentials, AtcoderSubmitTarget, AtcoderWatchSubmissionsCredentials,
    AtcoderWatchSubmissionsTarget, Codeforces, CodeforcesSubmitCredentials, CodeforcesSubmitTarget,
    CookieStorage, Submit, WatchSubmissions, Yukicoder, YukicoderSubmitCredentials,
    YukicoderSubmitTarget,
};
use std::{borrow::BorrowMut as _, cell::RefCell, path::PathBuf};
use structopt::StructOpt;
use strum::VariantNames as _;

static ATCODER_RUST_LANG_ID: &str = "4050";
static CODEFORCES_RUST_LANG_ID: &str = "49";
static YUKICODER_RUST_LANG_ID: &str = "rust";

#[derive(StructOpt, Debug)]
pub struct OptCompeteSubmit {
    /// Do not test before submitting
    #[structopt(long)]
    pub no_test: bool,

    /// Do not watch the submission
    #[structopt(long)]
    pub no_watch: bool,

    /// Test for only the test cases
    #[structopt(long, value_name("NAME"))]
    pub testcases: Option<Vec<String>>,

    /// Display limit for the test
    #[structopt(long, value_name("SIZE"), default_value("4KiB"))]
    pub display_limit: Size,

    /// Existing package to retrieving test cases for
    #[structopt(short, long, value_name("SPEC"))]
    pub package: Option<String>,

    /// When testing, build the artifact in release mode, with optimizations
    #[structopt(long)]
    pub release: bool,

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

    /// Problem Index
    pub problem: String,
}

pub(crate) fn run(opt: OptCompeteSubmit, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteSubmit {
        no_test,
        no_watch,
        testcases,
        display_limit,
        package,
        release,
        manifest_path,
        color,
        problem,
    } = opt;

    let crate::Context { cwd, shell } = ctx;

    shell.set_color_choice(color);

    let manifest_path = manifest_path
        .map(Ok)
        .unwrap_or_else(|| crate::project::locate_project(cwd))?;
    let metadata = crate::project::cargo_metadata(&manifest_path)?;

    let cargo_compete_config = metadata.read_compete_toml()?;

    let member = metadata.query_for_member(package.as_deref())?;

    let package_metadata_bin = member
        .read_package_metadata()?
        .bin
        .remove(&problem)
        .with_context(|| {
            format!(
                "could not find `{}` in `package.metadata.cargo-compete.bin`",
                problem
            )
        })?;

    if !no_test {
        crate::testing::test(crate::testing::Args {
            metadata: &metadata,
            member,
            cargo_compete_config_test_suite: &cargo_compete_config.test_suite,
            package_metadata_bin: &package_metadata_bin,
            release,
            test_case_names: testcases.map(|ss| ss.into_iter().collect()),
            display_limit,
            shell,
        })?;
    }

    let bin = member.bin_target(&package_metadata_bin.name)?;

    let code = if let Some(CargoCompeteConfigSubmitViaBinary {
        target,
        cross,
        strip,
        upx,
    }) = &cargo_compete_config.submit_via_binary
    {
        let original_source_code = crate::fs::read_to_string(&bin.src_path)?;

        let program = if let Some(cross) = cross {
            cross.clone()
        } else {
            crate::process::cargo_exe()?
        };

        crate::process::with_which(program, &metadata.workspace_root)?
            .args(&[
                "build",
                "--bin",
                &bin.name,
                "--release",
                "--target",
                &target,
            ])
            .cwd(member.manifest_path.parent().unwrap())
            .display_cwd()
            .exec_with_shell_status(shell)?;

        let orig_artifact = metadata
            .target_directory
            .join(&target)
            .join("release")
            .join(&bin.name);

        let artifact = tempfile::Builder::new()
            .prefix("cargo-compete-exec-base64-encoded-binary-")
            .tempfile()?
            .into_temp_path();

        std::fs::copy(orig_artifact, &artifact)?;

        if let Some(strip) = strip {
            crate::process::with_which(strip, &metadata.workspace_root)?
                .arg("-s")
                .arg(&artifact)
                .exec_with_shell_status(shell)?;
        }

        if let Some(upx) = upx {
            crate::process::with_which(upx, &metadata.workspace_root)?
                .arg("--best")
                .arg(&artifact)
                .exec_with_shell_status(shell)?;
        }

        let artifact_binary = crate::fs::read(&artifact)?;

        artifact.close()?;

        liquid::ParserBuilder::with_stdlib()
            .build()?
            .parse(include_str!(
                "../../resources/exec-base64-encoded-binary.rs.liquid"
            ))?
            .render(&object!({
                "source_code": original_source_code,
                "base64": base64::encode(artifact_binary),
            }))?
    } else {
        crate::fs::read_to_string(&bin.src_path)?
    };

    let source_code_len = code.len();

    let language_id = match package_metadata_bin.problem {
        TargetProblem::Atcoder { .. } => ATCODER_RUST_LANG_ID,
        TargetProblem::Codeforces { .. } => CODEFORCES_RUST_LANG_ID,
        TargetProblem::Yukicoder(_) => YUKICODER_RUST_LANG_ID,
    };

    let cookie_storage = CookieStorage::with_jsonl(credentials::cookies_path()?)?;
    let timeout = crate::web::TIMEOUT;

    let outcome = match &package_metadata_bin.problem {
        TargetProblem::Atcoder { contest, index, .. } => {
            let shell = RefCell::new(shell.borrow_mut());

            let credentials = AtcoderSubmitCredentials {
                username_and_password: &mut credentials::username_and_password(
                    &shell,
                    "Username: ",
                    "Password: ",
                ),
            };

            Atcoder::exec(Submit {
                target: AtcoderSubmitTarget {
                    contest: contest.clone(),
                    problem: index.clone(),
                },
                credentials,
                language_id: ATCODER_RUST_LANG_ID.to_owned(),
                code,
                watch_submission: false,
                cookie_storage,
                timeout,
                shell: &shell,
            })?
        }
        TargetProblem::Codeforces { contest, index, .. } => {
            let (api_key, api_secret) = credentials::codeforces_api_key_and_secret(shell)?;

            let shell = RefCell::new(shell.borrow_mut());

            let credentials = CodeforcesSubmitCredentials {
                username_and_password: &mut credentials::username_and_password(
                    &shell,
                    "Username: ",
                    "Password: ",
                ),
                api_key,
                api_secret,
            };

            Codeforces::exec(Submit {
                target: CodeforcesSubmitTarget {
                    contest: contest.clone(),
                    problem: index.clone(),
                },
                credentials,
                language_id: CODEFORCES_RUST_LANG_ID.to_owned(),
                code,
                watch_submission: false,
                cookie_storage,
                timeout,
                shell: &shell,
            })?
        }
        TargetProblem::Yukicoder(target_problem) => {
            let credentials = YukicoderSubmitCredentials {
                api_key: credentials::yukicoder_api_key(shell)?,
            };

            Yukicoder::exec(Submit {
                target: match target_problem {
                    TargetProblemYukicoder::Contest { contest, index, .. } => {
                        YukicoderSubmitTarget::Contest(contest.clone(), index.clone())
                    }
                    TargetProblemYukicoder::Problem { no, .. } => {
                        YukicoderSubmitTarget::ProblemNo(no.to_string())
                    }
                },
                credentials,
                language_id: YUKICODER_RUST_LANG_ID.to_owned(),
                code,
                watch_submission: false,
                cookie_storage: (),
                timeout,
                shell: shell.borrow_mut(),
            })?
        }
    };

    shell.status("Successfully", "submitted the code")?;

    let mut table = Table::new();

    *table.get_format() = FormatBuilder::new()
        .padding(1, 1)
        .column_separator('│')
        .borders('│')
        .separator(LinePosition::Top, LineSeparator::new('─', '┬', '┌', '┐'))
        .separator(LinePosition::Title, LineSeparator::new('─', '┼', '├', '┤'))
        .separator(LinePosition::Intern, LineSeparator::new('─', '┼', '├', '┤'))
        .separator(LinePosition::Bottom, LineSeparator::new('─', '┴', '└', '┘'))
        .build();

    table.add_row(row!["Language ID", language_id]);
    table.add_row(row!["Size", source_code_len]);
    table.add_row(row!["URL (submissions)", outcome.submissions_url]);
    table.add_row(row!["URL (detail)", outcome.submission_url]);

    write!(shell.err(), "{}", table)?;
    shell.err().flush()?;

    if !no_watch {
        let cookie_storage = CookieStorage::with_jsonl(credentials::cookies_path()?)?;
        let timeout = crate::web::TIMEOUT;

        match package_metadata_bin.problem {
            TargetProblem::Atcoder { contest, .. } => {
                let shell = RefCell::new(shell);

                let credentials = AtcoderWatchSubmissionsCredentials {
                    username_and_password: &mut credentials::username_and_password(
                        &shell,
                        "Username: ",
                        "Password: ",
                    ),
                };

                Atcoder::exec(WatchSubmissions {
                    target: AtcoderWatchSubmissionsTarget { contest },
                    credentials,
                    cookie_storage,
                    timeout,
                    shell: &shell,
                })?;
            }
            TargetProblem::Codeforces { .. } => {
                shell.warn("watching submissions for Codeforces is not implemented")?;
            }
            TargetProblem::Yukicoder(_) => {
                shell.warn("watching submissions for yukicoder is not implemented")?;
            }
        }
    }

    Ok(())
}
