# Architecture

## Runtime engine strategy
Plus использует системные web engines через Wry:
- Windows: WebView2
- macOS: WKWebView
- Linux: WebKitGTK

## Платформенная матрица (proxy + adblock)
| Платформа | Proxy для WebView | Перехват сетевых запросов (сабресурсы) | Ограничения |
| --- | --- | --- | --- |
| Windows (WebView2) | `--proxy-server=` через WebView2 args | Перехват выполняется на уровне локального HTTP proxy (adblock proxy) | Требует WebView2 Runtime |
| macOS (WKWebView) | `HTTP(S)/ALL_PROXY` env для процесса WebView | Перехват выполняется на уровне локального HTTP proxy (adblock proxy) | Полная прокси‑изоляция зависит от системных настроек |
| Linux (WebKitGTK) | `HTTP(S)/ALL_PROXY` env для процесса WebView | Перехват выполняется на уровне локального HTTP proxy (adblock proxy) | Требует WebKitGTK dev пакеты в CI |

`engine/` не реализует HTML/CSS/JS сам, а управляет:
- policy checks
- VPN route decision
- navigation validation
- privacy defaults

`renderer/` поднимает браузерное окно и webview runtime.

## Network pipeline
- `net/` — локальный HTTP proxy, применяющий ABP‑правила.
- WebView → local proxy → sing-box SOCKS5 (VPN) → сеть.
- `vpn/` управляет sing-box и локальным SOCKS5.
