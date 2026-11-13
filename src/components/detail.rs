//! The editor of one line, on the right: every column of its table as a
//! field, saved as it is typed.

use dioxus::prelude::*;

use super::Ui;
use crate::model::{Cockpit, Column, Kind, Sheet};

#[component]
pub fn Detail(sheet: Sheet, id: u64) -> Element {
    let mut ui = use_context::<Ui>();
    let mut cockpit = use_context::<Signal<Cockpit>>();
    // The line whose deletion is being asked about. By id, so the question
    // does not follow the person to the next line they open.
    let mut confirming = use_signal(|| None::<u64>);
    desktop_kit::ui::use_escape(use_callback(move |()| ui.selected.set(None)));

    let Some(row) = cockpit.read().row(sheet, id).cloned() else {
        return rsx! {};
    };
    let title = row.get(sheet.title_key()).trim();

    rsx! {
        aside { class: "detail-col",
            div { class: "col-head",
                div { class: "col-head-main",
                    div { class: "col-title",
                        if title.is_empty() { "New line" } else { "{title}" }
                    }
                    div { class: "col-sub", "{sheet.title()}" }
                }
                button {
                    class: "detail-close",
                    title: "Close (Esc)",
                    aria_label: "Close",
                    onclick: move |_| ui.selected.set(None),
                    "\u{d7}"
                }
            }
            div { class: "detail-body",
                for column in sheet.columns() {
                    Field {
                        key: "{id}-{column.key}",
                        sheet,
                        id,
                        column: *column,
                        value: row.get(column.key).to_string(),
                    }
                }
            }
            div { class: "detail-actions",
                if confirming() == Some(id) {
                    span { class: "detail-ask", "Delete this line?" }
                    button {
                        class: "btn btn-danger",
                        onclick: move |_| {
                            confirming.set(None);
                            ui.selected.set(None);
                            cockpit.write().remove(sheet, id);
                        },
                        "Yes, delete"
                    }
                    button { class: "btn", onclick: move |_| confirming.set(None), "Cancel" }
                } else {
                    button {
                        class: "btn",
                        title: "A copy of this line, right under it",
                        onclick: move |_| {
                            let copy = cockpit.write().duplicate(sheet, id);
                            if copy.is_some() {
                                ui.selected.set(copy);
                            }
                        },
                        "Duplicate"
                    }
                    span { class: "detail-gap" }
                    button {
                        class: "btn btn-danger",
                        onclick: move |_| confirming.set(Some(id)),
                        "Delete"
                    }
                }
            }
        }
    }
}

/// One column of the line, edited the way its kind calls for.
#[component]
fn Field(sheet: Sheet, id: u64, column: Column, value: String) -> Element {
    let mut cockpit = use_context::<Signal<Cockpit>>();
    let key = column.key;

    match column.kind {
        // A drop-down with a handful of entries reads better laid out: one
        // click instead of two, and the colours say what each one means.
        Kind::Choice(options) => rsx! {
            div { class: "field",
                span { "{column.label}" }
                div { class: "choices",
                    for option in options {
                        button {
                            key: "{option.label}",
                            class: if option.label == value {
                                "choice choice-on choice-{option.tone.class()}"
                            } else {
                                "choice"
                            },
                            onclick: move |_| cockpit.write().set(sheet, id, key, option.label.to_string()),
                            "{option.label}"
                        }
                    }
                }
            }
        },
        Kind::Long => rsx! {
            label { class: "field",
                span { "{column.label}" }
                textarea {
                    rows: "3",
                    value: "{value}",
                    oninput: move |e| cockpit.write().set(sheet, id, key, e.value()),
                }
            }
        },
        Kind::Date => rsx! {
            label { class: "field field-short",
                span { "{column.label}" }
                input {
                    r#type: "date",
                    value: "{value}",
                    oninput: move |e| cockpit.write().set(sheet, id, key, e.value()),
                }
            }
        },
        Kind::Week => rsx! {
            label { class: "field field-short",
                span { "{column.label}" }
                input {
                    value: "{value}",
                    placeholder: "W46",
                    oninput: move |e| cockpit.write().set(sheet, id, key, e.value()),
                }
            }
        },
        Kind::Text => rsx! {
            label { class: "field",
                span { "{column.label}" }
                input {
                    value: "{value}",
                    oninput: move |e| cockpit.write().set(sheet, id, key, e.value()),
                }
            }
        },
    }
}
