//! The editor of one line, on the right: every column of its table as a
//! field, saved as it is typed, then the files attached to it. A contact's
//! editor also lists what was done with them.

use dioxus::html::HasFileData;
use dioxus::prelude::*;

use super::Ui;
use desktop_kit::library::{Item, Status};

use crate::media;
use crate::model::{show_date, Attachment, Cockpit, Column, Kind, Row, Sheet};

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
    let archived = row.archived;
    let network = row.network;
    let named = if title.is_empty() {
        "This line".to_string()
    } else {
        format!("\u{201c}{title}\u{201d}")
    };

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
                if network {
                    div { class: "detail-archived detail-network",
                        span { "In your network: found by searching, out of Contacts, the counts and Follow-ups." }
                        button {
                            class: "btn btn-primary btn-small",
                            onclick: {
                                let named = named.clone();
                                move |_| {
                                cockpit.write().set_network(id, false);
                                ui.notice.set(Some(format!("{named} added to Contacts.")));
                                }
                            },
                            "Add to Contacts"
                        }
                    }
                }
                if archived {
                    div { class: "detail-archived",
                        "Archived: out of the table, its counts and Follow-ups until you restore it."
                    }
                }
                for column in sheet.columns() {
                    Field {
                        key: "{id}-{column.key}",
                        sheet,
                        id,
                        column: *column,
                        value: row.get(column.key).to_string(),
                    }
                }
                Attachments { sheet, id, files: row.attachments.clone() }
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
                    button {
                        class: "btn",
                        title: if archived {
                            "Back into the table, its counts and Follow-ups"
                        } else {
                            "Out of the table, its counts and Follow-ups, without losing it"
                        },
                        onclick: move |_| {
                            cockpit.write().set_archived(sheet, id, !archived);
                            ui.selected.set(None);
                            let note = if archived {
                                format!("{named} is back in {}.", sheet.title())
                            } else {
                                format!("{named} archived. Find it under Archived, beside the filters.")
                            };
                            ui.notice.set(Some(note));
                        },
                        if archived { "Restore" } else { "Archive" }
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
        Kind::Media => rsx! { MediaField { sheet, id, column, value } },
    }
}

/// The files attached to a line: each opens in the app the system uses for
/// it, and one moved or deleted since says so instead. Files are added with
/// a picker or dropped on the list, and kept where they are.
#[component]
fn Attachments(sheet: Sheet, id: u64, files: Vec<String>) -> Element {
    let mut ui = use_context::<Ui>();
    let mut cockpit = use_context::<Signal<Cockpit>>();
    let mut over = use_signal(|| false);
    let listed: Vec<Attachment> = files.iter().map(|p| Attachment::of(p)).collect();
    let mut add = move |paths: Vec<std::path::PathBuf>| {
        let mut refused = Vec::new();
        let mut c = cockpit.write();
        for path in paths {
            if let Err(e) = c.attach(sheet, id, &path.display().to_string()) {
                refused.push(e);
            }
        }
        if !refused.is_empty() {
            ui.notice
                .set(Some(format!("Not attached: {}", refused.join("; "))));
        }
    };

    rsx! {
        div {
            class: if over() { "field attachments attachments-over" } else { "field attachments" },
            ondragover: move |e| {
                e.prevent_default();
                over.set(true);
            },
            ondragleave: move |_| over.set(false),
            ondrop: move |e| {
                e.prevent_default();
                over.set(false);
                add(e.files().iter().map(|f| f.path()).collect());
            },
            span { "Attachments" }
            if !listed.is_empty() {
                div { class: "attachment-list",
                    for file in listed {
                        div {
                            key: "{file.path}",
                            class: if file.exists { "attachment" } else { "attachment attachment-missing" },
                            button {
                                class: "attachment-open",
                                title: if file.exists { "Open {file.path}" } else { "No longer at {file.path}" },
                                disabled: !file.exists,
                                onclick: {
                                    let path = file.path.clone();
                                    move |_| {
                                        if let Err(e) = desktop_kit::launch::open_file(std::path::Path::new(&path)) {
                                            ui.notice.set(Some(e));
                                        }
                                    }
                                },
                                span { class: "attachment-name", "{file.name}" }
                                if !file.exists {
                                    span { class: "pill pill-bad", "Missing" }
                                }
                            }
                            if file.exists {
                                button {
                                    class: "btn btn-small",
                                    title: "Show it in its folder",
                                    onclick: {
                                        let path = file.path.clone();
                                        move |_| {
                                            if let Err(e) = desktop_kit::launch::reveal(std::path::Path::new(&path)) {
                                                ui.notice.set(Some(e));
                                            }
                                        }
                                    },
                                    "Show"
                                }
                            }
                            button {
                                class: "attachment-remove",
                                title: "Take it off this line (the file itself stays)",
                                aria_label: "Remove {file.name}",
                                onclick: {
                                    let path = file.path.clone();
                                    move |_| {
                                        let _ = cockpit.write().detach(sheet, id, &path);
                                    }
                                },
                                "\u{d7}"
                            }
                        }
                    }
                }
            }
            div { class: "field-row",
                button {
                    class: "btn btn-small",
                    title: "Attach documents or images to this line. They stay where they are.",
                    onclick: move |_| {
                        spawn(async move {
                            let Some(picked) = rfd::AsyncFileDialog::new()
                                .set_title("Attach files")
                                .pick_files()
                                .await
                            else {
                                return;
                            };
                            add(picked.iter().map(|f| f.path().to_path_buf()).collect());
                        });
                    },
                    "+ Attach files"
                }
                span { class: "field-hint", "or drop them here" }
            }
        }
    }
}

/// The items a video-or-post field offers, newest first: enough to find
/// this month's work without scrolling through years of posts.
const MEDIA_CHOICES: usize = 80;

/// The video or post an action was about, from the media library the other
/// apps share: where it stands, and a way to open it where it was made.
#[component]
fn MediaField(sheet: Sheet, id: u64, column: Column, value: String) -> Element {
    let mut ui = use_context::<Ui>();
    let mut cockpit = use_context::<Signal<Cockpit>>();
    let library = use_context::<Signal<Vec<Item>>>();
    let key = column.key;
    let items = library.read();
    let linked = items.iter().find(|i| i.id == value).cloned();
    let mut choices: Vec<(String, String)> = items
        .iter()
        .take(MEDIA_CHOICES)
        .map(|i| {
            let title = if i.title.trim().is_empty() {
                "Untitled"
            } else {
                i.title.trim()
            };
            (
                i.id.clone(),
                format!(
                    "{title} \u{b7} {}, {}",
                    media::what(i),
                    i.status.label().to_lowercase()
                ),
            )
        })
        .collect();
    if let Some(item) = &linked {
        if !choices.iter().any(|(i, _)| *i == item.id) {
            choices.push((item.id.clone(), item.title.clone()));
        }
    }
    let gone = !value.is_empty() && linked.is_none();

    rsx! {
        div { class: "field",
            span { "{column.label}" }
            div { class: "field-row",
                select {
                    onchange: move |e| cockpit.write().set(sheet, id, key, e.value()),
                    option { value: "", selected: value.is_empty(), "None" }
                    if gone {
                        option { value: "{value}", selected: true, "No longer in the library" }
                    }
                    for (item, label) in choices {
                        option {
                            key: "{item}",
                            value: "{item}",
                            selected: item == value,
                            "{label}"
                        }
                    }
                }
            }
            if let Some(item) = linked {
                div { class: "field-row media-linked",
                    span { class: "pill pill-{media::tone(item.status).class()}", "{item.status.label()}" }
                    button {
                        class: "btn btn-small",
                        title: "Open it in {media::app_name(&item)}",
                        onclick: {
                            let item = item.clone();
                            move |_| {
                                if let Err(e) = media::open(&item) {
                                    ui.notice.set(Some(e));
                                }
                            }
                        },
                        "Open in {media::app_name(&item)} \u{203a}"
                    }
                    if let Some(url) = item.url.clone().filter(|u| desktop_kit::browser::is_web_url(u)) {
                        button {
                            class: "btn btn-small",
                            title: "{url}",
                            onclick: move |_| {
                                desktop_kit::browser::open(&url);
                            },
                            "See it online \u{203a}"
                        }
                    }
                }
            } else if library.read().is_empty() {
                div { class: "field-hint",
                    "What PolishCast, TrackCut and PublishKit make shows here, to say which video or post this was."
                }
            }
        }
    }
}

/// Who an action was with: one of the contacts, by name, and a way to
/// open them.
#[component]
fn ContactField(sheet: Sheet, id: u64, column: Column, value: String) -> Element {
    let ui = use_context::<Ui>();
    let mut cockpit = use_context::<Signal<Cockpit>>();
    let key = column.key;
    let linked = cockpit.read().contact(&value).map(|r| r.id);
    // The contacts in use, and the one already named even if archived.
    let mut people: Vec<(u64, String)> = cockpit
        .read()
        .contacts
        .iter()
        .filter(|r| (!r.archived && !r.network) || Some(r.id) == linked)
        .map(|r| (r.id, r.get("name").trim().to_string()))
        .collect();
    people.sort_by_key(|(_, name)| name.to_lowercase());

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

/// One action in a contact's history, as the list shows it.
struct HistoryRow {
    id: u64,
    date: String,
    title: String,
    result: String,
    /// The video or post it was about, and where that stands.
    media: Option<(String, Status)>,
}

/// What was done with a contact, from Field progress, the latest first, and
/// a way to log the next thing.
#[component]
fn History(id: u64) -> Element {
    let ui = use_context::<Ui>();
    let mut cockpit = use_context::<Signal<Cockpit>>();
    // Read so the list follows the library: a video sent them moves on.
    let library = use_context::<Signal<Vec<Item>>>();
    let rows: Vec<HistoryRow> = cockpit
        .read()
        .history(id)
        .into_iter()
        .map(|r: &Row| HistoryRow {
            id: r.id,
            date: show_date(r.get("date")),
            title: r.get("action").to_string(),
            result: r.get("result").to_string(),
            media: library
                .read()
                .iter()
                .find(|i| !r.get("media").is_empty() && i.id == r.get("media"))
                .map(|i| (format!("{}: {}", media::what(i), i.title.trim()), i.status)),
        })
        .collect();

    rsx! {
        div { class: "field",
            span { "What I did with them" }
            if rows.is_empty() {
                div { class: "history-empty", "Nothing logged in Field progress yet." }
            } else {
                div { class: "history",
                    for row in rows {
                        button {
                            key: "{row.id}",
                            class: "history-row",
                            title: "Open this action in Field progress",
                            onclick: move |_| ui.open(Sheet::Actions, row.id),
                            span { class: "history-date", "{row.date}" }
                            span { class: "history-main",
                                span { class: "history-title",
                                    if row.title.trim().is_empty() { "Action to describe" } else { "{row.title}" }
                                }
                                if !row.result.is_empty() {
                                    span { class: "history-sub", "{row.result}" }
                                }
                                if let Some((what, status)) = row.media {
                                    span { class: "history-media",
                                        span { class: "history-sub", "{what}" }
                                        span { class: "pill pill-{media::tone(status).class()}", "{status.label()}" }
                                    }
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
