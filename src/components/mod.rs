//! The window under the header: the cockpit's pages on the left, the page
//! that is open, and the editor of a selected line on the right.

mod detail;
mod guide;
mod sheet;
mod sidebar;
#[cfg(test)]
mod tests;

use dioxus::prelude::*;

use crate::model::Sheet;

use guide::Guide;
use sheet::SheetView;
use sidebar::Sidebar;

/// What the main pane shows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum View {
    /// How it works: what the cockpit is for, and this week's count.
    Guide,
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
        }
    }

    /// Opens a page as it is first seen: nothing selected, searched, filtered
    /// or sorted, since all of those meant something in the page being left.
    pub fn go(mut self, view: View) {
        self.selected.set(None);
        self.search.set(String::new());
        self.filter.set(None);
        self.sort.set(None);
        self.view.set(view);
    }
}

/// Everything under the header. Reads the cockpit and [`Ui`] from context.
#[component]
pub fn Workspace() -> Element {
    let ui = use_context::<Ui>();
    rsx! {
        div { class: "body",
            Sidebar {}
            match (ui.view)() {
                View::Guide => rsx! { Guide {} },
                View::Sheet(sheet) => rsx! { SheetView { sheet } },
            }
        }
    }
}
