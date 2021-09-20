use crate::{
    config::{BinLikeTargetKind, CargoCompeteConfigAdd},
    oj_api,
    project::{MetadataExt as _, PackageExt as _},
    shell::ColorChoice,
};
use anyhow::{anyhow, bail, ensure, Context as _};
use cargo_metadata as cm;
use liquid::object;
use maplit::{btreeset, hashmap};
use once_cell::sync::Lazy;
use snowchains_core::web::{PlatformKind, ProblemsInContest, YukicoderRetrieveTestCasesTargets};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use strum::VariantNames as _;
use url::Url;

#[derive(StructOpt, Debug)]
pub struct OptCompeteAdd {
    /// Retrieve system test cases
    #[structopt(long)]
    pub full: bool,

    /// Open URLs and files
    #[structopt(long)]
    pub open: bool,

    /// Package (see `cargo help pkgid`)
    #[structopt(short, long, value_name("SPEC"))]
    pub package: Option<String>,

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

    /// Arguments
    pub args: Vec<String>,
}

pub(crate) fn run(opt: OptCompeteAdd, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteAdd {
        full,
        open,
        package,
        manifest_path,
        color,
        args,
    } = opt;

    let crate::Context {
        cwd,
        cookies_path,
        shell,
    } = ctx;

    shell.set_color_choice(color);

    let manifest_path = manifest_path
        .map(|p| Ok(cwd.join(p.strip_prefix(".").unwrap_or(&p))))
        .unwrap_or_else(|| crate::project::locate_project(&cwd))?;
    let metadata = crate::project::cargo_metadata(&manifest_path, cwd)?;
    let member = metadata.query_for_member(package.as_deref())?;
    let (cargo_compete_config, cargo_compete_config_path) =
        crate::config::load_for_package(member, shell)?;
    let src_content = &cargo_compete_config
        .template(&cargo_compete_config_path, shell)?
        .src;
    let cargo_compete_config_add = cargo_compete_config
        .add
        .as_ref()
        .with_context(|| "`add` field is required for this command")?;

    let url = cargo_compete_config_add
        .url
        .render(&object!({ "args": &args }))?;
    ensure!(!url.is_empty(), "empty URL for {:?}", args);
    let url = url
        .parse::<Url>()
        .with_context(|| format!("could not parse {:?} as a URL", url))?;

    let is_contest = if let Some(args) = &cargo_compete_config_add.is_contest {
        ensure!(!args.is_empty(), "`add.is-contest` is empty");
        crate::process::process(&args[0])
            .args(&args[1..])
            .pipe_input(Some(url.as_str()))
            .cwd(&metadata.workspace_root)
            .status()?
            .success()
    } else {
        false
    };

    let problems = match PlatformKind::from_url(&url) {
        Ok(PlatformKind::Atcoder) => crate::web::retrieve_testcases::dl_from_atcoder(
            if is_contest {
                ProblemsInContest::Indexes {
                    contest: crate::web::url::atcoder_contest(&url)?,
                    problems: None,
                }
            } else {
                ProblemsInContest::Urls {
                    urls: btreeset!(url),
                }
            },
            full,
            &cookies_path,
            shell,
        )?
        .into_iter()
        .map(crate::web::retrieve_testcases::Problem::<Option<String>>::from)
        .collect::<Vec<_>>(),

        Ok(PlatformKind::Codeforces) => crate::web::retrieve_testcases::dl_from_codeforces(
            if is_contest {
                ProblemsInContest::Indexes {
                    contest: crate::web::url::codeforces_contest(&url)?,
                    problems: None,
                }
            } else {
                ProblemsInContest::Urls {
                    urls: btreeset!(url),
                }
            },
            &cookies_path,
            shell,
        )?
        .into_iter()
        .map(crate::web::retrieve_testcases::Problem::<Option<String>>::from)
        .collect::<Vec<_>>(),

        Ok(PlatformKind::Yukicoder) => crate::web::retrieve_testcases::dl_from_yukicoder(
            if is_contest {
                YukicoderRetrieveTestCasesTargets::Contest(
                    crate::web::url::codeforces_contest(&url)?,
                    None,
                )
            } else {
                YukicoderRetrieveTestCasesTargets::Urls(btreeset!(url))
            },
            full,
            shell,
        )?
        .into_iter()
        .map(crate::web::retrieve_testcases::Problem::<Option<String>>::from)
        .collect::<Vec<_>>(),

        Err(_) => if is_contest {
            oj_api::get_contest(&url, &metadata.workspace_root, shell)?
        } else {
            vec![url]
        }
        .iter()
        .map(|url| {
            let problem = oj_api::get_problem(url, full, &metadata.workspace_root, shell)?;
            Ok(crate::web::retrieve_testcases::Problem::from_oj_api(
                problem, full,
            ))
        })
        .collect::<anyhow::Result<_>>()?,
    };

    let manifest =
        &mut crate::fs::read_to_string(&member.manifest_path)?.parse::<toml_edit::Document>()?;

    let mut abs_bin_src_paths = vec![];
    let mut urls_to_open = vec![];
    let mut bin_names_by_url = hashmap!();
    let mut bin_aliases_by_url = hashmap!();

    for problem in &problems {
        let CargoCompeteConfigAdd {
            target_kind,
            bin_name,
            bin_alias,
            bin_src_path,
            ..
        } = cargo_compete_config_add;

        let bin_name = &*bin_name.render(&object!({
            "args": &args,
            "url": &problem.url,
        }))?;
        let bin_alias = &*bin_alias.render(&object!({
            "args": &args,
            "url": &problem.url,
            "bin_name": bin_name,
        }))?;
        let bin_src_path = &*bin_src_path
            .as_ref()
            .unwrap_or_else(|| {
                return match *target_kind {
                    BinLikeTargetKind::Bin => &DEFAULT_BIN_PATH,
                    BinLikeTargetKind::ExampleBin => &DEFAULT_EXAMPLE_PATH,
                };

                static DEFAULT_BIN_PATH: Lazy<liquid::Template> =
                    Lazy::new(|| parse("src/bin/{{ bin_alias }}.rs"));
                static DEFAULT_EXAMPLE_PATH: Lazy<liquid::Template> =
                    Lazy::new(|| parse("examples/{{ bin_alias }}.rs"));

                fn parse(template: &'static str) -> liquid::Template {
                    liquid::ParserBuilder::with_stdlib()
                        .build()
                        .unwrap()
                        .parse(template)
                        .unwrap()
                }
            })
            .render(&object!({
                "args": &args,
                "url": &problem.url,
                "bin_name": bin_name,
                "bin_alias": bin_alias,
            }))?;

        if member
            .all_bin_targets_sorted()
            .iter()
            .any(|cm::Target { name, .. }| name == bin_name)
        {
            bail!("binary `{}` already exists", bin_name);
        }

        let abs_bin_src_path = member.manifest_path.with_file_name("").join(bin_src_path);
        crate::fs::create_dir_all(abs_bin_src_path.with_file_name(""))?;
        crate::fs::write(&abs_bin_src_path, src_content)?;
        abs_bin_src_paths.push(abs_bin_src_path);
        urls_to_open.push(problem.url.clone());
        bin_names_by_url.insert(problem.url.clone(), bin_name.to_owned());
        bin_aliases_by_url.insert(problem.url.clone(), bin_alias.to_owned());

        let target_kind = match cargo_compete_config_add.target_kind {
            BinLikeTargetKind::Bin => "bin",
            BinLikeTargetKind::ExampleBin => "example",
        };

        let entry = &mut manifest["package"]["metadata"]["cargo-compete"][target_kind];
        if bin_name != bin_alias {
            entry[bin_name]["alias"] = toml_edit::value(bin_alias);
        }
        entry[bin_name]["problem"] = toml_edit::value(problem.url.as_str());

        let default_src_path = Path::new("src")
            .join("bin")
            .join(&bin_name)
            .with_extension("rs");

        if Path::new(bin_src_path)
            .strip_prefix(".")
            .unwrap_or_else(|_| bin_src_path.as_ref())
            != default_src_path
        {
            if let Some(bin) = manifest[target_kind].as_array_mut() {
                let mut tbl = toml_edit::InlineTable::default();
                tbl.get_or_insert("name", bin_name);
                tbl.get_or_insert("path", bin_src_path);
                bin.push(tbl)
                    .map_err(|_| anyhow!("could not add an element to `bin`"))?;
            } else {
                let bin = manifest[target_kind].or_insert(toml_edit::Item::ArrayOfTables(
                    toml_edit::ArrayOfTables::new(),
                ));
                if let Some(bin) = bin.as_array_of_tables_mut() {
                    let mut tbl = toml_edit::Table::new();
                    tbl["name"] = toml_edit::value(bin_name);
                    tbl["path"] = toml_edit::value(bin_src_path);
                    bin.append(tbl);
                }
            }
        }

        shell.status(
            "Added",
            format!("`{}` ({}) for {}", bin_name, target_kind, problem.url),
        )?;
    }

    crate::fs::write(&member.manifest_path, manifest.to_string())?;

    let file_paths = itertools::zip_eq(
        &abs_bin_src_paths,
        crate::web::retrieve_testcases::save_test_cases(
            &metadata.workspace_root,
            member.manifest_dir(),
            &cargo_compete_config.test_suite,
            true,
            problems,
            |url, _| vec![bin_names_by_url[url].clone()],
            |url, _| vec![bin_aliases_by_url[url].clone()],
            shell,
        )?,
    )
    .collect::<Vec<_>>();

    if open {
        crate::open::open(
            &urls_to_open,
            cargo_compete_config.open,
            &file_paths,
            member.manifest_dir(),
            &cargo_compete_config_path.with_file_name(""),
            shell,
        )?;
    }
    Ok(())
}
