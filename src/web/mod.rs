pub(crate) mod credentials;

use std::time::Duration;

pub(crate) const TIMEOUT: Option<Duration> = Some(Duration::from_secs(30));
