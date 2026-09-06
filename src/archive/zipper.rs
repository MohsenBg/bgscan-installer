use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

use zip::ZipArchive;

fn cleanup_paths(paths: &[PathBuf]) {
    for path in paths {
        if path.is_dir() {
            let _ = fs::remove_dir_all(path);
        } else if path.is_file() {
            let _ = fs::remove_file(path);
        }
    }
}

/// Extracts `zip` into `dest` (created if needed) and returns `dest`.
/// Zip-slip safe via `enclosed_name()`; already-written paths are
/// removed again if extraction fails halfway through.
pub fn unzip(zip: &Path, dest: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let file = fs::File::open(zip)?;
    let mut archive = ZipArchive::new(file)?;

    let mut paths: Vec<PathBuf> = Vec::new();

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                cleanup_paths(&paths);
                return Err(e.into());
            }
        };

        let out_path = match file.enclosed_name() {
            Some(path) => dest.join(path),
            None => continue,
        };

        paths.push(out_path.clone());

        if file.is_dir() {
            if let Err(e) = fs::create_dir_all(&out_path) {
                cleanup_paths(&paths);
                return Err(e.into());
            }

            continue;
        }

        if let Some(parent) = out_path.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            cleanup_paths(&paths);
            return Err(e.into());
        }

        if let Err(e) =
            fs::File::create(&out_path).and_then(|mut outfile| io::copy(&mut file, &mut outfile))
        {
            cleanup_paths(&paths);
            return Err(e.into());
        }
    }

    Ok(dest.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_zip(dir: &Path) -> PathBuf {
        let zip_path = dir.join("test.zip");
        let file = fs::File::create(&zip_path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        writer
            .start_file("bundle/bgscan", zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"fake-binary").unwrap();
        writer
            .start_file(
                "bundle/data/list.csv",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
        writer.write_all(b"a,b,c").unwrap();
        writer.finish().unwrap();
        zip_path
    }

    #[test]
    fn unzips_nested_entries() {
        let dir = tempfile::tempdir().unwrap();
        let zip = make_zip(dir.path());
        let dest = dir.path().join("extracted");

        let out = unzip(&zip, &dest).unwrap();

        assert_eq!(out, dest);
        assert_eq!(
            fs::read(dest.join("bundle/bgscan")).unwrap(),
            b"fake-binary"
        );
        assert_eq!(
            fs::read(dest.join("bundle/data/list.csv")).unwrap(),
            b"a,b,c"
        );
    }
}
