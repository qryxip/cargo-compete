use crate::{shell::ColorChoice, web::credentials};
use snowchains_core::web::{
    Atcoder, AtcoderWatchSubmissionsCredentials, AtcoderWatchSubmissionsTarget, CookieStorage,
    PlatformKind, WatchSubmissions,
};
use std::cell::RefCell;
use structopt::StructOpt;
use strum::VariantNames as _;

#[derive(StructOpt, Debug)]
pub struct OptCompeteWatchSubmissions {
    /// Coloring
    #[structopt(
        long,
        value_name("WHEN"),
        possible_values(ColorChoice::VARIANTS),
        default_value("auto")
    )]
    pub color: ColorChoice,

    /// Platform
    #[structopt(possible_value("atcoder"))]
    pub platform: PlatformKind,

    /// Contest ID
    pub contest: String,
}

pub(crate) fn run(opt: OptCompeteWatchSubmissions, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteWatchSubmissions {
        color,
        platform,
        contest,
    } = opt;

    let crate::Context { cwd: _, shell } = ctx;

    shell.set_color_choice(color);

    let cookie_storage = CookieStorage::with_jsonl(credentials::cookies_path()?)?;
    let timeout = crate::web::TIMEOUT;

    if platform == PlatformKind::Atcoder {
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
        })
    } else {
        unreachable!()
    }
}
