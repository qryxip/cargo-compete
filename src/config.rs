use anyhow::{bail, Context as _};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Config {
    pub(crate) workspace_members: WorkspaceMembers,
    #[serde(default)]
    pub(crate) atcoder: ConfigAtcoder,
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum WorkspaceMembers {
    IncludeAll,
    ExcludeAll,
    FocusOne,
}

#[derive(Default, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ConfigAtcoder {
    //dependencies: Option<toml::Value>,
    pub(crate) via_binary: Option<ConfigAtcoderViaBinary>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ConfigAtcoderViaBinary {
    pub(crate) use_cross: bool,
    pub(crate) strip_exe: Option<String>,
    pub(crate) upx_exe: Option<String>,
}

pub(crate) fn path() -> anyhow::Result<PathBuf> {
    let config_dir = dirs::config_dir().with_context(|| "could not find the config directory")?;
    Ok(config_dir.join("cargo-compete.toml"))
}

pub(crate) fn load_with_preserving_atcoder_dependencies(
) -> anyhow::Result<(Config, Option<toml_edit::Table>)> {
    let path = path()?;

    if !path.exists() {
        bail!("`{}` does not exist. Run `cargo compete init` first");
    }

    let content = crate::fs::read_to_string(&path)?;

    let config = toml::from_str(&content)
        .with_context(|| format!("could not parse the config file at `{}`", path.display()))?;

    let dependencies = content
        .parse::<toml_edit::Document>()
        .with_context(|| format!("could not parse the config file at `{}`", path.display()))?
        ["atcoder"]["dependencies"]
        .as_table()
        .cloned();

    Ok((config, dependencies))
}
