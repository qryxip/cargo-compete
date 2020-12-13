use crate::{
    project::{MetadataExt as _, PackageExt as _},
    shell::ColorChoice,
};
use anyhow::{anyhow, Context as _};
use if_chain::if_chain;
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use itertools::Itertools as _;
use snowchains_core::web::PlatformKind;
use std::{
    iter,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use strum::VariantNames as _;
use termcolor::Color;

#[derive(StructOpt, Debug)]
pub struct OptCompeteMigrateCargoAtcoder {
    /// Process glob patterns given with the `--glob` flag case insensitively
    #[structopt(long)]
    pub glob_case_insensitive: bool,

    /// Include or exclude manifest paths. For more detail, see the help of ripgrep
    #[structopt(short, long, value_name("GLOB"))]
    pub glob: Vec<String>,

    /// Path to `cargo-atcoder.toml`
    pub cargo_atcoder_config: Option<PathBuf>,

    /// Coloring
    #[structopt(
        long,
        value_name("WHEN"),
        possible_values(ColorChoice::VARIANTS),
        default_value("auto")
    )]
    pub color: ColorChoice,

    #[structopt(default_value("."))]
    pub path: PathBuf,
}

pub(crate) fn run(
    opt: OptCompeteMigrateCargoAtcoder,
    ctx: crate::Context<'_>,
) -> anyhow::Result<()> {
    let OptCompeteMigrateCargoAtcoder {
        glob_case_insensitive,
        glob,
        cargo_atcoder_config,
        color,
        path,
    } = opt;

    let crate::Context {
        cwd,
        cookies_path: _,
        shell,
    } = ctx;

    shell.set_color_choice(color);

    let path = cwd.join(path.strip_prefix(".").unwrap_or(&path));

    let manifest_paths = WalkBuilder::new(&path)
        .follow_links(true)
        .max_depth(Some(32))
        .overrides({
            let mut overrides = OverrideBuilder::new(&path);
            for glob in glob {
                overrides.add(&glob)?;
            }
            overrides.case_insensitive(glob_case_insensitive)?.build()?
        })
        .build()
        .map(|entry| {
            let manifest_path = entry?.into_path();
            Ok(
                if manifest_path.file_name() == Some("Cargo.toml".as_ref()) {
                    Some(manifest_path)
                } else {
                    None
                },
            )
        })
        .flat_map(Result::transpose)
        .collect::<Result<Vec<_>, ignore::Error>>()?;

    let mut packages = vec![];

    for manifest_path in manifest_paths.into_iter().sorted() {
        let metadata = crate::project::cargo_metadata_no_deps(&manifest_path, &cwd)?;
        if_chain! {
            if let [package] = *metadata.all_members();
            if package.manifest_path == manifest_path;
            then {
                shell.status("Found", format_args!("`{}`", manifest_path.display()))?;
                packages.push(package.clone());
            } else {
                shell.status_with_color(
                    "Ignoring",
                    format_args!("`{}`", manifest_path.display()),
                    Color::Cyan,
                )?;
            }
        }
    }

    for package in &packages {
        let mut manifest =
            crate::fs::read_to_string(&package.manifest_path)?.parse::<toml_edit::Document>()?;

        let bins = package.all_bin_targets_sorted();

        if manifest["package"]["metadata"]["cargo-compete"].is_none() {
            manifest["package"]["metadata"] = implicit_table();
            manifest["package"]["metadata"]["cargo-compete"] = implicit_table();
            manifest["package"]["metadata"]["cargo-compete"]["config"] = toml_edit::value({
                let manifest_dir = package.manifest_path.with_file_name("");
                if let Ok(rel_manifest_dir) = manifest_dir.strip_prefix(&path) {
                    rel_manifest_dir
                        .iter()
                        .map(|_| "..")
                        .chain(iter::once("compete.toml"))
                        .join(&std::path::MAIN_SEPARATOR.to_string())
                } else {
                    manifest_dir
                        .into_os_string()
                        .into_string()
                        .map_err(|s| anyhow!("invalid utf-8 path: {:?}", s))?
                }
            });
            manifest["package"]["metadata"]["cargo-compete"]["bin"] = toml_edit::Item::Table({
                let mut tbl = toml_edit::Table::new();
                for bin in &bins {
                    tbl[&bin.name]["name"] =
                        toml_edit::value(format!("{}-{}", package.name, bin.name));
                    tbl[&bin.name]["problem"] = toml_edit::value(format!(
                        "https://atcoder.jp/contests/{}/<FIXME: screen name of the problem>",
                        package.name,
                    ));
                }
                tbl
            });
        }

        if manifest["bin"].is_none() {
            manifest["bin"] = toml_edit::Item::ArrayOfTables({
                let mut arr = toml_edit::ArrayOfTables::new();
                for bin in bins {
                    let mut tbl = toml_edit::Table::new();
                    tbl["name"] = toml_edit::value(format!("{}-{}", package.name, bin.name));
                    tbl["path"] = toml_edit::value(format!("./src/bin/{}.rs", bin.name));
                    arr.append(tbl);
                }
                arr
            });
        }

        crate::fs::write(&package.manifest_path, manifest.to_string())?;
        shell.status("Modified", package.manifest_path.display())?;
    }

    for package in &packages {
        let lock_path = package.manifest_path.with_file_name("Cargo.lock");
        shell.status("Updating", lock_path.display())?;
        if let Err(err) = crate::project::cargo_metadata(&package.manifest_path, &cwd) {
            shell.warn(format!("broke `{}`!!!!!: {}", lock_path.display(), err))?;
        }
    }

    crate::fs::create_dir_all(path.join(".cargo"))?;
    let cargo_config_path = path.join(".cargo").join("config.toml");
    let mut cargo_config = if cargo_config_path.exists() {
        crate::fs::read_to_string(&cargo_config_path)?
    } else {
        r#"[build]
target-dir = ""
"#
        .to_owned()
    }
    .parse::<toml_edit::Document>()
    .with_context(|| {
        format!(
            "could not parse the TOML file at `{}`",
            cargo_config_path.display(),
        )
    })?;
    if cargo_config["build"]["target-dir"].is_none() {
        cargo_config["build"]["target-dir"] = toml_edit::value("../target");
        crate::fs::write(
            &cargo_config_path,
            cargo_config.to_string_in_original_order(),
        )?;
        shell.status("Wrote", cargo_config_path.display())?;
    }

    let cargo_atcoder_config = (|| -> _ {
        fn parse(path: &Path) -> anyhow::Result<toml_edit::Document> {
            crate::fs::read_to_string(path)?.parse().with_context(|| {
                format!(
                    "could not parse the cargo-atcoder config at `{}`",
                    path.display()
                )
            })
        }

        if let Some(cargo_atcoder_config) = cargo_atcoder_config {
            let cargo_atcoder_config = cwd.join(cargo_atcoder_config);
            if cargo_atcoder_config.exists() {
                return parse(&cargo_atcoder_config).map(Some);
            }
        }

        if let Some(config_dir) = dirs_next::config_dir() {
            let cargo_atcoder_config = config_dir.join("cargo-atcoder.toml");
            if cargo_atcoder_config.exists() {
                return parse(&cargo_atcoder_config).map(Some);
            }
        }

        Ok(None)
    })()?;

    let submit_via_binary = matches!(
        &cargo_atcoder_config,
        Some(c) if c["atcoder"]["submit_via_binary"].as_bool() == Some(true)
    );

    let compete_toml_path = path.join("compete.toml");
    let compete_toml = crate::config::generate(
        PlatformKind::Atcoder,
        None,
        cargo_atcoder_config
            .as_ref()
            .and_then(|c| c["dependencies"].as_table())
            .map(|t| t.to_string())
            .as_deref(),
        submit_via_binary,
    )?;

    crate::fs::write(&compete_toml_path, compete_toml)?;
    shell.status("Wrote", compete_toml_path.display())?;

    shell.status("Finished", "migrating")?;
    Ok(())
}

fn implicit_table() -> toml_edit::Item {
    let mut tbl = toml_edit::Table::new();
    tbl.set_implicit(true);
    toml_edit::Item::Table(tbl)
}
