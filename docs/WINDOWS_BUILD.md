# Windows Build

## Requirements
- Rust MSVC toolchain (`rustup default stable-x86_64-pc-windows-msvc`)
- Microsoft Build Tools / Visual Studio Build Tools
- WebView2 Runtime installed

## Build
```powershell
cargo build --workspace
```

## Run
```powershell
cargo run -p plus-desktop
```
