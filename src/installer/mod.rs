pub mod checksum;
pub mod error;
pub mod files;
pub mod github;
pub mod platform;

use error::UnsupportedPlatformError;

use std::{
    env,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tempfile::Builder;

use crate::archive::unzip;
use crate::downloader::{download_file, verify_sha256};
use crate::ui::{TerminalUI, UI};
use checksum::{CHECKSUM_FILE, parse_checksum_file};
use files::{copy_dir_contents, copy_dir_contents_skip_existing, find_binary, make_executable};
use github::{checksum_url_for, download_url, resolve_version};
use platform::PlatformInfo;

const TOTAL_STEPS: usize = 5;

pub struct Installer {
    info: PlatformInfo,
    has_installation: bool,
    installation_dir: PathBuf,
}

fn has_bgscan_binary(dir: &Path, binary_name: &str) -> bool {
    dir.join(binary_name).is_file()
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl Installer {
    pub fn new(info: PlatformInfo) -> Self {
        let cwd = env::current_dir().unwrap_or_default();

        let installation_dir = if has_bgscan_binary(&cwd, info.binary_name()) {
            cwd.to_path_buf()
        } else {
            cwd.join("bgscan")
        };

        let has_installation = installation_dir.exists();

        Self {
            info,
            has_installation,
            installation_dir,
        }
    }

    pub fn current() -> Result<Self, Box<dyn Error>> {
        Ok(Self::new(PlatformInfo::current()?))
    }

    pub fn install(&self, version: &str) -> Result<(), Box<dyn Error>> {
        let asset = self.info.asset_name().ok_or_else(|| {
            UnsupportedPlatformError(format!(
                "{:?}/{:?} combination not supported",
                self.info.platform, self.info.arch
            ))
        })?;

        let mut tui = TerminalUI::new(io::stdout());
        let mut is_update = false;

        tui.title("System");
        tui.table(&[
            ("OS", &self.info.platform.to_string()),
            ("Architecture", &self.info.arch.to_string()),
            ("Platform", &self.info.to_string()),
        ]);

        if self.has_installation {
            tui.raw("\n");
            tui.warn(&format!(
                "Existing installation found at {}",
                self.installation_dir.display()
            ));

            let choice = tui
                .menu(
                    "What would you like to do?",
                    &[
                        "Update installation (keeps your ips/assets/settings)",
                        "Clean install (removes existing installation)",
                        "Back up existing installation and install new version",
                    ],
                )
                .unwrap_or(4);

            match choice {
                1 => {
                    is_update = true;
                }
                2 => {
                    tui.step(1, TOTAL_STEPS, "Removing old installation");
                    fs::remove_dir_all(&self.installation_dir)?;
                    tui.success("Old installation removed");
                }
                3 => {
                    tui.step(1, TOTAL_STEPS, "Backing up existing installation");
                    let parent = self
                        .installation_dir
                        .parent()
                        .unwrap_or(&self.installation_dir)
                        .to_path_buf();
                    let backup = parent.join(format!("bgscan_{}", timestamp()));
                    fs::rename(&self.installation_dir, &backup)?;
                    tui.success(&format!("Backed up to {}", backup.display()));
                }
                _ => {
                    tui.info("Cancelled. Nothing was changed.");
                    return Ok(());
                }
            }
        }

        tui.step(2, TOTAL_STEPS, &format!("Resolving version '{version}'"));
        let resolved = resolve_version(version)?;
        let url = download_url(&self.info, &resolved)
            .ok_or_else(|| UnsupportedPlatformError(format!("no asset for {}", self.info)))?;
        tui.success(&format!("Using bgscan {resolved} ({asset})"));

        tui.step(3, TOTAL_STEPS, "Downloading and verifying");
        let tmp = Builder::new().prefix("bgscan_").tempdir()?;
        let zip_path = tmp.path().join(asset);
        download_file(&url, &zip_path)?;

        let checksum_url = checksum_url_for(&resolved);
        let checksum_path = tmp.path().join(CHECKSUM_FILE);
        download_file(&checksum_url, &checksum_path)?;

        let expected = parse_checksum_file(&checksum_path, asset)?;
        verify_sha256(&zip_path, &expected)?;
        tui.success("Downloaded and checksum verified");

        tui.step(4, TOTAL_STEPS, "Extracting archive");
        let extracted = tmp.path().join("extracted");
        unzip(&zip_path, &extracted)?;

        let binary_name = self.info.binary_name();
        let binary_src = find_binary(&extracted, binary_name)
            .ok_or_else(|| format!("'{binary_name}' not found inside {asset}"))?;
        tui.success("Archive extracted");

        let installed_path = if is_update {
            tui.step(5, TOTAL_STEPS, "Updating installation");
            fs::create_dir_all(&self.installation_dir)?;

            let source_root = binary_src.parent().unwrap();
            // Keep existing ips/assets/settings, only replace the binary
            // and add files that are missing.
            copy_dir_contents_skip_existing(source_root, &self.installation_dir)?;
            let dest_bin = self.installation_dir.join(binary_name);
            fs::copy(&binary_src, &dest_bin)?;
            make_executable(&dest_bin)?;
            tui.success("Installation updated");
            dest_bin
        } else {
            tui.step(5, TOTAL_STEPS, "Installing");
            fs::create_dir_all(&self.installation_dir)?;

            let source_root = binary_src.parent().unwrap();
            copy_dir_contents(source_root, &self.installation_dir)?;

            let relative = binary_src
                .strip_prefix(source_root)
                .unwrap_or(Path::new(binary_name));
            let installed = self.installation_dir.join(relative);
            make_executable(&installed)?;
            tui.success("Installation complete");
            installed
        };

        let relative_dir = self
            .installation_dir
            .strip_prefix(env::current_dir().unwrap_or_default())
            .unwrap_or(&self.installation_dir);

        tui.title(&format!("bgscan {resolved} is ready"));
        tui.table(&[
            ("Location", &installed_path.display().to_string()),
            (
                "Run",
                &format!("cd {} && ./{}", relative_dir.display(), binary_name),
            ),
        ]);

        Ok(())
    }
}
