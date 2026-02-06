# Regression Checklist (Windows)

Run before every PR:
```powershell
./tools/check_windows.ps1
```

Manual smoke:
1. `cargo run -p plus-desktop`
2. Open two tabs, navigate to yandex.ru
3. Open Diagnostics (🛡) and verify:
   - Proxy status is ON
   - Adblock stats increase on blocked URL
4. If VPN env vars configured, run Check IP and confirm IP changes.
