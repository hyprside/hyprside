#!/usr/bin/env bash
set -euo pipefail

# ================================================================
# Hyprside Initramfs Builder
# ================================================================
# This script builds the initramfs.img used by Hyprside.
# It compiles the init (stage 1) binary statically with musl,
# validates it, and packs it into a minimal initramfs image.
# ================================================================

# -------- Configuration --------
INIT_STAGE1_DIR="${INIT_STAGE1_DIR:-packages/init-stage-1}"
INIT_STAGE1_BIN="${INIT_STAGE1_BIN:-$INIT_STAGE1_DIR/target/x86_64-unknown-linux-musl/release/init-stage-1}"
BUILD_DIR="${BUILD_DIR:-build}"
ROOTFS_DIR="${ROOTFS_DIR:-$BUILD_DIR/rootfs}"
INITRAMFS="${INITRAMFS:-$1}"
RUSTFLAGS="${RUSTFLAGS:--C target-feature=+crt-static -C link-self-contained=yes -C link-args=-static}"

# ================================================================
# 🧾 List source dependencies
# ================================================================
echo "🔍 Listing code dependencies in $INIT_STAGE1_DIR..."
find "$INIT_STAGE1_DIR" -type f \
    \( -name "*.rs" -o -name "*.toml" -o -name "*.sh" -o -name "*.c" -o -name "*.h" \) \
    -not -path '*/target/*' | while read -r file; do
    echo "DEPENDENCY $file"
done

# ================================================================
# 1️⃣ Build init stage 1 (musl static)
# ================================================================
echo "🔧 Building init stage 1 (musl static)..."

if command -v musl-gcc >/dev/null 2>&1; then
    (cd "$INIT_STAGE1_DIR" && RUSTFLAGS="$RUSTFLAGS" cargo build --release --target x86_64-unknown-linux-musl)
else
    echo "⚠️  musl-gcc not found, using muslrust container..."
    docker run --rm -t -v "$(realpath "$INIT_STAGE1_DIR")":/volume \
        clux/muslrust:stable \
        cargo build --release
fi

# ================================================================
# 2️⃣ Validate binary
# ================================================================
echo "🧩 Validating binary integrity..."

file "$INIT_STAGE1_BIN" | grep -q "ELF 64-bit" || { echo "❌ Not a valid ELF binary!"; exit 1; }
ldd "$INIT_STAGE1_BIN" 2>&1 | grep -q "statically linked" || { echo "❌ Not statically linked!"; exit 1; }
! readelf -l "$INIT_STAGE1_BIN" | grep -q INTERP || { echo "❌ Has INTERP segment!"; exit 1; }
! readelf -d "$INIT_STAGE1_BIN" | grep -q NEEDED || { echo "❌ Has dynamic dependencies!"; exit 1; }

echo "✅ Binary validated: fully static and self-contained."

# ================================================================
# 3️⃣ Build initramfs
# ================================================================
echo "📁 Preparing rootfs..."
rm -rf "$ROOTFS_DIR"
mkdir -p "$ROOTFS_DIR"/{dev,proc,sys}
cp "$INIT_STAGE1_BIN" "$ROOTFS_DIR/init"
chmod +x "$ROOTFS_DIR/init"

echo "📦 Packing initramfs..."
mkdir -p "$BUILD_DIR"
export OLDPWD=$PWD
pushd "$ROOTFS_DIR" >/dev/null
mkdir -p $(dirname "$OLDPWD/$1")
rm "$OLDPWD/$1" -f
find . | cpio -o -H newc | zstd -19 -T0 -o "$OLDPWD/$1"
popd >/dev/null

echo "✅ Initramfs created at: $1"
