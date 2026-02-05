# Threat Model
## Threats
1. Credential theft from local storage.
2. Malicious script/network response injection.
3. Proxy/VPN config exfiltration.
4. Supply-chain dependency compromise.

## Mitigations
- Encrypted secret persistence and keychain usage.
- Block `file://` and enforce URL parsing/SOP baseline in network module.
- Scoped profile storage and SQLite-backed local data controls.
- Pinned lockfile and CI checks.
