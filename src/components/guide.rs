//! The guide: what the cockpit is for, the weekly routine in four steps,
//! and how this week's priorities stand.

use dioxus::prelude::*;

use super::{Ui, View};
use crate::model::{Choice, Cockpit, Sheet, GOAL_STATUS};

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

    rsx! {
        div { class: "guide",
            div { class: "guide-card",
                h1 { "My founder cockpit" }
                p { class: "subtitle",
                    "A simple dashboard to follow how my project is moving, week after week."
                }
                p { class: "guide-text",
                    "Each week it helps me answer four questions: what I meant to do, what I actually \
                     did, what I noticed, and what I need to clear up now."
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

                div { class: "guide-hint", "My routine in 4 steps" }
                div { class: "steps",
                    for (index, sheet) in Sheet::ALL.into_iter().enumerate() {
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
