//! MiniCRM — a weekly cockpit for someone starting a business.
//!
//! Each week it answers four questions, one table each: what I meant to do,
//! what I really did, what I noticed, and what I need to clear up now. Beside
//! them, the contacts the work is done with, where each one stands, and the
//! follow-ups that have come round. No automation and nothing connected to
//! other tools: a table goes in and out as a CSV file.

mod components;
mod kit;
mod mcp;
mod model;
mod seed;
mod store;
mod transfer;

use desktop_kit::ui::{use_theme, Banners, ConnectWindow, Header, SettingsWindow};
use dioxus::desktop::LogicalSize;
use dioxus::prelude::*;

use components::{Ui, Workspace};

const MAIN_CSS: &str = include_str!("../assets/main.css");

/// The native title bar, as in every mayorana app: name and version.
fn window_title() -> String {
    format!("MiniCRM {}", env!("CARGO_PKG_VERSION"))
}

fn main() {
    kit::init();
    // `minicrm mcp`: the cockpit over stdio for an MCP client, and no window.
    if std::env::args().nth(1).as_deref() == Some("mcp") {
        return mcp::serve();
    }

    // The stylesheet goes into the page head once: a `style` element inside
    // `App` would be diffed again on every keystroke.
    let cfg = dioxus::desktop::Config::new()
        .with_data_directory(store::data_dir().join("webview"))
        .with_custom_head(format!("<style>{MAIN_CSS}</style>"))
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title(window_title())
                .with_inner_size(LogicalSize::new(1320.0, 860.0))
                .with_window_icon(window_icon()),
        );
    LaunchBuilder::desktop().with_cfg(cfg).launch(App);
}

/// The window icon, decoded from the embedded logo — same as GitAgent.
///
/// build.rs embeds `assets/icon.ico` into the .exe resource, which covers the
/// Start menu and shortcuts — but the *window* (title bar, alt-tab, taskbar
/// button) shows only what the app sets at runtime.
///
/// Downscaled to 64px on the way in: tao hands the OS this single bitmap for
/// every size it needs, and stretching a 1024px source down to a 16px title
/// bar is what makes the icon look muddy.
fn window_icon() -> Option<dioxus::desktop::tao::window::Icon> {
    const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");
    const SIZE: u32 = 64;

    let img = image::load_from_memory(ICON_PNG).ok()?.resize_exact(
        SIZE,
        SIZE,
        image::imageops::FilterType::Lanczos3,
    );
    dioxus::desktop::tao::window::Icon::from_rgba(img.into_rgba8().into_raw(), SIZE, SIZE).ok()
}

#[component]
fn App() -> Element {
    // The cockpit lives in one signal every screen reads and writes; the
    // file on disk follows it.
    let cockpit = use_context_provider(|| Signal::new(store::load()));
    use_context_provider(Ui::new);
    use_effect(move || store::save(&cockpit.read()));
    // What Claude changes through `minicrm mcp` shows up here within moments.
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
            if let Some(fresh) = store::changed_on_disk() {
                let mut cockpit = cockpit;
                cockpit.set(fresh);
            }
        }
    });

    let theme = use_theme();
    let mut settings_open = use_signal(|| false);
    let mut connect_open = use_signal(|| false);

    rsx! {
        // A newer release and the notice from mayorana.ch. No usage
        // statistics: MiniCRM keeps none.
        Banners {}
        Header {
            on_settings: move |_| settings_open.set(true),
            on_mcp: move |_| connect_open.set(true),
        }
        if connect_open() {
            ConnectWindow {
                lead: "Ask Claude to keep your cockpit for you: log a call against a contact, move someone \
                       along the pipeline, set the next follow-up, or import your LinkedIn connections. It \
                       reads and writes the same file as this window, which shows each change within moments.",
                try_it: "\u{201c}Who do I need to follow up this week?\u{201d}, or \u{201c}I just called \
                         Karim, he wants a proposal: log it and move him on.\u{201d}",
                caution: "Your contacts stay on this computer; only what Claude reads through these tools \
                          goes into the conversation.",
                on_close: move |_| connect_open.set(false),
            }
        }
        if settings_open() {
            SettingsWindow { theme, on_close: move |_| settings_open.set(false),
                div { class: "kit-set-actions",
                    button {
                        class: "kit-set-btn",
                        title: "The cockpit.json file: copying it duplicates the cockpit",
                        onclick: move |_| {
                            let _ = desktop_kit::launch::reveal(&store::file());
                        },
                        "Show the cockpit file"
                    }
                }
            }
        }
        Workspace {}
    }
}
