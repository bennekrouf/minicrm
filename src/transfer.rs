//! A table in and out of a CSV file, to bring lines over from a
//! spreadsheet, Notion or a LinkedIn export, and to take them elsewhere.
//!
//! An export has one column per column of the table, under its label, and
//! reads contacts by name. An import matches the file's headings to the
//! table's columns by label, by key or by a common name for them ("Email
//! Address", "First Name"), and skips what it cannot place.

use chrono::NaiveDate;

use crate::model::{iso, Cockpit, Column, Kind, Sheet};

/// Where an export starts: tells Excel the file is UTF-8, so accents come
/// through.
const BOM: &str = "\u{feff}";

/// How many lines at the top of a file may come before its headings, as in
/// LinkedIn's export, which opens with a note.
const HEADER_SEARCH: usize = 20;

/// What an import did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Imported {
    pub added: usize,
    /// Contacts already in the cockpit under the same name.
    pub skipped: usize,
}

/// The table as CSV: a heading row of the column labels, then every line.
pub fn export(cockpit: &Cockpit, sheet: Sheet) -> String {
    let mut writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Necessary)
        .from_writer(Vec::new());
    let columns = sheet.columns();
    // Writing into memory cannot fail; a row that did would only be missing.
    let _ = writer.write_record(columns.iter().map(|c| c.label));
    for row in cockpit.rows(sheet) {
        let _ = writer.write_record(columns.iter().map(|c| cockpit.display(c, row)));
    }
    let bytes = writer.into_inner().unwrap_or_default();
    format!("{BOM}{}", String::from_utf8_lossy(&bytes))
}

/// Adds the lines of a CSV file to a table. Contacts whose name the cockpit
/// already has are left out, so importing the same file twice adds nothing.
pub fn import(
    cockpit: &mut Cockpit,
    sheet: Sheet,
    text: &str,
    today: NaiveDate,
) -> Result<Imported, String> {
    let text = text.trim_start_matches(BOM);
    // Comma, then the semicolon Excel writes in some countries, then tab:
    // whichever reads the most of the table's columns.
    let (records, header, mapping) = b",;\t"
        .iter()
        .copied()
        .map(|delimiter| {
            let records = read(text, delimiter);
            let (header, mapping) = find_header(sheet, &records);
            (records, header, mapping)
        })
        .max_by_key(|(_, _, mapping)| distinct(mapping))
        .unwrap_or_default();
    if mapping.is_empty() {
        let expected: Vec<&str> = sheet.columns().iter().take(3).map(|c| c.label).collect();
        return Err(format!(
            "None of this file's columns match {}. Its first line should name them, \
             for example: {}.",
            sheet.title(),
            expected.join(", ")
        ));
    }

    let mut done = Imported {
        added: 0,
        skipped: 0,
    };
    for record in records.iter().skip(header + 1) {
        let mut cells = gather(record, &mapping);
        if cells.iter().all(|(_, v)| v.trim().is_empty()) {
            continue;
        }
        if sheet == Sheet::Contacts {
            let name = cells
                .iter()
                .find(|(k, _)| *k == "name")
                .map_or("", |(_, v)| v.as_str());
            if cockpit.contact_named(name).is_some() {
                done.skipped += 1;
                continue;
            }
        }
        for (key, value) in &mut cells {
            let Some(column) = sheet.columns().iter().find(|c| c.key == *key) else {
                continue;
            };
            *value = clean(cockpit, column, value);
        }
        // A contact the cockpit does not know is kept as text, in the
        // column for whoever an action was with.
        if sheet == Sheet::Actions {
            place_unknown_contact(cockpit, &mut cells, record, &mapping);
        }

        let id = cockpit.add(sheet, today);
        // `add` gave the lines their defaults; the file's cells take over
        // where they say something.
        for (key, value) in cells {
            if !value.is_empty() {
                cockpit.set(sheet, id, key, value);
            }
        }
        done.added += 1;
    }
    Ok(done)
}

/// Every record of the file, whatever its length.
fn read(text: &str, delimiter: u8) -> Vec<Vec<String>> {
    csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .delimiter(delimiter)
        .from_reader(text.as_bytes())
        .records()
        .filter_map(Result::ok)
        .map(|r| r.iter().map(str::to_string).collect())
        .collect()
}

/// The heading row among the first lines (the one naming the most columns
/// of the table) and, for each of its cells that names one, that column's
/// key.
fn find_header(sheet: Sheet, records: &[Vec<String>]) -> (usize, Vec<(usize, &'static str)>) {
    records
        .iter()
        .take(HEADER_SEARCH)
        .enumerate()
        .map(|(index, record)| {
            let mapping: Vec<(usize, &'static str)> = record
                .iter()
                .enumerate()
                .filter_map(|(at, heading)| Some((at, column_for(sheet, heading)?)))
                .collect();
            (index, mapping)
        })
        // The first of those that tie: a heading row comes before the lines.
        .fold((0, Vec::new()), |best, next| {
            if distinct(&next.1) > distinct(&best.1) {
                next
            } else {
                best
            }
        })
}

/// How many of the table's columns a mapping fills.
fn distinct(mapping: &[(usize, &str)]) -> usize {
    let mut keys: Vec<&str> = mapping.iter().map(|(_, k)| *k).collect();
    keys.sort_unstable();
    keys.dedup();
    keys.len()
}

/// A heading with only its letters and digits, in lower case: "E-mail
/// Address" and "email address" are the same heading.
fn simplify(heading: &str) -> String {
    heading
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// The column a heading names: by its label, its key, or one of the names
/// other tools give it.
fn column_for(sheet: Sheet, heading: &str) -> Option<&'static str> {
    let heading = simplify(heading);
    if heading.is_empty() {
        return None;
    }
    sheet
        .columns()
        .iter()
        .find(|c| {
            simplify(c.label) == heading
                || simplify(c.key) == heading
                || also_called(sheet, c.key)
                    .iter()
                    .any(|a| simplify(a) == heading)
        })
        .map(|c| c.key)
}

/// What other tools call a column. LinkedIn's connections export, Google
/// and Apple contacts, and the usual spreadsheet headings.
fn also_called(sheet: Sheet, key: &str) -> &'static [&'static str] {
    match (sheet, key) {
        (Sheet::Contacts, "name") => &[
            "full name",
            "contact",
            "person",
            "first name",
            "last name",
            "given name",
            "family name",
            "surname",
        ],
        (Sheet::Contacts, "about") => &[
            "company",
            "position",
            "title",
            "job title",
            "role",
            "headline",
            "organization",
            "organisation",
            "organization name",
        ],
        (Sheet::Contacts, "source") => &["channel", "found on", "origin"],
        (Sheet::Contacts, "link") => &[
            "url",
            "profile url",
            "linkedin",
            "linkedin url",
            "website",
            "profile",
            "handle",
        ],
        (Sheet::Contacts, "email") => &["email address", "e-mail", "mail", "email 1 value"],
        (Sheet::Contacts, "stage") => &["status", "pipeline", "deal stage"],
        (Sheet::Contacts, "follow_up") => &["follow up", "follow-up date", "next contact"],
        (Sheet::Contacts, "notes") => &["note", "comment", "comments", "description"],
        (Sheet::Actions, "action") => &["activity", "what"],
        (Sheet::Actions, "target") => &["person", "who"],
        (_, "comment") => &["comments", "notes"],
        _ => &[],
    }
}

/// A record's cells by column key. Where several cells fill one column
/// (First Name and Last Name, Company and Position) they are joined in the
/// file's order.
fn gather(record: &[String], mapping: &[(usize, &'static str)]) -> Vec<(&'static str, String)> {
    let mut cells: Vec<(&'static str, String)> = Vec::new();
    for (at, key) in mapping {
        let value = record.get(*at).map_or("", |v| v.trim());
        if value.is_empty() {
            continue;
        }
        match cells.iter_mut().find(|(k, _)| k == key) {
            Some((_, kept)) => {
                let glue = if *key == "name" { " " } else { " \u{b7} " };
                kept.push_str(glue);
                kept.push_str(value);
            }
            None => cells.push((key, value.to_string())),
        }
    }
    // Every column the file has is in the list, even empty, so the caller
    // knows which ones the file spoke for.
    for (_, key) in mapping {
        if !cells.iter().any(|(k, _)| k == key) {
            cells.push((key, String::new()));
        }
    }
    cells
}

/// A cell from a file, as the column keeps it: an entry of the list as the
/// list writes it, a day year-first, a contact by id.
fn clean(cockpit: &Cockpit, column: &Column, value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }
    match column.kind {
        Kind::Choice(options) => options
            .iter()
            .find(|o| o.label.eq_ignore_ascii_case(value))
            .map_or_else(String::new, |o| o.label.to_string()),
        Kind::Date | Kind::Due => read_day(value).map_or_else(|| value.to_string(), iso),
        Kind::Contact => cockpit
            .contact_named(value)
            .map_or_else(String::new, |c| c.id.to_string()),
        Kind::Text | Kind::Long | Kind::Week => value.to_string(),
    }
}

/// A day written the way a spreadsheet or LinkedIn writes it. Day first
/// where it could be either, as MiniCRM shows days.
pub fn read_day(value: &str) -> Option<NaiveDate> {
    ["%Y-%m-%d", "%d %b %Y", "%d/%m/%Y", "%d.%m.%Y", "%Y/%m/%d"]
        .iter()
        .find_map(|format| NaiveDate::parse_from_str(value, format).ok())
}

/// An action's contact that matched no one: written into "Person / target"
/// when the file left that empty, so the name is not lost.
fn place_unknown_contact(
    cockpit: &Cockpit,
    cells: &mut Vec<(&'static str, String)>,
    record: &[String],
    mapping: &[(usize, &'static str)],
) {
    let Some(name) = mapping
        .iter()
        .find(|(_, k)| *k == "contact")
        .and_then(|(at, _)| record.get(*at))
        .map(|v| v.trim())
        .filter(|v| !v.is_empty() && cockpit.contact_named(v).is_none())
    else {
        return;
    };
    match cells.iter_mut().find(|(k, _)| *k == "target") {
        Some((_, target)) => {
            if target.is_empty() {
                *target = name.to_string();
            }
        }
        None => cells.push(("target", name.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed;

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 11).unwrap()
    }

    #[test]
    fn every_table_comes_back_as_it_went_out() {
        let examples = seed::examples(day());
        for sheet in Sheet::ALL {
            let csv = export(&examples, sheet);
            assert!(csv.starts_with(BOM));
            // Into a cockpit with the same contacts, so links resolve.
            let mut cockpit = Cockpit {
                contacts: if sheet == Sheet::Contacts {
                    Vec::new()
                } else {
                    examples.contacts.clone()
                },
                ..Cockpit::default()
            };
            let done = import(&mut cockpit, sheet, &csv, day()).unwrap();
            assert_eq!(done.added, examples.rows(sheet).len(), "{sheet:?}");
            let cells =
                |c: &Cockpit| -> Vec<_> { c.rows(sheet).iter().map(|r| r.cells.clone()).collect() };
            let mut expected = cells(&examples);
            // An empty cell is not written back.
            for row in &mut expected {
                row.retain(|_, v| !v.is_empty());
            }
            assert_eq!(cells(&cockpit), expected, "{sheet:?}");
        }
    }

    #[test]
    fn a_contact_already_there_is_not_added_twice() {
        let mut cockpit = seed::examples(day());
        let csv = export(&cockpit, Sheet::Contacts);
        let done = import(&mut cockpit, Sheet::Contacts, &csv, day()).unwrap();
        assert_eq!(
            done,
            Imported {
                added: 0,
                skipped: 5
            }
        );
    }

    #[test]
    fn a_linkedin_export_becomes_contacts() {
        let csv = "Notes:\n\
            \"When exporting your connection data, you may notice that some of the email addresses are missing.\"\n\
            \n\
            First Name,Last Name,URL,Email Address,Company,Position,Connected On\n\
            Ada,Lovelace,https://www.linkedin.com/in/ada,ada@example.com,Analytical Engines,Founder,14 Nov 2026\n\
            Alan,Turing,https://www.linkedin.com/in/alan,,,,03 Oct 2026\n";
        let mut cockpit = Cockpit::default();
        let done = import(&mut cockpit, Sheet::Contacts, csv, day()).unwrap();
        assert_eq!(done.added, 2);
        let ada = &cockpit.contacts[0];
        assert_eq!(ada.get("name"), "Ada Lovelace");
        assert_eq!(ada.get("link"), "https://www.linkedin.com/in/ada");
        assert_eq!(ada.get("email"), "ada@example.com");
        assert_eq!(ada.get("about"), "Analytical Engines \u{b7} Founder");
        assert_eq!(ada.get("stage"), "Lead");
        assert_eq!(cockpit.contacts[1].get("about"), "");
    }

    #[test]
    fn a_semicolon_file_with_other_names_and_formats_is_read() {
        // As Excel writes CSV where the comma is the decimal mark.
        let csv = "Name;Status;E-mail;Follow-up date;Hobby\n\
                   Grace Hopper;client;grace@example.com;20/11/2026;Sailing\n\
                   Edsger;Somewhere else;;;\n";
        let mut cockpit = Cockpit::default();
        import(&mut cockpit, Sheet::Contacts, csv, day()).unwrap();
        let grace = &cockpit.contacts[0];
        assert_eq!(grace.get("stage"), "Client");
        assert_eq!(grace.get("email"), "grace@example.com");
        assert_eq!(grace.get("follow_up"), "2026-11-20");
        // A stage that is not in the list falls back to the first.
        assert_eq!(cockpit.contacts[1].get("stage"), "Lead");
    }

    #[test]
    fn an_action_with_an_unknown_contact_keeps_the_name() {
        let mut cockpit = Cockpit::default();
        let lea = cockpit.add_with(Sheet::Contacts, &[("name", "Léa")]);
        let csv = "Date,Action taken,Contact\n\
                   2026-11-10,Call,léa\n\
                   2026-11-11,Coffee,Someone new\n";
        import(&mut cockpit, Sheet::Actions, csv, day()).unwrap();
        assert_eq!(cockpit.actions[0].get("contact"), lea.to_string());
        assert_eq!(cockpit.actions[1].get("contact"), "");
        assert_eq!(cockpit.actions[1].get("target"), "Someone new");
        // The type the file did not give is a new line's default.
        assert_eq!(cockpit.actions[1].get("type"), "Message sent");
    }

    #[test]
    fn a_file_that_matches_nothing_says_so() {
        let mut cockpit = Cockpit::default();
        let err = import(&mut cockpit, Sheet::Contacts, "a,b\n1,2\n", day()).unwrap_err();
        assert!(err.contains("Name, Who they are, Stage"), "{err}");
        assert!(cockpit.contacts.is_empty());
    }

    #[test]
    fn blank_lines_are_not_added() {
        let mut cockpit = Cockpit::default();
        let done = import(
            &mut cockpit,
            Sheet::Questions,
            "Question,Status\nWhy?,Clarified\n,\n\n",
            day(),
        )
        .unwrap();
        assert_eq!(done.added, 1);
        assert_eq!(cockpit.questions[0].get("status"), "Clarified");
    }
}
