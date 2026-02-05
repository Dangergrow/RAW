use aes_gcm_siv::{
    aead::{Aead, KeyInit},
    Aes256GcmSiv, Nonce,
};
use anyhow::{anyhow, Result};
use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VpnMode {
    Off,
    Global,
    DomainList(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnConfig {
    pub protocol: String,
    pub endpoint: String,
    pub raw: String,
    pub mode: VpnMode,
}

pub struct VpnManager {
    pub active: Option<VpnConfig>,
}

impl VpnManager {
    pub fn new() -> Self {
        Self { active: None }
    }

    pub fn import(&mut self, input: &str, mode: VpnMode) -> Result<VpnConfig> {
        let config = if input.trim_start().starts_with('{') {
            let v: serde_json::Value = serde_json::from_str(input)?;
            let endpoint = v
                .get("server")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown")
                .to_string();
            VpnConfig {
                protocol: "json".into(),
                endpoint,
                raw: input.to_string(),
                mode,
            }
        } else {
            let url = Url::parse(input)?;
            let protocol = url.scheme().to_string();
            if !["vmess", "vless", "trojan", "ss"].contains(&protocol.as_str()) {
                return Err(anyhow!("unsupported vpn scheme"));
            }
            let endpoint = format!(
                "{}:{}",
                url.host_str().unwrap_or("unknown"),
                url.port().unwrap_or(443)
            );
            VpnConfig {
                protocol,
                endpoint,
                raw: input.to_string(),
                mode,
            }
        };
        self.active = Some(config.clone());
        Ok(config)
    }

    pub fn browser_proxy(&self) -> Option<String> {
        self.active
            .as_ref()
            .map(|cfg| format!("socks5h://{}", cfg.endpoint))
    }

    pub fn store_secure(
        &self,
        service: &str,
        account: &str,
        path_fallback: impl AsRef<Path>,
        passphrase: &str,
    ) -> Result<()> {
        let raw = serde_json::to_string(&self.active)?;
        if let Ok(entry) = keyring::Entry::new(service, account) {
            if entry.set_password(&raw).is_ok() {
                return Ok(());
            }
        }

        let mut nonce = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce);
        let key = derive_key(passphrase);
        let cipher = Aes256GcmSiv::new_from_slice(&key)?;
        let data = cipher
            .encrypt(Nonce::from_slice(&nonce), raw.as_bytes())
            .map_err(|_| anyhow!("encrypt failed"))?;
        let blob = format!(
            "{}:{}",
            base64::engine::general_purpose::STANDARD.encode(nonce),
            base64::engine::general_purpose::STANDARD.encode(data)
        );
        fs::write(path_fallback, blob)?;
        Ok(())
    }
}

fn derive_key(passphrase: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(passphrase.as_bytes());
    let out = hasher.finalize();
    out.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_vmess() {
        let mut m = VpnManager::new();
        let cfg = m
            .import("vless://user@example.com:443", VpnMode::Global)
            .unwrap();
        assert_eq!(cfg.protocol, "vless");
        assert!(m.browser_proxy().is_some());
    }
}
