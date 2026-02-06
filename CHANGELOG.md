# Changelog

## Unreleased
- Added app shell UI with tabs/navigation, settings shell, and new-tab enhancements.
- Added local adblock proxy routing all WebView traffic with optional VPN SOCKS5 chaining.
- Added diagnostics page, regression checklist, and user guide docs.

### How to verify
```bash
cargo fmt --all
cargo test --workspace
```
Offline:
```bash
tools/vendorize.sh
cargo test --workspace --offline
```
