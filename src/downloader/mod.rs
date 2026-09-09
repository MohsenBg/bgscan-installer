use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use crate::{net::build_client, progress::Progress};

pub fn download_file(url: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let client = build_client("bgscan-installer")?;

    let mut response = client.get(url).send()?;

    response.error_for_status_ref()?;

    let total = response.content_length().unwrap_or(0);
    let progress = Progress::new(total);

    let mut file = File::create(path)?;

    copy_with_progress(&mut response, &mut file, &progress)?;

    progress.finish();

    Ok(())
}

fn copy_with_progress<R: Read, W: Write>(
    mut reader: R,
    mut writer: W,
    progress: &Progress,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;

        if bytes_read == 0 {
            break;
        }

        writer.write_all(&buffer[..bytes_read])?;
        progress.inc(bytes_read as u64);
    }

    Ok(())
}

fn sha256_hex_of_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// Verifies the SHA-256 of the file at `path` against `expected_hex`.
/// Returns an error showing both digests on mismatch.
pub fn verify_sha256(path: &Path, expected_hex: &str) -> Result<(), Box<dyn std::error::Error>> {
    let expected = expected_hex.trim().to_lowercase();
    let actual = sha256_hex_of_file(path)?;

    if actual != expected {
        return Err(format!(
            "sha256 mismatch for {}:\n  expected: {expected}\n  actual:   {actual}",
            path.display()
        )
        .into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn copies_response_to_writer() {
        let response = Cursor::new(b"hello world");
        let mut output = Vec::new();

        copy_without_progress(response, &mut output).unwrap();

        assert_eq!(output, b"hello world");
    }

    #[test]
    fn copies_empty_response() {
        let response = Cursor::new(b"");
        let mut output = Vec::new();

        copy_without_progress(response, &mut output).unwrap();

        assert!(output.is_empty());
    }

    #[test]
    fn copies_large_response() {
        let input = vec![b'a'; 20_000];
        let response = Cursor::new(&input);
        let mut output = Vec::new();

        copy_without_progress(response, &mut output).unwrap();

        assert_eq!(output, input);
    }

    fn copy_without_progress<R: Read, W: Write>(
        mut reader: R,
        mut writer: W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = reader.read(&mut buffer)?;

            if bytes_read == 0 {
                break;
            }

            writer.write_all(&buffer[..bytes_read])?;
        }

        Ok(())
    }

    #[test]
    fn sha256_of_known_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hello.txt");
        std::fs::write(&path, b"hello world").unwrap();

        assert_eq!(
            sha256_hex_of_file(&path).unwrap(),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn verify_sha256_accepts_match_case_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hello.txt");
        std::fs::write(&path, b"hello world").unwrap();

        verify_sha256(
            &path,
            "B94D27B9934D3E08A52E52D7DA7DABFAC484EFE37A5380EE9088F7ACE2EFCDE9",
        )
        .unwrap();
    }

    #[test]
    fn verify_sha256_rejects_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hello.txt");
        std::fs::write(&path, b"hello world").unwrap();

        assert!(verify_sha256(&path, &"0".repeat(64)).is_err());
    }
}
