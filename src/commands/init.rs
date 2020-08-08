use crate::{
    shell::{ColorChoice, Shell},
    ATCODER_RUST_VERSION, CODEFORCES_RUST_VERSION, YUKICODER_RUST_VERSION,
};
use anyhow::{bail, Context as _};
use cargo_metadata::MetadataCommand;
use snowchains_core::web::PlatformKind;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use strum::VariantNames as _;

#[derive(StructOpt, Debug)]
pub struct OptCompeteInit {
    /// Coloring
    #[structopt(
        long,
        value_name("WHEN"),
        possible_values(ColorChoice::VARIANTS),
        default_value("auto")
    )]
    pub color: ColorChoice,

    /// Path to create workspaces. Defaults to the Git repository root
    pub path: Option<PathBuf>,
}

pub(crate) fn run(opt: OptCompeteInit, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteInit { color, path } = opt;

    let crate::Context { cwd, shell } = ctx;

    shell.set_color_choice(color);

    let path = if let Some(path) = path {
        cwd.join(path.strip_prefix(".").unwrap_or(&path))
    } else {
        cwd.ancestors()
            .find(|p| p.join(".git").is_dir())
            .with_context(|| {
                "not a Git repository. run `git init` first, or specify a path with CLI arguments"
            })?
            .to_owned()
    };

    if let Some(path) = ["atcoder", "codeforces", "yukicoder"]
        .iter()
        .map(|p| path.join(p))
        .find(|p| p.exists())
    {
        bail!("`{}` already exists. aborting.", path.display());
    }

    writeln!(shell.err(), "Websites you compete in:")?;
    writeln!(shell.err(), "1 AtCoder")?;
    writeln!(shell.err(), "2 Codeforces")?;
    writeln!(shell.err(), "3 yukicoder")?;

    let platforms = loop {
        let platforms = shell
            .read_reply("Space-delimited numbers (defaults to all): ")?
            .split_whitespace()
            .map(|s| match s {
                "1" => Some(PlatformKind::Atcoder),
                "2" => Some(PlatformKind::Codeforces),
                "3" => Some(PlatformKind::Yukicoder),
                _ => None,
            })
            .collect::<Option<HashSet<_>>>();

        if let Some(platforms) = platforms {
            break platforms;
        }

        writeln!(shell.err(), "invalid number(s)")?;
    };

    let mut atcoder_crates = AtcoderCrates::None;

    if platforms.is_empty() || platforms.contains(&PlatformKind::Atcoder) {
        writeln!(shell.err(), "Do you use crates on AtCoder?")?;
        writeln!(shell.err(), "1 No")?;
        writeln!(shell.err(), "2 Yes")?;
        writeln!(shell.err(), "3 Yes, but I submit base64-encoded programs")?;

        atcoder_crates = loop {
            match shell.read_reply("Number: ")?.trim() {
                "1" => break AtcoderCrates::None,
                "2" => break AtcoderCrates::UseNormally,
                "3" => break AtcoderCrates::UseViaBinary,
                _ => writeln!(shell.err(), "Choose 1, 2, or 3.")?,
            }
        }
    }

    if platforms.is_empty() || platforms.contains(&PlatformKind::Atcoder) {
        let root_manifest_dir = path.join("atcoder");
        let root_manifest_path = path.join("atcoder").join("Cargo.toml");

        crate::fs::create_dir_all(&root_manifest_dir)?;
        let root_manifest = if atcoder_crates == AtcoderCrates::UseViaBinary {
            r#"[workspace]
members = ["cargo-compete-template"]
exclude = []

[profile.release]
lto = true
panic = "abort"
"#
        } else {
            r#"[workspace]
members = ["cargo-compete-template"]
exclude = []
"#
        };
        crate::fs::write(&root_manifest_path, root_manifest)?;
        shell.status("Wrote", root_manifest_path.display())?;

        if atcoder_crates != AtcoderCrates::UseViaBinary {
            let rust_toolchain_path = root_manifest_dir.join("rust-toolchain");
            let rust_toolchain = format!("{}\n", ATCODER_RUST_VERSION);
            crate::fs::write(&rust_toolchain_path, rust_toolchain)?;
            shell.status("Wrote", rust_toolchain_path.display())?;
        }

        if atcoder_crates == AtcoderCrates::UseNormally {
            let lock_path = root_manifest_dir.join("Cargo.lock");
            crate::fs::write(
                &lock_path,
                include_str!("../../resources/atcoder-cargo-lock.toml"),
            )?;
            shell.status("Wrote", lock_path.display())?;
        }

        let dependencies = match atcoder_crates {
            AtcoderCrates::None => None,
            AtcoderCrates::UseNormally => Some(include_str!("../../resources/atcoder-deps.toml")),
            AtcoderCrates::UseViaBinary => Some(
                r#"proconio = { version = "0.4.1", features = ["derive"] }
"#,
            ),
        };

        write_compete_toml(
            &root_manifest_dir.join("compete.toml"),
            PlatformKind::Atcoder,
            atcoder_crates,
            shell,
        )?;

        new_template_package(
            &root_manifest_dir.join("cargo-compete-template"),
            dependencies,
            if atcoder_crates == AtcoderCrates::None {
                include_str!("../../resources/template-main.rs")
            } else {
                include_str!("../../resources/atcoder-template-main.rs")
            },
            shell,
        )?;
    }

    for &platform in &[PlatformKind::Codeforces, PlatformKind::Yukicoder] {
        if platforms.is_empty() || platforms.contains(&platform) {
            let root_manifest_dir = path.join(platform.to_kebab_case_str());
            let root_manifest_path = root_manifest_dir.join("Cargo.toml");

            crate::fs::create_dir_all(&root_manifest_dir)?;

            crate::fs::write(
                &root_manifest_path,
                r#"[workspace]
members = ["cargo-compete-template"]
exclude = []
"#,
            )?;
            shell.status("Wrote", root_manifest_path.display())?;

            let toolchain = match platform {
                PlatformKind::Atcoder => unreachable!(),
                PlatformKind::Codeforces => &CODEFORCES_RUST_VERSION,
                PlatformKind::Yukicoder => &YUKICODER_RUST_VERSION,
            };

            let rust_toolchain_path = root_manifest_dir.join("rust-toolchain");
            let rust_toolchain = format!("{}\n", toolchain);
            crate::fs::write(&rust_toolchain_path, rust_toolchain)?;
            shell.status("Wrote", rust_toolchain_path.display())?;

            write_compete_toml(
                &root_manifest_dir.join("compete.toml"),
                platform,
                AtcoderCrates::None,
                shell,
            )?;

            new_template_package(
                &root_manifest_dir.join("cargo-compete-template"),
                None,
                include_str!("../../resources/template-main.rs"),
                shell,
            )?;
        }
    }

    Ok(())
}

fn write_compete_toml(
    path: &Path,
    platform: PlatformKind,
    atcoder_crates: AtcoderCrates,
    shell: &mut Shell,
) -> anyhow::Result<()> {
    let content =
        crate::project::gen_compete_toml(platform, atcoder_crates == AtcoderCrates::UseViaBinary)?;
    crate::fs::write(path, content)?;
    shell.status("Wrote", path.display())?;
    Ok(())
}

fn new_template_package(
    path: &Path,
    deps: Option<&str>,
    main_rs: &str,
    shell: &mut Shell,
) -> anyhow::Result<()> {
    crate::fs::create_dir_all(path)?;

    let new_pkg_manifest_path = path.join("Cargo.toml");

    let mut new_manifest = r#"[package]
name = "cargo-compete-template"
version = "0.1.0"
edition = "2018"
publish = false

[[bin]]
name = "cargo-compete-template"
path = "src/main.rs"
"#
    .to_owned();

    if let Some(deps) = deps {
        new_manifest += "\n";
        new_manifest += "[dependencies]\n";
        new_manifest += deps;
    }

    crate::fs::write(&new_pkg_manifest_path, new_manifest)?;
    crate::fs::create_dir_all(path.join("src"))?;
    crate::fs::write(path.join("src").join("main.rs"), main_rs)?;
    shell.status(
        "Created",
        format!("`cargo-compete-template` package at {}", path.display()),
    )?;

    shell.status("Updating", path.join("Cargo.lock").display())?;

    MetadataCommand::new()
        .manifest_path(new_pkg_manifest_path)
        .exec()?;
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum AtcoderCrates {
    None,
    UseNormally,
    UseViaBinary,
}
