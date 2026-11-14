//! The cockpit: four tables, one per weekly question, and the contacts the
//! work is done with.
//!
//! Each table is described once, as a list of columns, and everything else
//! (the grid, the editor on the right, the filter, a new line's defaults)
//! is read from that description. A line keeps its cells by column key, so
//! the saved file stays readable and a column added later reads as empty in
//! lines written before it.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{Datelike, NaiveDate};
use desktop_kit::library::Item;
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
    /// A video or post in the media library the other apps share, kept as
    /// its id there (see `crate::media`).
    Media,
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
    col("media", "Video or post", Kind::Media, Width::Wide, false),
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
    col("media", "Video or post", Kind::Media, Width::Wide, false),
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

    /// Its name for a tool call or a file: `contacts`.
    pub fn id(self) -> &'static str {
        match self {
            Sheet::Goals => "goals",
            Sheet::Actions => "actions",
            Sheet::Signals => "signals",
            Sheet::Questions => "questions",
            Sheet::Contacts => "contacts",
        }
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

/// The Monday that starts the week `day` is in.
pub fn start_of_week(day: NaiveDate) -> NaiveDate {
    end_of_week(day) - chrono::Duration::days(6)
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
    /// The posts from the media library already logged in Field progress.
    #[serde(default, skip_serializing_if = "PostLog::is_empty")]
    pub posts: PostLog,
}

/// Which posts that went out are already in Field progress, so each is
/// logged once, and one whose line was deleted stays deleted.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PostLog {
    /// The Monday of the week MiniCRM first read the library: what went out
    /// before is history, and isn't logged.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub since: String,
    /// Their ids in the library.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub logged: BTreeSet<String>,
}

impl PostLog {
    fn is_empty(&self) -> bool {
        self.since.is_empty() && self.logged.is_empty()
    }
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
                    Kind::Text | Kind::Long | Kind::Due | Kind::Contact | Kind::Media => {
                        return None
                    }
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

    /// A cell as it reads: a contact by their name, a video or post by its
    /// title, anything else as kept.
    pub fn display(&self, column: &Column, row: &Row) -> String {
        let value = row.get(column.key);
        match column.kind {
            Kind::Contact => self
                .contact(value)
                .map_or_else(String::new, |c| c.get("name").to_string()),
            Kind::Media => crate::media::title(value),
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
        self.due_by(today, end_of_week(today))
    }

    /// Every follow-up due on `until` or before, the oldest first.
    pub fn due_by(&self, today: NaiveDate, until: NaiveDate) -> Vec<FollowUp> {
        let until = iso(until);
        let until = until.as_str();
        let mut due: Vec<FollowUp> = Sheet::ALL
            .into_iter()
            .filter_map(|sheet| Some((sheet, sheet.follow_up()?)))
            .flat_map(|(sheet, (date_key, what_key))| {
                self.rows(sheet).iter().filter_map(move |row| {
                    let date = row.get(date_key);
                    let urgency = Urgency::of(date, today)?;
                    if date > until {
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

    /// Logs in Field progress every post from the media library that went
    /// out since MiniCRM first read it and isn't logged yet: on the day it
    /// went out, as a Post, with where it went, pointing at the post. The
    /// result and what was learned are the person's to write. Returns how
    /// many were logged.
    pub fn log_posts(&mut self, items: &[Item], today: NaiveDate) -> usize {
        if self.posts.since.is_empty() {
            self.posts.since = iso(start_of_week(today));
        }
        let today = iso(today);
        let mut out: Vec<&Item> = items
            .iter()
            .filter(|i| crate::media::is_post(i) && i.status.is_out())
            .filter(|i| i.day() >= self.posts.since.as_str() && i.day() <= today.as_str())
            .filter(|i| !self.posts.logged.contains(&i.id))
            .collect();
        out.sort_by(|a, b| a.date.cmp(&b.date));
        for item in &out {
            self.posts.logged.insert(item.id.clone());
            if self.actions.iter().any(|r| r.get("media") == item.id) {
                continue;
            }
            let place = crate::media::place(item);
            self.add_with(
                Sheet::Actions,
                &[
                    ("date", item.day()),
                    ("action", item.title.trim()),
                    ("type", "Post"),
                    ("media", &item.id),
                    ("target", &place),
                ],
            );
        }
        out.len()
    }

    /// Points every video-or-post cell that named a planned idea at what was
    /// made for it, once PolishCast, TrackCut or PublishKit took the idea
    /// over: the idea is gone from the library then, and the take or post
    /// keeps its id. Returns how many cells moved.
    pub fn follow_plans(&mut self, items: &[Item]) -> usize {
        let mut moved = 0;
        for sheet in Sheet::ALL {
            let keys: Vec<&str> = sheet
                .columns()
                .iter()
                .filter(|c| c.kind == Kind::Media)
                .map(|c| c.key)
                .collect();
            for row in self.rows_mut(sheet) {
                for key in &keys {
                    let id = row.get(key);
                    if id.is_empty() || items.iter().any(|i| i.id == id) {
                        continue;
                    }
                    if let Some(made) = items.iter().find(|i| i.plans.iter().any(|p| p == id)) {
                        row.cells.insert(key.to_string(), made.id.clone());
                        moved += 1;
                    }
                }
            }
        }
        moved
    }

    /// Keeps up with the media library: cells follow planned ideas to what
    /// was made for them, and posts that went out are logged.
    pub fn follow_library(&mut self, items: &[Item], today: NaiveDate) {
        self.follow_plans(items);
        self.log_posts(items, today);
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
    use desktop_kit::library::{Kind as MediaKind, Status};

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

    fn post(id: &str, kind: MediaKind, network: Option<&str>, status: Status, date: &str) -> Item {
        let mut item = Item::new(
            id,
            "publishkit",
            kind,
            id.trim_start_matches("publishkit:"),
            status,
        );
        item.network = network.map(String::from);
        item.date = date.into();
        item
    }

    fn library() -> Vec<Item> {
        vec![
            post(
                "publishkit:linkedin",
                MediaKind::Social,
                Some("linkedin"),
                Status::Posted,
                "2026-11-10T09:00",
            ),
            post(
                "publishkit:blog",
                MediaKind::Blog,
                None,
                Status::Published,
                "2026-11-09",
            ),
            post(
                "publishkit:last-month",
                MediaKind::Blog,
                None,
                Status::Published,
                "2026-10-01",
            ),
            post(
                "publishkit:tomorrow",
                MediaKind::Social,
                Some("x"),
                Status::Queued,
                "2026-11-12",
            ),
            post(
                "polishcast:demo",
                MediaKind::Video,
                None,
                Status::Posted,
                "2026-11-10",
            ),
        ]
    }

    #[test]
    fn posts_that_went_out_this_week_are_logged_once_the_oldest_first() {
        let mut cockpit = Cockpit::default();
        assert_eq!(cockpit.log_posts(&library(), day()), 2);
        assert_eq!(cockpit.posts.since, "2026-11-09");
        let logged: Vec<_> = cockpit
            .actions
            .iter()
            .map(|r| {
                (
                    r.get("date"),
                    r.get("action"),
                    r.get("type"),
                    r.get("target"),
                    r.get("media"),
                )
            })
            .collect();
        assert_eq!(
            logged,
            [
                ("2026-11-09", "blog", "Post", "Blog", "publishkit:blog"),
                (
                    "2026-11-10",
                    "linkedin",
                    "Post",
                    "LinkedIn",
                    "publishkit:linkedin"
                ),
            ]
        );
        assert_eq!(cockpit.log_posts(&library(), day()), 0);
        assert_eq!(cockpit.actions.len(), 2);
    }

    #[test]
    fn a_logged_post_whose_line_was_deleted_stays_deleted() {
        let mut cockpit = Cockpit::default();
        cockpit.log_posts(&library(), day());
        let first = cockpit.actions[0].id;
        cockpit.remove(Sheet::Actions, first);
        assert_eq!(cockpit.log_posts(&library(), day()), 0);
        assert_eq!(cockpit.actions.len(), 1);
    }

    #[test]
    fn a_queued_post_is_logged_once_it_went_out_and_its_day_came() {
        let mut cockpit = Cockpit::default();
        cockpit.log_posts(&library(), day());
        let mut items = library();
        items[3].status = Status::Posted;
        assert_eq!(cockpit.log_posts(&items, day()), 0);
        let thursday = day() + chrono::Duration::days(1);
        assert_eq!(cockpit.log_posts(&items, thursday), 1);
        assert_eq!(cockpit.actions[2].get("target"), "X");
    }

    #[test]
    fn a_post_an_action_already_names_is_not_logged_twice() {
        let mut cockpit = Cockpit::default();
        let mine = cockpit.add_with(
            Sheet::Actions,
            &[("action", "Launch post"), ("media", "publishkit:linkedin")],
        );
        assert_eq!(cockpit.log_posts(&library(), day()), 2);
        assert_eq!(cockpit.actions.len(), 2);
        assert_eq!(cockpit.actions[0].id, mine);
    }

    #[test]
    fn a_cockpit_without_posts_logged_saves_as_before() {
        let saved = serde_json::to_string(&Cockpit::default()).unwrap();
        assert!(!saved.contains("posts"), "{saved}");
        let old: Cockpit = serde_json::from_str(r#"{"actions":[]}"#).unwrap();
        assert_eq!(old.posts, PostLog::default());
        assert_eq!(
            start_of_week(day()),
            NaiveDate::from_ymd_opt(2026, 11, 9).unwrap()
        );
    }

    #[test]
    fn a_goal_s_planned_video_follows_it_once_it_is_recorded() {
        let mut cockpit = Cockpit::default();
        let goal = cockpit.add_with(
            Sheet::Goals,
            &[("goal", "Publish the demo"), ("media", "plan:1-demo")],
        );
        let planned = post(
            "plan:1-demo",
            MediaKind::Video,
            None,
            Status::Planned,
            "2026-11-10",
        );
        assert_eq!(cockpit.follow_plans(&[planned]), 0, "still planned");

        let mut take = post(
            "polishcast:demo",
            MediaKind::Video,
            None,
            Status::Ready,
            "2026-11-10",
        );
        take.plans = vec!["plan:1-demo".into()];
        assert_eq!(cockpit.follow_plans(&[take.clone()]), 1);
        assert_eq!(
            cockpit.row(Sheet::Goals, goal).unwrap().get("media"),
            "polishcast:demo"
        );
        assert_eq!(cockpit.follow_plans(&[take]), 0);
        // Gone with nothing made for it: left as it is.
        assert_eq!(cockpit.follow_plans(&[]), 0);
        assert_eq!(
            cockpit.row(Sheet::Goals, goal).unwrap().get("media"),
            "polishcast:demo"
        );
    }
}
