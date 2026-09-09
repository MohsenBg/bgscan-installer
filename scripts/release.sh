#!/usr/bin/env bash
# Builds the GitHub release files from whatever ./dist contains:
# a sha256 manifest and markdown release notes.
#
# Usage: ./scripts/release.sh <tag>   (expects GITHUB_REPOSITORY in CI,
# falls back to user/repo otherwise)

set -euo pipefail

if [[ -z "${1:-}" ]]; then
  echo "Usage: $0 <tag_version>"
  exit 1
fi

TAG_VERSION="$1"

ROOT_DIR="$PWD"
DIST_DIR="$ROOT_DIR/dist"
RELEASE_DIR="$ROOT_DIR/release"

CHECKSUM_FILE="$RELEASE_DIR/checksum.txt"
NOTES_FILE="$RELEASE_DIR/release_notes.md"

REPO_URL="${GITHUB_SERVER_URL:-https://github.com}/${GITHUB_REPOSITORY:-user/repo}"

mkdir -p "$RELEASE_DIR"

log() {
  echo
  echo "============================================================"
  echo "$*"
  echo "============================================================"
}

# display names for the release notes table
declare -A OS_MAP=(
  ["linux"]="🐧 Linux"
  ["windows"]="🪟 Windows"
  ["android"]="🤖 Android"
  ["macos"]="🍏 macOS"
)

declare -A ARCH_MAP=(
  ["linux-64"]="AMD64 / x64"
  ["linux-32"]="x86 / 32-bit"
  ["linux-arm64"]="ARM64"
  ["linux-arm32-v7a"]="ARM32 (ARMv7)"

  ["windows-64"]="AMD64 / x64"
  ["windows-arm64"]="ARM64"

  ["android-arm64-v8a"]="ARM64 (v8a)"
  ["android-armeabi-v7a"]="ARM32 (armeabi-v7a)"
  ["android-x86"]="x86 / 32-bit"
  ["android-x86_64"]="AMD64 / x64"

  ["macos-64"]="AMD64 / Intel x64"
  ["macos-arm64"]="ARM64 / Apple Silicon"
)

log "Validating dist/ directory"

if [[ ! -d "$DIST_DIR" ]] || [[ -z "$(ls -A "$DIST_DIR" 2>/dev/null)" ]]; then
  echo "ERROR: dist/ is empty or missing"
  exit 1
fi

log "Generating SHA256 checksums"

: >"$CHECKSUM_FILE"

cd "$DIST_DIR"

FILES=()

for file in *; do
  [[ -f "$file" ]] || continue

  sha256sum "$file" >>"$CHECKSUM_FILE"
  FILES+=("$file")
done

log "Generating release notes"

: >"$NOTES_FILE"

cat <<EOF >>"$NOTES_FILE"
# 🚀 bgscan-installer Release $TAG_VERSION

Automated multi-platform build artifacts generated via GitHub Actions CI.

All binaries are **raw executables (no compression)**.

---

## 📦 Download Table

| 🌍 Platform | 🧬 Architecture | 📥 Download |
|------------|----------------|------------|
EOF

# figure out platform/arch from the file name
for file in "${FILES[@]}"; do

  # normalize windows extension
  clean_file="${file%.exe}"

  # remove prefix
  key="${clean_file#bgscan-installer-}"

  # split OS / ARCH
  OS_KEY="${key%%-*}"
  ARCH_KEY="${key#*-}"

  FULL_KEY="${OS_KEY}-${ARCH_KEY}"

  OS_NAME="${OS_MAP[$OS_KEY]:-❓ Unknown}"
  ARCH_NAME="${ARCH_MAP[$FULL_KEY]:-$ARCH_KEY}"

  LINK="[$file]($REPO_URL/releases/download/$TAG_VERSION/$file)"

  echo "| $OS_NAME | $ARCH_NAME | $LINK |" >>"$NOTES_FILE"

done

cat <<EOF >>"$NOTES_FILE"

---

## 🔐 SHA256 Checksums

| File | SHA256 |
|------|--------|
EOF

while read -r hash file; do
  echo "| $file | $hash |" >>"$NOTES_FILE"
done <"$CHECKSUM_FILE"

cat <<EOF >>"$NOTES_FILE"

---

## ⚙️ Build Info

- Project: bgscan-installer
- Version: $TAG_VERSION
- Pipeline: GitHub Actions CI/CD
EOF

log "Release generation completed"
echo "Output directory: $RELEASE_DIR"
ls -lh "$RELEASE_DIR"
