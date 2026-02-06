use anyhow::Result;
use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
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
