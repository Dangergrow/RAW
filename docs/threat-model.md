# Threat Model
## Threats
1. Local credential theft.
2. Untrusted page scripts and tracking.
3. VPN config leakage.
4. Supply chain dependency tampering.

## Mitigations
- Keyring-backed secret storage + encrypted fallback.
- Adblock network filtering + browser policy constraints.
- Isolated profile directory with strict permissions.
- Cargo vendor source pinning for reproducible/offline builds.
