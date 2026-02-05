#!/usr/bin/env bash
set -euo pipefail
cargo build --release -p plus-desktop
mkdir -p dist
cp target/release/plus-desktop dist/Plus

echo "Package stubs:"
echo "- Linux: create AppImage/DEB/RPM from dist/Plus"
echo "- macOS: package .app + dmg"
echo "- Windows: package MSI/NSIS"
