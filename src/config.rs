use anyhow::Context as _;
use derivative::Derivative;
use heck::KebabCase as _;
use liquid::object;
use serde::{de::Error as _, Deserialize, Deserializer};
use snowchains_core::web::PlatformKind;
use std::{
    path::{Path, PathBuf},
    str,
};

pub(crate) fn generate(
    new_platform: PlatformKind,
    new_template_lockfile: Option<&str>,
    new_template_dependencies_content: Option<&str>,
    submit_via_bianry: bool,
) -> anyhow::Result<String> {
    let generated = liquid::ParserBuilder::with_stdlib()
        .build()?
        .parse(include_str!("../resources/compete.toml.liquid"))
        .unwrap()
        .render(&object!({
            "new_platform": new_platform.to_kebab_case_str(),
            "new_template_lockfile": new_template_lockfile,
            "new_template_dependencies_content": new_template_dependencies_content,
            "submit_via_binary": submit_via_bianry,
        }))
        .unwrap();
    Ok(generated)
}

pub(crate) fn locate(
    cwd: impl AsRef<Path>,
    cli_opt_path: Option<impl AsRef<Path>>,
) -> anyhow::Result<PathBuf> {
    let cwd = cwd.as_ref();

    if let Some(cli_opt_path) = cli_opt_path {
        let cli_opt_path = cli_opt_path.as_ref();
        Ok(cwd.join(cli_opt_path.strip_prefix(".").unwrap_or(cli_opt_path)))
    } else {
        cwd.ancestors()
            .map(|p| p.join("compete.toml"))
            .find(|p| p.exists())
            .with_context(|| {
                format!(
                    "could not find `compete.toml` in `{}` or any parent directory. first, create \
                     one  with `cargo compete init`",
                    cwd.display(),
                )
            })
    }
}

pub(crate) fn load(path: impl AsRef<Path>) -> anyhow::Result<CargoCompeteConfig> {
    let path = path.as_ref();
    toml::from_str(&crate::fs::read_to_string(path)?)
        .with_context(|| format!("could not parse the config file at `{}`", path.display()))
}

pub(crate) fn load_from_rel_path(
    manifest_path: &Path,
    rel_path: impl AsRef<Path>,
) -> anyhow::Result<CargoCompeteConfig> {
    load(manifest_path.with_file_name("").join(rel_path))
}

#[derive(Deserialize, Derivative)]
#[derivative(Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoCompeteConfig {
    #[derivative(Debug = "ignore")]
    #[serde(deserialize_with = "deserialize_liquid_template_with_custom_filter")]
    pub(crate) test_suite: liquid::Template,
    pub(crate) open: Option<String>,
    pub(crate) new: CargoCompeteConfigNew,
    #[serde(default)]
    pub(crate) submit: CargoCompeteConfigSubmit,
}

#[derive(Deserialize, Derivative)]
#[derivative(Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoCompeteConfigNew {
    #[serde(deserialize_with = "deserialize_platform_kind_in_kebab_case")]
    pub(crate) platform: PlatformKind,
    #[derivative(Debug = "ignore")]
    #[serde(deserialize_with = "deserialize_liquid_template_with_custom_filter")]
    pub(crate) path: liquid::Template,
    pub(crate) template: CargoCompeteConfigNewTemplate,
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoCompeteConfigNewTemplate {
    pub(crate) toolchain: Option<String>,
    pub(crate) lockfile: Option<PathBuf>,
    pub(crate) dependencies: CargoCompeteConfigNewTemplateDependencies,
    pub(crate) src: CargoCompeteConfigNewTemplateSrc,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub(crate) enum CargoCompeteConfigNewTemplateDependencies {
    Inline { content: String },
    ManifestFile { path: PathBuf },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub(crate) enum CargoCompeteConfigNewTemplateSrc {
    Inline { content: String },
    File { path: PathBuf },
}

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoCompeteConfigSubmit {
    pub(crate) transpile: Option<CargoCompeteConfigSubmitTranspile>,
    pub(crate) via_binary: Option<CargoCompeteConfigSubmitViaBinary>,
}

#[derive(Deserialize, Derivative)]
#[serde(rename_all = "kebab-case", tag = "kind")]
#[derivative(Debug)]
pub(crate) enum CargoCompeteConfigSubmitTranspile {
    Command {
        #[serde(deserialize_with = "deserialize_liquid_templates")]
        #[derivative(Debug = "ignore")]
        args: Vec<liquid::Template>,
        language_id: Option<String>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoCompeteConfigSubmitViaBinary {
    pub(crate) target: String,
    pub(crate) cross: Option<PathBuf>,
    pub(crate) strip: Option<PathBuf>,
    pub(crate) upx: Option<PathBuf>,
}

fn deserialize_liquid_templates<'de, D>(deserializer: D) -> Result<Vec<liquid::Template>, D::Error>
where
    D: Deserializer<'de>,
{
    use liquid::ParserBuilder;

    let parser = ParserBuilder::with_stdlib()
        .build()
        .map_err(D::Error::custom)?;

    Vec::<String>::deserialize(deserializer)?
        .iter()
        .map(|s| parser.parse(s))
        .collect::<Result<_, _>>()
        .map_err(D::Error::custom)
}

fn deserialize_liquid_template_with_custom_filter<'de, D>(
    deserializer: D,
) -> Result<liquid::Template, D::Error>
where
    D: Deserializer<'de>,
{
    liquid_template_with_custom_filter(&String::deserialize(deserializer)?)
        .map_err(D::Error::custom)
}

fn liquid_template_with_custom_filter(text: &str) -> Result<liquid::Template, String> {
    use liquid::ParserBuilder;
    use liquid_core::{Filter, Runtime, Value, ValueView};
    use liquid_derive::{Display_filter, FilterReflection, ParseFilter};

    return ParserBuilder::with_stdlib()
        .filter(Kebabcase)
        .build()
        .map_err(|e| e.to_string())?
        .parse(text)
        .map_err(|e| e.to_string());

    #[derive(Clone, ParseFilter, FilterReflection)]
    #[filter(
        name = "kebabcase",
        description = "Converts a string to kebab-case.",
        parsed(KebabcaseFilter)
    )]
    struct Kebabcase;

    #[derive(Default, Debug, Display_filter)]
    #[name = "kebabcase"]
    struct KebabcaseFilter;

    impl Filter for KebabcaseFilter {
        fn evaluate(&self, input: &dyn ValueView, _: &Runtime<'_>) -> liquid_core::Result<Value> {
            Ok(Value::scalar(input.to_kstr().to_kebab_case()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::CargoCompeteConfig;
    use itertools::iproduct;
    use liquid::object;
    use pretty_assertions::assert_eq;
    use snowchains_core::web::PlatformKind;

    #[test]
    fn generate() -> anyhow::Result<()> {
        fn generate(
            new_template_lockfile: bool,
            new_template_dependencies_content: bool,
            submit_via_bianry: bool,
        ) -> anyhow::Result<()> {
            let generated = super::generate(
                PlatformKind::Atcoder,
                if new_template_lockfile {
                    Some("./cargo-lock-template.toml")
                } else {
                    None
                },
                if new_template_dependencies_content {
                    Some(include_str!("../resources/atcoder-deps.toml"))
                } else {
                    None
                },
                submit_via_bianry,
            )?;

            toml::from_str::<CargoCompeteConfig>(&generated)?;
            Ok(())
        }

        for (&p1, &p2, &p3) in iproduct!(&[false, true], &[false, true], &[false, true]) {
            generate(p1, p2, p3)?;
        }
        Ok(())
    }

    #[test]
    fn liquid_template_with_custom_filter() -> anyhow::Result<()> {
        let output = super::liquid_template_with_custom_filter("{{ s | kebabcase }}")
            .map_err(anyhow::Error::msg)?
            .render(&object!({ "s": "FooBarBaz" }))?;
        assert_eq!("foo-bar-baz", output);
        Ok(())
    }
}
