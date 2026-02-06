#[cfg(test)]
mod smoke {
    use plus_adblock::AdblockEngine;
    use plus_net::NetClient;
    use plus_vpn::{VpnManager, VpnMode};

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
