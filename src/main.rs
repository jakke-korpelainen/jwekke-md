use pulldown_cmark::{html, Parser};
use std::fs;
use std::sync::{Arc, Mutex};
use tao::{
    event::{DeviceEvent, Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use wry::WebViewBuilder;

struct AppState {
    input: String,
    preview: String,
}

const WINDOW_TITLE: &str = "@jwekke/md";
const EDITOR_PATH: &str = "src/editor.html";

fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_inner_size(tao::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    let html = fs::read_to_string(EDITOR_PATH).expect("Failed to read src/index.html");
    let initial_app_state = Arc::new(Mutex::new(AppState {
        input: String::new(),
        preview: html.clone(),
    }));

    let webview = WebViewBuilder::new()
        .with_html(html)
        .with_ipc_handler({
            let app_state_for_ipc = initial_app_state.clone();
            move |msg: wry::http::Request<String>| {
                let output = parse_markdown(&app_state_for_ipc, msg);
                app_state_for_ipc.lock().unwrap().preview = output;
            }
        })
        .build(&window)?;

    fn parse_markdown(app_state: &Arc<Mutex<AppState>>, msg: wry::http::Request<String>) -> String {
        let mut state = app_state.lock().unwrap();

        state.input = msg.body().to_string();
        let parser = Parser::new(&state.input);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        return html_output;
    }

    fn refresh_preview(app_state: &Arc<Mutex<AppState>>, webview: &wry::WebView) {
        webview
            .evaluate_script(&format!(
                "document.getElementById('preview').innerHTML = `{}`;",
                app_state.lock().unwrap().preview
            ))
            .unwrap();
    }

    let app_state_for_events = initial_app_state.clone();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::DeviceEvent {
                event: DeviceEvent::Key(..),
                ..
            } => {
                refresh_preview(&app_state_for_events, &webview);
            }
            _ => (),
        }
    });
}
