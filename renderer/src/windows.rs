#[cfg(windows)]
use anyhow::{anyhow, Result};
#[cfg(windows)]
use std::sync::{Arc, Mutex};
#[cfg(windows)]
use webview2::{
    Environment, EnvironmentOptions, WebView, WebViewBuilder, WebViewController,
    WebViewControllerBuilder,
};
#[cfg(windows)]
use windows_sys::Win32::Foundation::HWND;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{GetClientRect, SetWindowPos, SWP_NOZORDER};

#[cfg(windows)]
use plus_adblock::AdblockEngine;

#[cfg(windows)]
pub struct WebViewHostWindows {
    hwnd_parent: HWND,
    controller: Option<WebViewController>,
    webview: Option<WebView>,
    proxy: Option<String>,
}

#[cfg(windows)]
impl WebViewHostWindows {
    pub fn new(hwnd_parent: HWND) -> Self {
        Self {
            hwnd_parent,
            controller: None,
            webview: None,
            proxy: None,
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        let options = self
            .proxy
            .as_ref()
            .map(|p| {
                EnvironmentOptions::builder()
                    .additional_browser_arguments(format!("--proxy-server={}", p))
                    .build()
            })
            .unwrap_or_else(EnvironmentOptions::new);
        let env = Environment::create_with_options(None, None, Some(options))?;
        let controller = WebViewControllerBuilder::new(env)
            .parent_window(self.hwnd_parent)
            .build()?;
        let webview = WebViewBuilder::new(controller.webview())?.build()?;
        self.controller = Some(controller);
        self.webview = Some(webview);
        Ok(())
    }

    pub fn set_proxy(&mut self, proxy: Option<String>) -> Result<()> {
        self.proxy = proxy;
        self.initialize()
    }

    pub fn navigate(&self, url: &str) -> Result<()> {
        self.webview
            .as_ref()
            .ok_or_else(|| anyhow!("webview not initialized"))?
            .navigate(url)?;
        Ok(())
    }

    pub fn reload(&self) -> Result<()> {
        self.webview
            .as_ref()
            .ok_or_else(|| anyhow!("webview not initialized"))?
            .reload()?;
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        self.webview
            .as_ref()
            .ok_or_else(|| anyhow!("webview not initialized"))?
            .stop()?;
        Ok(())
    }

    pub fn go_back(&self) -> Result<()> {
        self.webview
            .as_ref()
            .ok_or_else(|| anyhow!("webview not initialized"))?
            .go_back()?;
        Ok(())
    }

    pub fn go_forward(&self) -> Result<()> {
        self.webview
            .as_ref()
            .ok_or_else(|| anyhow!("webview not initialized"))?
            .go_forward()?;
        Ok(())
    }

    pub fn execute_script(&self, js: &str) -> Result<()> {
        self.webview
            .as_ref()
            .ok_or_else(|| anyhow!("webview not initialized"))?
            .execute_script(js, |_| {})?;
        Ok(())
    }

    pub fn set_bounds(&self, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
        unsafe {
            if let Some(controller) = &self.controller {
                let hwnd = controller.window();
                SetWindowPos(hwnd, 0, x, y, width, height, SWP_NOZORDER);
            } else {
                let mut rect = std::mem::zeroed();
                GetClientRect(self.hwnd_parent, &mut rect);
            }
        }
        Ok(())
    }

    pub fn add_adblock_handler(&self, adblock: Arc<Mutex<AdblockEngine>>) -> Result<()> {
        let webview = self
            .webview
            .as_ref()
            .ok_or_else(|| anyhow!("webview not initialized"))?;
        webview.add_web_resource_requested_filter("*", webview2::WebResourceContext::All)?;
        webview.add_web_resource_requested(move |_webview, args| {
            if let Ok(request) = args.request() {
                if let Ok(uri) = request.uri() {
                    let mut engine = adblock.lock().expect("adblock lock");
                    if engine.should_block(&uri, "about:blank", "resource") {
                        let response = webview2::WebResourceResponse::new("")
                            .with_status_code(204)
                            .with_reason_phrase("Blocked")
                            .build();
                        let _ = args.set_response(&response);
                    }
                }
            }
            Ok(())
        })?;
        Ok(())
    }
}

#[cfg(not(windows))]
pub struct WebViewHostWindows;

#[cfg(not(windows))]
impl WebViewHostWindows {
    pub fn new(_hwnd_parent: usize) -> Self {
        Self
    }
}
