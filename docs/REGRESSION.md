# Регрессионные проверки (Windows)

## Обязательные проверки перед PR
```powershell
./tools/check_windows.ps1
```

## Ручной чек‑лист UI
1. `cargo run -p plus-desktop`
2. Открыть 2 вкладки, перейти на yandex.ru
3. Открыть «Диагностика» и проверить:
   - Proxy активен
   - AdBlock hits увеличивается
4. Если VPN настроен — нажать Check IP

## Оффлайн‑режим
1. `tools/vendorize.sh`
2. `cargo test --workspace --offline`

## Важно
- Adblock обновлён до 0.12.1 для совместимости со сборкой на Windows.
- WebView2 crate обновлён до 0.2, Windows‑host синхронизирован с новым API.
- Локальный HTTP‑proxy теперь обрабатывает соединения последовательно, чтобы избежать !Send проблем с adblock.
