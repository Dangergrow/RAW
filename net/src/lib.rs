use anyhow::Result;
use chrono::Utc;
use plus_adblock::AdblockEngine;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
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
    filter_list: String,
    upstream_socks: Option<String>,
) -> Result<ProxyHandle> {
    let listener = TcpListener::bind(listen_addr)?;
    let addr = listener.local_addr()?;
    std::thread::spawn(move || {
        let mut adblock = match AdblockEngine::from_filter_list(&filter_list) {
            Ok(engine) => engine,
            Err(err) => {
                eprintln!("Не удалось инициализировать adblock прокси: {err}");
                return;
            }
        };
        adblock.set_enabled(true);
        for stream in listener.incoming() {
            let Ok(stream) = stream else {
                continue;
            };
            let _ = handle_client(stream, &mut adblock, upstream_socks.as_deref());
        }
    });
    Ok(ProxyHandle {
        listen_addr: format!("{}", addr),
    })
}

fn handle_client(
    mut client: TcpStream,
    adblock: &mut AdblockEngine,
    upstream_socks: Option<&str>,
) -> Result<()> {
    let mut buf = [0u8; 4096];
    let n = client.read(&mut buf)?;
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
        if adblock.should_block(&url, "about:proxy", "connect") {
            client.write_all(b"HTTP/1.1 403 Forbidden\r\n\r\n")?;
            return Ok(());
        }
        let mut upstream = connect_upstream(&host_port, upstream_socks)?;
        client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")?;
        tunnel(client, upstream)?;
    } else {
        let url = Url::parse(target)?;
        let host = url.host_str().unwrap_or_default();
        let port = url.port_or_known_default().unwrap_or(80);
        if adblock.should_block(url.as_str(), "about:proxy", "http") {
            client.write_all(b"HTTP/1.1 403 Forbidden\r\n\r\n")?;
            return Ok(());
        }
        let mut upstream = connect_upstream(&format!("{}:{}", host, port), upstream_socks)?;
        upstream.write_all(&buf[..n])?;
        tunnel(client, upstream)?;
    }
    Ok(())
}

fn connect_upstream(target: &str, upstream_socks: Option<&str>) -> Result<TcpStream> {
    if let Some(socks) = upstream_socks {
        let url = Url::parse(&format!("socks5://{}", socks))?;
        let host = url.host_str().unwrap_or("127.0.0.1");
        let port = url.port().unwrap_or(1080);
        let mut stream = TcpStream::connect((host, port))?;
        socks5_connect(&mut stream, target)?;
        Ok(stream)
    } else {
        let mut split = target.split(':');
        let host = split.next().unwrap_or("127.0.0.1");
        let port = split
            .next()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(80);
        Ok(TcpStream::connect((host, port))?)
    }
}

fn socks5_connect(stream: &mut TcpStream, target: &str) -> Result<()> {
    stream.write_all(&[0x05, 0x01, 0x00])?;
    let mut resp = [0u8; 2];
    stream.read_exact(&mut resp)?;
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
    stream.write_all(&req)?;
    let mut reply = [0u8; 10];
    stream.read_exact(&mut reply)?;
    if reply[1] != 0x00 {
        anyhow::bail!("SOCKS connect failed");
    }
    Ok(())
}

fn tunnel(mut client: TcpStream, mut upstream: TcpStream) -> Result<()> {
    let mut client_read = client.try_clone()?;
    let mut upstream_read = upstream.try_clone()?;
    let client_to_upstream = std::thread::spawn(move || {
        let _ = std::io::copy(&mut client_read, &mut upstream);
    });
    let upstream_to_client = std::thread::spawn(move || {
        let _ = std::io::copy(&mut upstream_read, &mut client);
    });
    let _ = client_to_upstream.join();
    let _ = upstream_to_client.join();
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
