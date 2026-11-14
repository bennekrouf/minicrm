//! One table: its lines in a grid, with a search box, a filter on its
//! drop-down column, and a sort on any column, as a spreadsheet's header row
//! would give. The table goes in and out as a CSV file from its heading.

use std::cmp::Ordering;

use chrono::NaiveDate;
use dioxus::prelude::*;

use super::detail::Detail;
use super::Ui;
use crate::model::{show_date, Choice, Cockpit, Column, Kind, Row, Sheet, Tone, Urgency, Width};
use crate::transfer;

/// The entries of a drop-down column; none for any other.
fn options(column: &Column) -> &'static [Choice] {
    match column.kind {
        Kind::Choice(options) => options,
        _ => &[],
    }
}

/// How an entry is coloured; neutral for text that is not in the list, which
/// only a file edited by hand can hold.
fn tone(column: &Column, value: &str) -> Tone {
    options(column)
        .iter()
        .find(|o| o.label == value)
        .map_or(Tone::Neutral, |o| o.tone)
}

/// Two lines' order in one column: a drop-down sorts as its list reads
/// (To do before Done), a contact by name, everything else as text. Dates
/// are kept year-first, so text order is date order; a line with no
/// follow-up date comes after those that have one.
fn compare(cockpit: &Cockpit, column: &Column, a: &Row, b: &Row) -> Ordering {
    let (a, b) = (cockpit.display(column, a), cockpit.display(column, b));
    match column.kind {
        Kind::Choice(options) => {
            let rank = |v: &str| options.iter().position(|o| o.label == v);
            rank(&a).cmp(&rank(&b))
        }
        Kind::Due => (a.is_empty(), &a).cmp(&(b.is_empty(), &b)),
        _ => a.to_lowercase().cmp(&b.to_lowercase()),
    }
}

/// Whether any column of a line reads `needle`, whatever the case; a
/// contact by name. `needle` is already lower case.
fn mentions(cockpit: &Cockpit, sheet: Sheet, row: &Row, needle: &str) -> bool {
    sheet
        .columns()
        .iter()
        .any(|c| cockpit.display(c, row).to_lowercase().contains(needle))
}

/// The lines to show: those the filter and the search keep, in the sort's
/// order.
fn visible(
    cockpit: &Cockpit,
    sheet: Sheet,
    filter: Option<&str>,
    search: &str,
    sort: Option<(usize, bool)>,
) -> Vec<Row> {
    let filter_key = sheet.filter_column().key;
    let needle = search.trim().to_lowercase();
    let mut kept: Vec<Row> = cockpit
        .rows(sheet)
        .iter()
        .filter(|r| filter.is_none_or(|f| r.get(filter_key) == f))
        .filter(|r| needle.is_empty() || mentions(cockpit, sheet, r, &needle))
        .cloned()
        .collect();
    if let Some((index, ascending)) = sort {
        if let Some(column) = sheet.columns().get(index) {
            // Stable, so lines that tie keep the order they were added in.
            kept.sort_by(|a, b| {
                let order = compare(cockpit, column, a, b);
                if ascending {
                    order
                } else {
                    order.reverse()
                }
            });
        }
    }
    kept
}

fn column_class(column: &Column) -> String {
    let width = match column.width {
        Width::Narrow => "w-narrow",
        Width::Medium => "w-medium",
        Width::Wide => "w-wide",
    };
    if column.primary {
        width.to_string()
    } else {
        format!("{width} col-extra")
    }
}

#[component]
pub fn SheetView(sheet: Sheet) -> Element {
    let mut ui = use_context::<Ui>();
    let mut cockpit = use_context::<Signal<Cockpit>>();
    let columns = sheet.columns();
    let filter_column = sheet.filter_column();

    let search = (ui.search)();
    let filter = (ui.filter)();
    let sort = (ui.sort)();
    let today = (ui.today)();
    let total = cockpit.read().rows(sheet).len();
    let rows = visible(&cockpit.read(), sheet, filter, &search, sort);
    // A line deleted, or one of another table, is not open.
    let open = (ui.selected)().filter(|id| cockpit.read().row(sheet, *id).is_some());
    let narrowed = filter.is_some() || !search.trim().is_empty();

    // A line just added sits at the end of the table, and a selected one may
    // be anywhere once sorted: bring it into view.
    use_effect(move || {
        if (ui.selected)().is_some() {
            document::eval(
                "requestAnimationFrame(() => document.querySelector('tr.row-on')\
                 ?.scrollIntoView({ block: 'nearest' }));",
            );
        }
    });

    rsx! {
        div { class: "main-col",
            div { class: "col-head",
                div { class: "col-head-main",
                    div { class: "col-title", "{sheet.title()}" }
                    div { class: "col-sub", "{sheet.question()}" }
                }
                input {
                    class: "search",
                    r#type: "search",
                    placeholder: "Search\u{2026}",
                    value: "{search}",
                    oninput: move |e| ui.search.set(e.value()),
                }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        // The new line must be on screen, whatever was
                        // filtered out a moment ago.
                        ui.filter.set(None);
                        ui.search.set(String::new());
                        let id = cockpit.write().add(sheet, today);
                        ui.selected.set(Some(id));
                    },
                    "{sheet.add_label()}"
                }
            }
            if let Some(note) = (ui.notice)() {
                div { class: "notice",
                    span { "{note}" }
                    button {
                        class: "notice-close",
                        aria_label: "Dismiss",
                        onclick: move |_| ui.notice.set(None),
                        "\u{d7}"
                    }
                }
            }
            div { class: "filters",
                span { class: "filters-label", "{filter_column.label}" }
                button {
                    class: if filter.is_none() { "chip chip-on" } else { "chip" },
                    onclick: move |_| ui.filter.set(None),
                    "All"
                    span { class: "chip-count", "{total}" }
                }
                for option in options(filter_column) {
                    button {
                        key: "{option.label}",
                        class: if filter == Some(option.label) { "chip chip-on" } else { "chip" },
                        onclick: move |_| ui.filter.set(Some(option.label)),
                        "{option.label}"
                        span { class: "chip-count",
                            "{cockpit.read().count(sheet, filter_column.key, option.label)}"
                        }
                    }
                }
                div { class: "filters-tools",
                    if narrowed {
                        span { class: "filters-shown", "{rows.len()} of {total}" }
                    }
                        button {
                            class: "btn btn-small",
                            title: "Add lines from a CSV file: a spreadsheet, Notion, or a LinkedIn export",
                            onclick: move |_| {
                                spawn(async move {
                                    let Some(file) = rfd::AsyncFileDialog::new()
                                        .set_title(format!("Import into {}", sheet.title()))
                                        .add_filter("CSV", &["csv", "txt", "tsv"])
                                        .pick_file()
                                        .await
                                    else {
                                        return;
                                    };
                                    let bytes = file.read().await;
                                    let text = String::from_utf8_lossy(&bytes);
                                    let done = transfer::import(&mut cockpit.write(), sheet, &text, today);
                                    ui.filter.set(None);
                                    ui.search.set(String::new());
                                    ui.notice.set(Some(imported(sheet, &file.file_name(), done)));
                                });
                            },
                            "Import CSV"
                        }
                        button {
                            class: "btn btn-small",
                            title: "Save every line of this table as a CSV file",
                            disabled: total == 0,
                            onclick: move |_| {
                                let csv = transfer::export(&cockpit.read(), sheet);
                                spawn(async move {
                                    let Some(file) = rfd::AsyncFileDialog::new()
                                        .set_title(format!("Export {}", sheet.title()))
                                        .set_file_name(sheet.file_name())
                                        .add_filter("CSV", &["csv"])
                                        .save_file()
                                        .await
                                    else {
                                        return;
                                    };
                                    let note = match std::fs::write(file.path(), csv) {
                                        Ok(()) => format!("{} lines saved to {}", total, file.path().display()),
                                        Err(e) => format!("Could not save {}: {e}", file.path().display()),
                                    };
                                    ui.notice.set(Some(note));
                                });
                            },
                            "Export CSV"
                        }
                }
            }
            if rows.is_empty() {
                div { class: "placeholder",
                    if narrowed {
                        div { class: "placeholder-title", "No line matches" }
                        div { class: "placeholder-sub",
                            "No line of this table passes the current filter or search."
                        }
                        button {
                            class: "btn",
                            onclick: move |_| {
                                ui.filter.set(None);
                                ui.search.set(String::new());
                            },
                            "Show all"
                        }
                    } else {
                        div { class: "placeholder-title", "This table is empty" }
                        div { class: "placeholder-sub",
                            "{sheet.question()} Add a first line with the \u{201c}{sheet.add_label()}\u{201d} button."
                        }
                    }
                }
            } else {
                div { class: "grid-scroll",
                    table { class: if open.is_some() { "grid grid-compact" } else { "grid" },
                        thead {
                            tr {
                                for (index, column) in columns.iter().enumerate() {
                                    th {
                                        key: "{column.key}",
                                        class: column_class(column),
                                        title: "Sort by \u{201c}{column.label}\u{201d}",
                                        onclick: move |_| {
                                            // Ascending, descending, then back
                                            // to the order of entry.
                                            let next = match sort {
                                                Some((i, true)) if i == index => Some((index, false)),
                                                Some((i, false)) if i == index => None,
                                                _ => Some((index, true)),
                                            };
                                            ui.sort.set(next);
                                        },
                                        "{column.label}"
                                        match sort {
                                            Some((i, ascending)) if i == index => rsx! {
                                                span { class: "sort-mark",
                                                    if ascending { "\u{25b2}" } else { "\u{25bc}" }
                                                }
                                            },
                                            _ => rsx! {},
                                        }
                                    }
                                }
                            }
                        }
                        tbody {
                            {rows.iter().map(|row| {
                                let id = row.id;
                                rsx! {
                                    tr {
                                        key: "{id}",
                                        class: if open == Some(id) { "row row-on" } else { "row" },
                                        onclick: move |_| ui.selected.set(Some(id)),
                                        for column in columns {
                                            td { key: "{column.key}", class: column_class(column),
                                                {cell(sheet, column, &cockpit.read().display(column, row), today)}
                                            }
                                        }
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }
        if let Some(id) = open {
            Detail { sheet, id }
        }
    }
}

/// What an import says it did.
fn imported(sheet: Sheet, file: &str, done: Result<transfer::Imported, String>) -> String {
    match done {
        Err(why) => format!("Nothing imported from {file}. {why}"),
        Ok(done) => {
            let lines = if done.added == 1 { "line" } else { "lines" };
            let mut note = format!(
                "{} {lines} added to {} from {file}",
                done.added,
                sheet.title()
            );
            if done.skipped > 0 {
                note.push_str(&format!(
                    "; {} left out, already among the contacts",
                    done.skipped
                ));
            }
            note.push('.');
            note
        }
    }
}

/// One cell of the grid, `value` as [`Cockpit::display`] reads it.
fn cell(sheet: Sheet, column: &Column, value: &str, today: NaiveDate) -> Element {
    // The cell that names the line is a button: a click anywhere on the row
    // opens it, and this is what the keyboard and a screen reader open it by.
    // It has no handler of its own; its click reaches the row's.
    if column.key == sheet.title_key() {
        return rsx! {
            button { class: "row-open", title: "Open this line",
                if value.trim().is_empty() {
                    span { class: "cell-empty", "To fill in" }
                } else {
                    span { class: "clamp", "{value}" }
                }
            }
        };
    }
    if value.trim().is_empty() {
        return rsx! {};
    }
    match column.kind {
        Kind::Choice(_) => rsx! {
            span { class: "pill pill-{tone(column, value).class()}", "{value}" }
        },
        Kind::Date => rsx! { span { class: "cell-date", "{show_date(value)}" } },
        // Coloured as it comes round: red once late, yellow this week.
        Kind::Due => {
            let tone = Urgency::of(value, today).map_or(Tone::Neutral, Urgency::tone);
            rsx! { span { class: "pill pill-{tone.class()} cell-date", "{show_date(value)}" } }
        }
        Kind::Week | Kind::Text | Kind::Long | Kind::Contact => {
            rsx! { div { class: "clamp", "{value}" } }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed;
    use chrono::NaiveDate;

    fn cockpit() -> Cockpit {
        seed::examples(NaiveDate::from_ymd_opt(2026, 11, 11).unwrap())
    }

    fn titles(sheet: Sheet, rows: &[Row]) -> Vec<String> {
        rows.iter()
            .map(|r| r.get(sheet.title_key()).to_string())
            .collect()
    }

    #[test]
    fn nothing_narrowed_shows_every_line_in_order_of_entry() {
        let c = cockpit();
        assert_eq!(visible(&c, Sheet::Actions, None, "", None), c.actions);
    }

    #[test]
    fn the_filter_keeps_one_entry_of_the_drop_down() {
        let c = cockpit();
        let rows = visible(&c, Sheet::Goals, Some("Done"), "", None);
        assert_eq!(
            titles(Sheet::Goals, &rows),
            vec!["Prepare 5 interview questions"]
        );
    }

    #[test]
    fn the_search_reads_every_column_whatever_the_case() {
        let c = cockpit();
        let rows = visible(&c, Sheet::Signals, None, "  LINKEDIN ", None);
        assert_eq!(rows.len(), 2);
        let rows = visible(&c, Sheet::Signals, Some("High"), "linkedin", None);
        assert!(rows.is_empty());
    }

    #[test]
    fn an_action_is_found_and_sorted_by_its_contact_name() {
        let c = cockpit();
        let rows = visible(&c, Sheet::Actions, None, "sophie", None);
        assert_eq!(
            titles(Sheet::Actions, &rows),
            vec!["Pre-sale offer sent to Sophie"]
        );
        let rows = visible(&c, Sheet::Actions, None, "léa martin", None);
        assert_eq!(
            titles(Sheet::Actions, &rows),
            vec!["Interview with a freelance graphic designer"]
        );
        let contact = Sheet::Actions
            .columns()
            .iter()
            .position(|col| col.key == "contact")
            .unwrap();
        let rows = visible(&c, Sheet::Actions, None, "", Some((contact, false)));
        assert_eq!(rows[0].get("action"), "Pre-sale offer sent to Sophie");
    }

    #[test]
    fn contacts_without_a_follow_up_sort_last() {
        let c = cockpit();
        let due = Sheet::Contacts
            .columns()
            .iter()
            .position(|col| col.key == "follow_up")
            .unwrap();
        let rows = visible(&c, Sheet::Contacts, None, "", Some((due, true)));
        assert_eq!(
            titles(Sheet::Contacts, &rows),
            vec![
                "Sophie Durand",
                "Karim Benali",
                "Léa Martin",
                "Nina Rossi",
                "Tom Leroy",
            ]
        );
    }

    #[test]
    fn an_import_says_what_it_did() {
        let done = Ok(transfer::Imported {
            added: 3,
            skipped: 2,
        });
        assert_eq!(
            imported(Sheet::Contacts, "Connections.csv", done),
            "3 lines added to Contacts from Connections.csv; 2 left out, already among the contacts."
        );
    }

    #[test]
    fn a_drop_down_sorts_as_its_list_reads() {
        let c = cockpit();
        let status_column = Sheet::Goals
            .columns()
            .iter()
            .position(|col| col.key == "status")
            .unwrap();
        let status = |rows: &[Row]| -> Vec<String> {
            rows.iter().map(|r| r.get("status").to_string()).collect()
        };
        let up = visible(&c, Sheet::Goals, None, "", Some((status_column, true)));
        assert_eq!(status(&up), vec!["To do", "In progress", "Done"]);
        let down = visible(&c, Sheet::Goals, None, "", Some((status_column, false)));
        assert_eq!(status(&down), vec!["Done", "In progress", "To do"]);
    }

    #[test]
    fn dates_sort_oldest_first() {
        let mut c = cockpit();
        let id = c.add_with(
            Sheet::Actions,
            &[("date", "2025-01-02"), ("action", "Before")],
        );
        let rows = visible(&c, Sheet::Actions, None, "", Some((0, true)));
        assert_eq!(rows[0].id, id);
    }
}
