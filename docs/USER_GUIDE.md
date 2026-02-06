# Plus Browser — User Guide

## Первые шаги
1. Запустите браузер: `cargo run -p plus-desktop`.
2. В адресной строке введите URL или запрос — поиск идёт через Яндекс.
3. Откройте Diagnostics: `plus://diagnostics-ui`.

## Горячие клавиши
- Ctrl/Cmd + T — новая вкладка
- Ctrl/Cmd + W — закрыть вкладку
- Ctrl/Cmd + L — фокус адресной строки

## VPN
1. Укажите переменные окружения:
   - `PLUS_SINGBOX_BIN=/path/to/sing-box`
   - `PLUS_VPN_IMPORT='vless://...'` или `vmess://` / `trojan://` / `ss://` / JSON
2. Запустите браузер — соединение поднимается через sing-box.
3. Проверить IP можно в Diagnostics.

## AdBlock
- Включён по умолчанию, статистика и последние блокировки доступны в Diagnostics.
- Для добавления whitelist домена используйте настройки (будет расширено).

## Диагностика и проблемы
Diagnostics отображает:
- Proxy и режим VPN
- Статистику блокировок и последние URL
- Проверку IP через ipify
