# Architecture

## Runtime engine strategy
Plus использует системные web engines через Wry:
- Windows: WebView2
- macOS: WKWebView
- Linux: WebKitGTK

`engine/` не реализует HTML/CSS/JS сам, а управляет:
- policy checks
- VPN route decision
- navigation validation
- privacy defaults

`renderer/` поднимает браузерное окно и webview runtime.

## Network pipeline
- `net/` — HTTP client для системных функций (downloads/ip checks/api).
- Webview engine — фактический рендер/загрузка страниц.
- VPN core (`vpn/`) поднимает локальный socks5 endpoint для маршрутизации.
- Adblock (`adblock/`) использует ABP-совместимый движок.
