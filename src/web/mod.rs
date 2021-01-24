pub(crate) mod credentials;
pub(crate) mod retrieve_testcases;
pub(crate) mod url;

use std::time::Duration;

pub(crate) const TIMEOUT: Option<Duration> = Some(Duration::from_secs(30));
