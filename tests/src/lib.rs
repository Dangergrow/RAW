#[cfg(test)]
mod smoke {
    use plus_adblock::AdblockEngine;
    use plus_net::{start_proxy, NetClient};
    use plus_vpn::{VpnManager, VpnMode};
    use std::sync::{Arc, Mutex};

    #[test]
    fn adblock_blocks_tracker() {
        let mut ad = AdblockEngine::from_filter_list("||doubleclick.net^").unwrap();
        assert!(ad.should_block(
            "https://doubleclick.net/pixel.js",
            "https://example.org",
            "script"
        ));
    }

    #[tokio::test]
    async fn proxy_blocks_blocked_domain() {
        let mut ad = AdblockEngine::from_filter_list("||blocked.example^").unwrap();
        ad.set_enabled(true);
        let ad = Arc::new(Mutex::new(ad));
        let proxy = start_proxy("127.0.0.1:0", ad, None).await.unwrap();
        let client = reqwest::Client::builder()
            .proxy(reqwest::Proxy::http(format!("http://{}", proxy.listen_addr)).unwrap())
            .build()
            .unwrap();
        let resp = client.get("http://blocked.example/").send().await.unwrap();
        assert_eq!(resp.status().as_u16(), 403);
    }

    #[tokio::test]
    async fn vpn_changes_egress_when_env_configured() {
        let Some(vpn_url) = std::env::var("PLUS_TEST_VPN_URL").ok() else {
            eprintln!("SKIP: PLUS_TEST_VPN_URL is not set");
            return;
        };
        let Some(singbox_bin) = std::env::var("PLUS_SINGBOX_BIN").ok() else {
            eprintln!("SKIP: PLUS_SINGBOX_BIN is not set");
            return;
        };

        let baseline_client = NetClient::new(None).unwrap();
        let baseline_ip = baseline_client.get_egress_ip().await.unwrap();

        let dir = tempfile::tempdir().unwrap();
        let mut vpn = VpnManager::new(singbox_bin, dir.path());
        vpn.import(&vpn_url, VpnMode::Global, true).unwrap();
        vpn.start_core().await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let proxied_client = NetClient::new(vpn.browser_proxy()).unwrap();
        let proxied_ip = proxied_client.get_egress_ip().await.unwrap();
        let _ = vpn.stop_core().await;

        assert_ne!(baseline_ip.trim(), proxied_ip.trim());
    }
}
