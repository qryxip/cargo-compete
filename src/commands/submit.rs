use crate::{
    config::{CargoCompeteConfigSubmitTranspile, CargoCompeteConfigSubmitViaBinary},
    project::{MetadataExt as _, PackageExt as _, TargetProblem, TargetProblemYukicoder},
    shell::ColorChoice,
    web::credentials,
};
use anyhow::{bail, Context as _};
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
use std::{borrow::BorrowMut as _, cell::RefCell, env, iter, path::PathBuf};
use structopt::StructOpt;
use strum::VariantNames as _;

static ATCODER_RUST_LANG_ID: &str = "4050";
static CODEFORCES_RUST_LANG_ID: &str = "49";
static YUKICODER_RUST_LANG_ID: &str = "rust";

#[derive(StructOpt, Debug)]
#[structopt(usage(
    r"cargo compete submit [OPTIONS] <index>
    cargo compete submit [OPTIONS] --src <PATH>",
))]
pub struct OptCompeteSubmit {
    /// Do not test before submitting
    #[structopt(long)]
    pub no_test: bool,

    /// Do not watch the submission
    #[structopt(long)]
    pub no_watch: bool,

    /// Path to the source code
    #[structopt(
        long,
        value_name("PATH"),
        required_unless("index"),
        conflicts_with("index")
    )]
    pub src: Option<PathBuf>,

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

    #[structopt(required_unless("src"))]
    /// Index for `package.metadata.cargo-compete.bin`
    pub index: Option<String>,
}

pub(crate) fn run(opt: OptCompeteSubmit, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteSubmit {
        no_test,
        no_watch,
        src,
        testcases,
        display_limit,
        package,
        release,
        manifest_path,
        color,
        index,
    } = opt;

    let crate::Context {
        cwd,
        cookies_path,
        shell,
    } = ctx;

    shell.set_color_choice(color);

    let manifest_path = manifest_path
        .map(|p| Ok(cwd.join(p.strip_prefix(".").unwrap_or(&p))))
        .unwrap_or_else(|| crate::project::locate_project(&cwd))?;
    let metadata = crate::project::cargo_metadata(&manifest_path, &cwd)?;
    let member = metadata.query_for_member(package.as_deref())?;
    let package_metadata = member.read_package_metadata()?;
    let cargo_compete_config =
        crate::config::load_from_rel_path(&member.manifest_path, &package_metadata.config)?;

    let (bin, package_metadata_bin) = if let Some(src) = src {
        let src = cwd.join(src.strip_prefix(".").unwrap_or(&src));
        let bin = member.bin_target_by_src_path(src)?;
        let package_metadata_bin = package_metadata.bin_by_bin_name(&bin.name)?;
        (bin, package_metadata_bin)
    } else if let Some(index) = index {
        let package_metadata_bin = package_metadata.bin_by_bin_index(index)?;
        let bin = member.bin_target_by_name(&package_metadata_bin.name)?;
        (bin, package_metadata_bin)
    } else {
        unreachable!()
    };

    if !no_test {
        crate::process::process(env::current_exe()?)
            .args(&["compete", "t", "--src"])
            .arg(&bin.src_path)
            .args(&if let Some(testcases) = testcases {
                iter::once("--testcases".into()).chain(testcases).collect()
            } else {
                vec![]
            })
            .args(&["--display-limit", &display_limit.to_string()])
            .args(if release { &["--release"] } else { &[] })
            .args(&["--manifest-path".as_ref(), member.manifest_path.as_os_str()])
            .args(&["--color", &color.to_string()])
            .cwd(&metadata.workspace_root)
            .exec_with_shell_status(shell)?;
    }

    let language_id = match &cargo_compete_config.submit.transpile {
        Some(CargoCompeteConfigSubmitTranspile::Command {
            language_id: Some(language_id),
            ..
        }) => language_id,
        _ => match package_metadata_bin.problem {
            TargetProblem::Atcoder { .. } => ATCODER_RUST_LANG_ID,
            TargetProblem::Codeforces { .. } => CODEFORCES_RUST_LANG_ID,
            TargetProblem::Yukicoder(_) => YUKICODER_RUST_LANG_ID,
        },
    };

    let mut code = crate::fs::read_to_string(&bin.src_path)?;

    if let Some(CargoCompeteConfigSubmitTranspile::Command { args, .. }) =
        &cargo_compete_config.submit.transpile
    {
        code = {
            if args.is_empty() {
                bail!("`submit.transpile.args` is empty");
            }

            let vars = object!({ "bin_name": &bin.name });

            let args = args
                .iter()
                .map(|t| t.render(&vars))
                .collect::<Result<Vec<_>, _>>()?;

            crate::process::with_which(&args[0], &metadata.workspace_root)?
                .args(&args[1..])
                .read_with_shell_status(shell)
                .with_context(|| "could not transpile the code")?
        };
    }

    if let Some(CargoCompeteConfigSubmitViaBinary {
        target,
        cross,
        strip,
        upx,
    }) = &cargo_compete_config.submit.via_bianry
    {
        code = {
            let original_source_code = code;

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
        };
    };

    let source_code_len = code.len();

    let cookie_storage = CookieStorage::with_jsonl(&cookies_path)?;
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
                language_id: language_id.to_owned(),
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
                language_id: language_id.to_owned(),
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
                language_id: language_id.to_owned(),
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
        let cookie_storage = CookieStorage::with_jsonl(cookies_path)?;
        let timeout = crate::web::TIMEOUT;

        match &package_metadata_bin.problem {
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
                    target: AtcoderWatchSubmissionsTarget {
                        contest: contest.clone(),
                    },
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
