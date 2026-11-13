//! The cockpit's pages, down the left: the guide, then the four tables in
//! the order of the weekly routine.

use dioxus::prelude::*;

use super::{Ui, View};
use crate::model::{Cockpit, Sheet};

#[component]
pub fn Sidebar() -> Element {
    let ui = use_context::<Ui>();
    let cockpit = use_context::<Signal<Cockpit>>();
    let current = (ui.view)();
    let row = |on: bool| {
        if on {
            "sidebar-row sidebar-row-on"
        } else {
            "sidebar-row"
        }
    };

    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar-head",
                span { class: "sidebar-title", "My cockpit" }
            }
            div { class: "sidebar-list",
                button {
                    class: row(current == View::Guide),
                    onclick: move |_| ui.go(View::Guide),
                    span { class: "step-num", "?" }
                    div { class: "sidebar-main",
                        div { class: "sidebar-label", "How it works" }
                        div { class: "sidebar-sub", "What this cockpit is for" }
                    }
                }
                for (index, sheet) in Sheet::ALL.into_iter().enumerate() {
                    button {
                        key: "{sheet.title()}",
                        class: row(current == View::Sheet(sheet)),
                        onclick: move |_| ui.go(View::Sheet(sheet)),
                        span { class: "step-num", "{index + 1}" }
                        div { class: "sidebar-main",
                            div { class: "sidebar-label", "{sheet.title()}" }
                            div { class: "sidebar-sub", "{sheet.question()}" }
                        }
                        span { class: "sidebar-count", "{cockpit.read().rows(sheet).len()}" }
                    }
                }
            }
            div { class: "sidebar-foot", "Saved after every change" }
        }
    }
}
