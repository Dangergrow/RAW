# Сборка на Windows

## Требования
- Rust MSVC toolchain (`rustup default stable-x86_64-pc-windows-msvc`)
- Microsoft Build Tools / Visual Studio Build Tools
- WebView2 Runtime (установлен)

## Сборка
```powershell
cargo build --workspace
```

## Запуск
```powershell
cargo run -p plus-desktop
```

## Типичные ошибки и решения
- **Нет WebView2 Runtime**: установите WebView2 Runtime с сайта Microsoft.
- **Блокировка firewall**: разрешите доступ приложению.
- **Права записи**: запускайте из каталога с правами на запись.
