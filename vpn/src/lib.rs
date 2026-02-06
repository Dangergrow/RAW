use aes_gcm_siv::{
    aead::{Aead, KeyInit},
    Aes256GcmSiv, Nonce,
};
use anyhow::{anyhow, Result};
use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::process::{Child, Command};
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
    pub dns_via_tunnel: bool,
}

pub struct VpnManager {
    pub active: Option<VpnConfig>,
    child: Option<Child>,
    local_socks: String,
    core_bin: PathBuf,
    workdir: PathBuf,
}

impl VpnManager {
    pub fn new(core_bin: impl AsRef<Path>, workdir: impl AsRef<Path>) -> Self {
        Self {
            active: None,
            child: None,
            local_socks: "127.0.0.1:2080".into(),
            core_bin: core_bin.as_ref().to_path_buf(),
            workdir: workdir.as_ref().to_path_buf(),
        }
    }

    pub fn import(
        &mut self,
        input: &str,
        mode: VpnMode,
        dns_via_tunnel: bool,
    ) -> Result<VpnConfig> {
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
                dns_via_tunnel,
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
                dns_via_tunnel,
            }
        };
        self.active = Some(config.clone());
        Ok(config)
    }

    pub fn browser_proxy(&self) -> Option<String> {
        self.active
            .as_ref()
            .map(|_| format!("socks5h://{}", self.local_socks))
    }

    pub async fn start_core(&mut self) -> Result<()> {
        let cfg = self
            .active
            .clone()
            .ok_or_else(|| anyhow!("vpn config missing"))?;
        fs::create_dir_all(&self.workdir)?;
        let cfg_file = self.workdir.join("singbox-config.json");
        let outbound = if cfg.protocol == "json" {
            serde_json::from_str::<serde_json::Value>(&cfg.raw)?
        } else {
            serde_json::json!({
                "type": cfg.protocol,
                "server": cfg.endpoint.split(':').next().unwrap_or("127.0.0.1"),
                "server_port": cfg.endpoint.split(':').nth(1).and_then(|p| p.parse::<u16>().ok()).unwrap_or(443),
                "tag": "proxy"
            })
        };
        let dns = if cfg.dns_via_tunnel {
            serde_json::json!({"strategy":"prefer_ipv4"})
        } else {
            serde_json::json!({})
        };
        let full = serde_json::json!({
            "log": {"level": "warn"},
            "dns": dns,
            "inbounds": [{"type": "socks", "listen": "127.0.0.1", "listen_port": 2080, "tag":"plus-in"}],
            "outbounds": [outbound, {"type":"direct","tag":"direct"}],
            "route": {"final": "proxy"}
        });
        fs::write(&cfg_file, serde_json::to_vec_pretty(&full)?)?;

        let child = Command::new(&self.core_bin)
            .arg("run")
            .arg("-c")
            .arg(cfg_file)
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .spawn()?;
        self.child = Some(child);
        Ok(())
    }

    pub async fn stop_core(&mut self) -> Result<()> {
        if let Some(child) = &mut self.child {
            child.kill().await?;
        }
        self.child = None;
        Ok(())
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
