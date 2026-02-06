use anyhow::Result;
use plus_adblock::AdblockEngine;
use plus_engine::EngineController;
use plus_yandex::{new_tab_html, omnibox_to_url};
use std::sync::{Arc, Mutex};
use wry::application::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::webview::WebViewBuilder;

#[cfg(target_os = "windows")]
use wry::webview::WebViewBuilderExtWindows;

pub fn run_desktop_browser(
    mut engine: EngineController,
    title: &str,
    proxy: Option<String>,
    adblock: Arc<Mutex<AdblockEngine>>,
) -> Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title(title).build(&event_loop)?;

    let mut builder = WebViewBuilder::new(window)?;
    if let Some(proxy_url) = proxy.clone() {
        #[cfg(target_os = "windows")]
        {
            builder =
                builder.with_additional_browser_args(&format!("--proxy-server={}", proxy_url));
        }
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            std::env::set_var("ALL_PROXY", &proxy_url);
            std::env::set_var("HTTPS_PROXY", &proxy_url);
            std::env::set_var("HTTP_PROXY", &proxy_url);
        }
    }

    let adblock_filter = adblock.clone();
    let _webview = builder
        .with_initialization_script(
            r#"
            window.plusNavigate = function(value){
                if(value.includes('://') || value.includes('.')) return value;
                return 'https://yandex.ru/search/?text=' + encodeURIComponent(value);
            };
        "#,
        )
        .with_navigation_handler(move |url| {
            let mut engine = adblock_filter.lock().expect("adblock lock");
            !engine.should_block(&url, "about:blank", "document")
        })
        .with_html(new_tab_html())?
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {}
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::UserEvent(()) => {
                let _ = engine.validate_navigation(&omnibox_to_url("yandex"));
            }
            _ => {}
        }
    });
}
