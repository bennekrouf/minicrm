//! The follow-ups: every contact and every action whose follow-up date is
//! this week or already past, the late ones first, so nobody waiting for an
//! answer is forgotten.

use dioxus::prelude::*;

use super::Ui;
use crate::model::{show_date, Cockpit, FollowUp, Sheet, Urgency};

/// The groups the page shows, in its order.
const GROUPS: [Urgency; 3] = [Urgency::Overdue, Urgency::Today, Urgency::ThisWeek];

#[component]
pub fn FollowUps() -> Element {
    let ui = use_context::<Ui>();
    let cockpit = use_context::<Signal<Cockpit>>();
    let due = cockpit.read().follow_ups((ui.today)());

    rsx! {
        div { class: "main-col",
            div { class: "col-head",
                div { class: "col-head-main",
                    div { class: "col-title", "Follow-ups" }
                    div { class: "col-sub", "Who do I get back to before Sunday?" }
                }
            }
            if due.is_empty() {
                div { class: "placeholder",
                    div { class: "placeholder-title", "Nothing to follow up this week" }
                    div { class: "placeholder-sub",
                        "A contact's \u{201c}Next follow-up\u{201d}, or an action's \u{201c}Follow up on\u{201d}, \
                         shows here once its day comes round."
                    }
                }
            } else {
                div { class: "grid-scroll followups",
                    for urgency in GROUPS {
                        {group(urgency, due.iter().filter(|f| f.urgency == urgency).cloned().collect())}
                    }
                }
            }
        }
    }
}

fn group(urgency: Urgency, items: Vec<FollowUp>) -> Element {
    if items.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { key: "{urgency.label()}", class: "followup-group",
            div { class: "followup-head followup-{urgency.tone().class()}",
                "{urgency.label()}"
                span { class: "chip-count", "{items.len()}" }
            }
            for item in items {
                Item { key: "{item.id}", item }
            }
        }
    }
}

#[component]
fn Item(item: FollowUp) -> Element {
    let ui = use_context::<Ui>();
    let mut cockpit = use_context::<Signal<Cockpit>>();
    let (sheet, id) = (item.sheet, item.id);
    let Some((date_key, _)) = sheet.follow_up() else {
        return rsx! {};
    };
    let today = (ui.today)();
    let kind = if sheet == Sheet::Contacts {
        "Contact"
    } else {
        "Action"
    };

    rsx! {
        div { class: "followup",
            span { class: "pill pill-{item.urgency.tone().class()} cell-date", "{show_date(&item.due)}" }
            button {
                class: "followup-main",
                title: "Open it in {sheet.title()}",
                onclick: move |_| ui.open(sheet, id),
                span { class: "followup-title",
                    span { class: "followup-kind", "{kind}" }
                    if item.title.trim().is_empty() { "To fill in" } else { "{item.title}" }
                    if !item.contact.is_empty() {
                        span { class: "followup-with", " with {item.contact}" }
                    }
                }
                if !item.what.trim().is_empty() {
                    span { class: "followup-what", "{item.what}" }
                }
            }
            button {
                class: "btn btn-small",
                title: "Ask again a week from now",
                onclick: move |_| cockpit.write().postpone(sheet, id, 7, today),
                "In a week"
            }
            button {
                class: "btn btn-small",
                title: "Followed up: clear the date",
                onclick: move |_| cockpit.write().set(sheet, id, date_key, String::new()),
                "Done"
            }
        }
    }
}
