# Limitations by Platform

## Windows (WebView2)
- Требуется установленный WebView2 Runtime.
- Прокси задаётся через аргумент `--proxy-server` в WebView2.

## macOS (WKWebView)
- Прокси задаётся через переменные среды процесса (`HTTP(S)/ALL_PROXY`), поведение зависит от системной сети.
- Для полной изоляции может требоваться системная настройка proxy.

## Linux (WebKitGTK)
- Требуются пакеты `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libsoup-3.0-dev`.
- Прокси задаётся через переменные среды процесса (`HTTP(S)/ALL_PROXY`).
