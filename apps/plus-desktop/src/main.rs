use anyhow::Result;
use eframe::egui;
use plus_adblock::AdblockEngine;
use plus_engine::{BrowserPolicy, EngineController};
use plus_net::{start_proxy, HistoryStore};
use plus_renderer::WebViewHostWindows;
use plus_vpn::{VpnManager, VpnMode};
use raw_window_handle::RawWindowHandle;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

#[derive(Clone, Default)]
struct Tab {
    title: String,
    url: String,
}

#[derive(Clone, Default)]
struct DownloadItem {
    url: String,
    path: String,
    status: String,
}

#[derive(Default)]
struct DiagnosticsState {
    ip: String,
    checking: bool,
}

struct PlusApp {
    tabs: Vec<Tab>,
    active: usize,
    closed_tabs: Vec<Tab>,
    drag_tab: Option<usize>,
    omnibox: String,
    history: VecDeque<String>,
    bookmarks: Vec<String>,
    downloads: Vec<DownloadItem>,
    download_url: String,
    show_settings: bool,
    show_diagnostics: bool,
    adblock: Arc<Mutex<AdblockEngine>>,
    engine: EngineController,
    runtime: Runtime,
    proxy: Option<String>,
    vpn: VpnManager,
    vpn_status: String,
    vpn_endpoint: String,
    diagnostics: DiagnosticsState,
    progress: f32,
    webview: Option<WebViewHostWindows>,
    history_store: HistoryStore,
}

impl PlusApp {
    fn new() -> Result<Self> {
        let runtime = Runtime::new()?;
        let history_store = HistoryStore::open("plus-history.db")?;
        let adblock =
            AdblockEngine::from_filter_list("||doubleclick.net^\n||googlesyndication.com^")?;
        Ok(Self {
            tabs: vec![Tab {
                title: "Новая вкладка".into(),
                url: plus_yandex::new_tab_data_url(),
            }],
            active: 0,
            closed_tabs: Vec::new(),
            drag_tab: None,
            omnibox: String::new(),
            history: VecDeque::with_capacity(50),
            bookmarks: Vec::new(),
            downloads: Vec::new(),
            download_url: String::new(),
            show_settings: false,
            show_diagnostics: false,
            adblock: Arc::new(Mutex::new(adblock)),
            engine: EngineController::new(BrowserPolicy::default()),
            runtime,
            proxy: None,
            vpn: VpnManager::new(
                std::env::var("PLUS_SINGBOX_BIN").unwrap_or_else(|_| "sing-box".into()),
                std::env::temp_dir().join("plus-vpn"),
            ),
            diagnostics: DiagnosticsState::default(),
            progress: 0.0,
            vpn_status: "disconnected".into(),
            vpn_endpoint: "".into(),
            webview: None,
            history_store,
        })
    }

    fn ensure_webview(&mut self, frame: &eframe::Frame) {
        if self.webview.is_some() {
            return;
        }
        #[cfg(windows)]
        {
            if let RawWindowHandle::Win32(handle) = frame.raw_window_handle() {
                let hwnd = handle.hwnd.get() as windows_sys::Win32::Foundation::HWND;
                let mut host = WebViewHostWindows::new(hwnd);
                let _ = host.initialize();
                let _ = host.set_proxy(self.proxy.clone());
                let _ = host.add_adblock_handler(self.adblock.clone());
                self.webview = Some(host);
                self.navigate_current();
            }
        }
    }

    fn navigate_current(&mut self) {
        let url = self.tabs[self.active].url.clone();
        let _ = self
            .history_store
            .add_visit(&url, &self.tabs[self.active].title);
        self.history.push_front(url.clone());
        self.progress = 0.2;
        if let Some(host) = &self.webview {
            let _ = host.navigate(&url);
        }
    }

    fn open_url(&mut self, input: &str) {
        let url = plus_yandex::omnibox_to_url(input);
        self.tabs[self.active].url = url.clone();
        self.omnibox = url;
        self.navigate_current();
    }

    fn new_tab(&mut self) {
        self.tabs.push(Tab {
            title: "Новая вкладка".into(),
            url: plus_yandex::new_tab_data_url(),
        });
        self.active = self.tabs.len() - 1;
        self.navigate_current();
    }

    fn close_tab(&mut self) {
        if self.tabs.len() == 1 {
            return;
        }
        let closed = self.tabs.remove(self.active);
        self.closed_tabs.push(closed);
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        }
        self.navigate_current();
    }

    fn close_other_tabs(&mut self) {
        let current = self.active;
        let tab = self.tabs[current].clone();
        self.tabs = vec![tab];
        self.active = 0;
        self.navigate_current();
    }

    fn close_right_tabs(&mut self) {
        self.tabs.truncate(self.active + 1);
        self.navigate_current();
    }

    fn duplicate_tab(&mut self) {
        let tab = self.tabs[self.active].clone();
        self.tabs.insert(self.active + 1, tab);
        self.active += 1;
        self.navigate_current();
    }
    fn restore_tab(&mut self) {
        if let Some(tab) = self.closed_tabs.pop() {
            self.tabs.push(tab);
            self.active = self.tabs.len() - 1;
            self.navigate_current();
        }
    }

    fn start_download(&mut self) {
        let url = self.download_url.trim().to_string();
        if url.is_empty() {
            return;
        }
        let filename = url.split('/').last().unwrap_or("download.bin");
        if filename.ends_with(".exe") || filename.ends_with(".msi") {
            self.downloads.push(DownloadItem {
                url: url.clone(),
                path: "blocked".into(),
                status: "blocked-unsafe".into(),
            });
            self.download_url.clear();
            return;
        }
        let downloads_dir = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());
        let path = downloads_dir.join(filename).to_string_lossy().to_string();
        let status = "downloading".to_string();
        let mut item = DownloadItem {
            url: url.clone(),
            path: path.clone(),
            status,
        };
        let result = self.runtime.block_on(async {
            let bytes = reqwest::get(url).await?.bytes().await?;
            tokio::fs::write(&path, &bytes).await?;
            Ok::<_, anyhow::Error>(())
        });
        item.status = if result.is_ok() {
            "completed".into()
        } else {
            "error".into()
        };
        self.downloads.push(item);
        self.download_url.clear();
    }

    fn handle_hotkeys(&mut self, ctx: &egui::Context) {
        let input = ctx.input(|i| i.clone());
        if input.modifiers.command && input.key_pressed(egui::Key::T) {
            self.new_tab();
        }
        if input.modifiers.command && input.modifiers.shift && input.key_pressed(egui::Key::T) {
            self.restore_tab();
        }
        if input.modifiers.command && input.key_pressed(egui::Key::W) {
            self.close_tab();
        }
        if input.modifiers.command && input.key_pressed(egui::Key::L) {
            ctx.memory_mut(|m| m.request_focus("omnibox".into()));
        }
        if input.modifiers.command && input.key_pressed(egui::Key::R) {
            self.progress = 0.2;
            if let Some(host) = &self.webview {
                let _ = host.reload();
            }
        }
        if input.modifiers.alt && input.key_pressed(egui::Key::ArrowLeft) {
            self.progress = 0.2;
            if let Some(host) = &self.webview {
                let _ = host.go_back();
            }
        }
        if input.modifiers.alt && input.key_pressed(egui::Key::ArrowRight) {
            self.progress = 0.2;
            if let Some(host) = &self.webview {
                let _ = host.go_forward();
            }
        }
        if input.key_pressed(egui::Key::F5) {
            self.progress = 0.2;
            if let Some(host) = &self.webview {
                let _ = host.reload();
            }
        }
        if input.key_pressed(egui::Key::Escape) {
            self.progress = 0.2;
            if let Some(host) = &self.webview {
                let _ = host.stop();
            }
        }
        if input.modifiers.command && input.key_pressed(egui::Key::D) {
            let url = self.tabs[self.active].url.clone();
            self.bookmarks.push(url);
        }
    }

    fn check_ip(&mut self) {
        if self.diagnostics.checking {
            return;
        }
        self.diagnostics.checking = true;
        let proxy = self.proxy.clone();
        let handle = self.runtime.handle().clone();
        let ip_state = Arc::new(Mutex::new(String::new()));
        let ip_state_clone = ip_state.clone();
        handle.spawn(async move {
            let client = plus_net::NetClient::new(proxy).unwrap();
            let ip = client
                .get_egress_ip()
                .await
                .unwrap_or_else(|_| "error".into());
            *ip_state_clone.lock().unwrap() = ip;
        });
        if let Ok(ip) = ip_state.lock() {
            self.diagnostics.ip = ip.clone();
        }
        self.diagnostics.checking = false;
    }
}

impl eframe::App for PlusApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.ensure_webview(frame);
        self.handle_hotkeys(ctx);

        egui::TopBottomPanel::top("tabs")
            .exact_height(40.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (idx, tab) in self.tabs.iter().enumerate() {
                        let selected = idx == self.active;
                        let label = if tab.title.is_empty() {
                            "Новая"
                        } else {
                            &tab.title
                        };
                        let response = ui.selectable_label(selected, label);
                        response.context_menu(|ui| {
                            if ui.button("Закрыть").clicked() {
                                self.active = idx;
                                self.close_tab();
                                ui.close_menu();
                            }
                            if ui.button("Закрыть другие").clicked() {
                                self.active = idx;
                                self.close_other_tabs();
                                ui.close_menu();
                            }
                            if ui.button("Закрыть справа").clicked() {
                                self.active = idx;
                                self.close_right_tabs();
                                ui.close_menu();
                            }
                            if ui.button("Дублировать").clicked() {
                                self.active = idx;
                                self.duplicate_tab();
                                ui.close_menu();
                            }
                        });
                        if response.clicked() {
                            self.active = idx;
                            self.navigate_current();
                        }
                        if response.drag_started() {
                            self.drag_tab = Some(idx);
                        }
                        if response.drag_released() {
                            if let Some(from) = self.drag_tab.take() {
                                let to = idx;
                                if from != to {
                                    let tab = self.tabs.remove(from);
                                    self.tabs.insert(to, tab);
                                    self.active = to;
                                }
                            }
                        }
                    }
                    if ui.button("+").clicked() {
                        self.new_tab();
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("⚙").clicked() {
                            self.show_settings = !self.show_settings;
                        }
                        if ui.button("🛡").clicked() {
                            self.show_diagnostics = !self.show_diagnostics;
                        }
                    });
                });
            });

        egui::TopBottomPanel::top("nav")
            .exact_height(48.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("←").clicked() {
                        self.progress = 0.2;
                        if let Some(host) = &self.webview {
                            let _ = host.go_back();
                        }
                    }
                    if ui.button("→").clicked() {
                        self.progress = 0.2;
                        if let Some(host) = &self.webview {
                            let _ = host.go_forward();
                        }
                    }
                    if ui.button("⟳").clicked() {
                        self.progress = 0.2;
                        if let Some(host) = &self.webview {
                            let _ = host.reload();
                        }
                    }
                    if ui.button("⏹").clicked() {
                        self.progress = 0.2;
                        if let Some(host) = &self.webview {
                            let _ = host.stop();
                        }
                    }
                    if ui.button("⌂").clicked() {
                        self.open_url("https://yandex.ru");
                    }
                    let response = ui.add_sized(
                        [ui.available_width() - 120.0, 28.0],
                        egui::TextEdit::singleline(&mut self.omnibox).id_source("omnibox"),
                    );
                    if response.has_focus() {
                        egui::popup::show_below_widget(
                            ui,
                            egui::Id::new("omnibox-popup"),
                            &response,
                            |ui| {
                                ui.set_min_width(response.rect.width());
                                for item in self.history.iter().take(5).chain(self.bookmarks.iter())
                                {
                                    if ui.button(item).clicked() {
                                        let input = item.clone();
                                        self.open_url(&input);
                                        ui.close_menu();
                                    }
                                }
                            },
                        );
                    }
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let input = self.omnibox.clone();
                        self.open_url(&input);
                    }
                    if ui.button("☆").clicked() {
                        self.bookmarks.push(self.tabs[self.active].url.clone());
                    }
                });
            });

        if self.progress > 0.0 {
            egui::TopBottomPanel::top("progress")
                .exact_height(2.0)
                .show(ctx, |ui| {
                    ui.add(egui::ProgressBar::new(self.progress).show_percentage(false));
                });
            self.progress = (self.progress + 0.05).min(1.0);
            if self.progress >= 1.0 {
                self.progress = 0.0;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            let pixels_per_point = ctx.pixels_per_point();
            let x = (rect.min.x * pixels_per_point) as i32;
            let y = (rect.min.y * pixels_per_point) as i32;
            let w = (rect.width() * pixels_per_point) as i32;
            let h = (rect.height() * pixels_per_point) as i32;
            self.progress = 0.2;
            if let Some(host) = &self.webview {
                let _ = host.set_bounds(x, y, w, h);
            }
        });

        if self.show_diagnostics {
            egui::Window::new("Diagnostics").show(ctx, |ui| {
                ui.label(format!(
                    "Proxy: {}",
                    if let Some(p) = &self.proxy {
                        p.clone()
                    } else {
                        "OFF".into()
                    }
                ));
                ui.label(format!("VPN: {}", self.vpn_status));
                ui.label(format!("VPN endpoint: {}", self.vpn_endpoint));
                if let Ok(ad) = self.adblock.lock() {
                    ui.label(format!("Adblock hits: {}", ad.stats.blocked));
                    for url in ad.last_blocked() {
                        ui.label(url);
                    }
                }
                if ui.button("Check IP").clicked() {
                    self.check_ip();
                }
                if !self.diagnostics.ip.is_empty() {
                    ui.label(format!("IP: {}", self.diagnostics.ip));
                }
            });
        }

        if self.show_settings {
            egui::Window::new("Settings").show(ctx, |ui| {
                ui.heading("Downloads");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.download_url);
                    if ui.button("Download").clicked() {
                        self.start_download();
                    }
                });
                for item in &self.downloads {
                    ui.label(format!("{} -> {} ({})", item.url, item.path, item.status));
                }
            });
        }
    }
}

fn main() -> Result<()> {
    let mut app = PlusApp::new()?;
    let mut vpn_socks = None;
    if let Ok(url) = std::env::var("PLUS_VPN_IMPORT") {
        let _ = app.vpn.import(&url, VpnMode::Global, true)?;
        app.vpn_status = "starting".into();
        app.vpn_endpoint = url.clone();
        app.runtime.block_on(app.vpn.start_core())?;
        vpn_socks = app.vpn.browser_proxy().map(|p| p.replace("socks5h://", ""));
        app.vpn_status = "connected".into();
    }
    let adblock = app.adblock.clone();
    let proxy_handle =
        app.runtime
            .block_on(start_proxy("127.0.0.1:0", adblock, vpn_socks.clone()))?;
    app.proxy = Some(format!("http://{}", proxy_handle.listen_addr));

    let options = eframe::NativeOptions::default();
    eframe::run_native("Plus", options, Box::new(|_| Ok(Box::new(app))))?;
    Ok(())
}
