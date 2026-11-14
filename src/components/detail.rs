//! The editor of one line, on the right: every column of its table as a
//! field, saved as it is typed. A contact's editor also lists what was done
//! with them.

use dioxus::prelude::*;

use super::Ui;
use crate::model::{show_date, Cockpit, Column, Kind, Row, Sheet};

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
                if sheet == Sheet::Contacts {
                    History { id }
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
    let today = (use_context::<Ui>().today)();
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
        // Empty until there is something to follow up, and emptied once it
        // is done.
        Kind::Due => {
            rsx! {
                div { class: "field field-short",
                    span { "{column.label}" }
                    div { class: "field-row",
                        input {
                            r#type: "date",
                            value: "{value}",
                            oninput: move |e| cockpit.write().set(sheet, id, key, e.value()),
                        }
                        if value.is_empty() {
                            button {
                                class: "btn btn-small",
                                title: "Follow up a week from today",
                                onclick: move |_| cockpit.write().postpone(sheet, id, 7, today),
                                "In a week"
                            }
                        } else {
                            button {
                                class: "btn btn-small",
                                title: "Nothing to follow up any more",
                                onclick: move |_| cockpit.write().set(sheet, id, key, String::new()),
                                "Clear"
                            }
                        }
                    }
                }
            }
        }
        Kind::Contact => rsx! { ContactField { sheet, id, column, value } },
    }
}

/// Who an action was with: one of the contacts, by name, and a way to
/// open them.
#[component]
fn ContactField(sheet: Sheet, id: u64, column: Column, value: String) -> Element {
    let ui = use_context::<Ui>();
    let mut cockpit = use_context::<Signal<Cockpit>>();
    let key = column.key;
    let mut people: Vec<(u64, String)> = cockpit
        .read()
        .contacts
        .iter()
        .map(|r| (r.id, r.get("name").trim().to_string()))
        .collect();
    people.sort_by_key(|(_, name)| name.to_lowercase());
    let linked = cockpit.read().contact(&value).map(|r| r.id);

    rsx! {
        div { class: "field",
            span { "{column.label}" }
            div { class: "field-row",
                select {
                    value: "{linked.map(|i| i.to_string()).unwrap_or_default()}",
                    onchange: move |e| cockpit.write().set(sheet, id, key, e.value()),
                    option { value: "", selected: linked.is_none(), "No one in particular" }
                    for (person, name) in people {
                        option {
                            key: "{person}",
                            value: "{person}",
                            selected: linked == Some(person),
                            if name.is_empty() { "Contact without a name" } else { "{name}" }
                        }
                    }
                }
                if let Some(person) = linked {
                    button {
                        class: "btn btn-small",
                        title: "Open this contact",
                        onclick: move |_| ui.open(Sheet::Contacts, person),
                        "Open \u{203a}"
                    }
                }
            }
        }
    }
}

/// What was done with a contact, from Field progress, the latest first, and
/// a way to log the next thing.
#[component]
fn History(id: u64) -> Element {
    let ui = use_context::<Ui>();
    let mut cockpit = use_context::<Signal<Cockpit>>();
    let rows: Vec<(u64, String, String, String)> = cockpit
        .read()
        .history(id)
        .into_iter()
        .map(|r: &Row| {
            (
                r.id,
                show_date(r.get("date")),
                r.get("action").to_string(),
                r.get("result").to_string(),
            )
        })
        .collect();

    rsx! {
        div { class: "field",
            span { "What I did with them" }
            if rows.is_empty() {
                div { class: "history-empty", "Nothing logged in Field progress yet." }
            } else {
                div { class: "history",
                    for (action, date, title, result) in rows {
                        button {
                            key: "{action}",
                            class: "history-row",
                            title: "Open this action in Field progress",
                            onclick: move |_| ui.open(Sheet::Actions, action),
                            span { class: "history-date", "{date}" }
                            span { class: "history-main",
                                span { class: "history-title",
                                    if title.trim().is_empty() { "Action to describe" } else { "{title}" }
                                }
                                if !result.is_empty() {
                                    span { class: "history-sub", "{result}" }
                                }
                            }
                        }
                    }
                }
            }
            button {
                class: "btn btn-small history-add",
                title: "A new line in Field progress, with this contact",
                onclick: move |_| {
                    let today = (ui.today)();
                    let action = {
                        let mut c = cockpit.write();
                        let action = c.add(Sheet::Actions, today);
                        c.set(Sheet::Actions, action, "contact", id.to_string());
                        action
                    };
                    ui.open(Sheet::Actions, action);
                },
                "+ Log an action with them"
            }
        }
    }
}
