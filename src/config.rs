use crate::{project::PackageExt as _, shell::Shell};
use anyhow::{bail, Context as _};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata as cm;
use derivative::Derivative;
use heck::KebabCase as _;
use indexmap::indexset;
use liquid::object;
use maplit::btreemap;
use serde::{de::Error as _, Deserialize, Deserializer};
use snowchains_core::web::PlatformKind;
use std::{
    collections::BTreeMap,
    fmt,
    path::Path,
    str::{self, FromStr},
};

pub(crate) fn generate(
    template_new_dependencies_content: Option<&str>,
    template_new_lockfile: Option<&str>,
    new_platform: PlatformKind,
    test_toolchain: &str,
    submit_via_bianry: bool,
) -> anyhow::Result<String> {
    let generated = liquid::ParserBuilder::with_stdlib()
        .build()?
        .parse(include_str!("../resources/compete.toml.liquid"))
        .unwrap()
        .render(&object!({
            "new_platform": new_platform.to_kebab_case_str(),
            "template_new_dependencies_content": template_new_dependencies_content,
            "template_new_lockfile": template_new_lockfile,
            "test_toolchain": test_toolchain,
            "submit_via_binary": submit_via_bianry,
        }))
        .unwrap();
    Ok(generated)
}

pub(crate) fn locate(
    cwd: impl AsRef<Path>,
    cli_opt_path: Option<impl AsRef<Utf8Path>>,
) -> anyhow::Result<Utf8PathBuf> {
    let cwd = cwd.as_ref();

    let config_path = if let Some(cli_opt_path) = cli_opt_path {
        let cli_opt_path = cli_opt_path.as_ref();
        cwd.join(cli_opt_path.strip_prefix(".").unwrap_or(cli_opt_path))
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
            })?
    };

    config_path
        .to_str()
        .map(Into::into)
        .with_context(|| format!("non UTF-8 path: {:?}", config_path.display()))
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
) -> anyhow::Result<(CargoCompeteConfig, Utf8PathBuf)> {
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
                    manifest_dir,
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
    template: Option<CargoCompeteConfigTemplate>,
    #[serde(default)]
    pub(crate) new: CargoCompeteConfigNew,
    pub(crate) add: Option<CargoCompeteConfigAdd>,
    #[serde(default)]
    pub(crate) test: CargoCompeteConfigTest,
    #[serde(default)]
    pub(crate) submit: CargoCompeteConfigSubmit,
}

impl CargoCompeteConfig {
    pub(crate) fn template(
        &self,
        config_path: &Utf8Path,
        shell: &mut Shell,
    ) -> anyhow::Result<CargoCompeteConfigTemplate> {
        if let Some(template) = &self.template {
            Ok(template.clone())
        } else if let Some(CargoCompeteConfigNewTemplate {
            lockfile,
            profile,
            dependencies,
            src,
            ..
        }) = self.new.template()
        {
            shell.warn("`new.template` is deprecated. see https://github.com/qryxip/cargo-compete#configuration")?;

            let read = |rel_path: &Utf8Path| -> _ {
                crate::fs::read_to_string(config_path.with_file_name("").join(rel_path))
            };

            let src = match src {
                CargoCompeteConfigNewTemplateSrc::Inline { content } => content.clone(),
                CargoCompeteConfigNewTemplateSrc::File { path } => read(path)?,
            };

            let profile = profile.clone().unwrap_or_default();

            let dependencies = match dependencies {
                CargoCompeteConfigNewTemplateDependencies::Inline { content } => {
                    content.parse::<toml_edit::Document>().with_context(|| {
                        "could not parse the toml value in `new.template.dependencies.content`"
                    })?
                }
                CargoCompeteConfigNewTemplateDependencies::ManifestFile { path } => {
                    let mut dependencies = toml_edit::Document::new();
                    let root = read(path)?.parse::<toml_edit::Document>()?["dependencies"].clone();
                    if root.is_table() {
                        dependencies.root = root;
                    } else if !root.is_none() {
                        bail!("`dependencies` is not a `Table`");
                    }
                    dependencies
                }
            };

            let copy_files = lockfile
                .as_ref()
                .map(|p| btreemap!(p.clone() => "Cargo.lock".into()))
                .unwrap_or_default();

            Ok(CargoCompeteConfigTemplate {
                src,
                new: Some(CargoCompeteConfigTemplateNew {
                    profile,
                    dependencies,
                    dev_dependencies: toml_edit::Document::new(),
                    copy_files,
                }),
            })
        } else {
            bail!("`template` or `new.template` is required: {}", config_path);
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoCompeteConfigTemplate {
    pub(crate) src: String,
    pub(crate) new: Option<CargoCompeteConfigTemplateNew>,
}

#[derive(Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoCompeteConfigTemplateNew {
    #[serde(default, with = "serde_with::rust::display_fromstr")]
    pub(crate) profile: toml_edit::Document,
    #[serde(default, with = "serde_with::rust::display_fromstr")]
    pub(crate) dependencies: toml_edit::Document,
    #[serde(default, with = "serde_with::rust::display_fromstr")]
    pub(crate) dev_dependencies: toml_edit::Document,
    #[serde(default)]
    pub(crate) copy_files: BTreeMap<Utf8PathBuf, Utf8PathBuf>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) enum CargoCompeteConfigNew {
    None,
    CargoCompete {
        platform: PlatformKind,
        #[derivative(Debug = "ignore")]
        path: liquid::Template,
        template: Option<CargoCompeteConfigNewTemplate>,
    },
    OjApi {
        #[derivative(Debug = "ignore")]
        url: liquid::Template,
        #[derivative(Debug = "ignore")]
        path: liquid::Template,
        template: Option<CargoCompeteConfigNewTemplate>,
    },
}

impl CargoCompeteConfigNew {
    pub(crate) fn path(&self) -> Option<&liquid::Template> {
        match self {
            Self::None => None,
            Self::CargoCompete { path, .. } | Self::OjApi { path, .. } => Some(path),
        }
    }

    fn template(&self) -> Option<&CargoCompeteConfigNewTemplate> {
        match self {
            Self::None => None,
            Self::CargoCompete { template, .. } | Self::OjApi { template, .. } => template.as_ref(),
        }
    }
}

impl<'de> Deserialize<'de> for CargoCompeteConfigNew {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        return match WithExplicitTag::deserialize(deserializer)? {
            WithExplicitTag::None => Ok(Self::None),
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
            None,
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
                template: Option<CargoCompeteConfigNewTemplate>,
            },
            Other(toml::Value),
        }

        #[derive(Deserialize)]
        struct CargoCompete {
            #[serde(deserialize_with = "deserialize_platform_kind_in_kebab_case")]
            platform: PlatformKind,
            #[serde(deserialize_with = "deserialize_liquid_template_with_custom_filter")]
            path: liquid::Template,
            template: Option<CargoCompeteConfigNewTemplate>,
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

impl Default for CargoCompeteConfigNew {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoCompeteConfigNewTemplate {
    toolchain: Option<String>,
    lockfile: Option<Utf8PathBuf>,
    #[serde(default, deserialize_with = "deserialize_option_from_str")]
    profile: Option<toml_edit::Document>,
    dependencies: CargoCompeteConfigNewTemplateDependencies,
    src: CargoCompeteConfigNewTemplateSrc,
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
enum CargoCompeteConfigNewTemplateDependencies {
    Inline { content: String },
    ManifestFile { path: Utf8PathBuf },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum CargoCompeteConfigNewTemplateSrc {
    Inline { content: String },
    File { path: Utf8PathBuf },
}

pub(crate) struct CargoCompeteConfigAdd {
    pub(crate) url: liquid::Template,
    pub(crate) is_contest: Option<Vec<String>>,
    pub(crate) target_kind: BinLikeTargetKind,
    pub(crate) bin_name: liquid::Template,
    pub(crate) bin_alias: liquid::Template,
    pub(crate) bin_src_path: Option<liquid::Template>,
}

impl<'de> Deserialize<'de> for CargoCompeteConfigAdd {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Repr {
            url,
            is_contest,
            target_kind,
            bin_name,
            bin_alias,
            bin_src_path,
        } = Repr::deserialize(deserializer)?;

        let bin_name = &bin_name;
        let bin_alias = bin_alias.as_deref().unwrap_or(bin_name);
        let bin_src_path = bin_src_path.as_deref();

        let parser = liquid::ParserBuilder::with_stdlib()
            .build()
            .map_err(D::Error::custom)?;
        let parse = |s| parser.parse(s).map_err(D::Error::custom);

        let url = parse(&url)?;
        let bin_name = parse(bin_name)?;
        let bin_alias = parse(bin_alias)?;
        let bin_src_path = bin_src_path.map(parse).transpose()?;

        return Ok(Self {
            url,
            is_contest,
            target_kind,
            bin_name,
            bin_alias,
            bin_src_path,
        });

        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct Repr {
            url: String,
            is_contest: Option<Vec<String>>,
            #[serde(default)]
            target_kind: BinLikeTargetKind,
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

#[derive(Deserialize, Copy, Clone, Debug)]
pub(crate) enum BinLikeTargetKind {
    #[serde(rename = "bin")]
    Bin,
    #[serde(rename = "example")]
    ExampleBin,
}

impl Default for BinLikeTargetKind {
    fn default() -> Self {
        Self::Bin
    }
}

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoCompeteConfigTest {
    pub(crate) toolchain: Option<String>,
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
        fn evaluate(&self, input: &dyn ValueView, _: &dyn Runtime) -> liquid_core::Result<Value> {
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
            template_new_dependencies_content: bool,
            template_new_lockfile: bool,
            submit_via_bianry: bool,
        ) -> anyhow::Result<()> {
            let generated = super::generate(
                template_new_dependencies_content
                    .then(|| include_str!("../resources/atcoder-deps.toml")),
                template_new_lockfile.then(|| "./cargo-lock-template.toml"),
                PlatformKind::Atcoder,
                "1.42.0",
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
