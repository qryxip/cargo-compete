use crate::{
    config::CargoCompeteConfigSubmitTranspile,
    oj_api,
    project::{MetadataExt as _, PackageExt as _},
    shell::{ColorChoice, Shell},
    web::credentials,
};
use anyhow::{bail, Context as _};
use human_size::Size;
use liquid::object;
use prettytable::{
    cell,
    format::{FormatBuilder, LinePosition, LineSeparator},
    row, Row, Table,
};
use snowchains_core::web::{
    Atcoder, AtcoderSubmitCredentials, AtcoderWatchSubmissionsCredentials,
    AtcoderWatchSubmissionsTarget, Codeforces, CodeforcesSubmitCredentials, CookieStorage,
    PlatformKind, ProblemInContest, Submit, WatchSubmissions, Yukicoder,
    YukicoderSubmitCredentials, YukicoderSubmitTarget,
};
use std::{borrow::BorrowMut as _, cell::RefCell, env, io, iter, path::PathBuf};
use structopt::StructOpt;
use strum::VariantNames as _;

static ATCODER_RUST_LANG_ID: &str = "4050";
static CODEFORCES_RUST_LANG_ID: &str = "49";
static YUKICODER_RUST_LANG_ID: &str = "rust";

#[derive(StructOpt, Debug)]
#[structopt(usage(
    r"cargo compete submit [OPTIONS] <bin-name-or-alias>
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
        required_unless("name-or-alias"),
        conflicts_with("name-or-alias")
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

    /// When testing, build in debug mode. Overrides `test.profile` in compete.toml
    #[structopt(long, conflicts_with("release"))]
    pub debug: bool,

    /// When testing, build in release mode. Overrides `test.profile` in compete.toml
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
    /// Name or alias for a `bin`/`example`
    pub name_or_alias: Option<String>,
}

pub(crate) fn run(opt: OptCompeteSubmit, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteSubmit {
        no_test,
        no_watch,
        src,
        testcases,
        display_limit,
        package,
        debug,
        release,
        manifest_path,
        color,
        name_or_alias,
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
    let package_metadata = member.read_package_metadata(shell)?;
    let (cargo_compete_config, _) = crate::config::load_for_package(&member, shell)?;

    let (bin, package_metadata_bin) = if let Some(src) = src {
        let src = cwd.join(src.strip_prefix(".").unwrap_or(&src));
        let bin = member.bin_target_by_src_path(src)?;
        let (_, pkg_md_bin) = package_metadata.bin_like_by_name_or_alias(&bin.name)?;
        (bin, pkg_md_bin)
    } else if let Some(name_or_alias) = &name_or_alias {
        let (bin_name, pkg_md_bin) = package_metadata.bin_like_by_name_or_alias(name_or_alias)?;
        let bin = member.bin_like_target_by_name(bin_name)?;
        (bin, pkg_md_bin)
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
            .args(if debug {
                &["--debug"]
            } else if release {
                &["--release"]
            } else {
                &[]
            })
            .args(&["--manifest-path".as_ref(), member.manifest_path.as_os_str()])
            .args(&["--color", &color.to_string()])
            .cwd(&metadata.workspace_root)
            .exec_with_shell_status(shell)?;
    }

    let language_id = match &cargo_compete_config.submit.transpile {
        Some(CargoCompeteConfigSubmitTranspile::Command {
            language_id: Some(language_id),
            ..
        }) => Some(&**language_id),
        _ => None,
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

    let source_code_len = code.len();

    if let Ok(platform) = PlatformKind::from_url(&package_metadata_bin.problem) {
        let language_id = language_id.unwrap_or(match platform {
            PlatformKind::Atcoder => ATCODER_RUST_LANG_ID,
            PlatformKind::Codeforces => CODEFORCES_RUST_LANG_ID,
            PlatformKind::Yukicoder => YUKICODER_RUST_LANG_ID,
        });

        let cookie_storage = CookieStorage::with_jsonl(&cookies_path)?;
        let timeout = crate::web::TIMEOUT;

        let outcome = match platform {
            PlatformKind::Atcoder => {
                let shell = RefCell::new(shell.borrow_mut());

                let credentials = AtcoderSubmitCredentials {
                    username_and_password: &mut credentials::username_and_password(
                        &shell,
                        "Username: ",
                        "Password: ",
                    ),
                };

                Atcoder::exec(Submit {
                    target: ProblemInContest::Url {
                        url: package_metadata_bin.problem.clone(),
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
            PlatformKind::Codeforces => {
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
                    target: ProblemInContest::Url {
                        url: package_metadata_bin.problem.clone(),
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
            PlatformKind::Yukicoder => {
                let credentials = YukicoderSubmitCredentials {
                    api_key: credentials::yukicoder_api_key(shell)?,
                };

                Yukicoder::exec(Submit {
                    target: YukicoderSubmitTarget::Url(package_metadata_bin.problem.clone()),
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

        print_status(
            shell,
            &[
                row!["Method", "cargo-compete"],
                row!["Language ID", language_id],
                row!["Size", source_code_len],
                row!["URL (submissions)", outcome.submissions_url],
                row!["URL (detail)", outcome.submission_url],
            ],
        )?;

        if !no_watch {
            let cookie_storage = CookieStorage::with_jsonl(cookies_path)?;
            let timeout = crate::web::TIMEOUT;

            match platform {
                PlatformKind::Atcoder => {
                    let contest =
                        snowchains_core::web::atcoder_contest_id(&package_metadata_bin.problem)?;

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
                PlatformKind::Codeforces => {
                    shell.warn("watching submissions for Codeforces is not implemented")?;
                }
                PlatformKind::Yukicoder => {
                    shell.warn("watching submissions for yukicoder is not implemented")?;
                }
            }
        }
    } else {
        let tempdir = tempfile::Builder::new()
            .prefix("cargo-compete-submit-code-with-oj-api-")
            .tempdir()?;

        let (source_code_path, language_id) = if let Some(language_id) = language_id {
            let source_code_path = tempdir.path().join("main");
            crate::fs::write(&source_code_path, &code)?;
            (source_code_path, language_id.to_owned())
        } else {
            let source_code_path = tempdir.path().join("main.rs");
            crate::fs::write(&source_code_path, &code)?;
            let language_id = oj_api::guess_language_id(
                &package_metadata_bin.problem,
                &source_code_path,
                &metadata.workspace_root,
                shell,
            )?;
            (source_code_path, language_id)
        };

        let url = oj_api::submit_code(
            &package_metadata_bin.problem,
            &source_code_path,
            &language_id,
            &metadata.workspace_root,
            shell,
        )?;

        print_status(
            shell,
            &[
                row!["Method", "oj-api"],
                row!["Language ID", language_id],
                row!["Size", source_code_len],
                row!["URL (detail)", url],
            ],
        )?;

        tempdir.close()?;
    }
    Ok(())
}

fn print_status(shell: &mut Shell, rows: &[Row]) -> io::Result<()> {
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
    table.extend(rows.iter().cloned());
    write!(shell.err(), "{}", table)?;
    shell.err().flush()?;
    shell.status("Successfully", "submitted the code")
}
