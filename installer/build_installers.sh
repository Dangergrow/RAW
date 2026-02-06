#!/usr/bin/env bash
set -euo pipefail
cargo build --release -p plus-desktop
mkdir -p dist

case "$(uname -s)" in
  Linux)
    cp target/release/plus-desktop dist/Plus
    ;;
  Darwin)
    cp target/release/plus-desktop dist/Plus
    hdiutil create -volname Plus -srcfolder dist -ov -format UDZO dist/Plus.dmg
    ;;
  MINGW*|MSYS*|CYGWIN*|Windows_NT)
    cp target/release/plus-desktop.exe dist/Plus.exe
    ;;
esac
