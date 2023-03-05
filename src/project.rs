use crate::shell::Shell;
use anyhow::{bail, Context as _};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata as cm;
use easy_ext::ext;
use indexmap::{indexset, IndexMap};
use itertools::Itertools as _;
use serde::{
    de::{Deserializer, Error as _, IntoDeserializer},
    Deserialize,
};
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    str,
};
use url::Url;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct PackageMetadataCargoCompete {
    pub(crate) config: Option<Utf8PathBuf>,
    #[serde(default, deserialize_with = "deserialize_bin_example")]
    pub(crate) bin: IndexMap<String, PackageMetadataCargoCompeteBinExample>,
    #[serde(default, deserialize_with = "deserialize_bin_example")]
    pub(crate) example: IndexMap<String, PackageMetadataCargoCompeteBinExample>,
}

fn deserialize_bin_example<'de, D>(
    deserializer: D,
) -> Result<IndexMap<String, PackageMetadataCargoCompeteBinExample>, D::Error>
where
    D: Deserializer<'de>,
{
    let map = IndexMap::<String, Repr>::deserialize(deserializer)?;
    return Ok(map
        .into_iter()
        .map(
            |(
                key,
                Repr {
                    name,
                    alias,
                    problem,
                },
            )| {
                let (name, alias) = if let Some(alias) = alias {
                    (key, alias)
                } else if let Some(name) = name {
                    (name, key)
                } else {
                    (key.clone(), key)
                };
                (
                    name,
                    PackageMetadataCargoCompeteBinExample { alias, problem },
                )
            },
        )
        .collect());

    #[derive(Deserialize)]
    #[serde(rename_all = "kebab-case")]
    struct Repr {
        name: Option<String>,
        alias: Option<String>,
        #[serde(deserialize_with = "deserialize_bin_problem")]
        problem: Url,
    }

    fn deserialize_bin_problem<'de, D>(deserializer: D) -> Result<Url, D::Error>
    where
        D: Deserializer<'de>,
    {
        return match Repr::deserialize(deserializer) {
            Ok(Repr::V1 { url }) | Ok(Repr::V2(url)) => Ok(url),
            Err(_) => Err(D::Error::custom(r#"expected `"<url>" | { url: "<url>" }`"#)),
        };

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            V1 { url: Url },
            V2(Url),
        }
    }
}

impl PackageMetadataCargoCompete {
    pub(crate) fn bin_like_by_name_or_alias(
        &self,
        name_or_alias: impl AsRef<str>,
    ) -> anyhow::Result<(&str, &PackageMetadataCargoCompeteBinExample)> {
        let bin_name_or_alias = name_or_alias.as_ref();

        match *itertools::chain(&self.bin, &self.example)
            .filter(
                |(name, PackageMetadataCargoCompeteBinExample { alias, .. })| {
                    [&**name, &**alias].contains(&bin_name_or_alias)
                },
            )
            .collect::<Vec<_>>()
        {
            [(k, v)] => Ok((k, v)),
            [] => bail!("no `problem` for: {}", bin_name_or_alias),
            [..] => bail!("multiple `problem`s for {}", bin_name_or_alias),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct PackageMetadataCargoCompeteBinExample {
    pub(crate) alias: String,
    pub(crate) problem: Url,
}

#[ext(MetadataExt)]
impl cm::Metadata {
    pub(crate) fn all_members(&self) -> Vec<&cm::Package> {
        self.packages
            .iter()
            .filter(|cm::Package { id, .. }| self.workspace_members.contains(id))
            .collect()
    }

    pub(crate) fn query_for_member<S: AsRef<str>>(
        &self,
        spec: Option<S>,
    ) -> anyhow::Result<&cm::Package> {
        if let Some(spec_str) = spec {
            let spec_str = spec_str.as_ref();
            let spec = spec_str.parse::<krates::PkgSpec>()?;

            match *self
                .packages
                .iter()
                .filter(|package| {
                    self.workspace_members.contains(&package.id) && spec.matches(package)
                })
                .collect::<Vec<_>>()
            {
                [] => bail!("package `{}` is not a member of the workspace", spec_str),
                [member] => Ok(member),
                [_, _, ..] => bail!("`{}` matched multiple members?????", spec_str),
            }
        } else {
            let current_member = self
                .resolve
                .as_ref()
                .and_then(|cm::Resolve { root, .. }| root.as_ref())
                .map(|root| &self[root]);

            if let Some(current_member) = current_member {
                Ok(current_member)
            } else {
                match *self.workspace_members.iter().collect::<Vec<_>>() {
                    [] => bail!("this workspace has no members",),
                    [one] => Ok(&self[one]),
                    [..] => {
                        bail!(
                            "this manifest is virtual, and the workspace has {} members. specify \
                             one with `--manifest-path` or `--package`",
                            self.workspace_members.len(),
                        );
                    }
                }
            }
        }
    }
}

#[ext(PackageExt)]
impl cm::Package {
    pub(crate) fn manifest_dir(&self) -> &Utf8Path {
        self.manifest_path
            .parent()
            .expect("`manifest_path` should end with `Cargo.toml`")
    }

    pub(crate) fn read_package_metadata(
        &self,
        shell: &mut Shell,
    ) -> anyhow::Result<PackageMetadataCargoCompete> {
        let unused = &mut indexset!();

        let deserializer = self
            .metadata
            .get("cargo-compete")
            .cloned()
            .unwrap_or_else(|| json!({}))
            .into_deserializer();

        let ret = serde_ignored::deserialize(deserializer, |path| {
            unused.insert(path.to_string());
        })
        .with_context(|| "could not parse `package.metadata.cargo-compete`")?;

        for unused in &*unused {
            shell.warn(format!(
                "unused key in `package.metadata.cargo-compete`: {unused}",
            ))?;
        }

        Ok(ret)
    }

    pub(crate) fn bin_like_target_by_name(
        &self,
        name: impl AsRef<str>,
    ) -> anyhow::Result<&cm::Target> {
        let name = name.as_ref();

        self.targets
            .iter()
            .find(|t| {
                t.name == name && t.kind == ["bin".to_owned()] || t.kind == ["example".to_owned()]
            })
            .with_context(|| format!("no bin/example target named `{}` in `{}`", name, self.name))
    }

    pub(crate) fn bin_target_by_src_path(
        &self,
        src_path: impl AsRef<Path>,
    ) -> anyhow::Result<&cm::Target> {
        let src_path = src_path.as_ref();

        self.targets
            .iter()
            .find(|t| t.src_path == src_path && t.kind == ["bin".to_owned()])
            .with_context(|| {
                format!(
                    "no bin target which `src_path` is `{}` in `{}`",
                    src_path.display(),
                    self.name,
                )
            })
    }

    pub(crate) fn all_bin_targets_sorted(&self) -> Vec<&cm::Target> {
        self.targets
            .iter()
            .filter(|cm::Target { kind, .. }| *kind == ["bin".to_owned()])
            .sorted_by(|t1, t2| t1.name.cmp(&t2.name))
            .collect()
    }
}

pub(crate) fn locate_project(cwd: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let cwd = cwd.as_ref();

    cwd.ancestors()
        .map(|p| p.join("Cargo.toml"))
        .find(|p| p.exists())
        .with_context(|| {
            format!(
                "could not find `Cargo.toml` in `{}` or any parent directory. first, run \
                 `cargo compete init` and `cd` to a workspace",
                cwd.display(),
            )
        })
}

pub(crate) fn cargo_metadata(
    manifest_path: impl AsRef<Path>,
    cwd: impl AsRef<Path>,
) -> cm::Result<cm::Metadata> {
    cm::MetadataCommand::new()
        .manifest_path(manifest_path.as_ref())
        .current_dir(cwd.as_ref())
        .exec()
}

pub(crate) fn cargo_metadata_no_deps(
    manifest_path: impl AsRef<Path>,
    cwd: impl AsRef<Path>,
) -> cm::Result<cm::Metadata> {
    cm::MetadataCommand::new()
        .manifest_path(manifest_path.as_ref())
        .no_deps()
        .current_dir(cwd.as_ref())
        .exec()
}

pub(crate) fn set_cargo_config_build_target_dir(
    dir: &Path,
    shell: &mut Shell,
) -> anyhow::Result<()> {
    crate::fs::create_dir_all(dir.join(".cargo"))?;

    let cargo_config_path = dir.join(".cargo").join("config.toml");

    let mut cargo_config = if cargo_config_path.exists() {
        crate::fs::read_to_string(&cargo_config_path)?
    } else {
        r#"[build]
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

    if cargo_config.get("build").is_none() {
        let mut tbl = toml_edit::Table::new();
        tbl.set_implicit(true);
        cargo_config["build"] = toml_edit::Item::Table(tbl);
    }
    if { &mut cargo_config["build"]["target-dir"] }.is_none() {
        cargo_config["build"]["target-dir"] = toml_edit::value("target");
        crate::fs::write(&cargo_config_path, cargo_config.to_string())?;
        shell.status("Wrote", cargo_config_path.display())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::project::{PackageMetadataCargoCompete, PackageMetadataCargoCompeteBinExample};
    use indexmap::indexmap;
    use pretty_assertions::assert_eq;
    use toml::toml;

    #[test]
    fn deserialize_package_metadata_cargo_compete() -> anyhow::Result<()> {
        let expected = PackageMetadataCargoCompete {
            config: None,
            bin: indexmap!(
                "practice-a".to_owned() => PackageMetadataCargoCompeteBinExample {
                    alias: "a".to_owned(),
                    problem: "https://atcoder.jp/contests/practice/tasks/practice_1"
                        .parse()
                        .unwrap(),
                },
                "practice-b".to_owned() => PackageMetadataCargoCompeteBinExample {
                    alias: "b".to_owned(),
                    problem: "https://atcoder.jp/contests/practice/tasks/practice_2"
                        .parse()
                        .unwrap(),
                },
            ),
            example: indexmap!(),
        };

        assert_eq!(
            expected,
            toml! {
                [bin]
                practice-a = { alias = "a", problem = "https://atcoder.jp/contests/practice/tasks/practice_1" }
                practice-b = { alias = "b", problem = "https://atcoder.jp/contests/practice/tasks/practice_2" }
            }
            .try_into::<PackageMetadataCargoCompete>()?,
        );

        let expected = PackageMetadataCargoCompete {
            config: None,
            bin: indexmap!(
                "aplusb".to_owned() => PackageMetadataCargoCompeteBinExample {
                    alias: "aplusb".to_owned(),
                    problem: "https://judge.yosupo.jp/problem/aplusb".parse().unwrap(),
                },
            ),
            example: indexmap!(),
        };

        assert_eq!(
            expected,
            toml! {
                [bin]
                aplusb = { problem = "https://judge.yosupo.jp/problem/aplusb" }
            }
            .try_into::<PackageMetadataCargoCompete>()?,
        );

        let expected = PackageMetadataCargoCompete {
            config: None,
            bin: indexmap!(),
            example: indexmap!(
                "aplusb".to_owned() => PackageMetadataCargoCompeteBinExample {
                    alias: "aplusb".to_owned(),
                    problem: "https://judge.yosupo.jp/problem/aplusb".parse().unwrap(),
                },
            ),
        };

        assert_eq!(
            expected,
            toml! {
                [example]
                aplusb = { problem = "https://judge.yosupo.jp/problem/aplusb" }
            }
            .try_into::<PackageMetadataCargoCompete>()?,
        );

        Ok(())
    }
}
