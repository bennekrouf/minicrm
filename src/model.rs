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
    choice("Reply received", Tone::Neutral),
    choice("Call", Tone::Neutral),
    choice("Interview", Tone::Neutral),
    choice("Test", Tone::Neutral),
    choice("Post", Tone::Neutral),
    choice("Research", Tone::Neutral),
    choice("Other", Tone::Neutral),
];

/// Whether an action was done or is only planned. A line without one, from
/// before actions had it, reads as done.
pub const ACTION_STATUS: &[Choice] = &[
    choice("Done", Tone::Neutral),
    choice("Planned", Tone::Soft),
    choice("Cancelled", Tone::Bad),
];

/// An action that was done: its own status, or none from before there was
/// one. Only those move a contact along or set their follow-up.
pub fn is_done(row: &Row) -> bool {
    matches!(row.get("status"), "" | "Done")
}

pub fn is_planned(row: &Row) -> bool {
    row.get("status") == "Planned"
}

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
    col(
        "status",
        "Done or planned",
        Kind::Choice(ACTION_STATUS),
        Width::Narrow,
        false,
    ),
    col("contact", "Contact", Kind::Contact, Width::Medium, true),
    col("media", "Video or post", Kind::Media, Width::Wide, false),
    col("target", "Person / target", Kind::Text, Width::Wide, false),
    col("notes", "Message or notes", Kind::Long, Width::Wide, false),
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

/// The cells of an action that say something about the contact it was
/// with: writing one brings the contact up to date.
const ACTION_KEYS_FOR_CONTACT: &[&str] = &[
    "action",
    "contact",
    "type",
    "status",
    "date",
    "follow_up",
    "follow_up_on",
];

/// The cell of a planned action that ties it to the other steps of the
/// sequence it was planned in: the first step's id. Not a column.
pub const SEQUENCE: &str = "sequence";

/// A stage that says the person answered or decided: past it, the steps
/// still planned in a sequence for them have no reason to go out.
fn ends_sequences(stage: &str) -> bool {
    let rank = |label: &str| STAGES.iter().position(|s| s.label == label);
    rank(stage) > rank("Contacted")
}

/// One step of a sequence to plan for a contact.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub date: NaiveDate,
    /// What will be done: the message's subject.
    pub action: String,
    /// The message itself, or notes for it.
    pub notes: String,
    /// One of the action types; Message sent when empty.
    pub kind: String,
}

/// The stage a contact moves to after an action of this type, if it moves
/// them forward. Only the early stages move on their own: Lead, Contacted
/// and In conversation. A proposal, a client and Not now are the person's
/// own decisions, and an action never undoes them.
fn forward_stage(stage: &str, action_type: &str) -> Option<&'static str> {
    let to = match action_type {
        "Message sent" => "Contacted",
        "Reply received" | "Call" | "Interview" => "In conversation",
        _ => return None,
    };
    let rank = |label: &str| STAGES.iter().position(|s| s.label == label);
    let from = if stage.is_empty() {
        Some(0)
    } else {
        rank(stage)
    }?;
    (from <= rank("In conversation")? && rank(to)? > from).then_some(to)
}

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

/// Who, then what to do next with them and when: what a glance at the
/// table is for. Who they are and how to reach them come after.
const CONTACTS: &[Column] = &[
    col("name", "Name", Kind::Text, Width::Wide, true),
    col("next", "Next step", Kind::Long, Width::Wide, true),
    col(
        "follow_up",
        "Next follow-up",
        Kind::Due,
        Width::Narrow,
        true,
    ),
    col("stage", "Stage", Kind::Choice(STAGES), Width::Medium, true),
    col("about", "Who they are", Kind::Text, Width::Wide, false),
    col(
        "source",
        "Where I found them",
        Kind::Text,
        Width::Medium,
        false,
    ),
    col("link", "Profile or website", Kind::Text, Width::Wide, false),
    col("email", "Email", Kind::Text, Width::Medium, false),
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
    /// An action still to do, listed on its own date.
    pub planned: bool,
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
    /// Put away: kept, with its history, but out of its table, the counts
    /// and Follow-ups until it is restored.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub archived: bool,
    /// A contact kept in the network: someone known, such as a LinkedIn
    /// connection, who is out of Contacts, the counts and Follow-ups, and
    /// shows only in a search until added to Contacts.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub network: bool,
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
        self.rows_mut(sheet).push(Row {
            id,
            cells,
            archived: false,
            network: false,
        });
        id
    }

    /// The lines of a table still in use, archived ones and the network
    /// left out.
    pub fn active(&self, sheet: Sheet) -> impl Iterator<Item = &Row> {
        self.rows(sheet)
            .iter()
            .filter(|r| !r.archived && !r.network)
    }

    /// How many lines of a table are archived.
    pub fn archived(&self, sheet: Sheet) -> usize {
        self.rows(sheet)
            .iter()
            .filter(|r| r.archived && !r.network)
            .count()
    }

    /// How many contacts wait in the network.
    pub fn network(&self) -> usize {
        self.contacts.iter().filter(|r| r.network).count()
    }

    /// Puts a contact in the network, or adds them to Contacts when
    /// `network` is false. False when there is no such contact.
    pub fn set_network(&mut self, id: u64, network: bool) -> bool {
        match self.contacts.iter_mut().find(|r| r.id == id) {
            Some(row) => {
                row.network = network;
                true
            }
            None => false,
        }
    }

    /// Archives a line, or brings it back when `archived` is false. False
    /// when the table has no such line.
    pub fn set_archived(&mut self, sheet: Sheet, id: u64, archived: bool) -> bool {
        match self.rows_mut(sheet).iter_mut().find(|r| r.id == id) {
            Some(row) => {
                row.archived = archived;
                true
            }
            None => false,
        }
    }

    /// Copies a line, right under the one it copies. Returns the copy's id.
    pub fn duplicate(&mut self, sheet: Sheet, id: u64) -> Option<u64> {
        let new_id = self.next_id();
        let rows = self.rows_mut(sheet);
        let at = rows.iter().position(|r| r.id == id)?;
        let mut copy = rows[at].clone();
        copy.id = new_id;
        // A copy is made to be worked on: it goes in the table.
        copy.archived = false;
        copy.network = false;
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

    /// Writes one cell. A cell of an action logged with a contact also
    /// brings that contact up to date: see [`Cockpit::update_contact`].
    pub fn set(&mut self, sheet: Sheet, id: u64, key: &str, value: String) {
        if let Some(row) = self.rows_mut(sheet).iter_mut().find(|r| r.id == id) {
            row.cells.insert(key.to_string(), value);
        }
        if sheet == Sheet::Actions && ACTION_KEYS_FOR_CONTACT.contains(&key) {
            self.update_contact(id);
        }
        if sheet == Sheet::Contacts && key == "stage" {
            let stage = self.row(sheet, id).map(|r| r.get("stage").to_string());
            if stage.is_some_and(|s| ends_sequences(&s)) {
                self.cancel_sequences(id);
            }
        }
    }

    /// Marks an action done now: a planned one takes today's date if it was
    /// planned for later, and moves its contact along as any action does.
    pub fn complete(&mut self, id: u64, today: NaiveDate) {
        let Some(row) = self.row(Sheet::Actions, id) else {
            return;
        };
        if row.get("date") > iso(today).as_str() {
            self.set(Sheet::Actions, id, "date", iso(today));
        }
        self.set(Sheet::Actions, id, "status", "Done".into());
    }

    /// Plans a sequence of actions with a contact, one per step, in the
    /// order of their dates. Each step is Planned: it shows in Follow-ups
    /// on its day, and moves the contact along only once marked Done, when
    /// the next step's date becomes their follow-up. Returns the steps' ids.
    pub fn plan_sequence(&mut self, contact: u64, steps: &[Step]) -> Vec<u64> {
        let mut steps = steps.to_vec();
        steps.sort_by_key(|s| s.date);
        let contact = contact.to_string();
        let mut ids: Vec<u64> = Vec::new();
        for (n, step) in steps.iter().enumerate() {
            let next = steps.get(n + 1);
            let kind = if step.kind.trim().is_empty() {
                "Message sent"
            } else {
                step.kind.as_str()
            };
            let first = ids.first().map(u64::to_string);
            let date = iso(step.date);
            let next_date = next.map(|s| iso(s.date)).unwrap_or_default();
            let mut cells = vec![
                ("date", date.as_str()),
                ("action", step.action.trim()),
                ("type", kind),
                ("status", "Planned"),
                ("contact", contact.as_str()),
                ("notes", step.notes.trim()),
                ("follow_up_on", next_date.as_str()),
                ("follow_up", next.map_or("", |s| s.action.trim())),
            ];
            cells.retain(|(_, v)| !v.is_empty());
            let id = self.add_with(Sheet::Actions, &cells);
            let first = first.unwrap_or_else(|| id.to_string());
            self.set_cell(Sheet::Actions, id, SEQUENCE, first);
            ids.push(id);
        }
        ids
    }

    /// Cancels the steps still planned in a contact's sequences, as a reply
    /// or a decision does. They stay in Field progress, marked Cancelled.
    /// Returns how many.
    pub fn cancel_sequences(&mut self, contact: u64) -> usize {
        let contact = contact.to_string();
        let mut n = 0;
        for row in &mut self.actions {
            if row.get("contact") == contact && is_planned(row) && !row.get(SEQUENCE).is_empty() {
                row.cells.insert("status".into(), "Cancelled".into());
                n += 1;
            }
        }
        n
    }

    /// Writes a cell with nothing following from it.
    fn set_cell(&mut self, sheet: Sheet, id: u64, key: &str, value: String) {
        if let Some(row) = self.rows_mut(sheet).iter_mut().find(|r| r.id == id) {
            row.cells.insert(key.to_string(), value);
        }
    }

    /// A contact that says nothing about anyone: no name, no email, nothing
    /// but the stage a new line starts with, and nothing done with them.
    pub fn is_blank_contact(&self, row: &Row) -> bool {
        let id = row.id.to_string();
        row.cells
            .iter()
            .all(|(k, v)| k == "stage" || v.trim().is_empty())
            && !self.actions.iter().any(|a| a.get("contact") == id)
    }

    /// Deletes the contacts left blank, such as one added and closed before
    /// anything was typed in it, but `keep`, the one being filled in. True
    /// when one went.
    pub fn drop_blank_contacts(&mut self, keep: Option<u64>) -> bool {
        let blank: Vec<u64> = self
            .contacts
            .iter()
            .filter(|r| Some(r.id) != keep && self.is_blank_contact(r))
            .map(|r| r.id)
            .collect();
        self.contacts.retain(|r| !blank.contains(&r.id));
        !blank.is_empty()
    }

    /// What logging an action with someone says about them, written on
    /// their contact so it doesn't have to be typed twice:
    ///
    /// * the stage moves forward: a message sent makes a lead Contacted, a
    ///   call or an interview puts them In conversation. It never moves
    ///   back, and a proposal sent, a client or someone set to Not now is
    ///   left as it is: those are the person's own calls.
    /// * the follow-up follows the latest action with them: the action's
    ///   own follow-up date (and what to do then) becomes the contact's
    ///   next follow-up and next step. An action with no date leaves the
    ///   contact's follow-up as it was: only the person clears one, from
    ///   Follow-ups or the contact. An older action edited later changes
    ///   nothing.
    ///
    /// An action counts once it says what was done ("Action taken") and
    /// names the contact: a new line already reads "Message sent", and
    /// must not move anyone before it is filled in.
    fn update_contact(&mut self, action: u64) {
        let Some(row) = self.row(Sheet::Actions, action) else {
            return;
        };
        if row.get("action").trim().is_empty() || !is_done(row) {
            return;
        }
        let Some(contact) = self.contact(row.get("contact")) else {
            return;
        };
        let contact_id = contact.id;
        // Something done with someone from the network makes them a contact.
        if contact.network {
            self.set_network(contact_id, false);
        }
        let Some(contact) = self.row(Sheet::Contacts, contact_id) else {
            return;
        };
        let Some(row) = self.row(Sheet::Actions, action) else {
            return;
        };
        let date = row.get("date").to_string();
        let due = row.get("follow_up_on").to_string();
        let what = row.get("follow_up").trim().to_string();

        let stage = forward_stage(contact.get("stage"), row.get("type"));
        let replied = row.get("type") == "Reply received";
        // Planned and cancelled actions were not done: the latest is among
        // those that were.
        let latest = self
            .history(contact_id)
            .into_iter()
            .find(|r| is_done(r))
            .is_some_and(|newest| newest.get("date") <= date.as_str());
        // A reply, or a step that moves them on, ends the sequence it was
        // in: the next step it pointed to will not go out.
        let ended = replied || stage.is_some_and(ends_sequences);
        let in_sequence = !row.get(SEQUENCE).is_empty();
        if replied {
            self.cancel_sequences(contact_id);
        }
        if let Some(stage) = stage {
            self.set(Sheet::Contacts, contact_id, "stage", stage.to_string());
        }
        if latest && !due.is_empty() && !(ended && in_sequence) {
            self.set(Sheet::Contacts, contact_id, "follow_up", due);
            if !what.is_empty() {
                self.set(Sheet::Contacts, contact_id, "next", what);
            }
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

    /// The contact whose email is `email`, whatever the case.
    pub fn contact_with_email(&self, email: &str) -> Option<&Row> {
        let email = email.trim().to_lowercase();
        if email.is_empty() {
            return None;
        }
        self.contacts
            .iter()
            .find(|r| r.get("email").trim().to_lowercase() == email)
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
                self.active(sheet).filter_map(move |row| {
                    let planned = sheet == Sheet::Actions && is_planned(row);
                    if sheet == Sheet::Actions && !planned && !is_done(row) {
                        return None;
                    }
                    let date = if planned {
                        row.get("date")
                    } else {
                        row.get(date_key)
                    };
                    let urgency = Urgency::of(date, today)?;
                    if date > until {
                        return None;
                    }
                    // An action's follow-up its contact took over (see
                    // update_contact) is listed once, under the contact.
                    if sheet == Sheet::Actions
                        && !planned
                        && self
                            .contact(row.get("contact"))
                            .is_some_and(|c| c.get("follow_up") == date)
                    {
                        return None;
                    }
                    // A contact whose follow-up is a step planned for that
                    // day is listed once, as the step: doing it is marking
                    // it done.
                    if sheet == Sheet::Contacts && self.planned_on(row.id, date) {
                        return None;
                    }
                    let what = if planned {
                        format!("Planned: {}", row.get("type"))
                    } else {
                        row.get(what_key).to_string()
                    };
                    Some(FollowUp {
                        sheet,
                        id: row.id,
                        due: date.to_string(),
                        urgency,
                        planned,
                        title: row.get(sheet.title_key()).to_string(),
                        what,
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

    /// Whether a step is planned with this contact on this day.
    fn planned_on(&self, contact: u64, date: &str) -> bool {
        let id = contact.to_string();
        self.active(Sheet::Actions)
            .any(|a| a.get("contact") == id && is_planned(a) && a.get("date") == date)
    }

    /// The cell a line is listed in Follow-ups by: a planned action's own
    /// date, else the table's follow-up date.
    pub fn due_key(&self, sheet: Sheet, id: u64) -> Option<&'static str> {
        let (key, _) = sheet.follow_up()?;
        let planned = sheet == Sheet::Actions && self.row(sheet, id).is_some_and(is_planned);
        Some(if planned { "date" } else { key })
    }

    /// Done with a follow-up: a planned action is marked done, today; any
    /// other line's follow-up date is cleared.
    pub fn follow_up_done(&mut self, sheet: Sheet, id: u64, today: NaiveDate) {
        match self.due_key(sheet, id) {
            Some("date") => self.complete(id, today),
            Some(key) => self.set(sheet, id, key, String::new()),
            None => {}
        }
    }

    /// Moves a line's follow-up date `days` later than it was, or than
    /// today when it has none. A planned action moves to that day.
    pub fn postpone(&mut self, sheet: Sheet, id: u64, days: i64, today: NaiveDate) {
        let Some(key) = self.due_key(sheet, id) else {
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

    /// How many lines of a table in use have `label` in the column `key`.
    pub fn count(&self, sheet: Sheet, key: &str, label: &str) -> usize {
        self.active(sheet).filter(|r| r.get(key) == label).count()
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

    fn log(cockpit: &mut Cockpit, contact: u64, cells: &[(&str, &str)]) -> u64 {
        let id = cockpit.add(Sheet::Actions, day());
        cockpit.set(Sheet::Actions, id, "contact", contact.to_string());
        for (key, value) in cells {
            cockpit.set(Sheet::Actions, id, key, value.to_string());
        }
        id
    }

    fn cell(cockpit: &Cockpit, contact: u64, key: &str) -> String {
        cockpit
            .row(Sheet::Contacts, contact)
            .unwrap()
            .get(key)
            .to_string()
    }

    #[test]
    fn logging_an_action_moves_the_contact_forward_never_back() {
        let mut cockpit = Cockpit::default();
        let lea = cockpit.add(Sheet::Contacts, day());
        // A new line reads "Message sent" before anything is typed in it.
        let id = log(&mut cockpit, lea, &[]);
        assert_eq!(cell(&cockpit, lea, "stage"), "Lead");
        cockpit.set(Sheet::Actions, id, "action", "Intro message".into());
        assert_eq!(cell(&cockpit, lea, "stage"), "Contacted");
        cockpit.set(Sheet::Actions, id, "type", "Call".into());
        assert_eq!(cell(&cockpit, lea, "stage"), "In conversation");
        log(
            &mut cockpit,
            lea,
            &[("action", "Thanks"), ("type", "Message sent")],
        );
        assert_eq!(cell(&cockpit, lea, "stage"), "In conversation");
        log(
            &mut cockpit,
            lea,
            &[("action", "Pricing page"), ("type", "Research")],
        );
        assert_eq!(cell(&cockpit, lea, "stage"), "In conversation");
    }

    #[test]
    fn a_proposal_a_client_or_not_now_stays_as_it_is() {
        for stage in ["Proposal sent", "Client", "Not now"] {
            let mut cockpit = Cockpit::default();
            let id = cockpit.add_with(Sheet::Contacts, &[("name", "Léa"), ("stage", stage)]);
            log(
                &mut cockpit,
                id,
                &[("action", "Call"), ("type", "Interview")],
            );
            assert_eq!(cell(&cockpit, id, "stage"), stage);
        }
    }

    #[test]
    fn the_latest_action_with_a_contact_sets_their_follow_up() {
        let mut cockpit = Cockpit::default();
        let lea = cockpit.add_with(
            Sheet::Contacts,
            &[
                ("name", "Léa"),
                ("follow_up", "2026-11-09"),
                ("next", "Send the offer"),
            ],
        );
        // Getting back to her, with no date for the next time: her late
        // follow-up stays until it is cleared by hand.
        let id = log(&mut cockpit, lea, &[("action", "Sent the offer")]);
        assert_eq!(cell(&cockpit, lea, "follow_up"), "2026-11-09");
        assert_eq!(cell(&cockpit, lea, "next"), "Send the offer");
        // A follow-up on the action becomes hers.
        cockpit.set(Sheet::Actions, id, "follow_up_on", "2026-11-18".into());
        cockpit.set(
            Sheet::Actions,
            id,
            "follow_up",
            "Ask what she thinks".into(),
        );
        assert_eq!(cell(&cockpit, lea, "follow_up"), "2026-11-18");
        assert_eq!(cell(&cockpit, lea, "next"), "Ask what she thinks");
        // Listed once, under her name.
        let until = NaiveDate::from_ymd_opt(2026, 11, 30).unwrap();
        let due: Vec<(Sheet, u64)> = cockpit
            .due_by(day(), until)
            .iter()
            .map(|f| (f.sheet, f.id))
            .collect();
        assert_eq!(due, vec![(Sheet::Contacts, lea)]);
        // An older action, written up later, changes nothing.
        log(
            &mut cockpit,
            lea,
            &[
                ("date", "2026-11-02"),
                ("action", "First call"),
                ("follow_up_on", "2026-11-12"),
            ],
        );
        assert_eq!(cell(&cockpit, lea, "follow_up"), "2026-11-18");
    }

    #[test]
    fn a_follow_up_still_to_come_is_kept() {
        let mut cockpit = Cockpit::default();
        let lea = cockpit.add_with(
            Sheet::Contacts,
            &[("name", "Léa"), ("follow_up", "2026-11-20")],
        );
        log(
            &mut cockpit,
            lea,
            &[("action", "Liked her post"), ("type", "Other")],
        );
        assert_eq!(cell(&cockpit, lea, "follow_up"), "2026-11-20");
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

    #[test]
    fn an_archived_line_is_kept_but_out_of_the_counts_and_follow_ups() {
        let mut cockpit = crate::seed::examples(day());
        let sophie = cockpit.contact_named("Sophie Durand").unwrap().id;
        let due = cockpit.follow_ups(day()).len();
        let stage = cockpit
            .row(Sheet::Contacts, sophie)
            .unwrap()
            .get("stage")
            .to_string();
        let at_stage = cockpit.count(Sheet::Contacts, "stage", &stage);

        assert!(cockpit.set_archived(Sheet::Contacts, sophie, true));
        assert_eq!(cockpit.archived(Sheet::Contacts), 1);
        assert_eq!(cockpit.active(Sheet::Contacts).count(), 4);
        assert_eq!(
            cockpit.count(Sheet::Contacts, "stage", &stage),
            at_stage - 1
        );
        assert!(cockpit
            .follow_ups(day())
            .iter()
            .all(|f| f.title != "Sophie Durand"));
        assert!(cockpit.follow_ups(day()).len() < due);
        // Still found by name, and what was done with her stays.
        assert_eq!(
            cockpit.contact_named("sophie durand").map(|r| r.id),
            Some(sophie)
        );
        assert!(!cockpit.history(sophie).is_empty());

        // A copy goes in the table; the original is brought back as it was.
        let copy = cockpit.duplicate(Sheet::Contacts, sophie).unwrap();
        assert!(!cockpit.row(Sheet::Contacts, copy).unwrap().archived);
        cockpit.remove(Sheet::Contacts, copy);
        assert!(cockpit.set_archived(Sheet::Contacts, sophie, false));
        assert_eq!(cockpit.follow_ups(day()).len(), due);
        assert!(
            !cockpit.set_archived(Sheet::Goals, sophie, true),
            "not a goal"
        );
    }

    #[test]
    fn only_an_archived_line_says_so_in_the_file() {
        let mut cockpit = Cockpit::default();
        let a = cockpit.add_with(Sheet::Goals, &[("goal", "A")]);
        cockpit.add_with(Sheet::Goals, &[("goal", "B")]);
        cockpit.set_archived(Sheet::Goals, a, true);
        let json = serde_json::to_string(&cockpit).unwrap();
        assert_eq!(json.matches("archived").count(), 1, "{json}");
        let back: Cockpit = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cockpit);
    }
    #[test]
    fn a_planned_action_changes_nothing_until_it_is_done() {
        let mut cockpit = Cockpit::default();
        let lea = cockpit.add_with(Sheet::Contacts, &[("name", "Léa")]);
        let id = cockpit.add(Sheet::Actions, day());
        assert_eq!(
            cockpit.row(Sheet::Actions, id).unwrap().get("status"),
            "Done"
        );
        cockpit.set(Sheet::Actions, id, "status", "Planned".into());
        cockpit.set(Sheet::Actions, id, "date", "2026-11-13".into());
        cockpit.set(Sheet::Actions, id, "action", "Email #2".into());
        cockpit.set(Sheet::Actions, id, "contact", lea.to_string());
        cockpit.set(Sheet::Actions, id, "follow_up_on", "2026-11-20".into());
        assert_eq!(cell(&cockpit, lea, "stage"), "");
        assert_eq!(cell(&cockpit, lea, "follow_up"), "");
        // Listed on its own day, as a step to do.
        let due = cockpit.follow_ups(day());
        assert_eq!(due.len(), 1);
        assert!(due[0].planned);
        assert_eq!(due[0].due, "2026-11-13");
        // Done today: it takes today's date and moves her along.
        cockpit.follow_up_done(Sheet::Actions, id, day());
        let row = cockpit.row(Sheet::Actions, id).unwrap();
        assert_eq!(row.get("status"), "Done");
        assert_eq!(row.get("date"), "2026-11-11");
        assert_eq!(cell(&cockpit, lea, "stage"), "Contacted");
        assert_eq!(cell(&cockpit, lea, "follow_up"), "2026-11-20");
    }

    fn steps() -> Vec<Step> {
        [0, 3, 7]
            .iter()
            .enumerate()
            .map(|(n, d)| Step {
                date: day() + chrono::Duration::days(*d),
                action: format!("Email #{}", n + 1),
                notes: String::new(),
                kind: String::new(),
            })
            .collect()
    }

    #[test]
    fn a_sequence_goes_one_step_at_a_time() {
        let mut cockpit = Cockpit::default();
        let lea = cockpit.add_with(Sheet::Contacts, &[("name", "Léa"), ("stage", "Lead")]);
        let ids = cockpit.plan_sequence(lea, &steps());
        assert_eq!(ids.len(), 3);
        assert_eq!(cell(&cockpit, lea, "stage"), "Lead");
        let until = NaiveDate::from_ymd_opt(2026, 11, 30).unwrap();
        let due: Vec<u64> = cockpit.due_by(day(), until).iter().map(|f| f.id).collect();
        assert_eq!(due, ids);

        cockpit.complete(ids[0], day());
        assert_eq!(cell(&cockpit, lea, "stage"), "Contacted");
        assert_eq!(cell(&cockpit, lea, "follow_up"), "2026-11-14");
        assert_eq!(cell(&cockpit, lea, "next"), "Email #2");
        // Her follow-up is the next step: listed once, as the step.
        let due: Vec<(Sheet, u64)> = cockpit
            .due_by(day(), until)
            .iter()
            .map(|f| (f.sheet, f.id))
            .collect();
        assert_eq!(
            due,
            vec![(Sheet::Actions, ids[1]), (Sheet::Actions, ids[2])]
        );
    }

    #[test]
    fn a_reply_or_a_decision_cancels_the_steps_left() {
        let mut cockpit = Cockpit::default();
        let lea = cockpit.add_with(Sheet::Contacts, &[("name", "Léa"), ("stage", "Lead")]);
        let ids = cockpit.plan_sequence(lea, &steps());
        cockpit.complete(ids[0], day());
        log(
            &mut cockpit,
            lea,
            &[("action", "She answered"), ("type", "Reply received")],
        );
        assert_eq!(cell(&cockpit, lea, "stage"), "In conversation");
        let status = |c: &Cockpit, id| c.row(Sheet::Actions, id).unwrap().get("status").to_string();
        assert_eq!(status(&cockpit, ids[0]), "Done");
        assert_eq!(status(&cockpit, ids[1]), "Cancelled");
        assert_eq!(status(&cockpit, ids[2]), "Cancelled");
        assert!(cockpit.follow_ups(day()).iter().all(|f| !f.planned));

        let tom = cockpit.add_with(Sheet::Contacts, &[("name", "Tom"), ("stage", "Contacted")]);
        let ids = cockpit.plan_sequence(tom, &steps());
        cockpit.set(Sheet::Contacts, tom, "stage", "Not now".into());
        assert!(ids.iter().all(|id| status(&cockpit, *id) == "Cancelled"));
    }

    #[test]
    fn a_contact_left_blank_goes_but_not_the_one_being_filled_in() {
        let mut cockpit = Cockpit::default();
        let blank = cockpit.add(Sheet::Contacts, day());
        let open = cockpit.add(Sheet::Contacts, day());
        let tied = cockpit.add(Sheet::Contacts, day());
        cockpit.add_with(Sheet::Actions, &[("contact", &tied.to_string())]);
        let named = cockpit.add_with(Sheet::Contacts, &[("email", "a@b.ch")]);
        assert!(cockpit.drop_blank_contacts(Some(open)));
        let left: Vec<u64> = cockpit.contacts.iter().map(|r| r.id).collect();
        assert_eq!(left, vec![open, tied, named]);
        assert!(!cockpit.drop_blank_contacts(Some(open)));
        assert!(!cockpit.contacts.iter().any(|r| r.id == blank));
    }
}
