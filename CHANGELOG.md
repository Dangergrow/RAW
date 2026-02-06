# Changelog

## Unreleased
- Migrated to native egui UI with WebView2 content area.
- Added Windows build docs and updated user guide/limitations.

### How to verify
```powershell
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
Manual smoke:
- Run `cargo run -p plus-desktop`
- Open two tabs, navigate to yandex.ru
- Open Diagnostics, check proxy/adblock hits
