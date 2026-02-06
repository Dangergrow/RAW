# Журнал изменений

## Unreleased
- Переход на нативный интерфейс egui с контентом WebView2.
- Документация Windows‑first и инструкции по проверкам.
- Обновление adblock до 0.12.1 для исправления сборки на Windows.

### Как проверить
```powershell
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
Ручной smoke:
- Запустить `cargo run -p plus-desktop`
- Открыть две вкладки, перейти на yandex.ru
- Открыть «Диагностика» и проверить AdBlock/Proxy
