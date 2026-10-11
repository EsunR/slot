#!/usr/bin/env bash
set -euo pipefail

# Run from the repository root in WSL after build-device-wsl.ps1 -Rg35xxsp.
binary=target-rg35xxsp/aarch64-unknown-linux-gnu/release/slot
out=target-rg35xxsp-sdcard
test -f "$binary"
mkdir -p vendor

if [ ! -f vendor/mgba_libretro.so ]; then
    curl -fsSL -o vendor/mgba-linux.zip \
        https://buildbot.libretro.com/nightly/linux/aarch64/latest/mgba_libretro.so.zip
    unzip -o -q vendor/mgba-linux.zip -d vendor
    rm -f vendor/mgba-linux.zip
fi
file vendor/mgba_libretro.so | grep -qi aarch64

if [ ! -f vendor/gpsp_libretro.so ] || [ ! -f vendor/gpsp-src.tar.gz ] || \
   [ ! -f vendor/gpsp-src.sha ] || [ ! -f vendor/gpsp-src.meta ]; then
    rm -f vendor/gpsp_libretro.so vendor/gpsp-src.tar.gz \
        vendor/gpsp-src.sha vendor/gpsp-src.meta
    curl -fsSL -o vendor/gpsp-linux.zip \
        https://buildbot.libretro.com/nightly/linux/aarch64/latest/gpsp_libretro.so.zip
    raw=$(unzip -Z -T vendor/gpsp-linux.zip gpsp_libretro.so | awk '/^-/ {print $(NF-1)}')
    binary_date=$(printf '%s' "$raw" | sed -E \
        's/^([0-9]{4})([0-9]{2})([0-9]{2})\.([0-9]{2})([0-9]{2})([0-9]{2})$/\1-\2-\3T\4:\5:\6Z/')
    [[ "$binary_date" == *Z ]]
    unzip -o -q vendor/gpsp-linux.zip -d vendor
    rm -f vendor/gpsp-linux.zip

    api_response=$(curl -fsS --max-time 10 \
        "https://api.github.com/repos/libretro/gpsp/commits?until=${binary_date}&per_page=1" \
        2>/dev/null || true)
    sha=$(printf '%s' "$api_response" | sed -nE \
        's/.*"sha": *"([0-9a-f]{40})".*/\1/p' | head -n 1)
    method=github-api
    if [ -z "$sha" ]; then
        sha=$(git ls-remote https://github.com/libretro/gpsp master | cut -f1)
        method=git-ls-remote-fallback
    fi
    [[ "$sha" =~ ^[0-9a-f]{40}$ ]]
    curl -fsSL -o vendor/gpsp-src.tar.gz \
        "https://github.com/libretro/gpsp/archive/${sha}.tar.gz"
    tar -tzf vendor/gpsp-src.tar.gz | grep "^gpsp-${sha}/Makefile$" > /dev/null
    printf '%s\n' "$sha" > vendor/gpsp-src.sha
    printf 'commit=%s\nbinary_date=%s\nresolved_via=%s\n' \
        "$sha" "$binary_date" "$method" > vendor/gpsp-src.meta
fi
file vendor/gpsp_libretro.so | grep -qi aarch64

# --init-root creates exactly the same content layout as the desktop test root.
qemu-aarch64 -L /usr/aarch64-linux-gnu "$binary" --init-root "$out"
cp "$binary" "$out/System/slot"
cp vendor/mgba_libretro.so vendor/gpsp_libretro.so "$out/System/"
cp -r licenses "$out/System/"
sha=$(cat vendor/gpsp-src.sha)
cp vendor/gpsp-src.tar.gz "$out/System/licenses/gpsp-${sha}.tar.gz"
cp vendor/gpsp-src.meta "$out/System/licenses/gpsp-${sha}.meta"

# README's release font is an optional card font, not the small fallback embedded in slot.
# Pin the upstream repository revision so repeat packages use the same typeface and licence.
plix_rev=9456532eaeeff7737dc7e72f86ecc0be62183f80
if [ ! -f vendor/plix-bold.zip ]; then
    curl -fsSL -o vendor/plix-bold.zip \
        "https://raw.githubusercontent.com/cathree3/Plix/${plix_rev}/Plix%E6%94%B9%E5%90%8D%E3%80%8C%E6%99%AE%E5%88%A9%E4%B8%96%E3%80%8D%E5%8D%87%E7%B4%9Av1.2%EF%BC%88%E7%B2%97%E7%9A%84%E9%83%A8%E5%88%86%EF%BC%89.zip"
fi
unzip -p vendor/plix-bold.zip Plix-bold-v1.2.otf > "$out/System/fonts/Plix-Bold.otf"
if [ ! -f vendor/Plix-OFL.txt ]; then
    curl -fsSL -o vendor/Plix-OFL.txt \
        "https://raw.githubusercontent.com/cathree3/Plix/${plix_rev}/LICENSE.txt"
fi
cp vendor/Plix-OFL.txt "$out/System/fonts/Plix-OFL.txt"
cp assets/overlays/default.png "$out/Overlay/GBA.png"
echo "RG35XXSP content tree: $out"
