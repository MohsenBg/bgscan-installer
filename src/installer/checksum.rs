use std::path::Path;

/// Release checksum file name. It lives at the same base URL as the zips,
/// e.g. `.../releases/download/v2.10.0/checksum.txt`.
pub const CHECKSUM_FILE: &str = "checksum.txt";

/// Parses a `sha256sum`-style checksum file and returns the expected hex
/// digest for `asset` (e.g. `bgscan-linux-64.zip`).
///
/// Accepts both `"<hash>  <file>"` and `"<hash> *<file>"` forms, ignores
/// blank lines, and matches on the file basename.
pub fn parse_checksum(content: &str, asset: &str) -> Option<String> {
    let wanted = asset.rsplit(['/', '\\']).next().unwrap_or(asset);

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let file = parts.next()?;
        // A third token would mean a malformed line (hashes have no spaces).
        if parts.next().is_some() {
            continue;
        }

        let file = file.strip_prefix('*').unwrap_or(file);
        let candidate = file.rsplit(['/', '\\']).next().unwrap_or(file);
        if candidate.eq_ignore_ascii_case(wanted) {
            return Some(hash.to_lowercase());
        }
    }

    None
}

/// Reads a downloaded checksum file and extracts the expected hex digest.
pub fn parse_checksum_file(path: &Path, asset: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    parse_checksum(&content, asset)
        .ok_or_else(|| format!("checksum for '{asset}' not found in {}", path.display()).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CHECKSUMS: &str = "49bb50210cca2e5f08b9a0f8e71437da3a3ffc1c5d01deb571923b88c5466eb5  bgscan-android-arm64-v8a.zip\n996d8b9e4422314cae6b2ca195ac8d05e15bd5e6078bf0f59b2e144a17ab54f6  bgscan-linux-64.zip\n";

    #[test]
    fn parses_checksum_for_asset() {
        assert_eq!(
            parse_checksum(SAMPLE_CHECKSUMS, "bgscan-linux-64.zip"),
            Some("996d8b9e4422314cae6b2ca195ac8d05e15bd5e6078bf0f59b2e144a17ab54f6".to_string())
        );
    }

    #[test]
    fn parses_checksum_missing_asset_returns_none() {
        assert_eq!(parse_checksum(SAMPLE_CHECKSUMS, "bgscan-nope.zip"), None);
    }

    #[test]
    fn parses_checksum_binary_marker_form() {
        let content = "d41d8cd98f00b204e9800998ecf8427e  *bgscan-linux-64.zip\n";
        assert_eq!(
            parse_checksum(content, "bgscan-linux-64.zip"),
            Some("d41d8cd98f00b204e9800998ecf8427e".to_string())
        );
    }

    #[test]
    fn parse_checksum_file_reads_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("checksum.txt");
        std::fs::write(&path, SAMPLE_CHECKSUMS).unwrap();

        assert_eq!(
            parse_checksum_file(&path, "bgscan-linux-64.zip").unwrap(),
            "996d8b9e4422314cae6b2ca195ac8d05e15bd5e6078bf0f59b2e144a17ab54f6"
        );
        assert!(parse_checksum_file(&path, "missing.zip").is_err());
    }
}
