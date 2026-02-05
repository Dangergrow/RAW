use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdblockStats {
    pub blocked: u64,
    pub allowed: u64,
}

pub struct AdblockEngine {
    patterns: Vec<Regex>,
    exceptions: Vec<Regex>,
    pub stats: AdblockStats,
}

impl AdblockEngine {
    pub fn from_filter_list(list: &str) -> Result<Self> {
        let mut patterns = Vec::new();
        let mut exceptions = Vec::new();
        for line in list.lines().map(str::trim).filter(|l| !l.is_empty()) {
            if line.starts_with('!') || line.starts_with('[') {
                continue;
            }
            let (target, is_exception) = if let Some(ex) = line.strip_prefix("@@") {
                (ex, true)
            } else {
                (line, false)
            };
            let escaped = regex::escape(target)
                .replace(r"\*", ".*")
                .replace(r"\^", "[/:?=&.]?");
            let re = Regex::new(&escaped)?;
            if is_exception {
                exceptions.push(re);
            } else {
                patterns.push(re);
            }
        }
        Ok(Self {
            patterns,
            exceptions,
            stats: AdblockStats::default(),
        })
    }

    pub fn should_block(&mut self, url: &str) -> bool {
        let Ok(parsed) = Url::parse(url) else {
            self.stats.allowed += 1;
            return false;
        };
        let hay = format!("{}{}", parsed.host_str().unwrap_or_default(), parsed.path());
        if self.exceptions.iter().any(|r| r.is_match(&hay)) {
            self.stats.allowed += 1;
            return false;
        }
        let blocked = self.patterns.iter().any(|r| r.is_match(&hay));
        if blocked {
            self.stats.blocked += 1;
        } else {
            self.stats.allowed += 1;
        }
        blocked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_ads() {
        let mut e =
            AdblockEngine::from_filter_list("||ads.example.com^\n@@||ads.example.com/good.js")
                .unwrap();
        assert!(e.should_block("https://ads.example.com/tracker.js"));
        assert!(!e.should_block("https://ads.example.com/good.js"));
    }
}
