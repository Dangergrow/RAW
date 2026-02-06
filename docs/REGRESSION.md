# Regression Checklist

Run before every PR:
```bash
tools/check.sh
```

Manual smoke:
1. `cargo run -p plus-desktop`
2. Open `plus://diagnostics-ui` and verify:
   - Proxy status is ON
   - Adblock stats increase on blocked URL
3. If VPN env vars configured, run Check IP and confirm IP changes.
