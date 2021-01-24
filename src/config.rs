use crate::{project::PackageExt as _, shell::Shell};
use anyhow::Context as _;
use derivative::Derivative;
use heck::KebabCase as _;
use indexmap::indexset;
use krates::cm;
use liquid::object;
use serde::{de::Error as _, Deserialize, Deserializer};
use snowchains_core::web::PlatformKind;
use std::{
    fmt,
    path::{Path, PathBuf},
    str::{self, FromStr},
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

pub(crate) fn load(
    path: impl AsRef<Path>,
    shell: &mut Shell,
) -> anyhow::Result<CargoCompeteConfig> {
    let path = path.as_ref();

    let unused = &mut indexset!();
    let config = serde_ignored::deserialize(
        &mut toml::Deserializer::new(&crate::fs::read_to_string(path)?),
        |path| {
            unused.insert(path.to_string());
        },
    )
    .with_context(|| format!("could not read a TOML file at `{}`", path.display()))?;

    for unused in &*unused {
        shell.warn(format!("unused key in compete.toml: {}", unused))?;
    }

    Ok(config)
}

pub(crate) fn load_for_package(
    package: &cm::Package,
    shell: &mut Shell,
) -> anyhow::Result<(CargoCompeteConfig, PathBuf)> {
    let manifest_dir = package.manifest_path.with_file_name("");
    let path = if let Some(config) = package.read_package_metadata(shell)?.config {
        manifest_dir.join(config)
    } else {
        manifest_dir
            .ancestors()
            .map(|p| p.join("compete.toml"))
            .find(|p| p.exists())
            .with_context(|| {
                format!(
                    "could not find `compete.toml` in `{}` or any parent directory. first, create \
                     one  with `cargo compete init`",
                    manifest_dir.display(),
                )
            })?
    };
    let config = load(&path, shell)?;
    Ok((config, path))
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
    pub(crate) add: Option<CargoCompeteConfigAdd>,
    #[serde(default)]
    pub(crate) test: CargoCompeteConfigTest,
    #[serde(default)]
    pub(crate) submit: CargoCompeteConfigSubmit,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) enum CargoCompeteConfigNew {
    CargoCompete {
        platform: PlatformKind,
        #[derivative(Debug = "ignore")]
        path: liquid::Template,
        template: CargoCompeteConfigNewTemplate,
    },
    OjApi {
        #[derivative(Debug = "ignore")]
        url: liquid::Template,
        #[derivative(Debug = "ignore")]
        path: liquid::Template,
        template: CargoCompeteConfigNewTemplate,
    },
}

impl CargoCompeteConfigNew {
    pub(crate) fn path(&self) -> &liquid::Template {
        match self {
            Self::CargoCompete { path, .. } | Self::OjApi { path, .. } => path,
        }
    }

    pub(crate) fn template(&self) -> &CargoCompeteConfigNewTemplate {
        match self {
            Self::CargoCompete { template, .. } | Self::OjApi { template, .. } => template,
        }
    }
}

impl<'de> Deserialize<'de> for CargoCompeteConfigNew {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        return match WithExplicitTag::deserialize(deserializer)? {
            WithExplicitTag::CargoCompete {
                rest:
                    CargoCompete {
                        platform,
                        path,
                        template,
                    },
                ..
            } => Ok(Self::CargoCompete {
                platform,
                path,
                template,
            }),
            WithExplicitTag::OjApi {
                url,
                path,
                template,
                ..
            } => Ok(Self::OjApi {
                url,
                path,
                template,
            }),
            WithExplicitTag::Other(value) => {
                let CargoCompete {
                    platform,
                    path,
                    template,
                } = value.try_into().map_err(D::Error::custom)?;
                Ok(Self::CargoCompete {
                    platform,
                    path,
                    template,
                })
            }
        };

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum WithExplicitTag {
            CargoCompete {
                #[allow(dead_code)]
                #[serde(deserialize_with = "cargo_compete_tag")]
                kind: (),
                #[serde(flatten)]
                rest: CargoCompete,
            },
            OjApi {
                #[allow(dead_code)]
                #[serde(deserialize_with = "oj_api_tag")]
                kind: (),
                #[serde(deserialize_with = "deserialize_liquid_template_with_custom_filter")]
                url: liquid::Template,
                #[serde(deserialize_with = "deserialize_liquid_template_with_custom_filter")]
                path: liquid::Template,
                template: CargoCompeteConfigNewTemplate,
            },
            Other(toml::Value),
        }

        #[derive(Deserialize)]
        struct CargoCompete {
            #[serde(deserialize_with = "deserialize_platform_kind_in_kebab_case")]
            platform: PlatformKind,
            #[serde(deserialize_with = "deserialize_liquid_template_with_custom_filter")]
            path: liquid::Template,
            template: CargoCompeteConfigNewTemplate,
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

        fn cargo_compete_tag<'de, D>(deserializer: D) -> Result<(), D::Error>
        where
            D: Deserializer<'de>,
        {
            if String::deserialize(deserializer)? != "cargo-compete" {
                return Err(D::Error::custom(""));
            }
            Ok(())
        }

        fn oj_api_tag<'de, D>(deserializer: D) -> Result<(), D::Error>
        where
            D: Deserializer<'de>,
        {
            if String::deserialize(deserializer)? != "oj-api" {
                return Err(D::Error::custom(""));
            }
            Ok(())
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoCompeteConfigNewTemplate {
    pub(crate) toolchain: Option<String>,
    pub(crate) lockfile: Option<PathBuf>,
    #[serde(default, deserialize_with = "deserialize_option_from_str")]
    pub(crate) profile: Option<toml_edit::Document>,
    pub(crate) dependencies: CargoCompeteConfigNewTemplateDependencies,
    pub(crate) src: CargoCompeteConfigNewTemplateSrc,
}

fn deserialize_option_from_str<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
    T::Err: fmt::Display,
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|s| s.parse().map_err(D::Error::custom))
        .transpose()
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

pub(crate) struct CargoCompeteConfigAdd {
    pub(crate) url: liquid::Template,
    pub(crate) is_contest: Option<Vec<String>>,
    pub(crate) bin_name: liquid::Template,
    pub(crate) bin_alias: liquid::Template,
    pub(crate) bin_src_path: liquid::Template,
}

impl<'de> Deserialize<'de> for CargoCompeteConfigAdd {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Repr {
            url,
            is_contest,
            bin_name,
            bin_alias,
            bin_src_path,
        } = Repr::deserialize(deserializer)?;

        let bin_name = &bin_name;
        let bin_alias = bin_alias.as_deref().unwrap_or(bin_name);
        let bin_src_path = bin_src_path
            .as_deref()
            .unwrap_or("src/bin/{{ bin_alias }}.rs");

        let parser = liquid::ParserBuilder::with_stdlib()
            .build()
            .map_err(D::Error::custom)?;
        let parse = |s| parser.parse(s).map_err(D::Error::custom);

        let url = parse(&url)?;
        let bin_name = parse(bin_name)?;
        let bin_alias = parse(bin_alias)?;
        let bin_src_path = parse(bin_src_path)?;

        return Ok(Self {
            url,
            is_contest,
            bin_name,
            bin_alias,
            bin_src_path,
        });

        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct Repr {
            url: String,
            is_contest: Option<Vec<String>>,
            bin_name: String,
            bin_alias: Option<String>,
            bin_src_path: Option<String>,
        }
    }
}

impl fmt::Debug for CargoCompeteConfigAdd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CargoCompeteConfigAdd")
            .field("url", &format_args!("_"))
            .field("is_contest", &self.is_contest)
            .field("bin_name", &format_args!("_"))
            .field("bin_alias", &format_args!("_"))
            .field("bin_src_path", &format_args!("_"))
            .finish()
    }
}

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoCompeteConfigTest {
    #[serde(default)]
    pub(crate) profile: CargoCompeteConfigTestProfile,
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CargoCompeteConfigTestProfile {
    Dev,
    Release,
}

impl Default for CargoCompeteConfigTestProfile {
    fn default() -> Self {
        Self::Dev
    }
}

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoCompeteConfigSubmit {
    pub(crate) transpile: Option<CargoCompeteConfigSubmitTranspile>,
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
