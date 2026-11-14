//! The cockpit's pages, down the left: the guide, the four tables in the
//! order of the weekly routine, then the contacts and their follow-ups.

use dioxus::prelude::*;

use super::{Ui, View};
use crate::model::{Cockpit, Sheet, Urgency};

#[component]
pub fn Sidebar() -> Element {
    let ui = use_context::<Ui>();
    let cockpit = use_context::<Signal<Cockpit>>();
    let current = (ui.view)();
    let due = cockpit.read().follow_ups((ui.today)());
    let late = due.iter().any(|f| f.urgency == Urgency::Overdue);
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
                for (index, sheet) in Sheet::ROUTINE.into_iter().enumerate() {
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
                div { class: "sidebar-head sidebar-section",
                    span { class: "sidebar-title", "My contacts" }
                }
                button {
                    class: row(current == View::Sheet(Sheet::Contacts)),
                    onclick: move |_| ui.go(View::Sheet(Sheet::Contacts)),
                    span { class: "step-num", "@" }
                    div { class: "sidebar-main",
                        div { class: "sidebar-label", "{Sheet::Contacts.title()}" }
                        div { class: "sidebar-sub", "Who they are and where they stand" }
                    }
                    span { class: "sidebar-count", "{cockpit.read().contacts.len()}" }
                }
                button {
                    class: row(current == View::FollowUps),
                    onclick: move |_| ui.go(View::FollowUps),
                    span { class: "step-num", "\u{21bb}" }
                    div { class: "sidebar-main",
                        div { class: "sidebar-label", "Follow-ups" }
                        div { class: "sidebar-sub", "Who to get back to this week" }
                    }
                    span {
                        class: if late { "sidebar-count sidebar-count-late" } else { "sidebar-count" },
                        title: if late { "Some are overdue" } else { "Due by Sunday" },
                        "{due.len()}"
                    }
                }
            }
            div { class: "sidebar-foot", "Saved after every change" }
        }
    }
}
