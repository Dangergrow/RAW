# Plus Browser (Windows-first MVP)

## Архитектура
- **Native UI**: egui + wgpu (Tabs/Navigation/Settings)
- **Content**: WebView2 (child HWND) только для страниц
- **Network**: local HTTP proxy → adblock → optional sing-box SOCKS5

## Запуск (Windows)
См. `docs/WINDOWS_BUILD.md`.

## Diagnostics
- Native panel: кнопка 🛡 в Tabs bar
- Check IP: показывает текущий IP через ipify

## Яндекс-only
- Поиск и новая вкладка работают только с Яндекс

## Как применяется Adblock/VPN
- WebView2 → local HTTP proxy → adblock → (VPN SOCKS5 если включён)

## Документация
- User guide: `docs/USER_GUIDE.md`
- Limitations: `docs/LIMITATIONS.md`
- Regression: `docs/REGRESSION.md`
- Windows build: `docs/WINDOWS_BUILD.md`


## Проверки (Windows)
```powershell
./tools/check_windows.ps1
```
