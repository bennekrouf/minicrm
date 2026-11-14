//! The cockpit: four tables, one per weekly question, and the contacts the
//! work is done with.
//!
//! Each table is described once, as a list of columns, and everything else
//! (the grid, the editor on the right, the filter, a new line's defaults)
//! is read from that description. A line keeps its cells by column key, so
//! the saved file stays readable and a column added later reads as empty in
//! lines written before it.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

/// One of the cockpit's tables.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sheet {
    Goals,
    Actions,
    Signals,
    Questions,
    /// The people: who they are and where each one stands.
    Contacts,
}

/// How a choice is coloured, in the grid and in the editor.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tone {
    Neutral,
    Soft,
    Good,
    Warn,
    Bad,
}

impl Tone {
    /// The stylesheet's name for it: `pill-good`, `choice-good`.
    pub fn class(self) -> &'static str {
        match self {
            Tone::Neutral => "neutral",
            Tone::Soft => "soft",
            Tone::Good => "good",
            Tone::Warn => "warn",
            Tone::Bad => "bad",
        }
    }
}

/// One entry of a drop-down list.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Choice {
    pub label: &'static str,
    pub tone: Tone,
}

const fn choice(label: &'static str, tone: Tone) -> Choice {
    Choice { label, tone }
}

/// What a column holds, which decides how it is shown and edited.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// One line of text.
    Text,
    /// A sentence or more.
    Long,
    /// A day, kept as `2026-11-09` so it sorts as text.
    Date,
    /// The week a priority belongs to, e.g. `W46`.
    Week,
    /// One of a fixed list.
    Choice(&'static [Choice]),
    /// A day something is due, kept as a [`Kind::Date`] is. Starts empty,
    /// and a line with one shows among the follow-ups once it comes round.
    Due,
    /// A line of the Contacts table, kept as its id.
    Contact,
}

/// How much room a column takes in the grid.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Width {
    Narrow,
    Medium,
    Wide,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Column {
    /// Its name in the saved file.
    pub key: &'static str,
    pub label: &'static str,
    pub kind: Kind,
    pub width: Width,
    /// Stays in the grid while the editor is open beside it; the others
    /// step aside and are read in the editor.
    pub primary: bool,
}

const fn col(
    key: &'static str,
    label: &'static str,
    kind: Kind,
    width: Width,
    primary: bool,
) -> Column {
    Column {
        key,
        label,
        kind,
        width,
        primary,
    }
}

pub const GOAL_STATUS: &[Choice] = &[
    choice("To do", Tone::Neutral),
    choice("In progress", Tone::Soft),
    choice("Done", Tone::Good),
    choice("Blocked", Tone::Bad),
];

const PRIORITY: &[Choice] = &[
    choice("1", Tone::Neutral),
    choice("2", Tone::Neutral),
    choice("3", Tone::Neutral),
];

const ACTION_TYPE: &[Choice] = &[
    choice("Message sent", Tone::Neutral),
    choice("Call", Tone::Neutral),
    choice("Interview", Tone::Neutral),
    choice("Test", Tone::Neutral),
    choice("Post", Tone::Neutral),
    choice("Research", Tone::Neutral),
    choice("Other", Tone::Neutral),
];

const IMPORTANCE: &[Choice] = &[
    choice("Low", Tone::Neutral),
    choice("Medium", Tone::Soft),
    choice("High", Tone::Warn),
];

const QUESTION_STATUS: &[Choice] = &[
    choice("To clarify", Tone::Neutral),
    choice("Being tested", Tone::Soft),
    choice("Clarified", Tone::Good),
];

const GOALS: &[Column] = &[
    col("week", "Week", Kind::Week, Width::Narrow, true),
    col(
        "priority",
        "Priority",
        Kind::Choice(PRIORITY),
        Width::Narrow,
        true,
    ),
    col("goal", "Goal", Kind::Text, Width::Wide, true),
    col("action", "Main action", Kind::Long, Width::Wide, false),
    col(
        "status",
        "Status",
        Kind::Choice(GOAL_STATUS),
        Width::Medium,
        true,
    ),
    col("next", "Next step", Kind::Long, Width::Wide, false),
    col("comment", "Comment", Kind::Long, Width::Wide, false),
];

const ACTIONS: &[Column] = &[
    col("date", "Date", Kind::Date, Width::Narrow, true),
    col("action", "Action taken", Kind::Text, Width::Wide, true),
    col(
        "type",
        "Type of action",
        Kind::Choice(ACTION_TYPE),
        Width::Medium,
        true,
    ),
    col("contact", "Contact", Kind::Contact, Width::Medium, true),
    col("target", "Person / target", Kind::Text, Width::Wide, false),
    col("result", "Result", Kind::Long, Width::Wide, false),
    col("learned", "What I learned", Kind::Long, Width::Wide, false),
    col("follow_up", "Follow-up", Kind::Long, Width::Wide, false),
    col(
        "follow_up_on",
        "Follow up on",
        Kind::Due,
        Width::Narrow,
        false,
    ),
];

const SIGNALS: &[Column] = &[
    col("date", "Date", Kind::Date, Width::Narrow, true),
    col("signal", "Signal observed", Kind::Long, Width::Wide, true),
    col("source", "Source", Kind::Text, Width::Medium, false),
    col(
        "meaning",
        "What it might mean",
        Kind::Long,
        Width::Wide,
        false,
    ),
    col(
        "importance",
        "Importance",
        Kind::Choice(IMPORTANCE),
        Width::Medium,
        true,
    ),
    col("action", "Possible action", Kind::Long, Width::Wide, false),
];

/// Where a contact stands, from first noticed to client.
pub const STAGES: &[Choice] = &[
    choice("Lead", Tone::Neutral),
    choice("Contacted", Tone::Soft),
    choice("In conversation", Tone::Soft),
    choice("Proposal sent", Tone::Warn),
    choice("Client", Tone::Good),
    choice("Not now", Tone::Bad),
];

const QUESTIONS: &[Column] = &[
    col("question", "Question", Kind::Long, Width::Wide, true),
    col("why", "Why it matters", Kind::Long, Width::Wide, false),
    col(
        "hypothesis",
        "Current hypothesis",
        Kind::Long,
        Width::Wide,
        false,
    ),
    col("check", "How I'll check", Kind::Long, Width::Wide, false),
    col(
        "status",
        "Status",
        Kind::Choice(QUESTION_STATUS),
        Width::Medium,
        true,
    ),
    col(
        "answer",
        "Answer / what I learned",
        Kind::Long,
        Width::Wide,
        false,
    ),
];

const CONTACTS: &[Column] = &[
    col("name", "Name", Kind::Text, Width::Wide, true),
    col("about", "Who they are", Kind::Text, Width::Wide, false),
    col("stage", "Stage", Kind::Choice(STAGES), Width::Medium, true),
    col(
        "source",
        "Where I found them",
        Kind::Text,
        Width::Medium,
        false,
    ),
    col("link", "Profile or website", Kind::Text, Width::Wide, false),
    col("email", "Email", Kind::Text, Width::Medium, false),
    col(
        "follow_up",
        "Next follow-up",
        Kind::Due,
        Width::Narrow,
        true,
    ),
    col("next", "Next step", Kind::Long, Width::Wide, false),
    col("notes", "Notes", Kind::Long, Width::Wide, false),
];

impl Sheet {
    /// The four tables of the weekly routine, in its order.
    pub const ROUTINE: [Sheet; 4] = [
        Sheet::Goals,
        Sheet::Actions,
        Sheet::Signals,
        Sheet::Questions,
    ];

    /// Every table: the routine's, then the contacts.
    pub const ALL: [Sheet; 5] = [
        Sheet::Goals,
        Sheet::Actions,
        Sheet::Signals,
        Sheet::Questions,
        Sheet::Contacts,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Sheet::Goals => "Weekly goals",
            Sheet::Actions => "Field progress",
            Sheet::Signals => "Weak signals",
            Sheet::Questions => "Open questions",
            Sheet::Contacts => "Contacts",
        }
    }

    /// The weekly question this table answers.
    pub fn question(self) -> &'static str {
        match self {
            Sheet::Goals => "What did I mean to do?",
            Sheet::Actions => "What did I actually do?",
            Sheet::Signals => "What did I notice?",
            Sheet::Questions => "What do I need to clear up now?",
            Sheet::Contacts => "Who am I talking to, and where do they stand?",
        }
    }

    /// The step of the weekly routine that fills this table.
    pub fn step(self) -> &'static str {
        match self {
            Sheet::Goals => "Set my 3 priorities for the week",
            Sheet::Actions => "Log my field actions",
            Sheet::Signals => "Capture the weak signals I notice",
            Sheet::Questions => "Clear up my open questions",
            Sheet::Contacts => "Keep my contacts and their stage up to date",
        }
    }

    /// What the button that adds a line says.
    pub fn add_label(self) -> &'static str {
        match self {
            Sheet::Goals => "+ Goal",
            Sheet::Actions => "+ Action",
            Sheet::Signals => "+ Signal",
            Sheet::Questions => "+ Question",
            Sheet::Contacts => "+ Contact",
        }
    }

    pub fn columns(self) -> &'static [Column] {
        match self {
            Sheet::Goals => GOALS,
            Sheet::Actions => ACTIONS,
            Sheet::Signals => SIGNALS,
            Sheet::Questions => QUESTIONS,
            Sheet::Contacts => CONTACTS,
        }
    }

    /// The column a line is named by, at the top of its editor.
    pub fn title_key(self) -> &'static str {
        match self {
            Sheet::Goals => "goal",
            Sheet::Actions => "action",
            Sheet::Signals => "signal",
            Sheet::Questions => "question",
            Sheet::Contacts => "name",
        }
    }

    /// The drop-down column the grid is filtered by.
    pub fn filter_column(self) -> &'static Column {
        let key = match self {
            Sheet::Goals | Sheet::Questions => "status",
            Sheet::Actions => "type",
            Sheet::Signals => "importance",
            Sheet::Contacts => "stage",
        };
        // Every table has the column named above; the first one stands in
        // rather than a panic if a rename ever misses this list.
        self.columns()
            .iter()
            .find(|c| c.key == key)
            .unwrap_or(&self.columns()[0])
    }

    /// The file an export of this table is offered as.
    pub fn file_name(self) -> &'static str {
        match self {
            Sheet::Goals => "weekly-goals.csv",
            Sheet::Actions => "field-progress.csv",
            Sheet::Signals => "weak-signals.csv",
            Sheet::Questions => "open-questions.csv",
            Sheet::Contacts => "contacts.csv",
        }
    }

    /// The column of a line's follow-up date and the one that says what to
    /// do then; none for a table without follow-ups.
    pub fn follow_up(self) -> Option<(&'static str, &'static str)> {
        match self {
            Sheet::Actions => Some(("follow_up_on", "follow_up")),
            Sheet::Contacts => Some(("follow_up", "next")),
            Sheet::Goals | Sheet::Signals | Sheet::Questions => None,
        }
    }
}

/// `2026-11-09` as `9 Nov 2026`. Anything else is shown as it was typed.
pub fn show_date(iso: &str) -> String {
    NaiveDate::parse_from_str(iso, "%Y-%m-%d")
        .map_or_else(|_| iso.to_string(), |d| d.format("%-d %b %Y").to_string())
}

pub fn today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

/// A day as the file keeps it: `2026-11-09`.
pub fn iso(day: NaiveDate) -> String {
    day.format("%Y-%m-%d").to_string()
}

/// The Sunday that ends the week `day` is in.
pub fn end_of_week(day: NaiveDate) -> NaiveDate {
    day + chrono::Duration::days(i64::from(6 - day.weekday().num_days_from_monday()))
}

/// How a due day stands against today.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Urgency {
    Overdue,
    Today,
    /// Later this week.
    ThisWeek,
    Later,
}

impl Urgency {
    /// None for a cell that does not hold a day.
    pub fn of(due: &str, today: NaiveDate) -> Option<Urgency> {
        let day = NaiveDate::parse_from_str(due, "%Y-%m-%d").ok()?;
        Some(if day < today {
            Urgency::Overdue
        } else if day == today {
            Urgency::Today
        } else if day <= end_of_week(today) {
            Urgency::ThisWeek
        } else {
            Urgency::Later
        })
    }

    pub fn tone(self) -> Tone {
        match self {
            Urgency::Overdue => Tone::Bad,
            Urgency::Today | Urgency::ThisWeek => Tone::Warn,
            Urgency::Later => Tone::Neutral,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Urgency::Overdue => "Overdue",
            Urgency::Today => "Today",
            Urgency::ThisWeek => "Later this week",
            Urgency::Later => "Later",
        }
    }
}

/// A follow-up that has come round: a line whose follow-up date is this
/// week or earlier.
#[derive(Clone, Debug, PartialEq)]
pub struct FollowUp {
    pub sheet: Sheet,
    pub id: u64,
    pub due: String,
    pub urgency: Urgency,
    /// What the line is called: the contact's name, the action taken.
    pub title: String,
    /// What to do: the next step, the follow-up.
    pub what: String,
    /// For an action, the contact it was with.
    pub contact: String,
}

/// The week a day falls in, as the Week column writes it: `W46`.
pub fn week_label(day: NaiveDate) -> String {
    format!("W{:02}", day.iso_week().week())
}

/// One line of a table.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Row {
    pub id: u64,
    #[serde(default)]
    pub cells: BTreeMap<String, String>,
}

impl Row {
    /// A cell's text; empty for one never filled in.
    pub fn get(&self, key: &str) -> &str {
        self.cells.get(key).map_or("", String::as_str)
    }
}

/// Everything the app keeps: the five tables.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Cockpit {
    #[serde(default)]
    pub goals: Vec<Row>,
    #[serde(default)]
    pub actions: Vec<Row>,
    #[serde(default)]
    pub signals: Vec<Row>,
    #[serde(default)]
    pub questions: Vec<Row>,
    /// Missing from a file written before contacts were kept: it opens
    /// with none.
    #[serde(default)]
    pub contacts: Vec<Row>,
}

impl Cockpit {
    pub fn rows(&self, sheet: Sheet) -> &[Row] {
        match sheet {
            Sheet::Goals => &self.goals,
            Sheet::Actions => &self.actions,
            Sheet::Signals => &self.signals,
            Sheet::Questions => &self.questions,
            Sheet::Contacts => &self.contacts,
        }
    }

    fn rows_mut(&mut self, sheet: Sheet) -> &mut Vec<Row> {
        match sheet {
            Sheet::Goals => &mut self.goals,
            Sheet::Actions => &mut self.actions,
            Sheet::Signals => &mut self.signals,
            Sheet::Questions => &mut self.questions,
            Sheet::Contacts => &mut self.contacts,
        }
    }

    pub fn row(&self, sheet: Sheet, id: u64) -> Option<&Row> {
        self.rows(sheet).iter().find(|r| r.id == id)
    }

    /// An id no line of any table has.
    fn next_id(&self) -> u64 {
        Sheet::ALL
            .iter()
            .flat_map(|s| self.rows(*s))
            .map(|r| r.id)
            .max()
            .map_or(1, |max| max + 1)
    }

    /// Adds a line at the end of a table, as a new row goes under the last
    /// one in a spreadsheet: today's date and week, and the first entry of
    /// each drop-down list. Returns its id.
    pub fn add(&mut self, sheet: Sheet, today: NaiveDate) -> u64 {
        let cells = sheet
            .columns()
            .iter()
            .filter_map(|c| {
                let value = match c.kind {
                    Kind::Date => iso(today),
                    Kind::Week => week_label(today),
                    Kind::Choice(options) => options.first()?.label.to_string(),
                    Kind::Text | Kind::Long | Kind::Due | Kind::Contact => return None,
                };
                Some((c.key.to_string(), value))
            })
            .collect();
        self.push(sheet, cells)
    }

    /// Adds a line with these cells, given as `(column key, text)`.
    pub fn add_with(&mut self, sheet: Sheet, cells: &[(&str, &str)]) -> u64 {
        let cells = cells
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.push(sheet, cells)
    }

    fn push(&mut self, sheet: Sheet, cells: BTreeMap<String, String>) -> u64 {
        let id = self.next_id();
        self.rows_mut(sheet).push(Row { id, cells });
        id
    }

    /// Copies a line, right under the one it copies. Returns the copy's id.
    pub fn duplicate(&mut self, sheet: Sheet, id: u64) -> Option<u64> {
        let new_id = self.next_id();
        let rows = self.rows_mut(sheet);
        let at = rows.iter().position(|r| r.id == id)?;
        let mut copy = rows[at].clone();
        copy.id = new_id;
        rows.insert(at + 1, copy);
        Some(new_id)
    }

    /// Deletes a line. A contact deleted leaves the actions done with them,
    /// no longer tied to anyone.
    pub fn remove(&mut self, sheet: Sheet, id: u64) {
        self.rows_mut(sheet).retain(|r| r.id != id);
        if sheet == Sheet::Contacts {
            let gone = id.to_string();
            for row in &mut self.actions {
                if row.get("contact") == gone {
                    row.cells.remove("contact");
                }
            }
        }
    }

    /// Writes one cell.
    pub fn set(&mut self, sheet: Sheet, id: u64, key: &str, value: String) {
        if let Some(row) = self.rows_mut(sheet).iter_mut().find(|r| r.id == id) {
            row.cells.insert(key.to_string(), value);
        }
    }

    /// The contact a [`Kind::Contact`] cell points to.
    pub fn contact(&self, cell: &str) -> Option<&Row> {
        let id = cell.parse().ok()?;
        self.row(Sheet::Contacts, id)
    }

    /// The contact whose name is `name`, whatever the case.
    pub fn contact_named(&self, name: &str) -> Option<&Row> {
        let name = name.trim().to_lowercase();
        if name.is_empty() {
            return None;
        }
        self.contacts
            .iter()
            .find(|r| r.get("name").trim().to_lowercase() == name)
    }

    /// A cell as it reads: a contact by their name, anything else as kept.
    pub fn display(&self, column: &Column, row: &Row) -> String {
        let value = row.get(column.key);
        match column.kind {
            Kind::Contact => self
                .contact(value)
                .map_or_else(String::new, |c| c.get("name").to_string()),
            _ => value.to_string(),
        }
    }

    /// The actions done with a contact, the latest first.
    pub fn history(&self, contact: u64) -> Vec<&Row> {
        let id = contact.to_string();
        let mut rows: Vec<&Row> = self
            .actions
            .iter()
            .filter(|r| r.get("contact") == id)
            .collect();
        rows.sort_by(|a, b| b.get("date").cmp(a.get("date")));
        rows
    }

    /// Every follow-up due by the end of `today`'s week, the oldest first:
    /// the contacts to get back to and the actions to follow up.
    pub fn follow_ups(&self, today: NaiveDate) -> Vec<FollowUp> {
        let mut due: Vec<FollowUp> = Sheet::ALL
            .into_iter()
            .filter_map(|sheet| Some((sheet, sheet.follow_up()?)))
            .flat_map(|(sheet, (date_key, what_key))| {
                self.rows(sheet).iter().filter_map(move |row| {
                    let date = row.get(date_key);
                    let urgency = Urgency::of(date, today)?;
                    if urgency == Urgency::Later {
                        return None;
                    }
                    Some(FollowUp {
                        sheet,
                        id: row.id,
                        due: date.to_string(),
                        urgency,
                        title: row.get(sheet.title_key()).to_string(),
                        what: row.get(what_key).to_string(),
                        contact: self
                            .contact(row.get("contact"))
                            .map_or_else(String::new, |c| c.get("name").to_string()),
                    })
                })
            })
            .collect();
        due.sort_by(|a, b| a.due.cmp(&b.due));
        due
    }

    /// Moves a line's follow-up date `days` later than it was, or than
    /// today when it has none.
    pub fn postpone(&mut self, sheet: Sheet, id: u64, days: i64, today: NaiveDate) {
        let Some((key, _)) = sheet.follow_up() else {
            return;
        };
        let from = self
            .row(sheet, id)
            .and_then(|r| NaiveDate::parse_from_str(r.get(key), "%Y-%m-%d").ok())
            .unwrap_or(today)
            .max(today);
        self.set(sheet, id, key, iso(from + chrono::Duration::days(days)));
    }

    /// How many lines of a table have `label` in the column `key`.
    pub fn count(&self, sheet: Sheet, key: &str, label: &str) -> usize {
        self.rows(sheet)
            .iter()
            .filter(|r| r.get(key) == label)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 11).unwrap()
    }

    #[test]
    fn a_new_line_starts_with_today_and_the_first_choices() {
        let mut cockpit = Cockpit::default();
        let id = cockpit.add(Sheet::Goals, day());
        let row = cockpit.row(Sheet::Goals, id).unwrap();
        assert_eq!(row.get("week"), "W46");
        assert_eq!(row.get("priority"), "1");
        assert_eq!(row.get("status"), "To do");
        assert_eq!(row.get("goal"), "");

        let id = cockpit.add(Sheet::Signals, day());
        let row = cockpit.row(Sheet::Signals, id).unwrap();
        assert_eq!(row.get("date"), "2026-11-11");
        assert_eq!(row.get("importance"), "Low");
    }

    #[test]
    fn ids_are_never_shared_between_tables() {
        let mut cockpit = Cockpit::default();
        let a = cockpit.add(Sheet::Goals, day());
        let b = cockpit.add(Sheet::Questions, day());
        let c = cockpit.duplicate(Sheet::Goals, a).unwrap();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    #[test]
    fn a_copy_sits_under_its_original() {
        let mut cockpit = Cockpit::default();
        let first = cockpit.add_with(Sheet::Actions, &[("action", "Call")]);
        let last = cockpit.add_with(Sheet::Actions, &[("action", "Test")]);
        let copy = cockpit.duplicate(Sheet::Actions, first).unwrap();
        let order: Vec<u64> = cockpit.actions.iter().map(|r| r.id).collect();
        assert_eq!(order, vec![first, copy, last]);
        assert_eq!(
            cockpit.row(Sheet::Actions, copy).unwrap().get("action"),
            "Call"
        );
    }

    #[test]
    fn set_remove_and_count() {
        let mut cockpit = Cockpit::default();
        let a = cockpit.add(Sheet::Goals, day());
        let b = cockpit.add(Sheet::Goals, day());
        cockpit.set(Sheet::Goals, a, "status", "Done".into());
        assert_eq!(cockpit.count(Sheet::Goals, "status", "Done"), 1);
        assert_eq!(cockpit.count(Sheet::Goals, "status", "To do"), 1);
        cockpit.remove(Sheet::Goals, b);
        assert_eq!(cockpit.goals.len(), 1);
        assert_eq!(cockpit.count(Sheet::Goals, "status", "To do"), 0);
    }

    #[test]
    fn every_table_names_columns_it_has() {
        for sheet in Sheet::ALL {
            let keys: Vec<&str> = sheet.columns().iter().map(|c| c.key).collect();
            assert!(keys.contains(&sheet.title_key()), "{sheet:?}");
            assert!(
                matches!(sheet.filter_column().kind, Kind::Choice(_)),
                "{sheet:?}"
            );
            let mut unique = keys.clone();
            unique.sort_unstable();
            unique.dedup();
            assert_eq!(unique.len(), keys.len(), "{sheet:?}");
        }
    }

    #[test]
    fn dates_read_day_first() {
        assert_eq!(show_date("2026-11-09"), "9 Nov 2026");
        assert_eq!(show_date("Monday"), "Monday");
        assert_eq!(show_date(""), "");
    }

    #[test]
    fn a_file_missing_a_table_still_opens() {
        let cockpit: Cockpit =
            serde_json::from_str(r#"{"goals":[{"id":4,"cells":{"goal":"x"}}]}"#).unwrap();
        assert_eq!(cockpit.goals[0].get("goal"), "x");
        assert!(cockpit.questions.is_empty());
        assert!(cockpit.contacts.is_empty());
    }

    #[test]
    fn a_new_contact_is_a_lead_with_no_follow_up() {
        let mut cockpit = Cockpit::default();
        let id = cockpit.add(Sheet::Contacts, day());
        let row = cockpit.row(Sheet::Contacts, id).unwrap();
        assert_eq!(row.get("stage"), "Lead");
        assert_eq!(row.get("follow_up"), "");
        let id = cockpit.add(Sheet::Actions, day());
        let row = cockpit.row(Sheet::Actions, id).unwrap();
        assert_eq!(row.get("contact"), "");
        assert_eq!(row.get("follow_up_on"), "");
    }

    #[test]
    fn an_action_reads_its_contact_by_name_and_shows_in_their_history() {
        let mut cockpit = Cockpit::default();
        let lea = cockpit.add_with(Sheet::Contacts, &[("name", "Léa")]);
        let old = cockpit.add_with(
            Sheet::Actions,
            &[("date", "2026-11-02"), ("contact", &lea.to_string())],
        );
        let new = cockpit.add_with(
            Sheet::Actions,
            &[("date", "2026-11-10"), ("contact", &lea.to_string())],
        );
        cockpit.add_with(Sheet::Actions, &[("date", "2026-11-11")]);
        let column = Sheet::Actions
            .columns()
            .iter()
            .find(|c| c.key == "contact")
            .unwrap();
        let row = cockpit.row(Sheet::Actions, old).unwrap();
        assert_eq!(cockpit.display(column, row), "Léa");
        let ids: Vec<u64> = cockpit.history(lea).iter().map(|r| r.id).collect();
        assert_eq!(ids, vec![new, old]);
        assert_eq!(cockpit.contact_named(" léa ").unwrap().id, lea);
    }

    #[test]
    fn deleting_a_contact_unties_their_actions() {
        let mut cockpit = Cockpit::default();
        let lea = cockpit.add_with(Sheet::Contacts, &[("name", "Léa")]);
        let action = cockpit.add_with(Sheet::Actions, &[("contact", &lea.to_string())]);
        cockpit.remove(Sheet::Contacts, lea);
        assert_eq!(
            cockpit.row(Sheet::Actions, action).unwrap().get("contact"),
            ""
        );
        assert!(cockpit.history(lea).is_empty());
    }

    #[test]
    fn follow_ups_are_those_due_by_the_end_of_the_week() {
        let mut cockpit = Cockpit::default();
        let late = cockpit.add_with(
            Sheet::Contacts,
            &[("name", "Late"), ("follow_up", "2026-11-09")],
        );
        cockpit.add_with(
            Sheet::Contacts,
            &[("name", "Next week"), ("follow_up", "2026-11-16")],
        );
        cockpit.add_with(Sheet::Contacts, &[("name", "None")]);
        let sunday = cockpit.add_with(
            Sheet::Actions,
            &[("action", "Post"), ("follow_up_on", "2026-11-15")],
        );
        let today = cockpit.add_with(
            Sheet::Actions,
            &[("action", "Call"), ("follow_up_on", "2026-11-11")],
        );
        let due = cockpit.follow_ups(day());
        let order: Vec<(u64, Urgency)> = due.iter().map(|f| (f.id, f.urgency)).collect();
        assert_eq!(
            order,
            vec![
                (late, Urgency::Overdue),
                (today, Urgency::Today),
                (sunday, Urgency::ThisWeek),
            ]
        );
    }

    #[test]
    fn postponing_counts_from_today_for_a_late_follow_up() {
        let mut cockpit = Cockpit::default();
        let id = cockpit.add_with(Sheet::Contacts, &[("follow_up", "2026-11-02")]);
        cockpit.postpone(Sheet::Contacts, id, 7, day());
        assert_eq!(
            cockpit.row(Sheet::Contacts, id).unwrap().get("follow_up"),
            "2026-11-18"
        );
        cockpit.postpone(Sheet::Contacts, id, 7, day());
        assert_eq!(
            cockpit.row(Sheet::Contacts, id).unwrap().get("follow_up"),
            "2026-11-25"
        );
    }

    #[test]
    fn the_week_ends_on_sunday() {
        assert_eq!(
            end_of_week(day()),
            NaiveDate::from_ymd_opt(2026, 11, 15).unwrap()
        );
        let sunday = NaiveDate::from_ymd_opt(2026, 11, 15).unwrap();
        assert_eq!(end_of_week(sunday), sunday);
    }
}
