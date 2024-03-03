pub(crate) mod credentials;
pub(crate) mod retrieve_testcases;
pub(crate) mod url;

use std::time::Duration;

pub(crate) const TIMEOUT: Option<Duration> = Some(Duration::from_secs(30));

pub(crate) static ATCODER_RUST_LANG_ID: &str = "5054";
pub(crate) static CODEFORCES_RUST_LANG_ID: &str = "75";
pub(crate) static YUKICODER_RUST_LANG_ID: &str = "rust";
