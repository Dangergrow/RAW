# Plus Browser (реальный MVP на системном web engine)

## Что было прототипом и не соответствовало требованиям
1. Самописный `engine` не рендерил реальные сайты: только извлечение текста и прямоугольники, без полноценного DOM/CSSOM/JS выполнения для SPA.  
2. `renderer` рисовал упрощённые боксы в PNG, но не был браузерным compositing-пайплайном для реальных страниц.  
3. VPN-модуль не поднимал реальный tunnel core (sing-box/xray), а лишь формировал proxy URL.  
4. Adblock был «похожей» regex-реализацией, не полноценной ABP/AdGuard-совместимостью через зрелый движок.  
5. UI не обеспечивал реальный браузерный view-компонент движка (WebKit/WebView2), только текстовый вывод из парсинга.  
6. Offline-reproducibility отсутствовала: не было `vendor`-режима Cargo.

## Что сделано в этой переработке
- Интегрирован реальный web engine слой через **Wry** (WebView2 на Windows, WKWebView на macOS, WebKitGTK на Linux).  
- `engine/` теперь отвечает за политики, изоляцию, валидацию навигации, VPN route decisions.  
- `renderer/` теперь запускает реальный browser view и загружает реальную HTML/JS страницу new-tab.  
- `adblock/` переведён на библиотеку `adblock` (Brave, ABP-совместимый движок).  
- `vpn/` запускает **sing-box subprocess** с генерируемым runtime config и локальным SOCKS5 inbound.  
- `net/` поддерживает egress IP checks и прокси-маршрутизацию для валидации VPN режима.  
- Добавлен отдельный offline-конфиг `.cargo/config.offline.toml` и скрипты, которые включают его только после подготовки `vendor/`.

## Структура
- `apps/plus-desktop` — запуск приложения и склейка модулей.
- `engine` — policy engine (SOP baseline, file:// restrictions, VPN route mode).
- `renderer` — нативный webview runtime.
- `net` — HTTP client + history + egress checks.
- `vpn` — import + secure storage + sing-box control plane.
- `adblock` — ABP-compatible filtering.
- `yandex` — Yandex-only omnibox/new-tab.
- `privacy` — профиль и privacy storage.

## Запуск
```bash
cargo run -p plus-desktop
```
Diagnostics UI: `plus://diagnostics-ui`

## Как применяется Adblock/VPN к WebView
- WebView использует локальный HTTP proxy (adblock proxy), который перехватывает **все** запросы и блокирует домены по ABP‑правилам.
- При активном VPN adblock‑proxy устанавливает соединения через локальный SOCKS5 sing-box.

## VPN запуск
```bash
export PLUS_SINGBOX_BIN=/path/to/sing-box
export PLUS_VPN_IMPORT='vless://user@example.com:443'
cargo run -p plus-desktop
```

## Тесты
```bash
cargo test --workspace
```
Интеграционный VPN-тест автоматически `skip`, если нет `PLUS_TEST_VPN_URL` и `PLUS_SINGBOX_BIN`.

## Offline vendor
### Включить offline режим
```bash
tools/vendorize.sh
```
После этого появится `.cargo/config.toml`, и оффлайн сборка будет работать:
```bash
cargo build --offline
```

### Скачать vendor из CI артефакта (если есть)
```bash
export PLUS_VENDOR_URL=https://example.com/vendor.tar.zst
bash installer/fetch_vendor.sh
```
