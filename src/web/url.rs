use anyhow::{anyhow, Context as _};
use std::borrow::Cow;
use url::Url;

pub(crate) fn atcoder_contest(url: &Url) -> anyhow::Result<String> {
    second_path_segment(url)
}

pub(crate) fn codeforces_contest(url: &Url) -> anyhow::Result<String> {
    second_path_segment(url)
}

pub(crate) fn yukicoder_contest(url: &Url) -> anyhow::Result<String> {
    second_path_segment(url)
}

fn second_path_segment(url: &Url) -> anyhow::Result<String> {
    let segments = url
        .path_segments()
        .map(|ss| ss.collect::<Vec<_>>())
        .unwrap_or_default();

    let segment = segments.get(1).with_context(|| {
        format!(
            "the number of path segments is {} but the index is 1: {}",
            segments.len(),
            url,
        )
    })?;

    let decodor = || percent_encoding::percent_decode_str(segment);

    decodor()
        .decode_utf8()
        .map(Cow::into_owned)
        .map_err(|e| anyhow!("{}: {}", e, decodor().decode_utf8_lossy()))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    #[test]
    fn atcoder_contest() -> anyhow::Result<()> {
        assert_eq!(
            "m-solutions2020",
            super::atcoder_contest(
                &"https://atcoder.jp/contests/m-solutions2020"
                    .parse()
                    .unwrap(),
            )?
        );
        Ok(())
    }
}
