# Security Notes
- Telemetry disabled by default.
- `file://` blocked by policy unless explicitly allowed.
- VPN secrets stored in OS keyring, encrypted fallback is AES-GCM-SIV.
- Profile directory permissions restricted (0700 on Unix).
