use crate::{
    shell::ColorChoice,
    web::{ATCODER_RUST_LANG_ID, CODEFORCES_RUST_LANG_ID, YUKICODER_RUST_LANG_ID},
};
use snowchains_core::web::PlatformKind;
use std::path::PathBuf;
use structopt::StructOpt;
use strum::VariantNames as _;

static TEMPLATE_CARGO_LOCK: &str = "./template-cargo-lock.toml";

static ATCODER_RUST_EDITION: &str = "2021";
static CODEFORCES_RUST_EDITION: &str = "2021";
static YUKICODER_RUST_EDITION: &str = "2018";

static ATCODER_RUST_VERSION: &str = "1.70.0";
static CODEFORCES_RUST_VERSION: &str = "1.57.0";
static YUKICODER_RUST_VERSION: &str = "1.53.0";

#[derive(StructOpt, Debug)]
pub struct OptCompeteInit {
    /// Coloring
    #[structopt(
        long,
        value_name("WHEN"),
        possible_values(ColorChoice::VARIANTS),
        default_value("auto")
    )]
    color: ColorChoice,

    /// Platform
    #[structopt(possible_values(PlatformKind::KEBAB_CASE_VARIANTS))]
    platform: PlatformKind,

    /// Path to create files
    #[structopt(default_value("."))]
    path: PathBuf,
}

pub(crate) fn run(opt: OptCompeteInit, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteInit {
        color,
        platform,
        path,
    } = opt;

    let crate::Context {
        cwd,
        cookies_path: _,
        shell,
    } = ctx;

    shell.set_color_choice(color);

    let path = cwd.join(path);

    let atcoder_crates = if platform == PlatformKind::Atcoder {
        writeln!(shell.err(), "Do you use crates on AtCoder?")?;
        writeln!(shell.err(), "1 No")?;
        writeln!(shell.err(), "2 Yes")?;
        writeln!(shell.err(), "3 Yes, but I submit base64-encoded programs")?;

        loop {
            match shell.read_reply("1..3: ")?.trim() {
                "1" => break AtcoderCrates::None,
                "2" => break AtcoderCrates::UseNormally,
                "3" => break AtcoderCrates::UseViaBinary,
                _ => writeln!(shell.err(), "Choose 1, 2, or 3.")?,
            }
        }
    } else {
        AtcoderCrates::None
    };

    crate::fs::create_dir_all(&path)?;

    let mut write_with_status = |file_name: &str, content: &str| -> anyhow::Result<()> {
        let path = path.join(file_name);
        crate::fs::write(&path, content)?;
        shell.status("Wrote", path.display())?;
        Ok(())
    };

    write_with_status(
        "compete.toml",
        &crate::config::generate(
            match platform {
                PlatformKind::Atcoder => ATCODER_RUST_EDITION,
                PlatformKind::Codeforces => CODEFORCES_RUST_EDITION,
                PlatformKind::Yukicoder => YUKICODER_RUST_EDITION,
            },
            (atcoder_crates == AtcoderCrates::UseNormally)
                .then_some(include_str!("../../resources/atcoder-deps.toml")),
            (atcoder_crates == AtcoderCrates::UseNormally).then_some(TEMPLATE_CARGO_LOCK),
            platform,
            match platform {
                PlatformKind::Atcoder => ATCODER_RUST_VERSION,
                PlatformKind::Codeforces => CODEFORCES_RUST_VERSION,
                PlatformKind::Yukicoder => YUKICODER_RUST_VERSION,
            },
            atcoder_crates == AtcoderCrates::UseViaBinary,
            match platform {
                PlatformKind::Atcoder => ATCODER_RUST_LANG_ID,
                PlatformKind::Codeforces => CODEFORCES_RUST_LANG_ID,
                PlatformKind::Yukicoder => YUKICODER_RUST_LANG_ID,
            },
        )?,
    )?;

    if atcoder_crates == AtcoderCrates::UseNormally {
        write_with_status(
            TEMPLATE_CARGO_LOCK.strip_prefix("./").unwrap(),
            include_str!("../../resources/atcoder-cargo-lock.toml"),
        )?;
    }

    crate::project::set_cargo_config_build_target_dir(&path, shell)?;
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum AtcoderCrates {
    None,
    UseNormally,
    UseViaBinary,
}
