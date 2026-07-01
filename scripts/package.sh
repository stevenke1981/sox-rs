#!/usr/bin/env bash
# Build sox release binary and create platform installation packages.
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

# Get version from Cargo metadata
VERSION="$(cargo metadata --format-version=1 --no-deps 2>/dev/null | python3 -c "import sys,json; pkgs=json.load(sys.stdin)['packages']; print([p['version'] for p in pkgs if p['name']=='rust-sox'][0])")"

echo "=== Packaging sox v$VERSION ==="

# Step 1: Build release
echo ""
echo "[1/4] Building release binary..."
cargo build --release

# Step 2: Detect target
RUST_TARGET="$(rustc -vV | sed -n 's/host: //p')"
ARCHIVE_NAME="sox-${VERSION}-${RUST_TARGET}"

# Step 3: Prepare dist directory
echo ""
echo "[2/4] Preparing dist/${ARCHIVE_NAME}..."
DIST_DIR="${PROJECT_ROOT}/dist"
PACKAGE_DIR="${DIST_DIR}/${ARCHIVE_NAME}"
mkdir -p "${PACKAGE_DIR}/examples"

cp "target/release/sox" "${PACKAGE_DIR}/sox"
cp README.md "${PACKAGE_DIR}/"
[ -f LICENSE-MIT ] && cp LICENSE-MIT "${PACKAGE_DIR}/"
[ -f LICENSE-LGPL ] && cp LICENSE-LGPL "${PACKAGE_DIR}/"
cp examples/* "${PACKAGE_DIR}/examples/" 2>/dev/null || true

# Step 4: Create archive
echo ""
echo "[3/4] Creating archive..."
ARCHIVE_FILE="${ARCHIVE_NAME}.tar.gz"
tar -czf "${DIST_DIR}/${ARCHIVE_FILE}" -C "${DIST_DIR}" "${ARCHIVE_NAME}"

# Step 5: Generate checksums
echo ""
echo "[4/4] Generating checksums..."
CHECKSUM_FILE="${DIST_DIR}/SHA256SUMS.txt"
cd "${DIST_DIR}"
sha256sum sox-*.* > "${CHECKSUM_FILE}" 2>/dev/null || true
cd "${PROJECT_ROOT}"

echo ""
echo "=== Package complete ==="
echo "Binary: target/release/sox"
echo "Package: dist/${ARCHIVE_FILE}"
echo "Checksums: dist/SHA256SUMS.txt"
