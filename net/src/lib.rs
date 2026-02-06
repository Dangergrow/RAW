use anyhow::Result;
use chrono::Utc;
use plus_adblock::AdblockEngine;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    pub url: String,
    pub status: u16,
    pub body: String,
    pub content_type: String,
}

pub struct NetClient {
    client: reqwest::Client,
}

impl NetClient {
    pub fn new(proxy: Option<String>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("PlusBrowser/0.2"));
        let mut builder = reqwest::Client::builder().default_headers(headers);
        if let Some(proxy_url) = proxy {
            builder = builder.proxy(reqwest::Proxy::all(proxy_url)?);
        }
        Ok(Self {
            client: builder.build()?,
        })
    }

    pub async fn get(&self, url: &str) -> Result<ResponseData> {
        let parsed = Url::parse(url)?;
        if parsed.scheme() == "file" {
            anyhow::bail!("file:// URLs are blocked by default");
        }
        let resp = self.client.get(url).send().await?;
        let status = resp.status().as_u16();
        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/plain")
            .to_string();
        let body = resp.text().await?;
        Ok(ResponseData {
            url: url.to_string(),
            status,
            body,
            content_type,
        })
    }

    pub async fn get_egress_ip(&self) -> Result<String> {
        Ok(self
            .client
            .get("https://api.ipify.org")
            .send()
            .await?
            .text()
            .await?)
    }
}

pub struct HistoryStore {
    conn: Connection,
}

pub struct ProxyHandle {
    pub listen_addr: String,
}

pub async fn start_proxy(
    listen_addr: &str,
    adblock: Arc<Mutex<AdblockEngine>>,
    upstream_socks: Option<String>,
) -> Result<ProxyHandle> {
    let listener = TcpListener::bind(listen_addr).await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            let ad = adblock.clone();
            let upstream = upstream_socks.clone();
            tokio::spawn(async move {
                let _ = handle_client(stream, ad, upstream).await;
            });
        }
    });
    Ok(ProxyHandle {
        listen_addr: format!("{}", addr),
    })
}

async fn handle_client(
    mut client: TcpStream,
    adblock: Arc<Mutex<AdblockEngine>>,
    upstream_socks: Option<String>,
) -> Result<()> {
    let mut buf = [0u8; 4096];
    let n = client.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }
    let req = String::from_utf8_lossy(&buf[..n]);
    let mut lines = req.lines();
    let Some(request_line) = lines.next() else {
        return Ok(());
    };
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }
    let method = parts[0];
    let target = parts[1];
    if method.eq_ignore_ascii_case("CONNECT") {
        let host_port = target.to_string();
        let url = format!("https://{}/", host_port);
        let mut ad = adblock.lock().expect("adblock lock");
        if ad.should_block(&url, "about:proxy", "connect") {
            client.write_all(b"HTTP/1.1 403 Forbidden\r\n\r\n").await?;
            return Ok(());
        }
        let mut upstream = connect_upstream(&host_port, upstream_socks).await?;
        client
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await?;
        tokio::io::copy_bidirectional(&mut client, &mut upstream).await?;
    } else {
        let url = Url::parse(target)?;
        let host = url.host_str().unwrap_or_default();
        let port = url.port_or_known_default().unwrap_or(80);
        let mut ad = adblock.lock().expect("adblock lock");
        if ad.should_block(url.as_str(), "about:proxy", "http") {
            client.write_all(b"HTTP/1.1 403 Forbidden\r\n\r\n").await?;
            return Ok(());
        }
        let mut upstream = connect_upstream(&format!("{}:{}", host, port), upstream_socks).await?;
        upstream.write_all(&buf[..n]).await?;
        tokio::io::copy_bidirectional(&mut client, &mut upstream).await?;
    }
    Ok(())
}

async fn connect_upstream(target: &str, upstream_socks: Option<String>) -> Result<TcpStream> {
    if let Some(socks) = upstream_socks {
        let url = Url::parse(&format!("socks5://{}", socks))?;
        let host = url.host_str().unwrap_or("127.0.0.1");
        let port = url.port().unwrap_or(1080);
        let mut stream = TcpStream::connect((host, port)).await?;
        socks5_connect(&mut stream, target).await?;
        Ok(stream)
    } else {
        let mut split = target.split(':');
        let host = split.next().unwrap_or("127.0.0.1");
        let port = split
            .next()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(80);
        Ok(TcpStream::connect((host, port)).await?)
    }
}

async fn socks5_connect(stream: &mut TcpStream, target: &str) -> Result<()> {
    stream.write_all(&[0x05, 0x01, 0x00]).await?;
    let mut resp = [0u8; 2];
    stream.read_exact(&mut resp).await?;
    if resp[1] != 0x00 {
        anyhow::bail!("SOCKS auth failed");
    }
    let mut parts = target.split(':');
    let host = parts.next().unwrap_or("127.0.0.1");
    let port = parts
        .next()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(80);
    let mut req = vec![0x05, 0x01, 0x00, 0x03, host.len() as u8];
    req.extend_from_slice(host.as_bytes());
    req.extend_from_slice(&port.to_be_bytes());
    stream.write_all(&req).await?;
    let mut reply = [0u8; 10];
    stream.read_exact(&mut reply).await?;
    if reply[1] != 0x00 {
        anyhow::bail!("SOCKS connect failed");
    }
    Ok(())
}

impl HistoryStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY,
                url TEXT NOT NULL,
                title TEXT NOT NULL,
                visited_at TEXT NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn add_visit(&self, url: &str, title: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO history(url, title, visited_at) VALUES(?1, ?2, ?3)",
            params![url, title, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }
}
