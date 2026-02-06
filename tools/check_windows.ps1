Write-Host "==> fmt"
cargo fmt --all

Write-Host "==> clippy"
cargo clippy --workspace --all-targets -- -D warnings

Write-Host "==> test (online)"
cargo test --workspace

if (Test-Path .cargo/config.toml) {
  Write-Host "==> test (offline)"
  cargo test --workspace --offline
}

Write-Host "Manual smoke:"
Write-Host "1) cargo run -p plus-desktop"
Write-Host "2) Open two tabs, navigate to yandex.ru"
Write-Host "3) Open Diagnostics and verify proxy/adblock hits"
