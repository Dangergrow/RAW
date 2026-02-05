use image::{Rgba, RgbaImage};
use plus_engine::Document;

pub struct Renderer {
    pub width: u32,
    pub height: u32,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn render_document(&self, doc: &Document) -> RgbaImage {
        let mut img = RgbaImage::from_pixel(self.width, self.height, Rgba([250, 250, 250, 255]));
        for b in &doc.boxes {
            let y = b.y.max(0.0) as u32;
            if y >= self.height {
                continue;
            }
            let h = b.height as u32;
            for yy in y..(y + h).min(self.height) {
                for xx in 10..(self.width - 10) {
                    if yy == y || yy == (y + h - 1).min(self.height - 1) {
                        img.put_pixel(xx, yy, Rgba([225, 225, 225, 255]));
                    }
                }
            }
        }
        img
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plus_engine::Engine;

    #[test]
    fn renders_non_empty_image() {
        let doc = Engine::new(800.0, 600.0).parse_and_layout("<body>Hello renderer</body>");
        let img = Renderer::new(400, 300).render_document(&doc);
        assert_eq!(img.width(), 400);
        assert_eq!(img.height(), 300);
    }
}
