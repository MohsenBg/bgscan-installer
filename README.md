# bgscan-installer

Installs [bgscan](https://github.com/MohsenBg/bgscan) from its GitHub releases.
It detects your platform, downloads the matching zip, verifies the sha256
checksum, and extracts it into the current directory.

Written in Rust, no runtime dependencies.

## Usage

Run the installer in the directory where you want bgscan to live:

```
bgscan-installer install
```

This installs the latest release. To pin a version (with or without the `v`
prefix):

```
bgscan-installer install --version 2.10.0
bgscan-installer install -v v2.10.0
```

Other commands:

```
bgscan-installer version    # print installer version
```

### Install location

If a `bgscan` binary already exists in the current directory, that directory
is treated as the installation. Otherwise everything goes into a `bgscan/`
subdirectory.

### Existing installations

When an existing installation is found, you get a menu:

```
[1] Update installation (keeps your ips/assets/settings)
[2] Clean install (removes existing installation)
[3] Back up existing installation and install new version
[4] Cancel
```

- **Update** replaces the binary and adds new files, but leaves existing
  `ips/`, `assets/` and settings untouched.
- **Backup** renames the old directory to `bgscan_<timestamp>` before
  installing.

## Supported platforms

Release assets are picked automatically based on the running OS and
architecture:

| OS      | Architectures                     |
|---------|-----------------------------------|
| Linux   | x86_64, i686, arm64, armv7        |
| macOS   | x86_64 (Intel), arm64 (Apple Silicon) |
| Windows | x86_64, arm64                     |
| Android | arm64-v8a, armeabi-v7a, x86_64, x86 |

Anything else exits with an unsupported platform error.

## How it works

1. Resolves the requested version through the GitHub API (`latest` resolves
   to the actual tag; pinned versions are checked to exist).
2. Downloads `bgscan-<platform>.zip` from the release and `checksum.txt`
   from the same location.
3. Verifies the zip against its sha256 and aborts on mismatch.
4. Extracts the zip into a temp dir and copies the bundle (binary, `ips/`,
   `assets/`) into the install directory. Temp files are cleaned up.

Downloaded zips are never run, only extracted.

## Building

Requires a stable Rust toolchain (2024 edition).

```
cargo build --release
```

The release profile is tuned for small binaries (`lto`, `strip`, `panic = "abort"`).

Cross builds for releases are handled by `scripts/build.sh`, which uses
[cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild) for Linux
(static musl), macOS and Windows, and the Android NDK for Android targets.
You normally don't call it by hand — the release workflow does it on tags.

For local builds the version comes from `Cargo.toml`; release builds override
it with the `APP_VERSION` env var at compile time.

## Development

```
cargo test              # unit tests
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

Source layout:

- `src/installer/` — platform detection, GitHub release API, checksum
  parsing, file operations and the install flow itself
- `src/downloader/` — HTTP download with a progress bar and sha256 verification
- `src/archive/` — zip extraction
- `src/ui/` — terminal output

CI runs fmt, clippy and tests on every push and PR.
