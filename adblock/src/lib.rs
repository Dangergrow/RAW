use adblock::engine::Engine;
use adblock::lists::ParseOptions;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdblockStats {
    pub blocked: u64,
    pub allowed: u64,
}

pub struct AdblockEngine {
    engine: Engine,
    pub stats: AdblockStats,
    enabled: bool,
    whitelist: Vec<String>,
    last_blocked: VecDeque<String>,
}

impl AdblockEngine {
    pub fn from_filter_list(list: &str) -> Result<Self> {
        let mut engine = Engine::new(true);
        engine
            .from_rules(
                &list.lines().map(ToOwned::to_owned).collect::<Vec<_>>(),
                ParseOptions::default(),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok(Self {
            engine,
            stats: AdblockStats::default(),
            enabled: true,
            whitelist: Vec::new(),
            last_blocked: VecDeque::with_capacity(32),
        })
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn add_whitelist_host(&mut self, host: String) {
        self.whitelist.push(host);
    }

    pub fn should_block(&mut self, url: &str, source_url: &str, resource_type: &str) -> bool {
        if !self.enabled || self.whitelist.iter().any(|w| url.contains(w)) {
            self.stats.allowed += 1;
            return false;
        }
        let matched = self
            .engine
            .check_network_urls(url, source_url, resource_type)
            .matched;
        if matched {
            self.stats.blocked += 1;
            self.track_block(url);
        } else {
            self.stats.allowed += 1;
        }
        matched
    }

    pub fn last_blocked(&self) -> Vec<String> {
        self.last_blocked.iter().cloned().collect()
    }

    fn track_block(&mut self, url: &str) {
        if self.last_blocked.len() == 32 {
            self.last_blocked.pop_front();
        }
        self.last_blocked.push_back(url.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_known_tracker_rule() {
        let rules = "||doubleclick.net^";
        let mut ad = AdblockEngine::from_filter_list(rules).unwrap();
        assert!(ad.should_block(
            "https://doubleclick.net/track.js",
            "https://example.org",
            "script"
        ));
    }
}
