use crate::shell::Shell;
use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, path::PathBuf};

pub(crate) fn cookies_path() -> anyhow::Result<PathBuf> {
    let data_local_dir =
        dirs::data_local_dir().with_context(|| "could not find the local data directory")?;

    Ok(data_local_dir.join("cargo-compete").join("cookies.jsonl"))
}

pub(crate) fn username_and_password<'a>(
    shell: &'a RefCell<&'a mut Shell>,
    username_prompt: &'static str,
    password_prompt: &'static str,
) -> impl 'a + FnMut() -> anyhow::Result<(String, String)> {
    move || -> _ {
        let mut shell = shell.borrow_mut();
        let username = shell.read_reply(username_prompt)?;
        let password = shell.read_password(password_prompt)?;
        Ok((username, password))
    }
}

pub(crate) fn dropbox_access_token() -> anyhow::Result<String> {
    let path = token_path("dropbox.json")?;

    let DropboxJson { access_token } = crate::fs::read_json(&path)
        .with_context(|| format!("First, save the access token to `{}`", path.display()))?;

    return Ok(access_token);

    #[derive(Deserialize)]
    struct DropboxJson {
        access_token: String,
    }
}

pub(crate) fn yukicoder_api_key(shell: &mut Shell) -> anyhow::Result<String> {
    let path = token_path("yukicoder.json")?;
    if path.exists() {
        crate::fs::read_json(path)
    } else {
        let api_key = shell.read_password("yukicoder API key: ")?;
        crate::fs::create_dir_all(path.parent().unwrap())?;
        crate::fs::write_json(path, &api_key)?;
        Ok(api_key)
    }
}

pub(crate) fn codeforces_api_key_and_secret(shell: &mut Shell) -> anyhow::Result<(String, String)> {
    let path = token_path("codeforces.json")?;

    let CodeforcesJson {
        api_key,
        api_secret,
    } = if path.exists() {
        crate::fs::read_json(path)?
    } else {
        let api_key = shell.read_password("Codeforces API key: ")?;
        let api_secret = shell.read_password("Codeforces API secret: ")?;
        let content = CodeforcesJson {
            api_key,
            api_secret,
        };
        crate::fs::create_dir_all(path.parent().unwrap())?;
        crate::fs::write_json(path, &content)?;
        content
    };

    return Ok((api_key, api_secret));

    #[derive(Deserialize, Serialize)]
    struct CodeforcesJson {
        api_key: String,
        api_secret: String,
    }
}

fn token_path(file_name: &str) -> anyhow::Result<PathBuf> {
    let data_local_dir =
        dirs::data_local_dir().with_context(|| "could not find the local data directory")?;

    Ok(data_local_dir
        .join("cargo-compete")
        .join("tokens")
        .join(file_name))
}
