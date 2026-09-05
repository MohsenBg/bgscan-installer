use std::fmt;

use super::error::{UnsupportedArchError, UnsupportedPlatformError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    Linux,
    Windows,
    MacOs,
    Android,
}

impl Platform {
    pub fn current() -> Result<Self, UnsupportedPlatformError> {
        match std::env::consts::OS {
            "linux" => Ok(Self::Linux),
            "windows" => Ok(Self::Windows),
            "macos" => Ok(Self::MacOs),
            "android" => Ok(Self::Android),
            other => Err(UnsupportedPlatformError(other.to_string())),
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::MacOs => "macos",
            Self::Android => "android",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arch {
    Amd64,
    Amd32,
    Arm64,
    Arm32,
}

impl Arch {
    pub fn current() -> Result<Self, UnsupportedArchError> {
        match std::env::consts::ARCH {
            "x86_64" => Ok(Self::Amd64),
            "x86" => Ok(Self::Amd32),
            "aarch64" => Ok(Self::Arm64),
            "arm" => Ok(Self::Arm32),
            other => Err(UnsupportedArchError(other.to_string())),
        }
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Amd64 => "amd64",
            Self::Amd32 => "386",
            Self::Arm64 => "arm64",
            Self::Arm32 => "arm",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlatformInfo {
    pub platform: Platform,
    pub arch: Arch,
}

impl PlatformInfo {
    #[allow(dead_code)]
    pub fn new(platform: Platform, arch: Arch) -> Self {
        Self { platform, arch }
    }

    pub fn current() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            platform: Platform::current()?,
            arch: Arch::current()?,
        })
    }

    /// Release asset filename for this platform/arch (e.g. `bgscan-linux-64.zip`).
    /// Returns `None` when the combination has no published asset.
    pub fn asset_name(&self) -> Option<&'static str> {
        match (self.platform, self.arch) {
            (Platform::Linux, Arch::Arm64) => Some("bgscan-linux-arm64.zip"),
            (Platform::Linux, Arch::Arm32) => Some("bgscan-linux-arm32-v7a.zip"),
            (Platform::Linux, Arch::Amd64) => Some("bgscan-linux-64.zip"),
            (Platform::Linux, Arch::Amd32) => Some("bgscan-linux-32.zip"),

            (Platform::Android, Arch::Arm64) => Some("bgscan-android-arm64-v8a.zip"),
            (Platform::Android, Arch::Arm32) => Some("bgscan-android-armeabi-v7a.zip"),
            (Platform::Android, Arch::Amd64) => Some("bgscan-android-x86_64.zip"),
            (Platform::Android, Arch::Amd32) => Some("bgscan-android-x86.zip"),

            (Platform::MacOs, Arch::Arm64) => Some("bgscan-macos-arm64.zip"),
            (Platform::MacOs, Arch::Amd64) => Some("bgscan-macos-64.zip"),

            (Platform::Windows, Arch::Amd64) => Some("bgscan-windows-64.zip"),
            (Platform::Windows, Arch::Arm64) => Some("bgscan-windows-arm64.zip"),

            _ => None,
        }
    }

    /// Executable name inside the zip: `bgscan.exe` on Windows, `bgscan` elsewhere.
    pub fn binary_name(&self) -> &'static str {
        match self.platform {
            Platform::Windows => "bgscan.exe",
            _ => "bgscan",
        }
    }
}

impl fmt::Display for PlatformInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.platform, self.arch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_amd64_asset() {
        let info = PlatformInfo::new(Platform::Linux, Arch::Amd64);
        assert_eq!(info.asset_name(), Some("bgscan-linux-64.zip"));
    }

    #[test]
    fn unsupported_combo_returns_none() {
        // macOS 32-bit has no asset.
        let info = PlatformInfo::new(Platform::MacOs, Arch::Amd32);
        assert_eq!(info.asset_name(), None);
    }

    #[test]
    fn binary_name_per_platform() {
        let win = PlatformInfo::new(Platform::Windows, Arch::Amd64);
        assert_eq!(win.binary_name(), "bgscan.exe");
        let linux = PlatformInfo::new(Platform::Linux, Arch::Amd64);
        assert_eq!(linux.binary_name(), "bgscan");
    }
}
