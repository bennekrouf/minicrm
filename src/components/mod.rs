//! The window under the header: the cockpit's pages on the left, the page
//! that is open, and the editor of a selected line on the right.

mod detail;
mod followups;
mod guide;
mod sheet;
mod sidebar;
#[cfg(test)]
mod tests;

use chrono::NaiveDate;
use dioxus::prelude::*;

use crate::model::{today, Cockpit, Sheet};

use followups::FollowUps;
use guide::Guide;
use sheet::SheetView;
use sidebar::Sidebar;

/// What the main pane shows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum View {
    /// How it works: what the cockpit is for, and this week's count.
    Guide,
    /// The follow-ups due by the end of the week, from every table.
    FollowUps,
    Sheet(Sheet),
}

/// Where the person is in the window. One copy, in context, so the sidebar,
/// the guide and a table agree on it.
#[derive(Clone, Copy, PartialEq)]
pub struct Ui {
    pub view: Signal<View>,
    /// The line open in the editor.
    pub selected: Signal<Option<u64>>,
    pub search: Signal<String>,
    /// The entry of the table's drop-down column that lines are filtered by.
    pub filter: Signal<Option<&'static str>>,
    /// The column lines are sorted by, and whether ascending. None is the
    /// order they were added in.
    pub sort: Signal<Option<(usize, bool)>>,
    /// The table shows its archived lines instead of those in use.
    pub archived: Signal<bool>,
    /// What the last import or export did, under the page's heading.
    pub notice: Signal<Option<String>>,
    /// The day follow-ups are counted against. Read again each time a page
    /// is opened, so a window left open overnight catches up.
    pub today: Signal<NaiveDate>,
}

impl Ui {
    /// Starts on the guide. Call it from a component: the signals belong to
    /// the one that does.
    pub fn new() -> Self {
        Self {
            view: Signal::new(View::Guide),
            selected: Signal::new(None),
            search: Signal::new(String::new()),
            filter: Signal::new(None),
            sort: Signal::new(None),
            archived: Signal::new(false),
            notice: Signal::new(None),
            today: Signal::new(today()),
        }
    }

    /// Opens a page as it is first seen: nothing selected, searched, filtered
    /// or sorted, since all of those meant something in the page being left.
    pub fn go(mut self, view: View) {
        self.selected.set(None);
        self.search.set(String::new());
        self.filter.set(None);
        self.sort.set(None);
        self.archived.set(false);
        self.notice.set(None);
        self.today.set(today());
        self.view.set(view);
    }

    /// Opens a line of any table in its editor.
    pub fn open(mut self, sheet: Sheet, id: u64) {
        self.go(View::Sheet(sheet));
        self.selected.set(Some(id));
    }
}

/// Everything under the header. Reads the cockpit and [`Ui`] from context.
#[component]
pub fn Workspace() -> Element {
    let ui = use_context::<Ui>();
    let mut cockpit = use_context::<Signal<Cockpit>>();
    // A contact added and left before anything was typed in it goes once
    // another line, or none, is open: it would only fill the counts.
    use_effect(move || {
        let keep = (ui.selected)();
        let blank = {
            let c = cockpit.peek();
            c.contacts
                .iter()
                .any(|r| Some(r.id) != keep && c.is_blank_contact(r))
        };
        if blank {
            cockpit.write().drop_blank_contacts(keep);
        }
    });
    rsx! {
        div { class: "body",
            Sidebar {}
            match (ui.view)() {
                View::Guide => rsx! { Guide {} },
                View::FollowUps => rsx! { FollowUps {} },
                View::Sheet(sheet) => rsx! { SheetView { sheet } },
            }
        }
    }
}
