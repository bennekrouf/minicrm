//! The guide: what the cockpit is for, the weekly routine in four steps,
//! how this week's priorities stand, and where the contacts are.

use dioxus::prelude::*;

use super::{Ui, View};
use crate::model::{Choice, Cockpit, Sheet, Urgency, GOAL_STATUS, STAGES};

/// The statuses counted on this page, in the order they are shown.
const COUNTED: [&str; 3] = ["Done", "In progress", "Blocked"];

#[component]
pub fn Guide() -> Element {
    let ui = use_context::<Ui>();
    let cockpit = use_context::<Signal<Cockpit>>();
    let counted: Vec<(Choice, usize)> = COUNTED
        .iter()
        .filter_map(|label| GOAL_STATUS.iter().find(|o| o.label == *label))
        .map(|o| {
            let count = cockpit.read().count(Sheet::Goals, "status", o.label);
            (*o, count)
        })
        .collect();
    let pipeline: Vec<(Choice, usize)> = STAGES
        .iter()
        .map(|o| (*o, cockpit.read().count(Sheet::Contacts, "stage", o.label)))
        .collect();
    let due = cockpit.read().follow_ups((ui.today)());
    let late = due.iter().filter(|f| f.urgency == Urgency::Overdue).count();

    rsx! {
        div { class: "guide",
            div { class: "guide-card",
                h1 { "My founder cockpit" }
                p { class: "subtitle",
                    "A simple dashboard to follow how my project is moving, week after week."
                }
                p { class: "guide-text",
                    "Each week it helps me answer four questions: what I meant to do, what I actually \
                     did, what I noticed, and what I need to clear up now. Beside them, it keeps the \
                     people I talk to, where each one stands, and when to get back to them."
                }

                div { class: "guide-hint", "My priorities" }
                div { class: "recap",
                    for (option, count) in counted {
                        button {
                            key: "{option.label}",
                            class: "recap-tile recap-{option.tone.class()}",
                            title: "Show the priorities that are \u{201c}{option.label}\u{201d}",
                            onclick: move |_| {
                                let mut ui = ui;
                                ui.go(View::Sheet(Sheet::Goals));
                                ui.filter.set(Some(option.label));
                            },
                            span { class: "recap-count", "{count}" }
                            span { class: "recap-label", "{option.label}" }
                        }
                    }
                }

                div { class: "guide-hint", "My contacts" }
                div { class: "recap recap-pipeline",
                    for (option, count) in pipeline {
                        button {
                            key: "{option.label}",
                            class: "recap-tile recap-{option.tone.class()}",
                            title: "Show the contacts at \u{201c}{option.label}\u{201d}",
                            onclick: move |_| {
                                let mut ui = ui;
                                ui.go(View::Sheet(Sheet::Contacts));
                                ui.filter.set(Some(option.label));
                            },
                            span { class: "recap-count", "{count}" }
                            span { class: "recap-label", "{option.label}" }
                        }
                    }
                }
                button {
                    class: if late > 0 { "guide-due guide-due-late" } else { "guide-due" },
                    onclick: move |_| ui.go(View::FollowUps),
                    match (due.len(), late) {
                        (0, _) => rsx! { "Nothing to follow up before Sunday" },
                        (1, 0) => rsx! { "1 follow-up due before Sunday" },
                        (n, 0) => rsx! { "{n} follow-ups due before Sunday" },
                        (n, late) => rsx! { "{n} follow-ups due before Sunday, {late} of them late" },
                    }
                    span { class: "step-open", "Follow-ups \u{203a}" }
                }

                div { class: "guide-hint", "My routine in 4 steps" }
                div { class: "steps",
                    for (index, sheet) in Sheet::ROUTINE.into_iter().enumerate() {
                        button {
                            key: "{sheet.title()}",
                            class: "step-row",
                            onclick: move |_| ui.go(View::Sheet(sheet)),
                            span { class: "step-num", "{index + 1}" }
                            div { class: "step-main",
                                div { class: "step-label", "{sheet.step()}" }
                                div { class: "step-sub", "{sheet.question()}" }
                            }
                            span { class: "step-open", "{sheet.title()} \u{203a}" }
                        }
                    }
                }

                p { class: "guide-reassure",
                    "This cockpit doesn't need to be perfect. It just needs to help me move forward each week."
                }
                p { class: "guide-note",
                    "The lines already here are examples from a made-up project: an entrepreneur launching \
                     a coaching offer that helps freelancers organise their sales prospecting. I can change \
                     them or delete them."
                }
            }
        }
    }
}
