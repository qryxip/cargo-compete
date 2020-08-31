use crate::shell::ColorChoice;
use anyhow::{anyhow, Context as _};
use derivative::Derivative;
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use krates::cm;
use liquid::object;
use serde::{Deserialize, Deserializer};
use snowchains_core::web::PlatformKind;
use std::path::PathBuf;
use structopt::StructOpt;
use strum::VariantNames as _;
use termcolor::Color;

#[derive(StructOpt, Debug)]
pub struct OptCompeteMigrateV04 {
    /// Process glob patterns given with the `--glob` flag case insensitively
    #[structopt(long)]
    pub glob_case_insensitive: bool,

    /// Include or exclude manifest paths. For more detail, see the help of ripgrep
    #[structopt(short, long, value_name("GLOB"))]
    pub glob: Vec<String>,

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
}

pub(crate) fn run(opt: OptCompeteMigrateV04, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteMigrateV04 {
        glob_case_insensitive,
        glob,
        manifest_path,
        color,
    } = opt;

    let crate::Context {
        cwd,
        cookies_path: _,
        shell,
    } = ctx;

    shell.set_color_choice(color);

    let ws_manifest_path = manifest_path
        .map(|p| Ok(cwd.join(p.strip_prefix(".").unwrap_or(&p))))
        .unwrap_or_else(|| crate::project::locate_project(&cwd))
        .map_err(|_| {
            anyhow!(
                "could not find `Cargo.toml` in `{}` or any parent directory. this command \
                 targets one workspace",
                cwd.display(),
            )
        })?;

    let cm::Metadata { workspace_root, .. } =
        crate::project::cargo_metadata_no_deps(&ws_manifest_path, &cwd)?;

    let config_path = workspace_root.join("compete.toml");

    let old_config = crate::fs::read_toml::<OldCargoCompeteConfig, _>(&config_path)?;

    let pkg_manifest_paths = WalkBuilder::new(&workspace_root)
        .follow_links(false)
        .max_depth(Some(2))
        .overrides({
            let mut overrides = OverrideBuilder::new(&workspace_root);
            for glob in glob {
                overrides.add(&glob)?;
            }
            overrides.case_insensitive(glob_case_insensitive)?.build()?
        })
        .build()
        .map(|entry| {
            let path = entry?.into_path();
            Ok(
                if path.file_name() == Some("Cargo.toml".as_ref()) && path != ws_manifest_path {
                    Some(path)
                } else {
                    None
                },
            )
        })
        .flat_map(Result::transpose)
        .collect::<Result<Vec<_>, ignore::Error>>()?;

    (|| -> anyhow::Result<()> {
        let mut new_config = liquid::ParserBuilder::with_stdlib()
            .build()?
            .parse(include_str!("../../resources/compete.toml.liquid"))?
            .render(&object!({
                "new_platform": old_config.template.platform.to_kebab_case_str(),
                "submit_via_binary": false,
            }))?
            .parse::<toml_edit::Document>()?;

        new_config["test-suite"] = toml_edit::value(old_config.test_suite);

        if let Some(open) = old_config.open {
            new_config["open"] = toml_edit::value(open);
        }

        new_config["new"]["template"]["dependencies"]["kind"] = toml_edit::value("manifest-file");
        new_config["new"]["template"]["dependencies"]["content"] = toml_edit::Item::None;
        new_config["new"]["template"]["dependencies"]["path"] =
            toml_edit::value(old_config.template.manifest);

        new_config["new"]["template"]["src"]["kind"] = toml_edit::value("file");
        new_config["new"]["template"]["src"]["content"] = toml_edit::Item::None;
        new_config["new"]["template"]["dependencies"]["path"] =
            toml_edit::value(old_config.template.src);

        if let Some(submit_via_bianry) = old_config.submit_via_binary {
            new_config["submit"] = implicit_table();
            new_config["submit"]["via-binary"] = implicit_table();

            let entry = &mut new_config["submit"]["via-binary"];

            entry["target"] = toml_edit::value(submit_via_bianry.target);
            if let Some(cross) = submit_via_bianry.cross {
                entry["cross"] = toml_edit::value(cross.into_os_string().into_string().unwrap());
            }
            if let Some(strip) = submit_via_bianry.strip {
                entry["strip"] = toml_edit::value(strip.into_os_string().into_string().unwrap());
            }
            if let Some(upx) = submit_via_bianry.upx {
                entry["upx"] = toml_edit::value(upx.into_os_string().into_string().unwrap());
            }
        }

        crate::fs::write(&config_path, new_config.to_string())?;
        shell.status("Wrote", config_path.display())?;

        crate::fs::remove_file(&ws_manifest_path)?;
        shell.status_with_color("Removed", ws_manifest_path.display(), Color::Red)?;

        let lockfile_path = workspace_root.join("Cargo.lock");
        if lockfile_path.exists() {
            crate::fs::remove_file(&lockfile_path)?;
            shell.status_with_color("Removed", lockfile_path.display(), Color::Red)?;
        }

        crate::fs::create_dir_all(ws_manifest_path.join(".cargo"))?;
        let cargo_config_path = ws_manifest_path.join(".cargo").join("config.toml");
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

        for pkg_manifest_path in pkg_manifest_paths {
            let mut manifest =
                crate::fs::read_to_string(&pkg_manifest_path)?.parse::<toml_edit::Document>()?;

            if !manifest["package"]["metadata"]["cargo-compete"].is_none() {
                manifest["package"]["metadata"]["cargo-compete"]["config"] =
                    toml_edit::value("../compete.toml");

                crate::fs::write(&pkg_manifest_path, manifest.to_string_in_original_order())?;
                shell.status("Wrote", pkg_manifest_path.display())?;
            }

            shell.status(
                "Updating",
                pkg_manifest_path.with_file_name("Cargo.lock").display(),
            )?;
            crate::project::cargo_metadata(pkg_manifest_path, &cwd)?;
        }

        Ok(())
    })()
    .with_context(|| {
        "could not migrate. Run `git clean -f && git restore .`, and this command again with \
         `--glob` option"
    })?;

    shell.status("Finished", "migrating")?;
    Ok(())
}

fn implicit_table() -> toml_edit::Item {
    let mut tbl = toml_edit::Table::new();
    tbl.set_implicit(true);
    toml_edit::Item::Table(tbl)
}

#[derive(Deserialize, Derivative)]
#[derivative(Debug)]
#[serde(rename_all = "kebab-case")]
struct OldCargoCompeteConfig {
    test_suite: String,
    open: Option<String>,
    template: CargoCompeteConfigTempate,
    submit_via_binary: Option<crate::config::CargoCompeteConfigSubmitViaBinary>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct CargoCompeteConfigTempate {
    #[serde(deserialize_with = "deserialize_platform_kind_in_kebab_case")]
    platform: PlatformKind,
    manifest: String,
    src: String,
}

fn deserialize_platform_kind_in_kebab_case<'de, D>(
    deserializer: D,
) -> Result<PlatformKind, D::Error>
where
    D: Deserializer<'de>,
{
    return PlatformKindKebabCased::deserialize(deserializer).map(|kind| match kind {
        PlatformKindKebabCased::Atcoder => PlatformKind::Atcoder,
        PlatformKindKebabCased::Codeforces => PlatformKind::Codeforces,
        PlatformKindKebabCased::Yukicoder => PlatformKind::Yukicoder,
    });

    #[derive(Deserialize)]
    #[serde(rename_all = "kebab-case")]
    enum PlatformKindKebabCased {
        Atcoder,
        Codeforces,
        Yukicoder,
    }
}
