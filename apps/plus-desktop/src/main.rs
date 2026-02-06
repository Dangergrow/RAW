use anyhow::Result;
use plus_adblock::AdblockEngine;
use plus_engine::{BrowserPolicy, EngineController};
use plus_privacy::{ensure_profile_dir, PrivacyStore};
use plus_renderer::run_desktop_browser;
use plus_vpn::{VpnManager, VpnMode};
use std::sync::{Arc, Mutex};

fn bootstrap() -> Result<()> {
    let profile = ensure_profile_dir("default")?;
    let settings_db = profile.join("privacy.db");
    let privacy = PrivacyStore::open(settings_db)?;
    privacy.save_setting("telemetry_enabled", "false")?;

    let mut vpn = VpnManager::new(
        std::env::var("PLUS_SINGBOX_BIN").unwrap_or_else(|_| "sing-box".into()),
        profile.join("vpn"),
    );
    let mut proxy = None;
    if let Ok(url) = std::env::var("PLUS_VPN_IMPORT") {
        let _ = vpn.import(&url, VpnMode::Global, true)?;
        let _ = vpn.store_secure(
            "plus-browser",
            "vpn-default",
            profile.join("vpn.sec"),
            "plus-local-passphrase",
        );
        proxy = vpn.browser_proxy();
    }

    let mut adblock =
        AdblockEngine::from_filter_list("||doubleclick.net^\n||googlesyndication.com^")?;
    adblock.set_enabled(true);
    let adblock = Arc::new(Mutex::new(adblock));

    let engine = EngineController::new(BrowserPolicy::default());
    run_desktop_browser(engine, "Plus", proxy, adblock)?;
    Ok(())
}

fn main() {
    if let Err(e) = bootstrap() {
        eprintln!("Plus startup error: {e}");
        std::process::exit(1);
    }
}
