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

/// The lines to show: those in use, or the archived ones when `archived`,
/// that the filter and the search keep, in the sort's order. A search of
/// the contacts in use also finds the network, after them.
fn visible(
    cockpit: &Cockpit,
    sheet: Sheet,
    archived: bool,
    filter: Option<&str>,
    search: &str,
    sort: Option<(usize, bool)>,
) -> Vec<Row> {
    let filter_key = sheet.filter_column().key;
    let needle = search.trim().to_lowercase();
    let mut kept: Vec<Row> = cockpit
        .rows(sheet)
        .iter()
        .filter(|r| !r.network && r.archived == archived)
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
    if sheet == Sheet::Contacts && !archived {
        kept.extend(
            cockpit
                .contacts
                .iter()
                .filter(|r| r.network && !needle.is_empty() && mentions(cockpit, sheet, r, &needle))
                .cloned(),
        );
    }
    kept
}

fn column_class(sheet: Sheet, column: &Column) -> String {
    let width = match column.width {
        Width::Narrow => "w-narrow",
        Width::Medium => "w-medium",
        Width::Wide => "w-wide",
    };
    let mut class = width.to_string();
    if !column.primary {
        class.push_str(" col-extra");
    }
    if frozen(sheet, column) {
        class.push_str(" col-freeze");
    }
    class
}

/// The column that names a line stays put when the table scrolls sideways,
/// as a frozen first column does in a spreadsheet: when it is the first.
fn frozen(sheet: Sheet, column: &Column) -> bool {
    sheet.columns().first().map(|c| c.key) == Some(column.key) && column.key == sheet.title_key()
}

/// Room a wide column keeps, at the least: about twenty characters, so a
/// name or a sentence is not broken every few letters.
const WIDE_MIN: u32 = 170;

/// The width under which the table scrolls sideways rather than squeezing
/// its columns: the narrow and medium ones as the stylesheet sets them
/// (`.w-narrow`, `.w-medium`), the wide ones at [`WIDE_MIN`].
fn min_width(sheet: Sheet) -> u32 {
    sheet
        .columns()
        .iter()
        .map(|c| match c.width {
            Width::Narrow => 116,
            Width::Medium => 168,
            Width::Wide => WIDE_MIN,
        })
        .sum()
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
    let archived = (ui.archived)();
    let put_away = cockpit.read().archived(sheet);
    let in_use = cockpit.read().active(sheet).count();
    let network = if sheet == Sheet::Contacts {
        cockpit.read().network()
    } else {
        0
    };
    // The lines of the view open: in use, or archived.
    let total = if archived { put_away } else { in_use };
    let at = |label: &str| {
        cockpit
            .read()
            .rows(sheet)
            .iter()
            .filter(|r| !r.network && r.archived == archived && r.get(filter_column.key) == label)
            .count()
    };
    let rows = visible(&cockpit.read(), sheet, archived, filter, &search, sort);
    // A line deleted, or one of another table, is not open.
    let open = (ui.selected)().filter(|id| cockpit.read().row(sheet, *id).is_some());
    let narrowed = filter.is_some() || !search.trim().is_empty();

    // A line just added sits at the end of the table, and a selected one may
    // be anywhere once sorted: bring it into view.
    use_effect(move || {
        if (ui.selected)().is_some() {
            // The table's row or the card, whichever the window's width
            // shows; the other is hidden and does not move.
            document::eval(
                "requestAnimationFrame(() => document.querySelectorAll('.row-on, .card-on')\
                 .forEach(e => e.scrollIntoView({ block: 'nearest' })));",
            );
        }
    });

    rsx! {
        div { class: "main-col",
            div { class: "col-head",
                div { class: "col-head-main",
                    div { class: "col-title",
                        if archived { "{sheet.title()} \u{b7} Archived" } else { "{sheet.title()}" }
                    }
                    div { class: "col-sub",
                        if archived {
                            "Put away: out of the table, its counts and Follow-ups. Open one to restore it."
                        } else {
                            "{sheet.question()}"
                        }
                    }
                }
                input {
                    class: "search",
                    r#type: "search",
                    placeholder: if network > 0 { "Search, network included\u{2026}" } else { "Search\u{2026}" },
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
                        ui.archived.set(false);
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
                        span { class: "chip-count", "{at(option.label)}" }
                    }
                }
                if put_away > 0 || archived {
                    button {
                        class: if archived { "chip chip-on chip-archive" } else { "chip chip-archive" },
                        title: if archived { "Back to the lines in use" } else { "The lines put away" },
                        onclick: move |_| {
                            ui.selected.set(None);
                            ui.archived.set(!archived);
                        },
                        "Archived"
                        span { class: "chip-count", "{put_away}" }
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
                                    let done = transfer::import(&mut cockpit.write(), sheet, &text, today, false);
                                    ui.filter.set(None);
                                    ui.search.set(String::new());
                                    ui.notice.set(Some(imported(sheet, &file.file_name(), done)));
                                });
                            },
                            "Import CSV"
                        }
                        if sheet == Sheet::Contacts {
                            button {
                                class: "btn btn-small",
                                title: "Add a whole LinkedIn export to your network: found by searching, out of Contacts, the counts and Follow-ups until you add someone",
                                onclick: move |_| {
                                    spawn(async move {
                                        let Some(file) = rfd::AsyncFileDialog::new()
                                            .set_title("Import into the network")
                                            .add_filter("CSV", &["csv", "txt", "tsv"])
                                            .pick_file()
                                            .await
                                        else {
                                            return;
                                        };
                                        let bytes = file.read().await;
                                        let text = String::from_utf8_lossy(&bytes);
                                        let done = transfer::import(&mut cockpit.write(), sheet, &text, today, true);
                                        ui.filter.set(None);
                                        ui.search.set(String::new());
                                        let note = match done {
                                            Ok(d) => format!(
                                                "{} people added to your network from {}{}. Search to find them.",
                                                d.added,
                                                file.file_name(),
                                                if d.skipped > 0 { format!(", {} already there", d.skipped) } else { String::new() },
                                            ),
                                            Err(e) => e,
                                        };
                                        ui.notice.set(Some(note));
                                    });
                                },
                                "Import to network"
                            }
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
                    } else if archived {
                        div { class: "placeholder-title", "Nothing archived" }
                        div { class: "placeholder-sub",
                            "A line you archive from its editor waits here, out of the way, until you restore it."
                        }
                        button { class: "btn", onclick: move |_| ui.archived.set(false), "Back to the table" }
                    } else {
                        div { class: "placeholder-title", "This table is empty" }
                        div { class: "placeholder-sub",
                            "{sheet.question()} Add a first line with the \u{201c}{sheet.add_label()}\u{201d} button."
                        }
                    }
                }
            } else {
                div { class: if open.is_some() { "grid-scroll grid-beside" } else { "grid-scroll" },
                    table {
                        class: if open.is_some() { "grid grid-compact" } else { "grid" },
                        style: "--grid-min: {min_width(sheet)}px",
                        thead {
                            tr {
                                for (index, column) in columns.iter().enumerate() {
                                    th {
                                        key: "{column.key}",
                                        class: column_class(sheet, column),
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
                                        class: match (open == Some(id), row.network) {
                                            (true, true) => "row row-on row-network",
                                            (true, false) => "row row-on",
                                            (false, true) => "row row-network",
                                            (false, false) => "row",
                                        },
                                        title: if row.network { "In your network: open to add to Contacts" } else { "" },
                                        onclick: move |_| ui.selected.set(Some(id)),
                                        for column in columns {
                                            td { key: "{column.key}", class: column_class(sheet, column),
                                                {cell(sheet, column, &cockpit.read().display(column, row), today)}
                                            }
                                        }
                                    }
                                }
                            })}
                        }
                    }
                    // The same lines as cards, which the stylesheet shows
                    // instead of the table once the window is too narrow
                    // for its columns.
                    div { class: "cards",
                        {rows.iter().map(|row| {
                            let id = row.id;
                            rsx! {
                                Card {
                                    key: "{id}",
                                    sheet,
                                    row: row.clone(),
                                    on: open == Some(id),
                                }
                            }
                        })}
                    }
                }
            }
        }
        if let Some(id) = open {
            // Behind the editor when it lies over the table, in a narrow
            // window: a click beside it closes it, as Esc does.
            div { class: "detail-backdrop", onclick: move |_| ui.selected.set(None) }
            Detail { sheet, id }
        }
    }
}

/// What a card shows of a line, under its name: the columns that stay in
/// the grid beside the editor, as text lines (a contact's next step) and as
/// a row of pills and dates (their follow-up and stage), each in the
/// table's order.
fn card_parts(sheet: Sheet) -> (Vec<&'static Column>, Vec<&'static Column>) {
    sheet
        .columns()
        .iter()
        .filter(|c| c.primary && c.key != sheet.title_key())
        .partition(|c| {
            matches!(
                c.kind,
                Kind::Text | Kind::Long | Kind::Contact | Kind::Media
            )
        })
}

/// One line as a card, for a narrow window. A click opens it in the editor,
/// as a click on its row does.
#[component]
fn Card(sheet: Sheet, row: Row, on: bool) -> Element {
    let mut ui = use_context::<Ui>();
    let cockpit = use_context::<Signal<Cockpit>>();
    let today = (ui.today)();
    let id = row.id;
    let title = row.get(sheet.title_key()).trim().to_string();
    let (lines, meta) = card_parts(sheet);
    let cockpit = cockpit.read();
    let lines: Vec<(&str, String)> = lines
        .into_iter()
        .map(|c| (c.label, cockpit.display(c, &row)))
        .filter(|(_, v)| !v.trim().is_empty())
        .collect();
    let meta: Vec<Element> = meta
        .into_iter()
        .map(|c| (c, cockpit.display(c, &row)))
        .filter(|(_, v)| !v.trim().is_empty())
        .map(|(c, value)| {
            // A bare "1" says nothing on a card: "Priority 1" does.
            let value = if value.chars().all(|ch| ch.is_ascii_digit()) {
                format!("{} {value}", c.label)
            } else {
                value
            };
            rsx! {
                span { key: "{c.key}", title: "{c.label}", {cell(sheet, c, &value, today)} }
            }
        })
        .collect();

    rsx! {
        button {
            class: if on { "card card-on" } else { "card" },
            title: "Open this line",
            onclick: move |_| ui.selected.set(Some(id)),
            if title.is_empty() {
                span { class: "card-title cell-empty", "To fill in" }
            } else {
                span { class: "card-title clamp", "{title}" }
            }
            for (label, value) in lines {
                span { key: "{label}", class: "card-line",
                    span { class: "card-label", "{label}" }
                    span { class: "clamp", "{value}" }
                }
            }
            if !meta.is_empty() {
                span { class: "card-meta", {meta.into_iter()} }
            }
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
        Kind::Week | Kind::Text | Kind::Long | Kind::Contact | Kind::Media => {
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
        assert_eq!(
            visible(&c, Sheet::Actions, false, None, "", None),
            c.actions
        );
    }

    #[test]
    fn the_network_shows_only_in_a_search_of_contacts() {
        let mut c = Cockpit::default();
        c.add_with(Sheet::Contacts, &[("name", "Léa Martin")]);
        let ada = c.add_with(Sheet::Contacts, &[("name", "Ada Lovelace")]);
        c.set_network(ada, true);
        let names = |search: &str| -> Vec<String> {
            visible(&c, Sheet::Contacts, false, None, search, None)
                .iter()
                .map(|r| r.get("name").to_string())
                .collect()
        };
        assert_eq!(names(""), ["Léa Martin"]);
        assert_eq!(names("ada"), ["Ada Lovelace"]);
        assert_eq!(names("a"), ["Léa Martin", "Ada Lovelace"]);
    }

    #[test]
    fn archived_lines_have_a_view_of_their_own() {
        let mut c = cockpit();
        let tom = c.contact_named("Tom Leroy").unwrap().id;
        c.set_archived(Sheet::Contacts, tom, true);
        let in_use = visible(&c, Sheet::Contacts, false, None, "", None);
        assert_eq!(in_use.len(), 4);
        assert!(in_use.iter().all(|r| r.id != tom));
        let put_away = visible(&c, Sheet::Contacts, true, None, "tom", None);
        assert_eq!(titles(Sheet::Contacts, &put_away), vec!["Tom Leroy"]);
    }

    #[test]
    fn the_filter_keeps_one_entry_of_the_drop_down() {
        let c = cockpit();
        let rows = visible(&c, Sheet::Goals, false, Some("Done"), "", None);
        assert_eq!(
            titles(Sheet::Goals, &rows),
            vec!["Prepare 5 interview questions"]
        );
    }

    #[test]
    fn the_search_reads_every_column_whatever_the_case() {
        let c = cockpit();
        let rows = visible(&c, Sheet::Signals, false, None, "  LINKEDIN ", None);
        assert_eq!(rows.len(), 2);
        let rows = visible(&c, Sheet::Signals, false, Some("High"), "linkedin", None);
        assert!(rows.is_empty());
    }

    #[test]
    fn an_action_is_found_and_sorted_by_its_contact_name() {
        let c = cockpit();
        let rows = visible(&c, Sheet::Actions, false, None, "sophie", None);
        assert_eq!(
            titles(Sheet::Actions, &rows),
            vec!["Pre-sale offer sent to Sophie"]
        );
        let rows = visible(&c, Sheet::Actions, false, None, "léa martin", None);
        assert_eq!(
            titles(Sheet::Actions, &rows),
            vec!["Interview with a freelance graphic designer"]
        );
        let contact = Sheet::Actions
            .columns()
            .iter()
            .position(|col| col.key == "contact")
            .unwrap();
        let rows = visible(&c, Sheet::Actions, false, None, "", Some((contact, false)));
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
        let rows = visible(&c, Sheet::Contacts, false, None, "", Some((due, true)));
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
    fn a_contact_card_reads_their_next_step_then_when_and_where_they_stand() {
        let keys = |cols: Vec<&Column>| cols.iter().map(|c| c.key).collect::<Vec<_>>();
        let (lines, meta) = card_parts(Sheet::Contacts);
        assert_eq!(keys(lines), vec!["next"]);
        assert_eq!(keys(meta), vec!["follow_up", "stage"]);
        let (lines, meta) = card_parts(Sheet::Actions);
        assert_eq!(keys(lines), vec!["contact"]);
        assert_eq!(keys(meta), vec!["date", "type"]);
        // Every table's card has something under its name.
        for sheet in Sheet::ALL {
            let (lines, meta) = card_parts(sheet);
            assert!(!lines.is_empty() || !meta.is_empty(), "{sheet:?}");
        }
    }

    #[test]
    fn a_table_scrolls_sideways_before_its_wide_columns_get_too_narrow() {
        // Name, Next step, Who they are, Profile, Notes at 170; Next
        // follow-up narrow; Stage, Where I found them, Email medium.
        assert_eq!(min_width(Sheet::Contacts), 5 * 170 + 116 + 3 * 168);
        let name = &Sheet::Contacts.columns()[0];
        assert!(frozen(Sheet::Contacts, name));
        assert!(column_class(Sheet::Contacts, name).contains("col-freeze"));
        // A goal's first column is its week, which is not worth freezing.
        assert!(!frozen(Sheet::Goals, &Sheet::Goals.columns()[0]));
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
        let up = visible(
            &c,
            Sheet::Goals,
            false,
            None,
            "",
            Some((status_column, true)),
        );
        assert_eq!(status(&up), vec!["To do", "In progress", "Done"]);
        let down = visible(
            &c,
            Sheet::Goals,
            false,
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
        let rows = visible(&c, Sheet::Actions, false, None, "", Some((0, true)));
        assert_eq!(rows[0].id, id);
    }
}
