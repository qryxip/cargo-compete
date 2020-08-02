use crate::{
    shell::{ColorChoice, Shell},
    web::credentials,
};
use snowchains_core::web::{
    Atcoder, AtcoderParticipateCredentials, AtcoderParticipateTarget, Codeforces,
    CodeforcesParticipateCredentials, CodeforcesParticipateTarget, CookieStorage, Participate,
    ParticipateOutcome, PlatformKind,
};
use std::{borrow::BorrowMut as _, cell::RefCell, io};
use structopt::StructOpt;
use strum::VariantNames as _;

#[derive(StructOpt, Debug)]
pub struct OptCompeteParticipate {
    /// Coloring
    #[structopt(
        long,
        value_name("WHEN"),
        possible_values(ColorChoice::VARIANTS),
        default_value("auto")
    )]
    pub color: ColorChoice,

    /// Platform
    #[structopt(possible_values(&["atcoder", "codeforces"]))]
    pub platform: PlatformKind,

    /// Contest ID
    pub contest: String,
}

pub(crate) fn run(opt: OptCompeteParticipate, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteParticipate {
        color,
        platform,
        contest,
    } = opt;

    let crate::Context { cwd: _, shell } = ctx;

    shell.set_color_choice(color);

    let cookie_storage = CookieStorage::with_jsonl(credentials::cookies_path()?)?;
    let timeout = crate::web::TIMEOUT;

    match platform {
        PlatformKind::Atcoder => {
            let outcome = {
                let shell = RefCell::new(shell.borrow_mut());

                let credentials = AtcoderParticipateCredentials {
                    username_and_password: &mut credentials::username_and_password(
                        &shell,
                        "Username: ",
                        "Password: ",
                    ),
                };

                Atcoder::exec(Participate {
                    target: AtcoderParticipateTarget { contest },
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

                let credentials = CodeforcesParticipateCredentials {
                    username_and_password: &mut credentials::username_and_password(
                        &shell,
                        "Handle/Email: ",
                        "Password: ",
                    ),
                };

                Codeforces::exec(Participate {
                    target: CodeforcesParticipateTarget { contest },
                    credentials,
                    cookie_storage,
                    timeout,
                    shell: &shell,
                })?
            };

            status(shell, outcome)?;
        }
        PlatformKind::Yukicoder => unreachable!(),
    }

    Ok(())
}

fn status(shell: &mut Shell, outcome: ParticipateOutcome) -> io::Result<()> {
    match outcome {
        ParticipateOutcome::Success => shell.status("Successfully", "participated"),
        ParticipateOutcome::AlreadyParticipated => shell.warn("already participated"),
        ParticipateOutcome::ContestIsFinished => shell.warn("the contest is finished"),
    }
}
