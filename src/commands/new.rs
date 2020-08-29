use crate::{
    config::{
        CargoCompeteConfig, CargoCompeteConfigNewTemplateDependencies,
        CargoCompeteConfigNewTemplateSrc,
    },
    shell::{ColorChoice, Shell},
};
use anyhow::{anyhow, Context as _};
use heck::KebabCase as _;
use itertools::Itertools as _;
use liquid::object;
use snowchains_core::web::{
    PlatformKind, RetrieveTestCasesOutcome, RetrieveTestCasesOutcomeContest,
    RetrieveTestCasesOutcomeProblem,
};
use std::{
    collections::BTreeMap,
    iter,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use strum::VariantNames as _;
use url::Url;

#[derive(StructOpt, Debug)]
pub struct OptCompeteNew {
    /// Retrieve system test cases
    #[structopt(long)]
    pub full: bool,

    /// Open URLs and files
    #[structopt(long)]
    pub open: bool,

    /// Retrieve only the problems
    #[structopt(long, value_name("INDEX"))]
    pub problems: Option<Vec<String>>,

    /// Path to `compete.toml`
    #[structopt(long, value_name("PATH"))]
    pub config: Option<PathBuf>,

    /// Coloring
    #[structopt(
        long,
        value_name("WHEN"),
        possible_values(ColorChoice::VARIANTS),
        default_value("auto")
    )]
    pub color: ColorChoice,

    /// Contest ID. Required for some platforms
    pub contest: Option<String>,
}

pub fn run(opt: OptCompeteNew, ctx: crate::Context<'_>) -> anyhow::Result<()> {
    let OptCompeteNew {
        full,
        open,
        problems,
        config,
        color,
        contest,
    } = opt;

    let crate::Context {
        cwd,
        cookies_path,
        shell,
    } = ctx;

    shell.set_color_choice(color);

    let cargo_compete_config_path = crate::config::locate(cwd, config)?;
    let cargo_compete_dir = cargo_compete_config_path.with_file_name("");
    let cargo_compete_config = crate::config::load(&cargo_compete_config_path)?;

    match cargo_compete_config.new.platform {
        PlatformKind::Atcoder => {
            let contest = contest.with_context(|| "`contest` is required for AtCoder")?;
            let problems = problems.map(|ps| ps.into_iter().collect());

            let outcome = crate::web::retrieve_testcases::dl_from_atcoder(
                &contest,
                problems,
                full,
                &cookies_path,
                shell,
            )?;

            let package_name = outcome
                .contest
                .as_ref()
                .map(|RetrieveTestCasesOutcomeContest { id, .. }| id)
                .unwrap_or(&contest);

            let problems = outcome
                .problems
                .iter()
                .map(|RetrieveTestCasesOutcomeProblem { index, url, .. }| (&**index, url))
                .collect();

            let urls = urls(&outcome);

            let (manifest_dir, src_paths) = create_new_package(
                &cargo_compete_config_path,
                &cargo_compete_config,
                package_name,
                &problems,
                false,
                shell,
            )?;

            let file_paths = itertools::zip_eq(
                src_paths,
                crate::web::retrieve_testcases::save_test_cases(
                    &cargo_compete_dir,
                    manifest_dir.to_str().with_context(|| "invalid utf-8")?,
                    &cargo_compete_config.test_suite,
                    outcome,
                    shell,
                )?,
            )
            .collect::<Vec<_>>();

            if open {
                crate::open::open(
                    &urls,
                    cargo_compete_config.open,
                    &file_paths,
                    &manifest_dir,
                    &cargo_compete_dir,
                    shell,
                )?;
            }
        }
        PlatformKind::Codeforces => {
            let contest = contest.with_context(|| "`contest` is required for Codeforces")?;
            let problems = problems.map(|ps| ps.into_iter().collect());

            let outcome = crate::web::retrieve_testcases::dl_from_codeforces(
                &contest,
                problems,
                &cookies_path,
                shell,
            )?;

            let package_name = outcome
                .contest
                .as_ref()
                .map(|RetrieveTestCasesOutcomeContest { id, .. }| id)
                .unwrap_or(&contest);

            let problems = outcome
                .problems
                .iter()
                .map(|RetrieveTestCasesOutcomeProblem { index, url, .. }| (&**index, url))
                .collect();

            let urls = urls(&outcome);

            let (manifest_dir, src_paths) = create_new_package(
                &cargo_compete_config_path,
                &cargo_compete_config,
                package_name,
                &problems,
                false,
                shell,
            )?;

            let file_paths = itertools::zip_eq(
                src_paths,
                crate::web::retrieve_testcases::save_test_cases(
                    &cargo_compete_dir,
                    manifest_dir.to_str().with_context(|| "invalid utf-8")?,
                    &cargo_compete_config.test_suite,
                    outcome,
                    shell,
                )?,
            )
            .collect::<Vec<_>>();

            if open {
                crate::open::open(
                    &urls,
                    cargo_compete_config.open,
                    &file_paths,
                    &manifest_dir,
                    &cargo_compete_dir,
                    shell,
                )?;
            }
        }
        PlatformKind::Yukicoder => {
            let contest = contest.as_deref();
            let problems = problems.map(|ps| ps.into_iter().collect());

            let outcome =
                crate::web::retrieve_testcases::dl_from_yukicoder(contest, problems, full, shell)?;

            let package_name = outcome
                .contest
                .as_ref()
                .map(|RetrieveTestCasesOutcomeContest { id, .. }| &**id)
                .or(contest);
            let is_no = package_name.is_none();
            let package_name = package_name.unwrap_or("problems");

            let problems = outcome
                .problems
                .iter()
                .map(|RetrieveTestCasesOutcomeProblem { index, url, .. }| (&**index, url))
                .collect();

            let urls = urls(&outcome);

            let (manifest_dir, src_paths) = create_new_package(
                &cargo_compete_config_path,
                &cargo_compete_config,
                package_name,
                &problems,
                is_no,
                shell,
            )?;

            let file_paths = itertools::zip_eq(
                src_paths,
                crate::web::retrieve_testcases::save_test_cases(
                    &cargo_compete_dir,
                    manifest_dir.to_str().with_context(|| "invalid utf-8")?,
                    &cargo_compete_config.test_suite,
                    outcome,
                    shell,
                )?,
            )
            .collect::<Vec<_>>();

            if open {
                crate::open::open(
                    &urls,
                    cargo_compete_config.open,
                    &file_paths,
                    &manifest_dir,
                    &cargo_compete_dir,
                    shell,
                )?;
            }
        }
    }
    Ok(())
}

fn urls(outcome: &RetrieveTestCasesOutcome) -> Vec<Url> {
    outcome.problems.iter().map(|p| p.url.clone()).collect()
}

fn create_new_package(
    cargo_compete_config_path: &Path,
    cargo_compete_config: &CargoCompeteConfig,
    package_name: &str,
    problems: &BTreeMap<&str, &Url>,
    problems_are_yukicoder_no: bool,
    shell: &mut Shell,
) -> anyhow::Result<(PathBuf, Vec<PathBuf>)> {
    crate::process::process(crate::process::cargo_exe()?)
        .args(&[
            "new",
            "-q",
            "--vcs",
            "none",
            "--name",
            package_name,
            package_name,
        ])
        .cwd(cargo_compete_config_path.with_file_name(""))
        .exec()?;

    let cargo_compete_config_dir = cargo_compete_config_path.with_file_name("");

    let manifest_dir = cargo_compete_config.new.path.render(&object!({
        "package_name": package_name,
    }))?;
    let manifest_dir = Path::new(&manifest_dir);
    let manifest_dir = cargo_compete_config_path
        .with_file_name(".")
        .join(manifest_dir.strip_prefix(".").unwrap_or(manifest_dir));

    let manifest_path = manifest_dir.join("Cargo.toml");

    let mut package_metadata_cargo_compete_bin = problems
        .keys()
        .map(|problem_index| {
            format!(
                r#"{} = {{ name = "", problem = {{ {} }} }}
"#,
                escape_key(&problem_index.to_kebab_case()),
                match (cargo_compete_config.new.platform, problems_are_yukicoder_no) {
                    (PlatformKind::Atcoder, _) | (PlatformKind::Codeforces, _) => {
                        r#"platform = "", contest = "", index = "", url = """#
                    }
                    (PlatformKind::Yukicoder, true) => {
                        r#"platform = "", kind = "no", no = "", url = """#
                    }
                    (PlatformKind::Yukicoder, false) => {
                        r#"platform = "", kind = "contest", contest = "", index = "", url = """#
                    }
                }
            )
        })
        .join("")
        .parse::<toml_edit::Document>()?;

    for (problem_index, problem_url) in problems {
        package_metadata_cargo_compete_bin[&problem_index.to_kebab_case()]["name"] =
            toml_edit::value(format!(
                "{}-{}",
                package_name,
                problem_index.to_kebab_case(),
            ));

        let tbl =
            &mut package_metadata_cargo_compete_bin[&problem_index.to_kebab_case()]["problem"];

        match cargo_compete_config.new.platform {
            PlatformKind::Atcoder => {
                tbl["platform"] = toml_edit::value("atcoder");
                tbl["contest"] = toml_edit::value(package_name);
                tbl["index"] = toml_edit::value(&**problem_index);
                tbl["url"] = toml_edit::value(problem_url.as_str());
            }
            PlatformKind::Codeforces => {
                tbl["platform"] = toml_edit::value("codeforces");
                tbl["contest"] = toml_edit::value(package_name);
                tbl["index"] = toml_edit::value(&**problem_index);
                tbl["url"] = toml_edit::value(problem_url.as_str());
            }
            PlatformKind::Yukicoder => {
                tbl["platform"] = toml_edit::value("yukicoder");
                if problems_are_yukicoder_no {
                    tbl["no"] = toml_edit::value(&**problem_index);
                } else {
                    tbl["contest"] = toml_edit::value(package_name);
                    tbl["index"] = toml_edit::value(&**problem_index);
                }
                tbl["url"] = toml_edit::value(problem_url.as_str());
            }
        }
    }

    let bin = toml_edit::Item::ArrayOfTables({
        let mut arr = toml_edit::ArrayOfTables::new();
        for problem_index in problems.keys() {
            let mut tbl = toml_edit::Table::new();
            tbl["name"] = toml_edit::value(format!(
                "{}-{}",
                package_name,
                problem_index.to_kebab_case(),
            ));
            tbl["path"] = toml_edit::value(format!("src/bin/{}.rs", problem_index.to_kebab_case()));
            arr.append(tbl);
        }
        arr
    });

    let dependencies = match &cargo_compete_config.new.template.dependencies {
        CargoCompeteConfigNewTemplateDependencies::Inline { content } => {
            content
                .parse::<toml_edit::Document>()
                .with_context(|| {
                    "could not parse the toml value in `new.template.dependencies.content`"
                })?
                .root
        }
        CargoCompeteConfigNewTemplateDependencies::ManifestFile { path } => {
            crate::fs::read_to_string(cargo_compete_config_path.with_file_name("").join(path))?
                .parse::<toml_edit::Document>()?["dependencies"]
                .clone()
        }
    };

    static DEFAULT_MANIFEST_END: &str = r"
# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
";

    let mut manifest = crate::fs::read_to_string(&manifest_path)?;
    if manifest.ends_with(DEFAULT_MANIFEST_END) {
        manifest = manifest.replace(
            DEFAULT_MANIFEST_END,
            r"
[bin]

[dependencies]
",
        );
    }
    let mut manifest = manifest.parse::<toml_edit::Document>()?;

    set_implicit_table_if_none(&mut manifest["package"]["metadata"]);
    set_implicit_table_if_none(&mut manifest["package"]["metadata"]["cargo-compete"]);
    set_implicit_table_if_none(&mut manifest["package"]["metadata"]["cargo-compete"]["bin"]);

    manifest["package"]["metadata"]["cargo-compete"]["config"] = toml_edit::value({
        if let Ok(rel_manifest_dir) = manifest_dir.strip_prefix(&cargo_compete_config_dir) {
            rel_manifest_dir
                .iter()
                .map(|_| "..")
                .chain(iter::once("compete.toml"))
                .join("/")
        } else {
            manifest_dir
                .clone()
                .into_os_string()
                .into_string()
                .map_err(|s| anyhow!("invalid utf-8 path: {:?}", s))?
        }
    });

    for (key, val) in package_metadata_cargo_compete_bin.as_table().iter() {
        manifest["package"]["metadata"]["cargo-compete"]["bin"][key] = val.clone();
    }

    manifest["bin"] = bin;
    manifest["dependencies"] = dependencies;

    if let Ok(new_manifest) = manifest
        .to_string()
        .replace("\"} }", "\" } }")
        .parse::<toml_edit::Document>()
    {
        manifest = new_manifest;
    }

    crate::fs::write(&manifest_path, manifest.to_string_in_original_order())?;

    let src = match &cargo_compete_config.new.template.src {
        CargoCompeteConfigNewTemplateSrc::Inline { content } => content.clone(),
        CargoCompeteConfigNewTemplateSrc::File { path } => {
            crate::fs::read_to_string(cargo_compete_config_path.with_file_name("").join(path))?
        }
    };

    let src_bin_dir = manifest_dir.join("src").join("bin");

    crate::fs::create_dir_all(&src_bin_dir)?;

    let src_paths = problems
        .keys()
        .map(|problem_index| {
            src_bin_dir
                .join(problem_index.to_kebab_case())
                .with_extension("rs")
        })
        .collect::<Vec<_>>();

    for src_path in &src_paths {
        crate::fs::write(src_path, &src)?;
    }
    crate::fs::remove_file(manifest_dir.join("src").join("main.rs"))?;

    shell.status(
        "Created",
        format!("`{}` package at {}", package_name, manifest_dir.display()),
    )?;

    Ok((manifest_dir, src_paths))
}

fn escape_key(s: &str) -> String {
    if s.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return toml::Value::String(s.to_owned()).to_string();
    }

    let mut doc = toml_edit::Document::new();
    doc[s] = toml_edit::value(0);
    doc.to_string()
        .trim_end()
        .trim_end_matches('0')
        .trim_end()
        .trim_end_matches('=')
        .trim_end()
        .to_owned()
}

fn set_implicit_table_if_none(item: &mut toml_edit::Item) {
    if item.is_none() {
        *item = {
            let mut tbl = toml_edit::Table::new();
            tbl.set_implicit(true);
            toml_edit::Item::Table(tbl)
        };
    }
}
