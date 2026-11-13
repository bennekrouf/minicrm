//! MiniCRM — a weekly cockpit for someone starting a business.
//!
//! Each week it answers four questions, one table each: what I meant to do,
//! what I really did, what I noticed, and what I need to clear up now. No
//! indicators, no automation, nothing connected to other tools: the tables,
//! their example lines, and a count of this week's priorities.

mod components;
mod kit;
mod model;
mod seed;
mod store;

use desktop_kit::ui::{use_theme, Banners, Header, SettingsWindow};
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

    let theme = use_theme();
    let mut settings_open = use_signal(|| false);

    rsx! {
        // A newer release and the notice from mayorana.ch. No usage
        // statistics: MiniCRM keeps none.
        Banners {}
        Header { on_settings: move |_| settings_open.set(true) }
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
