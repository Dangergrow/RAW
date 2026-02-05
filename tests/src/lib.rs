#[cfg(test)]
mod smoke {
    use plus_adblock::AdblockEngine;
    use plus_engine::Engine;
    use plus_vpn::{VpnManager, VpnMode};

    #[test]
    fn render_text_smoke() {
        let doc = Engine::new(800.0, 600.0).parse_and_layout("<h1>Smoke</h1>");
        assert!(doc.body_text.contains("Smoke"));
    }

    #[test]
    fn adblock_smoke() {
        let mut engine = AdblockEngine::from_filter_list("||ads.test^").unwrap();
        assert!(engine.should_block("https://ads.test/banner.js"));
    }

    #[test]
    fn vpn_smoke() {
        let mut vpn = VpnManager::new();
        vpn.import("trojan://password@127.0.0.1:1080", VpnMode::Global)
            .unwrap();
        assert!(vpn.browser_proxy().unwrap().contains("127.0.0.1"));
    }
}
