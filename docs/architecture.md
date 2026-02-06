# Architecture

## Windows-first
- UI: egui + wgpu (native)
- Content: WebView2 child HWND
- Network: local HTTP proxy → adblock → optional sing-box SOCKS5

## Data flow
WebView2 → local proxy → adblock → VPN SOCKS5 (optional) → сеть
