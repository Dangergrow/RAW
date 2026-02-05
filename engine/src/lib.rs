use html5ever::{parse_document, tendril::TendrilSink};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    pub color: [u8; 4],
    pub background: [u8; 4],
    pub font_size: f32,
    pub block: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            color: [30, 30, 30, 255],
            background: [255, 255, 255, 0],
            font_size: 16.0,
            block: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutBox {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub style: Style,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Document {
    pub title: String,
    pub body_text: String,
    pub boxes: Vec<LayoutBox>,
}

#[derive(Debug, Clone)]
pub struct Engine {
    pub viewport_width: f32,
    pub viewport_height: f32,
}

impl Engine {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            viewport_width,
            viewport_height,
        }
    }

    pub fn parse_and_layout(&self, html: &str) -> Document {
        let dom = parse_document(RcDom::default(), Default::default()).one(html);
        let mut texts = Vec::new();
        let mut title = String::new();
        walk(dom.document.clone(), &mut texts, &mut title);

        let mut y = 12.0;
        let mut boxes = Vec::new();
        for t in texts.iter().filter(|s| !s.trim().is_empty()) {
            let stripped = collapse_ws(t);
            let lines = wrap_line(&stripped, 90);
            for line in lines {
                boxes.push(LayoutBox {
                    text: line,
                    x: 12.0,
                    y,
                    width: self.viewport_width - 24.0,
                    height: 20.0,
                    style: Style::default(),
                });
                y += 22.0;
            }
            y += 4.0;
        }

        Document {
            title,
            body_text: texts.join("\n"),
            boxes,
        }
    }

    pub fn run_basic_js(&self, doc: &mut Document, script: &str) {
        if script.contains("document.title") && script.contains('=') {
            if let Some(val) = script.split('=').nth(1) {
                let cleaned = val
                    .trim()
                    .trim_matches(';')
                    .trim_matches('"')
                    .trim_matches('\'');
                doc.title = cleaned.to_string();
            }
        }
        if script.contains("appendText") {
            if let Some(start) = script.find('(') {
                if let Some(end) = script.rfind(')') {
                    let arg = script[start + 1..end]
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'');
                    doc.boxes.push(LayoutBox {
                        text: arg.to_string(),
                        x: 12.0,
                        y: doc.boxes.last().map(|b| b.y + 28.0).unwrap_or(12.0),
                        width: self.viewport_width - 24.0,
                        height: 20.0,
                        style: Style::default(),
                    });
                }
            }
        }
    }
}

fn walk(handle: Handle, texts: &mut Vec<String>, title: &mut String) {
    match &handle.data {
        NodeData::Text { contents } => texts.push(contents.borrow().to_string()),
        NodeData::Element { name, .. } if name.local.as_ref() == "title" => {
            for child in handle.children.borrow().iter() {
                if let NodeData::Text { contents } = &child.data {
                    *title = contents.borrow().to_string();
                }
            }
        }
        _ => {}
    }

    for child in handle.children.borrow().iter() {
        walk(child.clone(), texts, title);
    }
}

fn collapse_ws(input: &str) -> String {
    let re = Regex::new(r"\s+").expect("regex");
    re.replace_all(input.trim(), " ").to_string()
}

fn wrap_line(input: &str, max_chars: usize) -> Vec<String> {
    if input.len() <= max_chars {
        return vec![input.to_string()];
    }
    let mut out = Vec::new();
    let mut current = String::new();
    for w in input.split(' ') {
        if current.len() + w.len() + 1 > max_chars {
            out.push(current.clone());
            current.clear();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(w);
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_title_and_text() {
        let engine = Engine::new(1024.0, 800.0);
        let doc = engine.parse_and_layout(
            "<html><head><title>Hi</title></head><body><h1>Hello</h1></body></html>",
        );
        assert_eq!(doc.title, "Hi");
        assert!(doc.body_text.contains("Hello"));
        assert!(!doc.boxes.is_empty());
    }
}
