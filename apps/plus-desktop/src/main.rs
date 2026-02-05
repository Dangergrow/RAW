mod features;
use anyhow::Result;
use eframe::{egui, App};
use plus_adblock::AdblockEngine;
use plus_engine::Engine;
use plus_net::{HistoryStore, NetClient};
use plus_renderer::Renderer;
use plus_vpn::{VpnManager, VpnMode};
use plus_yandex::{omnibox_to_url, yandex_tiles};
use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Default, Clone)]
struct Tab {
    title: String,
    url: String,
    content: String,
}

struct PlusApp {
    tabs: Vec<Tab>,
    active: usize,
    address: String,
    tx: Sender<(usize, String, String)>,
    rx: Receiver<(usize, String, String)>,
    runtime: tokio::runtime::Runtime,
    history: HistoryStore,
    adblock: AdblockEngine,
    vpn: VpnManager,
    dark_websites: bool,
}

impl PlusApp {
    fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let history = HistoryStore::open("plus-history.db")?;
        let adblock =
            AdblockEngine::from_filter_list("||doubleclick.net^\n||googlesyndication.com^")?;
        Ok(Self {
            tabs: vec![Tab {
                title: "Новая вкладка".into(),
                ..Default::default()
            }],
            active: 0,
            address: String::new(),
            tx,
            rx,
            runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?,
            history,
            adblock,
            vpn: VpnManager::new(),
            dark_websites: true,
        })
    }

    fn navigate(&mut self, input: &str) {
        let url = omnibox_to_url(input);
        let idx = self.active;
        self.tabs[idx].url = url.clone();
        let tx = self.tx.clone();
        let proxy = self.vpn.browser_proxy();

        self.runtime.spawn(async move {
            let net = NetClient::new(proxy).expect("client");
            let resp = net.get(&url).await;
            match resp {
                Ok(data) => {
                    let _ = tx.send((idx, data.url, data.body));
                }
                Err(err) => {
                    let _ = tx.send((
                        idx,
                        url,
                        format!(
                            "<html><body><h2>Ошибка загрузки</h2><p>{}</p></body></html>",
                            err
                        ),
                    ));
                }
            }
        });
    }
}

impl App for PlusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok((idx, url, body)) = self.rx.try_recv() {
            if idx < self.tabs.len() {
                self.tabs[idx].content = body.clone();
                let engine = Engine::new(1024.0, 900.0);
                let doc = engine.parse_and_layout(&body);
                self.tabs[idx].title = if doc.title.is_empty() {
                    url.clone()
                } else {
                    doc.title.clone()
                };
                let _ = self.history.add_visit(&url, &self.tabs[idx].title);
                let _ = Renderer::new(1200, 900).render_document(&doc);
            }
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, tab) in self.tabs.iter().enumerate() {
                    let label = if tab.title.is_empty() {
                        "Новая"
                    } else {
                        &tab.title
                    };
                    if ui.selectable_label(i == self.active, label).clicked() {
                        self.active = i;
                    }
                }
                if ui.button("+").clicked() {
                    self.tabs.push(Tab {
                        title: "Новая вкладка".into(),
                        ..Default::default()
                    });
                    self.active = self.tabs.len() - 1;
                }
            });
            ui.horizontal(|ui| {
                let edit = ui.text_edit_singleline(&mut self.address);
                if edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let a = self.address.clone();
                    self.navigate(&a);
                }
                if ui.button("Открыть").clicked() {
                    let a = self.address.clone();
                    self.navigate(&a);
                }
                if ui.button("VPN ON").clicked() {
                    let _ = self
                        .vpn
                        .import("vless://user@127.0.0.1:1080", VpnMode::Global);
                }
                ui.checkbox(&mut self.dark_websites, "Тёмные сайты");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.tabs[self.active].content.is_empty() {
                ui.heading("Plus — Яндекс старт");
                ui.horizontal_wrapped(|ui| {
                    for tile in yandex_tiles() {
                        if ui.button(tile.name).clicked() {
                            self.address = tile.url.to_string();
                            let a = self.address.clone();
                            self.navigate(&a);
                        }
                    }
                });
            } else {
                let text = self.tabs[self.active].content.clone();
                let mut engine = Engine::new(ui.available_width(), ui.available_height());
                let mut doc = engine.parse_and_layout(&text);
                if self.dark_websites {
                    for b in &mut doc.boxes {
                        b.style.color = [240, 240, 240, 255];
                        b.style.background = [33, 33, 33, 255];
                    }
                }
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for b in doc.boxes {
                        ui.label(b.text);
                    }
                });
            }
        });
    }
}

fn main() -> eframe::Result {
    let app = PlusApp::new().expect("init app");
    let options = eframe::NativeOptions::default();
    eframe::run_native("Plus", options, Box::new(|_| Ok(Box::new(app))))
}
