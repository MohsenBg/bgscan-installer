use reqwest::blocking::Client;
use serde::Deserialize;
use std::error::Error;

use super::platform::PlatformInfo;

pub const GITHUB_OWNER_REPO: &str = "MohsenBg/bgscan";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
}

/// Normalizes user input into a release tag (`2.10.0` -> `v2.10.0`, `v2.10.0` stays).
pub fn normalize_tag(version: &str) -> String {
    let version = version.trim();
    if version.starts_with('v') || version.starts_with('V') {
        version.to_string()
    } else {
        format!("v{version}")
    }
}

pub fn is_latest(version: &str) -> bool {
    version.trim().eq_ignore_ascii_case("latest")
}

/// Base download URL for a version:
/// - `"latest"` -> `.../releases/latest/download/`
/// - `"2.10.0"` / `"v2.10.0"` -> `.../releases/download/v2.10.0/`
pub fn release_base_url(version: &str) -> String {
    if is_latest(version) {
        format!("https://github.com/{GITHUB_OWNER_REPO}/releases/latest/download/")
    } else {
        let tag = normalize_tag(version);
        format!("https://github.com/{GITHUB_OWNER_REPO}/releases/download/{tag}/")
    }
}

/// Full download URL for a platform + version. `None` when no asset exists.
pub fn download_url(info: &PlatformInfo, version: &str) -> Option<String> {
    let asset = info.asset_name()?;
    Some(format!("{}{asset}", release_base_url(version)))
}

/// URL of the `checksum.txt` for a (resolved) version. Same base URL as the zips.
pub fn checksum_url_for(resolved_version: &str) -> String {
    format!("{}checksum.txt", release_base_url(resolved_version))
}

fn github_client() -> Result<Client, Box<dyn Error>> {
    Ok(Client::builder().user_agent("bgscan-installer").build()?)
}

/// Fetches the real tag behind `latest` (e.g. `v2.10.0`).
fn latest_version() -> Result<String, Box<dyn Error>> {
    let release: Release = github_client()?
        .get(format!(
            "https://api.github.com/repos/{GITHUB_OWNER_REPO}/releases/latest"
        ))
        .send()?
        .error_for_status()?
        .json()?;

    Ok(release.tag_name)
}

/// Verifies a pinned version exists and returns its canonical tag.
fn verify_version_exists(version: &str) -> Result<String, Box<dyn Error>> {
    let tag = normalize_tag(version);
    let url = format!("https://api.github.com/repos/{GITHUB_OWNER_REPO}/releases/tags/{tag}");

    let response = github_client()?.get(&url).send()?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(format!(
            "version '{version}' (tag '{tag}') does not exist. Check available releases at https://github.com/{GITHUB_OWNER_REPO}/releases"
        )
        .into());
    }

    let release: Release = response.error_for_status()?.json()?;
    Ok(release.tag_name)
}

/// Resolves user input into a concrete tag:
/// - `"latest"` -> queries the API for the real latest tag
/// - `"2.10.0"` / `"v2.10.0"` -> verifies the tag exists, returns canonical tag
pub fn resolve_version(version: &str) -> Result<String, Box<dyn Error>> {
    if is_latest(version) {
        latest_version().map_err(|e| format!("failed to resolve 'latest' release: {e}").into())
    } else {
        verify_version_exists(version)
            .map_err(|e| format!("invalid version '{version}': {e}").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::installer::platform::{Arch, Platform};

    #[test]
    fn normalizes_missing_v_prefix() {
        assert_eq!(normalize_tag("2.10.0"), "v2.10.0");
        assert_eq!(normalize_tag("v2.10.0"), "v2.10.0");
        assert_eq!(normalize_tag("  2.10.0  "), "v2.10.0");
    }

    #[test]
    fn detects_latest_case_insensitive() {
        assert!(is_latest("latest"));
        assert!(is_latest("Latest"));
        assert!(!is_latest("v2.10.0"));
    }

    #[test]
    fn builds_expected_urls() {
        assert_eq!(
            release_base_url("latest"),
            "https://github.com/MohsenBg/bgscan/releases/latest/download/"
        );
        assert_eq!(
            release_base_url("2.10.0"),
            "https://github.com/MohsenBg/bgscan/releases/download/v2.10.0/"
        );
    }

    #[test]
    fn download_url_appends_asset() {
        let info = PlatformInfo::new(Platform::Linux, Arch::Amd64);
        assert_eq!(
            download_url(&info, "v2.10.0"),
            Some(
                "https://github.com/MohsenBg/bgscan/releases/download/v2.10.0/bgscan-linux-64.zip"
                    .to_string()
            )
        );
    }

    #[test]
    fn checksum_url_uses_same_base() {
        assert_eq!(
            checksum_url_for("v2.10.0"),
            "https://github.com/MohsenBg/bgscan/releases/download/v2.10.0/checksum.txt"
        );
    }
}
