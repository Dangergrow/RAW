# Архитектура

## Модули
- `apps/plus-desktop` — нативный UI (egui)
- `renderer` — WebView2 host (Windows)
- `net` — локальный HTTP‑proxy + цепочка в SOCKS5
- `adblock` — ABP‑движок
- `vpn` — sing-box менеджер
- `privacy` — профиль и хранилище
- `tests` — smoke/e2e

## Потоки данных
WebView2 → local proxy → adblock → (VPN SOCKS5) → Интернет

## Профиль и безопасность
Профиль хранится локально, чувствительные данные шифруются и/или идут через keychain.

## AdBlock + WebResourceRequested
На Windows используется WebView2 WebResourceRequested, чтобы перехватывать сабресурсы.
