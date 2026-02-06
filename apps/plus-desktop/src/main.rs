use anyhow::Result;
use plus_adblock::AdblockEngine;
use plus_engine::{BrowserPolicy, EngineController};
use plus_privacy::{ensure_profile_dir, PrivacyStore};
use plus_renderer::{run_desktop_browser, DiagnosticsInfo};
use plus_vpn::{VpnManager, VpnMode};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

fn bootstrap() -> Result<()> {
    let profile = ensure_profile_dir("default")?;
    let settings_db = profile.join("privacy.db");
    let privacy = PrivacyStore::open(settings_db)?;
    privacy.save_setting("telemetry_enabled", "false")?;

    let mut vpn = VpnManager::new(
        std::env::var("PLUS_SINGBOX_BIN").unwrap_or_else(|_| "sing-box".into()),
        profile.join("vpn"),
    );
    let runtime = Runtime::new()?;
    let mut proxy = None;
    let mut vpn_socks = None;
    if let Ok(url) = std::env::var("PLUS_VPN_IMPORT") {
        let _ = vpn.import(&url, VpnMode::Global, true)?;
        let _ = vpn.store_secure(
            "plus-browser",
            "vpn-default",
            profile.join("vpn.sec"),
            "plus-local-passphrase",
        );
        runtime.block_on(vpn.start_core())?;
        vpn_socks = vpn.browser_proxy().map(|p| p.replace("socks5h://", ""));
    }

    let mut adblock =
        AdblockEngine::from_filter_list("||doubleclick.net^\n||googlesyndication.com^")?;
    adblock.set_enabled(true);
    let adblock = Arc::new(Mutex::new(adblock));

    let proxy_handle = runtime.block_on(plus_net::start_proxy(
        "127.0.0.1:0",
        adblock.clone(),
        vpn_socks.clone(),
    ))?;
    proxy = Some(format!("http://{}", proxy_handle.listen_addr));

    let engine = EngineController::new(BrowserPolicy::default());
    let diagnostics = DiagnosticsInfo {
        vpn_mode: if vpn_socks.is_some() {
            "Global".to_string()
        } else {
            "Off".to_string()
        },
        socks5: vpn_socks.map(|s| format!("socks5h://{}", s)),
        proxy: proxy.clone(),
    };
    run_desktop_browser(engine, "Plus", proxy, adblock, diagnostics)?;
    Ok(())
}

fn main() {
    if let Err(e) = bootstrap() {
        eprintln!("Plus startup error: {e}");
        std::process::exit(1);
    }
}
