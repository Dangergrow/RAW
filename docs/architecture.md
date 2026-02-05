# Architecture
- `apps/plus-desktop`: UI, tabs, navigation, settings.
- `engine`: parse HTML and produce layout boxes.
- `renderer`: compositing and rasterization pipeline.
- `net`: URL loading, history storage, basic policy checks.
- `privacy`: telemetry/cookies policies and storage.
- `vpn`: import formats + secure secret storage + proxy routing mode.
- `adblock`: AdGuard-like filtering engine.
- `yandex`: search and new-tab Yandex-only services.
