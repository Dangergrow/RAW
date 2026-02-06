use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserPolicy {
    pub allow_file_scheme: bool,
    pub incognito: bool,
    pub telemetry_enabled: bool,
    pub yandex_only_search: bool,
    pub vpn_mode: VpnRouteMode,
    pub vpn_domain_list: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VpnRouteMode {
    Off,
    Global,
    DomainList,
}

impl Default for BrowserPolicy {
    fn default() -> Self {
        Self {
            allow_file_scheme: false,
            incognito: false,
            telemetry_enabled: false,
            yandex_only_search: true,
            vpn_mode: VpnRouteMode::Off,
            vpn_domain_list: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineController {
    policy: BrowserPolicy,
}

impl EngineController {
    pub fn new(policy: BrowserPolicy) -> Self {
        Self { policy }
    }

    pub fn policy(&self) -> &BrowserPolicy {
        &self.policy
    }

    pub fn set_vpn_mode(&mut self, mode: VpnRouteMode, domains: Vec<String>) {
        self.policy.vpn_mode = mode;
        self.policy.vpn_domain_list = domains.into_iter().collect();
    }

    pub fn validate_navigation(&self, url: &str) -> Result<()> {
        let parsed = Url::parse(url)?;
        if parsed.scheme() == "file" && !self.policy.allow_file_scheme {
            bail!("file:// URLs are forbidden by policy")
        }
        Ok(())
    }

    pub fn should_route_via_vpn(&self, url: &str) -> bool {
        let Ok(parsed) = Url::parse(url) else {
            return false;
        };
        match self.policy.vpn_mode {
            VpnRouteMode::Off => false,
            VpnRouteMode::Global => true,
            VpnRouteMode::DomainList => parsed
                .host_str()
                .map(|h| self.policy.vpn_domain_list.contains(h))
                .unwrap_or(false),
        }
    }
}
