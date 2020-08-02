use crate::{
    shell::{ColorChoice, Shell},
    web::credentials,
};
use snowchains_core::web::{
    Atcoder, AtcoderLoginCredentials, Codeforces, CodeforcesLoginCredentials, CookieStorage, Login,
    LoginOutcome, PlatformKind,
};
use std::{borrow::BorrowMut as _, cell::RefCell, io};
use structopt::StructOpt;
use strum::VariantNames as _;
use termcolor::Color;

#[derive(StructOpt, Debug)]
pub struct OptCompeteLogin {
    /// Coloring
    #[structopt(
        long,
        value_name("WHEN"),
        possible_values(ColorChoice::VARIANTS),
        default_value("auto")
    )]
    pub color: ColorChoice,

    /// Platform to login
    #[structopt(possible_values(PlatformKind::KEBAB_CASE_VARIANTS))]
    pub platform: PlatformKind,
}

pub(crate) fn run(opt: OptCompeteLogin, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteLogin { color, platform } = opt;

    let crate::Context { cwd: _, shell } = ctx;

    shell.set_color_choice(color);

    let cookie_storage = CookieStorage::with_jsonl(credentials::cookies_path()?)?;
    let timeout = crate::web::TIMEOUT;

    match platform {
        PlatformKind::Atcoder => {
            let outcome = {
                let shell = RefCell::new(shell.borrow_mut());

                let credentials = AtcoderLoginCredentials {
                    username_and_password: &mut credentials::username_and_password(
                        &shell,
                        "Username: ",
                        "Password: ",
                    ),
                };

                Atcoder::exec(Login {
                    credentials,
                    cookie_storage,
                    timeout,
                    shell: &shell,
                })?
            };

            status(shell, outcome)?;
        }
        PlatformKind::Codeforces => {
            let outcome = {
                let shell = RefCell::new(shell.borrow_mut());

                let credentials = CodeforcesLoginCredentials {
                    username_and_password: &mut credentials::username_and_password(
                        &shell,
                        "Handle/Email: ",
                        "Password: ",
                    ),
                };

                Codeforces::exec(Login {
                    credentials,
                    cookie_storage,
                    timeout,
                    shell: &shell,
                })?
            };

            status(shell, outcome)?;

            let (api_key, api_secret) = credentials::codeforces_api_key_and_secret(shell)?;

            writeln!(
                shell.err(),
                "API key: {}\nAPI secret: {}",
                api_key.chars().map(|_| '*').collect::<String>(),
                api_secret.chars().map(|_| '*').collect::<String>(),
            )?;
            shell.err().flush()?;
        }
        PlatformKind::Yukicoder => {
            let api_key = credentials::yukicoder_api_key(shell)?;

            writeln!(
                shell.err(),
                "API key: {}",
                api_key.chars().map(|_| '*').collect::<String>(),
            )?;
            shell.err().flush()?;
        }
    }

    Ok(())
}

fn status(shell: &mut Shell, outcome: LoginOutcome) -> io::Result<()> {
    let (status, message, color) = match outcome {
        LoginOutcome::Success => ("Successfully", "Logged in", Color::Green),
        LoginOutcome::AlreadyLoggedIn => ("Already", "Logged in", Color::Yellow),
    };

    shell.status_with_color(status, message, color)
}
