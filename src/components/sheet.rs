//! One table: its lines in a grid, with a search box, a filter on its
//! drop-down column, and a sort on any column, as a spreadsheet's header row
//! would give.

use std::cmp::Ordering;

use dioxus::prelude::*;

use super::detail::Detail;
use super::Ui;
use crate::model::{show_date, today, Choice, Cockpit, Column, Kind, Row, Sheet, Tone, Width};

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
/// (To do before Done), everything else as text. Dates are kept
/// year-first, so text order is date order.
fn compare(column: &Column, a: &Row, b: &Row) -> Ordering {
    let (a, b) = (a.get(column.key), b.get(column.key));
    match column.kind {
        Kind::Choice(options) => {
            let rank = |v: &str| options.iter().position(|o| o.label == v);
            rank(a).cmp(&rank(b))
        }
        _ => a.to_lowercase().cmp(&b.to_lowercase()),
    }
}

/// The lines to show: those the filter and the search keep, in the sort's
/// order.
fn visible(
    sheet: Sheet,
    rows: &[Row],
    filter: Option<&str>,
    search: &str,
    sort: Option<(usize, bool)>,
) -> Vec<Row> {
    let filter_key = sheet.filter_column().key;
    let needle = search.trim().to_lowercase();
    let mut kept: Vec<Row> = rows
        .iter()
        .filter(|r| filter.is_none_or(|f| r.get(filter_key) == f))
        .filter(|r| needle.is_empty() || r.mentions(&needle))
        .cloned()
        .collect();
    if let Some((index, ascending)) = sort {
        if let Some(column) = sheet.columns().get(index) {
            // Stable, so lines that tie keep the order they were added in.
            kept.sort_by(|a, b| {
                let order = compare(column, a, b);
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
    let total = cockpit.read().rows(sheet).len();
    let rows = visible(sheet, cockpit.read().rows(sheet), filter, &search, sort);
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
                        let id = cockpit.write().add(sheet, today());
                        ui.selected.set(Some(id));
                    },
                    "{sheet.add_label()}"
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
                if narrowed {
                    span { class: "filters-shown", "{rows.len()} of {total}" }
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
                                                {cell(sheet, column, row.get(column.key))}
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

/// One cell of the grid.
fn cell(sheet: Sheet, column: &Column, value: &str) -> Element {
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
        Kind::Week | Kind::Text | Kind::Long => rsx! { div { class: "clamp", "{value}" } },
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
        assert_eq!(
            visible(Sheet::Actions, &c.actions, None, "", None),
            c.actions
        );
    }

    #[test]
    fn the_filter_keeps_one_entry_of_the_drop_down() {
        let c = cockpit();
        let rows = visible(Sheet::Goals, &c.goals, Some("Done"), "", None);
        assert_eq!(
            titles(Sheet::Goals, &rows),
            vec!["Prepare 5 interview questions"]
        );
    }

    #[test]
    fn the_search_reads_every_column_whatever_the_case() {
        let c = cockpit();
        let rows = visible(Sheet::Signals, &c.signals, None, "  LINKEDIN ", None);
        assert_eq!(rows.len(), 2);
        let rows = visible(Sheet::Signals, &c.signals, Some("High"), "linkedin", None);
        assert!(rows.is_empty());
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
        let up = visible(
            Sheet::Goals,
            &c.goals,
            None,
            "",
            Some((status_column, true)),
        );
        assert_eq!(status(&up), vec!["To do", "In progress", "Done"]);
        let down = visible(
            Sheet::Goals,
            &c.goals,
            None,
            "",
            Some((status_column, false)),
        );
        assert_eq!(status(&down), vec!["Done", "In progress", "To do"]);
    }

    #[test]
    fn dates_sort_oldest_first() {
        let mut c = cockpit();
        let id = c.add_with(
            Sheet::Actions,
            &[("date", "2025-01-02"), ("action", "Before")],
        );
        let rows = visible(Sheet::Actions, &c.actions, None, "", Some((0, true)));
        assert_eq!(rows[0].id, id);
    }
}
