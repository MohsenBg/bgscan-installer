use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// Recursively searches `dir` for a file named `name`. Returns the first match.
pub fn find_binary(dir: &Path, name: &str) -> Option<PathBuf> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).ok()?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
                return Some(path);
            }
        }
    }
    None
}

/// Copies `src` dir contents into `dst`, merging and overwriting files.
pub fn copy_dir_contents(src: &Path, dst: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_contents(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Like `copy_dir_contents`, but skips files that already exist in `dst`.
/// New files and directories are still created.
pub fn copy_dir_contents_skip_existing(src: &Path, dst: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_contents_skip_existing(&from, &to)?;
        } else if !to.is_file() {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
pub fn make_executable(path: &Path) -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
pub fn make_executable(_path: &Path) -> Result<(), Box<dyn Error>> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("bundle-1.0/ips")).unwrap();
        fs::write(dir.path().join("bundle-1.0/bgscan"), b"bin").unwrap();
        fs::write(dir.path().join("bundle-1.0/ips/a.csv"), b"a").unwrap();
        dir
    }

    #[test]
    fn finds_nested_binary() {
        let dir = tree();
        assert_eq!(
            find_binary(dir.path(), "bgscan"),
            Some(dir.path().join("bundle-1.0/bgscan"))
        );
        assert_eq!(find_binary(dir.path(), "missing"), None);
    }

    #[test]
    fn copies_and_overwrites() {
        let dir = tree();
        let src = dir.path().join("bundle-1.0");
        let dst = dir.path().join("out");

        fs::create_dir_all(&dst).unwrap();
        fs::write(dst.join("bgscan"), b"old").unwrap();

        copy_dir_contents(&src, &dst).unwrap();

        assert_eq!(fs::read(dst.join("bgscan")).unwrap(), b"bin");
        assert_eq!(fs::read(dst.join("ips/a.csv")).unwrap(), b"a");
    }

    #[test]
    fn skip_existing_keeps_old_file() {
        let dir = tree();
        let src = dir.path().join("bundle-1.0");
        let dst = dir.path().join("out");

        fs::create_dir_all(dst.join("ips")).unwrap();
        fs::write(dst.join("bgscan"), b"old-binary").unwrap();
        fs::write(dst.join("ips/a.csv"), b"old-csv").unwrap();

        copy_dir_contents_skip_existing(&src, &dst).unwrap();

        // Existing files kept
        assert_eq!(fs::read(dst.join("bgscan")).unwrap(), b"old-binary");
        assert_eq!(fs::read(dst.join("ips/a.csv")).unwrap(), b"old-csv");
    }
}
