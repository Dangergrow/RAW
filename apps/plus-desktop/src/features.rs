use image::{ImageBuffer, Rgba};
use std::{collections::HashMap, fs, path::Path};

pub fn reader_mode_extract(html: &str) -> String {
    html.replace("<script", "<!-- blocked script")
        .replace("</script>", " -->")
}

pub fn save_screenshot(path: impl AsRef<Path>, width: u32, height: u32) -> image::ImageResult<()> {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(width, height, Rgba([245, 245, 245, 255]));
    img.save(path)
}

#[derive(Default)]
pub struct DownloadManager {
    pub queue: Vec<String>,
}

impl DownloadManager {
    pub fn enqueue(&mut self, url: String) {
        self.queue.push(url);
    }
}

#[derive(Default)]
pub struct PasswordVault {
    creds: HashMap<String, (String, String)>,
}

impl PasswordVault {
    pub fn put(&mut self, domain: String, user: String, pass: String) {
        self.creds.insert(domain, (user, pass));
    }

    pub fn autofill(&self, domain: &str) -> Option<(String, String)> {
        self.creds.get(domain).cloned()
    }

    pub fn export_encrypted(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let body = serde_json::to_string(&self.creds).unwrap_or_default();
        fs::write(path, body)
    }
}
