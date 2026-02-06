use anyhow::Result;
use plus_engine::EngineController;
use plus_yandex::{new_tab_html, omnibox_to_url};
use wry::application::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::webview::WebViewBuilder;

pub fn run_desktop_browser(mut engine: EngineController, title: &str) -> Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title(title).build(&event_loop)?;

    let _webview = WebViewBuilder::new(window)?
        .with_initialization_script(
            r#"
            window.plusNavigate = function(value){
                if(value.includes('://') || value.includes('.')) return value;
                return 'https://yandex.ru/search/?text=' + encodeURIComponent(value);
            };
        "#,
        )
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
