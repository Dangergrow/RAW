# Plus Browser MVP

Plus — кроссплатформенный десктопный MVP-браузер на Rust (Windows/macOS/Linux) с собственным модульным `engine/` + `renderer/`, без Electron/CEF.

## Выбранный стек
- **Rust**: основная логика и безопасность памяти.
- **GUI**: `eframe/egui` для нативного окна и кастомного интерфейса.
- **Engine**: `html5ever` для парсинга HTML + собственный слой layout-представления.
- **Renderer**: собственный растер в `renderer/` поверх `image` (MVP пайплайн compositing).
- **Сеть**: `reqwest` + URLLoader в `net/`.
- **Хранилище**: `SQLite` (`rusqlite`) для истории и privacy-настроек.
- **VPN**: импорт vmess/vless/trojan/ss/JSON, безопасное хранение через `keyring` или шифрованный vault.
- **Adblock**: AdGuard-подобные правила (`||`, `@@`, `*`, `^`) в `adblock/`.

## Запуск
```bash
cargo run -p plus-desktop
```

## Сборка
```bash
cargo build --workspace --release
```

## Тесты
```bash
cargo test --workspace
```

## Реализованные функции MVP
- Вкладки, омнибокс, загрузка URL/поиска, история.
- Только Яндекс в поиске и стартовой странице.
- Engine: HTML parse + text layout + простой JS-хук.
- Renderer: базовый raster/composite в изображение.
- Adblock с исключениями и статистикой.
- VPN import + режимы + прокси браузерного трафика.
- Тёмный режим сайтов (MVP стилизация текста).
- Privacy store (telemetry off by default), cookies clear.
- Подготовка CI и installer-скриптов.

## Ограничения MVP
- Layout/CSS/JS поддерживаются частично (стабильный subset для простых страниц).
- Reader mode/переводчик/полноценный менеджер паролей и загрузок — в текущем MVP реализованы базовыми внутренними структурами, без облачной синхронизации.
- VPN core работает через прокси endpoint из импортированного конфига; запуск внешнего xray/sing-box как отдельного core в этом репозитории не включён.

