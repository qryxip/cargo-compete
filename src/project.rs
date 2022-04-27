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
        manifest_path: impl AsRef<str>,
    ) -> anyhow::Result<(&str, &PackageMetadataCargoCompeteBinExample, bool)> {
        let bin_name_or_alias = name_or_alias.as_ref();

        match *itertools::chain(&self.bin, &self.example)
            .filter_map(|(name, metadata)| {
                let PackageMetadataCargoCompeteBinExample { alias, .. } = metadata;
                let names = [&*name, &**alias];
                let head = sanitize_target_name(bin_name_or_alias);
                if names.contains(&bin_name_or_alias) {
                    Some((name, metadata, false))
                } else if names.contains(&head.unwrap().as_str()) {
                    match add_new_bin(manifest_path.as_ref(), bin_name_or_alias) {
                        Ok(_) => Some((name, metadata, true)),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
        {
            [(k, v, w)] => Ok((k, v, w)),
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
                "unused key in `package.metadata.cargo-compete`: {}",
                unused,
            ))?;
        }

        Ok(ret)
    }

    pub(crate) fn bin_like_target_by_name(
        &self,
        name: impl AsRef<str>,
    ) -> anyhow::Result<&cm::Target> {
        let name = name.as_ref();
        let sanitize_name = sanitize_target_name(name)?;

        self.targets
            .iter()
            .find(|t| {
                t.name == sanitize_name && t.kind == ["bin".to_owned()]
                    || t.kind == ["example".to_owned()]
            })
            .with_context(|| format!("no bin/example target named `{}` in `{}`", name, self.name))
    }

    pub(crate) fn bin_target_by_src_path(
        &self,
        src_path: impl AsRef<Path>,
    ) -> anyhow::Result<&cm::Target> {
        let src_path = src_path.as_ref();
        let sanitize_src_path = sanitize_src(src_path)?;

        self.targets
            .iter()
            .find(|t| t.src_path == sanitize_src_path && t.kind == ["bin".to_owned()])
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

pub(crate) fn sanitize_target_name(target_name: impl AsRef<str>) -> anyhow::Result<String> {
    let target_name = target_name.as_ref();
    Ok(
        match target_name.split('_').collect::<Vec<&str>>().first() {
            Some(&s) => s,
            None => target_name,
        }
        .to_string(),
    )
}

pub(crate) fn sanitize_src(path: impl AsRef<Path>) -> anyhow::Result<String> {
    let path = path.as_ref();
    let (file_name_wo_ext, ext) = (
        path.file_stem()
            .ok_or(anyhow::anyhow!("failed to parse filename"))?
            .to_str()
            .unwrap(),
        path.extension()
            .ok_or(anyhow::anyhow!("failed to parse file extension"))?
            .to_str()
            .unwrap(),
    );
    let sanitized_file_name = sanitize_target_name(file_name_wo_ext)?;
    let dir = path.to_path_buf();
    let new_path = String::from(
        dir.parent()
            .ok_or(anyhow::anyhow!("failed to parse parent"))?
            .to_owned()
            .join(format!("{}.{}", sanitized_file_name, ext))
            .to_str()
            .unwrap(),
    );
    Ok(new_path)
}

pub(crate) fn add_new_bin(
    manifest_path: impl AsRef<str>,
    bin_name_or_alias: impl AsRef<str>,
) -> anyhow::Result<String> {
    let bin_name_or_alias = bin_name_or_alias.as_ref();
    let alias = match bin_name_or_alias.split('-').collect::<Vec<_>>()[..] {
        [_, name] => name,
        [name] => name,
        _ => bail!("unexpeted name or alias `{}`", bin_name_or_alias),
    };
    let mut manifest = crate::fs::read_to_string(manifest_path.as_ref())?
        .parse::<toml_edit::Document>()
        .unwrap();
    let name = format!(
        "{}-{}",
        manifest["package"]["name"]
            .as_str()
            .ok_or(anyhow::anyhow!("failed to parse package name"))?,
        alias
    );

    let bins = manifest["bin"].as_array_of_tables_mut().unwrap();
    let tbl = bins.iter().find(|that_tbl| {
        that_tbl
            .get("name")
            .map_or(false, |that_name| that_name.as_str().unwrap() == name)
    });
    match tbl {
        None => {
            let mut new_tbl = toml_edit::Table::new();
            new_tbl["name"] = toml_edit::value(&name);
            new_tbl["path"] = toml_edit::value(
                ["src", "bin"]
                    .iter()
                    .collect::<PathBuf>()
                    .join(format!("{}.rs", alias))
                    .into_os_string()
                    .into_string()
                    .unwrap(),
            );
            bins.push(new_tbl);
            crate::fs::write(manifest_path.as_ref(), manifest.to_string())?;
            Ok(format!("bin for file={} is created", &name))
        }
        Some(_) => Ok(format!("file={} already exists", &name)),
    }
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

    #[test]
    fn sanitize_str_test() -> anyhow::Result<()> {
        use crate::project::sanitize_src;
        use std::path::PathBuf;

        let dir: PathBuf = ["src", "bin"].iter().collect();
        let act1 = sanitize_src(dir.join("a_with_dfs.rc"))?;
        let act2 = sanitize_src(dir.join("a_ac.rc"))?;
        let expected = dir.join("a.rc").into_os_string().into_string().unwrap();

        assert_eq!(expected, act1);
        assert_eq!(expected, act2);

        Ok(())
    }
}
